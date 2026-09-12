//! CT-0 pass 5 eval — the transformer half, tested.
//!
//! Forward pass: standard llama semantics (RMSNorm weights 1.0, RoPE theta
//! 10000, causal attention, alpha=1 beta=0, residual, tied lm_head) over the
//! CT-0 compiled weights: per layer l, W_Q/W_K from the SVD of
//! P = Eᵀ A^{l_eff} E (l_eff = 1 + floor(l·diam/L)), W_V = Eᵀ diag(φ) A E,
//! W_O = pinv(W_V). The Clifford MLP is a no-op at compile init (LayerScale
//! gamma = 1e-5), so attention-only forward is the faithful "CT-0 at init".
//!
//! Probe: positions where one step of context is ambiguous but two steps
//! resolve — "X orbits ?" (moon -> its planet; planet -> SUN; bigram answers
//! SUN always) and "P harbors ?" (P's own moon; bigram sees all moons).
//! Ablations: bigram counts, pass-4 embedding dot, full CT-0 stack,
//! and a 1-step-only stack (every layer l_eff=1) — the multi-power ablation.
//!
//! Run: cargo run -p cyber-tru --release --example eval_attn

use std::collections::HashMap;

use tru::graph::Cyberlink;
use tru::pass::{arch, index};

// ---------- world (same corpus as eval_ct0) ----------

const PLANETS: &[&str] = &[
    "MERCURY", "VENUS", "EARTH", "MARS", "JUPITER", "SATURN", "URANUS", "NEPTUNE",
];
const CLASS_OF: &[(&str, &str)] = &[
    ("MERCURY", "ROCKY"),
    ("VENUS", "ROCKY"),
    ("EARTH", "ROCKY"),
    ("MARS", "ROCKY"),
    ("JUPITER", "GAS_GIANT"),
    ("SATURN", "GAS_GIANT"),
    ("URANUS", "ICE_GIANT"),
    ("NEPTUNE", "ICE_GIANT"),
];
const MOONS_OF: &[(&str, &str)] = &[
    ("EARTH", "LUNA"),
    ("MARS", "PHOBOS"),
    ("MARS", "DEIMOS"),
    ("JUPITER", "IO"),
    ("JUPITER", "EUROPA"),
    ("JUPITER", "GANYMEDE"),
    ("JUPITER", "CALLISTO"),
    ("SATURN", "TITAN"),
    ("NEPTUNE", "TRITON"),
];
const PROBES: &[&str] = &["VOYAGER_1", "VOYAGER_2", "PIONEER_10"];
const SUN: &str = "SUN";
const HOLDOUT: &[&str] = &["URANUS", "NEPTUNE"];

fn id(name: &str) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(cyber_hemera::hash(name.as_bytes()).as_bytes());
    h
}

