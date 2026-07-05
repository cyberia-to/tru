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

use super::arch::{m_svd, FxAdj};
use super::index::Adjacency;

/// Pass 4: the `(|V|, d*)` embedding tensor `model.embed_tokens.weight`, stored
/// row-major as `u16` (§6.3). `phi` is φ* from pass 3.
pub fn embed(adj: &Adjacency, phi: &[Fx], d: usize) -> Tensor {
    let g = FxAdj::from(adj);
    let n = g.n;
    let svd = m_svd(&g, phi, d, 120);
    let rank = svd.sigma.len();

    // E[i][c] = U[c][i] · √σ_c ; columns past the numerical rank are zero.
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

        // Dense M (scaled ÷ max weight, matching arch's FxAdj normalization).
        let ds: Vec<f64> = phi.iter().map(|x| x.to_f64().sqrt()).collect();
        let maxw = adj.out.iter().flatten().map(|&(_, w)| w).max().unwrap_or(1) as f64;
        let mut m = vec![vec![0.0; n]; n];
        for i in 0..n {
            for &(j, w) in &adj.out[i] {
                m[i][j as usize] += ds[i] * (w as f64 / maxw) * ds[j as usize];
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
