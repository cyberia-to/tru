//! Fixed-point arithmetic over the Goldilocks field (`specs/arithmetic.md`).
//!
//! tru computes rationals — probabilities, focus, weights — that must be
//! ranked, compared, summed, and logged, yet stay deterministic and provable.
//! Raw [`nebu::Goldilocks`] has no order or magnitude; float has no
//! determinism or proof. Fixed-point is the one representation with both: a
//! rational `x` is carried as the field element `round(x · 2^FRAC_BITS)`.
//!
//! [`Fx`] is the thin layer over `nebu::Goldilocks` that the tri-kernel and
//! CT-0 compute in. The field itself (add/mul/inv/…) is nebu's; `Fx` adds only
//! the scale: rescale-on-multiply, fixed-point div/recip/sqrt, and order.
//!
//! No floating point appears on any deterministic path. `to_f64` is display /
//! offline-analysis only and is never part of a computation tru proves.

use core::cmp::Ordering;
use core::ops::{Add, Mul, Neg, Sub};

use nebu::field::P;
use nebu::Goldilocks;

/// Number of fractional bits: a value `x` is stored as `x · 2^FRAC_BITS`.
///
/// At 32 bits the resolution is `2^-32 ≈ 2.3e-10` and the representable
/// magnitude is roughly `[2^-32, 2^31]`; a value whose integer part exceeds
/// `2^31` overflows and wraps mod p (the finite dynamic range of any
/// fixed-point scheme — block-scale if a distribution needs more).
pub const FRAC_BITS: u32 = 32;

/// The scale `Σ = 2^FRAC_BITS` as an i128 (the multiply widens into i128).
const SCALE: i128 = 1i128 << FRAC_BITS;

/// Half the prime, the boundary of the balanced (signed) residue range.
const HALF_P: u64 = P / 2;

/// A fixed-point rational carried as a Goldilocks field element.
#[derive(Clone, Copy, Eq)]
#[repr(transparent)]
pub struct Fx(Goldilocks);

// ── construction ──────────────────────────────────────────────────────

impl Fx {
    pub const ZERO: Fx = Fx(Goldilocks::ZERO);

    /// The scaled representative of 1.0 (`2^FRAC_BITS`).
    pub const ONE: Fx = Fx(Goldilocks::new(1u64 << FRAC_BITS));

    /// Exact integer `n` as a fixed-point value.
    #[inline]
    pub fn from_int(n: i64) -> Fx {
        Fx(from_signed((n as i128) << FRAC_BITS))
    }

    /// The ratio `num / den`, rounded to the nearest representable value.
    /// `den == 0` yields [`Fx::ZERO`].
    #[inline]
    pub fn from_ratio(num: i64, den: i64) -> Fx {
        if den == 0 {
            return Fx::ZERO;
        }
        Fx(from_signed(round_div((num as i128) << FRAC_BITS, den as i128)))
    }

    /// The underlying field element — the canonical storage / proof form.
    #[inline]
    pub fn raw(self) -> Goldilocks {
        self.0.canonicalize()
    }

    /// Rebuild from a raw scaled field element (inverse of [`Fx::raw`]).
    #[inline]
    pub fn from_raw(g: Goldilocks) -> Fx {
        Fx(g)
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    /// The signed scaled integer this value represents (in `(-p/2, p/2)`).
    #[inline]
    fn signed(self) -> i128 {
        to_signed(self.0)
    }

    /// Approximate the value as `f64`. Display and offline analysis ONLY —
    /// never a step in a computation tru proves.
    #[inline]
    pub fn to_f64(self) -> f64 {
        self.signed() as f64 / SCALE as f64
    }
}

// ── field-scale ops: add / sub / neg preserve the scale directly ──────

impl Add for Fx {
    type Output = Fx;
    #[inline]
    fn add(self, rhs: Fx) -> Fx {
        Fx(self.0 + rhs.0)
    }
}

impl Sub for Fx {
    type Output = Fx;
    #[inline]
    fn sub(self, rhs: Fx) -> Fx {
        Fx(self.0 - rhs.0)
    }
}

impl Neg for Fx {
    type Output = Fx;
    #[inline]
    fn neg(self) -> Fx {
        Fx(self.0.field_neg())
    }
}

// ── scale-changing ops: multiply must rescale by Σ ────────────────────

impl Mul for Fx {
    type Output = Fx;
    /// Two values at scale Σ produce scale Σ²; widen to i128, multiply,
    /// truncate the extra Σ (round half away from zero), reduce mod p.
    /// A field multiply would be wrong here — `·inv(Σ)` is a residue, not a
    /// truncated rational.
    #[inline]
    fn mul(self, rhs: Fx) -> Fx {
        let prod = self.signed() * rhs.signed();
        let bias = 1i128 << (FRAC_BITS - 1);
        let scaled = if prod >= 0 { (prod + bias) >> FRAC_BITS } else { -(((-prod) + bias) >> FRAC_BITS) };
        Fx(from_signed(scaled))
    }
}

impl Fx {
    /// `self / rhs`, rounded to nearest. `rhs == 0` yields [`Fx::ZERO`]
    /// (safe degradation; the tri-kernel normalizes so denominators are
    /// positive). Use [`Fx::checked_div`] to detect division by zero.
    #[inline]
    pub fn div(self, rhs: Fx) -> Fx {
        self.checked_div(rhs).unwrap_or(Fx::ZERO)
    }

    #[inline]
    pub fn checked_div(self, rhs: Fx) -> Option<Fx> {
        let d = rhs.signed();
        if d == 0 {
            return None;
        }
        Some(Fx(from_signed(round_div(self.signed() << FRAC_BITS, d))))
    }

