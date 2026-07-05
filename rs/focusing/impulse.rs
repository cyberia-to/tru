//! impulse — Δφ*, the proven focus shift one signal delivers (`specs/impulse.md`).
//!
//! A neuron's signal is a batch of new [[cyberlinks]]. Its impulse is how the
//! focus distribution φ* moves when the batch is applied to the graph the
//! neuron observed. This module computes the exact shift: build φ* before and
//! after, align by particle, take the difference.
//!
//! Two quantities come out. The sparse vector Δφ* = φ*_after − φ*_before is the
//! per-particle displacement (`specs/impulse.md`). The scalar directed impulse
//! Δφ⁺ = [J(φ*_after) − J(φ*_before)]₊ is the reward primitive: the clipped
//! gain in [[syntropy]] the batch produced (`rewards.md` §1–§2). Its exact
//! decomposition `ΔJ = (H_before − H_after) + ln(|P_after|/|P_before|)` splits
//! the locally-attributable entropy drop from the global discovery term.
//!
//! Everything is fixed-point over the Goldilocks field ([[arithmetic]]); the
//! before/after runs use the same derived T(ε) step count, so the shift is
//! bit-reproducible — the property one [[zheng]] proof certifies.
//!
//! This computes the shift over the *whole* supplied graph. The locality
//! theorem ([[tri-kernel]]) guarantees the same numbers arise from a recompute
//! restricted to the batch's O(log 1/ε)-hop neighbourhood; that neighbourhood
//! restriction is a later performance pass that must match this ground truth.

use crate::arithmetic::Fx;

use super::focusing::{compute_focusing, Context, FocusingGraph, FocusingParams, Link};
use super::measures::entropy;

/// The proven focus shift of one signal.
pub struct Impulse {
    /// Sparse Δφ* = φ*_after − φ*_before: `(particle, signed shift)` for every
    /// particle whose |shift| ≥ ε. Deterministic order (after-graph node order).
    pub delta: Vec<([u8; 32], Fx)>,
    /// ΔJ = J(φ*_after) − J(φ*_before): the exact syntropy change, signed.
    pub delta_j: Fx,
    /// Δφ⁺ = [ΔJ]₊ — the reward primitive: syntropy gain clipped at zero, so
    /// only sharpening is paid, never the noise a flattening link adds.
    pub directed: Fx,
    /// H(φ*_before) − H(φ*_after): the locally-attributable entropy drop.
    pub entropy_drop: Fx,
    /// ln(|P_after|/|P_before|): the global, non-attributable discovery term,
    /// nonzero only when the batch introduces new particles. Zero on an empty
    /// base (the ratio is undefined; ΔJ carries the whole shift).
    pub discovery: Fx,
    /// ‖Δφ*‖₁ = Σ_p |Δφ*(p)| — the unsigned magnitude of the shift.
    pub norm_l1: Fx,
}

/// |x| over the field.
fn abs(x: Fx) -> Fx {
    if x < Fx::ZERO {
        Fx::ZERO - x
    } else {
        x
    }
}

/// Compute the impulse of `batch` applied on top of `base`, under the attention
/// `ctx` and `params`. `epsilon` is the sparsity floor: Δφ* entries below it are
/// dropped (the same precision floor the locality bound uses).
pub fn impulse(
    base: &[Link],
    batch: &[Link],
    ctx: &Context,
    params: &FocusingParams,
    epsilon: Fx,
) -> Impulse {
    // φ* before: the graph the neuron observed.
    let g0 = FocusingGraph::build(base.iter().cloned(), ctx);
    let r0 = compute_focusing(&g0, params);

    // φ* after: the same graph with the signal's links added.
    let union: Vec<Link> = base.iter().cloned().chain(batch.iter().cloned()).collect();
    let g1 = FocusingGraph::build(union, ctx);
    let r1 = compute_focusing(&g1, params);

    // φ*_before(p) by particle hash (0 for particles the batch introduced).
    let before = |p: &[u8; 32]| -> Fx {
        match g0.node_ids().iter().position(|h| h == p) {
            Some(i) => r0.focus[i],
            None => Fx::ZERO,
        }
    };

    // Δφ* over the after-graph node set (a superset of the before set, since
    // base ⊆ union), sparsified at ε. ‖·‖₁ sums the full unsigned shift.
    let mut delta = Vec::new();
    let mut norm_l1 = Fx::ZERO;
    for (i, pid) in g1.node_ids().iter().enumerate() {
        let d = r1.focus[i] - before(pid);
        let m = abs(d);
        norm_l1 = norm_l1 + m;
        if m >= epsilon {
            delta.push((*pid, d));
        }
    }

    // ΔJ and its split. J = ln|P| − H, so ΔJ = (H_before − H_after) + ln(n1/n0);
    // discovery is that log-ratio, undefined (→ 0) when the base is empty.
    let delta_j = r1.syntropy - r0.syntropy;
    let entropy_drop = entropy(&r0.focus) - entropy(&r1.focus);
    let (n0, n1) = (g0.n(), g1.n());
    let discovery = if n0 == 0 || n1 == 0 {
        Fx::ZERO
    } else {
        Fx::from_ratio(n1 as i64, n0 as i64).ln()
    };
    let directed = if delta_j > Fx::ZERO {
        delta_j
    } else {
        Fx::ZERO
    };

    Impulse {
        delta,
        delta_j,
        directed,
        entropy_drop,
        discovery,
        norm_l1,
    }
}

