//! Attribution error of the gradient forms against the exact Shapley value.
//!
//! Random small graphs; the players are the axons whose weight changes between
//! two snapshots (new, raised, lowered, removed). The game is
//! `v(S) = J∪(φ*(A_prev + Δ_S)) − J∪(φ*(A_prev))` over the union particle set,
//! φ* at 256 steps (converged far below the printed precision). Exact Shapley
//! by subsets (`2^P` solves). Compared: the chain's midpoint rule and
//! 2/4-node Gauss–Legendre integrated gradients. Floats only in this report.
//!
//! ```text
//! cargo run -p cyber-tru --release --example epoch_shapley -- [cases] [players] [scale%]
//! TRU_MODIFY_ONLY=1 …   # only raise/lower existing axons (no add/remove)
//! ```

use tru::focusing::epoch::integrated::{AxonScore, integrated_gradient, midpoint_attribution};
use tru::focusing::epoch::{Axon, AxonKey, EpochParams, canonical, phi, syntropy};

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn pid(i: u64) -> [u8; 32] {
    let mut h = [0u8; 32];
    h[..8].copy_from_slice(&mix(i ^ 0xAB).to_be_bytes());
    h
}

/// A random case: base graph and `players` changed axons. `scale` is the
/// typical change relative to an axon's weight, in percent.
pub fn case(seed: u64, players: usize, scale: u64) -> (Vec<Axon>, Vec<Axon>) {
    let mut r = seed.wrapping_mul(0x5851_F42D_4C95_7F2D);
    let mut next = || {
        r = mix(r);
        r
    };
    let n = 8 + next() % 8;
    let mut base: Vec<Axon> = Vec::new();
    for _ in 0..(2 * n + next() % (2 * n)) {
        let (a, b) = (next() % n, next() % n);
        if a != b {
            base.push(Axon {
                from: pid(a),
                to: pid(b),
                weight: 1_000_000 + (next() % 9_000_000) as u128,
            });
        }
    }
    let base = canonical(&base);
    let mut cur = base.clone();
    let modify_only = std::env::var_os("TRU_MODIFY_ONLY").is_some();
    for _ in 0..players {
        let kind = if modify_only {
            1 + next() % 2
        } else {
            next() % 4
        };
        match kind {
            0 => {
                // a new axon (possibly onto a new particle)
                let (a, b) = (next() % (n + 2), next() % n);
                if a != b {
                    let w = (50_000 * scale * (1 + next() % 100) / 50) as u128;
                    cur.push(Axon {
                        from: pid(a),
                        to: pid(b),
                        weight: w.max(1),
                    });
                }
            }
            k => {
                let i = (next() as usize) % cur.len();
                let w = cur[i].weight;
                let d = w * scale as u128 * (1 + next() % 100) as u128 / 10_000;
                cur[i].weight = match k {
                    1 => w + d.max(1),
                    2 => w - d.min(w),
                    _ => 0,
                };
            }
        }
    }
    (base, canonical(&cur))
}

/// Exact Shapley over the changed axons.
pub fn shapley(prev: &[Axon], cur: &[Axon], p: &EpochParams) -> Vec<(AxonKey, f64)> {
    // union keyed, (w_prev, w_cur)
    let mut keys: Vec<([u8; 32], [u8; 32])> =
        prev.iter().chain(cur).map(|a| (a.from, a.to)).collect();
    keys.sort();
    keys.dedup();
    let w = |list: &[Axon], k: &([u8; 32], [u8; 32])| -> u128 {
        list.iter()
            .find(|a| (a.from, a.to) == *k)
            .map_or(0, |a| a.weight)
    };
    let all: Vec<(AxonKey, u128, u128)> =
        keys.iter().map(|k| (*k, w(prev, k), w(cur, k))).collect();
    let mut parts: Vec<[u8; 32]> = keys.iter().flat_map(|k| [k.0, k.1]).collect();
    parts.sort();
    parts.dedup();
    let nu = parts.len() as u64;
    let players: Vec<usize> = (0..all.len()).filter(|&i| all[i].1 != all[i].2).collect();
    let np = players.len();
    let value = |mask: usize| -> f64 {
        let axons: Vec<Axon> = all
            .iter()
            .enumerate()
            .map(|(i, &(k, a, b))| {
                let on = players
                    .iter()
                    .position(|&x| x == i)
                    .is_some_and(|j| mask >> j & 1 == 1);
                Axon {
                    from: k.0,
                    to: k.1,
                    weight: if on { b } else { a },
                }
            })
            .collect();
        let f = phi(&axons, p);
        syntropy(&f.phi, nu) as f64 / (1u64 << 60) as f64
    };
    let v: Vec<f64> = (0..1usize << np).map(value).collect();
    let fact = |k: usize| (1..=k).map(|x| x as f64).product::<f64>();
    players
        .iter()
        .enumerate()
        .map(|(j, &i)| {
            let mut s = 0.0;
            for mask in 0..1usize << np {
                if mask >> j & 1 == 0 {
                    let size = mask.count_ones() as usize;
                    let wgt = fact(size) * fact(np - size - 1) / fact(np);
                    s += wgt * (v[mask | 1 << j] - v[mask]);
                }
            }
            (all[i].0, s)
        })
        .collect()
}

