use super::csr::CsrMatrix;

/// Personalized PageRank with stake-weighted teleport.
///
/// rank_new[i] = α·stake[i] + (1-α)·(dangling_sum·stake[i] + (T·rank)[i])
///
/// `transition[i][j]` = w(j→i) / out_degree(j) — already normalized at build time.
/// `dangling[i]` = true when node i has no outgoing edges in the directed graph.
pub fn diffusion(
    n: usize,
    stake: &[f64],
    transition: &CsrMatrix,
    dangling: &[bool],
    alpha: f64,
    max_iter: usize,
    convergence: f64,
) -> Vec<f64> {
    let mut rank = stake.to_vec();
    let mut tmp = vec![0.0f64; n];

    for _ in 0..max_iter {
        let dangling_sum: f64 = rank
            .iter()
            .enumerate()
            .filter(|&(i, _)| dangling[i])
            .map(|(_, r)| r)
            .sum();

        transition.spmv(&rank, &mut tmp);
        let mut delta = 0.0f64;
        for i in 0..n {
            let new_r = alpha * stake[i]
                + (1.0 - alpha) * (dangling_sum * stake[i] + tmp[i]);
            delta += (new_r - rank[i]).abs();
            rank[i] = new_r;
        }
        if delta < convergence {
            break;
        }
    }
    rank
}

/// Screened Laplacian inverse via Jacobi iteration.
///
/// Solves (μI + L)·x = stake, where L is the weighted graph Laplacian.
/// L·x = D·x − W·x, so (μI + L)·x = (μ + d[i])·x[i] − (W·x)[i]
/// Jacobi: x_new[i] = (stake[i] + (W·x)[i]) / (μ + d[i])
///
/// `sym_weights[i][j]` = edge weight (symmetric). `und_degree[i]` = weighted degree.
pub fn springs(
    n: usize,
    stake: &[f64],
    sym_weights: &CsrMatrix,
    und_degree: &[f64],
    mu: f64,
    max_iter: usize,
    convergence: f64,
) -> Vec<f64> {
    let mut x = stake.to_vec();
    let mut wx = vec![0.0f64; n];
    let mut x_new = vec![0.0f64; n];

    for _ in 0..max_iter {
        sym_weights.spmv(&x, &mut wx);
        let mut delta = 0.0f64;
        for i in 0..n {
            x_new[i] = (stake[i] + wx[i]) / (mu + und_degree[i]);
            delta += (x_new[i] - x[i]).abs();
        }
        x.copy_from_slice(&x_new);
        if delta < convergence {
            break;
        }
    }

    normalize_l1(&mut x);
    x
}

/// Heat kernel approximation: e^{−τL} applied to stake via forward Euler substeps.
///
/// Each substep: s_new[i] = s[i] + dt·((N·s)[i] − s[i])
/// where N[i][j] = W[i][j] / d[j] (column-stochastic normalized adjacency).
///
/// `sym_weights` is symmetric. `und_degree` is weighted degree.
pub fn heat(
    n: usize,
    stake: &[f64],
    sym_weights: &CsrMatrix,
    und_degree: &[f64],
    tau: f64,
    substeps: usize,
) -> Vec<f64> {
    let mut s = stake.to_vec();
    let dt = tau / substeps as f64;
    let mut ns = vec![0.0f64; n];

    for _ in 0..substeps {
        // (N·s)[i] = Σ_j W[i][j]/d[j] · s[j]
        // We compute this as W·(s/d) element-wise then accumulate
        // Build scaled input: inp[j] = s[j] / d[j]
        let inp: Vec<f64> = (0..n).map(|j| {
            if und_degree[j] > 0.0 { s[j] / und_degree[j] } else { 0.0 }
        }).collect();
        sym_weights.spmv(&inp, &mut ns);
        for i in 0..n {
            s[i] += dt * (ns[i] - s[i]);
        }
    }

    normalize_l1(&mut s);
    s
}

pub fn normalize_l1(v: &mut [f64]) {
    let sum: f64 = v.iter().sum();
    if sum > 0.0 {
        v.iter_mut().for_each(|x| *x /= sum);
    }
}
