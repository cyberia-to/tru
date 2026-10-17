//! build the suggestion sidecar for a compiled .model — one offline
//! pass so queries run at memory bandwidth.
//!
//! usage: build_index <model> <links.jsonl> <out.bin>
//!
//! format: u64 n, u64 d, then per token [u8 len, 59B cid ascii], then
//! the f32 dequantized embedding (n*d, glia semantics f = i16/256).
//! the token order replays pass-1 interning (from, to, axon per link)
//! exactly — the same replay compile_graph uses.

use tru::Model;

fn cid_id(c: &str) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(cyber_hemera::hash(c.as_bytes()).as_bytes());
    h
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.len() != 3 {
        eprintln!("usage: build_index <model> <links.jsonl> <out.bin>");
        std::process::exit(2);
    }
    let t0 = std::time::Instant::now();
    let bytes = std::fs::read(&argv[0]).expect("model");
    let model = Model::from_bytes(&bytes).expect("parse model");
    let embed = model
        .tensors
        .iter()
        .find(|t| t.name == "model.embed_tokens.weight")
        .expect("embed tensor");
    let n = embed.shape[0] as usize;
    let d = embed.shape[1] as usize;

    // replay interning
    let mut order: Vec<Option<String>> = Vec::new();
    let mut seen: std::collections::HashMap<[u8; 32], usize> = std::collections::HashMap::new();
    let text = std::fs::read_to_string(&argv[1]).expect("links");
    for line in text.lines() {
        let cid = |key: &str| -> String {
            line.split_once(&format!("\"{key}\": \""))
                .and_then(|(_, r)| r.split_once('"'))
                .map(|(s, _)| s.to_string())
                .unwrap_or_default()
        };
        if !line.contains("\"f\":") {
            continue;
        }
        let fc = cid("f");
        let tc = cid("t");
        let fh = cid_id(&fc);
        let th = cid_id(&tc);
        for (h, c) in [
            (fh, Some(fc.clone())),
            (th, Some(tc)),
            (tru::pass::index::axon(&fh, &th), None),
        ] {
            if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(h) {
                e.insert(order.len());
                order.push(c);
            }
        }
    }
    assert_eq!(order.len(), n, "interning replay diverged from the model");

    let mut out = Vec::with_capacity(16 + n * 64 + n * d * 4);
    out.extend_from_slice(&(n as u64).to_le_bytes());
    out.extend_from_slice(&(d as u64).to_le_bytes());
    for cid in &order {
        let b = cid.as_deref().unwrap_or("").as_bytes();
        let len = b.len().min(59) as u8;
        out.push(len);
        let mut slot = [0u8; 59];
        slot[..len as usize].copy_from_slice(&b[..len as usize]);
        out.extend_from_slice(&slot);
    }
    for v in &embed.data {
        out.extend_from_slice(&(v.to_f64() as f32).to_le_bytes());
    }
    // FNV-1a 64 of each token's cid — a 48 MB streaming pre-filter so
    // the query lookup is a vectorizable u64 scan, not string compares
    let hbase = 16 + n * 60 + n * d * 4;
    let _ = hbase;
    for cid in &order {
        let s = cid.as_deref().unwrap_or("");
        let mut h = 0xcbf29ce484222325u64;
        for b in s.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        out.extend_from_slice(&h.to_le_bytes());
    }
    std::fs::write(&argv[2], &out).expect("write index");
    println!(
        "index {}: {n} tokens, {} bytes ({:?})",
        argv[2],
        out.len(),
        t0.elapsed()
    );
}
