//! DIAGNOSTIC (soft3 director plan, row P7): how big is the epoch engine's φ*
//! as a relation a prover would have to prove? Runs `phi`, `gradient` and
//! `path_gradient` on pussy's real cyberlinks and on a synthetic graph at the
//! audit's burial figures, with the `opcount` counters on, and prints one
//! machine-readable `P7 key=value ...` line per graph and entry point.
//! Run: `cargo test --release --features opcount --test p7_phi_count -- --nocapture`.
#![cfg(feature = "opcount")]

use std::collections::HashSet;

use tru::focusing::epoch::opcount::{self, Counts};
use tru::focusing::epoch::{
    Axon, EpochGraph, EpochParams, Position, gradient, net_axons, path_gradient, phi,
};

const LINKS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../eval/data/pussy_links.jsonl");
const STAKE: u128 = 1_000_000;
/// The audit's burial figures (`audit/epoch-settlement-2026-10.md`).
const BURIAL_LINKS: usize = 29_112;
const BURIAL_PARTICLES: u64 = 16_271;

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The string value of `"key": "..."` in one JSON line (CIDs hold no escapes).
fn field<'a>(line: &'a str, key: &str) -> &'a str {
    let pat = format!("\"{key}\": \"");
    let s = line.find(&pat).unwrap_or_else(|| panic!("no {key} in {line}")) + pat.len();
    let e = s + line[s..].find('"').expect("closing quote");
    &line[s..e]
}

/// Particle id: the hemera hash of the CID string's bytes.
fn pid(cid: &str) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(cyber_hemera::hash(cid.as_bytes()).as_bytes());
    h
}

fn positions(pairs: impl Iterator<Item = ([u8; 32], [u8; 32])>) -> Vec<Position> {
    pairs
        .map(|(from, to)| Position {
            from,
            to,
            stake: STAKE,
            valence: 1,
        })
        .collect()
}

/// Pussy's real links: one position per unique (f, t) pair, as `load()` does.
fn pussy() -> Vec<Axon> {
    let text = std::fs::read_to_string(LINKS).expect("eval/data/pussy_links.jsonl");
    let (mut raw, mut seen, mut parts) = (0usize, HashSet::new(), HashSet::new());
    let mut pairs = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        raw += 1;
        let (f, t) = (field(line, "f"), field(line, "t"));
        parts.insert(f.to_owned());
        parts.insert(t.to_owned());
        if seen.insert((f.to_owned(), t.to_owned())) {
            pairs.push((pid(f), pid(t)));
        }
    }
    println!(
        "P7 input graph=pussy raw_lines={raw} unique_pairs={} unique_particles={}",
        seen.len(),
        parts.len()
    );
    net_axons(&[], &positions(pairs.into_iter()))
}

/// Deterministic synthetic graph: a ring through every particle plus random
/// unique links up to `BURIAL_LINKS`.
fn synthetic() -> Vec<Axon> {
    let n = BURIAL_PARTICLES;
    let id = |i: u64| {
        let mut h = [0u8; 32];
        h[..8].copy_from_slice(&mix(i ^ 0xAB).to_be_bytes());
        h[8..12].copy_from_slice(&(i as u32).to_be_bytes());
        h
    };
    let mut seen: HashSet<(u64, u64)> = (0..n).map(|i| (i, (i + 1) % n)).collect();
    let mut r = 7u64;
    while seen.len() < BURIAL_LINKS {
        r = mix(r);
        let (a, b) = (r % n, mix(r) % n);
        if a != b {
            seen.insert((a, b));
        }
    }
    println!(
        "P7 input graph=synthetic unique_pairs={} particles={n}",
        seen.len()
    );
    net_axons(&[], &positions(seen.into_iter().map(|(a, b)| (id(a), id(b)))))
}

fn params(threads: usize) -> EpochParams {
    EpochParams {
        threads,
        ..EpochParams::default()
    }
}

fn report(graph: &str, entry: &str, g: &EpochGraph, p: &EpochParams, c: &Counts) {
    let steps = c.forward_step + c.adjoint_step;
    let mut line = format!(
        "P7 graph={graph} entry={entry} n={} nnz_sym={} nnz_dir={} steps={steps} K={}",
        g.n(),
        g.sym_nnz(),
        g.directed_nnz(),
        p.heat_terms
    );
    for (k, v) in c.fields() {
        line += &format!(" {k}={v}");
    }
    println!("{line}");
    let mut per = format!("P7 graph={graph} entry={entry} per_step");
    for (k, v) in c.fields() {
        per += &format!(" {k}={:.3}", v as f64 / steps as f64);
    }
    println!("{per}");
}

fn counted<T>(f: impl FnOnce() -> T) -> (T, Counts) {
    opcount::reset();
    let out = f();
    (out, opcount::snapshot())
}

fn measure(name: &str, cur: &[Axon]) {
    let p = params(1);
    let g = EpochGraph::build_with(cur, p.ground);
    let prev = &cur[..cur.len() - cur.len() / 100];
    println!(
        "P7 input graph={name} axons={} particles={} prev_axons={}",
        cur.len(),
        g.n(),
        prev.len()
    );

    let (f1, c_phi) = counted(|| phi(cur, &p));
    let (f2, c_phi2) = counted(|| phi(cur, &p));
    assert_eq!(c_phi, c_phi2, "{name}: phi counts differ between runs");
    assert_eq!(f1.phi, f2.phi, "{name}: phi output differs");
    assert_eq!(f1.syntropy, f2.syntropy);
    let (f4, c_phi4) = counted(|| phi(cur, &params(4)));
    assert_eq!(c_phi, c_phi4, "{name}: phi counts depend on threads");
    assert_eq!(f1.phi, f4.phi);
    report(name, "phi", &g, &p, &c_phi);

    let (_, c_g) = counted(|| gradient(cur, &p));
    let (_, c_g2) = counted(|| gradient(cur, &p));
    let (_, c_g4) = counted(|| gradient(cur, &params(4)));
    assert_eq!(c_g, c_g2, "{name}: gradient counts differ between runs");
    assert_eq!(c_g, c_g4, "{name}: gradient counts depend on threads");
    report(name, "gradient", &g, &p, &c_g);

    let (_, c_pg) = counted(|| path_gradient(prev, cur, &p));
    let (_, c_pg2) = counted(|| path_gradient(prev, cur, &p));
    let (_, c_pg4) = counted(|| path_gradient(prev, cur, &params(4)));
    assert_eq!(c_pg, c_pg2, "{name}: path_gradient counts differ");
    assert_eq!(c_pg, c_pg4, "{name}: path_gradient counts depend on threads");
    report(name, "path_gradient", &g, &p, &c_pg);
}

/// One test so the global counters are never shared between tests.
#[test]
fn p7_phi_count() {
    measure("pussy", &pussy());
    measure("synthetic_burial", &synthetic());
}
