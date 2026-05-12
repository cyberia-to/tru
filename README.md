# tru

two outputs from one graph: φ* (what the graph collectively knows) and .model (that knowledge made deployable).

## field and compile

the [[tri-kernel]] takes cyberlinks weighted by stake, karma, and [[ICBS]] market prices, and converges to φ*: the unique fixed point of the composite diffusion-springs-heat operator. φ* is the truth distribution — what the network's aggregate of honest signals converges to.

the [[CT-1]] pipeline takes a [[.graph]] snapshot and compiles a transformer. architecture parameters — embedding dimension d*, attention heads h*, layer count L* — are derived directly from φ* and graph structure. weights are computed analytically from the φ*-weighted adjacency via SVD. not trained: compiled.

these are not two separate tools. compilation requires φ*: the embedding matrix is the top singular vectors of the φ*-weighted adjacency. a transformer compiled from truth-weighted signals encodes what the network has collectively validated, not just what was asserted.

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
