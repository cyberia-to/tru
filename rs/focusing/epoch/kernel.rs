//! The epoch tri-kernel: forward φ* and its adjoint, integer-only, parallel
//! with bit-identical results for every thread count.
//!
//! One step maps the simplex to itself (up to rounding, corrected by the
//! normalisation):
//!
//! ```text
//! D(φ) = α·π·Σφ + (1−α)·(P φ + π·Σ_p s_p φ_p)            P = A·diag(1/N), N = out + w0, s = w0/N
//! S(φ) = (μ·π·Σφ + W φ) / (1+μ)                           W = A_sym·diag(1/deg)
//! H(φ) = e^{−τ} Σ_{k=0}^{K} τ^k/k! · W^k φ                (Horner, K SpMV)
//! φ'   = norm(λ_d·D + λ_s·S + λ_h·H)
//! ```
//!
//! `π = deg/Σdeg` is the stake prior (teleport and springs reference). Each
//! operator is column-stochastic, so the blend is an affine contraction on the
//! simplex with the a-priori rate `κ = λ_d(1−α) + λ_s/(1+μ) + λ_h` — a pure
//! function of the parameters, no spectral estimate needed.
//!
//! Parallelism: rows are split into fixed chunks; every output entry is a
//! function of its row alone and every global quantity is an exact integer
//! sum, so the chunking (and thread count) cannot change a single bit.

use super::EpochParams;
use super::fixed::{LN_BITS, P_BITS, PHI_BITS, PHI_ONE, Q_BITS, ln_q40, ln2_q40, shr_i, shr_u};
use super::graph::EpochGraph;

/// The parameters as Q32 integer coefficients.
pub(crate) struct Coeffs {
    pub alpha: u64,
    pub oma: u64,
    pub lam_d: u64,
    pub lam_s: u64,
    /// `λ_h · e^{−τ}`.
    pub lam_he: u64,
    /// `μ/(1+μ)`.
    pub mu1: u64,
    /// `1/(1+μ)`.
    pub mu2: u64,
    /// `τ/k` for `k = 1..=K` (index k−1).
    pub tau_k: Vec<u64>,
    /// `τ^k/k!` for `k = 1..=K` (index k−1) — the heat-gradient weights.
    pub c_k: Vec<u64>,
}

impl Coeffs {
    pub fn new(p: &EpochParams) -> Self {
        use super::fixed::q32;
        use crate::arithmetic::Fx;
        let k = p.heat_terms.max(1);
        let one_plus_mu = Fx::ONE + p.mu;
        let mut c_k = Vec::with_capacity(k);
        let mut term = Fx::ONE;
        for i in 1..=k {
            term = term * p.tau.div(Fx::from_int(i as i64));
            c_k.push(q32(term));
        }
        Self {
            alpha: q32(p.alpha),
            oma: q32(Fx::ONE - p.alpha),
            lam_d: q32(p.lambda_d),
            lam_s: q32(p.lambda_s),
            lam_he: q32(p.lambda_h * (Fx::ZERO - p.tau).exp()),
            mu1: q32(p.mu.div(one_plus_mu)),
            mu2: q32(Fx::ONE.div(one_plus_mu)),
            tau_k: (1..=k)
                .map(|i| q32(p.tau.div(Fx::from_int(i as i64))))
                .collect(),
            c_k,
        }
    }
}

