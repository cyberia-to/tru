//! The epoch graph: canonical CSR over u32 indices, built from the chain's
//! axon list (soft3 chain-economy §4.3).
//!
//! The *external* order is canonical: particles ascending by 32-byte id, axons
//! ascending by `from ‖ to`. Whatever order the input arrives in, the result
//! is the same. Internally the kernel relabels particles in a breadth-first
//! order of the undirected pattern (deterministic: roots and neighbours taken
//! in id order). The relabelling only improves memory locality — every kernel
//! output entry is an exact integer function of its row, so the permutation
//! changes no bit of φ*, J or the gradient, which are reported in id order.
//!
//! Three CSR views share one edge list: by source (the adjoint's `Pᵀ`), by
//! target (the forward `P`), and the symmetric pattern `A + Aᵀ` with both
//! degree normalisations (the forward and adjoint random walk of springs and
//! heat).

use super::fixed::{P_BITS, PHI_BITS, STAKE_BITS};

/// One axon of the focus graph as the chain hands it to tru: the net weight
/// `A_pq = max(0, G_pq + S⁺_pq − S⁻_pq)` (chain-economy §4.2), in token units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Axon {
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub weight: u128,
}

/// An axon's identity `(from, to)`.
pub type AxonKey = ([u8; 32], [u8; 32]);

pub(crate) const NONE: u32 = u32::MAX;

/// The epoch graph.
pub struct EpochGraph {
    /// Particles, ascending id (the external order).
    pub(crate) particles: Vec<[u8; 32]>,
    /// Internal index → external (id-order) index.
    pub(crate) ext: Vec<u32>,
    /// Input axon index → canonical axon index (NONE: dropped), and canonical
    /// axon → internal edge.
    pub(crate) input_axon: Vec<u32>,
    pub(crate) axon_edge: Vec<u32>,
    // Directed edges, ascending internal (src, dst): the by-source CSR.
    pub(crate) out_ptr: Vec<u32>,
    pub(crate) out_dst: Vec<u32>,
    // By-target CSR: in_src[k] is the source of in-entry k, in_edge[k] its edge.
    pub(crate) in_ptr: Vec<u32>,
    pub(crate) in_src: Vec<u32>,
    pub(crate) in_edge: Vec<u32>,
    // Symmetric pattern of A + Aᵀ.
    pub(crate) sym_ptr: Vec<u32>,
    pub(crate) sym_col: Vec<u32>,
    // ── numeric ──
    pub(crate) out_w: Vec<u64>,
    /// Out-row normaliser `N_p = out_p + w0_p` (the ground weight `w0_p`
    /// teleports; see [`Ground`]).
    pub(crate) norm_w: Vec<u64>,
    /// Undirected degree = node stake `Σ_incident A`.
    pub(crate) deg: Vec<u64>,
    /// `Σ deg = 2·Σ A`.
    pub(crate) total: u128,
    /// Teleport prior / springs reference `deg/total` at 2^60.
    pub(crate) tele: Vec<u64>,
    /// `A(p,q)/out(p)` at 2^40, per by-source entry (= per edge).
    pub(crate) p_out: Vec<u64>,
    /// The same values per by-target entry.
    pub(crate) p_in: Vec<u64>,
    /// `W(r,c)/deg(c)` at 2^40 — forward random walk, row r.
    pub(crate) sym_fwd: Vec<u64>,
    /// `W(r,c)/deg(r)` at 2^40 — its transpose, row r.
    pub(crate) sym_adj: Vec<u64>,
    /// Dangling nodes (stake but no out-edge).
    /// Teleport share of each particle's out-row at 2^40: `w0_p/N_p`, and
    /// 2^40 for a particle with stake and `N_p = 0` (dangling, no ground).
    pub(crate) tele_share: Vec<u64>,
    /// The ground rule the graph was built under.
    pub(crate) ground: Ground,
    /// Right shift applied to raw weights so `Σ A < 2^62`.
    pub(crate) shift: u32,
}

