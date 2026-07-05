//! Pass 5 — attention weights (`specs/ct0.md` §7).
//!
//! Each dialect becomes a head. For every layer `l` and dialect `s`, the
//! dialect adjacency `A^(s)` is raised to the layer power `l_eff`, projected
//! into embedding space `P = Eᵀ A^(s,l) E`, and factorized: the query/key
//! projections are its truncated SVD, the value projection reads the raw dialect
//! adjacency, and the output projection is the pseudoinverse of the stacked
//! values. Wedge scalars `(α,β)=(1,0)` initialize the geometric-attention score.
//! All fixed-point; the per-head SVD reuses the shared `svd` spine.

use crate::arithmetic::Fx;
use crate::model::{Encoding, Tensor};

use super::dialect::Dialects;
use super::index::Edge;
use super::svd::dense_svd;

/// Sparse positive-stake adjacency of one dialect (edges assigned to it),
/// Fx-normalized by the global max weight.
struct DialectAdj {
    n: usize,
    out: Vec<Vec<(u32, Fx)>>,
}

impl DialectAdj {
    fn build(edges: &[Edge], assign: &[usize], head: usize, n: usize, maxw: u128) -> Self {
        let mut acc: Vec<std::collections::HashMap<u32, i128>> =
            vec![std::collections::HashMap::new(); n];
        for (k, e) in edges.iter().enumerate() {
            if assign[k] == head && e.stake > 0 {
                *acc[e.src as usize].entry(e.tgt).or_insert(0) += e.stake;
            }
        }
        let out = acc
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|(j, w)| (j, Fx::ratio_u128(w as u128, maxw)))
                    .collect()
            })
            .collect();
        DialectAdj { n, out }
    }

    /// `(A·x)_i = Σ_j A[i][j] x_j`.
    fn apply(&self, x: &[Fx]) -> Vec<Fx> {
        let mut out = vec![Fx::ZERO; self.n];
        for i in 0..self.n {
            let mut s = Fx::ZERO;
            for &(j, w) in &self.out[i] {
                s = s + w * x[j as usize];
            }
            out[i] = s;
        }
        out
    }

    /// `A^p · x` by repeated application.
    fn apply_pow(&self, x: &[Fx], p: usize) -> Vec<Fx> {
        let mut v = x.to_vec();
        for _ in 0..p {
            v = self.apply(&v);
        }
        v
    }
}

/// A dense `n×d` view of the embedding tensor (row-major).
struct Embed<'a> {
    data: &'a [Fx],
    n: usize,
    d: usize,
}

impl Embed<'_> {
    fn col(&self, c: usize) -> Vec<Fx> {
        (0..self.n).map(|i| self.data[i * self.d + c]).collect()
    }
    /// `Eᵀ y` — project an `n`-vector into the `d`-dim embedding space.
    fn t_apply(&self, y: &[Fx]) -> Vec<Fx> {
        (0..self.d)
            .map(|c| (0..self.n).fold(Fx::ZERO, |a, i| a + self.data[i * self.d + c] * y[i]))
            .collect()
    }
}

/// `P^(s,l) = Eᵀ A^(s,l) E`, a `d×d` dense matrix.
fn project(a: &DialectAdj, e: &Embed, l_eff: usize) -> Vec<Vec<Fx>> {
    // Column c of A^(s,l) E = A^(s)^{l_eff} · E[:,c]; then project by Eᵀ.
    let cols: Vec<Vec<Fx>> = (0..e.d)
        .map(|c| e.t_apply(&a.apply_pow(&e.col(c), l_eff)))
        .collect();
    // cols[c] is column c of P; transpose to row-major P[i][j].
    (0..e.d)
        .map(|i| (0..e.d).map(|j| cols[j][i]).collect())
        .collect()
}

/// The Moore–Penrose pseudoinverse of a `d×d` matrix via SVD:
/// `P⁺ = Σ (1/σ) v uᵀ` over σ above a small threshold (§7.5). `rank_cap` bounds
/// the SVD block (the value projection has rank ≤ |V|).
fn pinv(p: &[Vec<Fx>], rank_cap: usize, iters: usize) -> Vec<Vec<Fx>> {
    let d = p.len();
    let svd = dense_svd(p, d.min(rank_cap).max(1), iters);
    let thresh = Fx::from_ratio(1, 100_000);
    let mut out = vec![vec![Fx::ZERO; d]; d];
    for c in 0..svd.sigma.len() {
        if svd.sigma[c] > thresh {
            let inv = Fx::ONE.div(svd.sigma[c]);
            for i in 0..d {
                for j in 0..d {
                    out[i][j] = out[i][j] + inv * svd.v[c][i] * svd.u[c][j];
                }
            }
        }
    }
    out
}

