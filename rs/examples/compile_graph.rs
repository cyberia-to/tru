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
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .unwrap_or_else(|| die("usage: compile_graph <links.jsonl> <out.model> [--name NAME]"));
    let output = args
        .next()
        .unwrap_or_else(|| die("usage: compile_graph <links.jsonl> <out.model> [--name NAME]"));
    let name = args.nth(1).unwrap_or_else(|| "compiled-graph".to_string());

    let t0 = std::time::Instant::now();
    let mut links: Vec<Cyberlink> = Vec::new();
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
        links.push(Cyberlink {
            neuron: cid_id("neuron-compile"),
            from: cid_id(&cid("f")),
            to: cid_id(&cid("t")),
            token: 1,
            amount: 1,
            valence: 1,
            block: h,
        });
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
    let model = tru::pass::compile::compile(&g).unwrap_or_else(|e| die(&format!("compile: {e}")));
    println!("compiled in {:?}", tc.elapsed());

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
    let _ = std::fs::remove_file(&graph_path);
}

fn die(msg: &str) -> ! {
    eprintln!("compile_graph: {msg}");
    std::process::exit(2)
}
