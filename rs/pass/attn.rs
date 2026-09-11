//! Pass 5 — attention weights (`specs/ct0.md` §7).
//!
//! Each dialect becomes a head. For every layer `l` and dialect `s`, the
//! layer projection is the DIRECTED transition `P_dir = Eᵀ (A^(s,l))ᵀ E`:
//! the score of token `t` attending `s` measures `l_eff`-step walks `s → t`,
//! so the query's own position carries no self-transition mass and causal
//! softmax is forced to retrieve from the prefix (§7.2-rev; measured in
//! `eval/attn_eval.md`: undirected `P` self-collapses at init). `W_Q/W_K` are
//! the truncated SVD of `P_dir` (`U√Σ` / `V√Σ` — the asymmetry is what makes
//! scores directed). The value/output pair is the retrieval head: `W_V = I`,
//! `W_O = OUT_GAIN · I`, so the attention output injects the retrieved token's
//! own (normalized) embedding into the residual stream and its full geometry
//! votes in the tied head. Wedge scalars `(α,β)=(1,0)` initialize the
//! geometric-attention score. All fixed-point; the per-head SVD reuses the
//! shared `svd` spine.

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

    /// `(Aᵀ·x)_j = Σ_i A[i][j]·x_i` — transposed application, accumulating
    /// inbound edge weight at the target.
    fn apply_t(&self, x: &[Fx]) -> Vec<Fx> {
        let mut out = vec![Fx::ZERO; self.n];
        for (i, row) in self.out.iter().enumerate() {
            for &(j, w) in row {
                out[j as usize] = out[j as usize] + w * x[i];
            }
        }
        out
    }

    /// `(Aᵀ)^p · x` by repeated transposed application.
    fn apply_t_pow(&self, x: &[Fx], p: usize) -> Vec<Fx> {
        let mut v = x.to_vec();
        for _ in 0..p {
            v = self.apply_t(&v);
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

/// `P_dir^(s,l) = Eᵀ (A^(s,l))ᵀ E`, a `d×d` dense matrix (§7.2-rev). The
/// directed layer projection: `score(t, s) ≈` walks `s → t` at length
/// `l_eff`. Computed with transposed sparse application; `A^(s,l)` itself is
/// never materialized.
fn project(a: &DialectAdj, e: &Embed, l_eff: usize) -> Vec<Vec<Fx>> {
    // Column c of (A^(s,l))ᵀ E = (Aᵀ)^{l_eff} · E[:,c]; then project by Eᵀ.
    let cols: Vec<Vec<Fx>> = (0..e.d)
        .map(|c| e.t_apply(&a.apply_t_pow(&e.col(c), l_eff)))
        .collect();
    // cols[c] is column c of P; transpose to row-major P[i][j].
    (0..e.d)
        .map(|i| (0..e.d).map(|j| cols[j][i]).collect())
        .collect()
}

/// Output gain of the retrieval head (§7.5-rev, tru#3): `W_O = c·I` with
///
/// ```text
/// c = clamp(1 + log2(σ₁/σ_k), 1, 64)
/// ```
///
/// where σ₁/σ_k is the pass-4 spectrum ratio from pass 3 (`Arch::sigma_ratio`)
/// — the same quantity that sets d* measures how sharply popularity dominates
/// the geometry, and the sharper the prior, the louder the retrieved token
/// must vote in the tied-head logits to outrank it (eval/attn_eval.md: the
/// toy needed c ≳ 13 against a ~25x prior). The certificate reports the value
/// (§7.5-rev).
pub fn out_gain(sigma_ratio: Fx) -> Fx {
    if sigma_ratio <= Fx::ONE {
        return Fx::ONE;
    }
    let two = Fx::ONE + Fx::ONE;
    let log2 = sigma_ratio.ln().div(two.ln());
    let c = Fx::ONE + log2;
    let hi = Fx::from_ratio(64, 1);
    if c > hi { hi } else { c }
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
    gain: Fx,
) -> Vec<Tensor> {
    let n = phi.len();
    let _ = phi;
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
            // Retrieval head: W_V = I on this head's slice — the value is the
            // (normalized) hidden state of the retrieved position itself, so
            // its full embedding geometry reaches the residual stream. (The
            // §7.4 value `Eᵀ diag(φ) A^(s) E` applied a single graph step —
            // which inverted to a no-op block under the old pinv W_O; see
            // eval/attn_eval.md.)
            for c in 0..d_h {
                v[base + c][base + c] = Fx::ONE;
            }
        }

        // W_O = gain · I.
        let mut o = vec![vec![Fx::ZERO; d]; d];
        for i in 0..d {
            o[i][i] = gain;
        }

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
        let t = attention(
            &edges,
            &dialects,
            &e.data,
            &a.phi,
            d,
            a.h,
            a.l,
            a.diameter,
            Fx::ONE,
        );
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
        assert!(
            a.iter().zip(&b).all(|(x, y)| x
                .data
                .iter()
                .zip(&y.data)
                .all(|(p, q)| p.raw() == q.raw()))
        );
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
            Fx::ONE,
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

    /// Directed fixture: one-way links only, so (Aˡ)ᵀ ≠ Aˡ and the emitted
    /// Q/K must be genuinely asymmetric.
    fn directed_setup(gain: Fx) -> (Vec<Tensor>, usize, usize) {
        let mut links = Vec::new();
        for (a, b) in [(1u8, 2u8), (2, 3), (3, 4), (4, 1), (1, 3)] {
            links.push(link(a, b)); // one way only
        }
        let (_v, edges, adj) = build(&[], &links);
        let dialects = dialect::discover(&_v, &edges);
        let ar = arch::compute(&adj, dialects.len(), 0);
        let d = 16.min(ar.d.max(4));
        let e_t = embed::embed(&adj, &ar.phi, d);
        let t = attention(
            &edges,
            &dialects,
            &e_t.data,
            &ar.phi,
            d,
            ar.h,
            ar.l,
            ar.diameter,
            gain,
        );
        (t, ar.l, d)
    }

    #[test]
    fn p_attn_scores_are_directed() {
        // §7.2-rev: W_Q·W_Kᵀ reconstructs the DIRECTED projection P_dir and is
        // not symmetric — the self-collapsing undirected variant is gone.
        let (t, _l, d) = directed_setup(Fx::ONE);
        let q = &t
            .iter()
            .find(|x| x.name == "model.layers.0.self_attn.q_proj.weight")
            .unwrap()
            .data;
        let k = &t
            .iter()
            .find(|x| x.name == "model.layers.0.self_attn.k_proj.weight")
            .unwrap()
            .data;
        let qk: Vec<Vec<f64>> = (0..d)
            .map(|i| {
                (0..d)
                    .map(|j| {
                        (0..d)
                            .map(|c| q[i * d + c].to_f64() * k[j * d + c].to_f64())
                            .sum()
                    })
                    .collect()
            })
            .collect();
        let mut sym = 0.0;
        let mut tot = 0.0;
        for i in 0..d {
            for j in 0..d {
                sym += (qk[i][j] - qk[j][i]).powi(2);
                tot += qk[i][j].powi(2);
            }
        }
        assert!(
            sym.sqrt() / tot.sqrt().max(1e-12) > 0.05,
            "Q·Kᵀ nearly symmetric ({}) — directed projection not in effect",
            sym.sqrt() / tot.sqrt().max(1e-12)
        );
    }

    #[test]
    fn value_and_output_are_the_retrieval_head() {
        // §7.4-rev/§7.5-rev: W_V is the identity and W_O is gain·I.
        let gain = Fx::from_ratio(3, 2); // arbitrary non-trivial gain
        let (t, _l, d) = directed_setup(gain);
        let v = &t
            .iter()
            .find(|x| x.name == "model.layers.0.self_attn.v_proj.weight")
            .unwrap()
            .data;
        let o = &t
            .iter()
            .find(|x| x.name == "model.layers.0.self_attn.o_proj.weight")
            .unwrap()
            .data;
        for i in 0..d {
            for j in 0..d {
                let want_v = if i == j { Fx::ONE } else { Fx::ZERO };
                assert_eq!(v[i * d + j].raw(), want_v.raw(), "W_V[{i}][{j}]");
                let want_o = if i == j { gain } else { Fx::ZERO };
                assert_eq!(o[i * d + j].raw(), want_o.raw(), "W_O[{i}][{j}]");
            }
        }
    }
}
