use std::collections::HashMap;

use crate::arithmetic::Fx;

use super::csr::{CsrBuilder, CsrMatrix};
use super::operators::{diffusion_step, heat_step, normalize_l1, springs_step};

// ── Parameters ────────────────────────────────────────────────────────

/// Tri-kernel parameters, fixed-point over the Goldilocks field.
pub struct FocusingParams {
    /// Diffusion teleport probability α.
    pub alpha: Fx,
    /// Springs screening strength μ.
    pub mu: Fx,
    /// Heat kernel time τ.
    pub tau: Fx,
    /// Blend weight for diffusion (λ_d + λ_s + λ_h = 1).
    pub lambda_d: Fx,
    /// Blend weight for springs.
    pub lambda_s: Fx,
    /// Blend weight for heat.
    pub lambda_h: Fx,
    /// Outer coupled iterations T (fixed step count → deterministic trace).
    pub iters: usize,
    /// Heat forward-Euler substeps (need τ/substeps ≤ 1 for stability).
    pub substeps: usize,
}

impl Default for FocusingParams {
    fn default() -> Self {
        Self {
            alpha: Fx::from_ratio(15, 100),
            mu: Fx::ONE,
            tau: Fx::ONE,
            lambda_d: Fx::from_ratio(5, 10),
            lambda_s: Fx::from_ratio(3, 10),
            lambda_h: Fx::from_ratio(2, 10),
            iters: 50,
            substeps: 20,
        }
    }
}

// ── Input link type ───────────────────────────────────────────────────

/// A single cyberlink contributing to the field.
pub struct Link {
    /// Source particle (32-byte hemera hash).
    pub from: [u8; 32],
    /// Target particle (32-byte hemera hash).
    pub to: [u8; 32],
    /// Stake amount (raw token units).
    pub amount: u128,
    /// Valence: +1 affirm, -1 challenge, 0 void/hold.
    pub valence: i8,
}

// ── FocusingGraph ─────────────────────────────────────────────────────

/// Pre-built adjacency structures for one coupled tri-kernel computation.
///
/// Effective adjacency is stake-weighted: `A_eff(p,q) = Σ_{ℓ: p→q} stake(ℓ)`.
/// (Karma and market price join in M3; until then stake is the weight.)
pub struct FocusingGraph {
    n: usize,
    /// Particle hash at each node index.
    node_ids: Vec<[u8; 32]>,
    /// Column-stochastic transition: `transition[q][p] = A_eff(p,q)/out_strength(p)`.
    transition: CsrMatrix,
    /// True for nodes with no outgoing strength.
    dangling: Vec<bool>,
    /// Symmetric weights `A_sym(i,j) = A_eff(i,j) + A_eff(j,i)`.
    sym_weights: CsrMatrix,
    /// Weighted undirected degree `d(i) = Σ_j A_sym(i,j)`.
    und_degree: Vec<Fx>,
    /// Stake-weighted teleport prior, normalized to sum 1.
    teleport: Vec<Fx>,
}

