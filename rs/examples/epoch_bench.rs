//! Epoch-settlement benchmark: φ* on real cybergraphs at chain scale.
//!
//! Reads the compact link file written by `eval/links_bin.py` (pussy: 29,112
//! links; bostrom: 2,949,732 links / 3,143,650 particles), assigns a synthetic
//! stake per link (uniform or power-law), and times the focus computation.
//! Floats appear only in this harness (stake generation, printing) — never in
//! the computation measured.
//!
//! ```text
//! cargo run -p cyber-tru --release --example epoch_bench -- legacy /tmp/tru-epoch/pussy.links uniform [max_steps]
//! cargo run -p cyber-tru --release --example epoch_bench -- epoch  /tmp/tru-epoch/bostrom.links powerlaw [threads]
//! cargo run -p cyber-tru --release --example epoch_bench -- settle /tmp/tru-epoch/bostrom.links powerlaw [threads] [new_links]
//! ```
//!
//! Peak memory: run under `/usr/bin/time -l` (maximum resident set size).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

/// Counting allocator: live and peak heap bytes, so a call's own peak can be
/// read apart from the harness's inputs.
struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            let now = LIVE.fetch_add(l.size(), Relaxed) + l.size();
            PEAK.fetch_max(now, Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) };
        LIVE.fetch_sub(l.size(), Relaxed);
    }
}
#[global_allocator]
static GLOBAL: Counting = Counting;

/// Heap peak above the live baseline while `f` runs, in MB.
fn peak_mb<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let base = LIVE.load(Relaxed);
    PEAK.store(base, Relaxed);
    let r = f();
    (r, (PEAK.load(Relaxed) - base) >> 20)
}

use tru::focusing::epoch::integrated::score_of;
use tru::focusing::epoch::{
    EpochGraph, EpochParams, Position, gradient, midpoint, net_axons, phi, solve, solve_steps,
};
use tru::focusing::focusing::{contraction, derived_steps, iterate};
use tru::{Context, FocusingGraph, FocusingParams, Link};

struct Links {
    from: Vec<u32>,
    to: Vec<u32>,
    neuron: Vec<u32>,
    particles: u32,
}

fn read_links(path: &str) -> Links {
    let bytes = std::fs::read(path).expect("read link file");
    assert_eq!(&bytes[0..4], b"TRUL", "not a TRUL link file");
    let u32_at = |o: usize| u32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
    let (count, particles) = (u32_at(8) as usize, u32_at(12));
    let mut l = Links {
        from: Vec::with_capacity(count),
        to: Vec::with_capacity(count),
        neuron: Vec::with_capacity(count),
        particles,
    };
    for i in 0..count {
        let o = 16 + 16 * i;
        l.from.push(u32_at(o));
        l.to.push(u32_at(o + 4));
        l.neuron.push(u32_at(o + 8));
    }
    l
}

