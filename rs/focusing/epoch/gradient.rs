//! The syntropy gradient `g_pq = ∂J(φ*(A))/∂A_pq` for every present axon.
//!
//! At the fixed point `φ* = M(A)·φ*` (M column-stochastic on the simplex) the
//! envelope argument gives `dJ = λᵀ (∂M/∂A) φ*` with the adjoint `λ` solving
//! `λ = Mᵀλ + ∇_φJ` (rewards.md §2, chain-economy §5.1). `∇_φJ = ln φ + const`
//! and the constant drops out (`∂M/∂A` moves no mass), so the iteration runs on
//! the centred log, from `λ = 0`, for the same fixed step count as φ*.
//!
//! Per directed edge `p → q` (degree `d`, out-strength `out`, prior
//! `π = d/Σd`, `c_t = πᵀλ`, `u_j = (Wᵀ)ʲλ`, `v_i = Wⁱφ`):
//!
//! ```text
//! diffusion   (1−α)·φ_p (λ_q − (Pᵀλ)_p)/out_p  +  (α + (1−α)·φ_dangling)·(λ_p + λ_q − 2c_t)/Σd
//! springs     [φ_p (λ_q − u₁_p)/d_p + φ_q (λ_p − u₁_q)/d_q]/(1+μ)  +  μ/(1+μ)·(λ_p + λ_q − 2c_t)/Σd
//! heat        e^{−τ} Σ_{k=1}^{K} τ^k/k! Σ_{j<k} [v_{k−1−j,p}(u_{j,q} − u_{j+1,p})/d_p + (p↔q)]
//! g_pq        λ_d·diffusion + λ_s·springs + λ_h·heat
//! ```
//!
//! Each term is the derivative of a column-normalised operator with respect
//! to one weight: `∂(xᵀ W y)/∂W_pq = y_p (x_q − (Wᵀx)_p)/d_p`. The dangling
//! set is piecewise constant and contributes no derivative.
//!
//! Scale: `g` is nats per raw weight unit at `2^112` (i128). The chain's
//! attribution `a(c) = Δs(c)·σ(c)·g_pq` is then a 256-bit product
//! (chain-economy §5.2); [`super::fixed::mul_wide_shr`] brings it to nats·2^60.

use super::fixed::{GRAD_BITS, LN_BITS, PHI_BITS, Q_BITS, mul_shift_div, shr_i};
use super::graph::EpochGraph;
use super::kernel::{Coeffs, dir_adj_row, sym_adj, sym_fwd};

