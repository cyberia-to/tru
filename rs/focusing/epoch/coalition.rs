//! coalition — the marginal `m(ℓ | S)` of one change against a coalition, on
//! the epoch engine, and the marginal sample `m(n)` of a settlement ticket
//! (tru `specs/rewards.md` §4, §7; soft3 `specs/chain-economy.md` §7.3).
//!
//! The game. The epoch's canonical input is the staked graph `G_E`: the axon
//! list `A_prev` at the previous boundary and the epoch's changes — one
//! [`Change`] per position (its `Δs·σ` on one axon, chain-economy §5.2) or per
//! link. A coalition `S` is a set of changes; its graph is
//! `A_S = max(0, A_prev + Σ_{ℓ∈S} Δ_ℓ)` per axon, and its value is the
//! syntropy it adds over the epoch's particle set `U` (every particle of
//! `A_prev` or of a change):
//!
//! ```text
//! v(S) = J_U(φ*(A_S)) − J_U(φ*(A_prev)),        v(∅) = 0
//! m(ℓ | S) = v(S ∪ {ℓ}) − v(S)
//! ```
//!
//! The marginal is the signed `ΔJ` — a change that lowers syntropy carries a
//! negative marginal, as the chain's attribution `a(c)` does (chain-economy
//! §5.2). It is not the clipped `Δφ⁺ = [ΔJ]₊` of rewards.md §2: the clips
//! apply after the sample, per position (`max(0, a(c))`, chain-economy §5.3)
//! and on the total (`min(v*(N), Δφ⁺(N))`, rewards.md §4); the ticket's
//! sample carries the signed marginal (rewards.md §7, `m_j(n)`).
//!
//! `φ*` is the engine's own solve ([`super::solve`] on the canonical build of
//! `A_S`, `PHI_STEPS` steps, cold) — the same function, the same bits, the
//! chain's settlement computes; `J_U` is [`super::syntropy`] with `n = |U|`
//! held fixed across coalitions, so a change that adds or removes a particle
//! is valued on the same scale as one that moves weight. Every quantity is an
//! integer at the engine's scales ([`super::fixed`]); no float appears.
//!
//! The sample. A ticket's ordering is `π(n)` of the cluster's changes drawn
//! from the epoch beacon — `beacon(E) = block_id(boundary(E))`, the hash of
//! the boundary block and nothing else (chain-economy §2.3) — by
//! `hemera(D_ORDER ‖ beacon ‖ cluster ‖ n)` ([`ordering`]); the sample
//! `m(n)` is the vector of marginals along it ([`Game::sample`]): the `j`-th
//! player of `π` receives `v(π_{≤j}) − v(π_{<j})`. One sample is `|N|` solves
//! (one per prefix; `v(∅)` is free) and sums exactly to `v(N)` — efficiency
//! holds bit for bit, by telescoping. The mean of `m(n)` over orderings is the
//! Shapley value ([`Game::shapley`] enumerates it for small clusters).
//!
//! The ticket. The chain's attempt is [`Game::ticket`] (rewards.md §7 steps
//! 1–3): `W_TICKET` entries — the marginals of `min(N, W_TICKET − 1)` cyclic
//! ranks of `π(n)` from a drawn rank `r`, then gradient [`probes`] at drawn
//! `(axon, t)` pairs of the boundary graph — at most `W_TICKET + 1` solves
//! whatever `N` is; [`Ticket::preimage`] is the header's commitment.
//!
//! The chain's attribution at genesis is the integrated gradient
//! ([`super::path_gradient`]): the Aumann–Shapley limit of this game. For a
//! one-change epoch the two agree to the quadrature's error
//! (`rs/tests/coalition_marginal.rs`); the sample is the exact game the
//! ticket rule set settles by.

use cyber_hemera::Hasher;

use super::fixed::div_round_i;
use super::graph::canonical;
use super::{Axon, AxonKey, EpochGraph, EpochParams, kernel, solve};
use crate::error::McError;

