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
    /// Convergence target ε: the iteration runs T(ε) = min t with κ^t ≤ ε.
    pub epsilon: Fx,
    /// Hard cap on outer iterations (used when κ is degenerate).
    pub iter_cap: usize,
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
            epsilon: Fx::from_ratio(1, 1_000_000),
            iter_cap: 500,
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
    /// Largest Laplacian eigenvalue ‖L‖ (for the contraction κ).
    lambda_max: Fx,
    /// Algebraic connectivity λ₂ (Fiedler value).
    lambda_2: Fx,
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
        let sym_weights = sym.build();

        // Spectrum for the contraction κ (graph-only; params fold in at compute).
        let lambda_max = super::spectral::lambda_max(&sym_weights, &und_degree, n, 60);
        let lambda_2 = super::spectral::lambda_2(&sym_weights, &und_degree, n, lambda_max, 120);

        Self {
            n,
            node_ids,
            transition: trans.build(),
            dangling,
            sym_weights,
            und_degree,
            teleport,
            lambda_max,
            lambda_2,
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
            lambda_max: Fx::ZERO,
            lambda_2: Fx::ZERO,
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

    /// Largest Laplacian eigenvalue ‖L‖.
    pub fn lambda_max(&self) -> Fx {
        self.lambda_max
    }

    /// Algebraic connectivity λ₂ (Fiedler value).
    pub fn lambda_2(&self) -> Fx {
        self.lambda_2
    }
}

// ── Output ────────────────────────────────────────────────────────────

/// Result of one tri-kernel computation.
pub struct FocusingResult {
    /// φ* focus distribution (fixed-point), indexed as [`FocusingGraph::node_ids`].
    pub focus: Vec<Fx>,
    /// Syntropy J(φ*) = D_KL(φ* ‖ u), emitted alongside φ* every epoch.
    pub syntropy: Fx,
    /// Diffusion component of the final step.
    pub diffusion: Vec<Fx>,
    /// Springs component of the final step.
    pub springs: Vec<Fx>,
    /// Heat component of the final step.
    pub heat: Vec<Fx>,
}

// ── Composite: one coupled iteration to the fixed point ───────────────

/// The composite contraction coefficient κ for this graph and params.
pub fn contraction(g: &FocusingGraph, p: &FocusingParams) -> Fx {
    super::spectral::kappa(p, g.lambda_max, g.lambda_2)
}

/// The step count T(ε) the coupled iteration runs: the smallest T with κ^T ≤ ε
/// ([[tri-kernel]] §2.2), capped by `p.iter_cap`.
pub fn derived_steps(g: &FocusingGraph, p: &FocusingParams) -> usize {
    super::spectral::steps_for(contraction(g, p), p.epsilon, p.iter_cap)
}

/// Compute φ* by iterating the coupled tri-kernel: each step applies D, S, and
/// H_τ to the same current φ, blends `λ_d·D + λ_s·S + λ_h·H`, normalizes onto
/// the simplex, and feeds φ back — for a fixed T(ε) steps derived from the
/// contraction κ. Fixed-point throughout, so two runs are bit-identical.
pub fn compute_focusing(g: &FocusingGraph, p: &FocusingParams) -> FocusingResult {
    iterate(g, p, derived_steps(g, p))
}

/// The coupled iteration run for an explicit step count.
pub fn iterate(g: &FocusingGraph, p: &FocusingParams, steps: usize) -> FocusingResult {
    if g.n == 0 {
        return FocusingResult { focus: vec![], syntropy: Fx::ZERO, diffusion: vec![], springs: vec![], heat: vec![] };
    }
    let n = g.n;
    let uniform = Fx::from_ratio(1, n as i64);
    let x0 = vec![uniform; n];

    let mut phi = vec![uniform; n];
    let mut diffusion = vec![Fx::ZERO; n];
    let mut springs = vec![Fx::ZERO; n];
    let mut heat = vec![Fx::ZERO; n];

    for _ in 0..steps {
        diffusion = diffusion_step(&phi, &g.transition, &g.dangling, &g.teleport, p.alpha);
        springs = springs_step(&phi, &g.sym_weights, &g.und_degree, p.mu, &x0);
        heat = heat_step(&phi, &g.sym_weights, &g.und_degree, g.lambda_max, p.tau);

        let blend: Vec<Fx> = (0..n)
            .map(|i| p.lambda_d * diffusion[i] + p.lambda_s * springs[i] + p.lambda_h * heat[i])
            .collect();
        phi = normalize_l1(&blend);
    }

    let syntropy = super::measures::syntropy(&phi);
    FocusingResult { focus: phi, syntropy, diffusion, springs, heat }
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
    fn contraction_below_one() {
        let links = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100), link(4, 1, 100)];
        let g = FocusingGraph::build(links);
        let p = FocusingParams::default();
        let kappa = contraction(&g, &p);
        assert!(kappa < Fx::ONE, "κ = {} must be < 1", kappa.to_f64());
        assert!(kappa > Fx::ZERO, "κ = {} must be > 0", kappa.to_f64());
        // λ_max is real (positive) for a nonempty graph.
        assert!(g.lambda_max() > Fx::ZERO, "λ_max should be positive");
    }

    #[test]
    fn derived_steps_reach_the_fixed_point() {
        let links = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100), link(4, 1, 100)];
        let g = FocusingGraph::build(links);
        let p = FocusingParams::default();
        let t = derived_steps(&g, &p);
        assert!(t > 0 && t < p.iter_cap, "derived T = {t} should be a real step count");
        // Past T(ε) the iterate barely moves — the fixed point is reached.
        let at_t = iterate(&g, &p, t);
        let past_t = iterate(&g, &p, t + 20);
        let drift: f64 = at_t.focus.iter().zip(&past_t.focus).map(|(a, b)| (a.to_f64() - b.to_f64()).abs()).sum();
        assert!(drift < 1e-4, "drift past T = {drift}, not converged");
    }

    #[test]
    fn heat_conserves_mass_and_smooths() {
        let links = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100), link(4, 1, 100)];
        let g = FocusingGraph::build(links);
        let n = g.n();
        let i1 = node_idx(&g, 1);
        // A delta spike at node 1.
        let mut v = vec![Fx::ZERO; n];
        v[i1] = Fx::ONE;
        let h = heat_step(&v, &g.sym_weights, &g.und_degree, g.lambda_max, Fx::ONE);
        // exp(−τL) conserves mass (L·1 = 0), up to series truncation.
        let mass: f64 = h.iter().map(|x| x.to_f64()).sum();
        assert!((mass - 1.0).abs() < 1e-2, "heat should conserve mass, got {mass}");
        // and it diffuses the spike off the peak while staying ~positive.
        assert!(h[i1].to_f64() < 1.0, "heat should spread mass off node 1");
        assert!(h.iter().all(|x| x.to_f64() > -1e-3), "heat stays approximately positive");
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
