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
        "{}\n  {}   {}\n  {}   {}\n  {}   {}\n  {}   {}\n  {}   {}\n\n  {}",
        dim("commands"),
        bold("inspect <file> "),
        dim("summarize any .cyb: type, name, sections"),
        bold("focus   <graph>"),
        dim("run the tri-kernel: cyberank · syntropy · telemetry"),
        bold("impulse <graph>"),
        dim("Δφ* of a new link: the directed syntropy gain Δφ⁺"),
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
    /// With no path, reads the local snapshot ($TRU_GRAPH or ~/cyb/my.graph).
    Focus {
        path: Option<PathBuf>,
        #[arg(short, long, default_value_t = 20)]
        top: usize,
    },
    /// Compute the impulse Δφ* of one new link on a `.graph`: the directed
    /// syntropy gain Δφ⁺ (the reward primitive) and the sparse focus shift.
    Impulse {
        path: PathBuf,
        /// Source particle, by hex prefix (matched against the graph).
        #[arg(long)]
        from: String,
        /// Target particle, by hex prefix.
        #[arg(long)]
        to: String,
        /// Stake on the new link (smallest units).
        #[arg(long, default_value_t = 1000)]
        stake: u128,
        #[arg(short, long, default_value_t = 10)]
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
        Cmd::Focus { path, top } => focus(path, top),
        Cmd::Impulse { path, from, to, stake, top } => impulse(&path, &from, &to, stake, top),
        Cmd::Vocab { path } => vocab(&path),
        Cmd::Model { path } => model(&path),
    }
}

fn kv(key: &str, val: &str) -> String {
    format!("{} {}", dim(key), val)
}

/// The default local cybergraph snapshot: `$TRU_GRAPH`, else `~/cyb/my.graph`.
/// `~/cyb` is visible, not dotfile-hidden — nothing about a neuron's own
/// graph needs hiding from its owner.
fn default_graph() -> PathBuf {
    if let Ok(p) = std::env::var("TRU_GRAPH") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    Path::new(&home).join("cyb").join("my.graph")
}

/// Resolve the graph path: the one given, or the local default. Errors with
/// guidance if the default is missing.
fn resolve_graph(path: Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(p) => Ok(p),
        None => {
            let p = default_graph();
            if p.exists() {
                Ok(p)
            } else {
                anyhow::bail!(
                    "no graph given and none at {}\n  {}\n  {}",
                    p.display(),
                    "pass a path:      tru focus <file.graph>",
                    "or set the default: export TRU_GRAPH=/path/to/snapshot.graph"
                )
            }
        }
    }
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

fn focus(path: Option<PathBuf>, top: usize) -> Result<()> {
    let path = resolve_graph(path)?;
    let g = Graph::open(&path)?;
    let n_links = g.cyberlinks()?.count();
    // Preserve the real neuron and valence from each record so karma wiring is a
    // one-line change; karma and ICBS price arrive from bbg, absent here, so this
    // pass runs stake-only (Karma::none(), neutral price = 1).
    let links = g.cyberlinks()?.map(|cl| Link {
        neuron: cl.neuron,
        from: cl.from,
        to: cl.to,
        amount: cl.amount,
        valence: cl.valence,
        price: tru::arithmetic::Fx::ONE,
    });
    let fg = FocusingGraph::build(links, &focusing::Karma::none());
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

    // Spectral embedding — the (x, y) mir reads for layout (Fiedler + λ₃).
    let emb = fg.embedding(2, 200);
    let has_xy = emb.k >= 2;

    let mut ranked: Vec<(usize, f64)> = result.focus.iter().map(|x| x.to_f64()).enumerate().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    let head = if has_xy { "cyberank φ*(p) · position (x,y)" } else { "cyberank φ*(p)" };
    println!("\n{}", dim(&format!("{head} — top {}", top.min(ranked.len()))));
    for (idx, phi) in ranked.iter().take(top) {
        let pos = if has_xy {
            dim(&format!("  ({:+.4}, {:+.4})", emb.coords[*idx][0].to_f64(), emb.coords[*idx][1].to_f64()))
        } else {
            String::new()
        };
        println!("  {}  {}{}", cyan(&hex8(fg.node_id(*idx))), yellow(&format!("{phi:.6}")), pos);
    }
    Ok(())
}

