//! `tru` — the convergence VM command line.
//!
//! Reads `.cyb` containers (`.graph` / `.vocab` / `.model`) and runs the
//! focusing engine over a graph: φ*, cyberank, syntropy, telemetry.

use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use tru::focusing::{self, FocusingGraph, FocusingParams, Link};
use tru::graph::frontmatter;
use tru::vocab::Vocab;
use tru::{Graph, Model};

// ── color ─────────────────────────────────────────────────────────────────
// ANSI only when stdout is a terminal — piped output stays clean.

fn tty() -> bool {
    io::stdout().is_terminal()
}

fn paint(code: &str, s: &str) -> String {
    if tty() {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

fn dim(s: &str) -> String {
    paint("90", s)
}
fn cyan(s: &str) -> String {
    paint("36", s)
}
fn green(s: &str) -> String {
    paint("32", s)
}
fn yellow(s: &str) -> String {
    paint("33", s)
}
fn bold(s: &str) -> String {
    paint("1", s)
}

/// The rainbow ANSI-Shadow wordmark — printed only on a terminal.
const LOGO: &str = "\
\x1b[31m████████╗██████╗ ██╗   ██╗\x1b[0m
\x1b[33m╚══██╔══╝██╔══██╗██║   ██║\x1b[0m
\x1b[32m   ██║   ██████╔╝██║   ██║\x1b[0m
\x1b[36m   ██║   ██╔══██╗██║   ██║\x1b[0m
\x1b[34m   ██║   ██║  ██║╚██████╔╝\x1b[0m
\x1b[35m   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ \x1b[0m";

/// Banner: wordmark + tagline + the field/kernel parameters, hemera-style.
fn banner() -> String {
    if !tty() {
        return String::new();
    }
    format!(
        "{LOGO}\n{tag}\n{params}\n",
        tag = paint("37", "    the convergence engine"),
        params = dim(
            "\n    Goldilocks field · p = 2^64 - 2^32 + 1\n    \
             tri-kernel · diffusion + springs + heat\n    \
             coupled iteration · fixed-point · converges to φ*\n"
        ),
    )
}

fn help() {
    println!(
        "{}\n  {}   {}\n  {}   {}\n  {}   {}\n  {}   {}\n\n  {}",
        dim("commands"),
        bold("inspect <file> "),
        dim("summarize any .cyb: type, name, sections"),
        bold("focus   <graph>"),
        dim("run the tri-kernel: cyberank · syntropy · telemetry"),
        bold("vocab   <file> "),
        dim("a .vocab dictionary + self-consistency"),
        bold("model   <file> "),
        dim("a .model checkpoint: tensors · config · particle"),
        dim("focus is the heart — it converges a graph to φ* in the Goldilocks field."),
    );
}

// ── cli ─────────────────────────────────────────────────────────────────────

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
        #[arg(short, long, default_value_t = 20)]
        top: usize,
    },
    /// Summarize a `.vocab` dictionary and check its self-consistency.
    Vocab { path: PathBuf },
    /// Summarize a `.model` checkpoint: tensors, config, particle.
    Model { path: PathBuf },
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || matches!(args[0].as_str(), "help" | "--help" | "-h") {
        print!("{}", banner());
        help();
        return Ok(());
    }
    match Cli::parse().cmd {
        Cmd::Inspect { path } => inspect(&path),
        Cmd::Focus { path, top } => focus(&path, top),
        Cmd::Vocab { path } => vocab(&path),
        Cmd::Model { path } => model(&path),
    }
}

fn kv(key: &str, val: &str) -> String {
    format!("{} {}", dim(key), val)
}

fn inspect(path: &Path) -> Result<()> {
    let bytes = std::fs::read(path)?;
    let (fm_str, body) = frontmatter::split(&bytes)?;
    let fm = frontmatter::parse(fm_str)?;
    let sections = frontmatter::index_sections(&bytes, body, &fm.files)?;

    println!("{}  {}", bold(&fm.cyb.name), dim(&format!("[{}]", fm.cyb.types.join(", "))));
    println!("{}", dim(&format!("{} bytes · {} sections", bytes.len(), fm.files.len())));
    for f in &fm.files {
        let sz = sections.get(&f.name).map(|&(s, e)| e - s).unwrap_or(0);
        println!("  {} {}  {}", cyan(&format!("{:<11}", f.name)), dim(&format!("{:<8}", f.format)), yellow(&format!("{sz} B")));
    }
    Ok(())
}

fn focus(path: &Path, top: usize) -> Result<()> {
    let g = Graph::open(path)?;
    let n_links = g.cyberlinks()?.count();
    let links = g.cyberlinks()?.map(|cl| Link { from: cl.from, to: cl.to, amount: cl.amount, valence: cl.valence });
    let fg = FocusingGraph::build(links);
    let params = FocusingParams::default();
    let result = tru::compute_focusing(&fg, &params);
    let tel = focusing::telemetry(&fg, &result, &params);

    println!("{} {}", green("focus"), bold(g.name()));
    let sep = dim(" · ");
    println!("  {}{sep}{}", kv("particles", &yellow(&fg.n().to_string())), kv("cyberlinks", &yellow(&n_links.to_string())));
    println!(
        "  {}{sep}{}",
        kv("syntropy J", &yellow(&format!("{:.4}", tel.syntropy.to_f64()))),
        kv("entropy H", &yellow(&format!("{:.4}", tel.entropy.to_f64())))
    );
    println!(
        "  {}{sep}{}{sep}{}",
        kv("κ", &yellow(&format!("{:.3}", tel.kappa.to_f64()))),
        kv("λ₂", &yellow(&format!("{:.3}", tel.lambda_2.to_f64()))),
        kv("T(ε)", &yellow(&tel.steps.to_string()))
    );

    let mut ranked: Vec<(usize, f64)> = result.focus.iter().map(|x| x.to_f64()).enumerate().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("\n{}", dim(&format!("cyberank φ*(p) — top {}", top.min(ranked.len()))));
    for (idx, phi) in ranked.iter().take(top) {
        println!("  {}  {}", cyan(&hex8(fg.node_id(*idx))), yellow(&format!("{phi:.6}")));
    }
    Ok(())
}

fn vocab(path: &Path) -> Result<()> {
    let v = Vocab::read(path)?;
    let inline = v.entries.iter().filter(|e| !e.data.is_empty()).count();
    println!("{} {}", green("vocab"), bold(&v.name));
    println!("  {}", kv("entries", &format!("{} {}", yellow(&v.entries.len().to_string()), dim(&format!("({inline} with data)")))));
    println!("  {}", kv("particle", &cyan(&hex(&v.particle()))));
    match v.verify() {
        Ok(()) => println!("  {} {}", dim("self-consistency"), green("OK")),
        Err(e) => println!("  {} {}", dim("self-consistency"), paint("31", &format!("FAIL: {e}"))),
    }
    Ok(())
}

fn model(path: &Path) -> Result<()> {
    let m = Model::read(path)?;
    println!("{} {}", green("model"), bold(&m.name));
    println!("  {}", kv("particle", &cyan(&hex(&m.particle()))));
    println!("  {}", kv("tensors", &yellow(&m.tensors.len().to_string())));
    for t in &m.tensors {
        println!(
            "    {} {} {} {}",
            cyan(&format!("{:<34}", t.name)),
            dim(&format!("{:?}", t.shape)),
            yellow(&format!("{:?}", t.encoding)),
            dim(&format!("{} values", t.data.len()))
        );
    }
    if !m.config.is_empty() {
        println!("  {}", dim("config"));
        for line in m.config.lines().take(8) {
            println!("    {}", dim(line));
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
