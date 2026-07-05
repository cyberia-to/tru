---
tags: cyber, tru, core, spec
crystal-type: property
crystal-domain: cyber
alias: attention, neuron attention, focus projection, attend
---
# attention

the [[focus]] a single [[neuron]] projects onto a target [[particle]] or [[axon]]. where collective [[focus]] (φ\*) is the whole graph's attention distribution, attention is one neuron's *contribution* to it — how much of that neuron's weight lands on each target. attention is the input; φ\* is the convergent output.

attention is not stored. it is computed from the two write-paths a neuron uses to place weight:

| path | scope | mechanism |
|------|-------|-----------|
| [[will]] | broad | locked [[balance]] auto-distributed across every link the neuron makes |
| [[conviction]] | per-link | a [[box]] $(\tau, a)$ locked into one specific edge |

a neuron sets a baseline posture with will, then expresses where it is most certain by raising conviction on particular links. its attention on an edge is the will-derived share plus the per-link conviction.

## into collective focus

each neuron's attention on $(p,q)$ — weighted by its [[karma]] and the [[inversely coupled bonding surface|ICBS]] market price — is one summand of the [[focusing|effective adjacency]] the [[tri-kernel]] converges over:

$$A^{\text{eff}}_{pq} = \sum_{\ell} \underbrace{a(\ell)}_{\text{attention}} \cdot \kappa(\nu(\ell)) \cdot f\big(m(\ell)\big)$$

collective φ\* is the fixed point of the tri-kernel over the sum of every neuron's attention. this is the **converge** step: individual attention aggregates, honesty-weighted, into a single equilibrium.

## the same word, twice

attention is doubly load-bearing, and not by coincidence. it is the per-neuron *input* to focusing — and it is the mechanism the compiled model *runs on*. when [[tru]] compiles the φ\*-weighted graph into a transformer ([[ct0]]), the attention heads are the graph's discovered dialects, and each head's query/key/value projections are factorized from the same effective adjacency attention built. the graph attends to compute φ\*; the model attends to run it. focusing is the continuous limit, the transformer is that limit frozen at finite depth — attention is the operation on both sides of the identity ([[focus-flow]]).

## reading attention

attention is exposed at the [[cybergraph]] boundary via a `query(from, to)` read: the quantity is defined here; the query interface is cybergraph's. valence does not enter attention directly — its epistemic effect is mediated through the [[market]] price.

see [[will]] for the broad budget · [[conviction]] for the per-link box · [[karma]] for the trust weight · [[focus]] for the collective distribution · [[focus-flow]] for the focusing-to-transformer identity.

discover all [[concepts]]
