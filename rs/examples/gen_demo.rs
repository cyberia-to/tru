//! Generate demo `.cyb` files for exercising the `tru` CLI.
//! Run: `cargo run -p tru --example gen_demo -- /tmp`

use std::path::PathBuf;

use tru::model::{Encoding, Tensor};
use tru::vocab::Vocab;
use tru::{Fx, Model};

fn hash(b: u8) -> [u8; 32] {
    let mut h = [0u8; 32];
    h[0] = b;
    h
}

fn record(from: u8, to: u8, amount: u128) -> [u8; 128] {
    let mut r = [0u8; 128];
    r[0] = 200; // neuron (unused by focusing)
    r[32..64].copy_from_slice(&hash(from));
    r[64..96].copy_from_slice(&hash(to));
    r[96..100].copy_from_slice(&1u32.to_le_bytes()); // token
    r[100..116].copy_from_slice(&amount.to_le_bytes());
    r[116] = 1; // valence +1
    r[117..125].copy_from_slice(&1000u64.to_le_bytes()); // block
    r
}

fn main() {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "/tmp".into()));

    // .graph
    let edges = [(1u8, 2u8, 100u128), (2, 3, 50), (3, 1, 200), (4, 1, 300), (1, 3, 80)];
    let mut records = Vec::new();
    for &(f, t, a) in &edges {
        records.extend_from_slice(&record(f, t, a));
    }
    let fm = format!(
        "[cyb]\ntypes = [\"graph\"]\nname = \"demo-graph\"\n\n[[files]]\nname = \"config\"\nformat = \"toml\"\n\n[[files]]\nname = \"cyberlinks\"\nformat = \"records\"\nsize = {}\n",
        records.len()
    );
    let mut g = Vec::new();
    g.extend_from_slice(fm.as_bytes());
    g.extend_from_slice(b"~~~config\nchain_id = \"demo\"\n");
    g.extend_from_slice(b"~~~cyberlinks\n");
    g.extend_from_slice(&records);
    std::fs::write(dir.join("demo.graph"), &g).unwrap();

    // .vocab
    let mut v = Vocab::new("demo-vocab", "# demo\nordering: first-appearance.");
    v.push(b"particle one: a concept".to_vec());
    v.push(b"BOOT".to_vec());
    v.register([9u8; 32]);
    v.write(dir.join("demo.vocab")).unwrap();

    // .model
    let mut m = Model::new("demo-model");
    m.config = "hidden_size = 8\nnum_hidden_layers = 2\nrms_norm_eps = 1000000\n".into();
    m.program = "transformer_llama".into();
    m.tensors = vec![
        Tensor { name: "model.embed_tokens.weight".into(), shape: vec![4, 8], encoding: Encoding::U16, data: (0..32i64).map(|i| Fx::from_ratio(i - 16, 25)).collect() },
        Tensor { name: "model.norm.weight".into(), shape: vec![8], encoding: Encoding::U32, data: vec![Fx::ONE; 8] },
    ];
    m.write(dir.join("demo.model")).unwrap();

    println!("wrote demo.graph, demo.vocab, demo.model to {}", dir.display());
}