impl FocusingGraph {
    /// Build from cyberlinks. Self-loops and zero-amount links are skipped.
    pub fn build(links: impl IntoIterator<Item = Link>) -> Self {
        let raw: Vec<([u8; 32], [u8; 32], Fx)> = links
            .into_iter()
            .filter_map(|l| {
                if l.amount == 0 || l.from == l.to {
                    None
                } else {
                    // Stake as fixed-point; realistic token amounts fit i64.
                    Some((l.from, l.to, Fx::from_int(l.amount.min(i64::MAX as u128) as i64)))
                }
            })
            .collect();

        if raw.is_empty() {
            return Self::empty();
        }

        // Node indices, assigned by first appearance (deterministic).
        let mut node_ids: Vec<[u8; 32]> = Vec::new();
        let mut node_index: HashMap<[u8; 32], usize> = HashMap::new();
        for &(from, to, _) in &raw {
            for hash in [from, to] {
                if !node_index.contains_key(&hash) {
                    node_index.insert(hash, node_ids.len());
                    node_ids.push(hash);
                }
            }
        }
        let n = node_ids.len();

        // Directed A_eff and per-node stake mass (for the teleport prior).
        let mut dir_weight: HashMap<(usize, usize), Fx> = HashMap::new();
        let mut out_strength = vec![Fx::ZERO; n];
        let mut node_stake = vec![Fx::ZERO; n];
        for &(from, to, w) in &raw {
            let (fi, ti) = (node_index[&from], node_index[&to]);
            let e = dir_weight.entry((fi, ti)).or_insert(Fx::ZERO);
            *e = *e + w;
            out_strength[fi] = out_strength[fi] + w;
            node_stake[fi] = node_stake[fi] + w;
            node_stake[ti] = node_stake[ti] + w;
        }

        // Transition (col-stochastic) and symmetric weights + degree.
        let mut trans = CsrBuilder::new(n);
        let mut sym = CsrBuilder::new(n);
        let mut und_degree = vec![Fx::ZERO; n];
        for (&(fi, ti), &w) in &dir_weight {
            trans.add(ti, fi, w.div(out_strength[fi])); // T[to][from]
            sym.add(fi, ti, w);
            sym.add(ti, fi, w);
            und_degree[fi] = und_degree[fi] + w;
            und_degree[ti] = und_degree[ti] + w;
        }
        let dangling: Vec<bool> = (0..n).map(|i| out_strength[i].is_zero()).collect();

        let teleport = normalize_l1(&node_stake);

        Self {
            n,
            node_ids,
            transition: trans.build(),
            dangling,
            sym_weights: sym.build(),
            und_degree,
            teleport,
        }
    }

