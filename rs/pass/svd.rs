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
    if x < Fx::ZERO { Fx::ZERO - x } else { x }
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
///
/// `dot(block[i], block[i])` is loop-invariant — column `i` is final once
/// step `i` completes (later steps only modify columns `j > i`) — so each
/// denominator is computed once. On real graphs (n = 65k, k = 256) the
/// recomputation tripled the orthonormalization cost.
pub fn orthonormalize(block: &mut [Vec<Fx>]) {
    let k = block.len();
    let mut denoms = vec![Fx::ZERO; k];
    for j in 0..k {
        for i in 0..j {
            if denoms[i].is_zero() {
                continue;
            }
            let coeff = dot(&block[i], &block[j]).div(denoms[i]);
            for x in 0..block[j].len() {
                block[j][x] = block[j][x] - coeff * block[i][x];
            }
        }
        abs_max_normalize(&mut block[j]);
        denoms[j] = dot(&block[j], &block[j]);
    }
}

/// Deterministic, linearly independent start block of `k` vectors of length `n`.
fn start_block(n: usize, k: usize) -> Vec<Vec<Fx>> {
    (0..k)
        .map(|c| {
            let mut v: Vec<Fx> = (0..n)
                .map(|i| Fx::from_int(((i * (2 * c + 3) + c) % 13 + 1) as i64))
                .collect();
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
pub fn top_svd(
    n: usize,
    apply_m: &(dyn Fn(&[Fx]) -> Vec<Fx> + Sync),
    apply_mt: &(dyn Fn(&[Fx]) -> Vec<Fx> + Sync),
    k: usize,
    iters: usize,
) -> Svd {
    let k = k.min(n);
    if k == 0 {
        return Svd {
            u: vec![],
            v: vec![],
            sigma: vec![],
        };
    }
    let mtm = |x: &[Fx]| -> Vec<Fx> { apply_mt(&apply_m(x)) };

    // Right singular vectors: eigenvectors of MᵀM by subspace iteration.
    let mut block = start_block(n, k);
    orthonormalize(&mut block);
    let _dbg = std::env::var_os("TRU_SVD_DEBUG").is_some();
    let dbg_t0 = std::time::Instant::now();
    let nthreads = std::thread::available_parallelism()
        .map(|v| v.get())
        .unwrap_or(1)
        .min(k);
    for it in 0..iters {
        if _dbg && it % 10 == 0 {
            eprintln!("[svd] iter {it}/{iters} k={k} n={n} {:?}", dbg_t0.elapsed());
        }
        // columns are independent through the operator; orthonormalize
        // (below) stays serial. at multi-million n this is the
        // compile-time governor (deflation correction included).
        if nthreads > 1 {
            // columns are independent through the operator; the
            // orthonormalize passes (below) stay serial. at
            // multi-million n this loop is the compile-time governor.
            let results: Vec<Vec<Vec<Fx>>> = std::thread::scope(|s| {
                let mut handles = Vec::new();
                for t in 0..nthreads {
                    let cols: Vec<usize> = (t..k).step_by(nthreads).collect();
                    let block = &block;
                    handles.push(s.spawn(move || {
                        cols.iter().map(|&c| mtm(&block[c])).collect::<Vec<_>>()
                    }));
                }
                handles.into_iter().map(|h| h.join().unwrap()).collect()
            });
            for (t, res) in results.iter().enumerate() {
                for (r, v) in res.iter().enumerate() {
                    block[t + r * nthreads].copy_from_slice(v);
                }
            }
        } else {
            for col in block.iter_mut() {
                *col = mtm(col);
            }
        }
        // Fixed-point Gram-Schmidt loses orthogonality when the spectrum is
        // steep (mixed 1+2-hop operators: sigma2/sigma1 ~ 0.1) — junk
        // directions then keep large Rayleigh values and the reconstruction
        // gains spurious components (observed: phantom sigma 1.125 on a
        // rank-3 matrix, rel err 0.86). Reorthogonalizing twice per iteration
        // is the standard cheap cure (loss of orthogonality is a one-step
        // phenomenon; two passes restore it).
        orthonormalize(&mut block);
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
    triples.sort_by_key(|t| core::cmp::Reverse(t.0));

    let mut svd = Svd {
        u: Vec::with_capacity(k),
        v: Vec::with_capacity(k),
        sigma: Vec::with_capacity(k),
    };
    for (s, u, v) in triples {
        svd.sigma.push(s);
        svd.u.push(u);
        svd.v.push(v);
    }
    svd
}

/// Banded SVD with exact field deflation — the fixed-point answer to the
/// tail collapse. A wide block over a near-degenerate tail loses
/// orthogonality faster than double Gram-Schmidt can restore it (the
/// pussy operator resolves only ~14 of 64 components before the rest
/// collapse to exact zero). Instead, compute the spectrum in bands:
/// each band is a well-separated HEAD problem where subspace iteration
/// converges cleanly, then deflate the operator by the exact field
/// rank-1 updates sigma_i * u_i v_i^T (subtraction in a finite field
/// carries no rounding beyond one ulp, and the correction is applied
/// inside every matvec). The caller's `k` is covered in ceil(k/band)
/// bands; per-band vectors are reorthogonalized against the previously
/// deflated ones so leakage cannot re-enter.
pub fn top_svd_banded(
    n: usize,
    apply_m: &(dyn Fn(&[Fx]) -> Vec<Fx> + Sync),
    apply_mt: &(dyn Fn(&[Fx]) -> Vec<Fx> + Sync),
    k: usize,
    band: usize,
    iters: usize,
) -> Svd {
    let k = k.min(n);
    let band = band.max(1).min(k);
    let mut done = Svd {
        u: Vec::with_capacity(k),
        v: Vec::with_capacity(k),
        sigma: Vec::with_capacity(k),
    };
    let mut remaining = k;
    while remaining > 0 {
        let this_band = band.min(remaining);
        let prev = done.u.len();
        // deflated actions: subtract the already-extracted triples
        let apply_d = |x: &[Fx]| -> Vec<Fx> {
            let mut y = apply_m(x);
            for i in 0..prev {
                let c = dot(&done.v[i], x) * done.sigma[i];
                if !c.is_zero() {
                    for j in 0..n {
                        y[j] = y[j] - c * done.u[i][j];
                    }
                }
            }
            y
        };
        let apply_dt = |x: &[Fx]| -> Vec<Fx> {
            let mut y = apply_mt(x);
            for i in 0..prev {
                let c = dot(&done.u[i], x) * done.sigma[i];
                if !c.is_zero() {
                    for j in 0..n {
                        y[j] = y[j] - c * done.v[i][j];
                    }
                }
            }
            y
        };
        let apply_d = &apply_d as &(dyn Fn(&[Fx]) -> Vec<Fx> + Sync);
        let apply_dt = &apply_dt as &(dyn Fn(&[Fx]) -> Vec<Fx> + Sync);
        let mut part = top_svd(n, apply_d, apply_dt, this_band, iters);
        // block reorthogonalization of the new vectors against the
        // deflated ones (inexact deflation leakage re-enters otherwise)
        for j in 0..part.u.len() {
            for i in 0..prev {
                let du = dot(&done.u[i], &part.u[j]);
                let dv = dot(&done.v[i], &part.v[j]);
                for t in 0..n {
                    let uj = part.u[j][t] - du * done.u[i][t];
                    let vj = part.v[j][t] - dv * done.v[i][t];
                    part.u[j][t] = uj;
                    part.v[j][t] = vj;
                }
            }
        }
        done.sigma.extend(part.sigma.iter().copied());
        done.u.extend(part.u);
        done.v.extend(part.v);
        remaining -= this_band;
    }
    // restore descending order across band boundaries (bands are
    // locally sorted; concatenation of sorted bands is nearly sorted)
    let mut order: Vec<usize> = (0..done.sigma.len()).collect();
    order.sort_by_key(|&i| core::cmp::Reverse(done.sigma[i]));
    let sigma = order.iter().map(|&i| done.sigma[i]).collect();
    let u = order.iter().map(|&i| done.u[i].clone()).collect();
    let v = order.iter().map(|&i| done.v[i].clone()).collect();
    Svd { u, v, sigma }
}

/// Convenience: SVD of a dense square matrix `p` (row-major `n×n`).
pub fn dense_svd(p: &[Vec<Fx>], k: usize, iters: usize) -> Svd {
    let n = p.len();
    let apply_m = |x: &[Fx]| -> Vec<Fx> { (0..n).map(|i| dot(&p[i], x)).collect() };
    let apply_mt = |x: &[Fx]| -> Vec<Fx> {
        let mut out = vec![Fx::ZERO; n];
        for i in 0..n {
            for j in 0..n {
                out[j] = out[j] + p[i][j] * x[i];
            }
        }
        out
    };
    let apply_m = &apply_m as &(dyn Fn(&[Fx]) -> Vec<Fx> + Sync);
    let apply_mt = &apply_mt as &(dyn Fn(&[Fx]) -> Vec<Fx> + Sync);
    top_svd(n, apply_m, apply_mt, k, iters)
}

#[cfg(test)]
mod banded_tests {
    use super::*;

    /// diagonal operator with a steep, near-degenerate spectrum: the
    /// regime where a wide block collapses (measured on pussy: 14 of
    /// 64 components survive). banded deflation must recover all of it.
    #[test]
    fn banded_recovers_a_degenerate_tail() {
        let n = 64;
        let mut sig = vec![Fx::ZERO; n];
        // spread over 5 orders + near-degenerate pairs and a cluster
        let truth: Vec<f64> = [
            100.0, 10.0, 9.5, 1.0, 0.5, 0.48, 0.1, 0.095, 0.01, 0.0099, 0.0098, 0.001,
            0.0009, 0.0005, 0.0002, 0.0001,
        ]
        .to_vec();
        for (i, &s) in truth.iter().enumerate() {
            sig[i] = Fx::from_ratio((s * 1e6) as i64, 1_000_000);
        }
        let apply = |x: &[Fx]| -> Vec<Fx> { (0..n).map(|i| sig[i] * x[i]).collect() };
        let apply = &apply as &(dyn Fn(&[Fx]) -> Vec<Fx> + Sync);
        let svd = top_svd_banded(n, apply, apply, 16, 8, 40);
        for (i, &t) in truth.iter().enumerate() {
            let got = svd.sigma[i].to_f64();
            assert!(
                (got - t).abs() / t < 3e-2,
                "sigma[{i}]: got {got}, want {t}"
            );
        }
        // the banded call must not collapse: every component alive
        assert!(svd.sigma.iter().take(16).all(|s| *s > Fx::ZERO));
    }
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
        assert!(
            svd.sigma[0] >= svd.sigma[1] && svd.sigma[1] >= svd.sigma[2],
            "σ must descend"
        );
        assert!(
            (svd.sigma[0].to_f64() - 5.0).abs() < 0.1,
            "top σ ≈ 5, got {}",
            svd.sigma[0].to_f64()
        );
    }
}
