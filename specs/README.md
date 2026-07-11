---
tags: cyber, tru, core, spec
alias: tru specs, tru spec map, what to build
---
# tru specs

the build map for [[tru]] — the convergence vm. one pipeline, `.graph` → φ* → Δφ* → reward, specified across four layers. focusing computes φ*; compilation freezes it into a model; **economics is why any of it runs** — the proven focus shift Δφ* is what a neuron self-mints against. this index says what each spec defines, what it produces, what depends on it, and whether it is built yet.

## the pipeline

```
                      .graph  (from cybergraph)
                        │
        ┌───────────────┴───────────────┐
        │                               │
   FORMAT layer                 FOCUSING layer
   ┌──────────┐              ┌─────────────────────┐
   │  vocab   │              │  tri-kernel  (D S H)│
   │  model   │              │  attention   (input)│
   └────┬─────┘              │  truth-scoring (κ,A)│
        │                    │  focusing → φ*, rank│
        │                    │  impulse → Δφ*      │
        │                    └──────────┬──────────┘
        │                               │ φ*
        │         COMPILE layer         │
        │       ┌──────────────────┐    │
        └──────►│ focus-flow  (why) │◄───┘
                │ ct0   (8 passes)  │
                └────────┬──────────┘
                         │
                      .model  (to glia)

   the telos ──────────────────────────────────────────┐
   ECONOMICS layer   rewards :  Δφ* (impulse) → $CYB    │  ◄── the reason
   self-mint against proven focus shift. no aggregator. │
   ───────────────────────────────────────────────────-┘
```

## status

- ✅ **built** — code exists, tests pass
- 🟡 **partial** — core built, pieces missing
- ⬜ **spec only** — no code yet
- 📐 **reference** — explains why; no code artifact of its own
- 🔜 **spec incomplete** — central, but the spec itself needs finishing before code

---

## format layer — the containers

the two on-disk formats. prerequisites for everything: vocab feeds pass 1, model is the output of pass 8.

| spec | defines | produces | status | step |
|------|---------|----------|--------|------|
| [vocab.md](vocab.md) | `.vocab` particle dictionary — content-addressed particle → bytes | `Vocab` lookup | ✅ parser + writer (M2, content-addressed, 5 tests) | 0a |
| [model.md](model.md) | `.model` container — the inference-ready artifact, mmap-able weights | `.model` file | ✅ container writer/reader (M2, page-aligned weights, P-DET) | 0b |

## focusing layer — computing φ*

the heart of tru. five specs that turn the weighted graph into the focus distribution φ* and its derived quantities. dependency order within the layer: `tri-kernel` (operators) → `attention` (per-neuron input) → `truth-scoring` (how stake/karma weight the graph) → `focusing` (assembles the epoch) → `impulse` (the per-signal delta).

| spec | defines | produces | status | step |
|------|---------|----------|--------|------|
| [tri-kernel.md](tri-kernel.md) | the three operators (diffusion D, springs S, heat H_τ), composite R, fixed-point + locality proofs, §2.4 five-way identity | φ* = fix(R) | ✅ conformant — coupled iteration in fixed-point `Fx`; heat = Chebyshev (`rs/focusing/`) | 1a |
| [attention.md](attention.md) | per-neuron focus projection — will-share + conviction box that sums into effective adjacency | A^eff summand | ✅ will (broad) + conviction (box) → A^eff, `Context{karma,will}` | 1b |
| [truth-scoring.md](truth-scoring.md) | BTS mechanism, karma accumulation, honesty-weighted effective adjacency | κ(ν), A^eff | ✅ `rs/truth_scoring.rs` — BTS score, karma accrual, surprise ρ (6 tests) | 1b |
| [focusing.md](focusing.md) | epoch computation: effective adjacency → tri-kernel → φ*, cyberank, syntropy | φ*, cyberank, syntropy | ✅ φ*, cyberank, syntropy, entropy, spectral positions, Δφ* (deterministic) | 1c |
| [impulse.md](impulse.md) | Δφ* — the proven focus shift one signal delivers; locality-bounded sparse vector | Δφ* + proof claim | ✅ `rs/focusing/impulse.rs` — Δφ*, Δφ⁺, ΔJ decomposition (proof σ external) | 1c |
| [superadditivity.md](superadditivity.md) | the collective-intelligence measure σ (collective φ* vs ego φ*_ν); generalized CFT — σ, J grow with algebraic connectivity λ₂ | σ_mean, σ_best, J(λ₂) | ✅ benchmark harness `rs/examples/superadditivity.rs` (Karate Club) | val |

