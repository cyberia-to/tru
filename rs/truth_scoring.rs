//! truth scoring — Bayesian Truth Serum → karma (`specs/truth-scoring.md`).
//!
//! The honesty layer. Each [[cyberlink]] is a BTS report: the link-plus-stake is
//! the first-order belief `p` (how likely the link is valid), the [[valence]] is
//! the meta-prediction `m` (where the neuron thinks the crowd will land). BTS
//! scores a neuron by how much private signal it added — its belief being closer
//! to what the crowd *actually* holds than to what the crowd *predicted*, minus
//! how badly it mispredicted the crowd (Prelec, 2004):
//!
//! ```text
//! s_ν = [ D_KL(p ‖ m̄_−ν) − D_KL(p ‖ p̄_−ν) ]  −  D_KL(p̄_−ν ‖ m)
//!         └──────── information gain ────────┘     └ prediction accuracy ┘
//! ```
//!
//! `p̄_−ν`, `m̄_−ν` are the leave-one-out geometric means of the other neurons'
//! beliefs and predictions. The score is positive exactly when a neuron
//! contributed private signal the crowd did not already hold and expect;
//! copying the consensus drives the information gain to zero.
//!
//! Scores accumulate into [[karma]] (the reputation the tri-kernel reads as
//! `κ(ν)`) and squash into the per-contribution surprise `ρ` the mint divides
//! (`rewards.md` §5). This module computes the scores and the update rule; a
//! writer ([[plumb]]) persists karma to [[bbg]], which [[focusing]] reads back.
//!
//! Binary outcome (valid / invalid) — the cyberlink case. Everything is
//! fixed-point over the Goldilocks field ([[arithmetic]]).

use std::collections::HashMap;

use crate::arithmetic::Fx;
use crate::focusing::Karma;

/// A neuron's BTS report on one question (cluster of a link's validity).
pub struct Report {
    /// The reporting neuron ν.
    pub neuron: [u8; 32],
    /// First-order belief `p` = P(the link is valid) ∈ [0,1].
    pub belief: Fx,
    /// Meta-prediction `m` = the neuron's estimate of the fraction of *others*
    /// who believe the link valid ∈ [0,1] — the [[valence]] signal.
    pub prediction: Fx,
}

/// Clamp a probability into `(δ, 1−δ)` so logarithms stay finite.
fn clampp(x: Fx) -> Fx {
    let d = Fx::from_ratio(1, 1_000_000);
    let hi = Fx::ONE - d;
    if x < d {
        d
    } else if x > hi {
        hi
    } else {
        x
    }
}

/// Binary KL divergence `D_KL((a,1−a) ‖ (b,1−b))` in nats, on yes-probabilities.
fn kl(a: Fx, b: Fx) -> Fx {
    let a = clampp(a);
    let b = clampp(b);
    a * a.div(b).ln() + (Fx::ONE - a) * (Fx::ONE - a).div(Fx::ONE - b).ln()
}

/// Normalized geometric-mean yes-probability of a set of binary beliefs — the
/// aggregate `p̄` BTS compares against (geometric, not arithmetic: it is the mean
/// in log-odds, robust to correlated over-confidence).
fn geo_mean(yes: &[Fx]) -> Fx {
    let m = yes.len();
    if m == 0 {
        return Fx::from_ratio(1, 2);
    }
    let mut sly = Fx::ZERO;
    let mut sln = Fx::ZERO;
    for &x in yes {
        let x = clampp(x);
        sly = sly + x.ln();
        sln = sln + (Fx::ONE - x).ln();
    }
    let inv = Fx::ONE.div(Fx::from_int(m as i64));
    let gy = (sly * inv).exp();
    let gn = (sln * inv).exp();
    gy.div(gy + gn)
}

/// The BTS score `s_ν` for every report, leave-one-out. Requires ≥ 2 reports
/// (BTS needs a crowd to score against); fewer returns all-zero.
pub fn bts_scores(reports: &[Report]) -> Vec<Fx> {
    let n = reports.len();
    if n < 2 {
        return vec![Fx::ZERO; n];
    }
    let beliefs: Vec<Fx> = reports.iter().map(|r| r.belief).collect();
    let preds: Vec<Fx> = reports.iter().map(|r| r.prediction).collect();

    (0..n)
        .map(|i| {
            let others_b: Vec<Fx> = (0..n).filter(|&k| k != i).map(|k| beliefs[k]).collect();
            let others_m: Vec<Fx> = (0..n).filter(|&k| k != i).map(|k| preds[k]).collect();
            let pbar = geo_mean(&others_b);
            let mbar = geo_mean(&others_m);
            let (p, m) = (beliefs[i], preds[i]);
            let info_gain = kl(p, mbar) - kl(p, pbar);
            let pred_acc = kl(pbar, m);
            info_gain - pred_acc
        })
        .collect()
}

/// Accumulate a BTS score into a neuron's [[karma]] `κ`: `κ' = max(0, κ + η·s)`.
/// Karma is the running record of honest signal — it rises on positive score,
/// falls on noise, and is floored at zero (the tri-kernel multiplier `κ(ν)` is
/// never negative). Starts from the neutral baseline `κ = 1` for a new neuron.
pub fn karma_step(kappa: Fx, score: Fx, eta: Fx) -> Fx {
    let next = kappa + eta * score;
    if next < Fx::ZERO {
        Fx::ZERO
    } else {
        next
    }
}