    fn empty() -> Self {
        Self {
            n: 0,
            node_ids: vec![],
            transition: CsrBuilder::new(0).build(),
            dangling: vec![],
            sym_weights: CsrBuilder::new(0).build(),
            und_degree: vec![],
            teleport: vec![],
        }
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn node_id(&self, idx: usize) -> &[u8; 32] {
        &self.node_ids[idx]
    }

    pub fn node_ids(&self) -> &[[u8; 32]] {
        &self.node_ids
    }
}

// ── Output ────────────────────────────────────────────────────────────

/// Result of one tri-kernel computation.
pub struct FocusingResult {
    /// φ* focus distribution (fixed-point), indexed as [`FocusingGraph::node_ids`].
    pub focus: Vec<Fx>,
    /// Diffusion component of the final step.
    pub diffusion: Vec<Fx>,
    /// Springs component of the final step.
    pub springs: Vec<Fx>,
    /// Heat component of the final step.
    pub heat: Vec<Fx>,
}

// ── Composite: one coupled iteration to the fixed point ───────────────

/// Compute φ* by iterating the coupled tri-kernel: each step applies D, S, and
/// H_τ to the same current φ, blends `λ_d·D + λ_s·S + λ_h·H`, normalizes onto
/// the simplex, and feeds φ back — repeated a fixed `iters` times. Fixed-point
/// throughout, so two runs on the same graph are bit-identical.
pub fn compute_focusing(g: &FocusingGraph, p: &FocusingParams) -> FocusingResult {
    if g.n == 0 {
        return FocusingResult { focus: vec![], diffusion: vec![], springs: vec![], heat: vec![] };
    }
    let n = g.n;
    let uniform = Fx::from_ratio(1, n as i64);
    let x0 = vec![uniform; n];

    let mut phi = vec![uniform; n];
    let mut diffusion = vec![Fx::ZERO; n];
    let mut springs = vec![Fx::ZERO; n];
    let mut heat = vec![Fx::ZERO; n];

    for _ in 0..p.iters {
        diffusion = diffusion_step(&phi, &g.transition, &g.dangling, &g.teleport, p.alpha);
        springs = springs_step(&phi, &g.sym_weights, &g.und_degree, p.mu, &x0);
        heat = heat_step(&phi, &g.sym_weights, &g.und_degree, p.tau, p.substeps);

        let blend: Vec<Fx> = (0..n)
            .map(|i| p.lambda_d * diffusion[i] + p.lambda_s * springs[i] + p.lambda_h * heat[i])
            .collect();
        phi = normalize_l1(&blend);
    }

    FocusingResult { focus: phi, diffusion, springs, heat }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(b: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    fn link(from: u8, to: u8, amount: u128) -> Link {
        Link { from: hash(from), to: hash(to), amount, valence: 1 }
    }

    fn node_idx(g: &FocusingGraph, b: u8) -> usize {
        g.node_ids().iter().position(|h| h[0] == b && h[1..] == [0u8; 31][..]).unwrap()
    }

    #[test]
    fn focus_sums_to_one() {
        let links = vec![link(1, 2, 100), link(2, 3, 50), link(3, 1, 200)];
        let g = FocusingGraph::build(links);
        let r = compute_focusing(&g, &FocusingParams::default());
        let total: f64 = r.focus.iter().map(|x| x.to_f64()).sum();
        assert!((total - 1.0).abs() < 1e-6, "focus sums to {total}");
        assert!(r.focus.iter().all(|x| *x > Fx::ZERO), "all focus positive");
    }

    #[test]
    fn deterministic_bit_identical() {
        let mk = || vec![link(1, 2, 100), link(2, 3, 50), link(3, 1, 200), link(4, 1, 300)];
        let a = compute_focusing(&FocusingGraph::build(mk()), &FocusingParams::default());
        let b = compute_focusing(&FocusingGraph::build(mk()), &FocusingParams::default());
        assert!(a.focus.iter().zip(&b.focus).all(|(x, y)| x.raw() == y.raw()), "φ* not bit-identical across runs");
    }

    #[test]
    fn converges_drift_shrinks() {
        let links = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100), link(4, 1, 100)];
        let g = FocusingGraph::build(links);
        let long = compute_focusing(&g, &FocusingParams::default());
        let short = compute_focusing(&g, &FocusingParams { iters: 25, ..FocusingParams::default() });
        let drift: f64 = long.focus.iter().zip(&short.focus).map(|(a, b)| (a.to_f64() - b.to_f64()).abs()).sum();
        assert!(drift < 1e-3, "drift 25→50 iters = {drift}, not converged");
    }

    #[test]
    fn high_in_stake_ranks_higher() {
        let links = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100), link(4, 1, 1000)];
        let g = FocusingGraph::build(links);
        let r = compute_focusing(&g, &FocusingParams::default());
        let (i1, i3) = (node_idx(&g, 1), node_idx(&g, 3));
        assert!(r.focus[i1] > r.focus[i3], "high-in-stake node 1 should outrank node 3");
    }

    #[test]
    fn well_linked_node_ranks_higher() {
        let links = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100), link(4, 1, 100)];
        let g = FocusingGraph::build(links);
        let r = compute_focusing(&g, &FocusingParams::default());
        let (i1, i2) = (node_idx(&g, 1), node_idx(&g, 2));
        assert!(r.focus[i1] > r.focus[i2], "well-linked node 1 should outrank node 2");
    }

    #[test]
    fn empty_graph() {
        let g = FocusingGraph::build(vec![]);
        assert_eq!(g.n(), 0);
        assert!(compute_focusing(&g, &FocusingParams::default()).focus.is_empty());
    }

    #[test]
    fn self_loops_excluded() {
        let g = FocusingGraph::build(vec![Link { from: hash(1), to: hash(1), amount: 100, valence: 1 }]);
        assert_eq!(g.n(), 0);
    }

    #[test]
    fn zero_amount_excluded() {
        let g = FocusingGraph::build(vec![link(1, 2, 0), link(2, 3, 50)]);
        assert_eq!(g.n(), 2);
    }
}