/// Bound on the game's weights: `Σ A_prev + Σ |Δ| < 2^124`, so that no
/// coalition's total saturates the engine's u128 sum and every build finds
/// the right shift (`Σ A < 2^62` after it). The chain is far inside it
/// (pussy's genesis `Σ G ≈ 2^55`).
pub const WEIGHT_BITS: u32 = 124;

/// Domain separator of a ticket's ordering,
/// `π(n) = hemera(D_ORDER ‖ beacon(E) ‖ cluster ‖ n)` (chain-economy §7.3;
/// the tag is proposed here, in the style of `D_PHI`).
pub const D_ORDER: &[u8] = b"soft3/chain/order/v1\0";

/// Domain separator of a sample's commitment preimage
/// (`commit(m(n))`, rewards.md §7 step 3): [`Sample::preimage`].
pub const D_SAMPLE: &[u8] = b"soft3/chain/sample/v1\0";

/// One player of the epoch's game: a signed weight change on one axon — a
/// position's `Δs·σ` (chain-economy §5.2) or a whole link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Change {
    pub from: [u8; 32],
    pub to: [u8; 32],
    /// Weight added to the axon (negative: removed; the net is clipped at 0).
    pub delta: i128,
}

/// The cooperative game of one epoch (or one cluster of it): the base graph,
/// the players, and the engine parameters every value is solved under.
#[derive(Clone, Debug)]
pub struct Game {
    /// `A_prev`, canonical.
    base: Vec<Axon>,
    players: Vec<Change>,
    /// `|U|` — the particles of the base and of every player.
    particles: u64,
    /// `J_U(φ*(A_prev))`, nats at 2^60.
    base_syntropy: i128,
    params: EpochParams,
}

/// One ticket's sample `m(n)`: the ordering and the marginal of every player
/// along it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sample {
    pub nonce: u64,
    /// `π(n)`: player indices in the order they join.
    pub order: Vec<usize>,
    /// `m[i]` — the marginal of player `i` (indexed like the game's players,
    /// not like `order`), nats at 2^60.
    pub marginals: Vec<i128>,
    /// `v(N) = Σ m`, exactly.
    pub total: i128,
}

impl Sample {
    /// The commitment preimage `D_SAMPLE ‖ k: u32 ‖ m[0]: i128 ‖ … ‖ m[k−1]`,
    /// little-endian, players in game order (the chain hashes it with its
    /// hemera; the ordering is implied by the nonce and need not be bound).
    pub fn preimage(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(D_SAMPLE.len() + 4 + 16 * self.marginals.len());
        v.extend_from_slice(D_SAMPLE);
        v.extend_from_slice(&(self.marginals.len() as u32).to_le_bytes());
        for m in &self.marginals {
            v.extend_from_slice(&m.to_le_bytes());
        }
        v
    }
}

/// The ticket's ordering `π(n)` over `n` players: Fisher–Yates driven by the
/// hemera XOF of `D_ORDER ‖ beacon ‖ cluster ‖ nonce` (little-endian u64
/// draws, `j = draw mod (i + 1)` for `i = n−1 … 1`). Miner-independent and a
/// pure function of the epoch beacon (chain-economy §2.3), so no ordering can
/// be chosen, only found. The residue of `mod (i + 1)` biases a draw by at
/// most `(i + 1)/2^64`, below `2^-46` for any cluster the chain admits
/// (`n < 2^18`); accepted.
pub fn ordering(beacon: &[u8; 32], cluster: &[u8; 32], nonce: u64, n: usize) -> Vec<usize> {
    let mut perm: Vec<usize> = (0..n).collect();
    if n < 2 {
        return perm;
    }
    let mut h = Hasher::new();
    h.update(D_ORDER);
    h.update(beacon);
    h.update(cluster);
    h.update(&nonce.to_le_bytes());
    let mut xof = h.finalize_xof();
    let mut draw = [0u8; 8];
    for i in (1..n).rev() {
        xof.fill(&mut draw);
        let j = (u64::from_le_bytes(draw) % (i as u64 + 1)) as usize;
        perm.swap(i, j);
    }
    perm
}