/// Row chunk boundaries balancing `rows + K·sym_nnz + in_nnz` per thread.
pub(crate) fn plan(g: &EpochGraph, threads: usize, k: usize) -> Vec<usize> {
    let n = g.n();
    let t = threads.max(1).min(n.max(1));
    let cost = |i: usize| -> u64 {
        i as u64 + k as u64 * g.sym_ptr[i] as u64 + g.in_ptr[i] as u64 + g.out_ptr[i] as u64
    };
    let total = cost(n);
    let mut b = vec![0usize];
    for c in 1..t {
        let target = total * c as u64 / t as u64;
        // first row whose prefix cost reaches the target
        let (mut lo, mut hi) = (*b.last().unwrap(), n);
        while lo < hi {
            let mid = (lo + hi) / 2;
            if cost(mid) < target {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        b.push(lo);
    }
    b.push(n);
    b
}

/// Fill `out[i] = f(i)` over the chunk plan, returning `Σ add(f(i))`. Runs on
/// the calling thread when the plan has one chunk.
pub(crate) fn par_fill<T, A, F>(bounds: &[usize], out: &mut [T], f: F) -> A
where
    T: Send,
    A: Send + Default + std::ops::Add<Output = A>,
    F: Fn(usize, &mut T) -> A + Sync,
{
    if bounds.len() <= 2 {
        let mut acc = A::default();
        for (i, slot) in out.iter_mut().enumerate() {
            acc = acc + f(i, slot);
        }
        return acc;
    }
    let mut chunks: Vec<(usize, &mut [T])> = Vec::with_capacity(bounds.len() - 1);
    let mut rest = out;
    for w in bounds.windows(2) {
        let (head, tail) = rest.split_at_mut(w[1] - w[0]);
        chunks.push((w[0], head));
        rest = tail;
    }
    let f = &f;
    std::thread::scope(|s| {
        let hs: Vec<_> = chunks
            .into_iter()
            .map(|(start, chunk)| {
                s.spawn(move || {
                    let mut acc = A::default();
                    for (j, slot) in chunk.iter_mut().enumerate() {
                        acc = acc + f(start + j, slot);
                    }
                    acc
                })
            })
            .collect();
        hs.into_iter()
            .fold(A::default(), |a, h| a + h.join().expect("kernel worker"))
    })
}

/// The teleported mass `Σ_p s_p φ_p` (ground share plus dangling rows).
pub(crate) fn teleported(g: &EpochGraph, phi: &[u64]) -> u128 {
    let s: u128 = g
        .tele_share
        .iter()
        .zip(phi)
        .map(|(&t, &x)| t as u128 * x as u128)
        .sum();
    shr_u(s, P_BITS)
}

/// `f_i = (Pᵀλ)_i + s_i·c_t` — the value of particle i's out-row under λ:
/// its out-axons' λ and, for the teleported share, the prior's `c_t = πᵀλ`.
#[inline(always)]
pub(crate) fn row_value(g: &EpochGraph, i: usize, lam: &[i64], ct: i128) -> i64 {
    dir_adj_row(g, i, lam) + shr_i(g.tele_share[i] as i128 * ct, P_BITS) as i64
}

/// `(W x)_i` at the scale of `x` — forward random-walk row `i`.
#[inline(always)]
fn sym_fwd_row(g: &EpochGraph, i: usize, x: &[u64]) -> u64 {
    let mut acc = 0u128;
    for k in g.sym_ptr[i] as usize..g.sym_ptr[i + 1] as usize {
        acc += g.sym_fwd[k] as u128 * x[g.sym_col[k] as usize] as u128;
    }
    shr_u(acc, P_BITS) as u64
}

/// `(Wᵀ y)_i` for signed `y`.
#[inline(always)]
fn sym_adj_row(g: &EpochGraph, i: usize, y: &[i64]) -> i64 {
    let mut acc = 0i128;
    for k in g.sym_ptr[i] as usize..g.sym_ptr[i + 1] as usize {
        acc += g.sym_adj[k] as i128 * y[g.sym_col[k] as usize] as i128;
    }
    shr_i(acc, P_BITS) as i64
}

/// `(P x)_i` — in-edges of `i`.
#[inline(always)]
fn dir_fwd_row(g: &EpochGraph, i: usize, x: &[u64]) -> u64 {
    let mut acc = 0u128;
    for k in g.in_ptr[i] as usize..g.in_ptr[i + 1] as usize {
        acc += g.p_in[k] as u128 * x[g.in_src[k] as usize] as u128;
    }
    shr_u(acc, P_BITS) as u64
}

/// `(Pᵀ y)_i` — out-edges of `i`.
#[inline(always)]
pub(crate) fn dir_adj_row(g: &EpochGraph, i: usize, y: &[i64]) -> i64 {
    let mut acc = 0i128;
    for e in g.out_ptr[i] as usize..g.out_ptr[i + 1] as usize {
        acc += g.p_out[e] as i128 * y[g.out_dst[e] as usize] as i128;
    }
    shr_i(acc, P_BITS) as i64
}

#[inline(always)]
fn qmul(a: u64, q: u64) -> u64 {
    shr_u(a as u128 * q as u128, Q_BITS) as u64
}

#[inline(always)]
fn qmul_i(a: i64, q: u64) -> i64 {
    shr_i(a as i128 * q as i128, Q_BITS) as i64
}

/// Working buffers for a solve, reused across steps.
pub(crate) struct Work<T> {
    pub s0: Vec<T>,
    pub ga: Vec<T>,
    pub gb: Vec<T>,
    pub next: Vec<T>,
}

impl<T: Clone + Default> Work<T> {
    pub fn new(n: usize) -> Self {
        Self {
            s0: vec![T::default(); n],
            ga: vec![T::default(); n],
            gb: vec![T::default(); n],
            next: vec![T::default(); n],
        }
    }
}

/// One forward step `φ → norm(R φ)`; returns the ℓ1 drift `Σ|φ' − φ|`.
pub(crate) fn forward_step(
    g: &EpochGraph,
    c: &Coeffs,
    b: &[usize],
    phi: &mut Vec<u64>,
    w: &mut Work<u64>,
) -> u128 {
    let k = c.tau_k.len();
    let mass: u128 = phi.iter().map(|&x| x as u128).sum();
    let dm = teleported(g, phi);
    let c_d = shr_u(c.alpha as u128 * mass + c.oma as u128 * dm, Q_BITS);
    let c_s = shr_u(c.mu1 as u128 * mass, Q_BITS);

    // s0 = W φ, and the innermost Horner term g_K = φ + (τ/K)·W φ.
    {
        let phi_r: &[u64] = phi;
        sym_fwd(g, b, phi_r, &mut w.s0);
        let (tk, s0) = (c.tau_k[k - 1], &w.s0);
        par_fill(b, &mut w.ga, |i, slot| {
            *slot = phi_r[i] + qmul(s0[i], tk);
            0u128
        });
    }
    // g_j = φ + (τ/j)·W g_{j+1}, j = K−1 .. 2.
    for j in (2..k).rev() {
        let tj = c.tau_k[j - 1];
        let (phi_r, ga) = (&*phi, &w.ga);
        par_fill(b, &mut w.gb, |i, slot| {
            *slot = phi_r[i] + qmul(sym_fwd_row(g, i, ga), tj);
            0u128
        });
        std::mem::swap(&mut w.ga, &mut w.gb);
    }
    // Final: H = e^{−τ}(φ + τ·W g_2), D, S, blend.
    let (phi_r, ga, s0) = (&*phi, &w.ga, &w.s0);
    let total: u128 = par_fill(b, &mut w.next, |i, slot| {
        let h1 = if k == 1 {
            ga[i]
        } else {
            phi_r[i] + qmul(sym_fwd_row(g, i, ga), c.tau_k[0])
        };
        let t = g.tele[i] as u128;
        let d = shr_u(t * c_d, PHI_BITS) as u64 + qmul(dir_fwd_row(g, i, phi_r), c.oma);
        let s = shr_u(t * c_s, PHI_BITS) as u64 + qmul(s0[i], c.mu2);
        let v = shr_u(
            c.lam_d as u128 * d as u128
                + c.lam_s as u128 * s as u128
                + c.lam_he as u128 * h1 as u128,
            Q_BITS,
        ) as u64;
        *slot = v;
        v as u128
    });
    // Normalise onto Σφ = 2^60 and measure the drift.
    let f = (1u128 << (PHI_BITS + Q_BITS))
        .checked_div(total)
        .unwrap_or(0);
    let next = &w.next;
    let drift: u128 = par_fill(b, phi, |i, slot| {
        let v = shr_u(next[i] as u128 * f, Q_BITS) as u64;
        let d = v.abs_diff(*slot) as u128;
        *slot = v;
        d
    });
    drift
}

/// One adjoint step `λ → Mᵀλ + g` (the transpose of the linear step on the
/// simplex; `g` is the centred `∇_φ J`).
pub(crate) fn adjoint_step(
    g: &EpochGraph,
    c: &Coeffs,
    b: &[usize],
    lam: &mut Vec<i64>,
    grad: &[i64],
    w: &mut Work<i64>,
) {
    let k = c.tau_k.len();
    let ct: i128 = shr_i(
        g.tele
            .iter()
            .zip(lam.iter())
            .map(|(&t, &l)| t as i128 * l as i128)
            .sum(),
        PHI_BITS,
    );
    {
        let lr: &[i64] = lam;
        sym_adj(g, b, lr, &mut w.s0);
        let (tk, s0) = (c.tau_k[k - 1], &w.s0);
        par_fill(b, &mut w.ga, |i, slot| {
            *slot = lr[i] + qmul_i(s0[i], tk);
            0i128
        });
    }
    for j in (2..k).rev() {
        let tj = c.tau_k[j - 1];
        let (lr, ga) = (&*lam, &w.ga);
        par_fill(b, &mut w.gb, |i, slot| {
            *slot = lr[i] + qmul_i(sym_adj_row(g, i, ga), tj);
            0i128
        });
        std::mem::swap(&mut w.ga, &mut w.gb);
    }
    let c_t = ct as i64;
    let tele_d = shr_i(c.alpha as i128 * ct, Q_BITS) as i64; // α·c_t
    let tele_s = qmul_i(c_t, c.mu1);
    let (lr, ga, s0) = (&*lam, &w.ga, &w.s0);
    par_fill(b, &mut w.next, |i, slot| {
        let h1 = if k == 1 {
            ga[i]
        } else {
            lr[i] + qmul_i(sym_adj_row(g, i, ga), c.tau_k[0])
        };
        let d = qmul_i(row_value(g, i, lr, ct), c.oma) + tele_d;
        let s = tele_s + qmul_i(s0[i], c.mu2);
        *slot = shr_i(
            c.lam_d as i128 * d as i128
                + c.lam_s as i128 * s as i128
                + c.lam_he as i128 * h1 as i128,
            Q_BITS,
        ) as i64
            + grad[i];
        0i128
    });
    std::mem::swap(lam, &mut w.next);
}

/// `W x` (forward random walk), parallel.
pub(crate) fn sym_fwd(g: &EpochGraph, b: &[usize], x: &[u64], out: &mut [u64]) {
    par_fill(b, out, |i, slot| {
        *slot = sym_fwd_row(g, i, x);
        0u128
    });
}

/// `Wᵀ y` (adjoint random walk), parallel.
pub(crate) fn sym_adj(g: &EpochGraph, b: &[usize], y: &[i64], out: &mut [i64]) {
    par_fill(b, out, |i, slot| {
        *slot = sym_adj_row(g, i, y);
        0i128
    });
}

/// Syntropy `J(φ) = Σ φ_i ln(n φ_i)` at scale 2^60 (nats), for `φ` at 2^60
/// over `n` particles.
pub(crate) fn syntropy_with_n(phi: &[u64], n: u64, b: &[usize]) -> i128 {
    // ln(n·x) with x = φ/2^60: ln φ − 60·ln 2 + ln n.
    let off = ln_q40(n) as i128 - PHI_BITS as i128 * ln2_q40() as i128;
    let mut scratch = vec![0u8; phi.len()];
    let s: i128 = par_fill(b, &mut scratch, |i, _| {
        let x = phi[i];
        if x == 0 {
            0i128
        } else {
            x as i128 * (ln_q40(x) as i128 + off)
        }
    });
    shr_i(s, LN_BITS)
}

/// The centred gradient `g_i = ln φ_i − Σ_j φ_j ln φ_j / Σφ` at 2^40 (zero
/// where `φ_i = 0`). Constants drop out: the step operator's perturbations
/// carry zero total mass.
pub(crate) fn centred_log(phi: &[u64], b: &[usize]) -> Vec<i64> {
    let mut g = vec![0i64; phi.len()];
    let s: i128 = par_fill(b, &mut g, |i, slot| {
        let x = phi[i];
        *slot = if x == 0 { 0 } else { ln_q40(x) };
        x as i128 * *slot as i128
    });
    let mass: u128 = phi.iter().map(|&x| x as u128).sum();
    let mean = if mass == 0 {
        0
    } else {
        (s / mass as i128) as i64
    };
    for (gi, &x) in g.iter_mut().zip(phi) {
        if x > 0 {
            *gi -= mean;
        }
    }
    g
}

/// The uniform start `⌊2^60/n⌋` per particle — every solve is cold.
pub(crate) fn uniform(n: usize) -> Vec<u64> {
    vec![if n == 0 { 0 } else { PHI_ONE / n as u64 }; n]
}
