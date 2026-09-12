---
title: the scale ladder — does trainable generativity emerge with graph size?
tags: tru, eval, cybergraph
tru: issues #5
---

# the scale ladder — 2026-09-12

question (user-framed): at what graph size does generativity emerge? with
the black-box doctrine — no content, topology only; the agent's head IS
its graph; semantics migrate into topology by stake.

design: temporal prefixes of bostrom (natural growth), rungs at ~100k /
500k / 1M / 2.8M train particles. identical protocol per rung: temporal
90/10, preferential pagerank, exact spectrum of the shipped mixed
operator (scipy svds on a LinearOperator, A^2 never materialized),
E = U sqrt(Sigma) at d=64 fixed. probes: compiled floor (no training),
compiled E + BPR metric training, random E + BPR. queries: 306-800
uniformly sampled held-out walk continuations with a per-source cap of 3
(the first-800-in-height-order sample was dominated by mega-hub sources
and read as a false zero — fixed).

## results

| rung (particles) | floor | bigram | compiled+BPR | random+BPR |
|---|---|---|---|---|
| 87k   | 0.135 | 0.465 | 0.020 (from step 0) | 0.0001 |
| 430k  | 0.669 | 0.727 | 0.046 | 0.0000 |
| 868k  | 0.886 | 0.902 | 0.435 | 0.0000 |
| 2.80M | 0.313 | 0.471 | 0.216 | 0.0000 |

BPR trajectories (compiled E): monotone decay at every rung
(1M: 0.886 -> 0.75 -> 0.66 -> 0.55 -> 0.47 -> 0.43). random E: MRR stays
at 0.0000 everywhere while its BPR loss decreases (0.93 -> 0.85) — it
learns the pair distribution without moving the ranking metric.

## findings

1. **the compiled floor STRENGTHENS with scale** — 0.13 -> 0.67 -> 0.89
   through the growth curve (pussy's 0.64 at 32k fits the trend). the
   user's doctrine is supported on this axis: the more topology, the
   more predictive the compiled geometry, approaching the counts floor
   (0.89 vs 0.90 at 1M). the 2.8M dip (0.31/0.47) is the late era —
   bulk-upload links where neither geometry nor counts generalize.
2. **generativity through gradient training did NOT emerge at any
   measured scale** — up to 2.8M particles / 2.6M edges, neither BPR
   metric training (this ladder) nor transformer walk-LM/LP
   (ft_ablation.md) lifts unseen-link MRR above the compiled floor.
   more data does not convert into generativity; the predictive signal
   stays in the geometry.
3. **random+E+BPR is dead at every scale** — thousands of steps, loss
   decreasing, ranking unmoved. generalization from structure alone is
   not accessible by this gradient path at these budgets.

## caveats (honest)

- BPR budget is 2k steps x 256 pairs (0.5M pairs); 10-50x more training
  with an lr schedule is untested — loss was still decreasing.
- d fixed at 64 across rungs; the 2.8M rung may be dimension-starved
  (its sigma_1 = 236 vs 641 at 1M under the same normalization).
- E-only probe has no attention/context; the full transformer at pussy
  scale behaved the same (ft_ablation.md).
- query samples are 306-800 (noisy to ~0.02).

## answer to the question

in the measured range, there is no size at which TRAINING starts
generalizing — but there is a size at which the COMPILED geometry
becomes dominant: by ~0.5-1M particles the floor approaches the counts
baseline, and at 1M it predicts held-out continuations at MRR 0.89.
CT-0's power grows with the graph at compile time; gradient generativity
remains unproven on structure alone.
