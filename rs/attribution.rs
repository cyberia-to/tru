//! attribution — the fair share of Δφ⁺ each contributor earned (`tru/specs/rewards.md`).
//!
//! tru owns the reward's *magnitude and division* — a pure function of the graph:
//! the coalition value `v★(S) = Δφ⁺(A^eff ∪ ρ·S)` (the surprise-weighted directed
//! focus impulse of a set of contributions), the marginal a contributor adds
//! under a given ordering, and the exact Shapley division that averages those
//! marginals. None of this touches money or consensus.
//!
//! What is *not* here, by design: the leaderless settlement **lottery** that
//! estimates the Shapley value at scale — drawing beacon-seeded orderings and
//! sampling marginals across a swarm — belongs to [[foculus]] (it needs the
//! epoch beacon and consensus). Conservation and the mint belong to [[tok]].
//! foculus's lottery calls [`marginals`] per sample; this module supplies the
//! deterministic per-ordering computation and the exact reference division.
//!
//! `v★` scales each contributed link's stake by its [[Bayesian Truth Serum]]
//! surprise `ρ ∈ [0,1]` (§5): a copy (`ρ→0`) enters weightless, so the economy
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
    /// The links surprise-weighted for `v★`: each link's market multiplier folds
    /// in `ρ`, so `stake·κ·f(price)` becomes `ρ·stake·κ·f(price)` — a copy
    /// contributes nothing.
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
pub fn value(
    base: &[Link],
    coalition: &[&Contribution],
    ctx: &Context,
    params: &FocusingParams,
) -> Fx {
    let mut batch = Vec::new();
    for c in coalition {
        batch.extend(c.weighted());
    }
    impulse(base, &batch, ctx, params, params.epsilon).directed
}

/// The per-contributor marginal along one `order` — the sample a settlement
/// miner computes for a given ordering (§7). `out[i]` is contributor `i`'s
/// marginal `v★(prefix ∪ {i}) − v★(prefix)`, `prefix` being the contributors
/// before `i` in `order`. This is the deterministic primitive foculus's lottery
/// draws against; the ordering itself is supplied by the caller (beacon-seeded
/// in settlement), never chosen here.
pub fn marginals(
    base: &[Link],
    contribs: &[Contribution],
    order: &[usize],
    ctx: &Context,
    params: &FocusingParams,
) -> Vec<Fx> {
    let mut out = vec![Fx::ZERO; contribs.len()];
    let mut prefix: Vec<&Contribution> = Vec::new();
    let mut prev = Fx::ZERO; // v★(∅) = 0
    for &i in order {
        prefix.push(&contribs[i]);
        let v = value(base, &prefix, ctx, params);
        out[i] = v - prev;
        prev = v;
    }
    out
}

/// The **exact** Shapley division of `v★` — the reference definition, averaging
/// [`marginals`] over *all* `n!` orderings. Deterministic and beacon-free, but
/// `O(n!)`: use it as the definition and for small clusters. Production settles
/// this by foculus's beacon-seeded sampling lottery, which estimates the same
/// value in `O(k·n)`. Conservation (clipping to realized Δφ⁺) is [[tok]]'s step.
pub fn shapley_exact(
    base: &[Link],
    contribs: &[Contribution],
    ctx: &Context,
    params: &FocusingParams,
) -> Vec<([u8; 32], Fx)> {
    let n = contribs.len();
    if n == 0 {
        return vec![];
    }
    let orders = permutations(n);
    let mut acc = vec![Fx::ZERO; n];
    for order in &orders {
        let m = marginals(base, contribs, order, ctx, params);
        for i in 0..n {
            acc[i] = acc[i] + m[i];
        }
    }
    let inv = Fx::ONE.div(Fx::from_int(orders.len() as i64));
    contribs
        .iter()
        .enumerate()
        .map(|(i, c)| (c.neuron, acc[i] * inv))
        .collect()
}

/// All permutations of `0..n` (Heap-style recursion). `O(n!)` — small `n` only.
fn permutations(n: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut cur: Vec<usize> = (0..n).collect();
    fn go(a: &mut Vec<usize>, k: usize, out: &mut Vec<Vec<usize>>) {
        if k == a.len() {
            out.push(a.clone());
            return;
        }
        for i in k..a.len() {
            a.swap(k, i);
            go(a, k + 1, out);
            a.swap(k, i);
        }
    }
    go(&mut cur, 0, &mut out);
    out
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
        Contribution {
            neuron: hash(neuron),
            links,
            surprise: rho,
        }
    }

    fn base() -> Vec<Link> {
        vec![
            Link::stake(hash(1), hash(2), 100),
            Link::stake(hash(2), hash(3), 100),
            Link::stake(hash(3), hash(1), 100),
        ]
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
    fn marginals_telescope_to_the_total() {
        // Along any ordering, the marginals sum to v★(all) — v★(∅) = v★(all).
        let a = contrib(10, vec![Link::stake(hash(2), hash(1), 8000)], Fx::ONE);
        let b = contrib(11, vec![Link::stake(hash(3), hash(1), 6000)], Fx::ONE);
        let cs = [a, b];
        let params = FocusingParams::default();
        let total = value(&base(), &[&cs[0], &cs[1]], &Context::none(), &params);
        let m = marginals(&base(), &cs, &[1, 0], &Context::none(), &params);
        let sum = m[0] + m[1];
        assert!(
            (sum.to_f64() - total.to_f64()).abs() < 1e-9,
            "marginals must telescope to v★(all)"
        );
    }

    #[test]
    fn shapley_exact_is_symmetric() {
        // Interchangeable contributors (same edge, equal surprise) split equally.
        let a = contrib(10, vec![Link::stake(hash(2), hash(1), 8000)], Fx::ONE);
        let b = contrib(11, vec![Link::stake(hash(2), hash(1), 8000)], Fx::ONE);
        let s = shapley_exact(
            &base(),
            &[a, b],
            &Context::none(),
            &FocusingParams::default(),
        );
        assert!(
            (s[0].1.to_f64() - s[1].1.to_f64()).abs() < 1e-9,
            "symmetric contributors split equally"
        );
    }

    #[test]
    fn shapley_exact_is_efficient() {
        // Σ shares = v★(all), exactly (full enumeration).
        let a = contrib(10, vec![Link::stake(hash(2), hash(1), 8000)], Fx::ONE);
        let b = contrib(11, vec![Link::stake(hash(3), hash(1), 6000)], Fx::ONE);
        let params = FocusingParams::default();
        let all = value(&base(), &[&a, &b], &Context::none(), &params);
        let s = shapley_exact(&base(), &[a, b], &Context::none(), &params);
        let sum: f64 = s.iter().map(|x| x.1.to_f64()).sum();
        assert!(
            (sum - all.to_f64()).abs() < 1e-9,
            "Σ Shapley {sum} ≠ v★(N) {}",
            all.to_f64()
        );
    }

    #[test]
    fn shapley_exact_gives_a_null_copy_zero() {
        let real = contrib(10, vec![Link::stake(hash(2), hash(1), 8000)], Fx::ONE);
        let copy = contrib(11, vec![Link::stake(hash(3), hash(1), 8000)], Fx::ZERO); // ρ=0
        let s = shapley_exact(
            &base(),
            &[real, copy],
            &Context::none(),
            &FocusingParams::default(),
        );
        assert!(
            s[1].1.to_f64().abs() < 1e-9,
            "ρ=0 copy earns 0 (got {})",
            s[1].1.to_f64()
        );
        assert!(
            s[0].1.to_f64() > 0.0,
            "the real contributor earns the value"
        );
    }
}
