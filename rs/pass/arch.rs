//! Pass 3 — architecture parameters (`specs/ct0.md` §5).
//!
//! Derive the transformer's shape from the graph's spectrum, not from a config
//! knob. The embedding dimension `d*` is the effective rank of the φ*-weighted
//! adjacency (exp-entropy of its singular values); the head count `h*` is the
//! number of dialects; the layer count `L*` is the graph diameter times the
//! mixing time implied by the spectral gap `λ₂`. Everything is fixed-point.
//!
//! `d*`'s singular spectrum is computed by subspace iteration on `MᵀM` — the
//! exact object the spec's randomized SVD (pass 4) accelerates; here it sizes
//! the architecture. On large graphs the truncated randomized form lands with
//! the pass-4 milestone; this computes the top `min(rank, |V|)` values directly.

use crate::arithmetic::Fx;

use super::index::Adjacency;

/// The derived architecture (§5.5). `phi` is carried for passes 4–5.
pub struct Arch {
    pub particles: usize,
    pub block: u64,
    /// Embedding dimension `d*` (multiple of `h*`, clamped [64, 4096]).
    pub d: usize,
    /// Head count `h* = |dialects|` (§5.3).
    pub h: usize,
    /// Layer count `L*` (clamped [4, 512]).
    pub l: usize,
    /// Contraction rate `κ = α(1−λ₂)`.
    pub kappa: Fx,
    /// Spectral gap `λ₂` of the normalized Laplacian.
    pub lambda2: Fx,
    /// Lower-bound graph diameter (BFS from the highest-degree node).
    pub diameter: usize,
    /// Focus distribution φ* (§5.1), kept for the embedding/attention passes.
    pub phi: Vec<Fx>,
}

/// PageRank damping α = 0.85 (§5.1).
fn alpha() -> Fx {
    Fx::from_ratio(85, 100)
}

// ── fixed-point vector helpers ────────────────────────────────────────

fn dot(a: &[Fx], b: &[Fx]) -> Fx {
    let mut s = Fx::ZERO;
    for i in 0..a.len() {
        s = s + a[i] * b[i];
    }
    s
}

fn fabs(x: Fx) -> Fx {
    if x < Fx::ZERO {
        Fx::ZERO - x
    } else {
        x
    }
}

fn abs_max_normalize(v: &mut [Fx]) {
    let mut m = Fx::ZERO;
    for &x in v.iter() {
        let a = fabs(x);
        if a > m {
            m = a;
        }
    }
    if !m.is_zero() {
        for x in v.iter_mut() {
            *x = x.div(m);
        }
    }
}

// ── normalized Fx adjacency ───────────────────────────────────────────

/// Directed adjacency with Fx weights (scaled by the max weight so they land in
/// (0,1]) plus the transpose lists, for the spectral computations. Shared with
/// passes 4–5 (embedding, attention).
pub(crate) struct FxAdj {
    pub(crate) n: usize,
    pub(crate) out: Vec<Vec<(u32, Fx)>>,
    pub(crate) inc: Vec<Vec<(u32, Fx)>>,
}

impl FxAdj {
    pub(crate) fn from(adj: &Adjacency) -> Self {
        let n = adj.n;
        let maxw = adj.out.iter().flatten().map(|&(_, w)| w).max().unwrap_or(1).max(1) as u128;
        let mut out = vec![Vec::new(); n];
        let mut inc = vec![Vec::new(); n];
        for (i, row) in adj.out.iter().enumerate() {
            for &(j, w) in row {
                let wf = Fx::ratio_u128(w as u128, maxw);
                out[i].push((j, wf));
                inc[j as usize].push((i as u32, wf));
            }
        }
        FxAdj { n, out, inc }
    }

    /// `(A·x)_i = Σ_j A[i][j] x_j`.
    pub(crate) fn a(&self, x: &[Fx], out: &mut [Fx]) {
        for i in 0..self.n {
            let mut s = Fx::ZERO;
            for &(j, w) in &self.out[i] {
                s = s + w * x[j as usize];
            }
            out[i] = s;
        }
    }