    /// Reciprocal `1 / self` (fixed-point `2^{2·FRAC_BITS} / A`).
    #[inline]
    pub fn recip(self) -> Fx {
        Fx::ONE.div(self)
    }

    /// Non-negative square root, `floor`-rounded. Negative input yields
    /// [`Fx::ZERO`] (focus and its derivatives are nonnegative).
    #[inline]
    pub fn sqrt(self) -> Fx {
        let a = self.signed();
        if a <= 0 {
            return Fx::ZERO;
        }
        // √(A/Σ) as fixed-point = √(A·Σ).
        Fx(from_signed(isqrt_u128((a as u128) << FRAC_BITS) as i128))
    }
}

// ── order (on the signed representative, not the residue) ─────────────

impl PartialEq for Fx {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Ord for Fx {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.signed().cmp(&other.signed())
    }
}

impl PartialOrd for Fx {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl core::fmt::Debug for Fx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Fx({})", self.to_f64())
    }
}

// ── residue ⇄ signed integer, integer sqrt ────────────────────────────

/// Canonical residue `[0, p)` → balanced signed integer `(-p/2, p/2)`.
#[inline]
fn to_signed(g: Goldilocks) -> i128 {
    let v = g.as_u64();
    if v > HALF_P {
        v as i128 - P as i128
    } else {
        v as i128
    }
}

/// Signed integer → canonical field residue (reduces mod p, handles sign).
#[inline]
fn from_signed(x: i128) -> Goldilocks {
    let p = P as i128;
    let mut r = x % p;
    if r < 0 {
        r += p;
    }
    Goldilocks::new(r as u64)
}

/// Round-half-away-from-zero signed division.
#[inline]
fn round_div(num: i128, den: i128) -> i128 {
    let half = den.abs() / 2;
    let bias = if (num >= 0) == (den > 0) { half } else { -half };
    (num + bias) / den
}

/// Floor integer square root of a u128, via Newton's method.
#[inline]
fn isqrt_u128(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = 1u128 << ((128 - n.leading_zeros() + 1) / 2);
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ULP: f64 = 1.0 / (SCALE as f64);

    fn close(a: Fx, want: f64) {
        let got = a.to_f64();
        assert!((got - want).abs() <= 4.0 * ULP, "got {got}, want {want}");
    }

    #[test]
    fn round_trips_encode_decode() {
        for &(n, d) in &[(1, 3), (2, 7), (-5, 8), (100, 1), (0, 1), (1, 1_000_000)] {
            close(Fx::from_ratio(n, d), n as f64 / d as f64);
        }
        assert_eq!(Fx::from_int(42).to_f64(), 42.0);
        assert_eq!(Fx::from_int(-7).to_f64(), -7.0);
    }

    #[test]
    fn add_sub_neg_exact_when_representable() {
        // dyadic values are exact: 1/4 + 1/2 = 3/4
        assert_eq!((Fx::from_ratio(1, 4) + Fx::from_ratio(1, 2)).to_f64(), 0.75);
        assert_eq!((-Fx::from_int(3)).to_f64(), -3.0);
        assert_eq!(Fx::from_ratio(1, 2) - Fx::from_ratio(3, 4), Fx::from_ratio(-1, 4));
        close(Fx::from_ratio(3, 10) - Fx::from_ratio(1, 2), -0.2);
    }

    #[test]
    fn mul_rescales_within_one_ulp() {
        assert_eq!((Fx::from_ratio(1, 2) * Fx::from_ratio(1, 2)).to_f64(), 0.25);
        close(Fx::from_ratio(1, 10) * Fx::from_ratio(1, 10), 0.01);
        close(Fx::from_int(-3) * Fx::from_ratio(1, 4), -0.75);
        // scale invariant: x·1 == x
        let x = Fx::from_ratio(7, 13);
        assert_eq!((x * Fx::ONE).raw(), x.raw());
    }

    #[test]
    fn div_recip_sqrt() {
        assert_eq!((Fx::from_ratio(3, 4).div(Fx::from_ratio(1, 4))).to_f64(), 3.0);
        close(Fx::from_int(1).div(Fx::from_int(3)) * Fx::from_int(3), 1.0);
        assert_eq!(Fx::from_ratio(1, 4).recip().to_f64(), 4.0);
        assert_eq!(Fx::from_ratio(1, 4).sqrt().to_f64(), 0.5);
        close(Fx::from_int(2).sqrt(), 2.0_f64.sqrt());
        // division by zero degrades to zero; checked form reports it
        assert_eq!(Fx::from_int(5).div(Fx::ZERO), Fx::ZERO);
        assert_eq!(Fx::from_int(5).checked_div(Fx::ZERO), None);
    }

    #[test]
    fn ordering_respects_sign_and_magnitude() {
        let mut v = [Fx::from_ratio(1, 2), Fx::from_ratio(-1, 5), Fx::from_int(3), Fx::ZERO, Fx::from_ratio(1, 10)];
        v.sort();
        let got: Vec<f64> = v.iter().map(|x| x.to_f64()).collect();
        assert!(got.windows(2).all(|w| w[0] <= w[1]), "not sorted: {got:?}");
        assert!(Fx::from_ratio(-1, 5) < Fx::ZERO);
        assert!(Fx::ZERO < Fx::from_ratio(1, 10));
    }

    #[test]
    fn field_element_round_trips() {
        let x = Fx::from_ratio(123, 456);
        assert_eq!(Fx::from_raw(x.raw()), x);
    }

    #[test]
    fn associative_add_and_distributive_scale() {
        // (a+b)-b ≈ a within rounding, over a spread of values.
        for &(n, d) in &[(1, 3), (7, 9), (-4, 11), (5, 2)] {
            let a = Fx::from_ratio(n, d);
            let b = Fx::from_ratio(2, 7);
            close((a + b) - b, n as f64 / d as f64);
        }
    }
}
