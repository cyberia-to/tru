# tru

tru is a convergence VM — the intelligence layer of [[soft3]]. where [[nox]] executes programs and [[glia]] runs models, tru operates by field convergence: it iterates the [[tri-kernel]] until φ* emerges as the unique fixed point.

convergence and derivation are different computation models. derivation proceeds from axioms to conclusions in bounded depth — every formal system, every program execution, every forward pass reaches only what its starting axioms can produce. convergence proceeds by iteration toward equilibrium — the result is the attractor, not a logical consequence. φ* is not derived from the graph: it emerges from it.

| vm | execution model | gödel status |
|----|-----------------|--------------|
| [[nox]] | derivation | confined |
| [[zheng]] | verification | confined |
| [[glia]] | forward pass | confined |
| [[tru]] | field convergence | free |

## field and compile

two jobs, one engine.

the field job runs the [[tri-kernel]] over every signal: reads signal.a (stake) and signal.v (valence) → composite diffusion-springs-heat operator R = λ_d·D + λ_s·S + λ_h·H_τ → iterates to fixed point → φ*. each particle's φ*(p) is the focus it has earned from the entire weighted graph. the system converges; no authority assigns the result.

the compile job reads the same φ*-weighted graph and derives the CT-0 transformer: embedding dimension d*, attention heads h*, layer count L*, weight matrix W from the top singular vectors of the φ*-weighted adjacency. not trained: compiled.

these are the same computation at different scales. the field is the continuous limit; the compiled model is that limit frozen at finite depth L*. the [[CT-0]] pipeline makes this precise — eight passes from .graph to .model, each pass an exact derivation of transformer parameters from φ*.

## the name

φ* is the collective truth distribution — the closest approximation to shared knowledge the network can compute. tru computes it.

the input to tru is already weighted by honesty: [[karma]] records each neuron's [[BTS]] score history (how often they were right before the crowd). [[ICBS]] prices encode the market's collective epistemic assessment of each link. stake is economic commitment. the fixed point of the tri-kernel over these three inputs is what tru calls truth.

## self-minting

tru computes Δφ* — the shift in φ* before and after a neuron's batch of links. a neuron proves Δφ* via [[zheng]] and self-mints [[CYB]] proportional to the proven shift. no central aggregator decides who contributed what. each neuron runs tru locally, generates the proof, submits the claim.

this is why tru runs locally on every [[neuron]], not only on validators. validators run it for consensus. every neuron runs it to earn.

## spec map

the specs are organized in dependency order. implement them in the sequence below.

### phase 0 — data formats

| spec | what it defines |
|------|----------------|
| [specs/vocab.md](specs/vocab.md) | `.vocab` particle dictionary: content-addressed particle → bytes mapping |
| [specs/model.md](specs/model.md) | `.model` container format: the inference-ready artifact CT-0 produces |

these two formats are prerequisites for everything else. vocab feeds pass 1; model is the output of pass 8.

### phase 1 — field computation

| spec | what it defines |
|------|----------------|
| [specs/tri-kernel.md](specs/tri-kernel.md) | the three operators (diffusion D, springs S, heat H_τ), composite R, fixed-point theorem, convergence proof |
| [specs/truth-scoring.md](specs/truth-scoring.md) | BTS mechanism, karma accumulation, honesty-weighted effective adjacency |
| [specs/field.md](specs/field.md) | epoch computation: effective adjacency → tri-kernel → φ*, cyberank, syntropy, Δφ* |

tri-kernel is the mathematical foundation. truth-scoring defines how stake and karma weight the graph. field assembles them into the per-epoch computation.

### phase 2 — model compilation

| spec | what it defines |
|------|----------------|
| [specs/focus-flow.md](specs/focus-flow.md) | the identity between continuous field convergence (path A) and compiled transformer inference (path B); architecture parameter derivation |
| [specs/ct0.md](specs/ct0.md) | CT-0 pipeline: 8 passes from `.graph` to `.model`; axon weights and effective adjacency are multivector-valued (§2.5–§2.6); wedge-augmented attention at §7.7; Clifford-block MLP at §8 |