/// The per-contribution surprise `ρ = clip(s / s_max, 0, 1)` the mint divides
/// (`rewards.md` §5): a copy of the consensus scores `s ≤ 0 → ρ = 0` and mints
/// nothing; a maximally surprising true report approaches `ρ = 1`.
pub fn surprise(score: Fx, s_max: Fx) -> Fx {
    if s_max <= Fx::ZERO {
        return Fx::ZERO;
    }
    let r = score.div(s_max);
    if r < Fx::ZERO {
        Fx::ZERO
    } else if r > Fx::ONE {
        Fx::ONE
    } else {
        r
    }
}

/// Fold one epoch of reports into an updated [[karma]] table: each reporting
/// neuron's prior κ (default 1) advances by its BTS score. The result is the
/// `Karma` the tri-kernel reads as `κ(ν)` — closing the honesty loop from
/// reports to effective adjacency.
pub fn accumulate(prior: &Karma, reports: &[Report], eta: Fx) -> Karma {
    let scores = bts_scores(reports);
    let mut next: HashMap<[u8; 32], Fx> = HashMap::new();
    for (r, &s) in reports.iter().zip(&scores) {
        let base = next.get(&r.neuron).copied().unwrap_or_else(|| prior.get(&r.neuron));
        next.insert(r.neuron, karma_step(base, s, eta));
    }
    Karma::from_pairs(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(b: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    fn report(id: u8, belief: f64, prediction: f64) -> Report {
        Report {
            neuron: hash(id),
            belief: Fx::from_ratio((belief * 1000.0) as i64, 1000),
            prediction: Fx::from_ratio((prediction * 1000.0) as i64, 1000),
        }
    }

    #[test]
    fn consensus_copier_scores_near_zero_information() {
        // Everyone believes and predicts the same — no private signal anywhere.
        let reports =
            vec![report(1, 0.7, 0.7), report(2, 0.7, 0.7), report(3, 0.7, 0.7), report(4, 0.7, 0.7)];
        let s = bts_scores(&reports);
        for (i, &si) in s.iter().enumerate() {
            assert!(si.to_f64().abs() < 1e-2, "copier {i} should score ≈ 0 (got {})", si.to_f64());
        }
    }

    #[test]
    fn accurate_meta_prediction_beats_an_inaccurate_one() {
        // Two neurons share the same belief; one predicts the crowd correctly,
        // the other wildly. The accurate predictor must score higher (the
        // prediction-accuracy term is the only difference).
        let reports = vec![
            report(1, 0.6, 0.6), // crowd context
            report(2, 0.6, 0.6),
            report(3, 0.6, 0.6),
            report(10, 0.6, 0.6), // accurate meta-prediction
            report(11, 0.6, 0.05), // same belief, badly wrong prediction
        ];
        let s = bts_scores(&reports);
        assert!(s[3] > s[4], "accurate predictor ({}) must beat inaccurate ({})", s[3].to_f64(), s[4].to_f64());
    }

    #[test]
    fn surprisingly_popular_report_beats_a_follower() {
        // The classic SP case. Most report "valid" (0.8) and expect others to as
        // well. An informed minority reports "invalid" (0.15) but *predicts* the
        // crowd will say valid (0.8) — its answer is more popular than the crowd
        // predicted. It should outscore a plain consensus follower.
        let reports = vec![
            report(1, 0.8, 0.8),
            report(2, 0.8, 0.8),
            report(3, 0.8, 0.8),
            report(4, 0.15, 0.8), // informed contrarian, accurate meta-prediction
        ];
        let s = bts_scores(&reports);
        let follower = s[0].to_f64();
        let contrarian = s[3].to_f64();
        assert!(contrarian > follower, "contrarian ({contrarian}) should beat follower ({follower})");
    }

    #[test]
    fn karma_accumulates_up_on_signal_and_floors_at_zero() {
        let eta = Fx::from_ratio(1, 2);
        // Positive score raises karma above the neutral baseline.
        let up = karma_step(Fx::ONE, Fx::from_ratio(1, 2), eta);
        assert!(up > Fx::ONE, "positive BTS score should raise karma");
        // A large negative score cannot push karma below zero.
        let floored = karma_step(Fx::from_ratio(1, 10), Fx::from_int(-100), eta);
        assert_eq!(floored.raw(), Fx::ZERO.raw(), "karma is floored at zero");
    }

    #[test]
    fn surprise_clips_to_unit_interval() {
        let smax = Fx::ONE;
        assert_eq!(surprise(Fx::from_int(-5), smax).raw(), Fx::ZERO.raw(), "noise → ρ = 0");
        assert_eq!(surprise(Fx::from_int(5), smax).raw(), Fx::ONE.raw(), "very surprising → ρ = 1");
        let mid = surprise(Fx::from_ratio(1, 2), smax);
        assert!((mid.to_f64() - 0.5).abs() < 1e-6, "ρ scales linearly in-range");
    }

    #[test]
    fn accumulate_closes_the_loop_to_a_karma_table() {
        // An informative contrarian ends with higher karma than a follower.
        let reports = vec![
            report(1, 0.8, 0.8),
            report(2, 0.8, 0.8),
            report(3, 0.8, 0.8),
            report(4, 0.15, 0.8),
        ];
        let karma = accumulate(&Karma::none(), &reports, Fx::from_ratio(1, 4));
        assert!(
            karma.get(&hash(4)) > karma.get(&hash(1)),
            "the contrarian's karma ({}) should exceed the follower's ({})",
            karma.get(&hash(4)).to_f64(),
            karma.get(&hash(1)).to_f64()
        );
    }
}
