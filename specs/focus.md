---
tags: cyber, tru, core, spec
crystal-type: entity
crystal-domain: cyber
alias: focus, collective focus, collective attention, focus distribution, cyberank distribution, pi star, phi star
---
# focus

the collective attention distribution over the [[cybergraph]]. focus is the [[tri-kernel]] stationary distribution $\phi^*$ over all [[particles]] — content-particles and [[axon]]-particles alike — emerging from the aggregate of every [[neuron]]'s signals. it is what the network, as one system, attends to.

$$\phi^* = \operatorname{norm}\big[\mathcal{R}(\phi^*)\big], \qquad \sum_i \phi^*(i) = 1, \qquad \phi^*(i) > 0\ \forall i$$

focus is computed, never assigned. no one votes on it. it is the unique fixed point the [[tri-kernel]] converges to — [[diffusion]] explores, [[springs]] enforce structure, [[heat]] adapts across scale — and by the [[collective focus theorem]] that fixed point exists and is unique.

## how focus emerges

| layer | what | per-what |
|-------|------|----------|
| [[balance]] | tokens held | [[neuron]] |
| [[will]] | balance locked × duration | [[neuron]] |
| [[attention]] | will allocated to targets | [[neuron]] × [[particle]] |
| focus | collective attention | [[particle]] |
| [[cyberank]] | focus read at one particle | [[particle]] |

[[neurons]] lock [[balance]] to create [[will]]. will auto-distributes across the [[cyberlinks]] a neuron creates, producing [[attention]] at target particles. the [[tri-kernel]] aggregates all attention — weighted by [[karma]] and [[inversely coupled bonding surface|ICBS]] price into effective adjacency (see [[focusing]]) — into a single probability distribution $\phi^*$. that distribution is focus.

## conservation

$$\sum_i \phi^*(i) = 1\quad\text{always, by normalization}$$

focus sums to one because it is a probability measure. concentrating attention on one particle defocuses every other. this is the normalization step of the tri-kernel, the same identity that makes [[syntropy]] well-defined.

## the slices of focus

focus is the whole distribution. its named projections:

- [[cyberank]] — focus per particle, $\phi^*(p)$. the canonical ordering of knowledge.
- [[karma]] — focus per neuron. how much of the collective's attention a neuron's honest history commands.
- [[attention]] — one neuron's contribution to focus, before aggregation.

## what focus drives

every [[cyberlink]] shifts $\phi^*$; the proven shift is an [[impulse]] $\Delta\phi^*$. learning and ranking are the same operation. focus feeds [[ct0]] model compilation (the embedding is built from $\phi^*$-weighted adjacency), [[foculus]] finality, [[mir]] world geometry, and [[glia]] routing. it is the single output everything downstream reads.

see [[syntropy]] for the order focus carries · [[cyberank]] for focus per particle · [[will]] for how attention is funded · [[tri-kernel]] and [[collective focus theorem]] for the convergence · [[focusing]] for the per-epoch computation.

discover all [[concepts]]
