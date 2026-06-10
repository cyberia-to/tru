---
tags: cyber, tru, core
crystal-type: measure
crystal-domain: cyber
alias: attention, neuron attention, focus projection
---
# attention

the [[focus]] a single [[neuron]] projects onto a target [[particle]] or [[axon]]. where collective $\phi^*$ is the whole graph's attention distribution, a neuron's attention is its individual contribution to that distribution — how much of its weight lands on each target.

attention is a derived focus quantity, not a stored field. it is computed by the [[tru]] from two write paths a neuron uses to place weight:

- [[will]] — broad staking. locking [[$CYB]] for a duration produces will, auto-distributed across every [[cyberlink]] the neuron creates. each link receives a share; longer lock → more will → more attention per link.
- conviction — per-link. the [[box]] $(\tau, a)$ a neuron locks into a specific edge directs attention to that target precisely. fine-tuning beyond the broad will default.

## aggregation into collective focus

individual attention is the per-neuron term that sums into the effective adjacency the [[tri-kernel]] runs on:

$$A^{\text{eff}}_{pq} = \sum_\ell a(\ell)\cdot \kappa(\nu(\ell))\cdot f(m(\ell))$$

each neuron's attention on $(p,q)$ — its will-derived share plus per-link conviction, weighted by its [[karma]] $\kappa$ and the [[inversely coupled bonding surface|ICBS]] market price $f(m)$ — is one summand. collective $\phi^*$ is the fixed point of the tri-kernel over the sum of all neurons' attention. attention is the input; $\phi^*$ is the convergent output.

## reading attention

attention is exposed as a read at the [[cybergraph]] boundary via `query(from, to)` — see [[cybergraph/specs/attention]] for the query-interface view. the query mechanism is cybergraph's; the quantity it returns is defined here.

see [[focus]] for the collective distribution · [[will]] for broad staking · [[box]] for per-link conviction · [[karma]] for the trust weight · [[focus-flow]] for the computation.
