//! Security-budget allocation between PoW and PoS (`specs/rewards.md` §10).
//!
//! `R_PoW = B·(1 − θ^α)`, `R_PoS = B·θ^α`, `α ∈ [0.3, 0.7]` with `α = 0.5`
//! the neutral prior. The security floor is derived from attack economics,
//! not chosen: `floor ≥ c_sec · (TVL/M) · r_atk`.

use crate::arithmetic::Fx;

/// `α ∈ [0.3, 0.7]`, clamped; `0.5` is the neutral prior (equal marginal
/// security cost between PoW and PoS).
#[inline]
pub fn clamp_alpha(alpha: Fx) -> Fx {
    let lo = Fx::from_ratio(3, 10);
    let hi = Fx::from_ratio(7, 10);
    if alpha < lo {
        lo
    } else if alpha > hi {
        hi
    } else {
        alpha
    }
}

/// `θ^α` via `exp(α · ln θ)`. `θ ≤ 0` yields zero (no stake bonded, all
/// security budget flows to PoW); `θ` above `1` is a caller error clamped
/// to `1` (the staking ratio is a fraction of supply).
pub fn theta_pow_alpha(theta: Fx, alpha: Fx) -> Fx {
    let theta = if theta > Fx::ONE { Fx::ONE } else { theta };
    if theta.is_zero() || theta < Fx::ZERO {
        return Fx::ZERO;
    }
    (alpha * theta.ln()).exp()
}

/// The PoW/PoS split of the security budget `B` at staking ratio `θ`:
/// `(R_PoW, R_PoS) = (B·(1 − θ^α), B·θ^α)`.
pub fn split(budget: Fx, theta: Fx, alpha: Fx) -> (Fx, Fx) {
    let alpha = clamp_alpha(alpha);
    let share_pos = theta_pow_alpha(theta, alpha);
    let share_pow = Fx::ONE - share_pos;
    (budget * share_pow, budget * share_pos)
}

/// The security floor derived from attack economics: `c_sec · (TVL/M) · r_atk`.
/// `c_sec` is the safety margin, `r_atk` the attacker's cost of capital per
/// epoch, `tvl` and `m` (circulating supply) share a unit so their ratio is
/// dimensionless. Both `c_sec` and `r_atk` are PID-controlled governance
/// inputs (§10), not fixed by this function.
pub fn security_floor(c_sec: Fx, tvl: Fx, m: Fx, r_atk: Fx) -> Fx {
    c_sec * tvl.div(m) * r_atk
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Fx, want: f64) {
        let got = a.to_f64();
        assert!((got - want).abs() < 1e-6, "got {got}, want {want}");
    }

    #[test]
    fn alpha_clamps_to_spec_bounds() {
        assert_eq!(clamp_alpha(Fx::from_ratio(1, 10)), Fx::from_ratio(3, 10));
        assert_eq!(clamp_alpha(Fx::from_ratio(9, 10)), Fx::from_ratio(7, 10));
        assert_eq!(clamp_alpha(Fx::from_ratio(1, 2)), Fx::from_ratio(1, 2));
    }

    #[test]
    fn theta_zero_all_pow() {
        let (pow, pos) = split(Fx::from_int(100), Fx::ZERO, Fx::from_ratio(1, 2));
        close(pow, 100.0);
        close(pos, 0.0);
    }

    #[test]
    fn theta_one_all_pos() {
        let (pow, pos) = split(Fx::from_int(100), Fx::ONE, Fx::from_ratio(1, 2));
        close(pow, 0.0);
        close(pos, 100.0);
    }

    #[test]
    fn alpha_half_matches_sqrt() {
        // θ^0.5 = √θ for θ ∈ (0,1); cross-check the ln/exp path against Fx::sqrt.
        for &n in &[1, 2, 3, 4, 5, 6, 7, 8, 9] {
            let theta = Fx::from_ratio(n, 10);
            let got = theta_pow_alpha(theta, Fx::from_ratio(1, 2));
            let want = theta.sqrt();
            let diff = (got.to_f64() - want.to_f64()).abs();
            assert!(diff < 1e-4, "n={n}: got {got:?}, want {want:?}");
        }
    }

    #[test]
    fn split_conserves_the_budget() {
        for &n in &[0, 1, 2, 5, 8, 10] {
            let theta = Fx::from_ratio(n, 10);
            let (pow, pos) = split(Fx::from_int(1000), theta, Fx::from_ratio(1, 2));
            close(pow + pos, 1000.0);
        }
    }

    #[test]
    fn pos_share_monotone_in_theta() {
        let alpha = Fx::from_ratio(1, 2);
        let mut prev = Fx::ZERO;
        for &n in &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            let theta = Fx::from_ratio(n, 10);
            let (_, pos) = split(Fx::from_int(1), theta, alpha);
            assert!(pos >= prev, "θ={theta:?}: pos {pos:?} < prev {prev:?}");
            prev = pos;
        }
    }

    #[test]
    fn higher_alpha_favors_pow_below_theta_one() {
        // For θ ∈ (0,1), θ^α is decreasing in α, so a higher α leaves a
        // larger PoW share for the same staking ratio.
        let theta = Fx::from_ratio(1, 2);
        let (pow_lo, _) = split(Fx::from_int(1), theta, Fx::from_ratio(3, 10));
        let (pow_hi, _) = split(Fx::from_int(1), theta, Fx::from_ratio(7, 10));
        assert!(pow_hi > pow_lo, "pow_hi {pow_hi:?} <= pow_lo {pow_lo:?}");
    }

    #[test]
    fn floor_scales_linearly_in_each_factor() {
        let base = security_floor(Fx::from_int(1), Fx::from_int(100), Fx::from_int(1000), Fx::from_ratio(1, 100));
        let double_c = security_floor(Fx::from_int(2), Fx::from_int(100), Fx::from_int(1000), Fx::from_ratio(1, 100));
        close(double_c, 2.0 * base.to_f64());

        let double_tvl = security_floor(Fx::from_int(1), Fx::from_int(200), Fx::from_int(1000), Fx::from_ratio(1, 100));
        close(double_tvl, 2.0 * base.to_f64());

        let double_ratk = security_floor(Fx::from_int(1), Fx::from_int(100), Fx::from_int(1000), Fx::from_ratio(2, 100));
        close(double_ratk, 2.0 * base.to_f64());
    }
}