fn link(from: &str, to: &str) -> Cyberlink {
    Cyberlink {
        neuron: id("neuron-demo"),
        from: id(from),
        to: id(to),
        token: 1,
        amount: 1,
        valence: 1,
        block: 1,
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn sentence(rng: &mut Rng) -> Vec<&'static str> {
    match rng.below(7) {
        0 => vec![PLANETS[rng.below(8)], "orbits", SUN],
        1 => {
            let (p, c) = CLASS_OF[rng.below(CLASS_OF.len())];
            vec![p, "is", c]
        }
        2 => {
            let &(_, m) = &MOONS_OF[rng.below(MOONS_OF.len())];
            let p = MOONS_OF.iter().find(|&&(_, mm)| mm == m).unwrap().0;
            vec![m, "orbits", p]
        }
        3 => {
            let &(p, m) = &MOONS_OF[rng.below(MOONS_OF.len())];
            vec![p, "harbors", m]
        }
        4 => {
            let a = PLANETS[rng.below(8)];
            let mut b = PLANETS[rng.below(8)];
            while b == a {
                b = PLANETS[rng.below(8)];
            }
            vec![PROBES[rng.below(PROBES.len())], "visited", a, b]
        }
        5 => {
            let i = rng.below(PLANETS.len() - 1);
            if rng.below(2) == 0 {
                vec![PLANETS[i], "neighbors", PLANETS[i + 1]]
            } else {
                vec![PLANETS[i + 1], "neighbors", PLANETS[i]]
            }
        }
        _ => vec![SUN, "is", "STAR"],
    }
}

// ---------- f64 linear algebra (eval-side; same discipline as eval_ct0) ----------

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn orthonormalize(q: &mut [Vec<f64>]) {
    for i in 0..q.len() {
        for j in 0..i {
            let d = dot(&q[j], &q[i]);
            for c in 0..q[i].len() {
                q[i][c] -= d * q[j][c];
            }
        }
        let nrm = dot(&q[i], &q[i]).sqrt();
        if nrm > 1e-12 {
            for c in q[i].iter_mut() {
                *c /= nrm;
            }
        }
    }
}

/// Truncated SVD, components as rows (u[c][i], v[c][i]). Rayleigh clamped —
/// an unclamped sqrt of a slightly-negative quotient is a NaN that reads as
/// "perfect rank 1" downstream (lesson from eval_ct0).
fn svd(m: &[Vec<f64>], k: usize, iters: usize) -> (Vec<Vec<f64>>, Vec<f64>, Vec<Vec<f64>>) {
    let n = m.len();
    let k = k.min(n);
    let mt: Vec<Vec<f64>> = (0..n).map(|j| (0..n).map(|i| m[i][j]).collect()).collect();
    let mut v: Vec<Vec<f64>> = (0..k)
        .map(|c| {
            (0..n)
                .map(|j| (((c + 1) * (j + 3)) % 7) as f64 * 0.1 + 0.05)
                .collect()
        })
        .collect();
    for _ in 0..iters {
        let mut q: Vec<Vec<f64>> = v.iter().map(|vc| matvec(m, vc)).collect();
        orthonormalize(&mut q);
        v = q.iter().map(|qc| matvec(&mt, qc)).collect();
        orthonormalize(&mut v);
    }
    let mut sigma = Vec::new();
    let mut u = Vec::new();
    for vc in &v {
        let mv = matvec(m, vc);
        let s = dot(&mv, &mv).max(0.0).sqrt(); // sigma = ||M v||: symmetric-only shortcut under-reads on directed M
        sigma.push(s);
        u.push(if s > 1e-12 {
            mv.iter().map(|x| x / s).collect()
        } else {
            vec![0.0; n]
        });
    }
    (u, sigma, v)
}

fn matvec(m: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    m.iter().map(|row| dot(row, x)).collect()
}

fn matmul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let bt: Vec<Vec<f64>> = (0..b[0].len())
        .map(|j| (0..n).map(|i| b[i][j]).collect())
        .collect();
    a.iter()
        .map(|row| bt.iter().map(|col| dot(row, col)).collect())
        .collect()
}

/// Moore-Penrose pseudoinverse via SVD with a relative cutoff.
fn pinv(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = m.len();
    let (u, s, v) = svd(m, n, 300);
    let smax = s.iter().cloned().fold(0.0, f64::max);
    let cutoff = smax * 1e-8;
    // pinv = V diag(1/s) Uᵀ
    let mut inv_sigma = vec![vec![0.0; n]; n];
    for c in 0..n {
        if s[c] > cutoff {
            inv_sigma[c][c] = 1.0 / s[c];
        }
    }
    let v_mat: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|c| v[c][i]).collect()).collect();
    let u_mat: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|c| u[c][i]).collect()).collect();
    let ut: Vec<Vec<f64>> = (0..n)
        .map(|c| (0..n).map(|i| u_mat[i][c]).collect())
        .collect();
    matmul(&matmul(&v_mat, &inv_sigma), &ut)
}

fn rmsnorm(x: &[f64]) -> Vec<f64> {
    let ms = dot(x, x) / x.len() as f64;
    let inv = 1.0 / (ms + 1e-5).sqrt(); // glia Op::RmsNorm { eps: 1e-5 }
    x.iter().map(|v| v * inv).collect()
}

fn matapply(w: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    // x @ W where W is (in, out) stored row-major as w[in][out]
    let out_dim = w[0].len();
    let mut y = vec![0.0; out_dim];
    for (i, &xi) in x.iter().enumerate() {
        for j in 0..out_dim {
            y[j] += xi * w[i][j];
        }
    }
    y
}

fn rope(x: &mut [f64], pos: usize, theta: f64) {
    let d = x.len();
    let mut i = 0;
    while i + 1 < d {
        let freq = theta.powf(-(i as f64) / d as f64);
        let ang = pos as f64 * freq;
        let (c, s) = (ang.cos(), ang.sin());
        let (a, b) = (x[i], x[i + 1]);
        x[i] = a * c - b * s;
        x[i + 1] = a * s + b * c;
        i += 2;
    }
}

