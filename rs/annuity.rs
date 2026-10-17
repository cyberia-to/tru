//! The yield stream — the annuity (`specs/rewards.md` §11–12).
//!
//! `R_{i→j}(T) = ∫₀ᵀ ω(t)·Δφ*_j(t) dt`: the delayed mint of foundational
//! work, paid as the target particle's [[cyberank]] keeps growing. The
//! present re-scores itself every epoch (§12), so this is a discrete
//! Riemann sum over epoch samples, not a closed-form integral — each
//! epoch contributes `ω(t)·Δφ*_j(t)` once, and the running total is
//! exposed after every sample so a caller can pay it out per epoch.

use crate::arithmetic::Fx;

/// One epoch's sample for an active (`v_ℓ ≠ 0`) link: the per-epoch weight
/// `ω(t)` (current ICBS price × karma, computed upstream) and the target
/// particle's focus delta `Δφ*_j(t)` this epoch.
#[derive(Clone, Copy, Debug)]
pub struct EpochSample {
    pub omega: Fx,
    pub delta_phi: Fx,
}

/// This epoch's contribution to the annuity, `ω(t)·[Δφ*_j(t)]₊`. The
/// integrand is read as the directed impulse of §2: growth of the target's
/// focus pays, a decline pays nothing — never a negative draw, because a
/// paid epoch is final (§12, no actor reaches back). `ω` is a price × karma
/// product and nonnegative by construction; a negative input is clipped
/// rather than trusted. Passive (`v_ℓ = 0`) stake never calls this —
/// eligibility is a caller-side gate (§9's two axes), not a property of
/// the integrand.
#[inline]
pub fn epoch_yield(sample: EpochSample) -> Fx {
    let omega = sample.omega.max(Fx::ZERO);
    let growth = sample.delta_phi.max(Fx::ZERO);
    omega * growth
}

/// The annuity accrued through epoch `T`, as the running discrete sum
/// `Σ_{t≤T} ω(t)·Δφ*_j(t)`. A link falsified at epoch `t` (its ICBS price,
/// hence `ω(t)`, collapses to zero) stops drawing from that epoch on but
/// keeps everything already accrued — the sum never revisits a past term,
/// matching §12: "self-correcting: a link later falsified simply stops
/// drawing it," never reversed.
pub fn accrue(samples: &[EpochSample]) -> Fx {
    samples.iter().copied().fold(Fx::ZERO, |acc, s| acc + epoch_yield(s))
}

/// `accrue` at every prefix, for a caller that pays the annuity out per
/// epoch rather than as one lump sum at `T`.
pub fn accrue_running(samples: &[EpochSample]) -> Vec<Fx> {
    let mut total = Fx::ZERO;
    samples
        .iter()
        .copied()
        .map(|s| {
            total = total + epoch_yield(s);
            total
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(omega_n: i64, omega_d: i64, delta_n: i64, delta_d: i64) -> EpochSample {
        EpochSample {
            omega: Fx::from_ratio(omega_n, omega_d),
            delta_phi: Fx::from_ratio(delta_n, delta_d),
        }
    }

    fn close(a: Fx, want: f64) {
        let got = a.to_f64();
        assert!((got - want).abs() < 1e-6, "got {got}, want {want}");
    }

    #[test]
    fn empty_history_accrues_nothing() {
        assert_eq!(accrue(&[]), Fx::ZERO);
        assert!(accrue_running(&[]).is_empty());
    }

    #[test]
    fn single_epoch_is_the_integrand() {
        let s = sample(1, 2, 1, 4); // ω=0.5, Δφ*=0.25
        close(accrue(&[s]), 0.125);
    }

    #[test]
    fn accrual_sums_across_epochs() {
        let samples = [sample(1, 1, 1, 10), sample(1, 1, 2, 10), sample(1, 1, 3, 10)];
        close(accrue(&samples), 0.6); // 0.1+0.2+0.3 at ω=1
    }

    #[test]
    fn running_total_never_drops() {
        // A zero-growth epoch is flat; a negative-growth epoch (the target's
        // focus fell) is flat too — the directed impulse pays descent only,
        // and a paid epoch is never clawed back (§2, §12).
        let samples = [
            sample(1, 1, 1, 10),
            sample(1, 1, 2, 10),
            sample(1, 1, 0, 10),
            sample(1, 1, -5, 10),
        ];
        let running = accrue_running(&samples);
        assert_eq!(running.len(), 4);
        for w in running.windows(2) {
            assert!(w[0] <= w[1], "running total dropped: {:?} -> {:?}", w[0], w[1]);
        }
        assert_eq!(running[1], running[3], "zero and negative growth must both be flat");
        assert_eq!(epoch_yield(sample(1, 1, -5, 10)), Fx::ZERO);
        close(running[3], accrue(&samples).to_f64());
    }

    #[test]
    fn falsified_link_stops_drawing_but_keeps_past_accrual() {
        // ω collapses to zero at epoch 3 (price → 0); the running total
        // freezes from there — it never reverses the first two epochs.
        let samples = [
            sample(1, 1, 3, 10),
            sample(1, 1, 3, 10),
            sample(0, 1, 100, 1), // falsified: ω=0 makes a huge Δφ* count for nothing
        ];
        let running = accrue_running(&samples);
        close(running[2], running[1].to_f64());
        assert!(running[1] > Fx::ZERO);
    }

    #[test]
    fn order_of_epochs_is_not_commutative_with_running_totals() {
        // accrue() (the final total) is order-independent, but the
        // intermediate running totals are not — a caller paying per epoch
        // must feed samples in epoch order.
        let a = sample(1, 1, 1, 10);
        let b = sample(1, 1, 2, 10);
        close(accrue(&[a, b]), accrue(&[b, a]).to_f64());
        assert_ne!(accrue_running(&[a, b])[0], accrue_running(&[b, a])[0]);
    }
}