/// The ground weight: every particle's out-row carries a virtual weight
/// `w0_p = ⌊(γ_d·d_p + γ_i·in_p) / 2^32⌋` to the teleport prior, so the row
/// normalises by `N_p = out_p + w0_p` and `w0_p/N_p` of the particle's focus
/// teleports. φ* is then continuous in `A`: a particle's first out-axon
/// redirects its focus gradually instead of all at once ([[epoch]] §2).
/// `γ` at 2^32; `γ_d + γ_i ≤ 2^32`. Zero/zero is the plain dangling rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ground {
    /// Coefficient on the particle's undirected degree `d_p`, Q32.
    pub degree: u64,
    /// Coefficient on the particle's in-strength, Q32.
    pub inbound: u64,
}

impl Ground {
    /// No ground: the plain dangling rule (φ* jumps at a first out-axon).
    pub const NONE: Ground = Ground {
        degree: 0,
        inbound: 0,
    };
    /// The protocol value: `w0_p = in_p / 4` ([[epoch]] §2, chosen by
    /// measurement in `audit/epoch-settlement-2026-10.md` §6).
    pub const PROTOCOL: Ground = Ground {
        degree: 0,
        inbound: 1 << 30,
    };
}

impl Default for Ground {
    fn default() -> Self {
        Self::PROTOCOL
    }
}

/// The right shift that brings `total` below `2^62`.
pub(crate) fn shift_for(total: u128) -> u32 {
    let bits = 128 - total.leading_zeros();
    bits.saturating_sub(STAKE_BITS)
}

/// Raw weight → kernel weight under `shift`; a positive weight never rounds
/// to zero (the axon stays present).
pub(crate) fn scaled(raw: u128, shift: u32) -> u64 {
    if raw == 0 {
        0
    } else {
        ((raw >> shift) as u64).max(1)
    }
}

/// Canonicalise an axon list: drop self-loops and zero weights, sort by
/// `from ‖ to`, sum duplicates (the chain hands one entry per axon; summing is
/// the order-independent reading of a duplicate).
pub fn canonical(axons: &[Axon]) -> Vec<Axon> {
    let mut v: Vec<Axon> = axons
        .iter()
        .filter(|a| a.from != a.to && a.weight > 0)
        .copied()
        .collect();
    v.sort_unstable_by_key(|x| (x.from, x.to));
    let mut out: Vec<Axon> = Vec::with_capacity(v.len());
    for a in v {
        match out.last_mut() {
            Some(l) if l.from == a.from && l.to == a.to => {
                l.weight = l.weight.saturating_add(a.weight)
            }
            _ => out.push(a),
        }
    }
    out
}

impl EpochGraph {
    /// Build from the chain's axon list, in any order: self-loops and zero
    /// weights are dropped, duplicates sum (see [`canonical`]); particles are
    /// indexed by ascending id.
    pub fn build(axons: &[Axon]) -> Self {
        Self::build_with(axons, Ground::default())
    }

