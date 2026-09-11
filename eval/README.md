# CT-0 eval — link prediction on live cybergraphs

the measurable half of "compile a transformer without training": can the
compiled geometry predict **new links** in a graph that real users built?

## data

- **space-pussy** (live testnet): 48,370 cyberlinks / 30,017 train particles,
  fetched from the node tx index (`fetch_links.py` over
  `tx_search "cyberlink.neuron EXISTS"`, attributes base64-decoded). heights
  recorded per link — temporal split is real.
- **bostrom** (halted mainnet, snapshot at height 25,120,712): 2,949,732
  cyberlinks / 3,143,650 particles, rebuilt bit-exact to the on-chain
  graph_stats by a full-history block scan (see the bostrom snapshot
  manifest: `cyberlinks_indexed.csv.gz`, IPFS
  `QmUFrsLYUK8USpNMEGBbUyi3nTuq12QcLiVWiyVMMVyyFf`, sha256
  `e442ff7a…e4ae`). too big for git — fetch from
  `deimos:/archive/snapshot/pub/cyberlinks_indexed.csv.gz` (or any IPFS
  gateway) and convert to jsonl; see `data/.gitignore`. results:
  [lp_bostrom.md](lp_bostrom.md).

## setup

```bash
python3 -m venv .venv && .venv/bin/pip install scipy numpy
.venv/bin/python fetch_links.py https://rpc.space-pussy.cybernode.ai data/pussy_links.jsonl
.venv/bin/python lp_eval.py data/pussy_links.jsonl --k 64
```

## task

temporal split: first 90% of links by height → train; last 10% → test.
negatives: same-vocab pairs that are non-edges in the **full** link set.
φ* is computed with the preferential dangling fix (cyberia-to/tru#1).

coverage caveat: only 519 / 4,837 test links are scorable — 89% of future
links touch particles that did not exist in train. the scorable subset is
therefore popularity-biased by construction; the numbers below describe
"which *old* nodes get linked next", not "what new content arrives".

## results — space-pussy, temporal split (n=519, novel n=372)

| model | AUC | AP | novel-AUC |
|---|---|---|---|
| **pa (φ_i·φ_j)** | **0.866** | 0.562 | **0.864** |
| ppmi directed, k=16 | 0.725 | 0.560 | 0.657 |
| ppmi directed, k=64 | 0.668 | 0.535 | 0.587 |
| ppmi directed, k=256 | 0.634 | 0.505 | 0.533 |
| directed (U,V), k=16 | 0.639 | 0.483 | 0.555 |
| tru-E (sym), any k | 0.52–0.56 | ~0.30 | ~0.53 |
| adamic-adar (sub) | 0.535 | 0.192 | — |

random-split control (k=64, n=3,272): directed 0.797 and ppmi 0.794 **beat**
pa 0.752 — spectral structure is real but transductive: it ranks links among
known nodes, not arrivals of new links.

## update (tru#2) — 2-hop mixing fixes the fine-structure floor

with `M <- diag(√φ)(A + 0.5·A²)diag(√φ)` (lp_eval.py, `--k 16` and 64):

| model | temporal AUC | random AUC | novel-AUC |
|---|---|---|---|
| directed (1-step) | 0.634 | 0.777 | 0.544 |
| **directed2 γ=0.5 (shipped)** | **0.742** | **0.785** | **0.687** |
| directed2 γ=0.25 | 0.751 | 0.771 | 0.689 |
| directed2 γ=1.0 | 0.745 | 0.821 | 0.693 |

the sibling-signal gap closes by ~0.11 AUC temporal / ~0.15 novel-AUC; γ=0.5
is the split-robust default. shipped in `rs/pass/arch.rs` (hop2_mix) +
spec §6.1-rev.

## what this says

1. **the live graph grows by preferential attachment.** popularity (φ*) is the
   single strongest predictor of future links on a temporal split. no spectral
   variant beats it; hybrids (`pa*ppmi`, `pa+ppmi(z)`) do not close the gap.
2. **direction-blindness costs on real data too.** tru-E (current pass 4,
   symmetric U√Σ) sits at the bottom at every k. the V-side fix is confirmed
   on live data, not only the toy: directed factorization beats it by
   ~0.10–0.14 AUC.
3. **small rank generalizes, large rank memorizes.** ppmi peaks at k=16 and
   degrades monotonically with more components (σ₆₄ = 0.0006 — dead
   dimensions). tru's d* = effective-rank heuristic is load-bearing; the 64
   floor may already be too high for this graph.
4. **89% of new links are unscorable** — embeddings cannot rank particles they
   have never seen. for the compile to matter for the real graph, pass 4 needs
   an inductive route to unseen particles (content-hash features, not only
   structural ids) — or the honest claim narrows to "ranks among known
   particles".

## pass 4 at scale — bostrom

see [lp_bostrom.md](lp_bostrom.md) — the pussy findings re-measured on the
full halted graph. headline: the popularity prior strengthens with scale
(pa temporal AUC 0.866 -> 0.943), 2-hop mixing replicates (1-step directed
at chance 0.5004 -> directed2 g=0.25 0.6166), and the tru#3 out-gain rule
scales: sigma_1/sigma_k = 8885.6 on the shipped mixed matrix -> gain 14.1.

## pass 5: the attention half

see [attn_eval.md](attn_eval.md) — llama forward over the compiled per-layer
weights. findings: (1) the spec'd construction self-collapses at init
(softmax self-wins, pinv(W_V) ≈ identity, multi-power layers invisible,
RoPE net-negative); (2) the weight-only fix shipped in `rs/pass/attn.rs`
— Q/K from the DIRECTED projection Eᵀ(Aˡ)ᵀE (self-transition killed, U≠V),
retrieval head W_V = I, W_O = c·I; (3) the fine-grained ceiling is pass 4:
structurally equivalent tokens get parallel embeddings, 2-hop sibling
signal lives in low-σ components that the rank heuristic discards.

## reproduction notes

- tx_search `order_by` 500s on the public node; heights are re-sorted locally.
- the earlier corpus-toy result (all variants MRR 1.000) was a **NaN artifact**:
  unclamped Rayleigh quotients in the toy's f64 SVD made every score
  comparison false → rank 1 everywhere. fixed by clamping; tru's own
  fixed-point SVD already clamps (`svd.rs`). lesson recorded: assert finite
  on all scores in an eval harness.
