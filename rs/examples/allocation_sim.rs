//! Simulation for `specs/rewards.md` §10 — the PoW/PoS allocation curve and
//! the attack-economics security floor. Run with `cargo run --example allocation_sim`.

use tru::allocation::{security_floor, split};
use tru::Fx;

fn pct(x: Fx) -> String {
    format!("{:.2}%", x.to_f64() * 100.0)
}

fn main() {
    println!("-- R_PoW / R_PoS split of a unit security budget B, by staking ratio θ --");
    for &(alpha_n, alpha_d, label) in &[(3, 10, "α=0.3 (PoW-favoring bound)"), (1, 2, "α=0.5 (neutral prior)"), (7, 10, "α=0.7 (PoS-favoring bound)")] {
        println!("{label}:");
        for theta_pct in [0, 10, 25, 50, 75, 90, 100] {
            let theta = Fx::from_ratio(theta_pct, 100);
            let alpha = Fx::from_ratio(alpha_n, alpha_d);
            let (pow, pos) = split(Fx::ONE, theta, alpha);
            println!(
                "  θ={theta_pct:>3}%  R_PoW={:<8} R_PoS={:<8}",
                pct(pow),
                pct(pos)
            );
        }
    }

    println!();
    println!("-- security floor = c_sec · (TVL/M) · r_atk, per-epoch attacker cost of capital r_atk = 1% --");
    let r_atk = Fx::from_ratio(1, 100);
    for &c_sec_n in &[1, 2, 3, 5] {
        let c_sec = Fx::from_int(c_sec_n);
        println!("c_sec={c_sec_n}x:");
        for tvl_over_m_pct in [10, 25, 50, 100, 200] {
            let tvl = Fx::from_int(tvl_over_m_pct);
            let m = Fx::from_int(100);
            let floor = security_floor(c_sec, tvl, m, r_atk);
            println!(
                "  TVL/M={tvl_over_m_pct:>3}%  floor={:.6} (of M per epoch)",
                floor.to_f64()
            );
        }
    }
}
