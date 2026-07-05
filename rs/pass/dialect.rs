//! Pass 2 — dialect discovery (`specs/ct0.md` §4).
//!
//! A dialect is a particle that labels many axons with heavy stake — the source
//! of "label edges" (edges whose target is itself an axon). Dialects become the
//! attention heads (§5.3): each head reads the sub-graph of one dialect. This
//! pass finds them, ranks them, and assigns every link to the dialect that most
//! strongly labels its axon.
//!
//! The score `usage · log₂(1+coverage)` (§4.3) is a ranking key for a discrete
//! structural decision — which particles are dialects — not a model weight, so
//! it is computed in `f64` (deterministic IEEE) rather than the field. The
//! emitted tensors remain fixed-point.

use std::collections::{HashMap, HashSet};

use super::index::{Edge, ParticleIndex};

/// The reserved default dialect `⊥ = 0x00×32`, appended at the highest index.
pub const BOTTOM: [u8; 32] = [0u8; 32];

/// The registered dialect set and per-link assignment (§4.6).
pub struct Dialects {
    /// Registered dialects by descending score, with `⊥` last. Head index = position.
    pub set: Vec<[u8; 32]>,
    /// `assign[k]` = the dialect id (index into `set`) of edge `k` (§4.5).
    pub assign: Vec<usize>,
    /// Per-dialect edge count and aggregate positive stake, aligned with `set`.
    pub edge_count: Vec<u64>,
    pub stake: Vec<i128>,
}

impl Dialects {
    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
}

const THETA: f64 = 1e-3;

/// Pass 2: discover dialects and assign every edge to one.
pub fn discover(v: &ParticleIndex, edges: &[Edge]) -> Dialects {
    // §4.1 — the axon set Ω: every axon that appears as some link's axon.
    let omega: HashSet<u32> = edges.iter().map(|e| e.axon).collect();

    // §4.2–4.3 — label edges (target is an axon) score their source as a dialect.
    // usage = Σ signed stake; coverage = # distinct axon targets labelled.
    let mut usage: HashMap<u32, i128> = HashMap::new();
    let mut coverage: HashMap<u32, HashSet<u32>> = HashMap::new();
    // (dialect source, labelled axon) → total stake, for the §4.5 argmax.
    let mut label_weight: HashMap<(u32, u32), i128> = HashMap::new();
    for e in edges {
        if omega.contains(&e.tgt) {
            *usage.entry(e.src).or_insert(0) += e.stake;
            coverage.entry(e.src).or_default().insert(e.tgt);
            *label_weight.entry((e.src, e.tgt)).or_insert(0) += e.stake;
        }
    }

    // §4.3 — score(p) = usage⁺ · log₂(1+coverage). Contested (net-negative) or
    // uncovered sources score zero and cannot register.
    let score = |p: u32| -> f64 {
        let u = (*usage.get(&p).unwrap_or(&0)).max(0) as f64;
        let c = coverage.get(&p).map(|s| s.len()).unwrap_or(0) as f64;
        u * (1.0 + c).log2()
    };
    let candidates: Vec<u32> = usage.keys().copied().collect();
    let max_score = candidates.iter().map(|&p| score(p)).fold(0.0_f64, f64::max);

    // §4.4 — register {p : score ≥ θ·max}, descending score, ties by ascending
    // particle hash. Append ⊥ at the highest index.
    let mut registered: Vec<(u32, f64)> = candidates
        .iter()
        .copied()
        .map(|p| (p, score(p)))
        .filter(|&(_, s)| max_score > 0.0 && s >= THETA * max_score)
        .collect();
    registered.sort_by(|a, b| {
        // scores are finite (non-negative × log₂); fall back to Equal if ever NaN.
        b.1.partial_cmp(&a.1)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| v.particle(a.0).cmp(&v.particle(b.0)))
    });

    let mut set: Vec<[u8; 32]> = registered.iter().map(|&(p, _)| v.particle(p)).collect();
    set.push(BOTTOM);
    let bottom_idx = set.len() - 1;

    // Dialect-source id → its head index in `set` (⊥ excluded here).
    let head_of: HashMap<u32, usize> = registered
        .iter()
        .enumerate()
        .map(|(h, &(p, _))| (p, h))
        .collect();

    // §4.5 — assign each edge to the registered dialect that most strongly
    // labels its axon; ⊥ when none does. Argmax ties break by ascending head.
    let mut assign = Vec::with_capacity(edges.len());
    for e in edges {
        let alpha = e.axon;
        let mut best: Option<(usize, i128)> = None; // (head, weight)
        for (&(s, ax), &w) in &label_weight {
            if ax == alpha {
                if let Some(&h) = head_of.get(&s) {
                    match best {
                        Some((bh, bw)) if w < bw || (w == bw && h >= bh) => {}
                        _ => best = Some((h, w)),
                    }
                }
            }
        }
        assign.push(best.map(|(h, _)| h).unwrap_or(bottom_idx));
    }

    // §4.6 — per-dialect edge count and aggregate positive stake.
    let mut edge_count = vec![0u64; set.len()];
    let mut stake = vec![0i128; set.len()];
    for (k, e) in edges.iter().enumerate() {
        let d = assign[k];
        edge_count[d] += 1;
        if e.stake > 0 {
            stake[d] += e.stake;
        }
    }

    Dialects {
        set,
        assign,
        edge_count,
        stake,
    }
}

#[cfg(test)]
mod tests {
    use super::super::index::{axon, build};
    use super::*;
    use crate::graph::Cyberlink;

    fn hash(b: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    fn link(from: u8, to: [u8; 32], amount: u128, valence: i8) -> Cyberlink {
        Cyberlink {
            neuron: hash(from),
            from: hash(from),
            to,
            token: 0,
            amount,
            valence,
            block: 0,
        }
    }

    #[test]
    fn bottom_is_always_present_and_last() {
        let links = vec![link(1, hash(2), 100, 1)];
        let (v, edges, _a) = build(&[], &links);
        let d = discover(&v, &edges);
        assert_eq!(
            *d.set.last().unwrap(),
            BOTTOM,
            "⊥ is the highest-index dialect"
        );
        assert!(d.assign.iter().all(|&h| h < d.set.len()));
    }

    #[test]
    fn a_heavy_labeller_registers_as_a_dialect() {
        // Build a graph where particle 1 links to axon(2,3) with heavy stake —
        // making 1 a dialect that labels that axon. First create the axon by a
        // 2→3 link, then have 1 label it.
        let ax23 = axon(&hash(2), &hash(3));
        let links = vec![
            link(2, hash(3), 100, 1), // creates axon(2,3) as a particle
            link(1, ax23, 5000, 1),   // particle 1 labels axon(2,3) heavily
        ];
        let (v, edges, _a) = build(&[], &links);
        let d = discover(&v, &edges);
        // particle 1 should be a registered dialect (not just ⊥).
        assert!(
            d.set.contains(&hash(1)),
            "the heavy labeller must register as a dialect"
        );
        assert!(d.len() >= 2, "at least the labeller + ⊥");
    }

    #[test]
    fn edge_counts_cover_every_edge() {
        let ax23 = axon(&hash(2), &hash(3));
        let links = vec![link(2, hash(3), 100, 1), link(1, ax23, 5000, 1)];
        let (v, edges, _a) = build(&[], &links);
        let d = discover(&v, &edges);
        let total: u64 = d.edge_count.iter().sum();
        assert_eq!(
            total as usize,
            edges.len(),
            "every edge assigned to exactly one dialect"
        );
    }
}