/// Read a graph's cyberlinks as focusing `Link`s (stake-only: neutral price,
/// no karma source wired yet).
fn read_links(g: &Graph) -> Result<Vec<Link>> {
    Ok(g.cyberlinks()?
        .map(|cl| Link {
            neuron: cl.neuron,
            from: cl.from,
            to: cl.to,
            amount: cl.amount,
            valence: cl.valence,
            price: tru::arithmetic::Fx::ONE,
        })
        .collect())
}

/// Resolve a hex prefix to a unique particle among `parts`.
fn resolve_particle(parts: &[[u8; 32]], prefix: &str) -> Result<[u8; 32]> {
    let p = prefix.trim().trim_end_matches('…').to_lowercase();
    let hits: Vec<[u8; 32]> = parts.iter().filter(|h| hex(*h).starts_with(&p)).copied().collect();
    match hits.as_slice() {
        [one] => Ok(*one),
        [] => anyhow::bail!("no particle matches prefix '{prefix}'"),
        many => anyhow::bail!("prefix '{prefix}' is ambiguous — {} particles match", many.len()),
    }
}

fn impulse(path: &Path, from: &str, to: &str, stake: u128, top: usize) -> Result<()> {
    let g = Graph::open(path)?;
    let base = read_links(&g)?;

    // Particle universe: every endpoint in the base graph.
    let mut parts: Vec<[u8; 32]> = Vec::new();
    for l in &base {
        for h in [l.from, l.to] {
            if !parts.contains(&h) {
                parts.push(h);
            }
        }
    }
    let from_h = resolve_particle(&parts, from)?;
    let to_h = resolve_particle(&parts, to)?;

    // The signal: one new link, authored by the source, at neutral market price.
    let batch = vec![Link {
        neuron: from_h,
        from: from_h,
        to: to_h,
        amount: stake,
        valence: 1,
        price: tru::arithmetic::Fx::ONE,
    }];
    let params = FocusingParams::default();
    let imp = tru::impulse(&base, &batch, &focusing::Karma::none(), &params, params.epsilon);

    println!("{} {} {} {}", green("impulse"), cyan(&hex8(&from_h)), dim("→"), cyan(&hex8(&to_h)));
    let sep = dim(" · ");
    println!("  {}", kv("stake", &yellow(&stake.to_string())));
    println!(
        "  {}{sep}{}",
        kv("Δφ⁺ reward", &yellow(&format!("{:.6}", imp.directed.to_f64()))),
        kv("ΔJ", &yellow(&format!("{:+.6}", imp.delta_j.to_f64())))
    );
    println!(
        "  {}{sep}{}{sep}{}",
        kv("entropy drop", &yellow(&format!("{:+.6}", imp.entropy_drop.to_f64()))),
        kv("discovery", &yellow(&format!("{:+.6}", imp.discovery.to_f64()))),
        kv("‖Δφ*‖₁", &yellow(&format!("{:.6}", imp.norm_l1.to_f64())))
    );

    let mut d = imp.delta.clone();
    d.sort_by(|a, b| b.1.to_f64().abs().total_cmp(&a.1.to_f64().abs()));
    println!("\n{}", dim(&format!("Δφ*(p) — top {} by |shift|", top.min(d.len()))));
    for (pid, dv) in d.iter().take(top) {
        let v = dv.to_f64();
        let arrow = if v >= 0.0 { green("▲") } else { paint("31", "▼") };
        println!("  {} {}  {}", cyan(&hex8(pid)), arrow, yellow(&format!("{v:+.6}")));
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
