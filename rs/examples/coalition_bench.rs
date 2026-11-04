//! Coalition-marginal benchmark at chain scale: one marginal, one full
//! Shapley sample (one ordering, every marginal) and samples per second on
//! the genesis graph of a root chain.
//!
//! Reads the genesis export of `soft3 genesis build --export DIR`
//! (`links.tsv`: `neuron p q height …`, 32-byte hex ids; `balances.tsv`:
//! `owner token amount address denom`) and weights every link by the chain's
//! genesis rule, `g(f) = max(1, ⌊B(author)/L(author)⌋)` with `B` the author's
//! native balance (chain-economy §4.1), so `G_pq` is the real `G_E`. Players
//! are deterministic: `players` changes over the graph — three in four raise
//! an existing axon by 30 % (a position), one in four adds a new axon between
//! existing particles. Floats appear only in the printing.
//!
//! ```text
//! cargo run -p cyber-tru --release --example coalition_bench -- DIR [players] [solve_threads] [workers] [nonces] [denom]
//! ```

use std::collections::HashMap;
use std::time::Instant;

use tru::focusing::epoch::{Axon, Change, EpochParams, Game};

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
        if f.len() >= 5
            && f[4] == denom
            && let (Some(owner), Ok(amount)) = (hex32(f[0]), f[2].parse::<u128>())
        {
            *balance.entry(owner).or_default() += amount;
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args
        .get(1)
        .expect("usage: coalition_bench DIR [players] [solve_threads] [workers] [nonces] [denom]");
    let arg = |i: usize, d: usize| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let (players, threads, workers, nonces) = (arg(2, 64), arg(3, 1), arg(4, 1), arg(5, 1));
    let denom = args.get(6).map(String::as_str).unwrap_or("pussy");

    let t = Instant::now();
    let raw = genesis_axons(dir, denom);
    let base = tru::focusing::epoch::canonical(&raw);
    let total: u128 = base.iter().map(|a| a.weight).sum();
    eprintln!(
        "genesis: {} frames → {} axons, Σ G = {total} ({:.2?})",
        raw.len(),
        base.len(),
        t.elapsed()
    );
    let m = base.len();
    let changes: Vec<Change> = (0..players)
        .map(|i| {
            let a = &base[(i * 7919) % m];
            if i % 4 == 3 {
                let b = &base[(i * 104_729 + 17) % m];
                Change {
                    from: a.from,
                    to: b.to,
                    delta: (total / m as u128) as i128,
                }
            } else {
                Change {
                    from: a.from,
                    to: a.to,
                    delta: (a.weight as i128) * 3 / 10,
                }
            }
        })
        .collect();
    let p = EpochParams {
        threads,
        ..EpochParams::default()
    };

    let t = Instant::now();
    let game = Game::new(&base, &changes, &p);
    let t_new = t.elapsed();
    eprintln!(
        "game: {} players over |U| = {} particles, base J = {:.6} nats, solve_threads = {threads} ({t_new:.2?})",
        players,
        game.particles(),
        game.base_syntropy() as f64 / (1u64 << 60) as f64
    );

    // one marginal, warm (the prefix value known: one solve) and cold (two)
    let t = Instant::now();
    let m0 = game.value(&[0]);
    let t_warm = t.elapsed();
    let t = Instant::now();
    let m01 = game.marginal(&[1], 0);
    let t_cold = t.elapsed();
    eprintln!(
        "marginal: warm (one solve) {t_warm:.2?} = {:.3e} nats; cold (two solves) {t_cold:.2?} = {:.3e} nats",
        m0 as f64 / (1u64 << 60) as f64,
        m01 as f64 / (1u64 << 60) as f64
    );

    // one full sample
    let beacon = [0x42u8; 32];
    let cluster = [0x01u8; 32];
    let t = Instant::now();
    let s = game.sample(&beacon, &cluster, 0);
    let t_sample = t.elapsed();
    let digest = cyber_hemera::hash(&s.preimage());
    eprintln!(
        "sample (nonce 0, {players} marginals): {t_sample:.2?} = {:.2?} per marginal; Σm = {:.3e} nats; digest {}",
        t_sample / players as u32,
        s.total as f64 / (1u64 << 60) as f64,
        digest.to_hex()
    );

    // throughput: `nonces` samples on `workers` threads
    if nonces > 0 {
        let ns: Vec<u64> = (1..=nonces as u64).collect();
        let t = Instant::now();
        let all = game.samples(&beacon, &cluster, &ns, workers);
        let el = t.elapsed();
        let check = all.iter().filter(|x| x.total == s.total).count();
        eprintln!(
            "throughput: {nonces} samples × {players} marginals on {workers} workers: {el:.2?} = {:.3} samples/s, {:.1} marginals/s ({check}/{nonces} sum to v(N))",
            nonces as f64 / el.as_secs_f64(),
            (nonces * players) as f64 / el.as_secs_f64()
        );
    }
}
