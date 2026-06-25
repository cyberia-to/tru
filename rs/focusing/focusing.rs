use std::collections::HashMap;

use super::csr::{CsrBuilder, CsrMatrix};
use super::operators::{self, normalize_l1};

// ── Parameters ────────────────────────────────────────────────────────

/// Tri-kernel parameters.
pub struct FocusingParams {
    /// PageRank teleport probability (diffusion).
    pub alpha: f64,
    /// Screened Laplacian screening strength (springs).
    pub mu: f64,
    /// Heat kernel temperature / time (heat).
    pub tau: f64,
    /// Blend weight for diffusion operator.
    pub lambda_d: f64,
    /// Blend weight for springs operator.
    pub lambda_s: f64,
    /// Blend weight for heat operator.
    pub lambda_h: f64,
    /// Max iterations for convergent operators (diffusion, springs).
    pub max_iter: usize,
    /// Convergence threshold (L1 norm of update).
    pub convergence: f64,
    /// Forward-Euler substeps for heat kernel approximation.
    pub heat_substeps: usize,
}

impl Default for FocusingParams {
    fn default() -> Self {
        Self {
            alpha: 0.15,
            mu: 1.0,
            tau: 1.0,
            lambda_d: 0.5,
            lambda_s: 0.3,
            lambda_h: 0.2,
            max_iter: 50,
            convergence: 1e-6,
            heat_substeps: 20,
        }
    }
}

// ── Input link type ───────────────────────────────────────────────────

/// A single cyberlink contributing to focusing.
pub struct Link {
    /// Source particle (32-byte hemera hash).
    pub from: [u8; 32],
    /// Target particle (32-byte hemera hash).
    pub to: [u8; 32],
    /// Stake amount (raw token units; cast to f64 for computation).
    pub amount: u128,
    /// Valence: +1 affirm, -1 challenge, 0 void/hold.
    pub valence: i8,
}

// ── FocusingGraph ────────────────────────────────────────────────────────

/// Pre-built adjacency structures for tri-kernel focusing computation.
///
/// Build once per snapshot with [`FocusingGraph::build`], then call
/// [`compute_focusing`] (possibly multiple times with different params).
pub struct FocusingGraph {
    n: usize,
    /// Particle hash at each node index.
    node_ids: Vec<[u8; 32]>,
    /// Transition matrix for diffusion: T[target][source] = w(src→tgt) / out_deg(src).
    transition: CsrMatrix,
    /// True for nodes with no outgoing edges.
    dangling: Vec<bool>,
    /// Symmetric weight matrix for springs/heat: W[i][j] = W[j][i].
    sym_weights: CsrMatrix,
    /// Weighted undirected degree: d[i] = Σ_j W[i][j].
    und_degree: Vec<f64>,
    /// Stake vector, normalized to sum to 1.
    stake: Vec<f64>,
}

impl FocusingGraph {
    /// Build from an iterator of cyberlinks.
    ///
    /// Self-loops and zero-amount links are silently skipped.
    /// Nodes are indexed only from valid edges.
    pub fn build(links: impl IntoIterator<Item = Link>) -> Self {
        // Pass 0: filter to valid raw edges (from, to, weight)
        let raw: Vec<([u8; 32], [u8; 32], f64)> = links
            .into_iter()
            .filter_map(|l| {
                let w = l.amount as f64;
                if w == 0.0 || l.from == l.to { None } else { Some((l.from, l.to, w)) }
            })
            .collect();

        if raw.is_empty() {
            return Self::empty();
        }

        // Pass 1: assign node indices (scope-limited to release borrows)
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

        // Pass 2: accumulate stake per node; record unique directed and undirected edges (binary topology)
        let mut dir_edges: HashMap<(usize, usize), ()> = HashMap::new();
        let mut und_edges: HashMap<(usize, usize), ()> = HashMap::new();
        let mut stake_raw = vec![0.0f64; n];

        for &(from, to, w) in &raw {
            let fi = node_index[&from];
            let ti = node_index[&to];
            dir_edges.entry((fi, ti)).or_insert(());
            und_edges.entry((fi.min(ti), fi.max(ti))).or_insert(());
            stake_raw[fi] += w;
            stake_raw[ti] += w;
        }

        // Binary out-degree per node
        let mut out_degree = vec![0usize; n];
        for &(fi, _) in dir_edges.keys() {
            out_degree[fi] += 1;
        }

        // Transition CSR: T[target][source] = 1 / out_deg(src)
        let mut trans_builder = CsrBuilder::new(n);
        for &(fi, ti) in dir_edges.keys() {
            if out_degree[fi] > 0 {
                trans_builder.add(ti, fi, 1.0 / out_degree[fi] as f64);
            }
        }
        let transition = trans_builder.build();
        let dangling: Vec<bool> = (0..n).map(|i| out_degree[i] == 0).collect();

        // Symmetric binary weight CSR + degree
        let mut und_degree = vec![0.0f64; n];
        let mut sym_builder = CsrBuilder::new(n);
        for &(a, b) in und_edges.keys() {
            sym_builder.add(a, b, 1.0);
            sym_builder.add(b, a, 1.0);
            und_degree[a] += 1.0;
            und_degree[b] += 1.0;
        }
        let sym_weights = sym_builder.build();

        let mut stake = stake_raw;
        normalize_l1(&mut stake);

        Self { n, node_ids, transition, dangling, sym_weights, und_degree, stake }
    }

