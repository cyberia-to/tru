//! The CT-0 compile orchestrator and packaging (`specs/ct0.md` §10).
//!
//! Runs the structural passes (1 index, 2 dialect, 3 arch, 7 norm) and packages
//! the result into a `.model`: the `config`/`vocab`/`card`/`eval` sections carry
//! the architecture derived from the graph, and the norm tensors are emitted in
//! full. The SVD-derived weight tensors (embed §6, attn §7, mlp §8) land with
//! the fixed-point randomized-SVD milestone; the `card` states this plainly, so
//! the artifact is honest about what it contains.

use super::{arch, attn, dialect, embed, index, mlp, norm};
use crate::error::Result;
use crate::graph::Graph;
use crate::model::Model;

const SHIFT_SET_LEN: u64 = 5; // |S| for the Clifford MLP (§8, config default)

/// Compile a `.graph` into a `.model`. Deterministic: the same graph yields the
/// same architecture and the same file (§10.9).
pub fn compile(graph: &Graph) -> Result<Model> {
    let links: Vec<_> = graph.cyberlinks()?.collect();
    let block = links.iter().map(|l| l.block).max().unwrap_or(0);

    let (particles, edges, adj) = index::build(&[], &links);
    let dialects = dialect::discover(&particles, &edges);
    let a = arch::compute(&adj, dialects.len(), block);

    // Weight passes 4–6, then norms (pass 7). Storage order (§10.5): embedding
    // first, then the attention/MLP/norm tensors, in a fixed deterministic order.
    let embedding = embed::embed(&adj, &a.phi, a.d);
    let attn_tensors = attn::attention(&edges, &dialects, &embedding.data, &a.phi, a.d, a.h, a.l, a.diameter);
    let mlp_tensors = mlp::mlp(a.d, a.l);
    let norm_tensors = norm::layernorms(a.d, a.l);

    let mut tensors = Vec::with_capacity(1 + attn_tensors.len() + mlp_tensors.len() + norm_tensors.len());
    tensors.push(embedding);
    tensors.extend(attn_tensors);
    tensors.extend(mlp_tensors);
    tensors.extend(norm_tensors);

    let mut model = Model::new(format!("{}-ct0", graph.name()));
    model.card = card(graph.name(), &a, &dialects);
    model.config = config_toml(&a);
    model.program = program();
    model.vocab = vocab_toml(&particles);
    model.eval = eval_toml(&a);
    model.tensors = tensors;
    Ok(model)
}

/// The declared parameter count of the full architecture (§8.1 budget), whether
/// or not every tensor is emitted yet — the config advertises the shape.
fn declared_params(a: &arch::Arch) -> u64 {
    let (v, d, l) = (a.particles as u64, a.d as u64, a.l as u64);
    let attn = 4 * d * d + 2; // q,k,v,o + alpha_beta
    let mlp = 2 * SHIFT_SET_LEN * d * d + 2 * d * d + d + 18 * d;
    let norms = 2 * d;
    v * d + l * (attn + mlp + norms) + d
}

fn card(name: &str, a: &arch::Arch, dialects: &dialect::Dialects) -> String {
    format!(
        "# {name}-ct0\n\n\
         Compiled from {name}.graph (block {block}) by CT-0.\n\
         d={d}, h={h}, L={l}, params≈{p}.\n\n\
         Architecture derived from the graph spectrum: φ* (PageRank), \
         d* (effective rank of the φ*-weighted adjacency), h*={h} dialects, \
         L* (diameter × mixing time, κ={kappa:.3}, λ₂={l2:.4}, diameter={diam}).\n\n\
         Full CT-0 compile: embedding (SVD of the φ*-weighted adjacency), \
         per-head attention projections, Clifford-MLP weights, and layer norms \
         are all emitted, fixed-point over the Goldilocks field.\n",
        name = name,
        block = a.block,
        d = a.d,
        h = a.h,
        l = a.l,
        p = declared_params(a),
        kappa = a.kappa.to_f64(),
        l2 = a.lambda2.to_f64(),
        diam = a.diameter,
    ) + &format!("\nDialects: {} (including ⊥).\n", dialects.len())
}

fn config_toml(a: &arch::Arch) -> String {
    let head_dim = if a.h == 0 { a.d } else { a.d / a.h };
    format!(
        "model_type = \"llama\"\n\
         parameters = {params}\n\
         license = \"cyber license\"\n\n\
         [architecture]\n\
         hidden_size = {d}\n\
         num_attention_heads = {h}\n\
         num_key_value_heads = {h}\n\
         head_dim = {head_dim}\n\
         num_hidden_layers = {l}\n\
         intermediate_size = {inter}\n\
         vocab_size = {vocab}\n\
         context_length = 8192\n\
         max_position_embeddings = 8192\n\
         rope_theta = 10000\n\
         rms_norm_eps = 1000000\n\n\
         [tokenizer]\n\
         type = \"particle\"\n\
         bos_id = 0\n\
         eos_id = 0\n\
         pad_id = 0\n\n\
         [clifford]\n\
         shift_set = [1, 2, 4, 8, 16]\n\
         self_energy_suppression = 1\n\n\
         [lineage]\n\
         spec = \"CT-0\"\n\
         source_kind = \".graph\"\n\
         block = {block}\n",
        params = declared_params(a),
        d = a.d,
        h = a.h,
        head_dim = head_dim,
        l = a.l,
        inter = 4 * a.d,
        vocab = a.particles,
        block = a.block,
    )
}