/// splitmix64 — the harness's deterministic stake generator.
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Synthetic stake for link `i`: uniform 10^6, or Pareto(α = 1.16, the 80/20
/// law) with scale 10^6, capped at 10^15.
fn stake(dist: &str, i: usize) -> u64 {
    match dist {
        "uniform" => 1_000_000,
        "powerlaw" => {
            let u = ((mix(i as u64) >> 11) as f64 + 1.0) / (1u64 << 53) as f64;
            (1e6 * u.powf(-1.0 / 1.16)).min(1e15) as u64
        }
        other => panic!("unknown stake distribution {other}"),
    }
}

/// A 32-byte particle/neuron id. Real ids are hashes, so by default the index
/// is mixed (random order, no locality); `TRU_SEQ_IDS=1` keeps first-appearance
/// order instead (the order a chain sequence number would give).
fn id(x: u32) -> [u8; 32] {
    let mut h = [0u8; 32];
    if std::env::var_os("TRU_SEQ_IDS").is_some() {
        h[..4].copy_from_slice(&x.to_be_bytes());
    } else {
        h[..8].copy_from_slice(&mix(x as u64 ^ 0xC0FF_EE00).to_be_bytes());
        h[8..12].copy_from_slice(&x.to_be_bytes());
    }
    h
}

fn legacy(l: &Links, dist: &str, max_steps: Option<usize>) {
    let links: Vec<Link> = (0..l.from.len())
        .map(|i| {
            let mut k = Link::stake(id(l.from[i]), id(l.to[i]), stake(dist, i) as u128);
            k.neuron = id(l.neuron[i]);
            k
        })
        .collect();
    let t0 = Instant::now();
    let g = FocusingGraph::build(links, &Context::none());
    let build = t0.elapsed();
    let p = FocusingParams::default();
    let kappa = contraction(&g, &p);
    let t = derived_steps(&g, &p);
    let lmax = g.lambda_max().to_f64();
    let heat_m = (lmax / 4.0).ceil().max(1.0) as usize; // s = τλ/2, split at s' ≤ 2
    println!(
        "legacy build: n={} build={:.2?} λ_max={:.3} λ₂={:.6} κ={:.4} T={} heat sub-applications m={} (×8 SpMV each)",
        g.n(),
        build,
        lmax,
        g.lambda_2().to_f64(),
        kappa.to_f64(),
        t,
        heat_m
    );
    let steps = max_steps.map_or(t, |m| m.min(t));
    let t1 = Instant::now();
    let r = iterate(&g, &p, steps);
    let el = t1.elapsed();
    println!(
        "legacy iterate: {steps} of T={t} steps in {:.2?} ({:.3?}/step) → projected full run {:.1?}",
        el,
        el / steps.max(1) as u32,
        el / steps.max(1) as u32 * t as u32
    );
    let sum: i128 = r.focus.iter().map(|x| x.to_f64() as i128).sum();
    let zeros = r.focus.iter().filter(|x| x.is_zero()).count();
    let mass: f64 = r.focus.iter().map(|x| x.to_f64()).sum();
    println!("legacy φ: Σ={mass:.9} zero entries={zeros} (int part sum {sum})");
    if max_steps.is_none() {
        let r2 = iterate(&g, &p, steps);
        let same = r
            .focus
            .iter()
            .zip(&r2.focus)
            .all(|(a, b)| a.raw() == b.raw());
        println!("legacy determinism (two runs bit-identical): {same}");
    }
}

/// Positions: one per link; valence +1, except every 13th passive (0) and
/// every 31st against (−1).
fn positions(l: &Links, dist: &str) -> Vec<Position> {
    (0..l.from.len())
        .map(|i| Position {
            from: id(l.from[i]),
            to: id(l.to[i]),
            stake: stake(dist, i) as u128,
            valence: if i % 31 == 30 {
                -1
            } else if i % 13 == 12 {
                0
            } else {
                1
            },
        })
        .collect()
}

const ONE: f64 = (1u64 << 60) as f64;

fn l1(a: &[u64], b: &[u64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(&x, &y)| x.abs_diff(y) as f64)
        .sum::<f64>()
        / ONE
}

fn epoch(l: &Links, dist: &str, threads: usize) {
    let p = EpochParams {
        threads,
        ..EpochParams::default()
    };
    let axons = net_axons(&[], &positions(l, dist));
    let t0 = Instant::now();
    let g = EpochGraph::build(&axons);
    let build = t0.elapsed();
    println!(
        "epoch build: axons={} n={} directed nnz={} sym nnz={} build={build:.2?} graph≈{} MB; κ={:.4} steps={} K={}",
        axons.len(),
        g.n(),
        g.directed_nnz(),
        g.sym_nnz(),
        g.heap_bytes() >> 20,
        p.kappa().to_f64(),
        p.steps,
        p.heat_terms
    );
    let t1 = Instant::now();
    let a = solve(&g, &p);
    let el = t1.elapsed();
    println!(
        "epoch φ* ({} steps, threads={threads}): {el:.2?} ({:.1?}/step), last drift={:.3e}, J={:.9}",
        p.steps,
        el / p.steps as u32,
        a.drift as f64 / ONE,
        a.syntropy as f64 / ONE
    );
    let b = solve(&g, &p);
    println!(
        "epoch determinism: two runs bit-identical={}",
        a.phi == b.phi
    );
    for th in [1usize, 4, 12] {
        if th == threads {
            continue;
        }
        let q = EpochParams {
            threads: th,
            ..EpochParams::default()
        };
        let t = Instant::now();
        let c = solve(&g, &q);
        println!(
            "epoch threads={th}: {:.2?}, bit-identical to threads={threads}: {}",
            t.elapsed(),
            c.phi == a.phi && c.syntropy == a.syntropy
        );
    }
    // Input order: reversed axon list gives the same bits.
    let mut rev = axons.clone();
    rev.reverse();
    let r = phi(&rev, &p);
    println!(
        "epoch order-independence (reversed input) bit-identical={}",
        r.phi == a.phi
    );
    let reference = solve_steps(&g, &p, 4 * p.steps);
    let mut traj = Vec::new();
    for s in [8usize, 16, 24, 32, 40, 48, 56, 64] {
        traj.push(format!(
            "{s}:{:.2e}",
            l1(&solve_steps(&g, &p, s).phi, &reference.phi)
        ));
    }
    println!(
        "epoch error vs {}-step reference (steps:ℓ1): {}  (reference last drift {:.2e})",
        4 * p.steps,
        traj.join(" "),
        reference.drift as f64 / ONE
    );
    let mass: u128 = a.phi.iter().map(|&x| x as u128).sum();
    let zeros = a.phi.iter().filter(|&&x| x == 0).count();
    println!(
        "epoch φ*: Σ={:.15} zero entries={zeros} min={} (×2^-60) max={:.6}",
        mass as f64 / ONE,
        a.phi.iter().min().unwrap(),
        *a.phi.iter().max().unwrap() as f64 / ONE
    );
}

fn settle(l: &Links, dist: &str, threads: usize, new_links: usize) {
    let p = EpochParams {
        threads,
        ..EpochParams::default()
    };
    // Previous boundary: all but the last `new_links` positions (file order =
    // height order); current: all, with every 97th earlier position doubled.
    let all = positions(l, dist);
    let cut = all.len() - new_links.min(all.len());
    let prev_pos: Vec<Position> = all[..cut].to_vec();
    let mut cur_pos = all.clone();
    for (i, x) in cur_pos.iter_mut().enumerate().take(cut) {
        if i % 97 == 0 {
            x.stake *= 2;
        }
    }
    let a_prev = net_axons(&[], &prev_pos);
    let a_cur = net_axons(&[], &cur_pos);
    let t = Instant::now();
    let s1 = phi(&a_cur, &p);
    let t1 = t.elapsed();
    let t = Instant::now();
    let mid = midpoint(&a_prev, &a_cur);
    let gr = gradient(&mid, &p);
    let t23 = t.elapsed();
    // Per-position attribution a(c) = Δs·σ·g on its axon (chain-side rule).
    let t = Instant::now();
    let mut changed = 0usize;
    let mut positive = 0usize;
    for (k, c) in cur_pos.iter().enumerate() {
        let before = if k < cut { prev_pos[k].stake } else { 0 };
        if c.stake == before || c.from == c.to {
            continue;
        }
        changed += 1;
        let sigma = if c.valence < 0 { -1 } else { 1 };
        let gk = mid
            .binary_search_by(|m| (m.from, m.to).cmp(&(c.from, c.to)))
            .map_or(0, |k| gr.by_axon[k]);
        let a = score_of(sigma * (c.stake as i128 - before as i128), gk);
        if a > 0 && c.valence != 0 {
            positive += 1;
        }
    }
    let ta = t.elapsed();
    println!(
        "settle ({} new positions): solve 1 φ*(A_cur) {t1:.2?}; solves 2+3 gradient(A_mid) {t23:.2?}; attribution of {changed} changed positions {ta:.2?} ({positive} earn); total {:.2?}",
        new_links,
        t1 + t23 + ta
    );
    println!(
        "settle: J(cur)={:.9} J(mid)={:.9}; gradient axons={}",
        s1.syntropy as f64 / ONE,
        gr.syntropy as f64 / ONE,
        gr.by_axon.len()
    );
    // Gradient convergence: 48 vs 96 steps, over the changed axons.
    let p2 = EpochParams {
        steps: 2 * p.steps,
        ..p.clone()
    };
    let g2 = gradient(&mid, &p2);
    let (mut num, mut den) = (0f64, 0f64);
    for (k, &x) in gr.by_axon.iter().enumerate() {
        let y = g2.by_axon[k] as f64;
        num += (x as f64 - y).abs();
        den += y.abs();
    }
    println!(
        "settle: gradient relative ℓ1 change 48→96 steps over all axons = {:.3e}",
        num / den
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "usage: epoch_bench <legacy|epoch|settle> <file.links> <uniform|powerlaw> [n] [m]"
        );
        std::process::exit(2);
    }
    let t = Instant::now();
    let l = read_links(&args[2]);
    println!(
        "{}: {} links, {} particles, stake={} (read {:.2?})",
        args[2],
        l.from.len(),
        l.particles,
        args[3],
        t.elapsed()
    );
    let n = args.get(4).and_then(|s| s.parse().ok());
    match args[1].as_str() {
        "legacy" => legacy(&l, &args[3], n),
        "epoch" => epoch(&l, &args[3], n.unwrap_or(1)),
        "phi-only" => {
            let p = EpochParams {
                threads: n.unwrap_or(1),
                ..EpochParams::default()
            };
            let axons = net_axons(&[], &positions(&l, &args[3]));
            drop(l);
            let t = Instant::now();
            let (f, mb) = peak_mb(|| phi(&axons, &p));
            println!(
                "phi-only: {:.2?}, J={:.9}, call heap peak {mb} MB (input axons {} MB, result {} MB)",
                t.elapsed(),
                f.syntropy as f64 / ONE,
                (axons.len() * std::mem::size_of::<tru::focusing::epoch::Axon>()) >> 20,
                (f.phi.len() * 40) >> 20
            );
        }
        "gradient-only" => {
            let p = EpochParams {
                threads: n.unwrap_or(1),
                ..EpochParams::default()
            };
            let axons = net_axons(&[], &positions(&l, &args[3]));
            drop(l);
            let t = Instant::now();
            let (g, mb) = peak_mb(|| gradient(&axons, &p));
            println!(
                "gradient-only: {:.2?}, axons={}, call heap peak {mb} MB (input axons {} MB)",
                t.elapsed(),
                g.by_axon.len(),
                (axons.len() * std::mem::size_of::<tru::focusing::epoch::Axon>()) >> 20
            );
        }
        "settle" => settle(
            &l,
            &args[3],
            n.unwrap_or(1),
            args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1000),
        ),
        other => panic!("unknown mode {other}"),
    }
}