// ---------- CT-0 forward ----------

struct Layer {
    wq: Vec<Vec<f64>>, // (d, d), rows = input dim
    wk: Vec<Vec<f64>>,
    wv: Vec<Vec<f64>>,
    wo: Vec<Vec<f64>>,
}

#[derive(Clone, Copy, PartialEq)]
enum Qk {
    /// spec §7: SVD of P = Eᵀ A^(l_eff) E (self-transitions smeared in).
    P,
    /// fix: SVD of P_dir = Eᵀ S E where S[t,s] = walks s→t at length
    /// l_eff and S[t,t] = 0 — the query's self-score has no transition mass,
    /// so causal softmax is forced to retrieve from the prefix. U for Q and
    /// V for K also makes scores directed (the pass-4 V-side fix).
    Sdir,
}

#[derive(Clone, Copy, PartialEq)]
enum Wo {
    /// spec §7.5: pinv(W_V) — inverts the value map back (block ≈ identity).
    Pinv,
    /// identity output — the block applies the graph step to the retrieved
    /// token instead of undoing it (residual carries the transition).
    Identity,
    /// retrieval head: W_V = I and W_O = c·I — the attention output is the
    /// gain-scaled retrieved token itself; its full embedding geometry votes
    /// in the tied head (2-hop signal in E reaches the logits directly).
    Inject,
}

#[allow(clippy::too_many_arguments)]
fn build_layer(
    e: &[Vec<f64>],
    phi: &[f64],
    a: &[Vec<f64>],
    l_eff: usize,
    k_svd: usize,
    qk: Qk,
    wo_mode: Wo,
    kg: f64,
    wo_scale: f64,
) -> Layer {
    let d = e[0].len();
    let v = e.len();
    // A^{l_eff}
    let mut apow = a.to_vec();
    for _ in 1..l_eff {
        apow = matmul(&apow, a);
    }
    // score-source matrix in token space
    let src: Vec<Vec<f64>> = match qk {
        Qk::P => apow.clone(),
        Qk::Sdir => {
            let mut s_dir = vec![vec![0.0; v]; v];
            for t in 0..v {
                for s in 0..v {
                    if s != t {
                        s_dir[t][s] = apow[s][t]; // walks s -> t
                    }
                }
            }
            s_dir
        }
    };
    // P = Eᵀ src E
    let et: Vec<Vec<f64>> = (0..d).map(|c| (0..v).map(|i| e[i][c]).collect()).collect();
    let e_src = {
        let mut m = vec![vec![0.0; v]; d];
        for c in 0..d {
            for i in 0..v {
                m[c][i] = (0..v).map(|j| et[c][j] * src[j][i]).sum();
            }
        }
        m
    };
    let mut p = vec![vec![0.0; d]; d];
    for c in 0..d {
        for cc in 0..d {
            p[c][cc] = (0..v).map(|i| e_src[c][i] * e[i][cc]).sum();
        }
    }
    let (u, s, vv) = svd(&p, k_svd, 300);
    // W_Q = U sqrt(Sigma) as (in, out): w[i][c] = u[c][i] sqrt(s[c]); kg is
    // the score gain (applied to both, so Q·K scales by kg^2 — restoring the
    // softmax dynamic range that normalizing A squeezed out).
    let k = u.len();
    // glia [out, in] layout: matapply computes x @ W^T, so row = output index.
    let mut wq = vec![vec![0.0; d]; k];
    let mut wk = vec![vec![0.0; d]; k];
    for i in 0..d {
        for c in 0..k {
            let sc = s[c].sqrt() * kg;
            wq[c][i] = u[c][i] * sc;
            wk[c][i] = vv[c][i] * sc;
        }
    }
    // W_V = Eᵀ diag(phi) A E  (d, d)
    let mut wv = vec![vec![0.0; d]; d];
    for c in 0..d {
        for cc in 0..d {
            let mut acc = 0.0;
            for i in 0..v {
                let arow_e: f64 = (0..v).map(|j| a[i][j] * e[j][cc]).sum();
                acc += et[c][i] * phi[i] * arow_e;
            }
            wv[c][cc] = acc;
        }
    }
    let wo = match wo_mode {
        Wo::Pinv => pinv(&wv),
        Wo::Identity | Wo::Inject => {
            let mut id = vec![vec![0.0; d]; d];
            for (i, row) in id.iter_mut().enumerate() {
                row[i] = 1.0;
            }
            id
        }
    };
    if wo_mode == Wo::Inject {
        // W_V = I: value passes the (normalized) hidden state through.
        for (i, row) in wv.iter_mut().enumerate() {
            for (j, x) in row.iter_mut().enumerate() {
                *x = if i == j { 1.0 } else { 0.0 };
            }
        }
    }
    // calibrated output gain: the retrieved signal must be able to outvote
    // the last token's frequency prior; final rmsnorm makes logits depend on
    // the residual/output mix ratio only, so a scalar on W_O is exact.
    let wo: Vec<Vec<f64>> = wo
        .iter()
        .map(|r| r.iter().map(|x| x * wo_scale).collect())
        .collect();
    Layer { wq, wk, wv, wo }
}

