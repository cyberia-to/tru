//! A cartel's extraction per epoch on the root chain's rules, measured on a
//! genesis export (rewards.md §7 "the payout", §15; chain-economy §5.2–§5.5,
//! §6.1, §9 "self-linking cycles").
//!
//! Reads `soft3 genesis build --export DIR` (`links.tsv`, `balances.tsv`) and
//! weights every link by the genesis rule `g(f) = max(1, ⌊B(author)/L(author)⌋)`
//! (chain-economy §4.1), so `A_prev` is the real `G_E`. The honest swarm is
//! `honest` positions that raise existing axons by 30 % with valence +1. The
//! cartel is a ring of `k` colluders, each locking stake `s` with valence +1 on
//! one axon of a cycle over `k` fresh particles; `anchored` adds one more link
//! from the heaviest existing particle into the ring. The epoch is settled by
//! the exact rule: the four-node path gradient, `a(c) = Δs·σ·ḡ`, `x = max(0, a)`
//! (every position is a new high on an unstaked side, so `N/R = 1`), the budget
//! `B = ⌊F/4⌋` of the pool (chain-economy §5.5), the pool fed per block by
//! `cut = SUBSIDY · min(5000, isqrt(active_{E−1}·10^8/supply)) / 10^4` — the
//! previous epoch's active stake, the honest swarm's (§6.1) — in a steady
//! state (four epochs of accrual, one paid: `B` = one epoch's inflow) and after
//! ten quiet epochs (`B` = 3.5 inflows); `pay = min(⌊Δs/100⌋, ⌊B·x/X⌋)`. The
//! external-attention gate that rewards.md §7 carried on 2026-11-12 is
//! reported beside, as the measurement that decided against it.
//!
//! ```text
//! cargo run -p cyber-tru --release --example ring_extraction -- DIR [honest] [denom] [threads]
//! ```
//!
//! Floats appear only in the printing.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use tru::focusing::epoch::integrated::score_of;
use tru::focusing::epoch::{Axon, EpochParams, Position, canonical, net_axons, path_gradient};

const S0_PUSSY: u128 = 1_099_389_810_279_114_036;
const SUBSIDY_PUSSY: u128 = 10_458_426_657;
const L: u128 = 240;
const FOCUS_MAX_BPS: u128 = 5_000;
const R_MAX_PPM: u128 = 10_000;

fn hex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(out)
}

/// The genesis axon list `G_pq` of the export under the §4.1 rule.
fn genesis_axons(dir: &str, denom: &str) -> Vec<Axon> {
    let balances = std::fs::read_to_string(format!("{dir}/balances.tsv")).expect("balances.tsv");
    let mut balance: HashMap<[u8; 32], u128> = HashMap::new();
    for line in balances.lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() >= 5 && f[4] == denom {
            if let (Some(owner), Ok(amount)) = (hex32(f[0]), f[2].parse::<u128>()) {
                *balance.entry(owner).or_default() += amount;
            }
        }
    }
    let links = std::fs::read_to_string(format!("{dir}/links.tsv")).expect("links.tsv");
    let mut frames: Vec<([u8; 32], [u8; 32], [u8; 32])> = Vec::new();
    let mut count: HashMap<[u8; 32], u128> = HashMap::new();
    for line in links.lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        if let (Some(n), Some(p), Some(q)) = (hex32(f[0]), hex32(f[1]), hex32(f[2])) {
            frames.push((n, p, q));
            *count.entry(n).or_default() += 1;
        }
    }
    frames
        .iter()
        .map(|&(n, p, q)| Axon {
            from: p,
            to: q,
            weight: (balance.get(&n).copied().unwrap_or(0) / count[&n]).max(1),
        })
        .collect()
}

fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = (n as f64).sqrt() as u128;
    while x * x > n {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= n {
        x += 1;
    }
    x
}

/// The pool's inflow over one epoch, `L · cut`, with `cut` reading the
/// previous epoch's active stake (chain-economy §6.1: `focus_bps(epoch(h) − 1)`)
/// and the supply at the settlement height.
fn inflow(active_prev: u128, height: u128) -> u128 {
    let supply = S0_PUSSY + height * SUBSIDY_PUSSY;
    let bps = FOCUS_MAX_BPS.min(isqrt(active_prev * 100_000_000 / supply));
    L * (SUBSIDY_PUSSY * bps / 10_000)
}

/// The payout of one epoch from the attribution `x` under the budget `B`
/// (chain-economy §5.5): `pay = min(⌊Δs/100⌋, ⌊B·x/X⌋)`.
fn pay_out(positions: &[Position], x: &[i128], budget: u128) -> Vec<u128> {
    let total: u128 = x.iter().map(|&v| v as u128).sum();
    positions
        .iter()
        .zip(x)
        .map(|(c, &xc)| {
            if total == 0 {
                return 0;
            }
            (c.stake * R_MAX_PPM / 1_000_000).min(budget * xc as u128 / total)
        })
        .collect()
}

