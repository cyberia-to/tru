//! End-to-end adequacy probe of the SHIPPED CT-0 compile on a real cybergraph.
//!
//! Pipeline: space-pussy cyberlinks (eval/data/pussy_links.jsonl) -> temporal
//! 90/10 split -> the actual passes (`index`, `dialect`, `arch`, `embed`,
//! `attn` with shipped hop2_mix and out_gain) -> a glia-semantics llama
//! forward (multi-head, RMSNorm eps 1e-5, RoPE theta 1e6 per the compiled
//! config; MLP skipped — its LayerScale gamma is 1e-5 at compile init).
//!
//! Task: predict the target of a held-out (temporal test) cyberlink given a
//! train-graph walk prefix ending at its source. Floors on identical queries:
//! unigram (phi*), add-k bigram, pass-4 embed geometry <E_f, E_j>, and a
//! zero-layer control (embedding -> final norm -> tied head) that must match
//! the embed floor — plumbing check.
//!
//! Run: cargo run -p cyber-tru --release --example eval_pussy

use std::collections::HashMap;

use tru::Fx;
use tru::graph::Cyberlink;
use tru::pass::{arch, attn, dialect, embed, index};

// ---------- small utils ----------

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

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn rmsnorm(x: &[f64]) -> Vec<f64> {
    let ms = dot(x, x) / x.len() as f64;
    let inv = 1.0 / (ms + 1e-5).sqrt();
    x.iter().map(|v| v * inv).collect()
}

fn matapply(w: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    // w rows = input dim (glia convention: y = W x with weight (out, in))
    w.iter().map(|row| dot(row, x)).collect()
}

fn rope(x: &mut [f64], pos: usize, theta: f64) {
    for c in (0..x.len() - 1).step_by(2) {
        let f = theta.powf(-(c as f64) / x.len() as f64);
        let a = pos as f64 * f;
        let (s, co) = (a.sin(), a.cos());
        let (a0, a1) = (x[c], x[c + 1]);
        x[c] = a0 * co - a1 * s;
        x[c + 1] = a0 * s + a1 * co;
    }
}

fn fx(m: &[Vec<Fx>]) -> Vec<Vec<f64>> {
    m.iter()
        .map(|r| r.iter().map(|x| x.to_f64()).collect())
        .collect()
}

// ---------- forward ----------

struct Layer {
    wq: Vec<Vec<f64>>,
    wk: Vec<Vec<f64>>,
    wv: Vec<Vec<f64>>,
    wo: Vec<Vec<f64>>,
}