struct ForwardOut {
    /// logits[p][j]: prediction of token j at position p+1, from context <= p.
    logits: Vec<Vec<f64>>,
    /// back[p]: attention mass of query p on position p-1 (the 2-back
    /// disambiguator), averaged over layers.
    back: Vec<f64>,
}

/// Llama-semantics forward. Query at p predicts token p+1 (standard LM
/// indexing — no self-leak through the tied head).
fn forward(layers: &[Layer], emb: &[Vec<f64>], seq: &[usize], use_rope: bool) -> ForwardOut {
    let d = emb[0].len();
    let v = emb.len();
    let mut h: Vec<Vec<f64>> = seq.iter().map(|&t| emb[t].clone()).collect();
    let mut back = vec![0.0; seq.len()];
    for l in layers {
        // pre-attention RMSNorm
        let hn: Vec<Vec<f64>> = h.iter().map(|x| rmsnorm(x)).collect();
        let mut q: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wq, x)).collect();
        let mut k: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wk, x)).collect();
        let val: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wv, x)).collect();
        if use_rope {
            for t in 0..seq.len() {
                rope(&mut q[t], t, 10000.0);
                rope(&mut k[t], t, 10000.0);
            }
        }
        // causal attention
        let mut ctx = vec![vec![0.0; d]; seq.len()];
        for t in 0..seq.len() {
            let mut scores = vec![0.0f64; t + 1];
            for s in 0..=t {
                scores[s] = dot(&q[t], &k[s]) / (d as f64).sqrt();
            }
            let mx = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let exps: Vec<f64> = scores.iter().map(|x| (x - mx).exp()).collect();
            let z: f64 = exps.iter().sum();
            for s in 0..=t {
                let w = exps[s] / z;
                if s + 1 == t {
                    back[t] += w;
                }
                for c in 0..d {
                    ctx[t][c] += w * val[s][c];
                }
            }
        }
        // output projection + residual
        for t in 0..seq.len() {
            let o = matapply(&l.wo, &ctx[t]);
            for c in 0..d {
                h[t][c] += o[c];
            }
        }
        // MLP: no-op at compile init (LayerScale gamma = 1e-5)
    }
    let final_norm: Vec<Vec<f64>> = h.iter().map(|x| rmsnorm(x)).collect();
    // tied lm_head: logits_j = h · E_j
    let logits = (0..seq.len())
        .map(|t| (0..v).map(|j| dot(&final_norm[t], &emb[j])).collect())
        .collect();
    let nl = layers.len().max(1) as f64;
    for b in back.iter_mut() {
        *b /= nl;
    }
    ForwardOut { logits, back }
}

// ---------- eval ----------

