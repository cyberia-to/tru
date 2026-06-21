---
tags: cyber, tru, core, spec
crystal-type: property
crystal-domain: cyber
alias: conviction, convictions, per-link conviction
---
# conviction

the economic commitment a [[neuron]] places on a single [[cyberlink]]: how much capital it locks behind one specific assertion. conviction is the per-link counterpart of [[will]] — where will is the broad budget auto-distributed across every link a neuron makes, conviction is the precise weight directed at one edge. together they are the two write-paths that sum into [[attention]], the input [[tru]] reads to build effective adjacency.

conviction is made concrete as a [[box]] — the pair $(\tau, a)$ of a token denomination and an [[amount]] bound to the cyberlink. [[cybergraph]] carries the box record and its value mechanics; tru reads its magnitude as the conviction weight on the edge.

## will and conviction

| path | scope | mechanism | tunes |
|------|-------|-----------|-------|
| [[will]] | broad | locked [[balance]] auto-distributed across all of a neuron's links | the baseline strategy |
| conviction | per-link | a [[box]] $(\tau, a)$ locked into one specific edge | the precise bet |

a neuron sets a default posture with will, then expresses where it is most certain by raising conviction on particular links. attention on an edge is the will-derived share plus the per-link conviction, before tru weights it by [[karma]] and [[inversely coupled bonding surface|ICBS]] price.

## the conviction spectrum

| box value | meaning |
|-----------|---------|
| $a = 0$ | bare assertion — structural presence, no economic exposure |
| $a$ small | low conviction — the neuron acknowledges the connection but risks little |
| $a$ large | high conviction — the neuron bets real capital that this link matters |
| $a \to$ burn | permanent conviction — tokens destroyed for an [[eternal cyberlinks\|eternal cyberlink]] |

conviction is expressed by how much and how long capital is tied up, not by destroying it — a box is recoverable (withdraw) while the structural assertion it backed remains. burning is the limit case where conviction is made irreversible.

## what conviction weighs

conviction is one of the three factors in effective adjacency (see [[focusing]]):

$$A^{\text{eff}}_{pq} = \sum_{\substack{\ell \in L \\ \text{src}(\ell)=p,\;\text{tgt}(\ell)=q}} a(\ell)\cdot\kappa(\nu(\ell))\cdot f\big(m(\ell)\big)$$

$a(\ell)$ is the conviction (box magnitude); $\kappa$ is the neuron's [[karma]]; $f(m)$ is the [[market inhibition]] multiplier. a large conviction behind a low-karma neuron, or on a link the market disbelieves, contributes little — all three must align. this is what makes a [[cyberlink]] a [[costly signal]]: conviction spent on one claim is conviction withheld from every other.

conviction is distinct from [[valence]]: valence is the epistemic prediction ($v \in \{-1,0,+1\}$), conviction is the economic depth ($a \in \mathbb{R}_+$). a neuron can hold high conviction with any valence — betting heavily that a link is true, or betting heavily while predicting the market will judge it false.

see [[will]] for the broad budget · [[box]] for the concrete container and its lifecycle · [[attention]] for how will and conviction combine · [[valence]] for the orthogonal epistemic field · [[focusing]] for effective adjacency.

discover all [[concepts]]