/// Flatten a `d×d` row-major dense matrix to a tensor payload.
fn dense_tensor(name: String, m: &[Vec<Fx>]) -> Tensor {
    let d = m.len();
    let mut data = Vec::with_capacity(d * d);
    for row in m {
        data.extend_from_slice(row);
    }
    Tensor {
        name,
        shape: vec![d as u64, d as u64],
        encoding: Encoding::U16,
        data,
    }
}

/// Pass 5: the attention tensors for every layer. `e_data` is the row-major
/// embedding (pass 4); `phi` is φ*; `d`/`h`/`l`/`diam` come from pass 3.
#[allow(clippy::too_many_arguments)]
pub fn attention(
    edges: &[Edge],
    dialects: &Dialects,
    e_data: &[Fx],
    phi: &[Fx],
    d: usize,
    h: usize,
    l: usize,
    diam: usize,
) -> Vec<Tensor> {
    let n = phi.len();
    let e = Embed { data: e_data, n, d };
    let d_h = d.checked_div(h).unwrap_or(d);
    let maxw = edges
        .iter()
        .filter(|e| e.stake > 0)
        .map(|e| e.stake as u128)
        .max()
        .unwrap_or(1)
        .max(1);

    // One dialect adjacency per head (index into dialects.set).
    let dialect_adj: Vec<DialectAdj> = (0..h)
        .map(|head| DialectAdj::build(edges, &dialects.assign, head, n, maxw))
        .collect();

    let mut tensors = Vec::new();
    for layer in 0..l {
        let l_eff = 1 + (layer * diam) / l.max(1);

        // Per-head Q/K/V blocks, concatenated along the column axis into d×d.
        let mut q = vec![vec![Fx::ZERO; d]; d];
        let mut k = vec![vec![Fx::ZERO; d]; d];
        let mut v = vec![vec![Fx::ZERO; d]; d];

        for (head, a) in dialect_adj.iter().enumerate() {
            let base = head * d_h;
            // W_Q, W_K from the SVD of the layer projection P^(s,l). The
            // projection has rank ≤ |V|, so cap the SVD block there.
            let p = project(a, &e, l_eff);
            let svd = dense_svd(&p, d_h.min(n).max(1), 60);
            for c in 0..d_h {
                let ss = if c < svd.sigma.len() {
                    svd.sigma[c].sqrt()
                } else {
                    Fx::ZERO
                };
                for i in 0..d {
                    if c < svd.u.len() {
                        q[i][base + c] = svd.u[c][i] * ss;
                        k[i][base + c] = svd.v[c][i] * ss;
                    }
                }
            }
            // W_V = Eᵀ diag(φ) A^(s) E[:, head slice].
            for c in 0..d_h {
                let ecol = e.col(base + c);
                let ae = a.apply(&ecol);
                let phi_ae: Vec<Fx> = (0..n).map(|i| phi[i] * ae[i]).collect();
                let wv = e.t_apply(&phi_ae); // length d
                for i in 0..d {
                    v[i][base + c] = wv[i];
                }
            }
        }

        // W_O = pseudoinverse of the stacked value projection.
        let o = pinv(&v, n, 60);

        tensors.push(dense_tensor(
            format!("model.layers.{layer}.self_attn.q_proj.weight"),
            &q,
        ));
        tensors.push(dense_tensor(
            format!("model.layers.{layer}.self_attn.k_proj.weight"),
            &k,
        ));
        tensors.push(dense_tensor(
            format!("model.layers.{layer}.self_attn.v_proj.weight"),
            &v,
        ));
        tensors.push(dense_tensor(
            format!("model.layers.{layer}.self_attn.o_proj.weight"),
            &o,
        ));
        // Wedge score scalars (α,β) = (1,0) — dot-product attention at init (§7.7).
        tensors.push(Tensor {
            name: format!("model.layers.{layer}.self_attn.alpha_beta.weight"),
            shape: vec![2],
            encoding: Encoding::U16,
            data: vec![Fx::ONE, Fx::ZERO],
        });
    }
    tensors
}

#[cfg(test)]
mod tests {
    use super::super::{arch, dialect, embed, index::build};
    use super::*;
    use crate::graph::Cyberlink;