    /// Build under an explicit ground rule.
    pub fn build_with(axons: &[Axon], ground: Ground) -> Self {
        assert!(
            ground.degree + ground.inbound <= 1 << 32,
            "γ_d + γ_i must be ≤ 1"
        );
        // Canonical order over input indices (no copy of the axons).
        let mut ord: Vec<u32> = (0..axons.len() as u32)
            .filter(|&k| {
                let a = &axons[k as usize];
                a.from != a.to && a.weight > 0
            })
            .collect();
        let key = |k: u32| {
            let a = &axons[k as usize];
            (&a.from, &a.to)
        };
        ord.sort_unstable_by(|&x, &y| key(x).cmp(&key(y)));
        let mut input_axon = vec![NONE; axons.len()];
        let mut canon: Vec<(u32, u128)> = Vec::with_capacity(ord.len()); // (first input index, weight)
        for &k in &ord {
            match canon.last_mut() {
                Some(c) if key(c.0) == key(k) => c.1 = c.1.saturating_add(axons[k as usize].weight),
                _ => canon.push((k, axons[k as usize].weight)),
            }
            input_axon[k as usize] = canon.len() as u32 - 1;
        }
        drop(ord);

        let mut particles: Vec<[u8; 32]> = Vec::with_capacity(canon.len() * 2);
        for &(k, _) in &canon {
            particles.push(axons[k as usize].from);
            particles.push(axons[k as usize].to);
        }
        particles.sort_unstable();
        particles.dedup();
        particles.shrink_to_fit();
        let n = particles.len();
        let idx = |p: &[u8; 32]| particles.binary_search(p).expect("endpoint indexed") as u32;
        let total: u128 = canon.iter().fold(0u128, |s, c| s.saturating_add(c.1));
        let shift = shift_for(total);

        // Locality: breadth-first order of the undirected pattern.
        let pairs: Vec<(u32, u32)> = canon
            .iter()
            .map(|&(k, _)| (idx(&axons[k as usize].from), idx(&axons[k as usize].to)))
            .collect();
        let int_of = bfs_order(n, &pairs);
        let mut ext = vec![0u32; n];
        for (e, &i) in int_of.iter().enumerate() {
            ext[i as usize] = e as u32;
        }
        let mut order: Vec<(u32, u32, u32)> = pairs
            .iter()
            .enumerate()
            .map(|(c, &(s, d))| (int_of[s as usize], int_of[d as usize], c as u32))
            .collect();
        drop(pairs);
        drop(int_of);
        order.sort_unstable();
        let edges: Vec<(u32, u32)> = order.iter().map(|t| (t.0, t.1)).collect();
        let weights: Vec<u64> = order
            .iter()
            .map(|t| scaled(canon[t.2 as usize].1, shift))
            .collect();
        let mut axon_edge = vec![0u32; canon.len()];
        for (e, t) in order.iter().enumerate() {
            axon_edge[t.2 as usize] = e as u32;
        }
        drop(order);
        drop(canon);
        Self::from_edges(
            particles, ext, input_axon, axon_edge, &edges, &weights, shift, ground,
        )
    }

    #[allow(clippy::too_many_arguments)] // private: the build's parts, assembled once
    fn from_edges(
        particles: Vec<[u8; 32]>,
        ext: Vec<u32>,
        input_axon: Vec<u32>,
        axon_edge: Vec<u32>,
        edges: &[(u32, u32)],
        weights: &[u64],
        shift: u32,
        ground: Ground,
    ) -> Self {
        let n = particles.len();
        let m = edges.len();
        assert!(m < NONE as usize, "edge count exceeds u32 indexing");
        assert!(n < NONE as usize, "particle count exceeds u32 indexing");

        let mut out_ptr = vec![0u32; n + 1];
        let mut in_ptr = vec![0u32; n + 1];
        for &(s, d) in edges {
            out_ptr[s as usize + 1] += 1;
            in_ptr[d as usize + 1] += 1;
        }
        for i in 0..n {
            out_ptr[i + 1] += out_ptr[i];
            in_ptr[i + 1] += in_ptr[i];
        }
        let out_dst: Vec<u32> = edges.iter().map(|e| e.1).collect();
        let mut in_src = vec![0u32; m];
        let mut in_edge = vec![0u32; m];
        let mut fill = in_ptr.clone();
        for (e, &(s, d)) in edges.iter().enumerate() {
            let k = fill[d as usize] as usize;
            in_src[k] = s;
            in_edge[k] = e as u32;
            fill[d as usize] += 1;
        }

        // Symmetric pattern: row r holds col c for every edge r→c or c→r.
        let mut cnt = vec![0u32; n + 1];
        for &(s, d) in edges {
            cnt[s as usize + 1] += 1;
            cnt[d as usize + 1] += 1;
        }
        for i in 0..n {
            cnt[i + 1] += cnt[i];
        }
        let mut raw: Vec<(u32, u32, bool)> = vec![(0, 0, false); 2 * m]; // (col, edge, row→col)
        let mut fill = cnt.clone();
        for (e, &(s, d)) in edges.iter().enumerate() {
            let k = fill[s as usize] as usize;
            raw[k] = (d, e as u32, true);
            fill[s as usize] += 1;
            let k = fill[d as usize] as usize;
            raw[k] = (s, e as u32, false);
            fill[d as usize] += 1;
        }
        let mut sym_ptr = vec![0u32; n + 1];
        let mut sym_col = Vec::with_capacity(2 * m);
        let mut sym_e = Vec::with_capacity(2 * m);
        for r in 0..n {
            let row = &mut raw[cnt[r] as usize..cnt[r + 1] as usize];
            row.sort_unstable_by(|x, y| x.0.cmp(&y.0).then(y.2.cmp(&x.2)));
            let mut k = 0;
            while k < row.len() {
                let c = row[k].0;
                let mut pair = [NONE, NONE];
                while k < row.len() && row[k].0 == c {
                    pair[usize::from(!row[k].2)] = row[k].1;
                    k += 1;
                }
                sym_col.push(c);
                sym_e.push(pair);
            }
            sym_ptr[r + 1] = sym_col.len() as u32;
        }
        drop(raw);

        let mut g = Self {
            particles,
            ext,
            input_axon,
            axon_edge,
            out_ptr,
            out_dst,
            in_ptr,
            in_src,
            in_edge,
            sym_ptr,
            sym_col,
            out_w: vec![],
            norm_w: vec![],
            deg: vec![],
            total: 0,
            tele: vec![],
            p_out: vec![],
            p_in: vec![],
            sym_fwd: vec![],
            sym_adj: vec![],
            tele_share: vec![],
            ground,
            shift,
        };
        g.set_weights(weights, &sym_e);
        g
    }