/// Domain separator of a ticket's gradient probes,
/// `(a_i, t_i)` from the hemera XOF of `D_PROBE ‖ beacon ‖ cluster ‖ nonce`
/// (rewards.md §7 step 1).
pub const D_PROBE: &[u8] = b"soft3/chain/probe/v1\0";

/// A ticket's probe draws: `count` pairs `(a_i, t_i)` — "`u64 mod |A_E|` and
/// `u32` from the hemera XOF of `D_PROBE ‖ b_E ‖ cluster ‖ nonce`"
/// (rewards.md §7 step 1), in draw order; `a_i` indexes the boundary's
/// present axons ascending, `t_i` is a fraction at 2^32. Little-endian; an
/// 8-byte then a 4-byte read per probe. Empty when the boundary has no axon.
pub fn probes(
    beacon: &[u8; 32],
    cluster: &[u8; 32],
    nonce: u64,
    count: usize,
    axons: usize,
) -> Vec<(usize, u32)> {
    if axons == 0 || count == 0 {
        return vec![];
    }
    let mut h = Hasher::new();
    h.update(D_PROBE);
    h.update(beacon);
    h.update(cluster);
    h.update(&nonce.to_le_bytes());
    let mut xof = h.finalize_xof();
    let (mut a, mut t) = ([0u8; 8], [0u8; 4]);
    (0..count)
        .map(|_| {
            xof.fill(&mut a);
            xof.fill(&mut t);
            (
                (u64::from_le_bytes(a) % axons as u64) as usize,
                u32::from_le_bytes(t),
            )
        })
        .collect()
}

/// One gradient probe of a ticket (rewards.md §7 step 2): the finite
/// difference of `J_U` along one present axon of the boundary graph at a
/// drawn fraction of its weight — "`m^probe_i(n) = J_U(φ*(A_E + u_i e_{a_i}))
/// − J_U(φ*_E)`, `u_i = max(1, ⌊A_{a_i} t_i / 2^32⌋)`", the one-axon form of
/// the path nodes of [[epoch]] §4.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    /// Index into the boundary's axons, ascending (`Game::base`).
    pub axon: usize,
    /// The drawn fraction at 2^32.
    pub t: u32,
    /// `u = max(1, ⌊A_a·t/2^32⌋)`, the weight added.
    pub u: u128,
    /// `J_U(φ*(A_E + u·e_a)) − J_U(φ*_E)`, nats at 2^60.
    pub value: i128,
}

/// A ticket's sample `m(n)` of `W_TICKET` entries (rewards.md §7 step 2):
/// the marginals of `W' = min(N, W_TICKET − 1)` cyclic ranks of `π(n)` from
/// the drawn rank `r`, then `W_TICKET − W'` gradient probes. "Every ticket
/// costs at most `W_TICKET + 1` solves — the same for an empty epoch, a game
/// of three and a game of a thousand."
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    pub nonce: u64,
    /// `r`, the window's first rank (the chain draws it from the template's
    /// digest, `u64_le(h[8..16]) mod N`; 0 when `N = 0`).
    pub rank: usize,
    /// `π(n)`.
    pub order: Vec<usize>,
    /// The covered players `c_1 < … < c_{W'}`, ascending by player index.
    pub covered: Vec<usize>,
    /// `m_{c_i}`, aligned with `covered`, nats at 2^60.
    pub marginals: Vec<i128>,
    /// The probes in draw order.
    pub probes: Vec<Probe>,
}

impl Ticket {
    /// `W_TICKET`, the entry count.
    pub fn width(&self) -> usize {
        self.marginals.len() + self.probes.len()
    }

