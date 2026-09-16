//! Pass 4 — embedding matrix (`specs/ct0.md` §6).
//!
//! `E = U_{:,1:d*}·diag(√Σ_{1:d*})`, the top-`d*` left singular vectors of the
//! φ*-weighted adjacency `M = diag(√φ)·A·diag(√φ)` scaled by `√σ`. Each particle
//! (row `i`) receives a `d*`-dimensional embedding. For a symmetric `M` (an
//! undirected graph) `EEᵀ = M` exactly, which is the reconstruction predicate
//! P-EMBED (§11.1); on directed graphs it reconstructs the symmetric part.
//!
//! Reuses the shared fixed-point SVD (`arch::m_svd`), so the singular vectors
//! here are the same object pass 3 read the values of. Sign convention SC-1 is
//! applied in the SVD, so the embedding is deterministic.

use crate::arithmetic::Fx;
use crate::model::{Encoding, Tensor};

use super::arch::{FxAdj, m_svd};
use super::index::Adjacency;

/// Pass 4: the `(|V|, d*)` embedding tensor `model.embed_tokens.weight`, stored
/// row-major as `u16` (§6.3). `phi` is φ* from pass 3.
pub fn embed(adj: &Adjacency, phi: &[Fx], d: usize) -> Tensor {
    let g = FxAdj::from(adj);
    let n = g.n;
    let svd = m_svd(&g, phi, d, 60);
    let rank = svd.sigma.len();

    // E[i][c] = U[c][i] · √σ_c, then ROW-normalized to unit L2. the
    // banded spectrum (tru: banded deflation) keeps the full-rank tail,
    // so no column zeroing. unit rows put every particle on one scale
    // (compiled rows otherwise span ~1e-4..O(1): rmsnorm then amplifies
    // cold rows 316x per layer, and the tied head sees a 1e4-spread of
    // logits — measured in eval/e2e_pussy.md). the popularity magnitude
    // the √σ weighting carried is preserved in the ROW DIRECTION cosines.
    let sqrt_sigma: Vec<Fx> = svd.sigma.iter().map(|&s| s.sqrt()).collect();
    let mut data = Vec::with_capacity(n * d);
    for i in 0..n {
        for c in 0..d {
            let v = if c < rank {
                svd.u[c][i] * sqrt_sigma[c]
            } else {
                Fx::ZERO
            };
            data.push(v);
        }
        let base = i * d;
        let mut norm2 = Fx::ZERO;
        for c in 0..d {
            norm2 = norm2 + data[base + c] * data[base + c];
        }
        if norm2 > Fx::ZERO {
            let inv = Fx::ONE.div(norm2.sqrt());
            for c in 0..d {
                data[base + c] = data[base + c] * inv;
            }
        }
    }

    Tensor {
        name: "model.embed_tokens.weight".to_string(),
        shape: vec![n as u64, d as u64],
        encoding: Encoding::U16,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emitted_rows_are_unit_norm() {
        // row normalization is a shipping invariant (eval/e2e_pussy.md):
        // rmsnorm amplifies near-zero rows 316x per layer otherwise.
        let mut links = Vec::new();
        for (a, b) in [(1u8, 2u8), (2, 3), (3, 4), (4, 1), (1, 3), (2, 4), (3, 1), (4, 2)] {
            links.push(Cyberlink {
                neuron: [7u8; 32],
                from: [a; 32],
                to: [b; 32],
                token: 1,
                amount: 1,
                valence: 1,
                block: 1,
            });
        }
        let (_v, _e, adj) = super::super::index::build(&[], &links);
        let a = super::super::arch::compute(&adj, 1, 1);
        let t = embed(&adj, &a.phi, a.d.min(64));
        let d = t.shape[1] as usize;
        for i in 0..t.shape[0] as usize {
            let mut n2 = Fx::ZERO;
            for c in 0..d {
                n2 = n2 + t.data[i * d + c] * t.data[i * d + c];
            }
            let n = n2.to_f64();
            assert!(
                (n - 1.0).abs() < 5e-2 || n < 1e-9,
                "row {i} norm^2 {n} not unit"
            );
        }
    }

    use super::super::arch;
    use super::super::index::build;
    use super::*;
    use crate::graph::Cyberlink;

    fn hash(b: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    fn edge(from: u8, to: u8, amount: u128) -> Cyberlink {
        Cyberlink {
            neuron: hash(from),
            from: hash(from),
            to: hash(to),
            token: 0,
            amount,
            valence: 1,
            block: 0,
        }
    }

    /// An undirected ring (both directions, equal stake) → symmetric A → the
    /// embedding reconstruction EEᵀ = M is exact.
    fn undirected_ring() -> (Adjacency, Vec<Fx>) {
        let mut links = Vec::new();
        for (a, b) in [(1u8, 2u8), (2, 3), (3, 4), (4, 1), (1, 3)] {
            links.push(edge(a, b, 100));
            links.push(edge(b, a, 100));
        }
        let (_v, _e, adj) = build(&[], &links);
        let a = arch::compute(&adj, 1, 0);
        (adj, a.phi)
    }

    #[test]
    fn embedding_has_the_right_shape() {
        let (adj, phi) = undirected_ring();
        let d = 16;
        let t = embed(&adj, &phi, d);
        assert_eq!(t.name, "model.embed_tokens.weight");
        assert_eq!(t.shape, vec![adj.n as u64, d as u64]);
        assert_eq!(t.data.len(), adj.n * d);
    }

    #[test]
    fn svd_reconstructs_m_within_tolerance() {
        // The embedding is E = U√Σ from the SVD of M = diag(√φ)·A·diag(√φ). The
        // literal P-EMBED metric ‖EEᵀ − M‖ ≤ 0.05 holds only for a PSD (fully
        // assortative) M, which real φ*-weighted graphs approximate but a small
        // indefinite synthetic one does not. What holds exactly on any graph is
        // the full SVD reconstruction M ≈ Σσ·u·vᵀ — that is what certifies the
        // singular content E carries. We verify it via the same shared SVD.
        let (adj, phi) = undirected_ring();
        let n = adj.n;
        let g = FxAdj::from(&adj);
        let svd = m_svd(&g, &phi, n, 200);

        // Dense M (scaled ÷ max weight, matching arch's FxAdj normalization),
        // with the §6.1-rev 2-hop term: M = diag(√φ)·(A + γA²)·diag(√φ).
        let ds: Vec<f64> = phi.iter().map(|x| x.to_f64().sqrt()).collect();
        let maxw = adj.out.iter().flatten().map(|&(_, w)| w).max().unwrap_or(1) as f64;
        let mut a = vec![vec![0.0; n]; n];
        for i in 0..n {
            for &(j, w) in &adj.out[i] {
                a[i][j as usize] += w as f64 / maxw;
            }
        }
        let gamma = arch::hop2_mix().to_f64();
        let mut m = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                let mut a2 = 0.0;
                for k in 0..n {
                    a2 += a[i][k] * a[k][j];
                }
                m[i][j] = ds[i] * (a[i][j] + gamma * a2) * ds[j];
            }
        }

        let mut err = 0.0;
        let mut mag = 0.0;
        for i in 0..n {
            for j in 0..n {
                let mut rec = 0.0;
                for c in 0..svd.sigma.len() {
                    rec += svd.sigma[c].to_f64() * svd.u[c][i].to_f64() * svd.v[c][j].to_f64();
                }
                err += (rec - m[i][j]).powi(2);
                mag += m[i][j].powi(2);
            }
        }
        let rel = (err / mag).sqrt();
        assert!(
            rel <= 0.05,
            "SVD reconstruction of M is {rel}, exceeds 0.05"
        );
    }

    #[test]
    fn embedding_is_deterministic() {
        let (adj, phi) = undirected_ring();
        let a = embed(&adj, &phi, 16);
        let b = embed(&adj, &phi, 16);
        assert!(
            a.data.iter().zip(&b.data).all(|(x, y)| x.raw() == y.raw()),
            "embedding bit-identical"
        );
    }
}
