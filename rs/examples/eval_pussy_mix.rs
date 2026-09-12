//! Role-mixing embedding sweep — can attention be made net-positive on real
//! walks by giving every particle BOTH source and target geometry?
//!
//! Problem (eval/e2e_pussy.md): the shipped attention is net-negative (-0.33
//! MRR vs floor). Causal attention on a directed walk cannot retrieve the
//! answer (t is never in the prefix) — but the prefix CAN help if token
//! embeddings carry target-side geometry: then retrieving predecessors
//! injects "where predecessors point" instead of only "what points at them".
//!
//! Pass 4 computes both U sqrt(Sigma) (source geometry, shipped) and
//! V sqrt(Sigma) (target geometry, discarded). This probe blends them:
//!   E_theta = cos(theta) * U sqrt(Sigma) + sin(theta) * V sqrt(Sigma)
//! so the tied-head floor gains the directed term
//!   <V_f sqrt(S), U_j sqrt(S)> = walks f -> j
//! and attention values become role-complete. theta = 0 reproduces the
//! shipped construction; theta = pi/4 is the equal blend.
//!
//! Sweeps: theta x out-gain x score orientation, f64 eval-side mirror of the
//! sparse graph (same discipline as eval_ct0/eval_attn). Attention: SVD of
//! P_dir = E' S E (S[t,s] = walks s->t, diag 0), W_Q = U sqrt(S), W_K =
//! V sqrt(S), W_V = I, W_O = gain * I; glia-semantics forward.
//!
//! Run: cargo run -p cyber-tru --release --example eval_pussy_mix


use tru::graph::Cyberlink;
use tru::pass::{arch, attn, dialect, index};

// ---------- f64 linear algebra (eval-side mirror) ----------

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

/// Randomized SVD of the square operator implied by apply/apply_t.
fn svd_op(
    n: usize,
    apply: &dyn Fn(&[f64]) -> Vec<f64>,
    apply_t: &dyn Fn(&[f64]) -> Vec<f64>,
    k: usize,
    iters: usize,
) -> (Vec<Vec<f64>>, Vec<f64>, Vec<Vec<f64>>) {
    let k = k.min(n);
    let mut v: Vec<Vec<f64>> = (0..k)
        .map(|c| {
            (0..n)
                .map(|j| (((c + 1) * (j + 3)) % 7) as f64 * 0.1 + 0.05)
                .collect()
        })
        .collect();
    for _ in 0..iters {
        let mut q: Vec<Vec<f64>> = v.iter().map(|vc| apply(vc)).collect();
        orthonormalize(&mut q);
        v = q.iter().map(|qc| apply_t(qc)).collect();
        orthonormalize(&mut v);
        orthonormalize(&mut v);
    }
    let mut sigma = Vec::new();
    let mut u = Vec::new();
    for vc in &v {
        let mv = apply(vc);
        // sigma = ||P v|| for a converged unit right singular vector —
        // sqrt(dot(mv, vc)) is only valid for SYMMETRIC operators; for the
        // directed P here it produced rank-2 garbage (sigma ~ 0 on most
        // columns, softmax collapsed to uniform).
        let s = dot(&mv, &mv).max(0.0).sqrt();
        sigma.push(s);
        u.push(if s > 1e-12 {
            mv.iter().map(|x| x / s).collect()
        } else {
            vec![0.0; n]
        });
    }
    (u, sigma, v.clone())
}

fn rmsnorm(x: &[f64]) -> Vec<f64> {
    let ms = dot(x, x) / x.len() as f64;
    let inv = 1.0 / (ms + 1e-5).sqrt();
    x.iter().map(|v| v * inv).collect()
}

