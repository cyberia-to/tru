//! epoch — φ*, J and the syntropy gradient at the root chain's epoch boundary
//! (soft3 `specs/chain-economy.md` §4–§5, tru `specs/rewards.md` §16).
//!
//! The kernel's entry points, each a pure function of one canonical axon list:
//!
//! - [`phi`]: φ* over `A` — `PHI_STEPS` tri-kernel steps from the uniform
//!   vector, φ at scale 2^60, syntropy `J`, and the canonical digest
//!   preimage ([`Phi::preimage`], hashed chain-side with `D_PHI`);
//! - [`gradient`]: `g_pq = ∂J(φ*(A))/∂A_pq` for every present axon of `A`
//!   (the chain calls it on `A_mid` = [`midpoint`]) — one forward and one
//!   adjoint solve, also `PHI_STEPS` each, both cold;
//! - [`net_axons`]: the chain's §4.2 net weight from genesis weight and
//!   valence-signed positions (a reference for the chain's own code and the
//!   tests).
//!
//! Consensus properties:
//!
//! - order-independent: particles are indexed by ascending id, axons by
//!   `from ‖ to`; input order never matters;
//! - deterministic: integer arithmetic only ([`fixed`]), a fixed step count,
//!   a cold start; every parallel reduction is an exact integer sum, so
//!   results are bit-identical across runs, machines and thread counts;
//! - fast and memory-bounded: CSR over u32 indices relabelled for locality,
//!   `K + 1` sparse passes per step, `O(n + nnz)` memory, no eigen-solve.
//!
//! Operator choice. Heat and springs run on the random-walk Laplacian
//! `I − A_sym·D⁻¹` ([[tri-kernel]] §1.2 admits the normalised form) rather
//! than the combinatorial `D − A`. The combinatorial heat kernel costs
//! `O(τ·λ_max)` sparse passes and `λ_max` grows with the largest weighted
//! degree — 36,026 on bostrom with uniform stake, ~72,000 passes per step.
//! The normalised operators are scale-free (one whale's stake does not rescale
//! everyone's heat time), mass- and positivity-preserving, so `H_τ` is a
//! nonnegative Taylor polynomial of fixed degree `K` (9 at τ = 1, ε = 10⁻⁶).
//! The springs reference and the teleport prior are both the stake prior
//! `π = deg/Σdeg`, so a particle without stake carries exactly zero focus.
//! The legacy combinatorial kernel (`focusing::compute_focusing`) stays as the
//! research reference.

pub mod fixed;
pub mod gradient;
pub mod graph;
pub mod integrated;
pub mod kernel;

pub use graph::{Axon, AxonKey, EpochGraph, canonical};
pub use integrated::{AxonScore, integrated_gradient, midpoint_attribution};

use crate::arithmetic::Fx;

use kernel::{Coeffs, Work, adjoint_step, centred_log, forward_step, plan, uniform};

/// Tri-kernel steps per solve, fixed for consensus (chain-economy §0
/// `PHI_STEPS`). Evidence: tru `audit/epoch-settlement-2026-10.md` — from the
/// uniform start the ℓ1 distance to the converged φ* after 48 steps is below
/// 10⁻⁷ on pussy and bostrom (uniform and power-law stake).
pub const PHI_STEPS: usize = 48;

/// Domain separator of the φ digest (chain-economy §2, §5.6).
pub const D_PHI: &[u8] = b"soft3/chain/phi/v1\0";

/// Epoch-kernel parameters. Defaults: α = 0.15, μ = 1, τ = 1, λ = (0.5, 0.3,
/// 0.2), ε = 10⁻⁶ (sets the heat degree K = 9), `steps = PHI_STEPS`.
#[derive(Clone, Debug)]
pub struct EpochParams {
    /// Teleport probability α.
    pub alpha: Fx,
    /// Springs screening μ.
    pub mu: Fx,
    /// Heat time τ ∈ [0, 2] (bounds the Horner partial sums below 2^63).
    pub tau: Fx,
    /// Blend weights, `λ_d + λ_s + λ_h = 1`.
    pub lambda_d: Fx,
    pub lambda_s: Fx,
    pub lambda_h: Fx,
    /// Heat truncation target: `K` is the smallest degree with tail `≤ ε`.
    pub epsilon: Fx,
    /// Heat Taylor degree `K` (derived).
    pub heat_terms: usize,
    /// Steps per solve (consensus: [`PHI_STEPS`]).
    pub steps: usize,
    /// Worker threads. Never changes a result bit.
    pub threads: usize,
}

impl Default for EpochParams {
    fn default() -> Self {
        Self {
            alpha: Fx::from_ratio(15, 100),
            mu: Fx::ONE,
            tau: Fx::ONE,
            lambda_d: Fx::from_ratio(5, 10),
            lambda_s: Fx::from_ratio(3, 10),
            lambda_h: Fx::from_ratio(2, 10),
            epsilon: Fx::from_ratio(1, 1_000_000),
            heat_terms: 0,
            steps: PHI_STEPS,
            threads: 1,
        }
        .derived()
    }
}

