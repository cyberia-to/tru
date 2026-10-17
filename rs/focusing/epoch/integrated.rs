//! Integrated-gradient attribution between two snapshots — the reference the
//! chain's one-point rule is measured against.
//!
//! The chain attributes with one gradient at the midpoint (chain-economy §5.1,
//! §5.2): per axon `ΔA·g(A_mid)`, then per position `Δs·σ·g`. This module
//! computes the same quantity at the axon level ([`midpoint_attribution`])
//! and its higher-order refinement, Gauss–Legendre quadrature of the gradient
//! along the straight path `A(t) = A_prev + t·ΔA` ([`integrated_gradient`]) —
//! the Aumann–Shapley value of rewards.md §4. Tests compare both with the
//! exact Shapley value on small graphs (`rs/tests/epoch_settlement.rs`); the
//! measured errors are in `audit/epoch-settlement-2026-10.md`.

use super::fixed::{GRAD_BITS, PHI_BITS, Q_BITS, mul_wide_floor, mul_wide_shr};
use super::{Axon, EpochGraph, EpochParams, canonical, gradient_of};

/// One axon's attribution: `(from, to)`, `ΔA = A_cur − A_prev`, and the score
/// in nats at scale 2^60 (signed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxonScore {
    pub axon: super::AxonKey,
    pub delta: i128,
    pub score: i128,
}

/// Gauss–Legendre nodes and weights on `[0, 1]` at scale 2^32 (weights sum to
/// exactly 2^32).
pub fn gauss_legendre(k: usize) -> Vec<(u64, u64)> {
    match k {
        1 => vec![(2_147_483_648, 4_294_967_296)],
        2 => vec![(907_633_386, 2_147_483_648), (3_387_333_910, 2_147_483_648)],
        3 => vec![
            (484_049_967, 1_193_046_471),
            (2_147_483_648, 1_908_874_354),
            (3_810_917_329, 1_193_046_471),
        ],
        4 => vec![
            (298_207_500, 747_012_592),
            (1_417_379_916, 1_400_471_056),
            (2_877_587_380, 1_400_471_056),
            (3_996_759_796, 747_012_592),
        ],
        _ => panic!("quadrature must be 1..=4 nodes"),
    }
}

/// `ΔA · g` at nats·2^60 for a gradient at scale 2^112.
pub fn score_of(delta: i128, g: i128) -> i128 {
    mul_wide_shr(delta, g, GRAD_BITS - PHI_BITS)
}

/// An axon key with its weight in the previous and the current snapshot.
type Pair = (super::AxonKey, u128, u128);

/// Union of both snapshots' axons with `(A_prev, A_cur)`, ascending.
fn pairs(prev: &[Axon], cur: &[Axon]) -> Vec<Pair> {
    let (a, b) = (canonical(prev), canonical(cur));
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::with_capacity(a.len().max(b.len()));
    while i < a.len() || j < b.len() {
        let ord = match (a.get(i), b.get(j)) {
            (Some(x), Some(y)) => (x.from, x.to).cmp(&(y.from, y.to)),
            (Some(_), None) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Greater,
        };
        out.push(match ord {
            std::cmp::Ordering::Less => {
                i += 1;
                ((a[i - 1].from, a[i - 1].to), a[i - 1].weight, 0)
            }
            std::cmp::Ordering::Greater => {
                j += 1;
                ((b[j - 1].from, b[j - 1].to), 0, b[j - 1].weight)
            }
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
                (
                    (a[i - 1].from, a[i - 1].to),
                    a[i - 1].weight,
                    b[j - 1].weight,
                )
            }
        });
    }
    out
}

/// The path-averaged gradient `ḡ = Σ_i w_i · g(A(t_i))` over the union of two
/// snapshots, `A(t) = A_prev + ⌊t·(A_cur − A_prev)⌋` (t at 2^32, floor), for
/// `p.quadrature` Gauss–Legendre nodes. With one node `A(½)` is exactly
/// `⌊(A_prev + A_cur)/2⌋`. The chain's attribution is `a(c) = Δs(c)·σ(c)·ḡ_pq`.
#[derive(Clone, Debug)]
pub struct PathGradient {
    /// The union axons `(from, to, A_cur)`, ascending — the order of `by_axon`.
    pub axons: Vec<Axon>,
    /// `ḡ` per axon, nats per weight unit at 2^112.
    pub by_axon: Vec<i128>,
}

/// [`PathGradient`] of the epoch from `prev` to `cur`: `p.quadrature`
/// gradient solves (each `p.steps` forward + `p.steps` adjoint, cold).
pub fn path_gradient(prev: &[Axon], cur: &[Axon], p: &EpochParams) -> PathGradient {
    let pr = pairs(prev, cur);
    let mut acc = vec![0i128; pr.len()];
    for (t, w) in gauss_legendre(p.quadrature) {
        let at: Vec<Axon> = pr
            .iter()
            .map(|&(axon, wa, wb)| Axon {
                from: axon.0,
                to: axon.1,
                weight: (wa as i128 + ((t as i128 * (wb as i128 - wa as i128)) >> Q_BITS)) as u128,
            })
            .collect();
        let graph = EpochGraph::build_with(&at, p.ground);
        drop(at); // the graph holds what the solves need
        let g = gradient_of(&graph, p);
        drop(graph);
        for (a, &gk) in acc.iter_mut().zip(&g.by_axon) {
            *a = a.saturating_add(mul_wide_floor(w as i128, gk, Q_BITS));
        }
    }
    PathGradient {
        axons: pr
            .iter()
            .map(|&(axon, _, wb)| Axon {
                from: axon.0,
                to: axon.1,
                weight: wb,
            })
            .collect(),
        by_axon: acc,
    }
}

/// `ΔA · ḡ` for every axon whose weight changed, with `k` quadrature nodes.
pub fn integrated_gradient(
    prev: &[Axon],
    cur: &[Axon],
    k: usize,
    p: &EpochParams,
) -> Vec<AxonScore> {
    let q = EpochParams {
        quadrature: k,
        ..p.clone()
    };
    let g = path_gradient(prev, cur, &q);
    pairs(prev, cur)
        .into_iter()
        .zip(g.by_axon)
        .filter(|(x, _)| x.1 != x.2)
        .map(|((axon, wa, wb), gk)| {
            let delta = wb as i128 - wa as i128;
            AxonScore {
                axon,
                delta,
                score: score_of(delta, gk),
            }
        })
        .collect()
}

/// The one-node rule (`ΔA · g(A_mid)`).
pub fn midpoint_attribution(prev: &[Axon], cur: &[Axon], p: &EpochParams) -> Vec<AxonScore> {
    integrated_gradient(prev, cur, 1, p)
}
