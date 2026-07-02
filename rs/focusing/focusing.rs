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
    /// Signing neuron ν — the key karma is looked up under.
    pub neuron: [u8; 32],
    /// Source particle (32-byte hemera hash).
    pub from: [u8; 32],
    /// Target particle (32-byte hemera hash).
    pub to: [u8; 32],
    /// Stake amount (raw token units).
    pub amount: u128,
    /// Valence: +1 affirm, -1 challenge, 0 void/hold. Does not enter `A_eff`
    /// directly ([[focusing]]): its epistemic effect is mediated through `price`.
    pub valence: i8,
    /// Market believability `f(price(ℓ)) ∈ [0,1]`: the ICBS price mapped to an
    /// edge multiplier. `Fx::ONE` is market-neutral (fully believed); `ZERO`
    /// is fully doubted, structurally pruning the edge. Supplied by [[bbg]].
    pub price: Fx,
}

impl Link {
    /// A stake-only link with a neutral market (`price = 1`) and its own neuron
    /// as author. Recovers the pre-karma/price behaviour exactly.
    pub fn stake(from: [u8; 32], to: [u8; 32], amount: u128) -> Self {
        Self { neuron: from, from, to, amount, valence: 1, price: Fx::ONE }
    }
}

/// Per-neuron [[karma]] `κ(ν)`: the non-transferable BTS trust multiplier read
/// from [[bbg]] each epoch. An unknown neuron scores the neutral baseline
/// `Fx::ONE` — new identities are karma-light, never karma-negative.
#[derive(Default)]
pub struct Karma(HashMap<[u8; 32], Fx>);

impl Karma {
    /// No karma data — every neuron scores the neutral baseline. Recovers the
    /// stake-only weighting.
    pub fn none() -> Self {
        Self(HashMap::new())
    }

    /// Karma from an explicit `(neuron, κ)` table.
    pub fn from_pairs(pairs: impl IntoIterator<Item = ([u8; 32], Fx)>) -> Self {
        Self(pairs.into_iter().collect())
    }

    /// `κ(ν)`, defaulting to the neutral baseline `Fx::ONE`.
    pub fn get(&self, neuron: &[u8; 32]) -> Fx {
        self.0.get(neuron).copied().unwrap_or(Fx::ONE)
    }
}

/// The believability multiplier `f(price)`, clamped to `[0,1]`.
fn clamp01(x: Fx) -> Fx {
    if x < Fx::ZERO {
        Fx::ZERO
    } else if x > Fx::ONE {
        Fx::ONE
    } else {
        x
    }
}

// ── FocusingGraph ─────────────────────────────────────────────────────

