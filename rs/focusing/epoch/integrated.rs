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

use super::fixed::{GRAD_BITS, PHI_BITS, Q_BITS, mul_wide_shr, shr_i};
use super::{Axon, EpochParams, canonical, gradient, midpoint};

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
    let mid = midpoint(&a, &b);
    let (mut i, mut j) = (0, 0);
    mid.iter()
        .map(|m| {
            let key = (m.from, m.to);
            let wa = if a.get(i).is_some_and(|x| (x.from, x.to) == key) {
                i += 1;
                a[i - 1].weight
            } else {
                0
            };
            let wb = if b.get(j).is_some_and(|x| (x.from, x.to) == key) {
                j += 1;
                b[j - 1].weight
            } else {
                0
            };
            (key, wa, wb)
        })
        .collect()
}

/// The chain's rule at the axon level: `ΔA · g(A_mid)` for every axon whose
/// weight changed, `A_mid = ⌊(A_prev + A_cur)/2⌋`, zero where `A_mid = 0`.
pub fn midpoint_attribution(prev: &[Axon], cur: &[Axon], p: &EpochParams) -> Vec<AxonScore> {
    let g = gradient(&midpoint(prev, cur), p);
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

/// `ΔA · Σ_t w_t · g(A_prev + t·ΔA)` by `k`-node Gauss–Legendre quadrature.
/// `k` cold gradient evaluations; reference and analysis, not consensus.
pub fn integrated_gradient(
    prev: &[Axon],
    cur: &[Axon],
    k: usize,
    p: &EpochParams,
) -> Vec<AxonScore> {
    let pr = pairs(prev, cur);
    let mut score = vec![0i128; pr.len()];
    for (t, w) in gauss_legendre(k) {
        let at: Vec<Axon> = pr
            .iter()
            .map(|&(axon, wa, wb)| {
                let d = wb as i128 - wa as i128;
                let x = wa as i128 + shr_i(t as i128 * d, Q_BITS);
                Axon {
                    from: axon.0,
                    to: axon.1,
                    weight: x.max(0) as u128,
                }
            })
            .collect();
        let g = gradient(&at, p);
        for ((s, &(_, wa, wb)), &gk) in score.iter_mut().zip(&pr).zip(&g.by_axon) {
            if wa != wb {
                let d = wb as i128 - wa as i128;
                *s += shr_i(w as i128 * score_of(d, gk), Q_BITS);
            }
        }
    }
    pr.into_iter()
        .zip(score)
        .filter(|(x, _)| x.1 != x.2)
        .map(|((axon, wa, wb), score)| AxonScore {
            axon,
            delta: wb as i128 - wa as i128,
            score,
        })
        .collect()
}