impl EpochParams {
    /// The a-priori ℓ1 contraction of one step on the simplex,
    /// `κ = λ_d(1−α) + λ_s/(1+μ) + λ_h` (diffusion contracts by `1−α`, the
    /// screened springs step by `1/(1+μ)`, the stochastic heat polynomial by
    /// at most 1). 0.775 at the defaults; the observed rate on pussy and
    /// bostrom is ≈ 0.72.
    pub fn kappa(&self) -> Fx {
        self.lambda_d * (Fx::ONE - self.alpha)
            + self.lambda_s.div(Fx::ONE + self.mu)
            + self.lambda_h
    }

    /// Recompute `heat_terms` from τ and ε.
    pub fn derived(mut self) -> Self {
        assert!(
            self.tau >= Fx::ZERO && self.tau <= Fx::from_int(2),
            "τ must lie in [0, 2]"
        );
        // Tail e^{−τ}Σ_{k>K} τ^k/k! ≤ 2·e^{−τ}·τ^{K+1}/(K+1)! for τ ≤ 2.
        let e = (Fx::ZERO - self.tau).exp();
        let mut term = Fx::ONE;
        let mut k = 0usize;
        loop {
            term = term * self.tau.div(Fx::from_int(k as i64 + 1));
            if Fx::from_int(2) * e * term <= self.epsilon || k >= 32 {
                break;
            }
            k += 1;
        }
        self.heat_terms = k.max(1);
        self
    }

    fn plan(&self, g: &EpochGraph) -> Vec<usize> {
        plan(g, self.threads, self.heat_terms)
    }
}

/// φ* over one epoch graph, in ascending particle-id order.
#[derive(Clone, Debug)]
pub struct Phi {
    /// Particle ids, ascending.
    pub particles: Vec<[u8; 32]>,
    /// φ at scale 2^60 (`Σ ≈ 2^60`), aligned with `particles`.
    pub phi: Vec<u64>,
    /// Syntropy `J(φ) = Σ φ ln(n φ)`, nats at scale 2^60.
    pub syntropy: i128,
    /// ℓ1 change of the last step at scale 2^60 (a convergence witness).
    pub drift: u128,
}

impl Phi {
    /// The digest preimage `D_PHI ‖ n: u32 ‖ φ[0]: u64 ‖ … ‖ φ[n−1]: u64`,
    /// little-endian (chain-economy §5.6; the chain hashes it with hemera).
    pub fn preimage(&self) -> Vec<u8> {
        preimage(&self.phi)
    }
}

/// The φ digest preimage for a raw focus vector (ascending particle order).
pub fn preimage(phi: &[u64]) -> Vec<u8> {
    let mut v = Vec::with_capacity(D_PHI.len() + 4 + 8 * phi.len());
    v.extend_from_slice(D_PHI);
    v.extend_from_slice(&(phi.len() as u32).to_le_bytes());
    for &x in phi {
        v.extend_from_slice(&x.to_le_bytes());
    }
    v
}

/// The gradient at one graph.
#[derive(Clone, Debug)]
pub struct Gradient {
    /// `J(φ*)` at the graph the gradient was taken at (nats·2^60).
    pub syntropy: i128,
    /// Last-step ℓ1 drift of that φ* (2^60).
    pub drift: u128,
    /// `∂J/∂A` per *input* axon (same index as the slice passed in), nats per
    /// weight unit at scale 2^112; zero for an axon of weight zero or a
    /// self-loop. Duplicated input axons each carry their axon's gradient.
    pub by_axon: Vec<i128>,
}

/// Run `steps` forward steps from the uniform vector (internal order).
fn forward(
    g: &EpochGraph,
    p: &EpochParams,
    c: &Coeffs,
    b: &[usize],
    steps: usize,
) -> (Vec<u64>, u128) {
    let mut phi = uniform(g.n());
    let mut w = Work::new(g.n());
    let mut drift = 0;
    for _ in 0..steps {
        drift = forward_step(g, c, b, &mut phi, &mut w);
    }
    let _ = p;
    (phi, drift)
}

/// φ* of a built graph with an explicit step count (analysis; consensus uses
/// [`solve`]).
pub fn solve_steps(g: &EpochGraph, p: &EpochParams, steps: usize) -> Phi {
    let mut f = solve_steps_inner(g, p, steps);
    f.particles = g.particles.clone();
    f
}

fn solve_steps_inner(g: &EpochGraph, p: &EpochParams, steps: usize) -> Phi {
    if g.n() == 0 {
        return Phi {
            particles: vec![],
            phi: vec![],
            syntropy: 0,
            drift: 0,
        };
    }
    let c = Coeffs::new(p);
    let b = p.plan(g);
    let (phi, drift) = forward(g, p, &c, &b, steps);
    let syntropy = kernel::syntropy_with_n(&phi, g.n() as u64, &b);
    Phi {
        particles: vec![],
        phi: g.to_ext(&phi),
        syntropy,
        drift,
    }
}

/// φ* of a built graph: `p.steps` steps, cold.
pub fn solve(g: &EpochGraph, p: &EpochParams) -> Phi {
    solve_steps(g, p, p.steps)
}