fn matapply(w: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
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

// ---------- attention layer from E' and the graph ----------

struct Layer {
    wq: Vec<Vec<f64>>,
    wk: Vec<Vec<f64>>,
    wo: Vec<Vec<f64>>,
    gain: f64,
}

/// P = E' S E where S[t,s] = walks s->t (dir=backward: shipped) or walks
/// t->s (dir=forward), diagonal killed. W_Q/W_K from its SVD.
fn build_layer(e: &[Vec<f64>], rows: &[Vec<(u32, f64)>], dir_fwd: bool, gain: f64) -> Layer {
    let d = e[0].len();
    let v = e.len();
    // T = S * E via sparse rows: S[t,s] = A[s,t] (backward) or A[t,s] (forward)
    let mut t = vec![vec![0.0; d]; v];
    for s in 0..v {
        for &(tt, w) in &rows[s] {
            let (row, col) = if dir_fwd {
                (s, tt as usize)
            } else {
                (tt as usize, s)
            };
            if row != col {
                for c in 0..d {
                    t[row][c] += w * e[col][c];
                }
            }
        }
    }
    let mut p = vec![vec![0.0; d]; d];
    for c in 0..d {
        for cc in 0..d {
            p[c][cc] = (0..v).map(|i| e[i][c] * t[i][cc]).sum();
        }
    }
    let (u, s, vv) = svd_op(
        d,
        &|x| matapply(&p, x),
        &|x| {
            // P^T x
            (0..d)
                .map(|c| (0..d).map(|cc| p[cc][c] * x[cc]).sum())
                .collect()
        },
        d,
        60,
    );
    if std::env::var_os("MIX_DEBUG").is_some() {
        let nf: f64 = p.iter().flatten().map(|x| x * x).sum::<f64>().sqrt();
        eprintln!(
            "dbg layer: ||P||_F={:.5} sigma_P_top={:.5?}",
            nf,
            &s[..s.len().min(5)]
        );
    }
    let mut wq = vec![vec![0.0; d]; d];
    let mut wk = vec![vec![0.0; d]; d];
    for c in 0..d {
        let sc = s[c].sqrt();
        for i in 0..d {
            // glia [out, in] layout — matapply computes x @ W^T
            wq[c][i] = u[c][i] * sc;
            wk[c][i] = vv[c][i] * sc;
        }
    }
    Layer {
        wq,
        wk,
        wo: {
            let mut id = vec![vec![0.0; d]; d];
            for (i, row) in id.iter_mut().enumerate() {
                row[i] = gain;
            }
            id
        },
        gain,
    }
}

fn forward(layers: &[Layer], emb: &[Vec<f64>], seq: &[usize]) -> Vec<f64> {
    let d = emb[0].len();
    let v = emb.len();
    let mut hs: Vec<Vec<f64>> = seq.iter().map(|&t| emb[t].clone()).collect();
    for l in layers {
        let hn: Vec<Vec<f64>> = hs.iter().map(|x| rmsnorm(x)).collect();
        let mut q: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wq, x)).collect();
        let mut k: Vec<Vec<f64>> = hn.iter().map(|x| matapply(&l.wk, x)).collect();
        let val = &hn; // W_V = I
        for t in 0..seq.len() {
            rope(&mut q[t], t, 1e6);
            rope(&mut k[t], t, 1e6);
        }
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
                for c in 0..d {
                    ctx[t][c] += w * val[s][c];
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
    (0..v).map(|j| dot(&last, &emb[j])).collect()
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
        let h: u64 = line
            .split_once("\"h\": ")
            .and_then(|(_, r)| r.split(',').next())
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let cid = |key: &str| -> String {
            line.split_once(&format!("\"{key}\": \""))
                .and_then(|(_, r)| r.split_once('"'))
                .unwrap()
                .0
                .to_string()
        };
        links.push((h, cid("f"), cid("t")));
    }
    links.sort_by_key(|l| l.0);
    let split = links.len() * 9 / 10;
    let (train, test) = links.split_at(split);

    let pid = |c: &str| -> [u8; 32] {
        let mut h = [0u8; 32];
        h.copy_from_slice(cyber_hemera::hash(c.as_bytes()).as_bytes());
        h
    };
    let neuron = pid("neuron-eval");
    let train_links: Vec<Cyberlink> = train
        .iter()
        .map(|r| Cyberlink {
            neuron,
            from: pid(&r.1),
            to: pid(&r.2),
            token: 1,
            amount: 1,
            valence: 1,
            block: r.0,
        })
        .collect();

    let (particles, edges, adj) = index::build(&[], &train_links);
    let dialects = dialect::discover(&particles, &edges);
    let a = arch::compute(&adj, dialects.len(), train[train.len() - 1].0);
    let n = particles.len();
    let d = a.d;
    println!(
        "arch: d*={} h*={} L*={} diam={} · {} particles (shipped probe; layers below are l_eff=1)",
        a.d, a.h, a.l, a.diameter, n
    );

    let phi: Vec<f64> = (0..n).map(|i| a.phi[i].to_f64()).collect();
    // aggregated binary adjacency (weights = stake / wmax; all stakes are 1)
    let mut rows: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    {
        let mut seen: std::collections::HashMap<(u32, u32), f64> = std::collections::HashMap::new();
        for e in &edges {
            if e.stake > 0 {
                *seen.entry((e.src, e.tgt)).or_insert(0.0) += e.stake as f64;
            }
        }
        for ((i, j), w) in seen {
            rows[i as usize].push((j, w));
        }
    }
    // Trusted spectrum: the eval-side subspace iteration collapses the tail
    // (exact zeros from Gram-Schmidt cancellation), so U/sigma/V come from
    // scipy svds on the same operator (eval/mirror_svd.py — the same spine
    // as lp_eval.py). Rust writes the graph, python writes the SVD.
    let graph_bin = "/tmp/pussy_graph.bin";
    let svd_bin = "/tmp/pussy_svd.bin";
    if !std::path::Path::new(svd_bin).exists() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(n as u64).to_le_bytes());
        buf.extend_from_slice(&(rows.iter().map(|r| r.len() as u64).sum::<u64>()).to_le_bytes());
        for (i, row) in rows.iter().enumerate() {
            for &(j, w) in row {
                buf.extend_from_slice(&(i as u32).to_le_bytes());
                buf.extend_from_slice(&j.to_le_bytes());
                buf.extend_from_slice(&w.to_le_bytes());
            }
        }
        for &x in &phi {
            buf.extend_from_slice(&x.to_le_bytes());
        }
        std::fs::write(graph_bin, &buf).unwrap();
        let st = std::process::Command::new("eval/.venv/bin/python3")
            .args(["eval/mirror_svd.py", graph_bin, svd_bin])
            .status()
            .unwrap();
        assert!(st.success(), "mirror_svd.py failed");
    }
    let raw = std::fs::read(svd_bin).unwrap();
    let k_loaded = u64::from_le_bytes(raw[0..8].try_into().unwrap()) as usize;
    assert_eq!(k_loaded, d, "mirror svd width mismatch");
    let f64s = |off: usize, len: usize| -> Vec<f64> {
        raw[off..off + len * 8]
            .chunks_exact(8)
            .map(|b| f64::from_le_bytes(b.try_into().unwrap()))
            .collect()
    };
    let sig = f64s(8, d);
    let u_flat = f64s(8 + d * 8, n * d);
    let v_flat = f64s(8 + d * 8 + n * d * 8, n * d);
    let uu: Vec<Vec<f64>> = (0..d)
        .map(|c| (0..n).map(|i| u_flat[i * d + c]).collect())
        .collect();
    let vv: Vec<Vec<f64>> = (0..d)
        .map(|c| (0..n).map(|i| v_flat[i * d + c]).collect())
        .collect();
    println!("f64 mirror svd spectrum ({} values):", sig.len());
    for (i, s) in sig.iter().enumerate() {
        if i % 8 == 0 {
            println!();
        }
        print!("{i}:{s:.4} ");
    }
    println!();

    // queries (identical construction to eval_pussy.rs)
    let mut preds: Vec<Vec<u32>> = vec![Vec::new(); n];
    for e in &edges {
        if e.stake > 0 {
            preds[e.tgt as usize].push(e.src);
        }
    }
    let mut rng = Rng(0x9e3779b97f4a7c15);
    let max_queries = 1500;
    let mut queries: Vec<(Vec<usize>, usize)> = Vec::new();
    let mut seen_q = std::collections::HashSet::new();
    'outer: for r in test {
        let (f, t) = (pid(&r.1), pid(&r.2));
        let (fi, ti) = match (particles.idx(&f), particles.idx(&t)) {
            (Some(x), Some(y)) => (x as usize, y as usize),
            _ => continue,
        };
        if preds[fi].is_empty() {
            continue;
        }
        for _ in 0..2 {
            let &b = &preds[fi][rng.below(preds[fi].len())];
            if !preds[b as usize].is_empty() {
                let &aa = &preds[b as usize][rng.below(preds[b as usize].len())];
                let seq = vec![aa as usize, b as usize, fi];
                if seen_q.insert(seq.clone()) {
                    queries.push((seq, ti));
                }
            }
            let seq = vec![b as usize, fi];
            if seen_q.insert(seq.clone()) {
                queries.push((seq, ti));
            }
        }
        let seq = vec![fi];
        if seen_q.insert(seq.clone()) {
            queries.push((seq, ti));
        }
        if queries.len() >= max_queries {
            break 'outer;
        }
    }
    println!("scorable queries: {}", queries.len());

    // spectral exponent x role-blend sweep. E = mix(U, V; theta) * diag(sigma^p):
    // p = 1/2 is the shipped sqrt-weighting (columns collapse as sqrt(sigma)
    // dies -> P starved, scores ~0); p = 0 whitens (E = U, all coordinates
    // O(1)) at the cost of the magnitude prior in the tied head.
    let shipped_gain = attn::out_gain(a.sigma_ratio).to_f64();
    println!(
        "\n{:<8} {:>8} {:>12} {:>12} {:>12} {:>12}",
        "p", "theta", "floor", "attn(g*)", "attn(g=1)", "attn(fwd)"
    );
    for &p_exp in &[0.0f64, 0.25, 0.5] {
        for &deg in &[0.0f64, 45.0] {
            let th = deg.to_radians();
            let (ct, st) = (th.cos(), th.sin());
            let mut e: Vec<Vec<f64>> = vec![vec![0.0; d]; n];
            for i in 0..n {
                for c in 0..d {
                    let w = sig[c].powf(p_exp);
                    e[i][c] = (ct * uu[c][i] + st * vv[c][i]) * w;
                }
            }
            // floors
            let mut floor_mrr = 0.0;
            let mut zero_mrr = 0.0;
            for (seq, gold) in &queries {
                let f = seq[seq.len() - 1];
                let sf: Vec<f64> = (0..n).map(|j| dot(&e[f], &e[j])).collect();
                let hn = rmsnorm(&e[f]);
                let sz: Vec<f64> = (0..n).map(|j| dot(&hn, &e[j])).collect();
                for (scores, acc) in [(&sf, &mut floor_mrr), (&sz, &mut zero_mrr)] {
                    debug_assert!(scores.iter().all(|x| x.is_finite()));
                    let sg = scores[*gold];
                    let rank = 1 + scores
                        .iter()
                        .enumerate()
                        .filter(|(j, s)| *j != *gold && **s > sg)
                        .count();
                    *acc += 1.0 / rank as f64;
                }
            }
            // attention variants (3 layers of l_eff=1, single head)
            let variants = [
                ("bwd", false, shipped_gain),
                ("bwd-g1", false, 1.0),
                ("fwd", true, shipped_gain),
            ];
            let mut attn_mrr = [0.0; 3];
            let layers_cache: Vec<Vec<Layer>> = variants
                .iter()
                .map(|&(_, fwd, gain)| (0..3).map(|_| build_layer(&e, &rows, fwd, gain)).collect())
                .collect();
            if std::env::var_os("MIX_DEBUG").is_some() && p_exp == 0.0 && deg == 0.0 {
                for (seq, _) in queries.iter().take(2.min(queries.len())) {
                    let d = e[0].len();
                    let hn: Vec<Vec<f64>> = seq.iter().map(|&t| rmsnorm(&e[t])).collect();
                    let q0 = matapply(&layers_cache[0][0].wq, &hn[seq.len() - 1]);
                    let k0: Vec<Vec<f64>> = hn
                        .iter()
                        .map(|x| matapply(&layers_cache[0][0].wk, x))
                        .collect();
                    let scores: Vec<f64> = (0..seq.len())
                        .map(|s| dot(&q0, &k0[s]) / (d as f64).sqrt())
                        .collect();
                    eprintln!("dbg p=0 scores={scores:?}");
                }
            }
            for (seq, gold) in &queries {
                for (vi, layers) in layers_cache.iter().enumerate() {
                    let logits = forward(layers, &e, seq);
                    debug_assert!(logits.iter().all(|x| x.is_finite()));
                    let sg = logits[*gold];
                    let rank = 1 + logits
                        .iter()
                        .enumerate()
                        .filter(|(j, s)| *j != *gold && **s > sg)
                        .count();
                    attn_mrr[vi] += 1.0 / rank as f64;
                }
            }
            let qn = queries.len() as f64;
            println!(
                "{:<8} {:>8.1} {:>12.4} {:>12.4} {:>12.4} {:>12.4}",
                format!("p={p_exp}"),
                deg,
                floor_mrr / qn,
                attn_mrr[0] / qn,
                attn_mrr[1] / qn,
                attn_mrr[2] / qn,
            );
            if (floor_mrr - zero_mrr).abs() / qn > 1e-9 {
                println!("  WARNING: zero-layer != floor (plumbing)");
            }
        }
    }
}
