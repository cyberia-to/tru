//! Derived measures over the focus distribution φ* (`docs/terms/`).
//!
//! [`syntropy`] is the quantity tru exists to grow; [`cyberank`] is the
//! per-particle focus other repos read; [`telemetry`] bundles the cheap
//! per-epoch monitors ([[tri-kernel]] §6.3). All fixed-point over the field.

use crate::arithmetic::Fx;

use super::focusing::{contraction, derived_steps, FocusingGraph, FocusingParams, FocusingResult};

/// Syntropy `J(φ*) = Σ_j φ*(j)·ln(|V|·φ*(j)) = D_KL(φ* ‖ u)` — the network
/// order in nats. Zero at the uniform distribution, positive otherwise.
pub fn syntropy(focus: &[Fx]) -> Fx {
    let n = focus.len();
    if n == 0 {
        return Fx::ZERO;
    }
    let nfx = Fx::from_int(n as i64);
    let mut j = Fx::ZERO;
    for &p in focus {
        if p > Fx::ZERO {
            j = j + p * (nfx * p).ln();
        }
    }
    j
}

/// Shannon entropy `H(φ*) = −Σ_j φ*(j)·ln φ*(j)` (nats). Related to syntropy by
/// `H = ln|V| − J`.
pub fn entropy(focus: &[Fx]) -> Fx {
    let mut h = Fx::ZERO;
    for &p in focus {
        if p > Fx::ZERO {
            h = h - p * p.ln();
        }
    }
    h
}

/// `cyberank(p) = φ*(p)`: a particle's focus by its hash (zero if absent).
pub fn cyberank(g: &FocusingGraph, result: &FocusingResult, particle: &[u8; 32]) -> Fx {
    match g.node_ids().iter().position(|h| h == particle) {
        Some(i) => result.focus[i],
        None => Fx::ZERO,
    }
}

/// Per-epoch telemetry: the cheap monitors of [[tri-kernel]] §6.3.
pub struct Telemetry {
    /// |V| — particles touched this epoch.
    pub particles: usize,
    /// Syntropy J(φ*) — negentropy, the vital sign.
    pub syntropy: Fx,
    /// Shannon entropy H(φ*).
    pub entropy: Fx,
    /// Algebraic connectivity λ₂ (spectral gap proxy).
    pub lambda_2: Fx,
    /// Composite contraction κ.
    pub kappa: Fx,
    /// Derived step count T(ε).
    pub steps: usize,
}

/// Assemble the telemetry for one focusing run.
pub fn telemetry(g: &FocusingGraph, result: &FocusingResult, p: &FocusingParams) -> Telemetry {
    Telemetry {
        particles: g.n(),
        syntropy: result.syntropy,
        entropy: entropy(&result.focus),
        lambda_2: g.lambda_2(),
        kappa: contraction(g, p),
        steps: derived_steps(g, p),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::focusing::{compute_focusing, FocusingGraph, Karma, Link};

    fn hash(b: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    fn link(from: u8, to: u8, amount: u128) -> Link {
        Link::stake(hash(from), hash(to), amount)
    }

    #[test]
    fn syntropy_zero_at_uniform() {
        let u = vec![Fx::from_ratio(1, 4); 4];
        assert!(syntropy(&u).to_f64().abs() < 1e-4, "J(uniform) should be 0");
    }

    #[test]
    fn syntropy_nonnegative_and_grows_with_concentration() {
        let uniform = vec![Fx::from_ratio(1, 4); 4];
        let peaked = vec![Fx::from_ratio(7, 10), Fx::from_ratio(1, 10), Fx::from_ratio(1, 10), Fx::from_ratio(1, 10)];
        let ju = syntropy(&uniform).to_f64();
        let jp = syntropy(&peaked).to_f64();
        assert!(ju >= -1e-4 && jp >= -1e-4, "syntropy ≥ 0 (J_u={ju}, J_p={jp})");
        assert!(jp > ju + 0.1, "a peaked distribution has more syntropy ({jp} vs {ju})");
    }

    #[test]
    fn cyberank_reads_focus_by_particle() {
        let g = FocusingGraph::build(vec![link(1, 2, 100), link(2, 3, 50), link(3, 1, 200)], &Karma::none());
        let r = compute_focusing(&g, &crate::focusing::FocusingParams::default());
        // present particle → its focus; a total-mass check ties it to φ*
        let sum: f64 = [1u8, 2, 3].iter().map(|&b| cyberank(&g, &r, &hash(b)).to_f64()).sum();
        assert!((sum - 1.0).abs() < 1e-6, "cyberank over all particles sums to 1, got {sum}");
        assert_eq!(cyberank(&g, &r, &hash(99)), Fx::ZERO, "absent particle has zero cyberank");
    }

    #[test]
    fn result_carries_syntropy() {
        let g = FocusingGraph::build(vec![link(1, 2, 100), link(2, 3, 50), link(4, 1, 300)], &Karma::none());
        let r = compute_focusing(&g, &crate::focusing::FocusingParams::default());
        assert!(r.syntropy.to_f64() >= -1e-4, "emitted syntropy ≥ 0");
        assert_eq!(r.syntropy.raw(), syntropy(&r.focus).raw(), "emitted J matches recomputed J");
    }
}
