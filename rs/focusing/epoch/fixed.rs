//! Integer fixed-point for the epoch kernel ([[arithmetic]] §2, §5).
//!
//! The epoch kernel carries every quantity as a nonnegative or balanced
//! integer at a declared power-of-two scale. Every value stays below 2^63, so
//! each one *is* a canonical Goldilocks residue (`p = 2^64 − 2^32 + 1`) and the
//! arithmetic below is the fixed-point semantics of [[arithmetic]] with the
//! scale chosen per quantity:
//!
//! | quantity | scale | range |
//! |---|---|---|
//! | focus φ | 2^60 (`Σφ = 2^60`) | `[0, 2^60]` |
//! | transition / degree-normalised weight | 2^40 | `[0, 2^40]` |
//! | parameter coefficient (α, λ, μ', τ/k) | 2^32 | `[0, 2^32]` |
//! | logarithm, adjoint λ | 2^40 | `|x| < 2^50` |
//! | syntropy J, attribution score | 2^60 (nats) | `|x| < 2^72` (i128) |
//!
//! Products widen into u128 / i128 and come back with round-half-up (or
//! round-half-away-from-zero for signed values). Integer addition without
//! overflow is associative, so any summation order — any thread count, any
//! chunking — yields the same bits.

use crate::arithmetic::Fx;

/// Focus scale: φ is stored as `φ · 2^60`, `Σφ = 2^60`.
pub const PHI_BITS: u32 = 60;
/// `2^60` — the unit mass.
pub const PHI_ONE: u64 = 1 << PHI_BITS;
/// Transition-matrix scale: a column-normalised weight `w/out ∈ [0,1]` is `·2^40`.
pub const P_BITS: u32 = 40;
/// Parameter-coefficient scale (α, λ, μ/(1+μ), τ/k, quadrature nodes).
pub const Q_BITS: u32 = 32;
/// `2^32`.
pub const Q_ONE: u64 = 1 << Q_BITS;
/// Logarithm / adjoint scale.
pub const LN_BITS: u32 = 40;
/// Edge weights are shifted so the total effective stake fits below `2^62`.
pub const STAKE_BITS: u32 = 62;
/// Gradient scale: `∂J/∂A_pq` is reported as nats per weight unit `· 2^112`.
pub const GRAD_BITS: u32 = 112;

/// `round(a·b / 2^bits)` for unsigned operands (round half up).
#[inline(always)]
pub fn mul_shr(a: u64, b: u64, bits: u32) -> u64 {
    ((a as u128 * b as u128 + (1u128 << (bits - 1))) >> bits) as u64
}

/// `round(x / 2^bits)` for an unsigned wide value.
#[inline(always)]
pub fn shr_u(x: u128, bits: u32) -> u128 {
    (x + (1u128 << (bits - 1))) >> bits
}

/// `round(x / 2^bits)` for a signed wide value, half away from zero — the
/// rounding is symmetric so `shr_i(−x) = −shr_i(x)`.
#[inline(always)]
pub fn shr_i(x: i128, bits: u32) -> i128 {
    let h = 1i128 << (bits - 1);
    if x >= 0 {
        (x + h) >> bits
    } else {
        -((-x + h) >> bits)
    }
}

/// An [`Fx`] (scale 2^32) as an unsigned Q32 coefficient, negatives clamped
/// to zero.
pub fn q32(x: Fx) -> u64 {
    let v = x.to_i64_scaled(32);
    if v < 0 { 0 } else { v as u64 }
}

/// Signed division rounded half away from zero (`den > 0`).
#[inline]
pub fn div_round_i(num: i128, den: i128) -> i128 {
    debug_assert!(den > 0);
    let h = den / 2;
    if num >= 0 {
        (num + h) / den
    } else {
        -((-num + h) / den)
    }
}

/// `num · 2^s / den`, truncated toward zero, without a 256-bit intermediate:
/// `(num/den)·2^s + ((num mod den)·2^s)/den`. Requires `den < 2^64`,
/// `s ≤ 63` and `|num/den| < 2^(126−s)`; saturates beyond that.
#[inline]
pub fn mul_shift_div(num: i128, s: u32, den: u64) -> i128 {
    debug_assert!(den > 0 && s <= 63);
    let d = den as i128;
    let (q, r) = (num / d, num % d);
    let lim = 1i128 << (126 - s);
    if q >= lim {
        return i128::MAX;
    }
    if q <= -lim {
        return i128::MIN;
    }
    (q << s) + (r << s) / d
}

