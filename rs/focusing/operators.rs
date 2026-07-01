//! The three tri-kernel operators as SINGLE-STEP maps (`specs/tri-kernel.md`).
//!
//! Each takes the current φ and returns one application — the coupled
//! iteration in [`super::compute_focusing`] applies all three to the same φ,
//! blends, normalizes, and feeds the result back. No operator solves itself to
//! its own fixed point (the non-conformant "blend of separate attractors").
//! All arithmetic is fixed-point over the Goldilocks field.

use crate::arithmetic::Fx;

use super::csr::CsrMatrix;

/// One diffusion step (personalized PageRank with stake-weighted teleport):
/// `φ' = α·u + (1−α)·(dangling_mass·u + T·φ)`, where `T[q][p] =
/// A_eff(p,q)/out_strength(p)` is column-stochastic and `u` is the teleport
/// prior. Answers: where does probability flow?
pub fn diffusion_step(phi: &[Fx], transition: &CsrMatrix, dangling: &[bool], teleport: &[Fx], alpha: Fx) -> Vec<Fx> {
    let n = phi.len();
    let one_minus_alpha = Fx::ONE - alpha;
    let mut dangling_mass = Fx::ZERO;
    for i in 0..n {
        if dangling[i] {
            dangling_mass = dangling_mass + phi[i];
        }
    }
    let mut tphi = vec![Fx::ZERO; n];
    transition.spmv(phi, &mut tphi);
    (0..n)
        .map(|i| alpha * teleport[i] + one_minus_alpha * (dangling_mass * teleport[i] + tphi[i]))
        .collect()
}

/// One springs relaxation step (Jacobi toward `(L+μI)x = μ·x₀`):
/// `x'[i] = (μ·x₀[i] + (W·φ)[i]) / (μ + d[i])`, with `W = A_sym`, `d` the
/// weighted degree, `x₀` the uniform reference. Answers: what shape satisfies
/// the elastic constraints?
pub fn springs_step(phi: &[Fx], sym_weights: &CsrMatrix, und_degree: &[Fx], mu: Fx, x0: &[Fx]) -> Vec<Fx> {
    let n = phi.len();
    let mut wphi = vec![Fx::ZERO; n];
    sym_weights.spmv(phi, &mut wphi);
    (0..n).map(|i| (mu * x0[i] + wphi[i]).div(mu + und_degree[i])).collect()
}

/// Chebyshev-truncated heat `H_τ = exp(−τL)` on the combinatorial Laplacian
/// (`specs/focusing.md`): `H_τ ≈ Σ_{k=0}^{K} c_k(τ) T_k(L̃)`, `L̃ = 2L/λ_max − I`.
///
/// The coefficients `c_k = (2−δ_{k0})(−1)^k e^{−s'} I_k(s')` use the modified
/// Bessel `I_k`. For large `s = τλ_max/2` a direct series overflows, so we use
/// the semigroup property `exp(−τL) = (exp(−(τ/m)L))^m`: split into `m`
/// sub-applications each with `s' = s/m ≤ S_SAFE`, where the Bessel series
/// converges with small, bounded terms. Everything is fixed-point over the field.
const HEAT_DEGREE: usize = 8;

pub fn heat_step(phi: &[Fx], sym_weights: &CsrMatrix, und_degree: &[Fx], lambda_max: Fx, tau: Fx) -> Vec<Fx> {
    if lambda_max.is_zero() {
        return phi.to_vec();
    }
    let s = tau * lambda_max.div(Fx::from_int(2)); // s = τ·λ_max/2
    let s_safe = Fx::from_int(2);
    let m = {
        let q = s.div(s_safe);
        let fl = q.floor_to_i64();
        (if Fx::from_int(fl) < q { fl + 1 } else { fl }).max(1) as usize
    };
    let s_prime = s.div(Fx::from_int(m as i64));
    let c = cheb_coeffs(s_prime, HEAT_DEGREE);

    let mut v = phi.to_vec();
    for _ in 0..m {
        v = cheb_apply(&v, &c, sym_weights, und_degree, lambda_max);
    }
    v
}

/// Chebyshev coefficients `c_k = (2−δ_{k0})(−1)^k e^{−s} I_k(s)` for `k ≤ degree`,
/// with `I_k(s) = Σ_m (s/2)^{2m+k}/(m!·(m+k)!)` summed to convergence.
fn cheb_coeffs(s: Fx, degree: usize) -> Vec<Fx> {
    let half = s.div(Fx::from_int(2));
    let half2 = half * half;
    let e_neg = (Fx::ZERO - s).exp();
    (0..=degree)
        .map(|k| {
            // m=0 term: half^k / k!
            let mut term = Fx::ONE;
            for _ in 0..k {
                term = term * half;
            }
            let mut kfact = Fx::ONE;
            for j in 1..=k {
                kfact = kfact * Fx::from_int(j as i64);
            }
            term = term.div(kfact);
            let mut ik = term;
            for mm in 0..12usize {
                let denom = Fx::from_int(((mm + 1) * (mm + 1 + k)) as i64);
                term = term * half2.div(denom);
                ik = ik + term;
            }
            let sign = if k % 2 == 0 { Fx::ONE } else { Fx::ZERO - Fx::ONE };
            let scale = if k == 0 { Fx::ONE } else { Fx::from_int(2) };
            scale * sign * e_neg * ik
        })
        .collect()
}

/// `Σ_k c_k T_k(L̃)·φ` by the Chebyshev three-term recurrence
/// `T_{k+1} = 2·L̃·T_k − T_{k−1}`, with `L̃·v = (2/λ_max)(D·v − A_sym·v) − v`.
fn cheb_apply(phi: &[Fx], c: &[Fx], sym_weights: &CsrMatrix, und_degree: &[Fx], lambda_max: Fx) -> Vec<Fx> {
    let n = phi.len();
    let two_over_lmax = Fx::from_int(2).div(lambda_max);
    let two = Fx::from_int(2);
    let ltilde = |v: &[Fx]| -> Vec<Fx> {
        let mut av = vec![Fx::ZERO; n];
        sym_weights.spmv(v, &mut av);
        (0..n).map(|i| two_over_lmax * (und_degree[i] * v[i] - av[i]) - v[i]).collect()
    };

    let mut t_prev = phi.to_vec(); // T_0 = φ
    let mut result: Vec<Fx> = t_prev.iter().map(|&x| c[0] * x).collect();
    if c.len() > 1 {
        let mut t_cur = ltilde(&t_prev); // T_1 = L̃·φ
        for i in 0..n {
            result[i] = result[i] + c[1] * t_cur[i];
        }
        for ck in c.iter().skip(2) {
            let lt = ltilde(&t_cur);
            let t_next: Vec<Fx> = (0..n).map(|i| two * lt[i] - t_prev[i]).collect();
            for i in 0..n {
                result[i] = result[i] + *ck * t_next[i];
            }
            t_prev = t_cur;
            t_cur = t_next;
        }
    }
    result
}

/// Normalize onto the simplex (`Σ = 1`). A zero vector is returned unchanged.
pub fn normalize_l1(v: &[Fx]) -> Vec<Fx> {
    let mut sum = Fx::ZERO;
    for &x in v {
        sum = sum + x;
    }
    if sum.is_zero() {
        return v.to_vec();
    }
    v.iter().map(|&x| x.div(sum)).collect()
}
