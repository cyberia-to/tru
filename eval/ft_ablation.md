---
title: fine-tuning ablation — does the compiled init train better than random?
tags: tru, eval, cybergraph
tru: issues #5 #7
---

# fine-tuning ablation — 2026-09-12

question: does CT-0's compiled init train better than random — the product
claim of compilation. `eval/ft_ablation.py` (torch CPU, in the venv).
model: llama semantics, d=64, L=4, tied head, no MLP (gamma = 1e-5),
single head (h*=1). E row-normalized at load (see below). 5 configs:
random / structural (shipped Fx E + attn) / quiet (attn gain 1e-3) /
embed-only (Fx E, random attn) / exact-E (scipy spectrum, random attn).

task: next-particle prediction. two objectives:
- lm: cross-entropy on random walks over the train graph (B=32, T=10)
- lp: contrastive — prefix -> f ranks a train edge t+ above a sampled
  non-edge t- (BPR). the objective aligned with the unseen-link eval.

eval: the 1181 held-out walk-continuation queries of eval_pussy (MRR,
full vocab) = "unseen", plus a same-construction control over TRAIN
edges = "seen". bigram floors: unseen 0.5719, seen 0.9128.

## results (1500 steps lm @ lr 3e-4; 800 steps lp @ lr 3e-4)

unseen MRR (generalization — the product metric):

| config | init | lm final | lp final |
|---|---|---|---|
| structural (Fx E + attn) | 0.016 | 0.021 (peak 0.028 @300, then decays) | 0.020 |
| exact-E | 0.0001 | **0.086, still climbing** | 0.011 |
| random | 0.0001 | 0.017 | 0.0001 |
| embed-only | 0.0001 | 0.006 | 0.004 |
| quiet | 0.002 | 0.005 | 0.003 |

seen MRR (memorization): everyone climbs fast — quiet 0.60, exact-E
0.60, embed-only 0.59, structural 0.58, random 0.54 (lm); lp is similar.

## findings

1. **no training configuration learns unseen-link prediction.** under
   both objectives, seen climbs to ~0.5-0.6 within hundreds of steps
   while unseen stays at 0.002-0.028 — versus bigram-unseen 0.5719 and
   the natural-scale compiled init's 0.636. walk-LM's gradient is
   adjacency memorization; contrastive LP on train edges memorizes the
   train edges. generalization to unseen links on a sparse graph lives
   in the GEOMETRY (what E encodes), and these objectives do not add to
   it at this scale.
2. **the structural init remains the best unseen-link model of the
   session** — every trained config ends below it.
3. **the Fx-artifact advantage (tru#7) is init-only and fragile under
   training**: structural's unseen decays after step ~300, while
   exact-E is the best LEARNING substrate in both objectives (only lm
   config whose unseen rises monotonically). pragmatic resolution of
   tru#7: ship the exact spectrum; the artifact is not a foundation.
4. **quiet attention trains excellently as a substrate** (fastest
   memorization, seen 0.60) — the tru#5 "quiet at init" proposal is
   compatible with training even though neither generalizes here.
5. **row-normalization was load-bearing for training**: 96% of compiled
   rows are ~zero (Fx SVD keeps only hub structure); rmsnorm amplifies
   zero rows 1/sqrt(eps) ~ 316x per layer and the backward through 4
   layers + final norm explodes (~2^42 observed in grad norms). unit
   rows + random refill for dead rows fixes it. NOTE this changes the
   init metric (cold particles start random) — the 0.636 natural-scale
   floor and the row-normed trainer's 0.016 init are different gauges.

## product consequence

CT-0 v1's measurable product value is the **compiled-geometry link
suggestion at init** (unseen MRR 0.64 vs bigram 0.57, hub queries rank
~1-2). fine-tuning for generalization needs a signal that is not in the
graph's edge list: content features for the 89-91% cold particles (the
known inductive gap), or objectives that exploit content. training on
structure alone optimizes memorization — measured, twice.

## open

- LP objective with harder negatives (share-a-neighbor near-misses),
  multi-epoch, lr sweep: does unseen move past noise?
- freeze-E + train attention: does attention learn to exploit the
  frozen geometry for unseen pairs (the information-ceiling proof says
  no at init; training on walks is memorization — but the combined
  question is not fully closed)?
- content-hash features (IPFS availability is 97.6% on the cybernode)
  for the inductive route.