    /// Install edge weights (internal edge order) and recompute every
    /// normalised operator.
    fn set_weights(&mut self, weights: &[u64], sym_e: &[[u32; 2]]) {
        let n = self.particles.len();
        let m = self.out_dst.len();
        assert_eq!(weights.len(), m);
        let total_a: u128 = weights.iter().map(|&w| w as u128).sum();
        assert!(total_a < 1u128 << STAKE_BITS, "weights must sum below 2^62");
        let mut out_w = vec![0u64; n];
        let mut deg = vec![0u64; n];
        for s in 0..n {
            for e in self.out_ptr[s] as usize..self.out_ptr[s + 1] as usize {
                let w = weights[e];
                out_w[s] += w;
                deg[s] += w;
                deg[self.out_dst[e] as usize] += w;
            }
        }
        // Ground weight and the out-row normaliser N = out + w0 (< 2^64:
        // out ≤ 2^62, w0 ≤ d ≤ 2^63).
        let gr = self.ground;
        let norm_w: Vec<u64> = (0..n)
            .map(|i| {
                let inb = (deg[i] - out_w[i]) as u128;
                let w0 =
                    ((gr.degree as u128 * deg[i] as u128 + gr.inbound as u128 * inb) >> 32) as u64;
                out_w[i] + w0
            })
            .collect();
        let total = 2 * total_a;
        // Reciprocals 2^126/x: one wide division per node; every per-edge
        // normalisation is then a multiply (products stay ≤ 2^126).
        let recip = |x: u64| -> u128 {
            if x == 0 {
                0
            } else {
                (1u128 << 126) / x as u128
            }
        };
        let norm = |w: u64, r: u128| -> u64 {
            ((w as u128 * r + (1u128 << (125 - P_BITS))) >> (126 - P_BITS)) as u64
        };
        let r_out: Vec<u128> = norm_w.iter().map(|&x| recip(x)).collect();
        let r_deg: Vec<u128> = deg.iter().map(|&x| recip(x)).collect();
        let mut p_out = vec![0u64; m];
        for s in 0..n {
            for e in self.out_ptr[s] as usize..self.out_ptr[s + 1] as usize {
                p_out[e] = norm(weights[e], r_out[s]);
            }
        }
        let p_in: Vec<u64> = self.in_edge.iter().map(|&e| p_out[e as usize]).collect();
        let nnz = self.sym_col.len();
        let mut sym_fwd = vec![0u64; nnz];
        let mut sym_adj = vec![0u64; nnz];
        for r in 0..n {
            for k in self.sym_ptr[r] as usize..self.sym_ptr[r + 1] as usize {
                let w: u64 = sym_e[k]
                    .iter()
                    .filter(|&&e| e != NONE)
                    .map(|&e| weights[e as usize])
                    .sum();
                sym_fwd[k] = norm(w, r_deg[self.sym_col[k] as usize]);
                sym_adj[k] = norm(w, r_deg[r]);
            }
        }
        let r_tot = (1u128 << 126).checked_div(total).unwrap_or(0);
        self.tele = deg
            .iter()
            .map(|&d| {
                ((d as u128 * r_tot + (1u128 << (125 - PHI_BITS))) >> (126 - PHI_BITS)) as u64
            })
            .collect();
        self.tele_share = (0..n)
            .map(|i| {
                if norm_w[i] == 0 {
                    if deg[i] > 0 { 1u64 << P_BITS } else { 0 }
                } else {
                    norm(norm_w[i] - out_w[i], r_out[i])
                }
            })
            .collect();
        self.out_w = out_w;
        self.norm_w = norm_w;
        self.deg = deg;
        self.total = total;
        self.p_out = p_out;
        self.p_in = p_in;
        self.sym_fwd = sym_fwd;
        self.sym_adj = sym_adj;
    }

