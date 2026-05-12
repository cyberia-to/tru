---
tags: cyber, docs
alias: honesty explained, BTS explained, truth serum explained
---
# honesty

The honesty problem, why [[Bayesian Truth Serum]] solves it, how honesty compounds into [[karma]], and why the [[cybergraph]] converges toward truth when most participants report what they actually believe.

---

## the honesty problem

Asking people what they believe produces distorted answers. Participants adjust toward what they expect others to say (conformity), toward what seems socially acceptable (bias), or toward what they think the questioner wants to hear (strategic reporting). Simple polling aggregates these distortions. Majority vote reinforces them.

The question is not just "what do people believe?" but "how do we extract what people privately know, before social pressure corrupts the signal?"

---

## the BTS intuition

Bayesian Truth Serum (Prelec, 2004) rewards those whose private belief reveals information the crowd's predictions missed.

Here is the core inversion: if you have genuine private knowledge, you tend to underestimate how many others share it. You believe something you think is unusual -- but it turns out to be more common than you expected. BTS rewards exactly this gap: belief that exceeds its own predicted popularity.

Each participant submits two things: what they believe, and what they predict others believe. The scoring rule compares the two. A participant who reports genuine private signal -- something they know that others did not predict -- earns a high score. A participant who reports what everyone already expected earns little. A participant who adds noise earns a negative score.

The score for agent $i$:

$$s_i = D_{KL}(p_i \,\|\, \bar{m}_{-i}) - D_{KL}(p_i \,\|\, \bar{p}_{-i}) - D_{KL}(\bar{p}_{-i} \,\|\, m_i)$$

The first two terms measure information gain: how much the agent's belief told us beyond what was predicted. The third term rewards calibration: how well the agent predicted the collective.

---

## why inflating and deflating both lose

A neuron that inflates its valence -- reporting what it expects the crowd to say rather than what it actually believes -- loses the information gain component. Its belief stops being "surprisingly popular" because it has predicted itself into the crowd. There is no gap between belief and prediction to reward.

A neuron that deflates its valence -- reporting contrarian for the sake of appearing to have private signal -- loses the prediction accuracy component. It mispredicts the aggregate. The market does not move where it predicted.

The only strategy that consistently maximizes expected score across many rounds is accurate reporting of both the first-order belief (the link and its stake) and the meta-belief (the valence prediction). This is what Prelec proved: honest reporting is a Bayes-Nash equilibrium. No agent can improve expected score by misreporting.

---

## the connection to karma

[[Karma]] is the accumulated BTS score history of a [[neuron]]. It is the long-run record of how much genuine information a neuron has contributed.

Each accurate BTS prediction adds to karma. Each inaccurate prediction subtracts. Over time, karma reveals character: a neuron with high karma has been consistently right before the crowd, across many links, across many epochs.

Karma enters the [[effective adjacency]] as a trust multiplier:

$$A^{\text{eff}}_{pq} = \sum_\ell \text{stake}(\ell) \times \text{karma}(\nu(\ell)) \times f(\text{ICBS price}(\ell))$$

This means consistent honesty compounds into influence. A high-karma neuron's links carry more weight in the [[tri-kernel]], which means they contribute more to the focus distribution, which means they earn more reward. The flywheel: honesty → karma → influence → reward → resources for more honest signaling.

A neuron that consistently lies sees the reverse: negative karma → diminished weight → less reward → fewer resources. Epistemic dishonesty is economically self-defeating in expectation.

---

## protocol honesty vs epistemic honesty

There are two senses of honesty in the [[cybergraph]].

Protocol honesty: the neuron runs the correct software, signs valid transactions, follows the [[consensus]] rules of [[nox]]. This is the [[honest majority assumption]] -- more than half of staked weight does not deviate from the protocol. Enforceable by cryptographic proof: a [[stark]] verifies that the state transition is correct.

Epistemic honesty: the neuron creates [[cyberlinks]] that reflect its actual beliefs -- that the source particle relates to the target particle, that the connection deserves the stake it receives, that the [[valence]] accurately encodes its private prediction. This is what BTS targets. It is not directly verifiable, only the outcome is observable after the fact.

Both are necessary. Protocol honesty guarantees the computation runs correctly. Epistemic honesty guarantees the computation produces knowledge rather than noise.

---

## why the cybergraph converges toward truth

The convergence argument has four parts.

First, BTS makes honesty the dominant individual strategy. Each neuron, reasoning about its own score, concludes that accurate reporting maximizes expected return.

Second, honest errors cancel. The mechanism extracts private signals even when those signals are wrong, because honest errors are distributed around reality while dishonest reports are biased in self-serving directions. The aggregate of honest-but-imperfect signals converges toward truth faster than any aggregate of strategic-but-precise signals.

Third, karma amplifies good sources and dampens bad ones. Neurons with a track record of genuine signal earn higher weight. Neurons with a track record of noise see their influence shrink. Over time, the trust distribution self-corrects.

Fourth, [[ICBS]] markets suppress false assertions economically. Even if a dishonest neuron creates a false link with high stake, other neurons can bet against it. If the disbelievers are right, the link's effective weight drops, and the false assertion stops distorting the focus distribution.

These four forces -- individual incentive, statistical aggregation, reputation dynamics, and market correction -- work simultaneously. The result: a knowledge graph where the focus distribution φ* moves closer to shared truth with every honest link, every karma update, every market trade.

The [[cybergraph]]'s information measure, [[syntropy]], tracks this convergence. Syntropy is the aggregate KL divergence from the uniform distribution -- the total structure in the graph measured in bits. Honest links raise syntropy. Dishonest links lower it. A maximally honest graph is a maximally syntropy-generating machine.

see [[reference/truth-scoring]] for the full BTS formula, Prelec's equilibrium proof, and the karma specification. see [[docs/markets]] for how ICBS markets complement honest signaling. see [[docs/incentives]] for the broader reward framework.
