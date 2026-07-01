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

/// One bounded heat application: `substeps` of forward Euler on the lazy
/// random walk, `s ← s + dt·(N·s − s)` with `dt = τ/substeps` and
/// `N[i][j] = W[i][j]/d[j]` (column-stochastic). For `dt ≤ 1` each substep is
/// a positive, mass-conserving convex combination. A bounded polynomial in the
/// normalized adjacency — field-native (no matrix exponential).
///
/// (The spec also admits a Chebyshev basis in the combinatorial L for tighter
/// accuracy; the Euler-on-N form here is the M1 operator.)
pub fn heat_step(phi: &[Fx], sym_weights: &CsrMatrix, und_degree: &[Fx], tau: Fx, substeps: usize) -> Vec<Fx> {
    let n = phi.len();
    let dt = tau.div(Fx::from_int(substeps as i64));
    let mut s = phi.to_vec();
    let mut ns = vec![Fx::ZERO; n];
    for _ in 0..substeps {
        let inp: Vec<Fx> = (0..n).map(|j| if und_degree[j].is_zero() { Fx::ZERO } else { s[j].div(und_degree[j]) }).collect();
        sym_weights.spmv(&inp, &mut ns);
        for i in 0..n {
            s[i] = s[i] + dt * (ns[i] - s[i]);
        }
    }
    s
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
