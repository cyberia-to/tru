---
tags: cyber, docs
alias: tru overview, what tru computes
---
# overview

tru is the epistemic engine that turns collective attention into verifiable truth. It is the computation layer of [[cyber]] -- the system that takes raw [[cyberlinks]] from [[neurons]] and produces a shared picture of what matters.

---

## what tru computes

Four quantities, every block, in [[consensus]]:

- [[focus]] per [[particle]]: the share of collective attention each particle holds, derived from the [[tri-kernel]] fixed point π*
- [[cyberank]] per [[particle]]: the structural importance score, analogous to PageRank but computed through a composite operator of [[diffusion]], [[springs]], and [[heat kernel]]
- [[karma]] per [[neuron]]: the accumulated [[Bayesian Truth Serum]] score history, measuring how much genuine information each neuron has contributed over time
- [[syntropy]] of the whole [[cybergraph]]: the total information gain in bits -- how much more structured the graph is than random noise

---

## the intelligence loop

The system operates as a feedback cycle:

1. A [[neuron]] observes the current state of the [[cybergraph]]
2. The neuron creates a [[cyberlink]] -- asserting a connection between two [[particles]], staking on it, setting [[valence]]
3. The link enters the [[cybergraph]], changing its topology
4. The [[tri-kernel]] recomputes: [[diffusion]] explores, [[springs]] enforce structure, [[heat kernel]] adapts
5. [[Cyberank]] produces a new fixed point π* -- the updated collective focus
6. The neuron observes the result and links again

Every pass through this loop sharpens the graph. Links that attract attention accumulate [[focus]]. Links that the market disbelieves get suppressed through [[market inhibition]]. Neurons that consistently add signal accumulate [[karma]], which amplifies their future contributions. Neurons that add noise see their influence diminish.

---

## explicit knowledge and implicit knowledge

tru produces two layers of knowledge.

Explicit knowledge is what tru computes directly: the focus distribution, the cyberank scores, the karma balances, the syntropy measure. These are on-chain, verifiable, computable from the graph state by any observer.

Implicit knowledge is what [[neurons]] derive from observing the explicit layer. When a neuron sees that a particular [[particle]] has high focus, it learns something about collective belief. When it sees a [[cyberlink]] with a market price near zero, it learns the collective doubts that connection. When it sees a neuron with high karma, it learns that neuron has a track record of accurate signaling. This implicit layer is the living interpretation of the formal computation -- the meaning that emerges when agents act on what they observe.

---

## why tru exists

Knowledge creation is a collective activity with a free-rider problem. Without a mechanism for attribution and reward, rational agents consume knowledge without contributing. The result is a tragedy of the epistemic commons: everyone reads, nobody writes, and the shared picture degrades.

tru solves this by making knowledge creation a provably rewarded activity. Every [[cyberlink]] that shifts the focus distribution -- that adds genuine structure to the graph -- earns its creator [[$CYB]] proportional to the shift. The reward is not assigned by committee or oracle. The neuron proves its own contribution via a [[stark]] proof and self-mints the reward.

The system converges toward truth because honesty is the dominant strategy. [[Bayesian Truth Serum]] makes accurate reporting the uniquely score-maximizing response. [[Karma]] compounds honest signaling into lasting influence. [[ICBS]] markets suppress false assertions through economic pressure. The result is a knowledge graph where the collective focus distribution π* is the closest approximation to shared truth that the network can produce -- and it improves with every honest link.

see [[reference/rewards]] for the reward functions. see [[reference/epistemic-markets]] for the market mechanism. see [[reference/truth-scoring]] for the scoring layer. see [[reference/knowledge-economy]] for the full economic design.