fn err(est: &[AxonScore], sh: &[(AxonKey, f64)]) -> f64 {
    let one = (1u64 << 60) as f64;
    let (mut num, mut den) = (0.0, 0.0);
    for (k, s) in sh {
        let e = est
            .iter()
            .find(|x| x.axon == *k)
            .map_or(0.0, |x| x.score as f64 / one);
        num += (e - s).abs();
        den += s.abs();
    }
    if den == 0.0 { 0.0 } else { num / den }
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() - 1) as f64 * q).round() as usize]
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let cases: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(100);
    let players: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
    let scale: u64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(30);
    let p = EpochParams {
        steps: 256,
        ..EpochParams::default()
    };
    // [smooth, new particle, dangling switch] × [midpoint, GL-2, GL-4]
    let mut e: [[Vec<f64>; 3]; 3] = Default::default();
    for seed in 0..cases {
        let (prev, cur) = case(seed, players, scale);
        let sh = shapley(&prev, &cur, &p);
        if sh.is_empty() {
            continue;
        }
        let c = structural(&prev, &cur);
        e[c][0].push(err(&midpoint_attribution(&prev, &cur, &p), &sh));
        e[c][1].push(err(&integrated_gradient(&prev, &cur, 2, &p), &sh));
        e[c][2].push(err(&integrated_gradient(&prev, &cur, 4, &p), &sh));
    }
    for (c, kind) in ["smooth", "new-particle", "dangling"].iter().enumerate() {
        for (name, v) in ["midpoint (chain)", "GL-2", "GL-4"]
            .iter()
            .zip(e[c].iter_mut())
        {
            if v.is_empty() {
                continue;
            }
            report(kind, name, v, players, scale);
        }
    }
}

/// 2 when a particle present on both sides gains its first or loses its last
/// out-axon (the dangling switch — a jump in φ*); 1 when particles only appear
/// or vanish (continuous: a new particle's focus grows from zero); 0 otherwise.
fn structural(prev: &[Axon], cur: &[Axon]) -> usize {
    let sets = |l: &[Axon]| {
        let mut all: Vec<[u8; 32]> = l
            .iter()
            .filter(|a| a.weight > 0)
            .flat_map(|a| [a.from, a.to])
            .collect();
        let mut out: Vec<[u8; 32]> = l.iter().filter(|a| a.weight > 0).map(|a| a.from).collect();
        all.sort();
        all.dedup();
        out.sort();
        out.dedup();
        (all, out)
    };
    let ((a0, o0), (a1, o1)) = (sets(prev), sets(cur));
    let both: Vec<&[u8; 32]> = a0.iter().filter(|x| a1.binary_search(x).is_ok()).collect();
    let switch = both
        .iter()
        .any(|x| o0.binary_search(x).is_ok() != o1.binary_search(x).is_ok());
    if switch {
        2
    } else if a0 != a1 {
        1
    } else {
        0
    }
}

fn report(kind: &str, name: &str, v: &mut [f64], players: usize, scale: u64) {
    {
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        println!(
            "{kind:>10} {name:>16}: cases={} players≤{players} change≈{scale}%  relative ℓ1 error vs exact Shapley: mean={:.4} median={:.4} p90={:.4} max={:.4}",
            v.len(),
            mean,
            pct(v, 0.5),
            pct(v, 0.9),
            pct(v, 1.0)
        );
    }
}
