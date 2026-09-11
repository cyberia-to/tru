---
title: CT-0 end-to-end on a real graph — space-pussy
tags: tru, eval, cybergraph
tru: issues #3, attention-orientation
---

# CT-0 end-to-end on a real graph — space-pussy

2026-09-11. the first full run of the SHIPPED pipeline on real data:
`pussy_links.jsonl` -> temporal 90/10 split -> passes 1-5 as compiled
(`index` -> `dialect` -> `arch` -> `embed` -> `attn`, shipped hop2_mix and
out_gain) -> glia-semantics llama forward (multi-head, RMSNorm eps 1e-5,
RoPE theta 1e6, MLP skipped: LayerScale gamma = 1e-5 at init).
`rs/examples/eval_pussy.rs`.

## compiled architecture

d*=64, h*=1 (single token denomination), L*=36, diam=9,
sigma_1/sigma_k = 218.8 (probe) -> out_gain = 8.77. vocab 65,121 particles
(32,560 link endpoints + 32,561 axons — every cyberlink is itself a
particle, §3). compile wall time: arch probe 40s + embed 192s (k=64,
120 iters) + attn ~1 min on an M-series laptop.

## task

predict the target of a held-out (temporal test) cyberlink from a
train-graph walk prefix ending at its source: [f], [b,f], [a,b,f] with
train edges a->b, b->f, held-out f->t. n = 1181 queries. metrics: MRR
over the full particle vocab.

## results

| model | MRR |
|---|---|
| embed-floor <E_f, E_j> | **0.6362** |
| zero-layer (E -> final norm -> tied head) | 0.6362 |
| bigram (add-k counts) | 0.5719 |
| fwd-1layer (shipped attn, first layer) | 0.3030 |
| fwd-full (shipped attn, 36 layers) | 0.3036 |
| unigram (phi*) | 0.0064 |

mean attention back-mass at the query: 0.382. zero-layer == embed-floor
exactly: the plumbing is sound (final rmsnorm is rank-invariant for the
pure-embedding path).

## the sober finding

**the compiled attention stack is net-negative on real walk
continuation: -0.33 MRR against the embedding floor it starts from.**
and it saturates in one layer (fwd-1layer == fwd-full to 3 decimals).

mechanism: the retrieval head retrieves the PREFIX. QK come from
P_dir = walks s -> t, so the query at f attends to its predecessors in
walk order (back-mass 0.38, the rest spread over earlier prefix). the
attention output is the gain-scaled (8.8x) mix of predecessor
embeddings injected into the residual at f. but the optimal
successor-prediction signal is already in E_f itself — the residual
carries it for free. predecessor geometry and successor geometry are
different subspaces of E (the V-side fix that bought +0.11 AUC in LP
exists precisely because U and V differ). diluting E_f with
8.8x-gained predecessor embeddings moves the tied-head ranking AWAY
from the successor cone. causal attention on a directed walk cannot
retrieve the answer (t is never in the prefix) — it can only add
context, and on this graph the context hurts.

on the toy corpus (eval_attn.md) retrieval helped because the toy
sentences are template-generated: prefix tokens (class words, probes)
share embedding geometry with the target. real walks are not
templates. the sign of the attention contribution flipped between the
toy and the graph — which is why the toy stopped being an arbiter.

## what this rules in / out

- OUT: "attention as compiled retrieves useful context" on structural
  walks. measured, not assumed.
- OUT: more layers helping — 36 layers == 1 layer.
- OPEN: orienting retrieval toward successor geometry. the pass-4 SVD
  already computes V sqrt(Sigma) (the target-side factor) and discards
  it: the shipped embedding ships only U sqrt(Sigma). a weight-only
  variant — Q from the V-side ("what would f point at"), K from the
  U-side ("what does s point at") — retrieves prefix tokens whose
  OUT-geometry matches f's out-geometry (homophilic context) instead of
  f's predecessors. testable eval-side in the eval_attn harness before
  touching the passes.
- OPEN: whether ANY structural init of attention can beat the
  embedding floor on walk continuation, or whether the honest compiled
  init is near-identity (small gamma) leaving attention to training.

## infra notes (fixed in the same session)

- the arch spectrum probe was m_svd(k=1024, iters=120): O(iters*k^2*n)
  fixed-point Gram-Schmidt = ~1.27s/iter at k=64, n=65k -> the probe
  alone was hours; the full toy-size compile never noticed. probe is
  now k=64, iters=30 (clamp range absorbs it; pass 4 reruns at d*).
- Gram-Schmidt recomputed every dot(block[i], block[i]) per pair; the
  denominator is loop-invariant in MGS (column i is final after step i)
  and is now cached. ~1.5-3x on all SVD passes.
- TRU_SVD_DEBUG=1 prints per-iter timing from top_svd.
