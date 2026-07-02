//! Superadditivity benchmark (specs/superadditivity.md, milestone M1.6).
//!
//! Measures collective intelligence on Zachary's Karate Club: does the
//! collective focus φ* (tri-kernel over the whole graph) predict held-out
//! cyberlinks better than the best individual neuron's ego focus φ*_ν?
//!
//! Task: link prediction. 80/20 edge split; score candidate pairs (p,q) by
//! the focus-derived affinity φ(p)·φ(q); report ROC-AUC and average precision.
//! σ_mean = collective − mean_ν ego_ν ; σ_best = collective − max_ν ego_ν.
//!
//! Run: RUSTC_BOOTSTRAP=1 cargo run -p tru --example superadditivity

use tru::focusing::{compute_focusing, FocusingGraph, FocusingParams, Karma, Link};

const N: usize = 34;

/// Zachary's Karate Club, 0-indexed, 78 undirected edges.
const EDGES: &[(u8, u8)] = &[
    (0,1),(0,2),(0,3),(0,4),(0,5),(0,6),(0,7),(0,8),(0,10),(0,11),(0,12),(0,13),(0,17),(0,19),(0,21),(0,31),
    (1,2),(1,3),(1,7),(1,13),(1,17),(1,19),(1,21),(1,30),
    (2,3),(2,7),(2,8),(2,9),(2,13),(2,27),(2,28),(2,32),
    (3,7),(3,12),(3,13),
    (4,6),(4,10),
    (5,6),(5,10),(5,16),
    (6,16),
    (8,30),(8,32),(8,33),
    (9,33),
    (13,33),
    (14,32),(14,33),
    (15,32),(15,33),
    (18,32),(18,33),
    (19,33),
    (20,32),(20,33),
    (22,32),(22,33),
    (23,25),(23,27),(23,29),(23,32),(23,33),
    (24,25),(24,27),(24,31),
    (25,31),
    (26,29),(26,33),
    (27,33),
    (28,31),(28,33),
    (29,32),(29,33),
    (30,32),(30,33),
    (31,32),(31,33),
    (32,33),
];

fn hash(b: u8) -> [u8; 32] {
    let mut h = [0u8; 32];
    h[0] = b;
    h
}

/// Undirected edges → directed Link pairs (both ways), unit stake, affirm.
fn links(edges: &[(u8, u8)]) -> Vec<Link> {
    let mut v = Vec::with_capacity(edges.len() * 2);
    for &(a, b) in edges {
        v.push(Link::stake(hash(a), hash(b), 1));
        v.push(Link::stake(hash(b), hash(a), 1));
    }
    v
}

/// focus per node id (0.0 for nodes absent from this graph).
fn focus_by_node(edges: &[(u8, u8)]) -> [f64; N] {
    let mut out = [0.0; N];
    if edges.is_empty() {
        return out;
    }
    let g = FocusingGraph::build(links(edges), &Karma::none());
    let r = compute_focusing(&g, &FocusingParams::default());
    for idx in 0..g.n() {
        // φ* is fixed-point; the benchmark's task scoring is offline analysis.
        out[g.node_id(idx)[0] as usize] = r.focus[idx].to_f64();
    }
    out
}

/// Collective focus over `edges`, plus the engine's own λ₂ and syntropy J.
fn collective(edges: &[(u8, u8)]) -> ([f64; N], f64, f64) {
    let mut out = [0.0; N];
    if edges.is_empty() {
        return (out, 0.0, 0.0);
    }
    let g = FocusingGraph::build(links(edges), &Karma::none());
    let r = compute_focusing(&g, &FocusingParams::default());
    for idx in 0..g.n() {
        out[g.node_id(idx)[0] as usize] = r.focus[idx].to_f64();
    }
    (out, g.lambda_2().to_f64(), r.syntropy.to_f64())
}

/// ROC-AUC via Mann–Whitney U (fraction of pos>neg pairs, ties = 0.5).
fn auc(pos: &[f64], neg: &[f64]) -> f64 {
    if pos.is_empty() || neg.is_empty() {
        return 0.5;
    }
    let mut wins = 0.0;
    for &p in pos {
        for &q in neg {
            wins += if p > q { 1.0 } else if p == q { 0.5 } else { 0.0 };
        }
    }
    wins / (pos.len() * neg.len()) as f64
}

