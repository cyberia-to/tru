//! rewrite a compiled .model's vocab so particle token strings are the
//! CIDs (whole-word tokenization needs the prompt words in the vocab).
//! one-off for artifacts compiled before compile_graph learned to emit
//! CID vocab.
//!
//! usage: rewrite_vocab <model> <links.jsonl> <out.model>

use tru::Model;

fn cid_id(c: &str) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(cyber_hemera::hash(c.as_bytes()).as_bytes());
    h
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.len() != 3 {
        eprintln!("usage: rewrite_vocab <model> <links.jsonl> <out.model>");
        std::process::exit(2);
    }
    let t0 = std::time::Instant::now();
    let mut model = Model::read(&argv[0]).expect("model");

    // replay pass-1 interning (from, to, axon per link)
    let mut order: Vec<Option<String>> = Vec::new();
    let mut seen: std::collections::HashMap<[u8; 32], usize> = std::collections::HashMap::new();
    let text = std::fs::read_to_string(&argv[1]).expect("links");
    for line in text.lines() {
        if !line.contains("\"f\":") {
            continue;
        }
        let cid = |key: &str| -> String {
            line.split_once(&format!("\"{key}\": \""))
                .and_then(|(_, r)| r.split_once('"'))
                .map(|(s, _)| s.to_string())
                .unwrap_or_default()
        };
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
    let n = model
        .tensors
        .iter()
        .find(|t| t.name == "model.embed_tokens.weight")
        .map(|t| t.shape[0] as usize)
        .expect("embed tensor");
    assert_eq!(order.len(), n, "interning replay diverged");

    let mut v = String::from("[tokens]\n");
    for (id, cid) in order.iter().enumerate() {
        match cid {
            Some(c) => v.push_str(&format!("{id} = \"{c}\"\n")),
            None => v.push_str(&format!("{id} = \"0x{id:064x}\"\n")),
        }
    }
    model.vocab = v;
    model.write(&argv[2]).expect("write");
    println!(
        "rewrote vocab ({n} tokens) -> {} ({:?})",
        argv[2],
        t0.elapsed()
    );
}