/// The **propose** claim — the first, local run of the reward (`rewards.md` §6).
///
/// A neuron computes its own standalone directed impulse `Δφ⁺_ν = v({ν}) − v(∅)`
/// — the focus shift its links produce on the graph it observed — proves it, and
/// self-mints against it before any settlement. This is `impulse(...).directed`,
/// named for its role: tru owns the local first run; the epoch-boundary Shapley
/// division among overlapping contributors is settlement, and lives in
/// [[foculus]]. Among substitutes this standalone value is a ceiling on the
/// settled share, so it is a safe claim to gossip immediately.
pub fn propose(base: &[Link], links: &[Link], ctx: &Context, params: &FocusingParams) -> Fx {
    impulse(base, links, ctx, params, params.epsilon).directed
}

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

    fn eps() -> Fx {
        Fx::from_ratio(1, 100_000)
    }

    // A batch that reshapes an existing region shifts φ* and gains syntropy.
    #[test]
    fn a_new_link_shifts_focus_and_is_local() {
        let base = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100)];
        let batch = vec![link(3, 1, 400)]; // reinforce 1's inbound
        let imp = impulse(
            &base,
            &batch,
            &Context::none(),
            &FocusingParams::default(),
            eps(),
        );
        assert!(!imp.delta.is_empty(), "a reshaping link must move φ*");
        assert!(imp.norm_l1 > Fx::ZERO, "‖Δφ*‖₁ must be positive");
        // directed is the clip of ΔJ.
        let expect = if imp.delta_j > Fx::ZERO {
            imp.delta_j
        } else {
            Fx::ZERO
        };
        assert_eq!(
            imp.directed.raw(),
            expect.raw(),
            "directed must equal [ΔJ]₊"
        );
    }

    // The syntropy identity ΔJ = (H_before − H_after) + ln(n1/n0) holds exactly.
    #[test]
    fn delta_j_decomposes_into_entropy_drop_plus_discovery() {
        let base = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100)];
        let batch = vec![link(1, 4, 250)]; // introduces particle 4 (discovery)
        let imp = impulse(
            &base,
            &batch,
            &Context::none(),
            &FocusingParams::default(),
            eps(),
        );
        let lhs = imp.delta_j.to_f64();
        let rhs = imp.entropy_drop.to_f64() + imp.discovery.to_f64();
        assert!(
            (lhs - rhs).abs() < 1e-3,
            "ΔJ ({lhs}) ≠ entropy_drop + discovery ({rhs})"
        );
    }

    // A link to a brand-new particle charges the discovery term; one that reuses
    // existing particles does not.
    #[test]
    fn discovery_term_fires_only_on_new_particles() {
        let base = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100)];
        let disc = impulse(
            &base,
            &[link(1, 9, 200)],
            &Context::none(),
            &FocusingParams::default(),
            eps(),
        );
        assert!(
            disc.discovery > Fx::ZERO,
            "a new particle must charge discovery > 0"
        );

        let reuse = impulse(
            &base,
            &[link(1, 3, 200)],
            &Context::none(),
            &FocusingParams::default(),
            eps(),
        );
        assert!(
            abs(reuse.discovery) < eps(),
            "no new particle ⇒ discovery ≈ 0 (got {})",
            reuse.discovery.to_f64()
        );
    }

    // A neuron's first links: empty base, φ*_before = 0 everywhere, ΔJ = J_after.
    #[test]
    fn first_links_from_an_empty_base() {
        let batch = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100)];
        let imp = impulse(
            &[],
            &batch,
            &Context::none(),
            &FocusingParams::default(),
            eps(),
        );
        assert!(!imp.delta.is_empty(), "first links must register a shift");
        assert_eq!(
            imp.discovery.raw(),
            Fx::ZERO.raw(),
            "empty base has no discovery ratio"
        );
        // every Δφ* entry equals φ*_after (before was zero).
        let mass: f64 = imp.delta.iter().map(|(_, d)| d.to_f64()).sum();
        assert!(
            (mass - 1.0).abs() < 1e-3,
            "Δφ* from empty base should sum to 1 (φ*_after), got {mass}"
        );
    }

    // Directed impulse never pays for a flattening (syntropy-lowering) batch.
    #[test]
    fn directed_clips_a_flattening_batch() {
        // Base concentrates focus on node 1 (heavy inbound). The batch spreads
        // node 1's out-mass across 2 and 3, flattening the distribution toward
        // uniform without adding particles — ΔJ < 0, so directed clips to 0.
        let base = vec![link(2, 1, 1000), link(3, 1, 1000), link(1, 2, 10)];
        let batch = vec![link(1, 2, 5000), link(1, 3, 5000)];
        let imp = impulse(
            &base,
            &batch,
            &Context::none(),
            &FocusingParams::default(),
            eps(),
        );
        assert!(
            imp.delta_j < Fx::ZERO,
            "this batch should lower syntropy (ΔJ={})",
            imp.delta_j.to_f64()
        );
        assert_eq!(
            imp.directed.raw(),
            Fx::ZERO.raw(),
            "a syntropy-lowering batch must mint nothing"
        );
    }

    // The propose claim (§6) is exactly the standalone directed impulse — the
    // local first run of the reward, before any settlement.
    #[test]
    fn propose_is_the_standalone_directed_impulse() {
        let base = vec![link(1, 2, 100), link(2, 3, 100), link(3, 1, 100)];
        let mine = vec![link(2, 1, 8000)]; // a sharpening link toward the hub
        let params = FocusingParams::default();
        let claim = propose(&base, &mine, &Context::none(), &params);
        let directed = impulse(&base, &mine, &Context::none(), &params, eps()).directed;
        assert_eq!(
            claim.raw(),
            directed.raw(),
            "propose = impulse(...).directed"
        );
        assert!(
            claim.to_f64() > 0.0,
            "a real contribution proposes a positive reward ceiling"
        );
    }
}