    /// `(Aᵀ·x)_j = Σ_i A[i][j] x_i`.
    pub(crate) fn at(&self, x: &[Fx], out: &mut [Fx]) {
        for j in 0..self.n {
            let mut s = Fx::ZERO;
            for &(i, w) in &self.inc[j] {
                s = s + w * x[i as usize];
            }
            out[j] = s;
        }
    }
}

// ── §5.1 focus distribution (PageRank) ────────────────────────────────

fn pagerank(g: &FxAdj) -> Vec<Fx> {
    let n = g.n;
    if n == 0 {
        return vec![];
    }
    let u = Fx::from_ratio(1, n as i64);
    let out_strength: Vec<Fx> = (0..n).map(|i| g.out[i].iter().fold(Fx::ZERO, |a, &(_, w)| a + w)).collect();
    let mut phi = vec![u; n];
    let eps = Fx::from_ratio(1, 100_000_000);
    for _ in 0..500 {
        // dangling mass (nodes with no out-edges) redistributes to the prior.
        let mut dangling = Fx::ZERO;
        for i in 0..n {
            if out_strength[i].is_zero() {
                dangling = dangling + phi[i];
            }
        }
        // (Pφ)_i = Σ_{(j,w)∈inc[i]} w·φ_j / out_strength[j]  + dangling·u
        let mut next = vec![Fx::ZERO; n];
        for i in 0..n {
            let mut s = Fx::ZERO;
            for &(j, w) in &g.inc[i] {
                let os = out_strength[j as usize];
                if !os.is_zero() {
                    s = s + w.div(os) * phi[j as usize];
                }
            }
            next[i] = alpha() * (s + dangling * u) + (Fx::ONE - alpha()) * u;
        }
        // L1 renormalize (guards drift) and test convergence.
        let sum: Fx = next.iter().fold(Fx::ZERO, |a, &x| a + x);
        if !sum.is_zero() {
            for x in next.iter_mut() {
                *x = x.div(sum);
            }
        }
        let drift: Fx = (0..n).fold(Fx::ZERO, |a, i| a + fabs(next[i] - phi[i]));
        phi = next;
        if drift < eps {
            break;
        }
    }
    phi
}

// ── §5.2 embedding dimension d* via singular spectrum of M ─────────────

/// Truncated SVD of `M = diag(√φ)·A·diag(√φ)` — the φ*-weighted adjacency
/// (§5.2). Shared by pass 3 (d* from σ) and pass 4 (embedding from U, σ).
pub(crate) fn m_svd(g: &FxAdj, phi: &[Fx], k: usize, iters: usize) -> super::svd::Svd {
    let n = g.n;
    let ds: Vec<Fx> = phi.iter().map(|&p| p.sqrt()).collect();
    let apply_m = |x: &[Fx]| -> Vec<Fx> {
        let t: Vec<Fx> = (0..n).map(|i| ds[i] * x[i]).collect();
        let mut ax = vec![Fx::ZERO; n];
        g.a(&t, &mut ax);
        (0..n).map(|i| ds[i] * ax[i]).collect()
    };
    let apply_mt = |x: &[Fx]| -> Vec<Fx> {
        let t: Vec<Fx> = (0..n).map(|i| ds[i] * x[i]).collect();
        let mut atx = vec![Fx::ZERO; n];
        g.at(&t, &mut atx);
        (0..n).map(|i| ds[i] * atx[i]).collect()
    };
    super::svd::top_svd(n, &apply_m, &apply_mt, k, iters)
}

/// d* = ceil(exp(−Σ σ̂ ln σ̂)) — the exp-entropy (effective rank) of the
/// normalized singular spectrum (§5.2).
fn effective_dim(sigmas: &[Fx]) -> usize {
    let total: Fx = sigmas.iter().fold(Fx::ZERO, |a, &s| a + s);
    if total.is_zero() {
        return 64;
    }
    let mut h = Fx::ZERO;
    for &s in sigmas {
        if s > Fx::ZERO {
            let sh = s.div(total);
            h = h - sh * sh.ln();
        }
    }
    h.exp().to_f64().ceil() as usize
}

