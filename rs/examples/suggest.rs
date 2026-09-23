//! the product surface: top-k link suggestions for a particle, served
//! straight from a compiled .model — no training, no runtime state.
//!
//! usage: suggest <model> <CID> [--k N] [--links <links.jsonl>]
//!
//! scores are cosine dot products of the row-normalized compiled
//! embedding (E ships unit-norm, §6.1). with --links, suggestions
//! resolve back to CIDs through the source snapshot (vocab carries
//! hemera ids, which are one-way hashes of the CID text).

use std::collections::HashMap;

use tru::Fx;
use tru::model::{Encoding, Model};

fn cid_id(c: &str) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(cyber_hemera::hash(c.as_bytes()).as_bytes());
    h
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        die("usage: suggest <model-or-index> <CID> [--k N] [--links <links.jsonl>]");
    }
    let model_path = &args[1];
    let cid = &args[2];
    let mut k = 10usize;
    let mut links_path: Option<String> = None;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--k" => {
                i += 1;
                k = args[i].parse().unwrap_or(10);
            }
            "--links" => {
                i += 1;
                links_path = Some(args[i].clone());
            }
            other => die(&format!("unknown flag {other}")),
        }
        i += 1;
    }

    // fast path: a build_index sidecar mmap's in O(1) and scores at
    // memory bandwidth over the f32 embedding. detected by magic —
    // .model files start with the [cyb] frontmatter.
    let magic = std::fs::read(model_path)
        .unwrap_or_else(|e| die(&format!("read: {e}")))
        .into_iter()
        .take(4)
        .collect::<Vec<u8>>();
    if magic != b"[cyb]" {
        fast_path(model_path, cid, k);
        return;
    }

    let t0 = std::time::Instant::now();
    let bytes = std::fs::read(model_path).unwrap_or_else(|e| die(&format!("read model: {e}")));
    let model = Model::from_bytes(&bytes).unwrap_or_else(|e| die(&format!("parse model: {e}")));
    let embed = model
        .tensors
        .iter()
        .find(|t| t.name == "model.embed_tokens.weight")
        .unwrap_or_else(|| die("model has no embed_tokens"));
    if embed.encoding != Encoding::U16 {
        die("unexpected embedding encoding");
    }
    let d = embed.shape[1] as usize;
    let n = embed.shape[0] as usize;
    println!(
        "model {} · vocab {} · d {} ({:?})",
        model.name,
        n,
        d,
        t0.elapsed()
    );

    // vocab: token id -> hemera id hex ("0x..."); build the reverse map
    let mut id2tok: HashMap<[u8; 32], usize> = HashMap::with_capacity(n);
    for line in model.vocab.lines() {
        if let Some((id_s, hex)) = line.split_once(" = ") {
            if let Ok(id) = id_s.trim().parse::<usize>() {
                let hex = hex.trim().trim_matches('"');
                // 256-bit hemera id: parse per-byte (a u128 parse would
                // silently truncate)
                if hex.len() == 66 && hex.starts_with("0x") {
                    let mut b = [0u8; 32];
                    let mut ok = true;
                    for j in 0..32 {
                        match u8::from_str_radix(&hex[2 + 2 * j..4 + 2 * j], 16) {
                            Ok(v) => b[j] = v,
                            Err(_) => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        id2tok.insert(b, id);
                    }
                }
            }
        }
    }

    let want = cid_id(cid);
    let src = match id2tok.get(&want) {
        Some(&t) => t,
        None => die("particle not in vocab"),
    };

    let e_src = &embed.data[src * d..(src + 1) * d];
    // cosine scores: rows are unit-norm at compile (§6.1), dot is the cosine
    let mut scored: Vec<(f64, usize)> = Vec::with_capacity(n);
    for v in 0..n {
        if v == src {
            continue;
        }
        let row = &embed.data[v * d..(v + 1) * d];
        let mut acc = 0.0f64;
        for c in 0..d {
            acc += e_src[c].to_f64() * row[c].to_f64();
        }
        scored.push((acc, v));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("suggestions for {cid} (top {k}):");
    let mut back: HashMap<[u8; 32], String> = HashMap::new();
    if let Some(lp) = &links_path {
        let text = std::fs::read_to_string(lp).unwrap_or_else(|e| die(&format!("links: {e}")));
        for line in text.lines() {
            let f = |key: &str| -> Option<String> {
                line.split_once(&format!("\"{key}\": \""))
                    .and_then(|(_, r)| r.split_once('"'))
                    .map(|(s, _)| s.to_string())
            };
            if let (Some(a), Some(b)) = (f("f"), f("t")) {
                back.insert(cid_id(&a), a.clone());
                back.insert(cid_id(&b), b);
            }
        }
    }
    let tok2id: HashMap<usize, [u8; 32]> = id2tok.iter().map(|(h, &t)| (t, *h)).collect();
    for (score, v) in scored.iter().take(k) {
        let hid_bytes = tok2id.get(v).copied().unwrap_or([0u8; 32]);
        let mut hid = String::from("0x");
        for b in hid_bytes {
            hid.push_str(&format!("{b:02x}"));
        }
        let resolved = back.get(&hid_bytes).cloned().unwrap_or(hid);
        println!("  {score:.4}  {resolved}");
    }
    println!("({:?})", t0.elapsed());
}

fn die(msg: &str) -> ! {
    eprintln!("suggest: {msg}");
    std::process::exit(2)
}

fn fast_path(index_path: &str, cid: &str, k: usize) {
    let t0 = std::time::Instant::now();
    let file = std::fs::File::open(index_path).unwrap_or_else(|e| die(&format!("open: {e}")));
    let map = unsafe { memmap2::Mmap::map(&file) }.unwrap_or_else(|e| die(&format!("mmap: {e}")));
    let u64_at = |o: usize| -> u64 {
        u64::from_le_bytes(map[o..o + 8].try_into().unwrap())
    };
    let n = u64_at(0) as usize;
    let d = u64_at(8) as usize;
    // locate the query token: FNV pre-filter over the hash array
    // (streaming u64 scan), then verify the candidate's cid text.
    let mut h = 0xcbf29ce484222325u64;
    for b in cid.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    let hashes = &map[16 + n * 60 + n * d * 4..];
    let mut src = usize::MAX;
    for tok in 0..n {
        let hv = u64::from_le_bytes(hashes[tok * 8..tok * 8 + 8].try_into().unwrap());
        if hv != h {
            continue;
        }
        let base = 16 + tok * 60;
        let len = map[base] as usize;
        if len > 0 && &map[base + 1..base + 1 + len] == cid.as_bytes() {
            src = tok;
            break;
        }
    }
    if src == usize::MAX {
        die("particle not in index");
    }
    let e_base = 16 + n * 60;
    let e = &map[e_base..];
    let row = |i: usize| -> &[u8] { &e[i * d * 4..(i + 1) * d * 4] };
    let f32s = |b: &[u8]| -> f32 { f32::from_le_bytes(b.try_into().unwrap()) };
    let qs: Vec<f32> = (0..d).map(|c| f32s(&row(src)[c * 4..c * 4 + 4])).collect();

    // cosine scores (rows are unit-norm at compile). on Apple Silicon
    // the gemv goes through honeycrisp's acpu (NEON/AMX SGEMM,
    // memory-bound); elsewhere a scalar pass.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let scores: Vec<f32> = {
        // zero-copy view of the mmap'd f32 embedding; the index layout
        // (16 + 60n bytes of header) keeps 4-byte alignment. hand NEON
        // gemv: 4 rows interleaved so the reduction latency hides
        // behind the next row's loads (memory-bound target ~60+ GB/s).
        let w: &[f32] =
            unsafe { std::slice::from_raw_parts(e.as_ptr() as *const f32, n * d) };
        neon_gemv(w, &qs, n, d)
    };
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    let scores: Vec<f32> = (0..n)
        .map(|v| {
            let rb = row(v);
            let mut acc = 0.0f32;
            for c in 0..d {
                acc += qs[c] * f32s(&rb[c * 4..c * 4 + 4]);
            }
            acc
        })
        .collect();
    let mut top: Vec<(f32, usize)> = Vec::with_capacity(k + 1);
    let mut worst = f32::NEG_INFINITY;
    for v in 0..n {
        if v == src {
            continue;
        }
        let acc = scores[v];
        if acc <= worst {
            continue;
        }
        top.push((acc, v));
        if top.len() > k {
            let wi = top
                .iter()
                .enumerate()
                .min_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap())
                .map(|(i, _)| i)
                .unwrap();
            top.swap_remove(wi);
            worst = top.iter().map(|t| t.0).fold(f32::NEG_INFINITY, f32::max);
        }
    }
    top.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("suggestions for {cid} (top {k}, {n} tokens, {:?}):", t0.elapsed());
    for (score, v) in &top {
        if *score <= 0.0 {
            continue; // zero rows carry no signal
        }
        let base = 16 + v * 60;
        let len = map[base] as usize;
        if len == 0 {
            continue; // axon placeholders have no cid to suggest
        }
        let s = std::str::from_utf8(&map[base + 1..base + 1 + len]).unwrap_or("");
        println!("  {score:.4}  {s}");
    }
}