/// One settled epoch's attribution: `x = max(0, a)` per position (every
/// position is a new high on an unstaked side, so `N/R = 1`).
fn attribute(prev: &[Axon], positions: &[Position], p: &EpochParams) -> Vec<i128> {
    let cur = net_axons(prev, positions);
    let pg = path_gradient(prev, &cur, p);
    let g: HashMap<([u8; 32], [u8; 32]), i128> = pg
        .axons
        .iter()
        .zip(&pg.by_axon)
        .map(|(a, &g)| ((a.from, a.to), g))
        .collect();
    positions
        .iter()
        .map(|c| {
            let gk = g.get(&(c.from, c.to)).copied().unwrap_or(0);
            let sigma = if c.valence < 0 { -1 } else { 1 };
            score_of(c.stake as i128 * sigma, gk).max(0)
        })
        .collect()
}

/// External attention of each position's region: the region is the closure
/// of the position's endpoints under this epoch's new axons; its external
/// attention `ext` is the pre-existing weight entering it from outside, its
/// internal attention `int` the stake this epoch's positions put on axons
/// into it (rewards.md §7). Returns `(ext, int)` of `region(c)` per position.
fn external_attention(prev: &[Axon], positions: &[Position]) -> Vec<(u128, u128)> {
    let existing: HashSet<([u8; 32], [u8; 32])> = prev.iter().map(|a| (a.from, a.to)).collect();
    // union-find over particles joined by new axons
    let mut parent: HashMap<[u8; 32], [u8; 32]> = HashMap::new();
    fn find(parent: &mut HashMap<[u8; 32], [u8; 32]>, x: [u8; 32]) -> [u8; 32] {
        let p = *parent.get(&x).unwrap_or(&x);
        if p == x {
            return x;
        }
        let r = find(parent, p);
        parent.insert(x, r);
        r
    }
    for c in positions {
        if !existing.contains(&(c.from, c.to)) {
            let (a, b) = (find(&mut parent, c.from), find(&mut parent, c.to));
            if a != b {
                parent.insert(a, b);
            }
        }
    }
    // pre-existing weight entering each region from outside it
    let mut ext: HashMap<[u8; 32], u128> = HashMap::new();
    for a in prev {
        let (rf, rt) = (find(&mut parent, a.from), find(&mut parent, a.to));
        if rf != rt {
            *ext.entry(rt).or_default() += a.weight;
        }
    }
    let mut int: HashMap<[u8; 32], u128> = HashMap::new();
    for c in positions {
        let r = find(&mut parent, c.to);
        *int.entry(r).or_default() += c.stake;
    }
    positions
        .iter()
        .map(|c| {
            let r = find(&mut parent, c.to);
            (
                ext.get(&r).copied().unwrap_or(0),
                int.get(&r).copied().unwrap_or(0),
            )
        })
        .collect()
}

/// The payout under the external-attention gate: `x` scaled by
/// `min(1, ext/int)` per position, then the budget rule as in [`settle`].
fn gated_pay(positions: &[Position], x: &[i128], att: &[(u128, u128)], budget: u128) -> Vec<u128> {
    let xg: Vec<u128> = x
        .iter()
        .zip(att)
        .map(|(&xc, &(ext, int))| {
            let xc = xc as u128;
            if int == 0 || ext >= int { xc } else { xc * ext / int }
        })
        .collect();
    let total: u128 = xg.iter().sum();
    positions
        .iter()
        .zip(&xg)
        .map(|(c, &xc)| {
            if total == 0 {
                return 0;
            }
            (c.stake * R_MAX_PPM / 1_000_000).min(budget * xc / total)
        })
        .collect()
}

