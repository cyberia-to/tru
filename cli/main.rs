//! `tru` — the convergence VM command line.
//!
//! Reads `.cyb` containers (`.graph` / `.vocab` / `.model`) and runs the
//! focusing engine over a graph: φ*, cyberank, syntropy, and telemetry.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use tru::focusing::{self, FocusingGraph, FocusingParams, Link};
use tru::graph::frontmatter;
use tru::vocab::Vocab;
use tru::{Graph, Model};

#[derive(Parser)]
#[command(name = "tru", version, about = "convergence VM: .graph → φ* → .model")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Summarize any `.cyb` container: type, name, sections.
    Inspect { path: PathBuf },

    /// Run the tri-kernel over a `.graph`: cyberank, syntropy, telemetry.
    Focus {
        path: PathBuf,
        /// How many top-ranked particles to print.
        #[arg(short, long, default_value_t = 20)]
        top: usize,
    },

    /// Summarize a `.vocab` dictionary and check its self-consistency.
    Vocab { path: PathBuf },

    /// Summarize a `.model` checkpoint: tensors, config, particle.
    Model { path: PathBuf },
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Inspect { path } => inspect(&path),
        Cmd::Focus { path, top } => focus(&path, top),
        Cmd::Vocab { path } => vocab(&path),
        Cmd::Model { path } => model(&path),
    }
}

fn inspect(path: &std::path::Path) -> Result<()> {
    let bytes = std::fs::read(path)?;
    let (fm_str, body) = frontmatter::split(&bytes)?;
    let fm = frontmatter::parse(fm_str)?;
    let sections = frontmatter::index_sections(&bytes, body, &fm.files)?;

    println!("{}  [types: {}]", fm.cyb.name, fm.cyb.types.join(", "));
    println!("{} bytes, {} sections:", bytes.len(), fm.files.len());
    for f in &fm.files {
        let sz = sections.get(&f.name).map(|&(s, e)| e - s).unwrap_or(0);
        println!("  {:<12} {:<9} {:>12} bytes", f.name, f.format, sz);
    }
    Ok(())
}

fn focus(path: &std::path::Path, top: usize) -> Result<()> {
    let g = Graph::open(path)?;
    let n_links = g.cyberlinks()?.count();
    let links = g.cyberlinks()?.map(|cl| Link { from: cl.from, to: cl.to, amount: cl.amount, valence: cl.valence });
    let fg = FocusingGraph::build(links);
    let params = FocusingParams::default();
    let result = tru::compute_focusing(&fg, &params);
    let tel = focusing::telemetry(&fg, &result, &params);

    println!("focus: {}", g.name());
    println!("  particles {}   cyberlinks {}", fg.n(), n_links);
    println!("  syntropy J {:.4}   entropy H {:.4}", tel.syntropy.to_f64(), tel.entropy.to_f64());
    println!("  contraction κ {:.3}   λ₂ {:.3}   steps T(ε) {}", tel.kappa.to_f64(), tel.lambda_2.to_f64(), tel.steps);

    let mut ranked: Vec<(usize, f64)> = result.focus.iter().map(|x| x.to_f64()).enumerate().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("\ntop {} particles by cyberank φ*(p):", top.min(ranked.len()));
    for (idx, phi) in ranked.iter().take(top) {
        println!("  {}  {:.6}", hex8(fg.node_id(*idx)), phi);
    }
    Ok(())
}

fn vocab(path: &std::path::Path) -> Result<()> {
    let v = Vocab::read(path)?;
    let inline = v.entries.iter().filter(|e| !e.data.is_empty()).count();
    println!("vocab: {}", v.name);
    println!("  entries {} ({} with inline data)", v.entries.len(), inline);
    println!("  file particle {}", hex(&v.particle()));
    match v.verify() {
        Ok(()) => println!("  self-consistency OK"),
        Err(e) => println!("  self-consistency FAIL: {e}"),
    }
    Ok(())
}

fn model(path: &std::path::Path) -> Result<()> {
    let m = Model::read(path)?;
    println!("model: {}", m.name);
    println!("  particle {}", hex(&m.particle()));
    println!("  tensors {}", m.tensors.len());
    for t in &m.tensors {
        println!("    {:<34} {:?}  {:?}  {} values", t.name, t.shape, t.encoding, t.data.len());
    }
    if !m.config.is_empty() {
        println!("  config:");
        for line in m.config.lines().take(8) {
            println!("    {line}");
        }
    }
    Ok(())
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn hex8(b: &[u8; 32]) -> String {
    format!("{}…", b[..8].iter().map(|x| format!("{x:02x}")).collect::<String>())
}