/// Per-edge gradient in internal edge order, `φ` and `λ` at the fixed point.
pub(crate) fn edge_gradient(
    g: &EpochGraph,
    c: &Coeffs,
    b: &[usize],
    phi: &[u64],
    lam: &[i64],
) -> Vec<i128> {
    let n = g.n();
    let kk = c.tau_k.len();
    // Heat chain: v_i = Wⁱφ (i < K), u_j = (Wᵀ)ʲλ (j ≤ K).
    let mut v: Vec<Vec<u64>> = Vec::with_capacity(kk);
    v.push(phi.to_vec());
    for i in 1..kk {
        let mut nx = vec![0u64; n];
        sym_fwd(g, b, &v[i - 1], &mut nx);
        v.push(nx);
    }
    let mut u: Vec<Vec<i64>> = Vec::with_capacity(kk + 1);
    u.push(lam.to_vec());
    for j in 1..=kk {
        let mut nx = vec![0i64; n];
        sym_adj(g, b, &u[j - 1], &mut nx);
        u.push(nx);
    }

    // Scalars of the prior's derivative.
    let mass: u128 = phi.iter().map(|&x| x as u128).sum();
    let dm: u128 = g.dangling.iter().map(|&i| phi[i as usize] as u128).sum();
    let ct: i128 = shr_i(
        g.tele
            .iter()
            .zip(lam)
            .map(|(&t, &l)| t as i128 * l as i128)
            .sum(),
        PHI_BITS,
    );
    let dm_q = (dm << Q_BITS).checked_div(mass).unwrap_or(0) as i128;
    let c_dq = c.alpha as i128 + shr_i(c.oma as i128 * dm_q, Q_BITS);
    let q = |a: i128, b: i128| shr_i(a * b, Q_BITS);
    let k_d = q(c.lam_d as i128, c.oma as i128);
    let k_t = q(c.lam_d as i128, c_dq) + q(c.lam_s as i128, c.mu1 as i128);
    let k_s = q(c.lam_s as i128, c.mu2 as i128);
    let k_h = c.lam_he as i128;
    let total = g.total.min(u64::MAX as u128) as u64;
    let s_bits = GRAD_BITS - PHI_BITS;

    let heat = |a: usize, bb: usize| -> i128 {
        let mut y = 0i128;
        for k in 1..=kk {
            let mut acc = 0i128;
            for j in 0..k {
                acc += v[k - 1 - j][a] as i128 * (u[j][bb] as i128 - u[j + 1][a] as i128);
            }
            y += shr_i(c.c_k[k - 1] as i128 * shr_i(acc, LN_BITS), Q_BITS);
        }
        y
    };
    let div = |num: i128, den: u64| {
        if den == 0 {
            0
        } else {
            mul_shift_div(num, s_bits, den)
        }
    };
    let row = |s: usize, out: &mut [i128]| {
        let pd = dir_adj_row(g, s, lam) as i128;
        let (fs, ls, u1s) = (phi[s] as i128, lam[s] as i128, u[1][s] as i128);
        for (k, e) in (g.out_ptr[s] as usize..g.out_ptr[s + 1] as usize).enumerate() {
            let d = g.out_dst[e] as usize;
            let (fd, ld, u1d) = (phi[d] as i128, lam[d] as i128, u[1][d] as i128);
            let n_d = shr_i(fs * (ld - pd), LN_BITS);
            let n_t = (ls + ld - 2 * ct) << (PHI_BITS - LN_BITS);
            let n_ss = shr_i(fs * (ld - u1s), LN_BITS);
            let n_sd = shr_i(fd * (ls - u1d), LN_BITS);
            let gsum = div(q(k_d, n_d), g.out_w[s])
                .saturating_add(div(q(k_t, n_t), total))
                .saturating_add(div(q(k_s, n_ss) + q(k_h, heat(s, d)), g.deg[s]))
                .saturating_add(div(q(k_s, n_sd) + q(k_h, heat(d, s)), g.deg[d]));
            out[k] = if g.shift == 0 {
                gsum
            } else {
                shr_i(gsum, g.shift)
            };
        }
    };

    let m = g.directed_nnz();
    let mut grad = vec![0i128; m];
    if b.len() <= 2 {
        for s in 0..n {
            let (lo, hi) = (g.out_ptr[s] as usize, g.out_ptr[s + 1] as usize);
            row(s, &mut grad[lo..hi]);
        }
        return grad;
    }
    let mut chunks: Vec<(usize, usize, &mut [i128])> = Vec::new();
    let mut rest: &mut [i128] = &mut grad;
    for w in b.windows(2) {
        let len = (g.out_ptr[w[1]] - g.out_ptr[w[0]]) as usize;
        let (head, tail) = rest.split_at_mut(len);
        chunks.push((w[0], w[1], head));
        rest = tail;
    }
    let row = &row;
    std::thread::scope(|sc| {
        for (lo, hi, out) in chunks {
            sc.spawn(move || {
                let base = g.out_ptr[lo] as usize;
                for s in lo..hi {
                    let (a, z) = (
                        g.out_ptr[s] as usize - base,
                        g.out_ptr[s + 1] as usize - base,
                    );
                    row(s, &mut out[a..z]);
                }
            });
        }
    });
    grad
}