### vocabulary — the terms tru owns

tru is a subgraph; every concept it owns is defined here, not scattered across the graph. these are definition pages, not build steps — they pin the meaning the specs above and the code below both rely on.

| term | is | owned because |
|------|----|----|
| [focus.md](../docs/terms/focus.md) | φ*, the collective attention distribution | tru computes it; the single most-referenced term |
| [cyberank.md](../docs/terms/cyberank.md) | focus per particle, φ*(p) — the canonical ordering | a named output other repos read |
| [syntropy.md](../docs/terms/syntropy.md) | network order in bits, J(φ*) — **the purpose** | the quantity the whole pipeline grows |
| [convergence.md](../docs/terms/convergence.md) | iteration toward a self-defined attractor — tru's execution model | "tru = convergence" |
| [valence.md](../docs/terms/valence.md) | the ternary epistemic field v ∈ {−1,0,+1} | cybergraph carries the field; tru runs the dynamics |
| [will.md](../docs/terms/will.md) | locked balance → the broad budget for attention | the input quantity focusing reads |
| [conviction.md](../docs/terms/conviction.md) | per-link economic commitment, the box (τ,a) on one edge | the per-link counterpart of will; box magnitude in A^eff |
| [axon.md](../docs/terms/axon.md) | the bundle of all cyberlinks on a pair, itself a particle | cybergraph is the umbrella; tru defines the weighting |
| [arithmetic.md](arithmetic.md) | fixed-point over the Goldilocks field — no floats, deterministic T(ε) steps | the representation contract every spec and the code inherit |

the [[collective focus theorem]] (convergence + uniqueness of φ*) is `tri-kernel.md §3` (normative) and [docs/collective-focus-theorem.md](../docs/explanation/collective-focus-theorem.md) (the standalone paper).

**settled — how φ\* is computed (tri-kernel §2.4, focusing.md):** φ\* is the fixed point of *one coupled iteration* — apply D, S, H_τ to the same current φ, blend, normalize, repeat. tru does **not** solve the three operators to their own fixed points and average (that minimizes no single free energy, has no single κ, and breaks the five-way identity). this is now explicit in the spec; no decision pending. `rs/focusing/` implements exactly this (M1): one coupled iteration in fixed-point `Fx` over the Goldilocks field, stake-weighted A_eff, single-step operators — the old averaging-in-`f64` form is gone, and φ* is bit-identical across runs.

## compile layer — φ* → transformer

| spec | defines | produces | status | step |
|------|---------|----------|--------|------|
| [focus-flow.md](../docs/explanation/focus-flow.md) | the identity between continuous focusing (path A) and compiled transformer inference (path B); architecture derivation | — (the why) | 📐 reference | — |
| [ct0.md](ct0.md) | the CT-0 pipeline — 8 passes from `.graph` to `.model`; multivector inputs §2.5–2.6, wedge attention §7.7, Clifford MLP §8 | `.model` weights | 🟡 all 8 passes built + `tru compile`; deterministic (P-DET) — refinements below | 2a–2g |

ct0 is the largest spec (738 lines). all 8 passes are implemented (`rs/pass/`), the CLI compiles `.graph` → `.model`, and two runs are byte-identical:

