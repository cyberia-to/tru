---
title: link prediction at scale — bostrom
tags: tru, eval, cybergraph
tru: issues #2 #3
---

# link prediction at scale — bostrom

2026-09-11. the pussy findings (lp_eval.md conclusions) re-measured on the
full bostrom graph: 2,949,732 cyberlinks / 3,143,650 particles at halt
(height 25,120,712, 2026-08-05). source: the canonical snapshot rebuild
(counts bit-exact to on-chain graph_stats; cyberlinks_indexed.csv.gz,
sha256 e442ff7a0e905591f1bda94dcac89fd350c28824ccc71b56992da1ae0555e4ae).

command: `python3 lp_eval.py data/bostrom_links.jsonl --k 16` (and
`--random-split`). k=16, the pussy spectral peak.

## results

temporal split (90/10 by height):

| model | AUC | AP | novel-AUC |
|-------|-----|-----|-----------|
| pa (phi_i*phi_j) | 0.9426 | 0.8069 | 0.9423 |
| tru-E (sym, 1-step) | 0.5207 | 0.2217 | 0.5199 |
| directed (U,V, 1-step) | 0.5004 | 0.3630 | 0.4976 |
| ppmi (U,V) | 0.6847 | 0.5340 | 0.6827 |
| tru-E2 g=0.25 (sym) | 0.5511 | 0.2577 | 0.5491 |
| directed2 g=0.25 | 0.6166 | 0.4899 | 0.6144 |
| directed2 g=0.5 | 0.6086 | 0.4817 | 0.6066 |
| directed2 g=1.0 | 0.6107 | 0.4861 | 0.6086 |
| adamic-adar (sub) | 0.5012 | 0.1253 | — |

random split: pa 0.8545 · tru-E 0.5826 · directed 0.6190 · ppmi 0.7136 ·
directed2 g=0.25 0.6895 · directed2 g=0.5 0.6876.

scorable test pairs: temporal 26,843 / 294,974 (9.1% — 91% of new links
touch a particle the train graph has never seen; pussy was 89% novel but a
far higher share stayed in-vocab). random 82,315 / 294,974 (27.9%).

## what scales and what changes

1. **the popularity prior strengthens with scale.** pa temporal AUC
   pussy 0.866 -> bostrom 0.943. the gap spectral-vs-pa widens from 0.115
   (pussy directed2@0.25) to 0.326 (bostrom). raw-count baselines are
   *better* on the bigger graph — the prior is the signal at scale, and
   it is exactly what tru#3 OUT_GAIN is for.

2. **2-hop mixing replicates.** 1-step directed is dead at chance
   (0.5004); directed2 g=0.25 lifts it to 0.6166 temporal / 0.6895
   random. g=0.25 edges out g=0.5 by <0.01 AUC on both splits — within
   noise; gamma = 1/2 stays the shipped default (split-robust choice from
   the pussy sweep, not re-tuned per graph).

3. **the out-gain rule scales with the corpus.** on the shipped mixed
   matrix (A + 0.5 A^2, k=16): bostrom sigma_1/sigma_k = 8885.6 ->
   out_gain = 1 + log2(8885.6) = 14.1 (clamped range [1,64] holds).
   larger corpus -> sharper popularity prior -> louder retrieval vote,
   as designed. Arch::sigma_ratio reads exactly this spectrum.

## popularity-bucket autopsy (temporal, directed2 g=0.25, k=16)

test pairs ranked by pa score, AUC within decile vs global negatives:

| decile | n | AUC_dir2 | AUC_sym |
|--------|-----|----------|---------|
| 0-10% (coldest) | 2684 | 0.5367 | 0.5029 |
| 40-50% | 2684 | 0.6539 | 0.5649 |
| 50-60% | 2684 | 0.8292 | 0.6533 |
| 60-70% | 2684 | 0.6211 | 0.5695 |
| 90-100% (hottest, mean pa 3e-8) | 2685 | 0.6546 | 0.5643 |

spectral signal peaks in the mid-popularity band and survives — weaker —
at both extremes. the top decile is 5 orders of magnitude above the rest
in pa mass (hub-to-hub links, e.g. the whole-corpus Wikipedia particle);
there the prior alone decides and spectral adds little *ranked against
global negatives*. this is the in-model justification for keeping the
popularity prior in the residual (it is free, via E geometry) while
OUT_GAIN amplifies retrieval for the middle band where structure, not
fame, selects the target.

## artifacts

- `pa+ppmi(z)` additive hybrid collapses to AUC 0.0903 on the random
  split (fine temporal: 0.9510). z-scoring a heavy-tailed pa score
  produces outlier scores that dominate the sum; the multiplicative
  hybrid pa*ppmi is stable (0.684/0.715). do not use additive z-score
  hybrids on this score distribution.
- ppmi (directed factorization) remains the strongest spectral single
  model (0.6847/0.7136) but is a count-reweighting, not a geometry; it
  has no in-model role in CT-0 (E comes from the pass-4 spectrum).

## open

- end-to-end attention-5 forward on real walks (pussy first — 32k
  particles fit the toy forward; bostrom scale needs the glia runtime).
- the 91% cold-endpoint share: content-addressable novel particles carry
  no in-graph signal; the inductivity gap stays open (tru roadmap).
