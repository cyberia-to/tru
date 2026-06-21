---
tags: cyber, tru, core, spec
crystal-type: relation
crystal-domain: cyber
alias: axon, axons
---
# axon

the bundle of all [[cyberlinks]] between two [[particles]], across every [[neuron]] and all time — itself a [[particle]], rankable and linkable. if a [[cyberlink]] is a synapse, an axon is the nerve fiber: it carries the aggregate signal of many neurons along one directed connection. axons are the natural unit the [[tri-kernel]] operates on — [[diffusion]] flows along them, [[springs]] constrain them, [[heat]] smooths across them.

axons emerge from the [[cybergraph]]; they are never created directly. the [[cybergraph]] is the umbrella that holds the cyberlink record and exposes the axon as axiom A6; [[tru]] defines how an axon is weighted and how [[focus]] flows through it.

## weight

the axon weight for the directed pair $(p, q)$ aggregates every cyberlink from $p$ to $q$:

$$w_{\text{axon}}(p, q) = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} r(\tau(\ell)) \cdot a(\ell)$$

where $a(\ell)$ is the [[box]] stake and $r(\tau(\ell))$ the token weight of the denomination. this is the raw bundle weight; [[focusing]] refines it into effective adjacency $A^{\text{eff}}_{pq}$ by further weighting each summand with the neuron's [[karma]] $\kappa(\nu(\ell))$ and the [[inversely coupled bonding surface|ICBS]] price $f(m(\ell))$.

## homoiconicity — axon as particle

every axon is itself a [[particle]]: $H(\text{from}, \text{to}) \in P$. the hash of the directed edge induces a content-addressed node in the [[cybergraph]]. so axons have [[cyberank]], receive [[focus]], carry [[value]], and can themselves be targets of [[cyberlinks]] — the graph ranks its own structure. this is [[cybergraph]] axiom A6.

## meta-annotation

because an axon is a particle, a neuron can [[cyberlink]] to an axon — meta-annotating a relationship — and stake on axon-particles, betting on the importance of a connection. [[focus]] flows through axon-particles alongside content-particles, so the [[tri-kernel]] ranks relationships and things on the same footing. an [[attention]] write can target an axon as readily as a content particle.

see [[cybergraph]] axiom A6 for the record-level definition · [[cyberlink]] for the atomic assertion · [[focusing]] for effective adjacency · [[focus]] for what flows through axons.

discover all [[concepts]]
