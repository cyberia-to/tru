# tru

tru is a convergence VM — the intelligence layer of [[soft3]]. where [[nox]] executes programs and [[glia]] runs models, tru operates by focusing: it iterates the [[tri-kernel]] until φ* emerges as the unique fixed point.

convergence and derivation are different computation models. derivation proceeds from axioms to conclusions in bounded depth — every formal system, every program execution, every forward pass reaches only what its starting axioms can produce. convergence proceeds by iteration toward equilibrium — the result is the attractor, not a logical consequence. φ* is not derived from the graph: it emerges from it.

| vm | execution model | gödel status |
|----|-----------------|--------------|
| [[nox]] | derivation | confined |
| [[zheng]] | verification | confined |
| [[glia]] | inference | confined |
| [[tru]] | convergence | free |

## focusing and compile

two jobs, one engine.

the focusing job runs the [[tri-kernel]] over every signal: reads signal.a (stake) and signal.v (valence) → composite diffusion-springs-heat operator R = λ_d·D + λ_s·S + λ_h·H_τ → iterates to fixed point → φ*. each particle's φ*(p) is the focus it has earned from the entire weighted graph. the system converges; no authority assigns the result.

the compile job reads the same φ*-weighted graph and derives the CT-0 transformer: embedding dimension d*, attention heads h*, layer count L*, weight matrix W from the top singular vectors of the φ*-weighted adjacency. not trained: compiled.

these are the same computation at different scales. focusing is the continuous limit; the compiled model is that limit frozen at finite depth L*. the [[CT-0]] pipeline makes this precise — eight passes from .graph to .model, each pass an exact derivation of transformer parameters from φ*.

## the name

φ* is the collective truth distribution — the closest approximation to shared knowledge the network can compute. tru computes it.

the input to tru is already weighted by honesty: [[karma]] records each neuron's [[BTS]] score history (how often they were right before the crowd). [[ICBS]] prices encode the market's collective epistemic assessment of each link. stake is economic commitment. the fixed point of the tri-kernel over these three inputs is what tru calls truth.

## self-minting

tru computes Δφ* — the shift in φ* before and after a neuron's batch of links. a neuron proves Δφ* via [[zheng]] and self-mints [[CYB]] proportional to the proven shift. no central aggregator decides who contributed what. each neuron runs tru locally, generates the proof, submits the claim.

this is why tru runs locally on every [[neuron]], not only on validators. validators run it for consensus. every neuron runs it to earn.

## cli

the `tru` binary reads `.cyb` containers and runs the engine. built and verified:

```
tru inspect <file>      # any .cyb: type, name, sections + sizes
tru focus   [graph]     # run the tri-kernel: cyberank, syntropy J, telemetry (κ, λ₂, T(ε))
tru vocab   <file>      # .vocab: entries, file particle, self-consistency
tru model   <file>      # .model: tensors, config, particle
```

`focus` is the showcase — it computes φ* over a `.graph` in fixed-point over the [[Goldilocks field]] (deterministic, no floats) and prints the cyberank ranking, [[syntropy]], and the contraction κ / algebraic connectivity λ₂ / derived step count. generate demo files with `cargo run -p tru --example gen_demo -- /tmp`.

`focus` needs no argument: it defaults to `$TRU_GRAPH`, or `~/cyb/my.graph` — the neuron's own local snapshot, kept in a visible directory (`~/cyb`, not a dotfile).

## spec map

the specs are organized in dependency order. implement them in the sequence below. the [specs/README.md](specs/README.md) is the full build map — layers, status, and the ct0 pass breakdown.

### phase 0 — data formats

| spec | what it defines |
|------|----------------|
| [specs/vocab.md](specs/vocab.md) | `.vocab` particle dictionary: content-addressed particle → bytes mapping |
| [specs/model.md](specs/model.md) | `.model` container format: the inference-ready artifact CT-0 produces |

