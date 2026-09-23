//! A generic PID controller over [`Fx`] (`specs/rewards.md` §10).
//!
//! §10 names three parameters (`α`, the security floor, `β`) that "follow
//! PID control on observable signals (security margin, fee coverage,
//! efficiency differential), so the system measures and adapts rather than
//! predicts" — but does not fix what those signals compute to, or the
//! gains. This module is the control primitive alone: a generic, clamped,
//! anti-windup PID loop over fixed-point error. Deciding what error each
//! parameter tracks, and its gains, is the next slice per-parameter — this
//! is the mechanism all three will share.
//!
//! No floating point: `Fx` carries the error, integral, derivative and gains,
//! matching `specs/arithmetic.md`.

use crate::arithmetic::Fx;

/// A clamped PID controller: `output = clamp(kp·e + ki·∫e + kd·Δe, min, max)`.
///
/// Integral anti-windup: the integral only accumulates while the
/// unclamped output is within bounds, so a controller saturated at its
/// output limit does not keep winding up an error it cannot act on faster.
pub struct Pid {
    kp: Fx,
    ki: Fx,
    kd: Fx,
    output_min: Fx,
    output_max: Fx,
    integral: Fx,
    prev_error: Option<Fx>,
}

impl Pid {
    /// `output_min` must not exceed `output_max`; gains may be zero (a
    /// zero `ki`/`kd` degrades to P-only / PI, the usual way to build up
    /// from a proportional controller).
    pub fn new(kp: Fx, ki: Fx, kd: Fx, output_min: Fx, output_max: Fx) -> Self {
        Self { kp, ki, kd, output_min, output_max, integral: Fx::ZERO, prev_error: None }
    }

    /// Advance the controller by one step given the current error
    /// (`setpoint - measurement`, the caller's convention) and return the
    /// clamped control output. `dt` scales the integral and derivative
    /// terms; pass `Fx::ONE` for a fixed-cadence loop (one epoch = one step).
    pub fn step(&mut self, error: Fx, dt: Fx) -> Fx {
        let candidate_integral = self.integral + error * dt;
        let derivative = match self.prev_error {
            Some(prev) if !dt.is_zero() => (error - prev).div(dt),
            _ => Fx::ZERO,
        };
        self.prev_error = Some(error);

        let unclamped = self.kp * error + self.ki * candidate_integral + self.kd * derivative;
        let clamped = self.clamp(unclamped);

        // Clamped-integrator anti-windup: only commit the integral step
        // when the unclamped output was already within bounds. While
        // saturated, accumulating more of the same-direction error would
        // only deepen the saturation and cause overshoot on the way back.
        if clamped == unclamped {
            self.integral = candidate_integral;
        }

        clamped
    }

    fn clamp(&self, x: Fx) -> Fx {
        if x < self.output_min {
            self.output_min
        } else if x > self.output_max {
            self.output_max
        } else {
            x
        }
    }

    /// Reset accumulated integral and derivative history — e.g. after a
    /// governance-approved setpoint change, so the old error history does
    /// not bias the next step.
    pub fn reset(&mut self) {
        self.integral = Fx::ZERO;
        self.prev_error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(n: i64, d: i64) -> Fx {
        Fx::from_ratio(n, d)
    }

    #[test]
    fn proportional_only_scales_the_error() {
        let mut pid = Pid::new(fx(1, 2), Fx::ZERO, Fx::ZERO, Fx::from_int(-100), Fx::from_int(100));
        let out = pid.step(Fx::from_int(10), Fx::ONE);
        assert_eq!(out, Fx::from_int(5));
    }

    #[test]
    fn integral_accumulates_across_steps() {
        let mut pid = Pid::new(Fx::ZERO, fx(1, 1), Fx::ZERO, Fx::from_int(-100), Fx::from_int(100));
        let first = pid.step(Fx::from_int(1), Fx::ONE);
        let second = pid.step(Fx::from_int(1), Fx::ONE);
        assert_eq!(first, Fx::from_int(1));
        assert_eq!(second, Fx::from_int(2));
    }

    #[test]
    fn derivative_reacts_to_the_change_in_error() {
        let mut pid = Pid::new(Fx::ZERO, Fx::ZERO, Fx::ONE, Fx::from_int(-100), Fx::from_int(100));
        let _ = pid.step(Fx::from_int(0), Fx::ONE);
        let second = pid.step(Fx::from_int(5), Fx::ONE);
        assert_eq!(second, Fx::from_int(5));
    }

    #[test]
    fn first_step_has_no_derivative_term() {
        let mut pid = Pid::new(Fx::ZERO, Fx::ZERO, Fx::ONE, Fx::from_int(-100), Fx::from_int(100));
        let out = pid.step(Fx::from_int(7), Fx::ONE);
        assert_eq!(out, Fx::ZERO);
    }

    #[test]
    fn output_clamps_to_bounds() {
        let mut pid = Pid::new(Fx::from_int(10), Fx::ZERO, Fx::ZERO, Fx::ZERO, Fx::from_int(1));
        let out = pid.step(Fx::from_int(5), Fx::ONE);
        assert_eq!(out, Fx::from_int(1));

        let out_low = pid.step(Fx::from_int(-5), Fx::ONE);
        assert_eq!(out_low, Fx::ZERO);
    }

    #[test]
    fn saturated_integral_does_not_wind_up_further() {
        // kp alone already saturates the output; the integral must not
        // keep accumulating the same-direction error while saturated,
        // or the controller would overshoot on the way back down.
        let mut pid = Pid::new(Fx::from_int(10), Fx::ONE, Fx::ZERO, Fx::ZERO, Fx::from_int(1));
        pid.step(Fx::from_int(5), Fx::ONE);
        pid.step(Fx::from_int(5), Fx::ONE);
        pid.step(Fx::from_int(5), Fx::ONE);

        assert_eq!(pid.integral, Fx::ZERO, "integral must not accumulate while saturated");
    }

    #[test]
    fn reset_clears_integral_and_derivative_history() {
        let mut pid = Pid::new(Fx::ZERO, Fx::ONE, Fx::ONE, Fx::from_int(-100), Fx::from_int(100));
        pid.step(Fx::from_int(3), Fx::ONE);
        pid.reset();

        let out = pid.step(Fx::from_int(3), Fx::ONE);
        // With integral and prev_error cleared, this behaves exactly like
        // the very first step: ki term is one step's worth, kd term is zero.
        assert_eq!(out, Fx::from_int(3));
    }

    #[test]
    fn zero_dt_skips_the_derivative_term_without_dividing_by_zero() {
        let mut pid = Pid::new(Fx::ZERO, Fx::ZERO, Fx::ONE, Fx::from_int(-100), Fx::from_int(100));
        pid.step(Fx::from_int(1), Fx::ONE);
        let out = pid.step(Fx::from_int(9), Fx::ZERO);
        assert_eq!(out, Fx::ZERO);
    }
}
