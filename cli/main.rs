use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tru::Graph;

#[derive(Parser)]
#[command(name = "tru", version, about = "model compilation: .graph → .model")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print a summary of a `.graph` file.
    Inspect { path: PathBuf },

    /// Compile a `.graph` into a `.model` (CT-0 pipeline).
    Compile {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().cmd {
        Cmd::Inspect { path } => inspect(&path)?,
        Cmd::Compile { input, output } => compile(&input, &output)?,
    }
    Ok(())
}

fn inspect(path: &std::path::Path) -> anyhow::Result<()> {
    let g = Graph::open(path)?;
    println!("name:       {}", g.name());
    println!("sections:   {}", g.frontmatter().files.len());
    for f in &g.frontmatter().files {
        match f.size {
            Some(sz) => println!("  - {} ({}, {} bytes)", f.name, f.format, sz),
            None => println!("  - {} ({}, text)", f.name, f.format),
        }
    }
    let count = g.cyberlinks()?.count();
    println!("cyberlinks: {count}");
    Ok(())
}

fn compile(input: &std::path::Path, _output: &std::path::Path) -> anyhow::Result<()> {
    use tru::focusing::{compute_focusing, FocusingGraph, FocusingParams, Link};

    let g = Graph::open(input)?;
    let n_links = g.cyberlinks()?.len();
    eprintln!("pass 0: building focusing graph from {n_links} cyberlinks…");

    let links = g.cyberlinks()?.map(|cl| Link {
        from: cl.from,
        to: cl.to,
        amount: cl.amount,
        valence: cl.valence,
    });

    let fg = FocusingGraph::build(links);
    eprintln!("        {} particles, {} edges", fg.n(), n_links);

    let result = compute_focusing(&fg, &FocusingParams::default());

    // Print top-10 by focus
    let mut ranked: Vec<(usize, f64)> = result.focus.iter().copied().enumerate().collect();
    ranked.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    eprintln!("top particles by φ*:");
    for (idx, phi) in ranked.iter().take(10) {
        let hash = fg.node_id(*idx);
        eprintln!("  {:016x}…  φ*={:.6}", u64::from_le_bytes(hash[..8].try_into().unwrap()), phi);
    }

    anyhow::bail!("CT-0 passes 1–8 not yet implemented; focus computed above")
}