focus-flow explains why compilation works. ct0 specifies exactly how to do it. multivector geometry is native to the spec — axon weights and effective adjacency carry scalar and bivector grades (§2.5–§2.6); the shifted wedge product is defined at §7.7 where first used in attention; the full Clifford(H,C;S) operator is defined at §8 for the MLP pass. when all bivector grades are zero, every Clifford term vanishes and CT-0 is byte-identical to a scalar compile.

### phase 3 — render (owned by mir)

the render spec lives in [[mir]]:

| spec | what it defines |
|------|----------------|
| [mir/specs/render.md](../mir/specs/render.md) | R-1.0 canonical render protocol: spectral layout, tiers T0–T∞, edges, navigation, determinism |
| [mir/specs/render-cyb.md](../mir/specs/render-cyb.md) | cyb implementation of R-1.0: Bevy ECS integration, honeycrisp backend, phase plan |

render depends on tru (for φ*, eigenvectors, cyberank) and on ct0 (§2.6 for bivector adjacency → edge saturation; §7.7 shifted wedge; §8 Clifford block for T∞ render).

### not yet ready — rewards

[specs/rewards.md](specs/rewards.md) defines the economic incentive model (Δφ* self-minting, attribution, token ops). the current definition is incomplete and this spec is excluded from the v0.1 implementation scope. it will be addressed as a standalone design task.

## implementation steps

the steps below are in dependency order. each step has a clear input, output, and verifiable predicate.

| step | what | output | verifies |
|------|------|--------|----------|
| 0a | implement `.vocab` parser | `Vocab` struct, particle → bytes lookup | round-trip: hash of parsed vocab matches declared particle |
| 0b | implement `.model` writer/reader | `.cyb` container, mmap-able weights | P-LOAD: cyb-llm loads and runs one forward pass |
| 1a | implement tri-kernel | operators D, S, H_τ; composite R; power iteration | contraction coefficient κ < 1; convergence in expected steps |
| 1b | implement truth-scoring | BTS score → karma; karma-weighted effective adjacency | karma monotone with correct predictions; adjacency matches §3.4 of field spec |
| 1c | implement field epoch | φ*, cyberank, syntropy, Δφ* per epoch | Σφ*(p) = 1; cyberank matches §3 of field spec |
| 2a | implement pass 1–2 | particle index, semcon set | P-DET: two runs produce identical index |
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

- [specs/field.md](specs/field.md) — effective adjacency, tri-kernel, φ*, eigensolver, cyberank, syntropy, Δφ*
- [specs/tri-kernel.md](specs/tri-kernel.md) — tri-kernel mathematics and convergence proof
- [specs/focus-flow.md](specs/focus-flow.md) — field-to-transformer identity, architecture derivation
- [specs/ct0.md](specs/ct0.md) — CT-0 pipeline (8 passes); multivector inputs §2.5–§2.6; wedge attention §7.7; Clifford MLP §8
- [specs/model.md](specs/model.md) — .model container format
- [specs/truth-scoring.md](specs/truth-scoring.md) — BTS, karma, honesty weighting
- [specs/vocab.md](specs/vocab.md) — .vocab particle dictionary format
- [specs/rewards.md](specs/rewards.md) — reward model (incomplete, out of v0.1 scope)

## docs

- [docs/overview.md](docs/overview.md) — what tru computes and why it exists
- [docs/tri-kernel.md](docs/tri-kernel.md) — why diffusion, springs, and heat are the minimal sufficient basis
- [docs/incentives.md](docs/incentives.md) — the economics of contribution
- [docs/markets.md](docs/markets.md) — ICBS and market inhibition
- [docs/honesty.md](docs/honesty.md) — BTS, karma, and why honesty is rational
