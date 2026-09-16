---
title: CT-0 end-to-end on a real graph — space-pussy
tags: tru, eval, cybergraph
tru: issues #5 #6 #7
---

# CT-0 end-to-end on a real graph — space-pussy

2026-09-11/12. the full run of the SHIPPED pipeline on real data:
`pussy_links.jsonl` -> temporal 90/10 split -> passes 1-5 as compiled
(`index` -> `dialect` -> `arch` -> `embed` -> `attn`, shipped hop2_mix and
out_gain) -> glia-semantics llama forward. `rs/examples/eval_pussy.rs`
(shipped Fx path), `rs/examples/eval_pussy_mix.rs` (f64 mirror on the
scipy spectrum, eval/mirror_svd.py).

## compiled architecture

d*=64, h*=1 (single token denomination), L*=36, diam=9,
sigma_1/sigma_k = 218.8 (probe) -> out_gain = 8.77. vocab 65,121
particles (32,560 link endpoints + 32,561 axons — every cyberlink is
itself a particle, §3).

## task

predict the target of a held-out (temporal test) cyberlink from a
train-graph walk prefix ending at its source: [f], [b,f], [a,b,f] with
train edges a->b, b->f, held-out f->t. n = 1181 queries. metric: MRR
over the full particle vocab.

## results (n=1181)

shipped pipeline, 2026-09-12 revision: banded field SVD (full-rank
tail), row-normalized E, quiet attention gain (tru#5).

| model | MRR |
|---|---|
| embed-floor (row-normed banded-Fx E) | **0.4886** |
| zero-layer (plumbing check) | 0.4886 |
| bigram (add-k counts) | 0.5719 |
| fwd-1layer (loud attn, this eval) | 0.1558 |
| fwd-full (loud attn, 36 layers) | 0.1558 |
| unigram (phi*) | 0.0064 |

zero-layer == embed-floor exactly: the plumbing is sound. the
spectral ratio is now sigma1/sigma_k = 1.58e8 — the banded spectrum
recovers the full 64-component tail (the collapsed 14-component
version read 218.8). NOTE on the history below: the 0.6362 floor of
the earlier revision rode on the collapsed-tail artifact (tru#7) —
the honest full-rank field number is 0.4886, still above the exact
scipy mirror at the same gauge (0.392 cosine). pussy remains the
hard small graph; the bostrom ladder is where the floor reaches the
counts baseline (0.89 at ~1M particles).

## the attention verdict (measured, multi-mechanism)

the compiled attention stack is net-negative in EVERY configuration
tested on real walks: shipped Fx weights, f64 mirror, walk kernels
(P_dir, forward/backward), Gram kernels (G, G^2), gains 0.25–8.8,
role-blend theta 0–45deg, spectral exponents p 0–0.5, rank truncations
4–64. three distinct failure mechanisms, all measured:

1. **information ceiling.** on a directed walk the answer t is never in
   the prefix; the only successor signal is E_f itself (the residual
   carries it for free). attention can inject functions of the prefix
   only, and on this graph they all hurt.
2. **walk kernels dilute.** P_dir retrieves predecessors; injecting
   gain-scaled predecessor embeddings moves the tied-head ranking away
   from the successor cone.
3. **Gram kernels self-collapse.** with near-orthonormal embeddings,
   G ~= the projector: scores become self-dominant (Cauchy-Schwarz),
   ctx comes out parallel to E_f, and the output is invariant to the
   gain (observed: identical MRR across a 35x gain sweep).

plus a scale pathology that made every gain meaningless: compiled
embedding ROW norms span ~1e-4 (long tail) to O(1) (hubs), so with
W_V = I any gain >= 1 swamps the residual 10^4x. rmsnorm inside the
block hides it; the residual add does not.

**design consequence (tru#5): attention ships as a training-ready
substrate, quiet at init** — the MLP's existing pattern (LayerScale
gamma = 1e-5). keep the structural Q/K (a training prior), keep
OUT_GAIN for post-training calibration, set the block's init gain to
~1e-2..1e-5. the open decisive experiment is a fine-tuning ablation:
structural init vs random init on walk continuation. the claim
"attention works" defers to that result; init-time MRR on pure
structural walks is information-capped and cannot demonstrate it.

## the embedding-floor anomaly (open, load-bearing — tru#7)

cross-checking the shipped Fx embedding against the exact scipy
spectrum of the same operator (mirror_svd.py):

- the Fx SVD (fixed-point subspace iteration, k=64, 120 iters)
  resolves only ~14 of 64 components; the rest collapse to zero. the
  14 columns are orthonormal; the middle ones are rotated 10-40 deg
  against the exact singular vectors (|cos| 0.76-1.0) and the spectrum
  is distorted (true sigma_3 = 0.48 rides at 0.16% of top; directions
  5..20 carry ~10x their true weight).
- floor MRR: shipped Fx E = **0.64**; exact scipy E at ANY truncation
  (k = 4..64), ANY spectral flattening (clamp, power law), ANY band
  pass = **0.38-0.41 — BELOW bigram (0.57)**.

so the current "compiled geometry beats counts" result on this task
rides on an uncontrolled fixed-point artifact. nothing in the exact
spectral subspace reproduces it. open: (a) reproduce from a clean
build; (b) identify the mechanism (candidates: fixed-point noise as
quasi-random features; Ritz-vector bias); (c) either control it
deliberately or narrow the claim. NOTE the earlier LP-eval results
(AUC-based, pairwise) are a different metric and remain valid on their
own terms.

## infra fixes this session

- **tru#5 (fixed):** attn.rs stored Q/K transposed ([in][out]) vs the
  .model [out,in] convention (glia matmul y = x @ W^T). a compiled model
  computed sqrt(S)(U^T V)sqrt(S) instead of P. W_V/W_O are identity-
  scaled and immune. found because mirror scores collapsed to ~1e-5.
- eval-side SVD used the symmetric-only Rayleigh shortcut
  sqrt(v^T M v); on directed M the correct sigma = ||M v|| (rank-2
  garbage otherwise). fixed in eval_ct0/eval_attn/eval_pussy_mix.
- eval-side subspace iteration collapses the tail (exact zeros past
  component 6 on n=65k) — mirror now loads scipy svds.
- arch probe (k=1024, iters=120) was hours at fixed-point Gram-Schmidt
  cost; now (64, 30). Gram-Schmidt caches loop-invariant denominators.
- TRU_SVD_DEBUG=1 / MIX_DEBUG=1 give per-iter / per-layer telemetry.

## honest status

compile machinery: works end-to-end, deterministic, glia-loadable.
embedding floor: strong BUT the margin over counts is currently an
uncontrolled artifact (tru#7). attention: net-negative at init in every
configuration; ships quiet as a training substrate (tru#6); the
fine-tuning ablation is the next real experiment.