    /// The commitment preimage of rewards.md §7 step 3: "`D_SAMPLE ‖
    /// W_TICKET: u32 ‖ m_{c_1} ‖ ⋯ ‖ m_{c_{W'}} ‖ m^probe_1 ‖ ⋯` (i128 LE
    /// each), the covered players `c_1 < ⋯ < c_{W'}` ascending by player
    /// index, then the probes in draw order"; the chain hashes it and puts
    /// the digest in the header's `sample` field.
    pub fn preimage(&self) -> Vec<u8> {
        let w = self.width();
        let mut v = Vec::with_capacity(D_SAMPLE.len() + 4 + 16 * w);
        v.extend_from_slice(D_SAMPLE);
        v.extend_from_slice(&(w as u32).to_le_bytes());
        for m in &self.marginals {
            v.extend_from_slice(&m.to_le_bytes());
        }
        for p in &self.probes {
            v.extend_from_slice(&p.value.to_le_bytes());
        }
        v
    }
}

impl Game {
    /// The game over `prev` (any order; canonicalised) and `players`, solved
    /// under `p`. Costs one solve (the base value). Errors when the weights
    /// exceed [`WEIGHT_BITS`] (`Σ A_prev + Σ |Δ| ≥ 2^124`).
    pub fn new(prev: &[Axon], players: &[Change], p: &EpochParams) -> Result<Self, McError> {
        let base = canonical(prev);
        let mass = base
            .iter()
            .map(|a| a.weight)
            .chain(players.iter().map(|c| c.delta.unsigned_abs()))
            .try_fold(0u128, |s, w| s.checked_add(w))
            .filter(|&s| s < 1u128 << WEIGHT_BITS);
        if mass.is_none() {
            return Err(McError::Range(format!(
                "the game's weights must sum below 2^{WEIGHT_BITS} (Σ A_prev + Σ |Δ|)"
            )));
        }
        let mut parts: Vec<[u8; 32]> = base
            .iter()
            .flat_map(|a| [a.from, a.to])
            .chain(players.iter().flat_map(|c| [c.from, c.to]))
            .collect();
        parts.sort_unstable();
        parts.dedup();
        let mut g = Self {
            base,
            players: players.to_vec(),
            particles: parts.len() as u64,
            base_syntropy: 0,
            params: p.clone(),
        };
        g.base_syntropy = g.syntropy_of(&g.base);
        Ok(g)
    }

    /// The players, in the order the game indexes them.
    pub fn players(&self) -> &[Change] {
        &self.players
    }

    /// `|U|`, the particle count every `J` is taken over.
    pub fn particles(&self) -> u64 {
        self.particles
    }

    /// `J_U(φ*(A_prev))`, nats at 2^60.
    pub fn base_syntropy(&self) -> i128 {
        self.base_syntropy
    }

    /// `J_U(φ*(A))` for an axon list: the engine's solve on its canonical
    /// build, syntropy over `|U|` particles.
    pub fn syntropy_of(&self, axons: &[Axon]) -> i128 {
        let g = EpochGraph::build_with(axons, self.params.ground);
        if g.n() == 0 {
            return 0;
        }
        let f = solve(&g, &self.params);
        kernel::syntropy_with_n(&f.phi, self.particles, &[0, f.phi.len()])
    }

    /// `A_S = max(0, A_prev + Σ_{ℓ∈S} Δ_ℓ)` per axon, ascending, zero
    /// weights kept (the build drops them). `coalition` is a set of player
    /// indices; a repeated index counts once.
    pub fn axons(&self, coalition: &[usize]) -> Vec<Axon> {
        let mut members: Vec<usize> = coalition.to_vec();
        members.sort_unstable();
        members.dedup();
        let mut delta: Vec<(AxonKey, i128)> = members
            .iter()
            .map(|&i| {
                let c = &self.players[i];
                ((c.from, c.to), c.delta)
            })
            .collect();
        delta.sort_unstable_by_key(|x| x.0);
        let mut out = Vec::with_capacity(self.base.len() + delta.len());
        let (mut i, mut j) = (0, 0);
        while i < self.base.len() || j < delta.len() {
            let key_b = self.base.get(i).map(|a| (a.from, a.to));
            let key_d = delta.get(j).map(|d| d.0);
            let (key, mut w): (AxonKey, i128) = match (key_b, key_d) {
                (Some(b), Some(d)) if d < b => (d, 0),
                (Some(b), _) => {
                    i += 1;
                    (b, self.base[i - 1].weight.min(i128::MAX as u128) as i128)
                }
                (None, Some(d)) => (d, 0),
                (None, None) => unreachable!(),
            };
            while j < delta.len() && delta[j].0 == key {
                w = w.saturating_add(delta[j].1);
                j += 1;
            }
            out.push(Axon {
                from: key.0,
                to: key.1,
                weight: w.max(0) as u128,
            });
        }
        out
    }