    /// Particle count `n`.
    pub fn n(&self) -> usize {
        self.particles.len()
    }

    /// Particles, ascending id — the order of every reported vector.
    pub fn particles(&self) -> &[[u8; 32]] {
        &self.particles
    }

    /// Directed edge count.
    pub fn directed_nnz(&self) -> usize {
        self.out_dst.len()
    }

    /// Symmetric-pattern entry count.
    pub fn sym_nnz(&self) -> usize {
        self.sym_col.len()
    }

    /// The right shift applied to raw weights (`Σ A < 2^62`).
    pub fn shift(&self) -> u32 {
        self.shift
    }

    /// Bytes held by the graph's arrays.
    pub fn heap_bytes(&self) -> usize {
        let (n, m, s) = (self.n(), self.directed_nnz(), self.sym_nnz());
        n * (32 + 4 + 3 * 4 + 3 * 8 + 1) + m * (4 + 4 + 4 + 4 + 8 + 8) + s * (4 + 8 + 8)
    }

    /// Internal → external index.
    pub(crate) fn to_ext<T: Copy + Default>(&self, x: &[T]) -> Vec<T> {
        let mut out = vec![T::default(); x.len()];
        for (i, &v) in x.iter().enumerate() {
            out[self.ext[i] as usize] = v;
        }
        out
    }
}

/// Breadth-first relabelling of the undirected pattern: roots in id order,
/// neighbours in id order. Returns `int_of[external] = internal`.
fn bfs_order(n: usize, pairs: &[(u32, u32)]) -> Vec<u32> {
    let mut ptr = vec![0u32; n + 1];
    for &(s, d) in pairs {
        ptr[s as usize + 1] += 1;
        ptr[d as usize + 1] += 1;
    }
    for i in 0..n {
        ptr[i + 1] += ptr[i];
    }
    let mut adj = vec![0u32; ptr[n] as usize];
    let mut fill = ptr.clone();
    for &(s, d) in pairs {
        adj[fill[s as usize] as usize] = d;
        fill[s as usize] += 1;
        adj[fill[d as usize] as usize] = s;
        fill[d as usize] += 1;
    }
    for i in 0..n {
        adj[ptr[i] as usize..ptr[i + 1] as usize].sort_unstable();
    }
    let mut int_of = vec![NONE; n];
    let mut queue: Vec<u32> = Vec::with_capacity(n);
    let mut next = 0u32;
    for root in 0..n as u32 {
        if int_of[root as usize] != NONE {
            continue;
        }
        let head0 = queue.len();
        queue.push(root);
        int_of[root as usize] = next;
        next += 1;
        let mut head = head0;
        while head < queue.len() {
            let u = queue[head] as usize;
            head += 1;
            for &v in &adj[ptr[u] as usize..ptr[u + 1] as usize] {
                if int_of[v as usize] == NONE {
                    int_of[v as usize] = next;
                    next += 1;
                    queue.push(v);
                }
            }
        }
    }
    int_of
}