fn main() {
    let mut rng = Rng(0x5eed);
    let corpus: Vec<Vec<&'static str>> = (0..4000).map(|_| sentence(&mut rng)).collect();
    // hold out orbit sentences of the two ice giants (2-back probes need them unseen)
    let held = |s: &[&str]| s.contains(&"orbits") && s.iter().any(|w| HOLDOUT.contains(w));
    let test: Vec<&Vec<&'static str>> = corpus.iter().filter(|s| held(s)).collect();
    let train: Vec<&Vec<&'static str>> = corpus.iter().filter(|s| !held(s)).collect();

    let mut vocab: Vec<&'static str> = PLANETS.to_vec();
    vocab.push(SUN);
    vocab.extend(MOONS_OF.iter().map(|&(_, m)| m));
    vocab.extend(CLASS_OF.iter().map(|&(_, c)| c));
    vocab.extend(PROBES);
    vocab.extend(["orbits", "is", "harbors", "visited", "neighbors", "STAR"]);
    vocab.sort();
    vocab.dedup();
    let vpos: HashMap<&str, usize> = vocab.iter().enumerate().map(|(i, w)| (*w, i)).collect();
    let v = vocab.len();

    let train_links: Vec<Cyberlink> = train
        .iter()
        .flat_map(|s| s.windows(2).map(|w| link(w[0], w[1])))
        .collect();
    let (particles, edges, adj) = index::build(&[], &train_links);
    let a_arch = arch::compute(&adj, 1, 1);
    let idx = |name: &str| particles.idx(&id(name)).unwrap() as usize;

    // content-space count matrix, Fx-normalized by max weight — same
    // discipline as tru's DialectAdj; raw counts (1e2-1e3) would swamp the
    // residual stream and saturate every calibration knob.
    let mut bigram = vec![vec![0.0f64; v]; v];
    let mut present = vec![false; v];
    for e in &edges {
        if e.stake <= 0 {
            continue;
        }
        let i = NAME_OF(&particles.particle(e.src)).and_then(|f| vpos.get(f).copied());
        let j = NAME_OF(&particles.particle(e.tgt)).and_then(|t| vpos.get(t).copied());
        if let (Some(i), Some(j)) = (i, j) {
            bigram[i][j] += e.stake as f64;
            present[i] = true;
            present[j] = true;
        }
    }
    // phi over content words
    let mut phi = vec![0.0f64; v];
    for (w, &i) in &vpos {
        phi[i] = a_arch.phi[idx(w)].to_f64();
    }
    let phisum: f64 = phi.iter().sum();
    for p in phi.iter_mut() {
        *p /= phisum;
    }

    let maxw = bigram
        .iter()
        .flatten()
        .cloned()
        .fold(0.0f64, f64::max)
        .max(1.0);
    let a_norm: Vec<Vec<f64>> = bigram
        .iter()
        .map(|r| r.iter().map(|x| x / maxw).collect())
        .collect();
    // E = U sqrt(Sigma) of the SHIPPED mixed matrix (tru#2):
    // M = diag(sqrt(phi)) (A + 0.5 A^2) diag(sqrt(phi)), full content rank.
    let sqp: Vec<f64> = phi.iter().map(|p| p.sqrt()).collect();
    let a2 = matmul(&a_norm, &a_norm);
    let mut m = vec![vec![0.0; v]; v];
    for i in 0..v {
        for j in 0..v {
            let mixed = a_norm[i][j] + 0.5 * a2[i][j];
            if mixed > 0.0 {
                m[i][j] = sqp[i] * mixed * sqp[j];
            }
        }
    }
    let d = v; // eval-side: no 64 clamp on a 30-word vocab
    let (u, s, _vv) = svd(&m, d, 300);
    let emb: Vec<Vec<f64>> = (0..v)
        .map(|i| (0..d).map(|c| u[c][i] * s[c].sqrt()).collect())
        .collect();
    // diameter (exact, tiny graph) for l_eff schedule
    let diam = diameter(&bigram);
    let l_total = 4usize;
    let leff = |l: usize| 1 + (l * diam) / l_total;
    let build_stack = |e: &[Vec<f64>], qk: Qk, wo: Wo, kg: f64, og: f64| -> Vec<Layer> {
        (0..l_total)
            .map(|l| build_layer(e, &phi, &a_norm, leff(l), d, qk, wo, kg, og))
            .collect()
    };

    println!("=== CT-0 pass 5 eval — llama forward over compiled weights ===");
    println!(
        "vocab {v} · d {d} · L {l_total} · diam {diam} · l_eff {:?}",
        (0..l_total).map(leff).collect::<Vec<_>>()
    );
    println!("train {} · test {} sentences", train.len(), test.len());

    // probes: positions needing 2-back context
    let is_moon = |w: &str| MOONS_OF.iter().any(|&(_, m)| m == w);
    let is_planet = |w: &str| PLANETS.contains(&w);
    let amb2 = |s: &[&str], t: usize| -> bool {
        if t == 0 {
            return false;
        }
        let prev = s[t - 1];
        (prev == "orbits" && t >= 2 && (is_moon(s[t - 2]) || is_planet(s[t - 2])))
            || (prev == "harbors" && t >= 2 && is_planet(s[t - 2]))
    };

    let bigram_score = |i: usize, j: usize| bigram[i][j] + 0.1; // add-k

    // sanity: all scores finite (NaN discipline from eval_ct0)
    let check_finite = |name: &str, logits: &[Vec<f64>]| {
        let bad = logits.iter().flatten().filter(|x| !x.is_finite()).count();
        if bad > 0 {
            println!("!! {name}: {bad} non-finite logits");
        }
    };

    // config space: qk source x W_O x score-gain x output-gain. Two separate
    // calibrations: score-gain restores the softmax dynamic range that A
    // normalization killed (back-mass went 0.59 -> 0.50 = uniform), output-gain
    // sets the attention/residual mix. All weight-only, standard llama forward.
    struct Cfg {
        name: &'static str,
        qk: Qk,
        wo: Wo,
        kg: f64,
        og: f64,
        rope: bool,
    }
    let cfgs = [
        Cfg {
            name: "Sdir+pinv k1 o1",
            qk: Qk::Sdir,
            wo: Wo::Pinv,
            kg: 1.0,
            og: 1.0,
            rope: false,
        },
        Cfg {
            name: "Sdir+ident k1 o1",
            qk: Qk::Sdir,
            wo: Wo::Identity,
            kg: 1.0,
            og: 1.0,
            rope: false,
        },
        Cfg {
            name: "Sdir+ident k10 o1",
            qk: Qk::Sdir,
            wo: Wo::Identity,
            kg: 10.0,
            og: 1.0,
            rope: false,
        },
        Cfg {
            name: "Sdir+ident k30 o1",
            qk: Qk::Sdir,
            wo: Wo::Identity,
            kg: 30.0,
            og: 1.0,
            rope: false,
        },
        Cfg {
            name: "Sdir+ident k30 o8",
            qk: Qk::Sdir,
            wo: Wo::Identity,
            kg: 30.0,
            og: 8.0,
            rope: false,
        },
        Cfg {
            name: "Sdir+ident k30 o1 rope",
            qk: Qk::Sdir,
            wo: Wo::Identity,
            kg: 30.0,
            og: 1.0,
            rope: true,
        },
        Cfg {
            name: "P+ident k30 o8",
            qk: Qk::P,
            wo: Wo::Identity,
            kg: 30.0,
            og: 8.0,
            rope: false,
        },
        Cfg {
            name: "inject c4",
            qk: Qk::Sdir,
            wo: Wo::Inject,
            kg: 30.0,
            og: 4.0,
            rope: false,
        },
        Cfg {
            name: "inject c13",
            qk: Qk::Sdir,
            wo: Wo::Inject,
            kg: 30.0,
            og: 13.0,
            rope: false,
        },
        Cfg {
            name: "inject c30",
            qk: Qk::Sdir,
            wo: Wo::Inject,
            kg: 30.0,
            og: 30.0,
            rope: false,
        },
    ];
    let mut reps: Vec<(String, Report, Report)> = Vec::new();
    for c in &cfgs {
        let e: &[Vec<f64>] = &emb;
        let layers = build_stack(e, c.qk, c.wo, c.kg, c.og);
        let mut amb = Report::default();
        let mut oth = Report::default();
        for s in &test {
            let seq: Vec<usize> = s.iter().map(|w| vpos[w]).collect();
            let out = forward(&layers, e, &seq, c.rope);
            check_finite(c.name, &out.logits);
            for t in 1..s.len() {
                let j = seq[t];
                let ctx1 = seq[t - 1];
                if !present[ctx1] {
                    continue;
                }
                let rep = if amb2(s, t) { &mut amb } else { &mut oth };
                rep.bigram += rr(|jj| bigram_score(ctx1, jj), j, v);
                rep.embed += rr(|jj| dot(&e[ctx1], &e[jj]), j, v);
                rep.ct0 += rr(|jj| out.logits[t - 1][jj], j, v);
                rep.back += out.back[t - 1];
                rep.n += 1;
            }
        }
        reps.push((c.name.to_string(), amb, oth));
    }
    for (name, amb, oth) in &reps {
        amb.print(name);
        oth.print(name);
    }

    // --- norms: is the attention output actually zero? ---
    {
        let e: &[Vec<f64>] = &emb;
        let layers = build_stack(e, Qk::Sdir, Wo::Identity, 30.0, 8.0);
        let seq: Vec<usize> = ["TITAN", "orbits", "SATURN"]
            .iter()
            .map(|w| vpos[w])
            .collect();
        let mut h: Vec<Vec<f64>> = seq.iter().map(|&t| e[t].clone()).collect();
        println!("\n=== layer norms (Sdir+ident k30 o8) ===");
        for (li, l) in layers.iter().enumerate() {
            let hn: Vec<Vec<f64>> = h.iter().map(|x| rmsnorm(x)).collect();
            let val: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wv, x)).collect();
            let k: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wk, x)).collect();
            let q: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wq, x)).collect();
            // attention weights for query 1
            let sc: Vec<f64> = (0..=1)
                .map(|s| dot(&q[1], &k[s]) / (d as f64).sqrt())
                .collect();
            let mx = sc.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let ex: Vec<f64> = sc.iter().map(|x| (x - mx).exp()).collect();
            let z: f64 = ex.iter().sum();
            let w: Vec<f64> = ex.iter().map(|x| x / z).collect();
            let ctx: Vec<f64> = (0..d)
                .map(|c| (0..=1).map(|s| w[s] * val[s][c]).sum())
                .collect();
            let o = matapply(&l.wo, &ctx);
            let nrm = |x: &[f64]| dot(x, x).sqrt();
            println!(
                "L{li}: |val0| {:.2e} |val1| {:.2e} w={:.3}/{:.3} |ctx| {:.2e} |o| {:.2e} |h1| {:.2e} |wv_f| {:.2e} |wq_f| {:.2e}",
                nrm(&val[0]),
                nrm(&val[1]),
                w[0],
                w[1],
                nrm(&ctx),
                nrm(&o),
                nrm(&h[1]),
                l.wv.iter().flatten().map(|x| x * x).sum::<f64>().sqrt(),
                l.wq.iter().flatten().map(|x| x * x).sum::<f64>().sqrt()
            );
            for t in 0..h.len() {
                for c in 0..d {
                    h[t][c] += o[c];
                }
            }
        }
    }

    // --- autopsy of the inject head: what direction does h actually take? ---
    {
        let probe2: Vec<usize> = ["TITAN", "orbits", "SATURN"]
            .iter()
            .map(|w| vpos[w])
            .collect();
        let layers = build_stack(&emb, Qk::Sdir, Wo::Inject, 30.0, 13.0);
        let out = forward(&layers, &emb, &probe2, false);
        // reconstruct h by re-running the forward manually is overkill; use
        // logits to infer: compare against pure-direction rankings.
        let cosrank = |target: usize| -> Vec<(usize, f64)> {
            let t = &emb[target];
            let tn = dot(t, t).sqrt();
            let mut r: Vec<(usize, f64)> = (0..v)
                .map(|j| {
                    (
                        j,
                        dot(t, &emb[j]) / tn / dot(&emb[j], &emb[j]).sqrt().max(1e-12),
                    )
                })
                .collect();
            r.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            r
        };
        let mut lg: Vec<(usize, f64)> = out.logits[1].iter().copied().enumerate().collect();
        lg.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        print!("inject c13 logits@orbits top5:");
        for (j, s) in lg.iter().take(5) {
            print!(" {}={:.3}", vocab[*j], s);
        }
        println!();
        print!("cosine-rank by E[TITAN]  top5:");
        for (j, s) in cosrank(probe2[0]).iter().take(5) {
            print!(" {}={:.3}", vocab[*j], s);
        }
        println!();
        print!("cosine-rank by E[orbits] top5:");
        for (j, s) in cosrank(probe2[1]).iter().take(5) {
            print!(" {}={:.3}", vocab[*j], s);
        }
        println!();
    }

    // --- single-sentence autopsy: where do the logits actually come from? ---
    let probe: Vec<usize> = ["TITAN", "orbits", "SATURN"]
        .iter()
        .map(|w| vpos[w])
        .collect();
    let layers = build_stack(&emb, Qk::Sdir, Wo::Identity, 30.0, 8.0);
    let out = forward(&layers, &emb, &probe, false);
    let gold = probe[2];
    println!("\n=== autopsy: TITAN orbits SATURN (Sdir+ident k30 o8, normed A) ===");
    println!(
        "maxw {maxw:.1} a_norm[orbits][SUN] {:.3e} |emb[orbits]| {:.3e} |emb[SUN]| {:.3e}",
        a_norm[probe[1]][vpos["SUN"]],
        dot(&emb[probe[1]], &emb[probe[1]]).sqrt(),
        dot(&emb[vpos["SUN"]], &emb[vpos["SUN"]]).sqrt()
    );
    println!(
        "E dots: orbits·SUN {:.3e} · orbits·SATURN {:.3e} · TITAN·SATURN {:.3e} · TITAN·SUN {:.3e} · TITAN·orbits {:.3e}",
        dot(&emb[probe[1]], &emb[vpos["SUN"]]),
        dot(&emb[probe[1]], &emb[gold]),
        dot(&emb[probe[0]], &emb[gold]),
        dot(&emb[probe[0]], &emb[vpos["SUN"]]),
        dot(&emb[probe[0]], &emb[probe[1]])
    );
    println!(
        "bigram: orbits->SUN {:.0} · orbits->SATURN {:.0}",
        bigram[probe[1]][vpos["SUN"]], bigram[probe[1]][gold]
    );
    println!("bigram: TITAN->? ");
    for j in 0..v {
        if bigram[probe[0]][j] > 0.0 {
            println!("    TITAN->{} {:.0}", vocab[j], bigram[probe[0]][j]);
        }
    }
    println!("back-mass at query 'orbits': {:.3}", out.back[1]);
    let mut ranked: Vec<(usize, f64)> = out.logits[1].iter().copied().enumerate().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    print!("logits@orbits top5:");
    for (j, s) in ranked.iter().take(5) {
        print!(" {}={:.3}", vocab[*j], s);
    }
    println!();
    let rank_of_gold = ranked.iter().position(|(j, _)| *j == gold).unwrap() + 1;
    println!("gold SATURN rank: {rank_of_gold}");
    // residual-only ranking for comparison
    let l_resid: Vec<f64> = (0..v).map(|j| dot(&emb[probe[1]], &emb[j])).collect();
    let mut r2: Vec<(usize, f64)> = l_resid.iter().copied().enumerate().collect();
    r2.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    print!("residual-only top5:");
    for (j, s) in r2.iter().take(5) {
        print!(" {}={:.3}", vocab[*j], s);
    }
    println!();
}

