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
        die("usage: suggest <model> <CID> [--k N] [--links <links.jsonl>]");
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
