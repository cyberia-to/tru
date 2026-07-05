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

/// The k leading eigenvectors of `M = λ_max·I − L` on `1^⊥` — equivalently the
/// bottom-k nontrivial eigenvectors of the Laplacian `L` (the Fiedler vector
/// first, then the next-lowest frequencies). These are the spectral embedding
/// [[focusing]] emits to [[mir]]: structurally similar particles land near each
/// other. Ordered most-dominant-first (ascending Laplacian eigenvalue).
///
/// Subspace (orthogonal) iteration: apply `M` to a block of `k` vectors,
/// project each off the constant null space, re-orthogonalize by modified
/// Gram–Schmidt, repeat a fixed `iters`. Given spectral gaps the columns
/// converge to the individual eigenvectors; a final Rayleigh-quotient sort
/// fixes the order. Fixed-point throughout, so the embedding is reproducible.
/// (The spec names LOBPCG as the eventual accelerator; this is the exact object
/// it must converge to.)
pub fn spectral_vectors(
    sym: &CsrMatrix,
    degree: &[Fx],
    n: usize,
    lambda_max: Fx,
    k: usize,
    iters: usize,
) -> (Vec<Vec<Fx>>, Vec<Fx>) {
    let k = k.min(n.saturating_sub(1));
    if k == 0 {
        return (vec![], vec![]);
    }

    // Deterministic, linearly independent start block: distinct stride per
    // column so modified Gram–Schmidt does not collapse them.
    let mut block: Vec<Vec<Fx>> = (0..k)
        .map(|j| {
            let mut v: Vec<Fx> = (0..n)
                .map(|i| Fx::from_int(((i * (2 * j + 3) + j) % 13 + 1) as i64))
                .collect();
            deflate_mean(&mut v);
            v
        })
        .collect();
    orthonormalize(&mut block);

    let mut mv = vec![Fx::ZERO; n];
    for _ in 0..iters {
        for col in block.iter_mut() {
            l_matvec(sym, degree, col, &mut mv);
            for i in 0..n {
                col[i] = lambda_max * col[i] - mv[i]; // (λ_max·I − L)·v
            }
            deflate_mean(col);
        }
        orthonormalize(&mut block);
    }

    // Rayleigh quotient of M per column → Laplacian eigenvalue λ = λ_max − μ.
    let mut ranked: Vec<(Fx, Vec<Fx>)> = block
        .into_iter()
        .map(|v| {
            l_matvec(sym, degree, &v, &mut mv);
            for i in 0..n {
                mv[i] = lambda_max * v[i] - mv[i];
            }
            let mu = dot(&v, &mv).div(dot(&v, &v));
            (mu, v)
        })
        .collect();
    // Most-dominant M-eigenvalue first (= smallest Laplacian eigenvalue first).
    ranked.sort_by_key(|t| core::cmp::Reverse(t.0));

    let mut vectors = Vec::with_capacity(k);
    let mut eigenvalues = Vec::with_capacity(k);
    for (mu, v) in ranked {
        let lam = lambda_max - mu;
        eigenvalues.push(if lam < Fx::ZERO { Fx::ZERO } else { lam });
        vectors.push(v);
    }
    (vectors, eigenvalues)
}

/// Modified Gram–Schmidt: make the block mutually orthogonal (inner products
/// via `dot`), each column re-scaled to bounded magnitude. No square root — the
/// projection uses `dot(u,v)/dot(u,u)`, so unit L2 norm is unnecessary.
fn orthonormalize(block: &mut [Vec<Fx>]) {
    let k = block.len();
    for j in 0..k {
        for i in 0..j {
            let denom = dot(&block[i], &block[i]);
            if denom.is_zero() {
                continue;
            }
            let coeff = dot(&block[i], &block[j]).div(denom);
            for x in 0..block[j].len() {
                block[j][x] = block[j][x] - coeff * block[i][x];
            }
        }
        abs_max_normalize(&mut block[j]);
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