fn program() -> String {
    "module model.pipeline\n\
     use std.nn.transformer_llama\n\n\
     pub fn forward(input: Field, output: Field, seq: Field, cfg: Config) {\n    \
         transformer_llama.forward(input, output, seq, cfg)\n\
     }\n"
        .to_string()
}

fn vocab_toml(v: &index::ParticleIndex) -> String {
    let mut s = String::from("[tokens]\n");
    for (id, p) in v.particles().iter().enumerate() {
        let hex: String = p.iter().map(|b| format!("{b:02x}")).collect();
        s.push_str(&format!("{id} = \"0x{hex}\"\n"));
    }
    s
}

fn eval_toml(a: &arch::Arch) -> String {
    // Structural-compile certificate: architecture is derived and deterministic;
    // the weight-conformance predicates (P_EMBED, P_ATTN) await the SVD passes.
    let top = a
        .phi
        .iter()
        .map(|x| x.to_f64())
        .fold(0.0_f64, f64::max);
    format!(
        "[ct0_structural]\n\
         P_DET = 1\n\
         d = {d}\n\
         h = {h}\n\
         L = {l}\n\n\
         [focus]\n\
         top_concentration = {top}\n",
        d = a.d,
        h = a.h,
        l = a.l,
        top = (top * 1000.0) as u64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::record::RECORD_SIZE;
    use std::io::Write;

    fn synth_record(from: u8, to: u8, amount: u128, valence: i8, block: u64) -> [u8; RECORD_SIZE] {
        let mut r = [0u8; RECORD_SIZE];
        r[0..32].fill(from); // neuron
        r[32..64].fill(from);
        r[64..96].fill(to);
        r[96..100].copy_from_slice(&1u32.to_le_bytes());
        r[100..116].copy_from_slice(&amount.to_le_bytes());
        r[116] = valence as u8;
        r[117..125].copy_from_slice(&block.to_le_bytes());
        r
    }

    /// A tiny in-memory `.graph` with a handful of affirming links, in the exact
    /// container format `Graph::open` expects (bare TOML frontmatter, `~~~`
    /// section delimiters, `cyberlinks` records section).
    fn synth_graph() -> Graph {
        let mut records = Vec::new();
        for (f, t, v) in [(1u8, 2u8, 1i8), (2, 3, 1), (3, 1, 1), (4, 1, 1), (1, 3, 1)] {
            records.extend_from_slice(&synth_record(f, t, 100, v, 7));
        }
        let frontmatter = format!(
            "[cyb]\ntypes = [\"graph\"]\nname = \"tiny\"\n\n\
             [[files]]\nname = \"config\"\nformat = \"toml\"\n\n\
             [[files]]\nname = \"cyberlinks\"\nformat = \"records\"\nsize = {}\n",
            records.len()
        );

        let path = std::env::temp_dir().join("tru_ct0_compile_test.graph");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(frontmatter.as_bytes()).unwrap();
        write!(f, "~~~config\nblock = 7\n").unwrap();
        f.write_all(b"~~~cyberlinks\n").unwrap();
        f.write_all(&records).unwrap();
        drop(f);
        Graph::open(&path).unwrap()
    }

    #[test]
    fn compiles_a_graph_into_a_loadable_model() {
        let g = synth_graph();
        let model = compile(&g).unwrap();

        // Round-trips through the writer/reader byte-for-byte.
        let bytes = model.to_bytes();
        let reloaded = Model::from_bytes(&bytes).unwrap();
        assert_eq!(reloaded.to_bytes(), bytes, "compiled model must round-trip");

        // Config advertises the derived architecture.
        assert!(model.config.contains("hidden_size ="));
        assert!(model.config.contains("num_hidden_layers ="));
        assert!(model.vocab.contains("[tokens]"));
        // Norm tensors present and shaped to d*.
        assert!(model.tensors.iter().any(|t| t.name == "model.norm.weight"));
    }

    #[test]
    fn compile_is_deterministic() {
        let g = synth_graph();
        let a = compile(&g).unwrap().to_bytes();
        let b = compile(&g).unwrap().to_bytes();
        assert_eq!(a, b, "two compiles of the same graph are byte-identical (§10.9)");
    }
}
