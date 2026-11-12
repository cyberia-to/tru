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
//! `B = L · cut` with `cut = SUBSIDY · min(5000, isqrt(active·10^8/supply)) / 10^4`
//! (one epoch's accrual, the steady state of the quarter release), `pay =
//! min(⌊Δs/100⌋, ⌊B·x/X⌋)`. The external-attention gate of rewards.md §7 is
//! reported beside: a region closed under the epoch's new axons with no
//! pre-existing weight entering it earns nothing.
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

/// One settled epoch: `(pay per position, x per position, B, X)`.
fn settle(
    prev: &[Axon],
    positions: &[Position],
    supply: u128,
    p: &EpochParams,
) -> (Vec<u128>, Vec<i128>, u128, u128) {
    let cur = net_axons(prev, positions);
    let pg = path_gradient(prev, &cur, p);
    let g: HashMap<([u8; 32], [u8; 32]), i128> = pg
        .axons
        .iter()
        .zip(&pg.by_axon)
        .map(|(a, &g)| ((a.from, a.to), g))
        .collect();
    let x: Vec<i128> = positions
        .iter()
        .map(|c| {
            let gk = g.get(&(c.from, c.to)).copied().unwrap_or(0);
            let sigma = if c.valence < 0 { -1 } else { 1 };
            score_of(c.stake as i128 * sigma, gk).max(0)
        })
        .collect();
    let active: u128 = positions.iter().map(|c| c.stake).sum();
    let bps = FOCUS_MAX_BPS.min(isqrt(active * 100_000_000 / supply));
    let cut = SUBSIDY_PUSSY * bps / 10_000;
    let budget = L * cut;
    let total: u128 = x.iter().map(|&v| v as u128).sum();
    let pay: Vec<u128> = positions
        .iter()
        .zip(&x)
        .map(|(c, &xc)| {
            if total == 0 {
                return 0;
            }
            let cap = c.stake * R_MAX_PPM / 1_000_000;
            let share = budget * xc as u128 / total;
            cap.min(share)
        })
        .collect();
    (pay, x, budget, total)
}

/// External attention of each position's region: the region is the closure
/// of the position's endpoints under this epoch's new axons; its external
/// attention is the pre-existing weight entering it from outside (rewards.md
/// §7). Returns `ext(region(c))` per position.
fn external_attention(prev: &[Axon], positions: &[Position]) -> Vec<u128> {
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
    positions
        .iter()
        .map(|c| {
            let r = find(&mut parent, c.to);
            ext.get(&r).copied().unwrap_or(0)
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

    // the honest epoch alone
    let (pay0, _, b0, _) = settle(&prev, &honest_pos, S0_PUSSY, &p);
    let honest_pay0: u128 = pay0.iter().sum();
    println!(
        "honest alone: stake {honest_stake}, budget {b0}, paid {honest_pay0} ({:.2} % of the budget)",
        100.0 * honest_pay0 as f64 / b0.max(1) as f64
    );
    println!();
    println!("| anchored | k | s (per colluder) | ring stake / honest stake | ring x share | budget B | ring paid | ring paid / B | ring paid / ring stake | cap Σ⌊s/100⌋ | ext(region) | honest paid |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|");
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
                let (pay, x, budget, total_x) = settle(&prev, &pos, S0_PUSSY, &p);
                let ext = external_attention(&prev, &pos);
                let ring_pay: u128 = pay[honest..].iter().sum();
                let ring_x: u128 = x[honest..].iter().map(|&v| v as u128).sum();
                let honest_pay: u128 = pay[..honest].iter().sum();
                let ring_stake = s * n_ring as u128;
                let cap: u128 = pos[honest..].iter().map(|c| c.stake / 100).sum();
                eprintln!("settled k={k} s={s} anchored={anchored} in {:.2?}", t.elapsed());
                println!(
                    "| {} | {k} | {s} | {:.3} | {:.4} % | {budget} | {ring_pay} | {:.3} % | {:.5} % | {cap} | {} | {honest_pay} |",
                    if anchored { "yes" } else { "no" },
                    ring_stake as f64 / honest_stake as f64,
                    100.0 * ring_x as f64 / total_x.max(1) as f64,
                    100.0 * ring_pay as f64 / budget.max(1) as f64,
                    100.0 * ring_pay as f64 / ring_stake as f64,
                    ext[honest],
                );
            }
        }
    }
}
