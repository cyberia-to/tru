//! rewards — the value function and Shapley attribution (`tru/specs/rewards.md`).
//!
//! tru owns the *magnitude* layer of the reward (§14): the coalition value
//! `v(S) = Δφ⁺(A^eff ∪ S)` — the directed focus impulse of applying a set of
//! links to the graph — its surprise-weighted form `v★`, and the fair division
//! `Shapley(v★)`. The settlement *lottery* that distributes the Shapley sampling
//! across a leaderless swarm is [[foculus]]; conservation and mint are [[tok]].
//! What lives here is the estimator a settlement miner evaluates: given an
//! ordering, the marginal a contributor adds.
//!
//! `v★` scales each contributed link's stake by its [[Bayesian Truth Serum]]
//! surprise `ρ ∈ [0,1]` (§5): a copy (`ρ→0`) enters weightless, so the mint
//! divides *surprising* syntropy. Everything is fixed-point over the field.

use crate::arithmetic::Fx;
use crate::focusing::{impulse, Context, FocusingParams, Link};

/// One contributor's stake in a contested cluster: the neuron, its links, and
/// the per-contribution surprise `ρ` (from [[truth_scoring]]).
pub struct Contribution {
    pub neuron: [u8; 32],
    pub links: Vec<Link>,
    pub surprise: Fx,
}

impl Contribution {
    /// The links surprise-weighted for `v★`: each link's market multiplier
    /// folds in `ρ`, so the effective stake `stake·κ·f(price)` becomes
    /// `ρ·stake·κ·f(price)` — a copy contributes nothing.
    fn weighted(&self) -> Vec<Link> {
        let rho = clamp01(self.surprise);
        self.links
            .iter()
            .cloned()
            .map(|mut l| {
                l.price = clamp01(l.price) * rho;
                l
            })
            .collect()
    }
}

fn clamp01(x: Fx) -> Fx {
    if x < Fx::ZERO {
        Fx::ZERO
    } else if x > Fx::ONE {
        Fx::ONE
    } else {
        x
    }
}

/// `v★(S) = Δφ⁺(A^eff ∪ ρ·S)`: the surprise-weighted directed impulse of the
/// coalition `S` (a subset of contributions) applied on top of `base`.
pub fn value(base: &[Link], coalition: &[&Contribution], ctx: &Context, params: &FocusingParams) -> Fx {
    let mut batch = Vec::new();
    for c in coalition {
        batch.extend(c.weighted());
    }
    impulse(base, &batch, ctx, params, params.epsilon).directed
}

/// A deterministic permutation of `0..n` seeded by `beacon ‖ nonce` — the
/// settlement ordering `π(n)` (§7). Fisher–Yates driven by a hemera stream.
pub fn ordering(n: usize, beacon: &[u8; 32], nonce: u64) -> Vec<usize> {
    let mut perm: Vec<usize> = (0..n).collect();
    if n < 2 {
        return perm;
    }
    let mut buf = [0u8; 40];
    buf[..32].copy_from_slice(beacon);
    buf[32..].copy_from_slice(&nonce.to_le_bytes());
    let mut digest = *cyber_hemera::hash(&buf).as_bytes();
    let mut byte = 0usize;
    let rand_u64 = |digest: &mut [u8; 32], byte: &mut usize| -> u64 {
        if *byte + 8 > 32 {
            *digest = *cyber_hemera::hash(digest).as_bytes();
            *byte = 0;
        }
        let v = u64::from_le_bytes(digest[*byte..*byte + 8].try_into().unwrap());
        *byte += 8;
        v
    };
    for i in (1..n).rev() {
        let j = (rand_u64(&mut digest, &mut byte) % (i as u64 + 1)) as usize;
        perm.swap(i, j);
    }
    perm
}

/// The marginal a contributor adds under one ordering — the sample `m(n)` a
/// settlement miner computes (§7). Returns the per-contributor marginal
/// `v★(prefix ∪ {c}) − v★(prefix)` in ordering order.
pub fn sample_marginals(base: &[Link], contribs: &[Contribution], perm: &[usize], ctx: &Context, params: &FocusingParams) -> Vec<([u8; 32], Fx)> {
    let mut prefix: Vec<&Contribution> = Vec::new();
    let mut prev = Fx::ZERO; // v★(∅) = 0
    let mut out = Vec::with_capacity(perm.len());
    for &m in perm {
        prefix.push(&contribs[m]);
        let v = value(base, &prefix, ctx, params);
        out.push((contribs[m].neuron, v - prev));
        prev = v;
    }
    out
}

