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
| [vocab.md](vocab.md) | `.vocab` particle dictionary — content-addressed particle → bytes | `Vocab` lookup | ⬜ spec only | 0a |
| [model.md](model.md) | `.model` container — the inference-ready artifact, mmap-able weights | `.model` file | 🟡 writer scaffold (`unimplemented!`) | 0b, 2g |

## focusing layer — computing φ*

the heart of tru. five specs that turn the weighted graph into the focus distribution φ* and its derived quantities. dependency order within the layer: `tri-kernel` (operators) → `attention` (per-neuron input) → `truth-scoring` (how stake/karma weight the graph) → `focusing` (assembles the epoch) → `impulse` (the per-signal delta).

| spec | defines | produces | status | step |
|------|---------|----------|--------|------|
| [tri-kernel.md](tri-kernel.md) | the three operators (diffusion D, springs S, heat H_τ), composite R, fixed-point + locality proofs, §2.4 five-way identity | φ* = fix(R) | 📐 spec complete; `rs/focusing/` stub is non-conformant (averaging form + `f64`, both rewritten at M1) | 1a |
| [attention.md](attention.md) | per-neuron focus projection — will-share + conviction box that sums into effective adjacency | A^eff summand | ⬜ spec only | 1b |
| [truth-scoring.md](truth-scoring.md) | BTS mechanism, karma accumulation, honesty-weighted effective adjacency | κ(ν), A^eff | ⬜ spec only | 1b |
| [focusing.md](focusing.md) | epoch computation: effective adjacency → tri-kernel → φ*, cyberank, syntropy | φ*, cyberank, syntropy | 🟡 φ* only — cyberank/syntropy missing | 1c |
| [impulse.md](impulse.md) | Δφ* — the proven focus shift one signal delivers; locality-bounded sparse vector | Δφ* + proof claim | ⬜ spec only | 1c |
| [superadditivity.md](superadditivity.md) | the collective-intelligence measure σ (collective φ* vs ego φ*_ν); generalized CFT — σ, J grow with algebraic connectivity λ₂ | σ_mean, σ_best, J(λ₂) | 📐 spec; benchmark to run | val |

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

**settled — how φ\* is computed (tri-kernel §2.4, focusing.md):** φ\* is the fixed point of *one coupled iteration* — apply D, S, H_τ to the same current φ, blend, normalize, repeat. tru does **not** solve the three operators to their own fixed points and average (that minimizes no single free energy, has no single κ, and breaks the five-way identity). this is now explicit in the spec; no decision pending. the `rs/focusing/` stub (ported from optica) currently does the averaging form in `f64` — non-conformant on both axes (averaging, and float where [[arithmetic]] requires fixed-point over the Goldilocks field), rewritten together at M1. not a concern now: we are specifying, not building.

## compile layer — φ* → transformer

| spec | defines | produces | status | step |
|------|---------|----------|--------|------|
| [focus-flow.md](../docs/explanation/focus-flow.md) | the identity between continuous focusing (path A) and compiled transformer inference (path B); architecture derivation | — (the why) | 📐 reference | — |
| [ct0.md](ct0.md) | the CT-0 pipeline — 8 passes from `.graph` to `.model`; multivector inputs §2.5–2.6, wedge attention §7.7, Clifford MLP §8 | `.model` weights | ⬜ spec only | 2a–2g |

ct0 is the largest spec (738 lines) and the bulk of remaining work. its passes map directly to steps 2a–2g:

| pass | ct0 § | builds | step |
|------|-------|--------|------|
| 1–2 | §3–4 | particle index, dialect set | 2a |
| 3 | §5 | architecture params d*, h*, L* | 2b |
| 4 | §6 | embedding matrix E | 2c |
| 5 | §7.1–7.6 | attention weights W_Q/K/V/O | 2d |
| 5+ | §7.7 | wedge-augmented score (Clifford) | 2e |
| 6 | §8 | Clifford-block MLP | 2f |
| 7–8 | §9–10 | norms, RoPE, `.model` packaging | 2g |

## economics layer — the reason

this is the point. tru is not a ranking engine that happens to have rewards bolted on — it is a minting engine whose unit of account is proven focus shift. focusing, the compile, the proof: all of it exists so a neuron can convert Δφ* into [[$CYB]] with no aggregator deciding who contributed what. `impulse.md` defines the quantity; `rewards.md` defines the conversion.

| spec | defines | status |
|------|---------|--------|
| [rewards.md](rewards.md) | surprising-syntropy self-minting, Shapley attribution + settlement mining, the three streams (mint/subsidy/fee), supply/allocation, timing & accrual | 📐 spec complete — the destination; settlement spans [[foculus]]/[[tok]] |

---

## what's left to build (summary)

| | spec | done |
|---|------|------|
| 📐 | tri-kernel | spec complete (§2.4 added); stub is non-conformant, rewrite at implementation |
| 🟡 | focusing | φ* computed; **need** cyberank, syntropy |
| 🟡 | model | writer scaffold; **need** real serialize/load |
| ⬜ | vocab | parser |
| ⬜ | attention + truth-scoring | will/conviction input, BTS → karma, effective adjacency |
| ⬜ | impulse | Δφ* delta computation |
| ⬜ | ct0 | all 8 passes (2a–2g) — the bulk of the work |
| 📐 | rewards | spec complete — the telos; cross-layer settlement in foculus/tok |

**built: 1 of 8 active specs. the critical path is** `tri-kernel reconcile → focusing (cyberank, syntropy) → ct0 passes`, with `vocab`/`model` formats needed before pass 1 and after pass 8, and `rewards` now spec-complete — the reason the pipeline exists, its settlement spanning foculus/tok.

see the [implementation steps table](../README.md#implementation-steps) in the repo readme for the step-by-step build order with verifiable predicates, and [roadmap/implementation.md](../roadmap/implementation.md) for the full milestone plan — module layout, per-spec algorithm, and cross-repo blockers.
