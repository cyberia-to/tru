//! compile a cybergraph snapshot (tru eval jsonl: {"h","f","t"} per line)
//! into a CT-0 .model — the product artifact path.
//!
//! usage: compile_graph <links.jsonl> <out.model> [--name NAME]
//!
//! particle ids are hemera hashes of the CID text (same convention as
//! the eval harnesses); every link is valence +1, amount 1, token 1.
//! prints the certificate numbers (d*, h*, L*, gain, wall time).

use std::io::Write;

use tru::graph::Cyberlink;
use tru::graph::record::RECORD_SIZE;
use tru::{Fx, Model};

fn cid_id(c: &str) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(cyber_hemera::hash(c.as_bytes()).as_bytes());
    h
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.len() < 2 {
        die("usage: compile_graph <links.jsonl> <out.model> [--name NAME] [--index PATH]");
    }
    let input = &argv[0];
    let output = &argv[1];
    let mut name = "compiled-graph".to_string();
    let mut index_path: Option<String> = None;
    let mut i = 2;
    while i < argv.len() {
        match argv[i].as_str() {
            "--name" => {
                i += 1;
                name = argv.get(i).cloned().unwrap_or(name);
            }
            "--index" => {
                i += 1;
                index_path = argv.get(i).cloned();
            }
            other => die(&format!("unknown flag {other}")),
        }
        i += 1;
    }

    let t0 = std::time::Instant::now();
    let mut links: Vec<Cyberlink> = Vec::new();
    // interning order replayed exactly as pass 1 (from, to, axon per
    // link) so the suggestion index maps token -> cid without guessing.
    let mut order: Vec<Option<String>> = Vec::new();
    let mut seen: std::collections::HashMap<[u8; 32], usize> =
        std::collections::HashMap::new();
    let text =
        std::fs::read_to_string(&input).unwrap_or_else(|e| die(&format!("read {input}: {e}")));
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let h: u64 = line
            .split_once("\"h\": ")
            .and_then(|(_, r)| r.split(',').next())
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or_else(|| die("bad height field"));
        let cid = |key: &str| -> String {
            line.split_once(&format!("\"{key}\": \""))
                .and_then(|(_, r)| r.split_once('"'))
                .unwrap_or_else(|| die("bad cid field"))
                .0
                .to_string()
        };
        let fc = cid("f");
        let tc = cid("t");
        let fh = cid_id(&fc);
        let th = cid_id(&tc);
        links.push(Cyberlink {
            neuron: cid_id("neuron-compile"),
            from: fh,
            to: th,
            token: 1,
            amount: 1,
            valence: 1,
            block: h,
        });
        // replay pass-1 interning: from, to, then axon(p,q) with the
        // particle ids — matches index::build byte for byte.
        for (h, cid) in [(fh, Some(fc.clone())), (th, Some(tc)), (tru::pass::index::axon(&fh, &th), None)] {
            if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(h) {
                e.insert(order.len());
                order.push(cid);
            }
        }
    }
    println!("loaded {} links ({:?})", links.len(), t0.elapsed());

    // materialize a .graph container so tru::compile reads exactly the
    // production path (mmap, sections, records).
    let mut records = Vec::with_capacity(links.len() * RECORD_SIZE);
    for l in &links {
        let mut r = [0u8; RECORD_SIZE];
        r[0..32].copy_from_slice(&l.neuron);
        r[32..64].copy_from_slice(&l.from);
        r[64..96].copy_from_slice(&l.to);
        r[96..100].copy_from_slice(&l.token.to_le_bytes());
        r[100..116].copy_from_slice(&l.amount.to_le_bytes());
        r[116] = l.valence as u8;
        r[117..125].copy_from_slice(&l.block.to_le_bytes());
        records.extend_from_slice(&r);
    }
    let graph_path = std::env::temp_dir().join(format!("tru_compile_{}.graph", std::process::id()));
    let frontmatter = format!(
        "[cyb]\ntypes = [\"graph\"]\nname = \"{name}\"\n\n[[files]]\nname = \"config\"\nformat = \"toml\"\n\n[[files]]\nname = \"cyberlinks\"\nformat = \"records\"\nsize = {}\n",
        records.len()
    );
    {
        let mut f = std::fs::File::create(&graph_path).unwrap();
        f.write_all(frontmatter.as_bytes()).unwrap();
        f.write_all(b"~~~config\nchain_id = \"compile\"\n").unwrap();
        f.write_all(b"~~~cyberlinks\n").unwrap();
        f.write_all(&records).unwrap();
    }
    println!("graph file {} ({:?})", graph_path.display(), t0.elapsed());

    let g =
        tru::graph::Graph::open(&graph_path).unwrap_or_else(|e| die(&format!("open graph: {e}")));
    println!("compiling {} ...", g.name());
    let tc = std::time::Instant::now();
    let mut model = tru::pass::compile::compile(&g).unwrap_or_else(|e| die(&format!("compile: {e}")));
    // particle token strings are the CIDs (tokenization is whole-word
    // lookup in the runtime). axons keep their hex form — they are
    // never prompt words.
    {
        let mut v = String::from("[tokens]\n");
        for (id, cid) in order.iter().enumerate() {
            match cid {
                Some(c) => v.push_str(&format!("{id} = \"{c}\"\n")),
                None => {
                    let hex = &model.vocab;
                    let _ = hex;
                    v.push_str(&format!("{id} = \"0x{id:064x}\"\n"));
                }
            }
        }
        model.vocab = v;
    }
    println!("compiled in {:?}", tc.elapsed());
    {
        let t = &model.tensors[0];
        let d = t.shape[1] as usize;
        for i in [0usize, 1, 100] {
            let mut n2 = 0.0f64;
            for c in 0..d {
                let v = t.data[i * d + c].to_f64();
                n2 += v * v;
            }
            println!("row-norm check [{i}]: ||E||^2 = {n2:.4}");
        }
    }

    model
        .write(&output)
        .unwrap_or_else(|e| die(&format!("write model: {e}")));
    let bytes = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
    let cfg = &model.config;
    let grab = |key: &str| -> String {
        cfg.lines()
            .find_map(|l| {
                l.split_once('=')
                    .filter(|(k, _)| k.trim() == key)
                    .map(|(_, v)| v.trim().to_string())
            })
            .unwrap_or_default()
    };
    println!(
        "model {output}: {bytes} bytes · d*={} h*={} L*={} · wall {:?}",
        grab("hidden_size"),
        grab("num_attention_heads"),
        grab("num_hidden_layers"),
        t0.elapsed()
    );
    if let Some(ip) = &index_path {
        let t = &model.tensors[0];
        let d = t.shape[1] as usize;
        let n = t.shape[0] as usize;
        let mut out = Vec::with_capacity(16 + n * 64 + n * d * 4);
        out.extend_from_slice(&(n as u64).to_le_bytes());
        out.extend_from_slice(&(d as u64).to_le_bytes());
        for cid in &order {
            let s = cid.as_deref().unwrap_or("");
            let b = s.as_bytes();
            let len = b.len().min(59) as u8;
            out.push(len);
            let mut slot = [0u8; 59];
            slot[..len as usize].copy_from_slice(&b[..len as usize]);
            out.extend_from_slice(&slot);
        }
        // f32 dequantized embedding (glia semantics: f = i16 / 256) so
        // the scorer streams at memory bandwidth instead of converting
        // per query.
        for v in &t.data {
            out.extend_from_slice(&(v.to_f64() as f32).to_le_bytes());
        }
        std::fs::write(ip, &out).unwrap_or_else(|e| die(&format!("index: {e}")));
        println!("index {ip}: {} tokens ({} bytes)", n, out.len());
    }
    let _ = std::fs::remove_file(&graph_path);
}

fn die(msg: &str) -> ! {
    eprintln!("compile_graph: {msg}");
    std::process::exit(2)
}
