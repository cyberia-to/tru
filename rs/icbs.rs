//! ICBS — inversely coupled bonding surface (`docs/explanation/epistemic-markets.md`).
//!
//! The market layer of the [[cybergraph]]: every [[cyberlink]] can carry an
//! ICBS position, `C(s_YES, s_NO) = λ√(s_YES² + s_NO²)`, self-scaling
//! liquidity over two outcome-token reserves. This module computes the cost
//! surface and the prices that emerge as its partial derivatives. It holds
//! no trading state and settles nothing — `strong-truthfulness.md` proves
//! the market is not a truth-scorer and routes truthfulness through the
//! serum ([[truth_scoring]]) instead; this is the liquidity skin only.
//!
//! Fixed-point over the Goldilocks field ([[arithmetic]]), no floats.

use crate::arithmetic::Fx;

/// Token supplies on the two outcome sides of one edge's ICBS market.
#[derive(Clone, Copy, Debug)]
pub struct Reserves {
    pub s_yes: Fx,
    pub s_no: Fx,
}

fn hypot(a: Fx, b: Fx) -> Fx {
    (a * a + b * b).sqrt()
}

/// The cost function `C(s_YES, s_NO) = λ√(s_YES² + s_NO²)` — the price of
/// reaching this reserve state from the origin, and by self-scaling design
/// the total value a solvent market holds at these reserves.
pub fn cost(r: Reserves, lambda: Fx) -> Fx {
    lambda * hypot(r.s_yes, r.s_no)
}

/// Spot price of YES, the partial derivative
/// `∂C/∂s_YES = λ·s_YES/√(s_YES²+s_NO²)`. Zero reserves price at zero
/// (the safe `Fx` division-by-zero convention — an empty market has no
/// spot price yet, not an undefined one).
pub fn price_yes(r: Reserves, lambda: Fx) -> Fx {
    lambda * r.s_yes.div(hypot(r.s_yes, r.s_no))
}

/// Spot price of NO, the partial derivative `∂C/∂s_NO`.
pub fn price_no(r: Reserves, lambda: Fx) -> Fx {
    lambda * r.s_no.div(hypot(r.s_yes, r.s_no))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(yes: i64, no: i64) -> Reserves {
        Reserves {
            s_yes: Fx::from_int(yes),
            s_no: Fx::from_int(no),
        }
    }

    #[test]
    fn zero_reserves_price_at_zero() {
        let lambda = Fx::ONE;
        assert_eq!(cost(r(0, 0), lambda).to_f64(), 0.0);
        assert_eq!(price_yes(r(0, 0), lambda).to_f64(), 0.0);
        assert_eq!(price_no(r(0, 0), lambda).to_f64(), 0.0);
    }

    #[test]
    fn cost_grows_with_traded_volume() {
        let lambda = Fx::ONE;
        let c_small = cost(r(1, 1), lambda).to_f64();
        let c_large = cost(r(10, 10), lambda).to_f64();
        assert!(
            c_large > c_small,
            "TVL should grow with reserves: {c_small} vs {c_large}"
        );
    }

    #[test]
    fn prices_stay_on_the_lambda_circle() {
        // p_YES^2 + p_NO^2 == lambda^2 — the geometric identity the iso-cost
        // circle guarantees, not the probability simplex.
        let lambda = Fx::from_int(2);
        for (yes, no) in [(1, 1), (5, 1), (1, 5), (7, 3), (100, 1)] {
            let py = price_yes(r(yes, no), lambda).to_f64();
            let pn = price_no(r(yes, no), lambda).to_f64();
            let sum_sq = py * py + pn * pn;
            let target = lambda.to_f64() * lambda.to_f64();
            assert!(
                (sum_sq - target).abs() < 1e-3,
                "({yes},{no}): p_YES^2+p_NO^2 = {sum_sq}, want {target}"
            );
        }
    }

    #[test]
    fn prices_bounded_by_lambda() {
        let lambda = Fx::from_int(3);
        for (yes, no) in [(1, 0), (0, 1), (1, 1), (1000, 1), (1, 1000)] {
            let py = price_yes(r(yes, no), lambda).to_f64();
            let pn = price_no(r(yes, no), lambda).to_f64();
            assert!((0.0..=lambda.to_f64() + 1e-6).contains(&py), "p_YES={py}");
            assert!((0.0..=lambda.to_f64() + 1e-6).contains(&pn), "p_NO={pn}");
        }
    }

    #[test]
    fn symmetric_reserves_split_price_evenly() {
        let lambda = Fx::ONE;
        let py = price_yes(r(9, 9), lambda).to_f64();
        let pn = price_no(r(9, 9), lambda).to_f64();
        assert!((py - pn).abs() < 1e-3, "p_YES={py}, p_NO={pn}");
        // On the circle at 45°: p = lambda / sqrt(2).
        let expect = lambda.to_f64() / std::f64::consts::SQRT_2;
        assert!((py - expect).abs() < 1e-3, "p_YES={py}, want {expect}");
    }

    #[test]
    fn buying_yes_raises_its_price_and_suppresses_no() {
        // Inverse coupling: ∂p_YES/∂s_NO < 0 and ∂p_NO/∂s_YES < 0, checked
        // as a finite difference — moving reserves toward YES.
        let lambda = Fx::ONE;
        let before = r(5, 5);
        let after = r(9, 5); // buy YES: s_YES rises, s_NO unchanged.

        let py_before = price_yes(before, lambda).to_f64();
        let py_after = price_yes(after, lambda).to_f64();
        let pn_before = price_no(before, lambda).to_f64();
        let pn_after = price_no(after, lambda).to_f64();

        assert!(
            py_after > py_before,
            "buying YES should raise its own price: {py_before} -> {py_after}"
        );
        assert!(
            pn_after < pn_before,
            "buying YES should suppress NO's price: {pn_before} -> {pn_after}"
        );
    }
}
