# CT-0 pass 5 eval — init-degeneracy of the compiled attention, and the fix

question: spec §7–10 compiles per-layer attention weights from the graph and
runs a standard llama forward (RMSNorm w=1, RoPE theta 10000, causal,
alpha=1 beta=0, tied lm_head). does the "second half of the transformer"
compose multi-hop semantics at init, or is it a scaffold awaiting training?

harness: `rs/examples/eval_attn.rs` — solar-corpus toy (vocab 30, d=30,
entity holdout URANUS/NEPTUNE), llama forward in f64 with tru's φ*
(preferential dangling fix, tru#1), A Fx-normalized by max weight. probes:
positions where 1 step of context is ambiguous but 2 steps resolve
("X orbits ?" — moon -> its planet, planet -> SUN; "P harbors ?" — P's
moon). query at p predicts token p+1 (standard LM indexing; an earlier
variant scored position t with context <= t and leaked the answer through
the tied head — caught and fixed). NaN discipline: assert finite on all
scores.

## round 1 — the spec'd construction is degenerate at init

| config | amb2 MRR | other MRR | back-mass |
|---|---|---|---|
| bigram add-k (1-step ceiling) | 0.737 | 0.381 | — |
| pass-4 embed dot | 0.764 | 0.350 | — |
| P-projection Q/K + pinv W_O, rope on/off | 0.737 | 0.384 | 0.25–0.59 |
| 1-step-only stack (all l_eff=1), rope on/off | 0.737 | 0.384 | — |

findings, in the order they were isolated:

1. **self-collapse.** with causal softmax over the prefix INCLUSIVE of the
   current position, the self dot-product always wins (Cauchy-Schwarz), and
   W_V -> pinv(W_V) makes the layer ≈ identity. attention never retrieves.
2. **multi-power layers are invisible.** full (l_eff = 1,2,3,4) and
   1-step-only stacks produce identical MRR to 3 decimals — the A^(l_eff)
   family entering through W_Q/W_K never reaches the output.
3. **RoPE is net-negative at init.** back-mass on the disambiguator drops
   0.59 -> 0.25 with rope; self-similarity is rotation-invariant, backward
   matching is not.
4. **branch scale incoherence is load-bearing.** raw counts make the value
   path 10^4x the residual (the model IS a bigram counter — every knob
   saturates and rankings equal bigram exactly); max-normalizing A without
   a score gain collapses softmax to uniform (back-mass = 0.500 exactly).
   the attention/residual mix ratio and the softmax temperature are two
   separate calibrations the spec did not pin.

## round 2 — the retrieval head

weight-only redesign (standard llama forward unchanged, glia-compatible):

- **Q/K from the DIRECTED off-diagonal transition**: S[t,s] = walks s -> t
  at length l_eff, S[t,t] = 0; P_dir = Eᵀ S E; W_Q = U√Σ, W_K = V√Σ.
  The query's own position carries no self-transition mass, so causal
  softmax must retrieve from the prefix; the U/V asymmetry makes scores
  directed (same fix pass 4 needs).
- **W_V = I, W_O = c·I (retrieval head)**: the attention output is the
  gain-scaled retrieved position itself; its full embedding geometry votes
  in the tied-head logits. (the old W_V = Eᵀ diag(φ) A E applies ONE graph
  step — for the probes that points back at the verb, not the answer; and
  pinv(W_V) inverts whatever the value map did.)

measured: back-mass at real probe positions rises to ~1.0 (layer norms:
L0 0.02/0.98, L1–L3 1.00/0.00 backward on the probe sentence).

**the remaining ceiling is pass 4, not pass 5.** cosine-rank by E[TITAN]:
CALLISTO = GANYMEDE = DEIMOS = IO = PHOBOS = 1.000 — structurally
equivalent tokens (every moon links only to "orbits"/"harbors" with equal
counts) get parallel embeddings at any rank: the which-planet signal lives
at graph distance 2 and E = f(1-step M) does not carry it at fine
granularity. attention cannot retrieve what E does not distinguish; the
fine-grained amb2 probes therefore saturate at the embed baseline (0.764,
class-level signal: "a planet", not "SATURN"). eval_ct0's gen MRR 1.0 was
the same physics at coarser granularity (class-level distinctions survive;
sibling-level ones do not).

## what shipped in tru (this session)

`rs/pass/attn.rs` (spec §7.2–7.5 revised, evidence linked from §7.3):

- `project` now builds P_dir = Eᵀ (A^(s,l))ᵀ E via transposed sparse
  application (`apply_t`); Q/K from its SVD (U vs V — directed).
- retrieval head: W_V = I, W_O = OUT_GAIN · I (pinv removed — it made the
  block a no-op).
- OUT_GAIN is a named pub constant (= 1), flagged in §7.5 as the open
  calibration: c must let the retrieved token outvote the last token's
  frequency prior (toy bound c ≳ 13 against a ~25x prior); a graph-derived
  rule for c is open.
- tests: P-ATTN vs the directed projection, QKᵀ asymmetry on a directed
  fixture, W_V = I / W_O = c·I invariants, determinism.
- fixed a flaky compile test (fixed temp-file path shared across parallel
  tests; observed "no `~~~` delimiter found" ~1/3 of runs).

## round 3 — the shipped pipeline (2-hop E + retrieval head + gain rule)

re-running the probes with the shipped construction (M <- diag(√φ)(A +
½A²)diag(√φ), spec §6.1-rev):

| | amb2 MRR | other MRR |
|---|---|---|
| bigram add-k | 0.737 | 0.381 |
| pass-4 embed (mixed) | 0.724 | 0.486 |
| CT-0 (any attention config) | 0.716–0.724 | **0.543** |

1. **first measured win over the 1-step ceiling**: CT-0 other-MRR 0.543 >
   bigram 0.381 — mixed E alone gives 0.486, attention adds +0.06 on top.
   the compiled stack now predicts unambiguous next tokens better than raw
   counts.
2. **amb2 stays at the embed floor** — the toy corpus is REGULAR by
   construction (uniform rng counts): every moon has identical A rows, and
   identical A² rows (same multiset of 2-walks), so no mixing power splits
   the moons here. the margin ratios DID improve (TITAN·SATURN/TITAN·SUN
   1.5x -> 3.4x; prior 25x -> 16x) — on an irregular graph (pussy) the same
   construction lifts directed LP AUC 0.634 -> 0.742 (README table).
3. the toy cannot arbitrate fine-grained init claims anymore; pussy
   end-to-end (walk-generated sequences over learned E) is the next
   decisive experiment.

## shipped follow-ups (this round)

1. **2-hop mixing (tru#2)**: `M <- diag(√φ)(A + ½A²)diag(√φ)` in
   `rs/pass/arch.rs`; validated on pussy (see README table: directed AUC
   0.634 -> 0.742 temporal, novel-AUC 0.544 -> 0.687). The steeper mixed
   spectrum exposed fixed-point loss of orthogonality in the SVD spine —
   phantom σ=1.125 on a rank-3 matrix, reconstruction rel err 0.86; cured
   by reorthogonalizing twice per subspace iteration (`rs/pass/svd.rs`).
2. **OUT_GAIN rule (tru#3)**: `c = clamp(1 + log2(σ₁/σ_k), 1, 64)` from
   `Arch::sigma_ratio`; reported in the compile certificate.
3. **RoPE at init (tru#4)**: config ships `rope_theta = 1e6`
   (near-identity rotation), spec §9.2 documents the measured regression
   and the certificate check.

end-to-end pussy validation of the full forward (sequences over learned E
via graph walks) is still open; the toy ceiling above says the embedding
limit must be lifted before the attention gain shows up in fine-grained
probes — the 2-hop mixing is exactly that lift, so the probe should be
re-run on the toy with the shipped pipeline.

## still open (superseded)

tracked as issues: [#2](https://github.com/cyberia-to/tru/issues/2) (pass 4
fine structure), [#3](https://github.com/cyberia-to/tru/issues/3) (OUT_GAIN
calibration), [#4](https://github.com/cyberia-to/tru/issues/4) (RoPE at init).

1. **gain rule for c** — the one knob that decides whether retrieval
   outvotes frequency priors; needs a graph-derived formula (candidate:
   from the observed prior margin σ-ratio at compile time), validated on
   pussy. [#3]
2. **pass 4 fine structure** — sibling tokens get parallel embeddings;
   candidates: mix 2-hop mass into M (M <- normed(A + γA²)), or rank
   selection that keeps the low-σ components carrying fine 2-hop signal
   (pussy peaked at k=16; the 64-floor discards them). [#2]
3. **RoPE at init** — net-negative for compiled weights; spec could ship a
   larger theta or make rotation optional until fine-tuned. glia-side
   config, not a tru weight. [#4]
4. **end-to-end validation on pussy** — needs sequences over learned E
   (pussy is a link graph, no corpus; generate sequences by graph walks);
   the toy ceiling above says the embedding limit must be lifted before
   the attention gain shows up in fine-grained probes.

## glia note

the program is standard llama architecture — the compiled artifact targets
the glia runtime (`run/ir`: RmsNorm{eps:1e-5}, Rope{base:10000}, Sdpa,
gguf import); this eval cross-checks the same forward semantics locally in
f64. the findings are about weights and architecture, not numerics; no
glia-specific divergence was exercised or observed.