    /// `v(S) = J_U(φ*(A_S)) − J_U(φ*(A_prev))`, nats at 2^60; one solve.
    pub fn value(&self, coalition: &[usize]) -> i128 {
        if coalition.is_empty() {
            return 0;
        }
        self.syntropy_of(&self.axons(coalition)) - self.base_syntropy
    }

    /// `m(ℓ | S) = v(S ∪ {ℓ}) − v(S)`, the signed `ΔJ` marginal: two solves
    /// (one when `S` is empty). A player already in `S` has marginal 0 (the
    /// coalition is a set); an index beyond the players panics.
    pub fn marginal(&self, coalition: &[usize], player: usize) -> i128 {
        let mut with = coalition.to_vec();
        with.push(player);
        self.value(&with) - self.value(coalition)
    }

    /// The marginals along `order` (a permutation of the players): `|N|`
    /// solves, one per prefix; `Σ m = v(N)` exactly.
    pub fn sample_along(&self, order: &[usize]) -> Vec<i128> {
        debug_assert!(
            {
                let mut o = order.to_vec();
                o.sort_unstable();
                o == (0..self.players.len()).collect::<Vec<_>>()
            },
            "order must be a permutation of the players"
        );
        let mut m = vec![0i128; self.players.len()];
        let mut prefix = Vec::with_capacity(order.len());
        let mut prev = 0i128;
        for &i in order {
            prefix.push(i);
            let v = self.value(&prefix);
            m[i] = v - prev;
            prev = v;
        }
        m
    }

    /// The ticket's sample `m(n)` under `π(n)` ([`ordering`]).
    pub fn sample(&self, beacon: &[u8; 32], cluster: &[u8; 32], nonce: u64) -> Sample {
        let order = ordering(beacon, cluster, nonce, self.players.len());
        let marginals = self.sample_along(&order);
        let total = marginals.iter().sum::<i128>();
        Sample {
            nonce,
            order,
            marginals,
            total,
        }
    }

    /// The boundary's axons `A_E`, ascending — the index space of a probe.
    pub fn base(&self) -> &[Axon] {
        &self.base
    }

    /// One gradient probe at `(axon, t)`: one solve against the boundary's
    /// own `J_U(φ*_E)` ([`Probe`]).
    pub fn probe(&self, axon: usize, t: u32) -> Probe {
        let a = self.base[axon];
        let u = ((a.weight * t as u128) >> 32).max(1);
        let mut with = self.base.clone();
        with[axon].weight = a.weight.saturating_add(u);
        Probe {
            axon,
            t,
            u,
            value: self.syntropy_of(&with) - self.base_syntropy,
        }
    }

