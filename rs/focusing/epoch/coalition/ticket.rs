//! ticket — the chain's ticket of `specs/rewards.md` §7 as tru#45 proposes
//! it (see [`super`], which states what that citation means): the probe draws
//! ([`probes`], [`D_PROBE`]), one gradient [`Probe`], and the [`Ticket`] of a
//! window of marginals plus probes ([`Game::ticket`]) with its commitment
//! preimage ([`Ticket::preimage`]). The game, the ordering `π(n)` and the
//! full sample are the parent module's.

use cyber_hemera::Hasher;

use super::{D_SAMPLE, Game, ordering};

/// Domain separator of a ticket's gradient probes,
/// `(a_i, t_i)` from the hemera XOF of `D_PROBE ‖ beacon ‖ cluster ‖ nonce`
/// (rewards.md §7 (tru#45) step 1).
pub const D_PROBE: &[u8] = b"soft3/chain/probe/v1\0";

/// A ticket's probe draws: `count` pairs `(a_i, t_i)` — "`u64 mod |A_{E−1}|` and
/// `u32` from the hemera XOF of `D_PROBE ‖ b_E ‖ cluster ‖ nonce`"
/// (rewards.md §7 (tru#45) step 1), in draw order; `a_i` indexes the boundary's
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

/// One gradient probe of a ticket (rewards.md §7 (tru#45) step 2): the
/// finite difference of `J_U` along one present axon of the boundary graph
/// at a drawn fraction of its weight — "`m^probe_i(n) = J_U(φ*(A_{E−1} + u_i e_{a_i}))
/// − J_U(φ*_{E−1})`, `u_i = max(1, ⌊A_{a_i} t_i / 2^32⌋)`", the one-axon form of
/// the path nodes of [[epoch]] §4.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    /// Index into the boundary's axons, ascending (`Game::base`).
    pub axon: usize,
    /// The drawn fraction at 2^32.
    pub t: u32,
    /// `u = max(1, ⌊A_a·t/2^32⌋)`, the weight added.
    pub u: u128,
    /// `J_U(φ*(A_{E−1} + u·e_a)) − J_U(φ*_{E−1})`, nats at 2^60.
    pub value: i128,
}

/// A ticket's sample `m(n)` of `W_TICKET` entries (rewards.md §7 (tru#45)
/// step 2): the marginals of `W' = min(N, W_TICKET − 1)` cyclic ranks of
/// `π(n)` from the drawn rank `r`, then `W_TICKET − W'` gradient probes.
/// "Every ticket costs at most `W_TICKET + 1` solves — the same for an empty
/// epoch, a game of three and a game of a thousand."
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

    /// The commitment preimage of rewards.md §7 (tru#45) step 3: "`D_SAMPLE ‖
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
    /// One gradient probe at `(axon, t)`: one solve against the boundary's
    /// own `J_U(φ*_{E−1})` (the base) ([`Probe`]).
    pub fn probe(&self, axon: usize, t: u32) -> Probe {
        let a = self.base[axon];
        // ⌊A·t/2^32⌋ without a 160-bit product: A = hi·2^32 + lo, so
        // ⌊A·t/2^32⌋ = hi·t + ⌊lo·t/2^32⌋ exactly; A < 2^124 keeps hi·t < 2^124.
        let (hi, lo) = (a.weight >> 32, a.weight & 0xFFFF_FFFF);
        let u = (hi * t as u128 + ((lo * t as u128) >> 32)).max(1);
        let mut with = self.base.clone();
        with[axon].weight = a.weight.saturating_add(u);
        Probe {
            axon,
            t,
            u,
            value: self.syntropy_of(&with) - self.base_syntropy,
        }
    }

    /// The ticket `m(n)` of rewards.md §7 (tru#45) step 2 for a width
    /// `w = W_TICKET` and a drawn rank `r`: "the first
    /// `W' = min(N, W_TICKET − 1)` are the marginals of the ranks
    /// `r, r+1, …` of `π(n)`, taken cyclically modulo
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
        let mut prev = if w_m == 0 { 0 } else { self.value(&prefix) };
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
}
