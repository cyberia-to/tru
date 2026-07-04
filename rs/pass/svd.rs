//! Fixed-point truncated SVD by subspace iteration (`specs/ct0.md` §5.2, §6).
//!
//! The compile's numerical spine. Given a square operator `M` (supplied as its
//! matvec `M·x` and transpose `Mᵀ·x`), extract the top-`k` singular triples
//! `(uᵢ, σᵢ, vᵢ)` such that `M ≈ Σ σᵢ uᵢ vᵢᵀ`. Subspace-iterate `MᵀM` for the
//! right vectors `V` (eigenvectors, eigenvalue σ²), recover `U = M·V/σ`, and
//! fix signs by convention SC-1 (largest-magnitude entry of each `u` positive).
//!
//! Deterministic: a fixed start block, a fixed iteration count, all fixed-point
//! over the Goldilocks field. This is the exact object the spec's ChaCha-seeded
//! randomized SVD accelerates on large graphs; here it is computed directly.

use crate::arithmetic::Fx;

/// A truncated SVD: columns of `u`/`v` are the left/right singular vectors,
/// `sigma` the singular values, all sorted descending by `sigma`.
pub struct Svd {
    pub u: Vec<Vec<Fx>>,
    pub v: Vec<Vec<Fx>>,
    pub sigma: Vec<Fx>,
}

pub fn dot(a: &[Fx], b: &[Fx]) -> Fx {
    let mut s = Fx::ZERO;
    for i in 0..a.len() {
        s = s + a[i] * b[i];
    }
    s
}

fn fabs(x: Fx) -> Fx {
    if x < Fx::ZERO {
        Fx::ZERO - x
    } else {
        x
    }
}

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

/// Modified Gram–Schmidt (sqrt-free: projection via `dot(u,v)/dot(u,u)`).
pub fn orthonormalize(block: &mut [Vec<Fx>]) {
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

/// Deterministic, linearly independent start block of `k` vectors of length `n`.
fn start_block(n: usize, k: usize) -> Vec<Vec<Fx>> {
    (0..k)
        .map(|c| {
            let mut v: Vec<Fx> = (0..n).map(|i| Fx::from_int(((i * (2 * c + 3) + c) % 13 + 1) as i64)).collect();
            abs_max_normalize(&mut v);
            v
        })
        .collect()
}

/// L2-normalize `v` (unit length), returning it unchanged if it is ~zero.
fn l2_normalize(v: &mut [Fx]) {
    let norm2 = dot(v, v);
    if norm2 > Fx::ZERO {
        let inv = Fx::ONE.div(norm2.sqrt());
        for x in v.iter_mut() {
            *x = *x * inv;
        }
    }
}

/// Sign convention SC-1: flip `(u, v)` so `u`'s largest-magnitude entry is
/// positive (leaves `σ u vᵀ` invariant).
fn sc1(u: &mut [Fx], v: &mut [Fx]) {
    let mut peak = Fx::ZERO;
    let mut sign_neg = false;
    for &x in u.iter() {
        let a = fabs(x);
        if a > peak {
            peak = a;
            sign_neg = x < Fx::ZERO;
        }
    }
    if sign_neg {
        for x in u.iter_mut() {
            *x = Fx::ZERO - *x;
        }
        for x in v.iter_mut() {
            *x = Fx::ZERO - *x;
        }
    }
}

/// Top-`k` SVD of a square operator on `n` dims, given `M·x` and `Mᵀ·x`.
pub fn top_svd(n: usize, apply_m: &dyn Fn(&[Fx]) -> Vec<Fx>, apply_mt: &dyn Fn(&[Fx]) -> Vec<Fx>, k: usize, iters: usize) -> Svd {
    let k = k.min(n);
    if k == 0 {
        return Svd { u: vec![], v: vec![], sigma: vec![] };
    }
    let mtm = |x: &[Fx]| -> Vec<Fx> { apply_mt(&apply_m(x)) };

    // Right singular vectors: eigenvectors of MᵀM by subspace iteration.
    let mut block = start_block(n, k);
    orthonormalize(&mut block);
    for _ in 0..iters {
        for col in block.iter_mut() {
            *col = mtm(col);
        }
        orthonormalize(&mut block);
    }

    // Assemble triples: σ = √(Rayleigh(MᵀM)), u = M v / σ, normalized, SC-1.
    let mut triples: Vec<(Fx, Vec<Fx>, Vec<Fx>)> = block
        .into_iter()
        .map(|mut v| {
            l2_normalize(&mut v);
            let mvv = mtm(&v);
            let eig = dot(&v, &mvv);
            let sigma = if eig < Fx::ZERO { Fx::ZERO } else { eig.sqrt() };
            let mut u = apply_m(&v);
            l2_normalize(&mut u);
            sc1(&mut u, &mut v);
            (sigma, u, v)
        })
        .collect();
    triples.sort_by(|a, b| b.0.cmp(&a.0));

    let mut svd = Svd { u: Vec::with_capacity(k), v: Vec::with_capacity(k), sigma: Vec::with_capacity(k) };
    for (s, u, v) in triples {
        svd.sigma.push(s);
        svd.u.push(u);
        svd.v.push(v);
    }
    svd
}

/// Convenience: SVD of a dense square matrix `p` (row-major `n×n`).
pub fn dense_svd(p: &[Vec<Fx>], k: usize, iters: usize) -> Svd {
    let n = p.len();
    let apply_m = |x: &[Fx]| -> Vec<Fx> {
        (0..n).map(|i| dot(&p[i], x)).collect()
    };
    let apply_mt = |x: &[Fx]| -> Vec<Fx> {
        let mut out = vec![Fx::ZERO; n];
        for i in 0..n {
            for j in 0..n {
                out[j] = out[j] + p[i][j] * x[i];
            }
        }
        out
    };
    top_svd(n, &apply_m, &apply_mt, k, iters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconstructs_a_small_symmetric_matrix() {
        // A rank-2 symmetric 3×3: reconstruction Σσ u vᵀ ≈ P.
        let p = vec![
            vec![Fx::from_int(2), Fx::from_int(1), Fx::ZERO],
            vec![Fx::from_int(1), Fx::from_int(2), Fx::ZERO],
            vec![Fx::ZERO, Fx::ZERO, Fx::from_int(3)],
        ];
        let svd = dense_svd(&p, 3, 200);
        // Reconstruct and compare Frobenius error.
        let n = 3;
        let mut err = 0.0;
        let mut mag = 0.0;
        for i in 0..n {
            for j in 0..n {
                let mut r = 0.0;
                for c in 0..svd.sigma.len() {
                    r += svd.sigma[c].to_f64() * svd.u[c][i].to_f64() * svd.v[c][j].to_f64();
                }
                err += (r - p[i][j].to_f64()).powi(2);
                mag += p[i][j].to_f64().powi(2);
            }
        }
        let rel = (err / mag).sqrt();
        assert!(rel < 0.05, "SVD reconstruction error {rel} too large");
    }

    #[test]
    fn singular_values_descend() {
        let p = vec![
            vec![Fx::from_int(5), Fx::ZERO, Fx::ZERO],
            vec![Fx::ZERO, Fx::from_int(3), Fx::ZERO],
            vec![Fx::ZERO, Fx::ZERO, Fx::from_int(1)],
        ];
        let svd = dense_svd(&p, 3, 100);
        assert!(svd.sigma[0] >= svd.sigma[1] && svd.sigma[1] >= svd.sigma[2], "σ must descend");
        assert!((svd.sigma[0].to_f64() - 5.0).abs() < 0.1, "top σ ≈ 5, got {}", svd.sigma[0].to_f64());
    }
}
