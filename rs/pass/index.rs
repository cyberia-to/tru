//! Pass 1 — particle index and adjacency (`specs/ct0.md` §3).
//!
//! Assign every particle a stable id in insertion order (vocab refs first, then
//! signal order: p, q, axon(p,q) per link), and build the directed positive-
//! stake adjacency the later passes read. Determinism (§3.3) is the whole point:
//! the id a particle receives is a pure function of the `.graph` and its vocab.

use std::collections::HashMap;

use crate::graph::Cyberlink;

/// The axon particle of an ordered pair: `axon(p,q) = hemera(p ‖ q)` (§2.3,
/// [[cybergraph]] A6).
pub fn axon(p: &[u8; 32], q: &[u8; 32]) -> [u8; 32] {
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(p);
    buf[32..].copy_from_slice(q);
    let mut out = [0u8; 32];
    out.copy_from_slice(cyber_hemera::hash(&buf).as_bytes());
    out
}

/// Signed effective stake of a link (§2.4): `+a·ρ` for affirm, `0` for void,
/// `−a·ρ` for challenge. `ρ_τ` is the token-denomination weight; until
/// `config.tokens` is threaded it is 1 for every denomination.
pub fn effective_stake(valence: i8, amount: u128, rho: i128) -> i128 {
    match valence {
        1 => amount as i128 * rho,
        -1 => -(amount as i128 * rho),
        _ => 0,
    }
}

/// A resolved cyberlink in id space: endpoints and axon as indices, plus the
/// signed effective stake. Produced once by pass 1 and reused by passes 2 and 5.
#[derive(Clone, Copy)]
pub struct Edge {
    pub src: u32,
    pub tgt: u32,
    pub axon: u32,
    pub stake: i128,
    pub valence: i8,
}

/// The ordered particle set `V` with its id assignment (§3.1).
pub struct ParticleIndex {
    particles: Vec<[u8; 32]>,
    index: HashMap<[u8; 32], u32>,
}

impl ParticleIndex {
    fn new() -> Self {
        Self { particles: Vec::new(), index: HashMap::new() }
    }

    /// Insert a particle if absent, returning its id.
    fn intern(&mut self, p: [u8; 32]) -> u32 {
        if let Some(&i) = self.index.get(&p) {
            return i;
        }
        let i = self.particles.len() as u32;
        self.index.insert(p, i);
        self.particles.push(p);
        i
    }

    pub fn idx(&self, p: &[u8; 32]) -> Option<u32> {
        self.index.get(p).copied()
    }

    pub fn particle(&self, i: u32) -> [u8; 32] {
        self.particles[i as usize]
    }

    pub fn particles(&self) -> &[[u8; 32]] {
        &self.particles
    }

    pub fn len(&self) -> usize {
        self.particles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }
}

/// The directed, positive-stake adjacency (§3.4): `A[i][j] = Σ w(ℓ)` over links
/// `p→q` with `w(ℓ) > 0`. Only affirmations contribute; challenges (negative
/// stake) and voids (zero) are excluded before matrix construction.
pub struct Adjacency {
    pub n: usize,
    /// `out[i]` = the nonzero columns `(j, weight)` of row `i`, ascending by `j`.
    pub out: Vec<Vec<(u32, i128)>>,
}

impl Adjacency {
    /// Total out-strength `Σ_j A[i][j]` of node `i`.
    pub fn out_strength(&self, i: u32) -> i128 {
        self.out[i as usize].iter().map(|&(_, w)| w).sum()
    }
}