/// φ* of an axon list (chain-economy §4.4 solve 1).
pub fn phi(axons: &[Axon], p: &EpochParams) -> Phi {
    let mut g = EpochGraph::build(axons);
    let mut f = solve_steps_inner(&g, p, p.steps);
    f.particles = std::mem::take(&mut g.particles);
    f
}

/// The gradient on a built graph, indexed like the axon slice it was built
/// from.
pub fn gradient_of(g: &EpochGraph, p: &EpochParams) -> Gradient {
    if g.n() == 0 {
        return Gradient {
            syntropy: 0,
            drift: 0,
            by_axon: vec![0; g.input_axon.len()],
        };
    }
    let c = Coeffs::new(p);
    let b = p.plan(g);
    let (phi, drift) = forward(g, p, &c, &b, p.steps);
    let grad = centred_log(&phi, &b);
    let mut lam = vec![0i64; g.n()];
    let mut w = Work::new(g.n());
    for _ in 0..p.steps {
        adjoint_step(g, &c, &b, &mut lam, &grad, &mut w);
    }
    drop(w);
    drop(grad);
    let syntropy = kernel::syntropy_with_n(&phi, g.n() as u64, &b);
    let per_edge = gradient::edge_gradient(g, &c, &b, &phi, &lam);
    let by_axon = g
        .input_axon
        .iter()
        .map(|&a| {
            if a == graph::NONE {
                0
            } else {
                per_edge[g.axon_edge[a as usize] as usize]
            }
        })
        .collect();
    Gradient {
        syntropy,
        drift,
        by_axon,
    }
}

/// `g = ∂J(φ*(A))/∂A` for every axon of `A`, aligned with the input slice
/// (chain-economy §5.1 solves 2 and 3, called on `A_mid`).
pub fn gradient(axons: &[Axon], p: &EpochParams) -> Gradient {
    gradient_of(&EpochGraph::build(axons), p)
}

/// `A_mid = ⌊(A_prev + A_cur)/2⌋` per axon present in either list, ascending
/// (zero-weight midpoints kept, so every changed axon has an entry).
pub fn midpoint(prev: &[Axon], cur: &[Axon]) -> Vec<Axon> {
    let (a, b) = (canonical(prev), canonical(cur));
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::with_capacity(a.len().max(b.len()));
    while i < a.len() || j < b.len() {
        let ord = match (a.get(i), b.get(j)) {
            (Some(x), Some(y)) => (x.from, x.to).cmp(&(y.from, y.to)),
            (Some(_), None) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Greater,
        };
        let (from, to, wa, wb) = match ord {
            std::cmp::Ordering::Less => {
                i += 1;
                (a[i - 1].from, a[i - 1].to, a[i - 1].weight, 0)
            }
            std::cmp::Ordering::Greater => {
                j += 1;
                (b[j - 1].from, b[j - 1].to, 0, b[j - 1].weight)
            }
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
                (a[i - 1].from, a[i - 1].to, a[i - 1].weight, b[j - 1].weight)
            }
        };
        out.push(Axon {
            from,
            to,
            weight: (wa / 2) + (wb / 2) + (wa % 2 + wb % 2) / 2,
        });
    }
    out
}

/// A staked position on an axon (chain-economy §3, §4.2).
#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub stake: u128,
    /// `v ∈ {−1, 0, +1}`: `0` and `+1` add weight, `−1` subtracts.
    pub valence: i8,
}

/// The chain's §4.2 net weight `A_pq = max(0, G_pq + S⁺_pq − S⁻_pq)` per axon,
/// canonical, absent axons (`A = 0`) dropped.
pub fn net_axons(genesis: &[Axon], positions: &[Position]) -> Vec<Axon> {
    let mut v: Vec<([u8; 32], [u8; 32], u128, u128)> = genesis
        .iter()
        .map(|a| (a.from, a.to, a.weight, 0))
        .chain(positions.iter().map(|p| {
            if p.valence < 0 {
                (p.from, p.to, 0, p.stake)
            } else {
                (p.from, p.to, p.stake, 0)
            }
        }))
        .collect();
    v.sort_unstable_by_key(|x| (x.0, x.1));
    let mut out = Vec::new();
    let mut k = 0;
    while k < v.len() {
        let (f, t) = (v[k].0, v[k].1);
        let (mut plus, mut minus) = (0u128, 0u128);
        while k < v.len() && v[k].0 == f && v[k].1 == t {
            plus = plus.saturating_add(v[k].2);
            minus = minus.saturating_add(v[k].3);
            k += 1;
        }
        let w = plus.saturating_sub(minus);
        if w > 0 && f != t {
            out.push(Axon {
                from: f,
                to: t,
                weight: w,
            });
        }
    }
    out
}

/// Syntropy `J(φ) = Σ φ_i ln(n φ_i)` at scale 2^60 for φ at scale 2^60 over
/// `n` particles (zero entries contribute nothing).
pub fn syntropy(phi: &[u64], n: u64) -> i128 {
    kernel::syntropy_with_n(phi, n, &[0, phi.len()])
}