/// Pre-built adjacency structures for one coupled tri-kernel computation.
///
/// Effective adjacency is the honesty-weighted sum ([[focusing]], [[truth-scoring]]):
/// `A_eff(p,q) = Σ_{ℓ: p→q} stake(ℓ)·κ(ν(ℓ))·f(price(ℓ))`. Stake is the economic
/// commitment, `κ(ν)` the neuron's [[karma]], `f(price)` the ICBS believability.
/// With `Karma::none()` and neutral `price = 1` this reduces to stake-weighting.
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
    /// Build the honesty-weighted effective adjacency from cyberlinks and the
    /// epoch's [[karma]] table. Self-loops, zero-stake links, and links the
    /// market fully doubts (`f(price)·κ = 0`) are skipped — a market-rejected
    /// link is structurally absent, not merely light.
    pub fn build(links: impl IntoIterator<Item = Link>, karma: &Karma) -> Self {
        // Keep raw stake as exact integers; the karma·price multipliers are
        // bounded and fold onto the fixed-point stake weight below.
        let kept: Vec<Link> =
            links.into_iter().filter(|l| l.amount != 0 && l.from != l.to).collect();

        if kept.is_empty() {
            return Self::empty();
        }

        // Stake weights are scale-invariant for φ*; normalize by the largest so
        // they land in (0,1] (comparable to μ=τ=1) and never overflow the field.
        // Effective weight w = stake·κ(ν)·f(price): κ(ν) ≥ 0 (default 1),
        // f(price) ∈ [0,1], so w ≤ stake ≤ 1 and overflow safety is preserved.
        let max_amount = kept.iter().map(|l| l.amount).max().unwrap_or(1);
        let raw: Vec<([u8; 32], [u8; 32], Fx)> = kept
            .iter()
            .map(|l| {
                let stake = Fx::ratio_u128(l.amount, max_amount);
                let w = stake * karma.get(&l.neuron) * clamp01(l.price);
                (l.from, l.to, w)
            })
            .filter(|&(_, _, w)| !w.is_zero())
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
        Link::stake(hash(from), hash(to), amount)
    }

    fn node_idx(g: &FocusingGraph, b: u8) -> usize {
        g.node_ids().iter().position(|h| h[0] == b && h[1..] == [0u8; 31][..]).unwrap()
    }

    #[test]
    fn focus_sums_to_one() {
        let links = vec![link(1, 2, 100), link(2, 3, 50), link(3, 1, 200)];
        let g = FocusingGraph::build(links, &Karma::none());
        let r = compute_focusing(&g, &FocusingParams::default());
        let total: f64 = r.focus.iter().map(|x| x.to_f64()).sum();
        assert!((total - 1.0).abs() < 1e-6, "focus sums to {total}");
        assert!(r.focus.iter().all(|x| *x > Fx::ZERO), "all focus positive");
    }

    #[test]
    fn deterministic_bit_identical() {
        let mk = || vec![link(1, 2, 100), link(2, 3, 50), link(3, 1, 200), link(4, 1, 300)];
        let a = compute_focusing(&FocusingGraph::build(mk(), &Karma::none()), &FocusingParams::default());
        let b = compute_focusing(&FocusingGraph::build(mk(), &Karma::none()), &FocusingParams::default());
        assert!(a.focus.iter().zip(&b.focus).all(|(x, y)| x.raw() == y.raw()), "φ* not bit-identical across runs");
    }

    #[test]
    fn contraction_below_one() {
        let links = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100), link(4, 1, 100)];
        let g = FocusingGraph::build(links, &Karma::none());
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
        let g = FocusingGraph::build(links, &Karma::none());
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
        let g = FocusingGraph::build(links, &Karma::none());
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
        let g = FocusingGraph::build(links, &Karma::none());
        let r = compute_focusing(&g, &FocusingParams::default());
        let (i1, i3) = (node_idx(&g, 1), node_idx(&g, 3));
        assert!(r.focus[i1] > r.focus[i3], "high-in-stake node 1 should outrank node 3");
    }

    #[test]
    fn well_linked_node_ranks_higher() {
        let links = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100), link(4, 1, 100)];
        let g = FocusingGraph::build(links, &Karma::none());
        let r = compute_focusing(&g, &FocusingParams::default());
        let (i1, i2) = (node_idx(&g, 1), node_idx(&g, 2));
        assert!(r.focus[i1] > r.focus[i2], "well-linked node 1 should outrank node 2");
    }

    #[test]
    fn large_stakes_are_scale_invariant() {
        // 10^15-scale stakes must not overflow and must give the SAME φ* as the
        // proportionally-smaller graph (weights are scale-invariant).
        let big = 1_000_000_000_000_000u128;
        let large = vec![
            Link::stake(hash(1), hash(2), big),
            Link::stake(hash(2), hash(3), big / 2),
            Link::stake(hash(4), hash(1), big * 3),
        ];
        let small = vec![link(1, 2, 1000), link(2, 3, 500), link(4, 1, 3000)];
        let rl = compute_focusing(&FocusingGraph::build(large, &Karma::none()), &FocusingParams::default());
        let rs = compute_focusing(&FocusingGraph::build(small, &Karma::none()), &FocusingParams::default());
        let total: f64 = rl.focus.iter().map(|x| x.to_f64()).sum();
        assert!((total - 1.0).abs() < 1e-6, "large-stake φ* sums to {total}");
        assert!(rl.focus.iter().all(|x| *x > Fx::ZERO));
        assert!(rl.focus.iter().zip(&rs.focus).all(|(a, b)| a.raw() == b.raw()), "φ* not scale-invariant");
    }

    #[test]
    fn empty_graph() {
        let g = FocusingGraph::build(vec![], &Karma::none());
        assert_eq!(g.n(), 0);
        assert!(compute_focusing(&g, &FocusingParams::default()).focus.is_empty());
    }

    #[test]
    fn self_loops_excluded() {
        let g = FocusingGraph::build(vec![Link::stake(hash(1), hash(1), 100)], &Karma::none());
        assert_eq!(g.n(), 0);
    }

    #[test]
    fn zero_amount_excluded() {
        let g = FocusingGraph::build(vec![link(1, 2, 0), link(2, 3, 50)], &Karma::none());
        assert_eq!(g.n(), 2);
    }

    // ── honesty weighting: karma and market price ─────────────────────

    // A voter V(=5) splits its focus between two otherwise-symmetric
    // candidates A(=1) and B(=2), each of which feeds a sink C(=3) that returns
    // to V. V has two out-edges, so re-weighting one genuinely redirects its
    // diffusion mass — the regime where honesty weighting shows up in the rank.
    fn voter_graph(w_a: (/*neuron*/ [u8; 32], /*price*/ Fx), w_b: ([u8; 32], Fx)) -> Vec<Link> {
        vec![
            Link { neuron: w_a.0, from: hash(5), to: hash(1), amount: 100, valence: 1, price: w_a.1 },
            Link { neuron: w_b.0, from: hash(5), to: hash(2), amount: 100, valence: 1, price: w_b.1 },
            link(1, 3, 100), // A → C
            link(2, 3, 100), // B → C
            link(3, 5, 100), // C → V
        ]
    }

    #[test]
    fn karma_amplifies_the_link_it_weights() {
        let (sign_a, sign_b) = (hash(7), hash(8));
        let mk = || voter_graph((sign_a, Fx::ONE), (sign_b, Fx::ONE));

        // Neutral: A and B are symmetric, so they share focus exactly.
        let g0 = FocusingGraph::build(mk(), &Karma::none());
        let r0 = compute_focusing(&g0, &FocusingParams::default());
        let gap = (r0.focus[node_idx(&g0, 1)].to_f64() - r0.focus[node_idx(&g0, 2)].to_f64()).abs();
        assert!(gap < 1e-6, "A and B must be symmetric under no karma (Δ={gap})");

        // κ=3 on A's signer sends more of V's focus to A: A outranks B.
        let g1 = FocusingGraph::build(mk(), &Karma::from_pairs([(sign_a, Fx::from_int(3))]));
        let r1 = compute_focusing(&g1, &FocusingParams::default());
        assert!(
            r1.focus[node_idx(&g1, 1)] > r1.focus[node_idx(&g1, 2)],
            "κ=3 on the A-link should make A outrank the symmetric B"
        );
    }

    #[test]
    fn market_price_scales_the_link_it_weights() {
        let (na, nb) = (hash(7), hash(8));
        // f(price)=0.5 on the B-link, full belief on A: A outranks B.
        let g = FocusingGraph::build(
            voter_graph((na, Fx::ONE), (nb, Fx::from_ratio(1, 2))),
            &Karma::none(),
        );
        let r = compute_focusing(&g, &FocusingParams::default());
        assert!(
            r.focus[node_idx(&g, 1)] > r.focus[node_idx(&g, 2)],
            "a fully-believed link should outrank a half-believed one"
        );
    }

    #[test]
    fn market_doubt_prunes_the_edge() {
        // f(price) = 0 (fully doubted): the edge is structurally absent, so its
        // endpoints never enter the graph.
        let doubted = Link { neuron: hash(9), from: hash(7), to: hash(8), amount: 100, valence: 1, price: Fx::ZERO };
        let g = FocusingGraph::build(vec![doubted, link(1, 2, 100)], &Karma::none());
        assert_eq!(g.n(), 2, "a fully-doubted edge must create no nodes");
        assert!(
            !g.node_ids().iter().any(|h| h[0] == 7 || h[0] == 8),
            "endpoints of the doubted edge must be absent"
        );
    }
}
