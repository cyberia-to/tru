//! Spectral estimates for the contraction κ and the step count T(ε).
//!
//! The composite operator contracts at rate
//! `κ = λ_d·α + λ_s·‖L‖/(‖L‖+μ) + λ_h·e^{−τλ₂}` ([[tri-kernel]] §2.2), where
//! `‖L‖ = λ_max` and `λ₂` is the algebraic connectivity (Fiedler value) of the
//! weighted Laplacian `L = D − A_sym`. Both come from power iteration on L.
//! The iteration then runs a fixed `T(ε) = min t : κ^t ≤ ε` — the smallest
//! count that provably reaches ε, computed by iterating κ (no logarithm).

use crate::arithmetic::Fx;

use super::csr::CsrMatrix;
use super::focusing::FocusingParams;

#[inline]
fn fabs(x: Fx) -> Fx {
    if x < Fx::ZERO {
        Fx::ZERO - x
    } else {
        x
    }
}

fn dot(a: &[Fx], b: &[Fx]) -> Fx {
    let mut s = Fx::ZERO;
    for i in 0..a.len() {
        s = s + a[i] * b[i];
    }
    s
}

/// Scale so the largest-magnitude entry is 1 (sign-safe; keeps φ bounded).
fn abs_max_normalize(v: &mut [Fx]) {
    let mut m = Fx::ZERO;
    for &x in v.iter() {
        let a = fabs(x);
        if a > m {
            m = a;
        }
    }
    if !m.is_zero() {
        for x in v.iter_mut() {
            *x = x.div(m);
        }
    }
}

/// Remove the component along the constant vector `1` (project onto `1^⊥`).
fn deflate_mean(v: &mut [Fx]) {
    let n = v.len();
    let mut sum = Fx::ZERO;
    for &x in v.iter() {
        sum = sum + x;
    }
    let mean = sum.div(Fx::from_int(n as i64));
    for x in v.iter_mut() {
        *x = *x - mean;
    }
}

/// `out = L·v = D·v − A_sym·v`.
fn l_matvec(sym: &CsrMatrix, degree: &[Fx], v: &[Fx], out: &mut [Fx]) {
    sym.spmv(v, out);
    for i in 0..v.len() {
        out[i] = degree[i] * v[i] - out[i];
    }
}

/// A deterministic, non-constant start vector (constant is L's null space).
fn start(n: usize) -> Vec<Fx> {
    let mut v: Vec<Fx> = (0..n).map(|i| Fx::from_int((i % 7 + 1) as i64)).collect();
    abs_max_normalize(&mut v);
    v
}

/// Largest Laplacian eigenvalue ‖L‖ = λ_max, by power iteration.
pub fn lambda_max(sym: &CsrMatrix, degree: &[Fx], n: usize, iters: usize) -> Fx {
    if n == 0 {
        return Fx::ZERO;
    }
    let mut v = start(n);
    let mut lv = vec![Fx::ZERO; n];
    let mut lam = Fx::ZERO;
    for _ in 0..iters {
        l_matvec(sym, degree, &v, &mut lv);
        lam = dot(&v, &lv).div(dot(&v, &v)); // Rayleigh quotient
        v.copy_from_slice(&lv);
        abs_max_normalize(&mut v);
    }
    lam
}

/// Algebraic connectivity λ₂ (Fiedler value): the smallest nonzero Laplacian
/// eigenvalue. Power-iterate `M = λ_max·I − L` on `1^⊥`; its dominant
/// eigenvalue there is `λ_max − λ₂`.
pub fn lambda_2(sym: &CsrMatrix, degree: &[Fx], n: usize, lambda_max: Fx, iters: usize) -> Fx {
    if n < 2 {
        return Fx::ZERO;
    }
    let mut v = start(n);
    deflate_mean(&mut v);
    abs_max_normalize(&mut v);
    let mut mv = vec![Fx::ZERO; n];
    let mut mu = Fx::ZERO;
    for _ in 0..iters {
        l_matvec(sym, degree, &v, &mut mv);
        for i in 0..n {
            mv[i] = lambda_max * v[i] - mv[i]; // (λ_max·I − L)·v
        }
        deflate_mean(&mut mv);
        mu = dot(&v, &mv).div(dot(&v, &v));
        v.copy_from_slice(&mv);
        abs_max_normalize(&mut v);
    }
    let l2 = lambda_max - mu;
    if l2 < Fx::ZERO {
        Fx::ZERO
    } else {
        l2
    }
}

/// The composite contraction coefficient κ ([[tri-kernel]] §2.2).
pub fn kappa(p: &FocusingParams, lambda_max: Fx, lambda_2: Fx) -> Fx {
    let heat = (Fx::ZERO - p.tau * lambda_2).exp(); // e^{−τλ₂}
    let springs = lambda_max.div(lambda_max + p.mu); // ‖L‖/(‖L‖+μ)
    p.lambda_d * p.alpha + p.lambda_s * springs + p.lambda_h * heat
}

/// The smallest T with `κ^T ≤ ε`, capped. If κ ≥ 1 (no contraction) returns
/// the cap. Computed by iterating κ — no logarithm needed.
pub fn steps_for(kappa: Fx, epsilon: Fx, cap: usize) -> usize {
    if kappa >= Fx::ONE {
        return cap;
    }
    let mut p = Fx::ONE;
    let mut t = 0;
    while p > epsilon && t < cap {
        p = p * kappa;
        t += 1;
    }
    t
}
