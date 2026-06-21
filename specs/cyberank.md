---
tags: cyber, tru, core, spec
crystal-type: measure
crystal-domain: cyber
alias: cyberank, cyber rank, cyberanks
---
# cyberank

[[focus]] read at a single [[particle]]: $\mathrm{cyberank}(p) = \phi^*(p)$. it is the fixed-point probability that the [[tri-kernel]] random process observes particle $p$ — the canonical, network-wide ordering of knowledge.

$$\mathrm{cyberank}(p) = \phi^*(p), \qquad \sum_{p} \mathrm{cyberank}(p) = 1$$

the [[tri-kernel]] computes cyberank by iterating three coupled operators — [[diffusion]], [[springs]], and [[heat]] — to [[equilibrium]]. each particle receives a score proportional to the [[focus]] flowing through its [[cyberlinks]]. the more [[neurons]] stake on paths leading to a particle, the higher its cyberank.

## inheritance from PageRank

cyberank keeps PageRank's recursive idea: a particle is important when important particles point to it. the [[cyber]] variant replaces web hyperlinks with [[cyberlinks]] and replaces the uniform random surfer with a [[focus]]-weighted walker whose teleport distribution reflects real economic [[stake]] and [[karma]]. it extends PageRank with two further operators — [[springs]] for structural constraint and [[heat]] for multi-scale context — so cyberank is the leading term of the full [[tri-kernel]] fixed point, not diffusion alone (see [[tri-kernel]] §2.4 for the five equivalent readings of $\phi^*$).

## what reads it

cyberank is the canonical ordering of knowledge: [[search]] results, feed rankings, [[glia]] routing, and [[karma]] calculations all derive from it. because the computation is deterministic and verifiable, every [[node]] arrives at the same ranking — a shared, checkable measure of collective [[attention]], the consensus reality of what matters.

when a [[neuron]] creates a new [[cyberlink]], it redistributes focus across the graph and [[tru]] recomputes cyberank at the next epoch. the per-signal change is an [[impulse]] $\Delta\phi^*$.

see [[focus]] for the full distribution · [[karma]] for the per-neuron projection · [[tri-kernel]] for the operators · [[focusing]] for the epoch computation.

discover all [[concepts]]