// ── §5.4 spectral gap λ₂ of the normalized Laplacian ──────────────────

/// λ₂ of `L_norm = I − D^{-1/2} A_sym D^{-1/2}`: `1 − μ₂`, where `μ₂` is the
/// second-largest eigenvalue of `N = D^{-1/2} A_sym D^{-1/2}` (the largest is 1,
/// eigenvector `√d`). Power-iterate `N` deflated against `√d`.
fn lambda2_normalized(g: &FxAdj, iters: usize) -> Fx {
    let n = g.n;
    if n < 2 {
        return Fx::ZERO;
    }
    // Symmetrized weighted degree d_i = Σ_j (A[i][j] + A[j][i]).
    let mut deg = vec![Fx::ZERO; n];
    for i in 0..n {
        for &(j, w) in &g.out[i] {
            deg[i] = deg[i] + w;
            deg[j as usize] = deg[j as usize] + w;
        }
    }
    let inv_sqrt_d: Vec<Fx> = deg.iter().map(|&d| if d.is_zero() { Fx::ZERO } else { Fx::ONE.div(d.sqrt()) }).collect();

    // N x: (N x)_i = Σ_j A_sym[i][j] · x_j / (√d_i √d_j).
    let nmat = |x: &[Fx], out: &mut [Fx]| {
        for o in out.iter_mut() {
            *o = Fx::ZERO;
        }
        for i in 0..n {
            for &(j, w) in &g.out[i] {
                let c = w * inv_sqrt_d[i] * inv_sqrt_d[j as usize];
                out[i] = out[i] + c * x[j as usize];
                out[j as usize] = out[j as usize] + c * x[i]; // symmetric
            }
        }
    };

    // v1 = √d (unnormalized top eigenvector), for deflation.
    let v1: Vec<Fx> = deg.iter().map(|&d| d.sqrt()).collect();
    let v1n = dot(&v1, &v1);

    let deflate = |v: &mut [Fx]| {
        if !v1n.is_zero() {
            let c = dot(&v1, v).div(v1n);
            for i in 0..n {
                v[i] = v[i] - c * v1[i];
            }
        }
    };

    let mut v: Vec<Fx> = (0..n).map(|i| Fx::from_int(((i % 7) + 1) as i64)).collect();
    deflate(&mut v);
    abs_max_normalize(&mut v);
    let mut nv = vec![Fx::ZERO; n];
    let mut mu = Fx::ZERO;
    for _ in 0..iters {
        nmat(&v, &mut nv);
        deflate(&mut nv);
        mu = dot(&v, &nv).div(dot(&v, &v));
        v.copy_from_slice(&nv);
        abs_max_normalize(&mut v);
    }
    let l2 = Fx::ONE - mu;
    if l2 < Fx::ZERO {
        Fx::ZERO
    } else {
        l2
    }
}

// ── §5.4 diameter (BFS lower bound) ───────────────────────────────────

fn diameter(g: &FxAdj) -> usize {
    let n = g.n;
    if n == 0 {
        return 0;
    }
    // Undirected neighbour lists.
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for i in 0..n {
        for &(j, _) in &g.out[i] {
            adj[i].push(j);
            adj[j as usize].push(i as u32);
        }
    }
    // Start from the highest-degree node.
    let start = (0..n).max_by_key(|&i| adj[i].len()).unwrap_or(0);
    let mut dist = vec![usize::MAX; n];
    dist[start] = 0;
    let mut q = std::collections::VecDeque::new();
    q.push_back(start);
    let mut ecc = 0;
    while let Some(u) = q.pop_front() {
        for &w in &adj[u] {
            if dist[w as usize] == usize::MAX {
                dist[w as usize] = dist[u] + 1;
                ecc = ecc.max(dist[w as usize]);
                q.push_back(w as usize);
            }
        }
    }
    ecc.max(1)
}

// ── assembly ──────────────────────────────────────────────────────────

fn round_to_multiple(x: usize, m: usize) -> usize {
    if m == 0 {
        return x;
    }
    let r = x % m;
    if r == 0 {
        x
    } else {
        x + (m - r)
    }
}