/// Llama-semantics multi-head forward. Query at p predicts token p+1 (no
/// self-leak through the tied head). Returns per-position logits and the
/// attention mass on the previous position at the last query.
fn forward(layers: &[Layer], emb: &[Vec<f64>], seq: &[usize], h: usize) -> (Vec<f64>, f64) {
    let d = emb[0].len();
    let d_h = d / h;
    let v = emb.len();
    let mut hs: Vec<Vec<f64>> = seq.iter().map(|&t| emb[t].clone()).collect();
    let mut back = 0.0;
    let nl = layers.len().max(1) as f64;
    for l in layers {
        let hn: Vec<Vec<f64>> = hs.iter().map(|x| rmsnorm(x)).collect();
        let mut q: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wq, x)).collect();
        let mut k: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wk, x)).collect();
        let val: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wv, x)).collect();
        // rope: theta = 1e6 per the compiled config — near-identity at these
        // lengths, but applied properly (in place, q and k).
        for t in 0..seq.len() {
            rope(&mut q[t], t, 1e6);
            rope(&mut k[t], t, 1e6);
        }
        let mut ctx = vec![vec![0.0; d]; seq.len()];
        for t in 0..seq.len() {
            for hd in 0..h {
                let qb = &q[t][hd * d_h..(hd + 1) * d_h];
                let mut scores = vec![0.0f64; t + 1];
                for s in 0..=t {
                    let kb = &k[s][hd * d_h..(hd + 1) * d_h];
                    scores[s] = dot(qb, kb) / (d_h as f64).sqrt();
                }
                let mx = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let exps: Vec<f64> = scores.iter().map(|x| (x - mx).exp()).collect();
                let z: f64 = exps.iter().sum();
                for s in 0..=t {
                    let w = exps[s] / z;
                    if s + 1 == t {
                        back += w / (h as f64);
                    }
                    for c in 0..d_h {
                        ctx[t][hd * d_h + c] += w * val[s][hd * d_h + c];
                    }
                }
            }
        }
        for t in 0..seq.len() {
            let o = matapply(&l.wo, &ctx[t]);
            for c in 0..d {
                hs[t][c] += o[c];
            }
        }
    }
    let last = rmsnorm(&hs[seq.len() - 1]);
    let logits: Vec<f64> = (0..v).map(|j| dot(&last, &emb[j])).collect();
    (logits, back / nl)
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "eval/data/pussy_links.jsonl".into());
    let mut links: Vec<(u64, String, String)> = Vec::new();
    for line in std::fs::read_to_string(&path).unwrap().lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // fixed shape: {"h": <height>, "f": "<cid>", "t": "<cid>"} — cid
        // text has no escapes, so a split parse is exact.
        let h: u64 = line
            .split_once("\"h\": ")
            .and_then(|(_, r)| r.split(',').next())
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let cid = |key: &str| -> String {
            line.split_once(&format!("\"{key}\": \""))
                .and_then(|(_, r)| r.split_once('\"'))
                .unwrap()
                .0
                .to_string()
        };
        links.push((h, cid("f"), cid("t")));
    }
    links.sort_by_key(|l| l.0);
    let split = links.len() * 9 / 10;
    let (train, test) = links.split_at(split);
    println!("=== CT-0 shipped-compile adequacy — space-pussy ===");
    println!(
        "links {} (train {} · test {}) heights {}..{}",
        links.len(),
        train.len(),
        test.len(),
        links[0].0,
        links[links.len() - 1].0
    );

    // particle id: the .graph format carries hemera particle ids; the snapshot
    // handle available here is the CID text, hashed consistently.
    let pid = |c: &str| -> [u8; 32] {
        let mut h = [0u8; 32];
        h.copy_from_slice(cyber_hemera::hash(c.as_bytes()).as_bytes());
        h
    };
    let neuron = pid("neuron-eval");
    let to_cyber = |r: &(u64, String, String)| Cyberlink {
        neuron,
        from: pid(&r.1),
        to: pid(&r.2),
        token: 1,
        amount: 1,
        valence: 1,
        block: r.0,
    };
    let train_links: Vec<Cyberlink> = train.iter().map(to_cyber).collect();

    // ---- the shipped passes ----
    let t0 = std::time::Instant::now();
    use std::io::Write as _;
    let t0 = std::time::Instant::now();
    let (particles, edges, adj) = index::build(&[], &train_links);
    eprintln!("[{:?}] index done", t0.elapsed());
    let dialects = dialect::discover(&particles, &edges);
    eprintln!("[{:?}] dialect done", t0.elapsed());
    let a = arch::compute(&adj, dialects.len(), train[train.len() - 1].0);
    eprintln!("[{:?}] arch done", t0.elapsed());
    println!(
        "arch: d*={} h*={} L*={} diam={} sigma1/sigma_k={:.1} · dialects={} · {} particles",
        a.d,
        a.h,
        a.l,
        a.diameter,
        a.sigma_ratio.to_f64(),
        dialects.len(),
        particles.len()
    );
    use std::io::Write as _;
    std::io::stdout().flush().unwrap();
    let emb_t = embed::embed(&adj, &a.phi, a.d);
    eprintln!("[{:?}] embed done", t0.elapsed());
    let gain = attn::out_gain(a.sigma_ratio);
    let attn_tensors = attn::attention(
        &edges,
        &dialects,
        &emb_t.data,
        &a.phi,
        a.d,
        a.h,
        a.l,
        a.diameter,
        gain,
    );
    println!(
        "compile: d*={} h*={} L*={} diam={} gain={:.3} sigma1/sigma_k={:.1} · dialects={} · {} particles · {:?}",
        a.d,
        a.h,
        a.l,
        a.diameter,
        gain.to_f64(),
        a.sigma_ratio.to_f64(),
        dialects.len(),
        particles.len(),
        t0.elapsed()
    );

    let n = particles.len();
    let d = a.d;
    let mut emb: Vec<Vec<f64>> = vec![vec![0.0; d]; n];
    for i in 0..n {
        for c in 0..d {
            emb[i][c] = emb_t.data[i * d + c].to_f64();
        }
    }
    let layers: Vec<Layer> = (0..a.l)
        .map(|l| {
            let get = |suffix: &str| -> Vec<Vec<f64>> {
                let t = attn_tensors
                    .iter()
                    .find(|t| t.name == format!("model.layers.{l}.self_attn.{suffix}"))
                    .unwrap();
                let m: Vec<Vec<Fx>> = (0..d)
                    .map(|i| t.data[i * d..(i + 1) * d].to_vec())
                    .collect();
                fx(&m)
            };
            Layer {
                wq: get("q_proj.weight"),
                wk: get("k_proj.weight"),
                wv: get("v_proj.weight"),
                wo: get("o_proj.weight"),
            }
        })
        .collect();

    // ---- floors ----
    let mut outc = vec![0.0f64; n];
    let mut phi = vec![0.0f64; n];
    let mut bigram: HashMap<(u32, u32), f64> = HashMap::new();
    let mut preds: Vec<Vec<u32>> = vec![Vec::new(); n];
    for e in &edges {
        if e.stake <= 0 {
            continue;
        }
        let w = e.stake as f64;
        bigram
            .entry((e.src, e.tgt))
            .and_modify(|x| *x += w)
            .or_insert(w);
        outc[e.src as usize] += w;
        preds[e.tgt as usize].push(e.src);
    }
    for i in 0..n {
        phi[i] = a.phi[i].to_f64();
    }
    let k_sm = 0.1;

    // ---- test queries: train-walk prefix -> held-out link ----
    let mut rng = Rng(0x9e3779b97f4a7c15);
    let max_queries = 1500;
    let mut queries: Vec<(Vec<usize>, usize)> = Vec::new(); // (prefix seq incl source, gold target)
    let mut seen = std::collections::HashSet::new();
    'outer: for r in test {
        let (f, t) = (pid(&r.1), pid(&r.2));
        let (fi, ti) = match (particles.idx(&f), particles.idx(&t)) {
            (Some(x), Some(y)) => (x as usize, y as usize),
            _ => continue,
        };
        if preds[fi].is_empty() {
            continue;
        }
        // prefixes: [f] (no context), [b,f], [a,b,f] with train edges a->b, b->f
        for _ in 0..2 {
            let &b = &preds[fi][rng.below(preds[fi].len())];
            if !preds[b as usize].is_empty() {
                let &aa = &preds[b as usize][rng.below(preds[b as usize].len())];
                let seq = vec![aa as usize, b as usize, fi];
                if seen.insert(seq.clone()) {
                    queries.push((seq, ti));
                }
            }
            let seq = vec![b as usize, fi];
            if seen.insert(seq.clone()) {
                queries.push((seq, ti));
            }
        }
        let seq = vec![fi];
        if seen.insert(seq.clone()) {
            queries.push((seq, ti));
        }
        if queries.len() >= max_queries {
            break 'outer;
        }
    }
    println!("scorable queries: {} (prefix lens mixed)", queries.len());

    // dump E + queries for offline arbitration against the scipy mirror
    {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(n as u64).to_le_bytes());
        buf.extend_from_slice(&(d as u64).to_le_bytes());
        for row in &emb {
            for &x in row {
                buf.extend_from_slice(&x.to_le_bytes());
            }
        }
        buf.extend_from_slice(&(queries.len() as u64).to_le_bytes());
        for (seq, gold) in &queries {
            buf.extend_from_slice(&(seq.len() as u64).to_le_bytes());
            for &t in seq {
                buf.extend_from_slice(&(t as u64).to_le_bytes());
            }
            buf.extend_from_slice(&(*gold as u64).to_le_bytes());
        }
        // first TRAIN_L attention layers, raw [out,in] rows as stored in the model
        const TRAIN_L: usize = 4;
        buf.extend_from_slice(&(TRAIN_L.min(a.l) as u64).to_le_bytes());
        for l in 0..TRAIN_L.min(a.l) {
            for suffix in ["q_proj.weight", "k_proj.weight", "v_proj.weight", "o_proj.weight"] {
                let t = attn_tensors
                    .iter()
                    .find(|t| t.name == format!("model.layers.{l}.self_attn.{suffix}"))
                    .unwrap();
                for x in &t.data {
                    buf.extend_from_slice(&x.to_f64().to_le_bytes());
                }
            }
        }
        std::fs::write("/tmp/e2e_dump.bin", &buf).unwrap();
    }
    // ---- score ----
    let mut mrr = HashMap::<&str, f64>::new();
    let mut cnt_m = HashMap::<&str, usize>::new();
    let mut back_sum = 0.0;
    for (seq, gold) in &queries {
        let f = seq[seq.len() - 1];
        let (logits, back) = forward(&layers, &emb, seq, a.h);
        back_sum += back;
        let embed_floor: Vec<f64> = (0..n).map(|j| dot(&emb[f], &emb[j])).collect();
        let zero_layer = {
            let hn = rmsnorm(&emb[f]);
            (0..n).map(|j| dot(&hn, &emb[j])).collect::<Vec<f64>>()
        };
        let models: [(&str, Vec<f64>); 6] = [
            ("fwd-full", logits),
            ("fwd-1layer", {
                if layers.is_empty() {
                    vec![f64::NAN; n]
                } else {
                    forward(&layers[..1], &emb, seq, a.h).0
                }
            }),
            ("zero-layer", zero_layer),
            ("embed-floor", embed_floor),
            (
                "bigram",
                (0..n)
                    .map(|j| {
                        (bigram.get(&(f as u32, j as u32)).copied().unwrap_or(0.0) + k_sm)
                            / (outc[f] + k_sm * n as f64)
                    })
                    .collect(),
            ),
            ("unigram", phi.clone()),
        ];
        for (name, scores) in models {
            debug_assert!(
                scores.iter().all(|s| s.is_finite()),
                "non-finite score in {name} — NaN poisons rank comparisons"
            );
            let sg = scores[*gold];
            let rank = 1 + scores
                .iter()
                .enumerate()
                .filter(|(j, s)| *j != *gold && **s > sg)
                .count();
            *mrr.entry(name).or_insert(0.0) += 1.0 / rank as f64;
            *cnt_m.entry(name).or_insert(0) += 1;
        }
    }
    println!("\n{:<12} {:>8} {:>10}", "model", "MRR", "n");
    for name in [
        "fwd-full",
        "fwd-1layer",
        "zero-layer",
        "embed-floor",
        "bigram",
        "unigram",
    ] {
        let c = cnt_m[name];
        if c > 0 {
            println!("{:<12} {:>8.4} {:>10}", name, mrr[name] / c as f64, c);
        }
    }
    println!(
        "mean attn back-mass at query: {:.3}",
        back_sum / queries.len() as f64
    );
}