fn pid(tag: u8, i: usize) -> [u8; 32] {
    let mut id = [0u8; 32];
    id[0] = 0xee;
    id[1] = tag;
    id[2..10].copy_from_slice(&(i as u64).to_le_bytes());
    id
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args.get(1).expect("usage: ring_extraction DIR [honest] [denom] [threads]");
    let honest: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(64);
    let denom = args.get(3).map(String::as_str).unwrap_or("pussy");
    let threads: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
    let p = EpochParams {
        threads,
        ..EpochParams::default()
    };

    let t = Instant::now();
    let prev = canonical(&genesis_axons(dir, denom));
    let total: u128 = prev.iter().map(|a| a.weight).sum();
    let m = prev.len();
    println!(
        "genesis: {m} axons, Σ G = {total} ({:.2?}); honest = {honest}; supply = {S0_PUSSY}",
        t.elapsed()
    );
    // the honest swarm: 30 % raises on existing axons
    let honest_pos: Vec<Position> = (0..honest)
        .map(|i| {
            let a = &prev[(i * 7919) % m];
            Position {
                from: a.from,
                to: a.to,
                stake: (a.weight * 3 / 10).max(1),
                valence: 1,
            }
        })
        .collect();
    let honest_stake: u128 = honest_pos.iter().map(|c| c.stake).sum();
    // the heaviest existing particle by out-weight, the anchor
    let mut out: HashMap<[u8; 32], u128> = HashMap::new();
    for a in &prev {
        *out.entry(a.from).or_default() += a.weight;
    }
    let hub = *out.iter().max_by_key(|e| *e.1).unwrap().0;

    // the pool: fed by the previous epoch's active stake — the honest swarm's,
    // the cartel's stake arriving in this epoch (chain-economy §6.1); the
    // settlement at height 2L + D (epoch 1's game); the budget is a quarter of
    // the pool (§5.5): steady state (four epochs of accrual, one paid), and
    // after `quiet` epochs that paid nothing.
    let quiet: u128 = 10;
    let height = 2 * L + 120;
    let flow = inflow(honest_stake, height);
    let b_steady = (4 * flow) / 4;
    let b_quiet = ((4 + quiet) * flow) / 4;
    let x0 = attribute(&prev, &honest_pos, &p);
    let honest_pay0: u128 = pay_out(&honest_pos, &x0, b_steady).iter().sum();
    println!(
        "pool inflow per epoch at the honest swarm's stake: {flow}; budget steady ⌊F/4⌋ = {b_steady}, after {quiet} quiet epochs {b_quiet}; honest alone paid {honest_pay0} ({:.2} % of the steady budget)",
        100.0 * honest_pay0 as f64 / b_steady.max(1) as f64
    );
    println!();
    println!("| anchored | k | s (per colluder) | ring stake / honest stake | ring x share | ring paid (steady) | ring paid / B | ring paid / ring stake | ring paid ({quiet} quiet) | / ring stake | cap Σ⌊s/100⌋ | honest paid (steady) | ext(region) | int(region) | ring paid, gated (steady) | honest paid, gated (steady) |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    let ks = [2usize, 4, 8, 16, 32, 64];
    let ss: [u128; 4] = [100_000_000_000, 1_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000];
    for anchored in [false, true] {
        for &k in &ks {
            for &s in &ss {
                let mut pos = honest_pos.clone();
                let ring: Vec<[u8; 32]> = (0..k).map(|i| pid(k as u8, i)).collect();
                for i in 0..k {
                    pos.push(Position {
                        from: ring[i],
                        to: ring[(i + 1) % k],
                        stake: s,
                        valence: 1,
                    });
                }
                if anchored {
                    pos.push(Position {
                        from: hub,
                        to: ring[0],
                        stake: s,
                        valence: 1,
                    });
                }
                let n_ring = pos.len() - honest;
                let t = Instant::now();
                let x = attribute(&prev, &pos, &p);
                let total_x: u128 = x.iter().map(|&v| v as u128).sum();
                let pay = pay_out(&pos, &x, b_steady);
                let pay_q = pay_out(&pos, &x, b_quiet);
                let att = external_attention(&prev, &pos);
                let gated = gated_pay(&pos, &x, &att, b_steady);
                let ring_gated: u128 = gated[honest..].iter().sum();
                let honest_gated: u128 = gated[..honest].iter().sum();
                let ring_pay: u128 = pay[honest..].iter().sum();
                let ring_pay_q: u128 = pay_q[honest..].iter().sum();
                let ring_x: u128 = x[honest..].iter().map(|&v| v as u128).sum();
                let honest_pay: u128 = pay[..honest].iter().sum();
                let ring_stake = s * n_ring as u128;
                let cap: u128 = pos[honest..].iter().map(|c| c.stake / 100).sum();
                eprintln!("settled k={k} s={s} anchored={anchored} in {:.2?}", t.elapsed());
                println!(
                    "| {} | {k} | {s} | {:.3} | {:.4} % | {ring_pay} | {:.3} % | {:.5} % | {ring_pay_q} | {:.5} % | {cap} | {honest_pay} | {} | {} | {ring_gated} | {honest_gated} |",
                    if anchored { "yes" } else { "no" },
                    ring_stake as f64 / honest_stake as f64,
                    100.0 * ring_x as f64 / total_x.max(1) as f64,
                    100.0 * ring_pay as f64 / b_steady.max(1) as f64,
                    100.0 * ring_pay as f64 / ring_stake as f64,
                    100.0 * ring_pay_q as f64 / ring_stake as f64,
                    att[honest].0,
                    att[honest].1,
                );
            }
        }
    }
}
