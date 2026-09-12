//! Toy eval for the CT-0 compile — corpus edition, entity-holdout generalization.
//!
//! Split: every sentence mentioning a held-out entity (URANUS, NEPTUNE, TITAN,
//! TRITON) goes to test. The entities keep their *other* roles in train (class,
//! neighbors, flybys) but their orbit facts are unseen — so next-token after a
//! held-out entity is an unseen bigram that can only be ranked by distributional
//! geometry (the entity must land near its class siblings), not by counts.
//!
//! Variants: unigram · add-k bigram (the "trained" bar) · tru-E (current pass 4,
//! symmetric U√Σ) · directed (U√Σ source, V√Σ target) · ppmi (directed PPMI).
//! k sweep over the truncation rank. Metrics: MRR on next-token ranking, split
//! memorization (seen context) vs generalization (held-out context).
//!
//! Run: cargo run -p cyber-tru --release --example eval_ct0

use std::collections::HashMap;

use tru::Fx;
use tru::graph::Cyberlink;
use tru::pass::{arch, index};

// ---------- world ----------

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
/// entities whose *sentences* are held out — orbit facts unseen in train
const HOLDOUT: &[&str] = &["URANUS", "NEPTUNE", "TITAN", "TRITON"];

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

// ---------- f64 SVD (eval-side; tests the math with exact rank control) ----------