    fn empty() -> Self {
        Self {
            n: 0,
            node_ids: vec![],
            transition: CsrBuilder::new(0).build(),
            dangling: vec![],
            sym_weights: CsrBuilder::new(0).build(),
            und_degree: vec![],
            stake: vec![],
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

    pub fn stake(&self) -> &[f64] {
        &self.stake
    }
}

// ── Output ────────────────────────────────────────────────────────────

/// Result of tri-kernel focusing computation.
pub struct FocusingResult {
    /// φ* focus distribution indexed by node index (same order as [`FocusingGraph::node_ids`]).
    pub focus: Vec<f64>,
    /// D component (personalized PageRank), normalized.
    pub diffusion: Vec<f64>,
    /// S component (screened Laplacian inverse), normalized.
    pub springs: Vec<f64>,
    /// H component (heat kernel), normalized.
    pub heat: Vec<f64>,
}

// ── Composite ────────────────────────────────────────────────────────

/// Compute φ* = λ_d·D + λ_s·S + λ_h·H, normalized to sum to 1.
pub fn compute_focusing(g: &FocusingGraph, p: &FocusingParams) -> FocusingResult {
    if g.n == 0 {
        return FocusingResult { focus: vec![], diffusion: vec![], springs: vec![], heat: vec![] };
    }

    let diffusion = operators::diffusion(
        g.n, &g.stake, &g.transition, &g.dangling,
        p.alpha, p.max_iter, p.convergence,
    );
    let springs = operators::springs(
        g.n, &g.stake, &g.sym_weights, &g.und_degree,
        p.mu, p.max_iter, p.convergence,
    );
    let heat = operators::heat(
        g.n, &g.stake, &g.sym_weights, &g.und_degree,
        p.tau, p.heat_substeps,
    );

    let mut focus = vec![0.0f64; g.n];
    for i in 0..g.n {
        focus[i] = p.lambda_d * diffusion[i] + p.lambda_s * springs[i] + p.lambda_h * heat[i];
    }
    normalize_l1(&mut focus);

    FocusingResult { focus, diffusion, springs, heat }
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
        let total: f64 = r.focus.iter().sum();
        assert!((total - 1.0).abs() < 0.01, "focus sums to {total}");
    }

    #[test]
    fn high_in_stake_ranks_higher() {
        // Node 1 is targeted by a 1000-amount link from node 4 — total stake touching node 1
        // (1200) is 6× that of node 3 (200). Node 1 should dominate focus.
        let links = vec![
            link(1, 2, 100),
            link(2, 3, 100),
            link(3, 1, 100),
            link(4, 1, 1000),
        ];
        let g = FocusingGraph::build(links);
        let r = compute_focusing(&g, &FocusingParams::default());
        let i1 = node_idx(&g, 1);
        let i3 = node_idx(&g, 3);
        assert!(
            r.focus[i1] > r.focus[i3],
            "high-in-stake node 1 ({:.4}) should outrank node 3 ({:.4})",
            r.focus[i1], r.focus[i3]
        );
    }

    #[test]
    fn well_linked_node_ranks_higher() {
        let links = vec![
            link(1, 2, 100),
            link(2, 3, 100),
            link(3, 1, 100),
            link(4, 1, 100),
        ];
        let g = FocusingGraph::build(links);
        let r = compute_focusing(&g, &FocusingParams::default());
        let i1 = node_idx(&g, 1);
        let i2 = node_idx(&g, 2);
        assert!(
            r.focus[i1] > r.focus[i2],
            "well-linked node 1 ({:.4}) should outrank node 2 ({:.4})",
            r.focus[i1], r.focus[i2]
        );
    }

    #[test]
    fn empty_graph() {
        let g = FocusingGraph::build(vec![]);
        assert_eq!(g.n(), 0);
        let r = compute_focusing(&g, &FocusingParams::default());
        assert!(r.focus.is_empty());
    }

    #[test]
    fn self_loops_excluded() {
        let g = FocusingGraph::build(vec![Link { from: hash(1), to: hash(1), amount: 100, valence: 1 }]);
        assert_eq!(g.n(), 0);
    }

    #[test]
    fn zero_amount_excluded() {
        let links = vec![link(1, 2, 0), link(2, 3, 50)];
        let g = FocusingGraph::build(links);
        assert_eq!(g.n(), 2);
    }
}