/// Pass 1: build the particle index, the resolved edge list, and the adjacency.
/// `vocab_seed` is the particle order from any referenced `.vocab` files (§3.1
/// step 2); pass an empty slice when the graph declares none.
pub fn build(vocab_seed: &[[u8; 32]], links: &[Cyberlink]) -> (ParticleIndex, Vec<Edge>, Adjacency) {
    let mut v = ParticleIndex::new();

    // Step 2 — seed from vocab refs, in declared order.
    for &p in vocab_seed {
        v.intern(p);
    }

    // Step 3 — append from signals: p, then q, then axon(p,q).
    let mut edges = Vec::with_capacity(links.len());
    for l in links {
        let a = axon(&l.from, &l.to);
        let si = v.intern(l.from);
        let ti = v.intern(l.to);
        let ai = v.intern(a);
        edges.push(Edge {
            src: si,
            tgt: ti,
            axon: ai,
            stake: effective_stake(l.valence, l.amount, 1),
            valence: l.valence,
        });
    }

    // §3.4 — positive-stake directed adjacency, accumulated per (i,j).
    let n = v.len();
    let mut acc: Vec<HashMap<u32, i128>> = vec![HashMap::new(); n];
    for e in &edges {
        if e.stake > 0 {
            *acc[e.src as usize].entry(e.tgt).or_insert(0) += e.stake;
        }
    }
    let out: Vec<Vec<(u32, i128)>> = acc
        .into_iter()
        .map(|row| {
            let mut cols: Vec<(u32, i128)> = row.into_iter().collect();
            cols.sort_by_key(|&(j, _)| j);
            cols
        })
        .collect();

    (v, edges, Adjacency { n, out })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(b: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    fn link(from: u8, to: u8, amount: u128, valence: i8) -> Cyberlink {
        Cyberlink {
            neuron: hash(from),
            from: hash(from),
            to: hash(to),
            token: 0,
            amount,
            valence,
            block: 0,
        }
    }

    #[test]
    fn axon_is_deterministic_and_oriented() {
        let (p, q) = (hash(1), hash(2));
        assert_eq!(axon(&p, &q), axon(&p, &q), "axon must be a pure function");
        assert_ne!(axon(&p, &q), axon(&q, &p), "axon is oriented: H(p‖q) ≠ H(q‖p)");
    }

    #[test]
    fn ids_follow_insertion_order_p_q_axon() {
        let links = vec![link(1, 2, 100, 1)];
        let (v, _e, _a) = build(&[], &links);
        // p=1 → 0, q=2 → 1, axon(1,2) → 2.
        assert_eq!(v.idx(&hash(1)), Some(0));
        assert_eq!(v.idx(&hash(2)), Some(1));
        assert_eq!(v.idx(&axon(&hash(1), &hash(2))), Some(2));
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn vocab_seed_takes_the_low_ids() {
        let seed = [hash(9), hash(8)];
        let links = vec![link(1, 2, 100, 1)];
        let (v, _e, _a) = build(&seed, &links);
        assert_eq!(v.idx(&hash(9)), Some(0), "vocab seed comes first");
        assert_eq!(v.idx(&hash(8)), Some(1));
        assert_eq!(v.idx(&hash(1)), Some(2), "signal particles follow the seed");
    }

    #[test]
    fn adjacency_sums_affirmations_and_drops_the_rest() {
        // Two affirmations 1→2 accumulate; a challenge and a void do not.
        let links = vec![
            link(1, 2, 100, 1),
            link(1, 2, 50, 1),
            link(1, 3, 200, -1), // challenge → negative → excluded
            link(1, 4, 200, 0),  // void → zero → excluded
        ];
        let (v, _e, a) = build(&[], &links);
        let (i1, i2) = (v.idx(&hash(1)).unwrap(), v.idx(&hash(2)).unwrap());
        let row = &a.out[i1 as usize];
        let w12 = row.iter().find(|&&(j, _)| j == i2).map(|&(_, w)| w);
        assert_eq!(w12, Some(150), "affirmations 100+50 accumulate");
        // node 3 and 4 exist as particles but carry no positive inbound edge.
        assert!(a.out[i1 as usize].iter().all(|&(j, _)| j == i2), "only the affirmed edge survives");
    }
}
