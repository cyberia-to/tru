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

the compile job reads the same φ*-weighted graph and derives the CT-1 transformer: embedding dimension d*, attention heads h*, layer count L*, weight matrix W from the top singular vectors of the φ*-weighted adjacency. not trained: compiled.

these are the same computation at different scales. the field is the continuous limit; the compiled model is that limit frozen at finite depth L*. the [[CT-1]] pipeline makes this precise — eight passes from .graph to .model, each pass an exact derivation of transformer parameters from φ*.

## the name

φ* is the collective truth distribution — the closest approximation to shared knowledge the network can compute. tru computes it.

the input to tru is already weighted by honesty: [[karma]] records each neuron's [[BTS]] score history (how often they were right before the crowd). [[ICBS]] prices encode the market's collective epistemic assessment of each link. stake is economic commitment. the fixed point of the tri-kernel over these three inputs is what tru calls truth.

## self-minting

tru computes Δφ* — the shift in φ* before and after a neuron's batch of links. a neuron proves Δφ* via [[zheng]] and self-mints [[CYB]] proportional to the proven shift. no central aggregator decides who contributed what. each neuron runs tru locally, generates the proof, submits the claim.

this is why tru runs locally on every [[neuron]], not only on validators. validators run it for consensus. every neuron runs it to earn.

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
- [specs/ct1.md](specs/ct1.md) — CT-1 compilation pipeline: eight passes from .graph to .model
- [specs/model.md](specs/model.md) — .model container format

the .graph input format is specified in [[cybergraph]]: [cybergraph/specs/graph.md](../cybergraph/specs/graph.md).

## docs

- [docs/overview.md](docs/overview.md) — what tru computes and why it exists
- [docs/tri-kernel.md](docs/tri-kernel.md) — why diffusion, springs, and heat are the minimal sufficient basis
- [docs/incentives.md](docs/incentives.md) — the economics of contribution
- [docs/markets.md](docs/markets.md) — ICBS and market inhibition
- [docs/honesty.md](docs/honesty.md) — BTS, karma, and why honesty is rational