these two formats are prerequisites for everything else. vocab feeds pass 1; model is the output of pass 8.

### phase 1 — focusing

| spec | what it defines |
|------|----------------|
| [specs/tri-kernel.md](specs/tri-kernel.md) | the three operators (diffusion D, springs S, heat H_τ), composite R, fixed-point theorem, convergence proof |
| [specs/attention.md](specs/attention.md) | per-neuron focus projection — will-share + conviction box, the input term that sums into effective adjacency |
| [specs/truth-scoring.md](specs/truth-scoring.md) | BTS mechanism, karma accumulation, honesty-weighted effective adjacency |
| [specs/focusing.md](specs/focusing.md) | epoch computation: effective adjacency → tri-kernel → φ*, cyberank, syntropy |
| [specs/impulse.md](specs/impulse.md) | Δφ* — the proven focus shift one signal delivers; locality-bounded sparse vector |

tri-kernel is the mathematical foundation. attention defines the per-neuron input; truth-scoring defines how stake and karma weight the graph. focusing assembles them into the per-epoch computation, and impulse is the per-signal delta Δφ*.

### phase 2 — model compilation

| spec | what it defines |
|------|----------------|
| [specs/focus-flow.md](docs/explanation/focus-flow.md) | the identity between continuous focusing (path A) and compiled transformer inference (path B); architecture parameter derivation |
| [specs/ct0.md](specs/ct0.md) | CT-0 pipeline: 8 passes from `.graph` to `.model`; axon weights and effective adjacency are multivector-valued (§2.5–§2.6); wedge-augmented attention at §7.7; Clifford-block MLP at §8 |

focus-flow explains why compilation works. ct0 specifies exactly how to do it. multivector geometry is native to the spec — axon weights and effective adjacency carry scalar and bivector grades (§2.5–§2.6); the shifted wedge product is defined at §7.7 where first used in attention; the full Clifford(H,C;S) operator is defined at §8 for the MLP pass. when all bivector grades are zero, every Clifford term vanishes and CT-0 is byte-identical to a scalar compile.

### phase 3 — render (owned by mir)

the render spec lives in [[mir]]:

| spec | what it defines |
|------|----------------|
| [mir/specs/render.md](../mir/specs/render.md) | R-1.0 canonical render protocol: spectral layout, tiers T0–T∞, edges, navigation, determinism |
| [mir/specs/render-cyb.md](../mir/specs/render-cyb.md) | cyb implementation of R-1.0: Bevy ECS integration, honeycrisp backend, phase plan |

render depends on tru (for φ*, eigenvectors, cyberank) and on ct0 (§2.6 for bivector adjacency → edge saturation; §7.7 shifted wedge; §8 Clifford block for T∞ render).

### phase 4 — rewards (cross-layer)

| spec | what it defines |
|------|----------------|
| [specs/rewards.md](specs/rewards.md) | the reward function: surprising-syntropy mint, stake-weighted Shapley settlement mining, the stake/work/fee streams, timing and accrual |

the reward layer turns φ* into [[CYB]]: a neuron self-mints the surprising syntropy of its links — Δφ* weighted by [[BTS]] surprise — divided by Shapley and settled by a leaderless sampling lottery. it binds three layers, magnitude in tru, finality in [[foculus]], conservation and mint in [[tok]], so it is a complete design spec that spans the stack rather than a single-repo build step.

## implementation steps

the steps below are in dependency order. each step has a clear input, output, and verifiable predicate. the full milestone plan — module layout, per-spec algorithm, predicate names, and cross-repo blockers — is [roadmap/implementation.md](roadmap/implementation.md).