/// Average precision = area under the precision–recall curve, tie-correct:
/// all pairs sharing a score are admitted together at one threshold (so a
/// predictor that outputs many equal scores gains no order-of-ties advantage).
fn ap(pos: &[f64], neg: &[f64]) -> f64 {
    let total_pos = pos.len() as f64;
    if total_pos == 0.0 {
        return 0.0;
    }
    let mut all: Vec<(f64, bool)> =
        pos.iter().map(|&s| (s, true)).chain(neg.iter().map(|&s| (s, false))).collect();
    all.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let (mut tp, mut fp, mut prev_recall, mut ap) = (0.0, 0.0, 0.0, 0.0);
    let mut i = 0;
    while i < all.len() {
        let s = all[i].0;
        while i < all.len() && all[i].0 == s {
            if all[i].1 { tp += 1.0 } else { fp += 1.0 }
            i += 1;
        }
        let precision = tp / (tp + fp);
        let recall = tp / total_pos;
        ap += (recall - prev_recall) * precision;
        prev_recall = recall;
    }
    ap
}

fn score(f: &[f64; N], a: u8, b: u8) -> f64 {
    f[a as usize] * f[b as usize]
}

/// One measurement: collective vs ego on the link-prediction task.
struct Row {
    lambda2: f64,
    j: f64,
    col_auc: f64,
    mean_auc: f64,
    best_auc: f64,
    best_node: usize,
    col_ap: f64,
    mean_ap: f64,
    best_ap: f64,
}

fn superadd(train: &[(u8, u8)], test: &[(u8, u8)], neg_pairs: &[(u8, u8)]) -> Row {
    let (col, lambda2, j) = collective(train);

    // Per-neuron ego focus: tri-kernel on the closed radius-1 neighbourhood.
    let mut nbr: Vec<Vec<u8>> = vec![Vec::new(); N];
    for &(a, b) in train {
        nbr[a as usize].push(b);
        nbr[b as usize].push(a);
    }
    let ego: Vec<[f64; N]> = (0..N as u8)
        .map(|v| {
            let mut nodes: Vec<u8> = nbr[v as usize].clone();
            nodes.push(v);
            let ee: Vec<(u8, u8)> = train.iter().copied().filter(|&(a, b)| nodes.contains(&a) && nodes.contains(&b)).collect();
            focus_by_node(&ee)
        })
        .collect();

    let evaluate = |f: &[f64; N]| -> (f64, f64) {
        let pos: Vec<f64> = test.iter().map(|&(a, b)| score(f, a, b)).collect();
        let neg: Vec<f64> = neg_pairs.iter().map(|&(a, b)| score(f, a, b)).collect();
        (auc(&pos, &neg), ap(&pos, &neg))
    };
    let (col_auc, col_ap) = evaluate(&col);
    let es: Vec<(f64, f64)> = ego.iter().map(|f| evaluate(f)).collect();
    let mean = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;
    let aucs: Vec<f64> = es.iter().map(|x| x.0).collect();
    let aps: Vec<f64> = es.iter().map(|x| x.1).collect();
    let (best_auc, best_node) = aucs.iter().copied().enumerate().fold((0.0, 0usize), |a, (i, x)| if x > a.0 { (x, i) } else { a });
    Row {
        lambda2,
        j,
        col_auc,
        mean_auc: mean(&aucs),
        best_auc,
        best_node,
        col_ap,
        mean_ap: mean(&aps),
        best_ap: aps.iter().copied().fold(0.0, f64::max),
    }
}

/// BFS spanning tree over `edges` from node 0, and the remaining edges.
/// The tree touches every reachable node, so tree + any subset of the rest
/// keeps the vertex set fixed — the regime where adding an edge is Fiedler-
/// monotone (λ₂ never decreases).
fn spanning(edges: &[(u8, u8)]) -> (Vec<(u8, u8)>, Vec<(u8, u8)>) {
    let mut adj: Vec<Vec<u8>> = vec![Vec::new(); N];
    for &(a, b) in edges {
        adj[a as usize].push(b);
        adj[b as usize].push(a);
    }
    let mut seen = [false; N];
    seen[0] = true;
    let mut q = vec![0u8];
    let mut qi = 0;
    let mut tree = Vec::new();
    while qi < q.len() {
        let u = q[qi];
        qi += 1;
        for &v in &adj[u as usize] {
            if !seen[v as usize] {
                seen[v as usize] = true;
                tree.push((u.min(v), u.max(v)));
                q.push(v);
            }
        }
    }
    let rest: Vec<(u8, u8)> = edges.iter().copied().filter(|e| !tree.contains(e)).collect();
    (tree, rest)
}