| pass | ct0 § | builds | code | status |
|------|-------|--------|------|--------|
| 1–2 | §3–4 | particle index, dialect set | `index.rs`, `dialect.rs` | ✅ |
| 3 | §5 | architecture params d*, h*, L* | `arch.rs` | ✅ (φ*, d* via SVD effective-rank, λ₂, κ, diameter) |
| 4 | §6 | embedding matrix E | `embed.rs` | ✅ (E = U√Σ, shared `svd.rs`) |
| 5 | §7.1–7.6 | attention weights W_Q/K/V/O | `attn.rs` | ✅ (per-head SVD, pinv output) |
| 5+ | §7.7 | wedge score scalars (α,β) | `attn.rs` | ✅ (α,β)=(1,0); wedge *op* is inference-time |
| 6 | §8 | Clifford-block MLP | `mlp.rs` | ✅ (seeded init; Clifford *op* is inference-time) |
| 7–8 | §9–10 | norms, RoPE, `.model` packaging | `norm.rs`, `compile.rs` | ✅ |

**deferred refinements (none block the pipeline):** `config.tokens` ρ_τ (defaults to 1) and vocab-ref seed loading are unwired; SVD is exact subspace iteration, not the randomized+ChaCha form (§6.2) — correctness-equivalent and deterministic, but cross-implementation byte-identity needs the exact ChaCha seeding; impulse reuse (§5.1) is unimplemented; the scale path (randomized SVD / GPU for d=300, L=290) is the open frontier. conformance: P-DET ✅, P-EMBED ✅ (PSD caveat), P-ATTN/P-LAYER not yet asserted, P-LOAD/P-CLIFFORD need the cyb-llm runtime.

## economics layer — the reason

this is the point. tru is not a ranking engine that happens to have rewards bolted on — it is a minting engine whose unit of account is proven focus shift. focusing, the compile, the proof: all of it exists so a neuron can convert Δφ* into [[$CYB]] with no aggregator deciding who contributed what. `impulse.md` defines the quantity; `rewards.md` defines the conversion.

| spec | defines | status |
|------|---------|--------|
| [rewards.md](rewards.md) | surprising-syntropy self-minting, Shapley attribution + settlement mining, the three streams (mint/subsidy/fee), supply/allocation, timing & accrual | ✅ tru layer built — `rs/rewards.rs`: value `v★(S)=Δφ⁺(A^eff∪ρ·S)`, Shapley (3 axioms tested), settlement ordering. lottery→[[foculus]], conservation/mint→[[tok]] |

---

## what's built (summary)

every build spec is implemented. the full pipeline `.graph → φ* → Δφ⁺ → v★ → Shapley` and `.graph → .model` both run end-to-end in fixed-point, deterministic.

| | spec | done |
|---|------|------|
| ✅ | arithmetic | fixed-point `Fx` over Goldilocks; sqrt/exp/ln/rescale; T(ε) (11 tests) |
| ✅ | vocab | parser + writer, content-addressed (5 tests) |
| ✅ | model | container writer/reader, page-aligned, round-trips (4 tests) |
| ✅ | tri-kernel | coupled iteration, fixed-point, heat=Chebyshev, deterministic |
| ✅ | focusing | φ*, cyberank, syntropy, entropy, spectral positions, Δφ* |
| ✅ | attention | will (broad) + conviction (box) → A^eff |
| ✅ | truth-scoring | BTS score → karma, surprise ρ (6 tests) |
| ✅ | impulse | Δφ*, Δφ⁺, ΔJ decomposition (5 tests) |
| ✅ | superadditivity | σ benchmark harness (Karate Club) |
| 🟡 | ct0 | all 8 passes + `tru compile`, deterministic; refinements deferred (see compile layer) |
| ✅ | rewards | tru layer: value, v★, Shapley (3 axioms); lottery→foculus, mint→tok |

**~90 tests green, warning-clean, no stubs.** the whole intelligence layer (focusing → φ*), compile layer (`.graph → .model`), and economics magnitude layer (Δφ⁺ → surprise → Shapley) are conformant code. what remains is not new specs but scale (randomized SVD / GPU for production-size graphs), the small ct0 wirings noted above, and the cross-repo settlement plumbing that belongs in [[foculus]] and [[tok]] by design.

see the [implementation steps table](../#implementation-steps) in the repo readme, and [roadmap/implementation.md](../roadmap/implementation.md) for the milestone plan and cross-repo blockers.