fn svd(m: &[Vec<f64>], k: usize, iters: usize) -> (Vec<Vec<f64>>, Vec<f64>, Vec<Vec<f64>>) {
    let n = m.len();
    let k = k.min(n);
    let mut v: Vec<Vec<f64>> = (0..k)
        .map(|c| {
            (0..n)
                .map(|j| (((c + 1) * (j + 3)) % 7) as f64 * 0.1 + 0.05)
                .collect()
        })
        .collect();
    let mt = transpose(m);
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
        // Rayleigh quotient can go slightly negative under imperfect
        // convergence — clamp, or NaN poisons every downstream comparison
        // (and `x > NaN` is false, which silently reads as "perfect rank 1")
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

fn transpose(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = m.len();
    let mut t = vec![vec![0.0; n]; n];
    for (i, row) in m.iter().enumerate() {
        for (j, &x) in row.iter().enumerate() {
            t[j][i] = x;
        }
    }
    t
}

fn matvec(m: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    m.iter().map(|row| dot(row, x)).collect()
}

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

// ---------- eval ----------

fn main() {
    let mut rng = Rng(0x5eed);
    let corpus: Vec<Vec<&'static str>> = (0..4000).map(|_| sentence(&mut rng)).collect();
    // hold out only ORBIT-fact sentences of held-out entities; their other
    // roles (class, neighbors, flybys) stay in train — this is what makes the
    // held-out context seen but the held-out bigram unseen
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
    let a = arch::compute(&adj, 1, 1);
    // some held-out moons (TITAN, TRITON) appear in zero train sentences:
    // they are not interned at all — treat their φ* as absent
    let content_idx: Vec<Option<usize>> = vocab
        .iter()
        .map(|w| particles.idx(&id(w)).map(|i| i as usize))
        .collect();

    let mut bigram = vec![vec![0.0f64; v]; v];
    let mut outc = vec![0.0f64; v];
    let mut inc = vec![0.0f64; v];
    for e in &edges {
        if e.stake <= 0 {
            continue;
        }
        let i = NAME_OF(&particles.particle(e.src)).and_then(|f| vpos.get(f).copied());
        let j = NAME_OF(&particles.particle(e.tgt)).and_then(|t| vpos.get(t).copied());
        if let (Some(i), Some(j)) = (i, j) {
            bigram[i][j] += e.stake as f64;
            outc[i] += e.stake as f64;
            inc[j] += e.stake as f64;
        }
    }
    let phi: Vec<f64> = content_idx
        .iter()
        .map(|i| i.map(|x| a.phi[x].to_f64()).unwrap_or(0.0))
        .collect();
    let axon_mass: f64 = (0..particles.len())
        .filter(|&i| NAME_OF(&particles.particle(i as u32)).is_none())
        .map(|i| a.phi[i].to_f64())
        .sum();

    // matrices
    let sqp: Vec<f64> = phi.iter().map(|p| p.sqrt()).collect();
    let mut m_tru = vec![vec![0.0; v]; v];
    let total: f64 = bigram.iter().flatten().sum();
    let mut m_ppmi = vec![vec![0.0; v]; v];
    for i in 0..v {
        for j in 0..v {
            if bigram[i][j] > 0.0 {
                m_tru[i][j] = sqp[i] * bigram[i][j] * sqp[j];
                if outc[i] > 0.0 && inc[j] > 0.0 {
                    let x = ((bigram[i][j] * total) / (outc[i] * inc[j])).ln();
                    if x > 0.0 {
                        m_ppmi[i][j] = x;
                    }
                }
            }
        }
    }

    println!("=== CT-0 corpus eval — entity holdout ===");
    println!(
        "train {} · test {} sentences · vocab {} · links {}",
        train.len(),
        test.len(),
        v,
        train_links.len()
    );
    println!("held-out entities: {HOLDOUT:?} — their orbit facts are unseen in train");
    println!(
        "φ* mass on non-content particles (axons): {:.1}%",
        100.0 * axon_mass
    );

    // scoring closure over test positions, per-variant MRR buckets
    let eval_positions = |f: &dyn Fn(usize, usize) -> f64| -> (f64, usize, f64, usize) {
        let (mut mem_mrr, mut mem_n, mut gen_mrr, mut gen_n) = (0.0, 0usize, 0.0, 0usize);
        for s in &test {
            for w in s.windows(2) {
                let i = match vpos.get(w[0]) {
                    Some(&x) => x,
                    None => continue,
                };
                let j = vpos[w[1]];
                if outc[i] == 0.0 {
                    continue;
                }
                let sj = f(i, j);
                let mut rank = 1usize;
                for jj in 0..v {
                    if jj != j && f(i, jj) > sj {
                        rank += 1;
                    }
                }
                if HOLDOUT.contains(&w[0]) {
                    gen_mrr += 1.0 / rank as f64;
                    gen_n += 1;
                } else {
                    mem_mrr += 1.0 / rank as f64;
                    mem_n += 1;
                }
            }
        }
        let m = if mem_n > 0 {
            mem_mrr / mem_n as f64
        } else {
            0.0
        };
        let g = if gen_n > 0 {
            gen_mrr / gen_n as f64
        } else {
            0.0
        };
        (m, mem_n, g, gen_n)
    };

    let k_sm = 0.1;
    let inc_u = inc.clone();
    let bg_k = bigram.clone();
    let oc_k = outc.clone();
    println!(
        "\n{:<18} {:>12} {:>16}",
        "variant", "mem MRR", "gen MRR (unseen)"
    );
    for (name, f) in [
        (
            "unigram",
            Box::new(move |_: usize, j: usize| inc_u[j]) as Box<dyn Fn(usize, usize) -> f64>,
        ),
        (
            "bigram-k",
            Box::new(move |i: usize, j: usize| (bg_k[i][j] + k_sm) / (oc_k[i] + k_sm * v as f64)),
        ),
    ] {
        let (m, mn, g, gn) = eval_positions(&f);
        println!("k=–  {name:<13} {m:>12.3} {g:>9.3} (n={mn}/{gn})");
    }

    for k in [8usize, 16, 24] {
        let (u1, s1, _) = svd(&m_tru, k, 200);
        let (u2, s2, v2) = svd(&m_tru, k, 200);
        let (u3, s3, v3) = svd(&m_ppmi, k, 200);
        let scale = |u: &[Vec<f64>], s: &[f64], i: usize| -> Vec<f64> {
            (0..k)
                .map(|c| {
                    if c < s.len() {
                        u[c][i] * s[c].sqrt()
                    } else {
                        0.0
                    }
                })
                .collect()
        };
        let tru: Vec<Vec<f64>> = (0..v).map(|i| scale(&u1, &s1, i)).collect();
        let dsrc: Vec<Vec<f64>> = (0..v).map(|i| scale(&u2, &s2, i)).collect();
        let dtgt: Vec<Vec<f64>> = (0..v).map(|j| scale(&v2, &s2, j)).collect();
        let psrc: Vec<Vec<f64>> = (0..v).map(|i| scale(&u3, &s3, i)).collect();
        let ptgt: Vec<Vec<f64>> = (0..v).map(|j| scale(&v3, &s3, j)).collect();
        // in-sample check: how well does directed dot rank train edges?
        let mut edge_pairs: Vec<(usize, usize)> = Vec::new();
        for i in 0..v {
            for j in 0..v {
                if bigram[i][j] > 0.0 {
                    edge_pairs.push((i, j));
                }
            }
        }
        let mut rr = Rng(0xd1ec7 as u64);
        let mut sampled = Vec::new();
        while sampled.len() < edge_pairs.len() {
            let i = rr.below(v);
            let j = rr.below(v);
            if i != j && bigram[i][j] == 0.0 {
                sampled.push((i, j));
            }
        }
        let mut wins = 0.0;
        let mut tot = 0.0;
        for &(i, j) in &edge_pairs {
            let s = dot(&dsrc[i], &dtgt[j]);
            for &(a, b) in sampled.iter().take(20) {
                tot += 1.0;
                if s > dot(&dsrc[a], &dtgt[b]) {
                    wins += 1.0;
                }
            }
        }
        println!(
            "  [in-sample] directed edge-vs-nonedge AUC ≈ {:.3} ({} edges)",
            wins / tot,
            edge_pairs.len()
        );
        for (name, f) in [
            (
                "tru-E",
                Box::new(move |i: usize, j: usize| dot(&tru[i], &tru[j]))
                    as Box<dyn Fn(usize, usize) -> f64>,
            ),
            (
                "directed",
                Box::new(move |i: usize, j: usize| dot(&dsrc[i], &dtgt[j])),
            ),
            (
                "ppmi",
                Box::new(move |i: usize, j: usize| dot(&psrc[i], &ptgt[j])),
            ),
        ] {
            let (m, mn, g, gn) = eval_positions(&f);
            println!("k={k:<3} {name:<13} {m:>12.3} {g:>9.3} (n={mn}/{gn})");
        }
    }

    // --- direction probe ---
    // pairs with edges BOTH ways (orbits vs harbors): does the model rank the
    // observed direction above its reverse? tru-E is symmetric by construction
    // and must score exactly at chance here.
    // one-way pairs only: c(i,j) > 0, c(j,i) = 0 — ground truth is an actual
    // observed edge vs its absent reverse, not sampling noise
    let mut pairs = Vec::new();
    for i in 0..v {
        for j in 0..v {
            if i != j && bigram[i][j] > 0.0 && bigram[j][i] == 0.0 {
                pairs.push((i, j));
            }
        }
    }
    println!(
        "\ndirection probe: {} one-way pairs (edge vs absent reverse)",
        pairs.len()
    );
    println!("{:<18} {:>12}", "variant", "dir acc");
    for k in [16usize] {
        let (u1, s1, _) = svd(&m_tru, k, 200);
        let (u2, s2, v2) = svd(&m_tru, k, 200);
        let (u3, s3, v3) = svd(&m_ppmi, k, 200);
        let scale = |u: &[Vec<f64>], s: &[f64], i: usize| -> Vec<f64> {
            (0..k)
                .map(|c| {
                    if c < s.len() {
                        u[c][i] * s[c].sqrt()
                    } else {
                        0.0
                    }
                })
                .collect()
        };
        let tru: Vec<Vec<f64>> = (0..v).map(|i| scale(&u1, &s1, i)).collect();
        let dsrc: Vec<Vec<f64>> = (0..v).map(|i| scale(&u2, &s2, i)).collect();
        let dtgt: Vec<Vec<f64>> = (0..v).map(|j| scale(&v2, &s2, j)).collect();
        let psrc: Vec<Vec<f64>> = (0..v).map(|i| scale(&u3, &s3, i)).collect();
        let ptgt: Vec<Vec<f64>> = (0..v).map(|j| scale(&v3, &s3, j)).collect();
        for (name, f) in [
            (
                "tru-E",
                Box::new(move |i: usize, j: usize| dot(&tru[i], &tru[j]))
                    as Box<dyn Fn(usize, usize) -> f64>,
            ),
            (
                "directed",
                Box::new(move |i: usize, j: usize| dot(&dsrc[i], &dtgt[j])),
            ),
            (
                "ppmi",
                Box::new(move |i: usize, j: usize| dot(&psrc[i], &ptgt[j])),
            ),
        ] {
            let mut acc = 0.0;
            for &(i, j) in &pairs {
                let (fwd, rev) = (f(i, j), f(j, i));
                if fwd > rev {
                    acc += 1.0;
                } else if fwd == rev {
                    acc += 0.5;
                }
            }
            println!("k={k:<3} {name:<13} {:>8.3}", acc / pairs.len() as f64);
        }
    }
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