/// Pass 3: derive the architecture from the graph and the head count `h*`.
pub fn compute(adj: &Adjacency, h_star: usize, block: u64) -> Arch {
    let g = FxAdj::from(adj);
    let n = g.n;
    let h = h_star.max(1);

    if n == 0 {
        return Arch { particles: 0, block, d: 64, h, l: 4, kappa: Fx::ZERO, lambda2: Fx::ZERO, diameter: 0, phi: vec![] };
    }

    let phi = pagerank(&g);
    let sigmas = m_svd(&g, &phi, 1024, 120).sigma;
    let d0 = effective_dim(&sigmas).clamp(64, 4096);
    let d = round_to_multiple(d0, h).clamp(64, 4096);

    let lambda2 = lambda2_normalized(&g, 200);
    // κ = α(1−λ₂), clamped to a genuine contraction so L* is finite.
    let kappa = {
        let k = alpha() * (Fx::ONE - lambda2);
        let lo = Fx::from_ratio(1, 1000);
        let hi = Fx::from_ratio(999, 1000);
        if k < lo {
            lo
        } else if k > hi {
            hi
        } else {
            k
        }
    };

    let diam = diameter(&g);
    // L* = diam · ceil(ln(1/ε_L)/ln(1/κ)), ε_L = 0.01.
    let eps_l = Fx::from_ratio(1, 100);
    let mixing = (Fx::ZERO - eps_l.ln()).div(Fx::ZERO - kappa.ln()); // ln(1/ε)/ln(1/κ)
    let steps = mixing.to_f64().ceil().max(1.0) as usize;
    let l = (diam * steps).clamp(4, 512);

    Arch { particles: n, block, d, h, l, kappa, lambda2, diameter: diam, phi }
}

#[cfg(test)]
mod tests {
    use super::super::index::build;
    use super::*;
    use crate::graph::Cyberlink;

    fn hash(b: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = b;
        h
    }

    fn link(from: u8, to: u8, amount: u128, valence: i8) -> Cyberlink {
        Cyberlink { neuron: hash(from), from: hash(from), to: hash(to), token: 0, amount, valence, block: 0 }
    }

    fn ring() -> Adjacency {
        let links = vec![link(1, 2, 100, 1), link(2, 3, 100, 1), link(3, 1, 100, 1), link(4, 1, 100, 1)];
        let (_v, _e, a) = build(&[], &links);
        a
    }

    #[test]
    fn phi_sums_to_one() {
        let a = ring();
        let arch = compute(&a, 3, 42);
        let total: f64 = arch.phi.iter().map(|x| x.to_f64()).sum();
        assert!((total - 1.0).abs() < 1e-3, "φ* sums to 1, got {total}");
    }

    #[test]
    fn arch_params_are_in_bounds_and_shaped() {
        let a = ring();
        let arch = compute(&a, 3, 42);
        assert!((64..=4096).contains(&arch.d), "d* clamped, got {}", arch.d);
        assert_eq!(arch.d % arch.h, 0, "d* is a multiple of h*");
        assert!((4..=512).contains(&arch.l), "L* clamped, got {}", arch.l);
        assert!(arch.kappa.to_f64() > 0.0 && arch.kappa.to_f64() < 1.0, "κ a contraction, got {}", arch.kappa.to_f64());
        assert!(arch.diameter >= 1);
        assert_eq!(arch.block, 42);
    }

    #[test]
    fn arch_is_deterministic() {
        let a = ring();
        let x = compute(&a, 3, 42);
        let y = compute(&a, 3, 42);
        assert_eq!(x.d, y.d);
        assert_eq!(x.l, y.l);
        assert_eq!(x.kappa.raw(), y.kappa.raw());
        assert!(x.phi.iter().zip(&y.phi).all(|(p, q)| p.raw() == q.raw()), "φ* bit-identical");
    }

    #[test]
    fn empty_graph_yields_default_arch() {
        let (_v, _e, a) = build(&[], &[]);
        let arch = compute(&a, 1, 0);
        assert_eq!(arch.d, 64);
        assert_eq!(arch.l, 4);
    }
}
