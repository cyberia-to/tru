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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::csr::CsrBuilder;

    fn fx(n: i64) -> Fx {
        Fx::from_int(n)
    }

    /// The 4-cycle 0-1-2-3-0, unit edge weights both directions: a graph whose
    /// Laplacian eigenvalues are exactly known (2 − 2cos(2πk/4) for k=0..3),
    /// i.e. {0, 2, 2, 4} — so `lambda_max` and `lambda_2` have closed-form
    /// answers to check the power iteration against. `spectral.rs` (this
    /// file) has no test coverage on origin/master today even though rows 1
    /// and 2 ("φ* exists, unique, converges with κ < 1", "Σφ*ᵢ = 1") and
    /// every open reward row (3, 24, 26, 27, 28) all read `kappa`/`steps_for`
    /// through it.
    fn cycle4() -> (CsrMatrix, Vec<Fx>) {
        let mut b = CsrBuilder::new(4);
        for i in 0..4usize {
            let j = (i + 1) % 4;
            b.add(i, j, fx(1));
            b.add(j, i, fx(1));
        }
        (b.build(), vec![fx(2); 4]) // every node has degree 2
    }

    #[test]
    fn lambda_max_matches_the_known_cycle4_spectrum() {
        let (sym, degree) = cycle4();
        let lm = lambda_max(&sym, &degree, 4, 60);
        assert!(
            (lm.to_f64() - 4.0).abs() < 1e-3,
            "C4's largest Laplacian eigenvalue is 4, got {}",
            lm.to_f64()
        );
    }

    #[test]
    fn lambda_2_matches_the_known_cycle4_spectrum() {
        let (sym, degree) = cycle4();
        let lm = lambda_max(&sym, &degree, 4, 60);
        let l2 = lambda_2(&sym, &degree, 4, lm, 60);
        assert!(
            (l2.to_f64() - 2.0).abs() < 1e-3,
            "C4's algebraic connectivity is 2, got {}",
            l2.to_f64()
        );
    }

    #[test]
    fn lambda_2_is_zero_below_two_nodes() {
        let sym = CsrBuilder::new(1).build();
        assert_eq!(lambda_2(&sym, &[fx(0)], 1, fx(0), 10).raw(), Fx::ZERO.raw());
    }

    #[test]
    fn lambda_max_is_zero_on_the_empty_graph() {
        let sym = CsrBuilder::new(0).build();
        assert_eq!(lambda_max(&sym, &[], 0, 10).raw(), Fx::ZERO.raw());
    }

    #[test]
    fn kappa_matches_its_closed_form() {
        let p = FocusingParams::default();
        let lm = fx(4);
        let l2 = fx(2);
        let k = kappa(&p, lm, l2);
        let heat = (Fx::ZERO - p.tau * l2).exp();
        let springs = lm.div(lm + p.mu);
        let expect = p.lambda_d * p.alpha + p.lambda_s * springs + p.lambda_h * heat;
        assert_eq!(k.raw(), expect.raw());
    }

    #[test]
    fn steps_for_is_monotonic_in_epsilon_and_respects_the_cap() {
        let kappa = Fx::from_ratio(1, 2);
        let loose = steps_for(kappa, Fx::from_ratio(1, 10), 100);
        let tight = steps_for(kappa, Fx::from_ratio(1, 10_000), 100);
        assert!(
            tight > loose,
            "a smaller ε must never need fewer steps ({tight} vs {loose})"
        );
        // κ^t ≤ ε actually holds at the returned t (unless capped).
        let mut p = Fx::ONE;
        for _ in 0..tight {
            p = p * kappa;
        }
        assert!(p.to_f64() <= Fx::from_ratio(1, 10_000).to_f64() + 1e-9);
    }

    #[test]
    fn steps_for_returns_the_cap_when_kappa_does_not_contract() {
        assert_eq!(steps_for(Fx::ONE, Fx::from_ratio(1, 10), 37), 37);
        assert_eq!(steps_for(fx(2), Fx::from_ratio(1, 10), 37), 37);
    }
}