/// `Shapley(v★)` by Monte-Carlo over `samples` beacon-seeded orderings (§4, §7):
/// the average marginal each contributor adds. Returns `(neuron, share)` in
/// contribution order. Conservation (clipping to realized Δφ⁺) is [[tok]]'s step.
pub fn shapley(base: &[Link], contribs: &[Contribution], ctx: &Context, params: &FocusingParams, samples: u64, beacon: &[u8; 32]) -> Vec<([u8; 32], Fx)> {
    let n = contribs.len();
    let mut acc = vec![Fx::ZERO; n];
    if n == 0 || samples == 0 {
        return contribs.iter().map(|c| (c.neuron, Fx::ZERO)).collect();
    }
    for s in 0..samples {
        let perm = ordering(n, beacon, s);
        let marginals = sample_marginals(base, contribs, &perm, ctx, params);
        // marginals are in ordering order; map back to contributor index.
        for (k, &m) in perm.iter().enumerate() {
            acc[m] = acc[m] + marginals[k].1;
        }
    }
    let inv = Fx::ONE.div(Fx::from_int(samples as i64));
    contribs.iter().enumerate().map(|(i, c)| (c.neuron, acc[i] * inv)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(b: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    fn contrib(neuron: u8, links: Vec<Link>, rho: Fx) -> Contribution {
        Contribution { neuron: hash(neuron), links, surprise: rho }
    }

    fn beacon() -> [u8; 32] {
        hash(0xBE)
    }

    // A base graph the contributions attach to.
    fn base() -> Vec<Link> {
        vec![Link::stake(hash(1), hash(2), 100), Link::stake(hash(2), hash(3), 100), Link::stake(hash(3), hash(1), 100)]
    }

    #[test]
    fn value_of_empty_coalition_is_zero() {
        let v = value(&base(), &[], &Context::none(), &FocusingParams::default());
        assert_eq!(v.raw(), Fx::ZERO.raw());
    }

    #[test]
    fn a_copy_contributes_nothing() {
        // Same link, but ρ=0 (a copy of consensus) → v★ adds nothing.
        let c = contrib(9, vec![Link::stake(hash(3), hash(1), 400)], Fx::ZERO);
        let v = value(&base(), &[&c], &Context::none(), &FocusingParams::default());
        assert_eq!(v.raw(), Fx::ZERO.raw(), "a ρ=0 copy must add zero value");
    }

    #[test]
    fn ordering_is_deterministic_and_a_permutation() {
        let p1 = ordering(6, &beacon(), 3);
        let p2 = ordering(6, &beacon(), 3);
        assert_eq!(p1, p2, "same beacon+nonce → same ordering");
        let mut sorted = p1.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1, 2, 3, 4, 5], "must be a permutation of 0..n");
        assert_ne!(ordering(6, &beacon(), 3), ordering(6, &beacon(), 4), "different nonce → different ordering");
    }

    #[test]
    fn shapley_is_symmetric_for_identical_contributors() {
        // Two contributors staking the *same* edge with equal surprise are
        // interchangeable (v(S∪a)=v(S∪b) for all S), so Shapley must split their
        // credit equally.
        let a = contrib(10, vec![Link::stake(hash(2), hash(1), 8000)], Fx::ONE);
        let b = contrib(11, vec![Link::stake(hash(2), hash(1), 8000)], Fx::ONE);
        let shares = shapley(&base(), &[a, b], &Context::none(), &FocusingParams::default(), 8, &beacon());
        let (sa, sb) = (shares[0].1.to_f64(), shares[1].1.to_f64());
        assert!((sa - sb).abs() < 1e-6, "interchangeable contributors must split equally: {sa} vs {sb}");
    }

    #[test]
    fn shapley_is_efficient() {
        // Σ shares = v★(all) — the efficiency axiom, exact for full enumeration
        // and within Monte-Carlo error for sampling.
        let a = contrib(10, vec![Link::stake(hash(2), hash(1), 8000)], Fx::ONE);
        let b = contrib(11, vec![Link::stake(hash(3), hash(1), 6000)], Fx::ONE);
        let params = FocusingParams::default();
        let all = value(&base(), &[&a, &b], &Context::none(), &params);
        let shares = shapley(&base(), &[a, b], &Context::none(), &params, 12, &beacon());
        let sum: f64 = shares.iter().map(|s| s.1.to_f64()).sum();
        assert!((sum - all.to_f64()).abs() < 1e-3, "Σ Shapley {sum} ≠ v★(N) {}", all.to_f64());
    }

    #[test]
    fn a_null_copy_earns_near_zero_shapley() {
        let real = contrib(10, vec![Link::stake(hash(2), hash(1), 8000)], Fx::ONE);
        let copy = contrib(11, vec![Link::stake(hash(3), hash(1), 8000)], Fx::ZERO); // ρ=0
        let shares = shapley(&base(), &[real, copy], &Context::none(), &FocusingParams::default(), 8, &beacon());
        assert!(shares[1].1.to_f64().abs() < 1e-6, "ρ=0 copy earns ≈ 0 (got {})", shares[1].1.to_f64());
        assert!(shares[0].1.to_f64() > 0.0, "the real contributor earns the value");
    }
}
