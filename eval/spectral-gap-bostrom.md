---
title: spectral gap of bostrom, observed from convergence
tags: tru, eval, bostrom
date: 2026-03-23
---

# spectral gap of bostrom, observed from convergence

2026-03-23, first full-graph compilation of [[bostrom]] (uniform link weights,
2,921,230 particles). the method is the one in
[docs/explanation/convergence.md](../docs/explanation/convergence.md), observing
the gap: read the contraction $\kappa$ off the ratio of successive PageRank
$\ell_1$ differences, then $\lambda_2 = 1 - \kappa/\alpha$ with $\alpha = 0.85$.
this is the diffusion gap of the normalized Laplacian -- the $\lambda_2$ that
sets the [[ct0]] layer count -- not the Fiedler value of the weighted Laplacian
used inside the composite $\kappa$ of focusing.

## what the eigensolver did

ARPACK shift-invert Lanczos on the $2{,}921{,}230 \times 2{,}921{,}230$ normalized
Laplacian: 101 iterations, 1,932 seconds, zero converged eigenvectors. the
failure is structural -- one zero eigenvalue per disconnected component gives a
massive null space, and the target $\lambda_2$ sits near zero inside it.

## what the iteration said

```
PageRank iterations: 23  (converged at ε < 1e-6)
last diffs:      d19 = 4.83e-05   d20 = 3.57e-05   d21 = 2.64e-05
                 d22 = 1.96e-05   d23 = 7.91e-07  (below threshold, stopped)
ratios:          r20 = 0.739      r21 = 0.740      r22 = 0.742
κ̂  = median = 0.740
λ̂₂ = 1 − 0.740 / 0.85 = 0.129
```

zero extra seconds: the loop computes $d_t$ for its own convergence check.

## the correction

the architecture paper's estimate, $\lambda_2 \approx 0.0015$, came from Lanczos on
a 50,000-link sample -- a contiguous early subset, denser than the full graph,
and the solver did not converge on it either. the full-graph observation is two
orders of magnitude larger.

| parameter | paper ($\lambda_2 = 0.0015$) | observed ($\lambda_2 = 0.129$) |
|---|---|---|
| contraction $\kappa$ | 0.851 | 0.740 |
| convergence iterations | 29 | 17 |
| $L^*$ (transformer layers) | 290 | 102 |
| model size at $h^* = 12$ | 16.8 GB | 5.9 GB |

the network mixes faster than the sample predicted; the compiled model needs a
third of the layers.

## caveats

- uniform link weights, March 2026 -- before stake-weighted $A^{\text{eff}}$,
  before the preferential dangling fix (tru#1), before 2-hop mixing (tru#2).
  the pipeline that produced the $L^*$ column has moved on; the observed gap
  is the durable part.
- $L^*$ and model size use the March architecture formulas; today's pass 3 is
  `rs/pass/arch.rs`.
- one run, one graph. the method is general; the number is bostrom's.

see [lp_bostrom.md](lp_bostrom.md) for link prediction on the same snapshot,
and the bostrom snapshot manifest in [README.md](README.md) for the data.