/// `a·b / 2^s` with a 256-bit intermediate, magnitude truncated (toward zero),
/// saturating at the i128 range. `s < 128`.
pub fn mul_wide_shr(a: i128, b: i128, s: u32) -> i128 {
    let neg = (a < 0) != (b < 0);
    let (ua, ub) = (a.unsigned_abs(), b.unsigned_abs());
    let (a1, a0) = (ua >> 64, ua & u64::MAX as u128);
    let (b1, b0) = (ub >> 64, ub & u64::MAX as u128);
    let (p00, p01, p10, p11) = (a0 * b0, a0 * b1, a1 * b0, a1 * b1);
    let mid = (p00 >> 64) + (p01 & u64::MAX as u128) + (p10 & u64::MAX as u128);
    let lo = (p00 & u64::MAX as u128) | (mid << 64);
    let hi = p11 + (p01 >> 64) + (p10 >> 64) + (mid >> 64);
    let (rhi, rlo) = if s == 0 {
        (hi, lo)
    } else {
        (hi >> s, (lo >> s) | (hi << (128 - s)))
    };
    if rhi != 0 || rlo > i128::MAX as u128 {
        return if neg { i128::MIN } else { i128::MAX };
    }
    if neg { -(rlo as i128) } else { rlo as i128 }
}

/// `⌊a·b / 2^s⌋` with a 256-bit intermediate (floor, toward −∞),
/// saturating at the i128 range. `0 < s < 128`. The rounding of the path
/// gradient's weighted sum ([[epoch]] §4).
pub fn mul_wide_floor(a: i128, b: i128, s: u32) -> i128 {
    let t = mul_wide_shr(a, b, s);
    let neg = (a < 0) != (b < 0) && a != 0 && b != 0;
    if !neg || t == i128::MIN {
        return t;
    }
    // exact iff the low s bits of |a|·|b| are zero
    let (ua, ub) = (a.unsigned_abs(), b.unsigned_abs());
    let low = if s >= 64 {
        // low s bits zero iff the lowest set bits of ua and ub sum to ≥ s
        ua.trailing_zeros() + ub.trailing_zeros() < s
    } else {
        ua.wrapping_mul(ub) & ((1u128 << s) - 1) != 0
    };
    if low { t - 1 } else { t }
}

// ── natural logarithm, integer only ──────────────────────────────────────

/// `2·atanh(z)` at scale 2^62 for `z = num/den ≤ 1/3`, by its odd power
/// series (each term at most `(1/9)^k`; 40 terms reach far below 2^-62).
const fn atanh2_q62(num: u128, den: u128) -> u128 {
    let z = (num << 62) / den;
    let z2 = (z * z) >> 62;
    let mut term = z;
    let mut sum = 0u128;
    let mut k = 0u128;
    while k < 40 {
        sum += term / (2 * k + 1);
        term = (term * z2) >> 62;
        k += 1;
    }
    2 * sum
}

/// `ln 2` at scale 2^62: `ln 2 = 2·atanh(1/3)`.
const LN2_Q62: u128 = atanh2_q62(1, 3);

/// `ln(1 + i/256)` at scale 2^62 for `i = 0..=256`: `ln(1+x) = 2·atanh(x/(2+x))`.
const LN_TABLE: [u128; 257] = {
    let mut t = [0u128; 257];
    let mut i = 1;
    while i <= 256 {
        t[i] = atanh2_q62(i as u128, 512 + i as u128);
        i += 1;
    }
    t
};