| step | what | output | verifies |
|------|------|--------|----------|
| 0a | implement `.vocab` parser | `Vocab` struct, particle → bytes lookup | round-trip: hash of parsed vocab matches declared particle |
| 0b | implement `.model` writer/reader | `.cyb` container, mmap-able weights | P-LOAD: cyb-llm loads and runs one forward pass |
| 1a | implement tri-kernel | operators D, S, H_τ; composite R; power iteration | contraction coefficient κ < 1; convergence in expected steps |
| 1b | implement truth-scoring | BTS score → karma; karma-weighted effective adjacency | karma monotone with correct predictions; adjacency matches §3.4 of focusing spec |
| 1c | implement focusing epoch | φ*, cyberank, syntropy, Δφ* per epoch | Σφ*(p) = 1; cyberank matches §3 of focusing spec |
| 2a | implement pass 1–2 | particle index, dialect set | P-DET: two runs produce identical index |
| 2b | implement pass 3 | d*, h*, L* from φ* and graph structure | arch params within clamped bounds; kappa matches contraction rate |
| 2c | implement pass 4 | embedding matrix E | P-EMBED: ‖EE⊤ − M‖_F / ‖M‖_F ≤ 0.05 |
| 2d | implement pass 5 | attention weights W_Q, W_K, W_V, W_O; alpha_beta | P-ATTN: Pearson ≥ 0.7 per head |
| 2e | implement pass 5 Clifford | wedge-augmented score (ct0 §7.7); alpha_beta tensor | P-CLIFFORD-A: Wedge(H,H) = 0; P-CLIFFORD-B: zero-bivector degeneracy |
| 2f | implement pass 6 | Clifford-block MLP weights | P-CLIFFORD-C: jet equivalence |
| 2g | implement pass 7–8 | norms, RoPE config, full `.model` packaging | P-DET: byte-identical on two runs; P-LOAD: loads and runs |
| 3 | implement render phases 1–3 | cyb WorldState::Graph | P-RENDER-TOPO, P-RENDER-POS, P-RENDER-FPS |

## in the stack

| # | repo | what it produces |
|---|------|-----------------|
| 6 | [[cybergraph]] | signals, .graph snapshots |
| 8 | tru | φ*, spectral positions, .model |
| 9 | [[glia]] | inference outputs from .model |
| 10 | [[mir]] | R-1.0 world from φ* and neural features |
| 13 | [[foculus]] | finality from φ* topology |

tru is the only component in the stack that understands graph structure. [[glia]], [[mir]], and [[foculus]] receive its outputs and remain graph-agnostic.

## specs

start with [specs/README.md](specs/README.md) — the build map with layers and status.

- [specs/focusing.md](specs/focusing.md) — effective adjacency, tri-kernel, φ*, eigensolver, cyberank, syntropy
- [specs/tri-kernel.md](specs/tri-kernel.md) — tri-kernel mathematics and convergence proof
- [specs/attention.md](specs/attention.md) — per-neuron focus projection (will + conviction)
- [specs/impulse.md](specs/impulse.md) — Δφ*, the proven per-signal focus shift
- [specs/focus-flow.md](docs/explanation/focus-flow.md) — focusing-to-transformer identity, architecture derivation
- [specs/ct0.md](specs/ct0.md) — CT-0 pipeline (8 passes); multivector inputs §2.5–§2.6; wedge attention §7.7; Clifford MLP §8
- [specs/model.md](specs/model.md) — .model container format
- [specs/truth-scoring.md](specs/truth-scoring.md) — BTS, karma, honesty weighting
- [specs/vocab.md](specs/vocab.md) — .vocab particle dictionary format
- [specs/rewards.md](specs/rewards.md) — reward model (incomplete, out of v0.1 scope)

## docs

- [docs/overview.md](docs/explanation/overview.md) — what tru computes and why it exists
- [docs/tri-kernel.md](docs/explanation/tri-kernel.md) — why diffusion, springs, and heat are the minimal sufficient basis
- [docs/incentives.md](docs/explanation/incentives.md) — the economics of contribution
- [docs/markets.md](docs/explanation/markets.md) — ICBS and market inhibition
- [docs/honesty.md](docs/explanation/honesty.md) — BTS, karma, and why honesty is rational
