---
tags: cyber, docs
alias: markets explained, ICBS explained, epistemic markets explained
---
# markets

Why every [[cyberlink]] needs a market, how the [[inversely coupled bonding surface]] works, and how market price acts as an inhibitory synapse in the [[cybergraph]].

---

## the problem of false assertions

In any knowledge graph, false assertions persist. Someone links "the earth is flat" to "scientific fact" and stakes on it. Without a mechanism for the collective to push back, the link sits there, accumulating attention, distorting the focus distribution. Deleting links requires governance. Governance is slow, political, and does not scale.

tru's answer: attach a market to every link and let the collective express disbelief with capital. If enough neurons bet against a link, its effective weight drops toward zero. The link still exists in the topology, but the market has muted it. No committee, no vote, no deletion -- just economic pressure.

---

## ICBS: the circle metaphor

The [[inversely coupled bonding surface]] is a two-sided bonding curve where YES and NO live on a shared circle.

Imagine a circle in a plane. One axis is YES supply, the other is NO supply. The cost of the market at any point equals the distance from the origin:

$$C(s_{YES},\, s_{NO}) = \lambda \sqrt{s_{YES}^2 + s_{NO}^2}$$

Trading moves outward along the circle's surface. Buying YES pushes the point toward the YES axis. Buying NO pushes it toward the NO axis. The key geometric fact: moving toward one axis moves away from the other. Buying YES directly suppresses the price of NO, and buying NO directly suppresses the price of YES.

This is the inverse coupling. TRUE and FALSE are genuine opposites on a shared geometric surface. There is no way to increase belief in one without decreasing belief in the other.

---

## how market price acts as inhibitory synapse

In a standard graph, every link contributes positively to attention flow. [[Diffusion]] spreads focus along edges without discrimination. A false link with high stake carries the same weight as a true link with high stake.

Market inhibition changes this. The ICBS market price enters the [[tri-kernel]] as a multiplier on the link's effective weight:

$$w_{\text{eff}}(\ell) \;=\; w(\ell) \cdot \bigl(1 - \alpha \cdot m(\ell)\bigr)$$

When the market believes a link (price near YES), the multiplier is close to 1 -- full focus flows through. When the market doubts a link (price near NO), the multiplier drops toward zero -- the link is effectively silenced. This is the computational analog of an inhibitory synapse in a biological neural network: excitation (staking) says "this matters," inhibition (market disbelief) says "this is wrong."

The result is a graph where every link carries two signals simultaneously -- how much stake supports it, and how much the market believes it. Both must be high for the link to carry weight.

---

## self-scaling liquidity

ICBS markets grow their own liquidity. The total value locked always equals the cost function: $TVL = C(s_{YES}, s_{NO})$. Every trade adds to the pool. The most-contested links -- the ones where belief and disbelief are both heavily staked -- become the most liquid markets, which produce the most accurate prices.

No external liquidity providers are needed. No market makers. No governance to bootstrap a new market. The act of disagreeing IS the act of providing liquidity.

---

## why the market is perpetual

There is no oracle. There is no resolution event. The ICBS price is the living, continuous verdict of collective epistemic assessment. A link that was disbelieved yesterday can be vindicated tomorrow if new evidence shifts the market. A link that was trusted for years can be suppressed if the collective changes its mind.

This is critical for a knowledge graph. Truth is not a one-time event. Understanding evolves. The market must evolve with it.

---

## the 2|3 architecture

Every [[cyberlink]] carries three simultaneous signals:

1. Topology: the edge exists -- the neuron asserts this structural connection (binary)
2. Market: the ICBS price -- the collective's assessment of the link's validity (continuous)
3. Meta-prediction: the [[valence]] -- the neuron's prediction of where the market will converge (ternary: $v \in \{-1, 0, +1\}$)

One atomic act -- creating a link -- simultaneously asserts structural knowledge, opens a perpetual prediction market, and registers a BTS meta-prediction. Three channels of epistemic signal from a single action.

The effective adjacency weight combines all three:

$$A^{\text{eff}}_{pq} = \sum_\ell \text{stake}(\ell) \times \text{karma}(\nu(\ell)) \times f(\text{ICBS price}(\ell))$$

Stake measures commitment. [[Karma]] measures track record. Market price measures collective belief. A link must score high on all three to carry real weight in the [[cybergraph]].

---

## the cost of disbelief

Inhibition is symmetric with assertion. A neuron that inhibits a link must stake into the ICBS market against it. If the link turns out to be valid, the inhibitor loses stake. Belief and disbelief are both costly -- cheap talk in either direction is eliminated.

Capital flows from incorrect beliefs to correct ones. The neurons who consistently bet correctly accumulate wealth. The neurons who consistently bet wrong lose stake. This is the market's contribution to the evolutionary pressure that drives the graph toward truth.

see [[reference/epistemic-markets]] for the full ICBS specification, formulas, and properties table. see [[docs/honesty]] for how BTS interacts with market signals. see [[docs/incentives]] for how market positions fit into the broader reward system.