fn rr(f: impl Fn(usize) -> f64, gold: usize, v: usize) -> f64 {
    let sg = f(gold);
    debug_assert!(sg.is_finite(), "non-finite gold score");
    let mut rank = 1usize;
    for jj in 0..v {
        if jj != gold && f(jj) > sg {
            rank += 1;
        }
    }
    1.0 / rank as f64
}

#[derive(Default)]
struct Report {
    bigram: f64,
    embed: f64,
    ct0: f64,
    back: f64,
    n: usize,
}

impl Report {
    fn print(&self, title: &str) {
        if self.n == 0 {
            return;
        }
        let n = self.n as f64;
        println!(
            "{:<14} bigram {:.3} · embed {:.3} · ct0 {:.3} · back-mass {:.3} (n={})",
            title,
            self.bigram / n,
            self.embed / n,
            self.ct0 / n,
            self.back / n,
            self.n
        );
    }
}

fn diameter(a: &[Vec<f64>]) -> usize {
    let v = a.len();
    let mut best = 0;
    for s in 0..v {
        // BFS
        let mut dist = vec![usize::MAX; v];
        dist[s] = 0;
        let mut q = vec![s];
        while let Some(x) = q.pop() {
            for y in 0..v {
                if a[x][y] > 0.0 && dist[y] == usize::MAX {
                    dist[y] = dist[x] + 1;
                    q.push(y);
                }
            }
        }
        let reach = dist.iter().filter(|&&d| d != usize::MAX).count();
        if reach > 1 {
            let ecc = dist
                .iter()
                .cloned()
                .filter(|&d| d != usize::MAX)
                .max()
                .unwrap_or(0);
            best = best.max(ecc);
        }
    }
    best.max(1)
}

thread_local! {
    static NAMES: HashMap<[u8; 32], &'static str> = {
        let mut m = HashMap::new();
        for &n in PLANETS.iter().chain(PROBES.iter()).chain([SUN].iter()) {
            m.insert(id(n), n);
        }
        for &(_, mname) in MOONS_OF {
            m.insert(id(mname), mname);
        }
        for &(_, cname) in CLASS_OF {
            m.insert(id(cname), cname);
        }
        for verb in ["orbits", "is", "harbors", "visited", "neighbors", "STAR"] {
            m.insert(id(verb), verb);
        }
        m
    };
}

fn NAME_OF(p: &[u8; 32]) -> Option<&'static str> {
    NAMES.with(|m| m.get(p).copied())
}