    fn hash(b: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    fn link(from: u8, to: u8) -> Cyberlink {
        Cyberlink {
            neuron: hash(from),
            from: hash(from),
            to: hash(to),
            token: 0,
            amount: 100,
            valence: 1,
            block: 0,
        }
    }

    fn setup() -> (Vec<Tensor>, usize) {
        let mut links = Vec::new();
        for (a, b) in [(1u8, 2u8), (2, 3), (3, 4), (4, 1), (1, 3)] {
            links.push(link(a, b));
            links.push(link(b, a));
        }
        let (_v, edges, adj) = build(&[], &links);
        let dialects = dialect::discover(&_v, &edges);
        let a = arch::compute(&adj, dialects.len(), 0);
        let d = 16.min(a.d.max(4)); // keep the test small
        let e = embed::embed(&adj, &a.phi, d);
        let t = attention(&edges, &dialects, &e.data, &a.phi, d, a.h, a.l, a.diameter);
        (t, a.l)
    }

    #[test]
    fn emits_five_tensors_per_layer_with_right_shapes() {
        let (t, l) = setup();
        assert_eq!(t.len(), 5 * l, "q,k,v,o,alpha_beta per layer");
        let q0 = t
            .iter()
            .find(|t| t.name == "model.layers.0.self_attn.q_proj.weight")
            .unwrap();
        assert_eq!(q0.shape.len(), 2);
        assert_eq!(q0.data.len() as u64, q0.shape[0] * q0.shape[1]);
        let ab = t
            .iter()
            .find(|t| t.name == "model.layers.0.self_attn.alpha_beta.weight")
            .unwrap();
        assert_eq!(ab.shape, vec![2]);
        assert_eq!(ab.data[0].raw(), Fx::ONE.raw());
        assert_eq!(ab.data[1].raw(), Fx::ZERO.raw());
    }

    #[test]
    fn attention_is_deterministic() {
        let (a, _) = setup();
        let (b, _) = setup();
        assert!(a.iter().zip(&b).all(|(x, y)| x
            .data
            .iter()
            .zip(&y.data)
            .all(|(p, q)| p.raw() == q.raw())));
    }

    fn pearson(x: &[f64], y: &[f64]) -> f64 {
        let n = x.len() as f64;
        let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
        let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
        for i in 0..x.len() {
            let (dx, dy) = (x[i] - mx, y[i] - my);
            sxy += dx * dy;
            sxx += dx * dx;
            syy += dy * dy;
        }
        if sxx == 0.0 || syy == 0.0 {
            0.0
        } else {
            sxy / (sxx.sqrt() * syy.sqrt())
        }
    }

    #[test]
    fn p_attn_qk_reconstructs_the_projection() {
        // P-ATTN (§11.2): Pearson(flatten(W_Q·W_Kᵀ), flatten(P^(s,l))) ≥ 0.7.
        // W_Q·W_Kᵀ = (U√Σ)(V√Σ)ᵀ = UΣVᵀ is the rank-d_h truncation of P, so on the
        // emitted head-0 layer-0 tensors it must track the recomputed projection.
        let mut links = Vec::new();
        for (a, b) in [(1u8, 2u8), (2, 3), (3, 4), (4, 1), (1, 3)] {
            links.push(link(a, b));
            links.push(link(b, a));
        }
        let (v, edges, adj) = build(&[], &links);
        let dialects = dialect::discover(&v, &edges);
        let ar = arch::compute(&adj, dialects.len(), 0);
        let d = 16.min(ar.d.max(4));
        let n = ar.phi.len();
        let e_t = embed::embed(&adj, &ar.phi, d);
        let tensors = attention(
            &edges,
            &dialects,
            &e_t.data,
            &ar.phi,
            d,
            ar.h,
            ar.l,
            ar.diameter,
        );

        // Recompute P^(head 0, layer 0) — l_eff = 1 at layer 0.
        let e = Embed {
            data: &e_t.data,
            n,
            d,
        };
        let maxw = edges
            .iter()
            .filter(|x| x.stake > 0)
            .map(|x| x.stake as u128)
            .max()
            .unwrap_or(1)
            .max(1);
        let a0 = DialectAdj::build(&edges, &dialects.assign, 0, n, maxw);
        let p = project(&a0, &e, 1);

        let q = &tensors
            .iter()
            .find(|t| t.name == "model.layers.0.self_attn.q_proj.weight")
            .unwrap()
            .data;
        let k = &tensors
            .iter()
            .find(|t| t.name == "model.layers.0.self_attn.k_proj.weight")
            .unwrap()
            .data;
        let (mut xs, mut ys) = (Vec::new(), Vec::new());
        for i in 0..d {
            for j in 0..d {
                let mut s = 0.0;
                for c in 0..d {
                    s += q[i * d + c].to_f64() * k[j * d + c].to_f64();
                }
                xs.push(s);
                ys.push(p[i][j].to_f64());
            }
        }
        let r = pearson(&xs, &ys);
        assert!(r >= 0.7, "P-ATTN Pearson {r} < 0.7");
    }
}
