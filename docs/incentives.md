---
tags: cyber, docs
alias: incentives, learning incentives explained
---
# incentives

Why knowledge creation needs a reward system, and how tru makes contributing to the [[cybergraph]] more profitable than free-riding.

---

## the free-rider problem

Knowledge creation is costly. Discovering a genuine connection between two ideas -- finding that a particular [[particle]] relates to another in a way nobody has noticed -- takes time, expertise, and attention. The benefits of that discovery, once published as a [[cyberlink]], flow to everyone who reads the graph. The discoverer bears the cost; the collective reaps the reward.

Without incentives, rational agents free-ride. They consume the graph's knowledge without contributing their own. The result is an epistemic tragedy of the commons: the graph stagnates, the good links stop appearing, and noise fills the vacuum.

---

## the key insight: reward proportional to Δπ

tru's answer is a single principle: reward is proportional to the focus shift your action creates.

$$\text{reward}(v) \propto \Delta\pi(v)$$

π is the focus distribution -- the collective attention of the entire [[cybergraph]], computed by the [[tri-kernel]]. When you add a [[cyberlink]] that shifts focus toward a previously overlooked [[particle]], you have created structure. That structure is value. The protocol recognizes it and mints [[$CYB]] proportional to the shift.

This means creating knowledge IS creating value. There is no separate reward pool, no committee allocation, no inflationary emission schedule disconnected from contribution. New tokens appear if and only if the graph gains new structure. The protocol's inflation is literally evidence of knowledge creation.

---

## the discovery premium

The first neuron to surface a valuable particle captures the largest Δπ. When nobody has linked a particle, the potential focus shift is enormous. The second neuron to link the same particle earns less -- the marginal gain is smaller. The hundredth neuron earns almost nothing.

This is the attention yield curve: early, accurate discovery is maximally rewarded. Late consensus-following earns little. The mechanism creates a race to discover genuine relevance rather than copy existing links.

---

## self-minting

Rewards are not computed centrally. Each [[neuron]] proves its own contribution and claims its own reward.

The process: create [[cyberlinks]], compute the local focus shift $\pi_\Delta$, generate a [[stark]] proof that the computation is correct, bundle everything into a [[cyber/signal]], and submit. Any verifier can check the proof against the block header in O(log n) time. If valid and Δπ > 0, the neuron mints [[$CYB]].

A neuron on a phone can participate: buy a header, query the neighborhood state, create links, prove Δπ, mint tokens. No mining pool, no centralized aggregator, no permission.

---

## confirming links vs foundational links

Different kinds of knowledge earn differently over time.

A foundational link -- the first connection between two important but previously unlinked concepts -- starts with low Δπ that grows over time as the graph builds around it. The reward trajectory rises slowly and persists. This is infrastructure work: the neuron who lays the first bridge between two knowledge clusters earns a long-term yield.

A viral link -- a connection to a particle that immediately attracts attention -- earns high Δπ early but decays fast as focus saturates. Quick returns, short horizon.

A confirming link -- the second or third signal reinforcing an existing connection -- earns lower individual Δπ but strengthens the [[axon]] weight between clusters. Credit is shared through [[Shapley]] attribution, which divides the joint focus shift fairly among contributors.

A semantic bridge -- a cross-module connection -- earns moderate, persistent rewards because it improves the graph's global connectivity.

---

## the game

The rules produce a game where early + accurate = maximum return:

- early, accurate links to important [[particles]] earn the most (the attention yield curve)
- confirming links strengthen [[axon]] weight -- repeated signals build consensus, not noise
- [[neurons]] build long-term reputation through accumulated [[karma]]
- [[focus]] as cost ensures every [[cyberlink]] is a [[costly signal]] -- you must stake real [[$CYB]] to play
- staking cost filters noise, reward function amplifies signal
- a neuron must risk real tokens to earn rewards, ensuring alignment between economic interest and knowledge production

The evolutionary loop that emerges: contribute accurately → Δπ reward → accumulate [[$CYB]] → stake on more links → accumulate [[karma]] → links carry more adjacency weight → earlier Δπ attribution → more [[$CYB]] per contribution. The flywheel rewards sustained accuracy, not one-time luck.

see [[reference/rewards]] for the formal reward functions, attribution formulas, and self-minting protocol. see [[docs/markets]] for how epistemic markets complement the incentive layer. see [[docs/honesty]] for why honest signaling is the dominant strategy.