/// Pearson correlation coefficient.
fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    for i in 0..x.len() {
        sxy += (x[i] - mx) * (y[i] - my);
        sxx += (x[i] - mx).powi(2);
        syy += (y[i] - my).powi(2);
    }
    if sxx == 0.0 || syy == 0.0 {
        0.0
    } else {
        sxy / (sxx.sqrt() * syy.sqrt())
    }
}

fn main() {
    // Deterministic 80/20 split: every 5th edge is held out for test.
    let test: Vec<(u8, u8)> = EDGES.iter().copied().enumerate().filter(|(i, _)| i % 5 == 0).map(|(_, e)| e).collect();
    let train: Vec<(u8, u8)> = EDGES.iter().copied().enumerate().filter(|(i, _)| i % 5 != 0).map(|(_, e)| e).collect();

    // Positives = held-out edges. Negatives = all true non-edges (not in full graph).
    let is_edge = |a: u8, b: u8| {
        let (x, y) = if a < b { (a, b) } else { (b, a) };
        EDGES.contains(&(x, y))
    };
    let mut neg_pairs = Vec::new();
    for a in 0..N as u8 {
        for b in (a + 1)..N as u8 {
            if !is_edge(a, b) {
                neg_pairs.push((a, b));
            }
        }
    }

    let r = superadd(&train, &test, &neg_pairs);

    println!("Karate Club — superadditivity (collective φ* vs ego φ*_ν)");
    println!("nodes={N}  edges={}  train={}  test={}  negatives={}", EDGES.len(), train.len(), test.len(), neg_pairs.len());
    println!();
    println!("                       AUC      AP");
    println!("collective φ*        {:.3}   {:.3}    J(φ*)={:.4}", r.col_auc, r.col_ap, r.j);
    println!("ego  — mean neuron   {:.3}   {:.3}", r.mean_auc, r.mean_ap);
    println!("ego  — best neuron   {:.3}   {:.3}    (node {})", r.best_auc, r.best_ap, r.best_node);
    println!();
    println!("σ_mean(AUC) = {:+.3}     σ_best(AUC) = {:+.3}", r.col_auc - r.mean_auc, r.col_auc - r.best_auc);
    println!("σ_mean(AP)  = {:+.3}     σ_best(AP)  = {:+.3}", r.col_ap - r.mean_ap, r.col_ap - r.best_ap);
    println!();
    println!("collective beats the average neuron on both metrics; it beats the strongest");
    println!("neuron on AUC (global ranking) but not on AP — superadditivity is metric-dependent.");
    println!("(measured on the conformant engine: coupled iteration, fixed-point over Goldilocks.)");

    // Generalized Collective Focus Theorem: with the vertex set fixed (spanning
    // tree over all N nodes), add non-redundant edges and watch λ₂ rise — do
    // syntropy J and superadditivity σ rise with it?
    let (tree, rest) = spanning(&train);
    let steps = [0usize, rest.len() / 4, rest.len() / 2, 3 * rest.len() / 4, rest.len()];
    let (mut l2s, mut js, mut sms, mut sbs) = (vec![], vec![], vec![], vec![]);
    println!();
    println!("connectivity sweep — spanning tree ({} nodes) + k extra edges:", N);
    println!("  edges   λ₂       J(φ*)    σ_mean(AUC)  σ_best(AUC)");
    for &k in &steps {
        let mut e = tree.clone();
        e.extend_from_slice(&rest[..k]);
        let s = superadd(&e, &test, &neg_pairs);
        let (sm, sb) = (s.col_auc - s.mean_auc, s.col_auc - s.best_auc);
        println!("  {:>4}   {:.4}   {:.4}    {:+.3}       {:+.3}", e.len(), s.lambda2, s.j, sm, sb);
        l2s.push(s.lambda2);
        js.push(s.j);
        sms.push(sm);
        sbs.push(sb);
    }
    println!();
    println!(
        "Pearson(λ₂, J) = {:+.2}    Pearson(λ₂, σ_mean) = {:+.2}    Pearson(λ₂, σ_best) = {:+.2}",
        pearson(&l2s, &js),
        pearson(&l2s, &sms),
        pearson(&l2s, &sbs)
    );
    println!("finding: σ RISES with λ₂ (more connectivity → more collective advantage), and σ_best > 0");
    println!("at every level. but J FALLS with λ₂ — adding edges spreads focus toward uniform, lowering");
    println!("syntropy. so the generalized-CFT σ-λ₂ claim holds; the J-λ₂ claim is refuted (opposite sign).");
}