/// scores = W q over n rows of width d — hand NEON, 4-row interleave.
/// d is a compile-time small multiple of 8 in practice (64); the tail
/// loop handles the remainder.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn neon_gemv(w: &[f32], q: &[f32], n: usize, d: usize) -> Vec<f32> {
    use std::arch::aarch64::*;
    let mut out = vec![0.0f32; n];
    unsafe {
        let chunks = d / 8;
        let tail = d % 8;
        let mut r = 0usize;
        while r + 4 <= n {
            let mut acc = [vld1q_dup_f32(&0.0f32); 4];
            for c in 0..chunks {
                let qv = vld1q_f32(q.as_ptr().add(c * 8));
                for j in 0..4 {
                    let wv = vld1q_f32(w.as_ptr().add((r + j) * d + c * 8));
                    acc[j] = vfmaq_f32(acc[j], wv, qv);
                }
            }
            for j in 0..4 {
                let s = vaddvq_f32(acc[j]);
                // scalar tail (d % 8)
                let mut acc2 = 0.0f32;
                for c in chunks * 8..chunks * 8 + tail {
                    acc2 += w[(r + j) * d + c] * q[c];
                }
                out[r + j] = s + acc2;
            }
            r += 4;
        }
        while r < n {
            let z = 0.0f32;
            let mut acc = vld1q_dup_f32(&z);
            for c in 0..chunks {
                let qv = vld1q_f32(q.as_ptr().add(c * 8));
                let wv = vld1q_f32(w.as_ptr().add(r * d + c * 8));
                acc = vfmaq_f32(acc, wv, qv);
            }
            let mut acc2 = 0.0f32;
            for c in chunks * 8..chunks * 8 + tail {
                acc2 += w[r * d + c] * q[c];
            }
            out[r] = vaddvq_f32(acc) + acc2;
            r += 1;
        }
    }
    out
}