/// `ln(v)` for an integer `v ≥ 1`, at scale 2^40 (`ln_q40(0)` = 0 by
/// convention; callers never feed it zero mass). Range reduction
/// `v = 2^e·(1 + i/256)·(1 + δ)` with a compile-time table for the middle
/// factor and a 5-term series for `ln(1+δ)`, `δ < 2^-8`: absolute error below
/// 2^-41. Pure integer arithmetic.
pub fn ln_q40(v: u64) -> i64 {
    if v == 0 {
        return 0;
    }
    let e = 63 - v.leading_zeros() as u64;
    let y = v << (63 - e); // mantissa in [2^63, 2^64), Q63
    let i = ((y >> 55) & 0xFF) as usize;
    let r = y & ((1u64 << 55) - 1);
    // δ = r / (2^55·(256+i)), at scale 2^62.
    let d = ((r << 7) / (256 + i as u64)) as i128;
    let d2 = (d * d) >> 62;
    let d3 = (d2 * d) >> 62;
    let d4 = (d3 * d) >> 62;
    let d5 = (d4 * d) >> 62;
    let ln1p = d - d2 / 2 + d3 / 3 - d4 / 4 + d5 / 5;
    let total = e as i128 * LN2_Q62 as i128 + LN_TABLE[i] as i128 + ln1p;
    shr_i(total, 62 - LN_BITS) as i64
}

/// `ln 2` at scale 2^40.
pub fn ln2_q40() -> i64 {
    shr_i(LN2_Q62 as i128, 62 - LN_BITS) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ln_matches_reference_within_two_ulp() {
        let ulp = 1.0 / (1u64 << LN_BITS) as f64;
        for &v in &[
            1u64,
            2,
            3,
            7,
            255,
            256,
            257,
            1000,
            65_537,
            1 << 30,
            (1 << 40) + 12345,
            PHI_ONE / 3_143_650,
            PHI_ONE,
            u64::MAX >> 1,
            u64::MAX,
        ] {
            let got = ln_q40(v) as f64 * ulp;
            let want = (v as f64).ln();
            assert!(
                (got - want).abs() < 2.0 * ulp + want.abs() * 1e-15,
                "ln({v}) = {got}, want {want}"
            );
        }
    }

    #[test]
    fn ln_is_monotone_on_a_sweep() {
        let mut prev = i64::MIN;
        let mut v = 1u64;
        while v < u64::MAX / 3 {
            let l = ln_q40(v);
            assert!(l >= prev, "ln not monotone at {v}");
            prev = l;
            v = v + v / 7 + 1;
        }
    }

    #[test]
    fn wide_multiply_matches_narrow_and_saturates() {
        for &(a, b, s) in &[
            (3i128, 5i128, 0u32),
            (-7, 9, 1),
            (1 << 70, 1 << 70, 80),
            (-(1 << 90), 3 << 60, 100),
            (i128::MAX, 2, 1),
        ] {
            let want = (a as f64) * (b as f64) / 2f64.powi(s as i32);
            let got = mul_wide_shr(a, b, s) as f64;
            assert!(
                (got - want).abs() <= want.abs() * 1e-12 + 1.0,
                "{a}·{b}>>{s}"
            );
        }
        assert_eq!(mul_wide_shr(i128::MAX, i128::MAX, 0), i128::MAX);
        assert_eq!(mul_wide_shr(i128::MIN + 1, i128::MAX, 0), i128::MIN);
        assert_eq!(mul_shift_div(-7, 4, 2), -56);
        assert_eq!(mul_wide_floor(-3, 1, 1), -2);
        assert_eq!(mul_wide_floor(-4, 1, 1), -2);
        assert_eq!(mul_wide_floor(3, 1, 1), 1);
        assert_eq!(mul_wide_floor(-(1 << 100), 3, 32), -(3 << 68));
        assert_eq!(
            mul_wide_floor(-((1 << 100) + 1), 1 << 31, 32),
            -(1 << 99) - 1
        );
        assert_eq!(mul_shift_div(1 << 100, 20, 1 << 40), 1 << 80);
    }

    #[test]
    fn signed_shift_is_symmetric() {
        for x in [-5i128, -4, -3, -1, 0, 1, 3, 4, 5, 1 << 40, -(1 << 40) - 7] {
            assert_eq!(shr_i(-x, 2), -shr_i(x, 2));
        }
        assert_eq!(div_round_i(-7, 2), -4);
        assert_eq!(div_round_i(7, 2), 4);
    }
}