    /// The ticket `m(n)` of rewards.md §7 step 2 for a width `w = W_TICKET`
    /// and a drawn rank `r`: "the first `W' = min(N, W_TICKET − 1)` are the
    /// marginals of the ranks `r, r+1, …` of `π(n)`, taken cyclically modulo
    /// `N`, `m_j(n) = v(π[0..j] ∪ {π[j]}) − v(π[0..j])`; the remaining
    /// `W_TICKET − W'` entries are probes of the gradient, one per drawn
    /// pair `(a_i, t_i)`". Along the window each prefix value is one cold
    /// solve and the next marginal reuses it; `v(∅) = 0` is free on the
    /// wrap; a probe is one solve: at most `w + 1` solves in all. `w ≥ 1`;
    /// `r` is taken modulo `N` (0 when `N = 0`).
    pub fn ticket(
        &self,
        beacon: &[u8; 32],
        cluster: &[u8; 32],
        nonce: u64,
        rank: usize,
        w: usize,
    ) -> Ticket {
        assert!(w >= 1, "a ticket has at least one entry");
        let n = self.players.len();
        let order = ordering(beacon, cluster, nonce, n);
        let rank = if n == 0 { 0 } else { rank % n };
        let w_m = n.min(w - 1);
        // ranks r .. r + w_m − 1 cyclic: the prefix before the first rank is
        // one solve (free when r = 0), then one solve per rank; on the wrap
        // the prefix is empty again.
        let mut by_player: Vec<(usize, i128)> = Vec::with_capacity(w_m);
        let mut prefix: Vec<usize> = order[..rank].to_vec();
        let mut prev = self.value(&prefix);
        for k in 0..w_m {
            let j = (rank + k) % n;
            if j == 0 && k > 0 {
                prefix.clear();
                prev = 0;
            }
            prefix.push(order[j]);
            let v = self.value(&prefix);
            by_player.push((order[j], v - prev));
            prev = v;
        }
        by_player.sort_unstable_by_key(|x| x.0);
        let probes = probes(beacon, cluster, nonce, w - w_m, self.base.len())
            .into_iter()
            .map(|(a, t)| self.probe(a, t))
            .collect();
        Ticket {
            nonce,
            rank,
            order,
            covered: by_player.iter().map(|x| x.0).collect(),
            marginals: by_player.iter().map(|x| x.1).collect(),
            probes,
        }
    }

    /// Samples for many nonces on `workers` threads (nonces are independent;
    /// each solve runs with the game's own `threads`). The miner's loop.
    pub fn samples(
        &self,
        beacon: &[u8; 32],
        cluster: &[u8; 32],
        nonces: &[u64],
        workers: usize,
    ) -> Vec<Sample> {
        let workers = workers.max(1).min(nonces.len().max(1));
        let chunk = nonces.len().div_ceil(workers);
        std::thread::scope(|s| {
            let hs: Vec<_> = nonces
                .chunks(chunk.max(1))
                .map(|ns| {
                    s.spawn(move || {
                        ns.iter()
                            .map(|&n| self.sample(beacon, cluster, n))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            hs.into_iter()
                .flat_map(|h| h.join().expect("sample worker"))
                .collect()
        })
    }

    /// The exact Shapley value by subset enumeration, `2^|N|` solves — the
    /// reference for small clusters (`|N| ≤ 18`: the weighted sum
    /// `n!·max|m|` must stay inside i128, `2^52·2^72` at 18).
    /// `round(Σ_S |S|!(n−|S|−1)! · m(ℓ | S) / n!)` per player, nats at
    /// 2^60; each player rounds on its own, so `Σ shapley` may differ from
    /// `v(N)` by up to `n/2` units of 2^-60 nats.
    pub fn shapley(&self) -> Vec<i128> {
        let n = self.players.len();
        assert!(n <= 18, "exact Shapley enumerates 2^n coalitions, n ≤ 18");
        let v: Vec<i128> = (0..1usize << n)
            .map(|mask| {
                let s: Vec<usize> = (0..n).filter(|&i| mask >> i & 1 == 1).collect();
                self.value(&s)
            })
            .collect();
        let fact = |k: usize| (1..=k as i128).product::<i128>();
        (0..n)
            .map(|i| {
                let num: i128 = (0..1usize << n)
                    .filter(|m| m >> i & 1 == 0)
                    .map(|m| {
                        let z = m.count_ones() as usize;
                        fact(z) * fact(n - z - 1) * (v[m | 1 << i] - v[m])
                    })
                    .sum();
                div_round_i(num, fact(n))
            })
            .collect()
    }
}
