---
tags: cyber, cybics, core, reference
alias: Bayesian Truth Serum, BTS, peer prediction, truth serum, serum, karma, honesty, honest, epistemic honesty
---
# truth scoring

The scoring layer of the [[cybergraph]]: how [[Bayesian Truth Serum]] (Prelec, 2004) extracts honest private signals from [[neurons]], how those scores accumulate into [[karma]], and how the protocol makes honesty the uniquely rational strategy.

---

## BTS score formula

Each participant submits two things: a personal belief (probability distribution over outcomes) and a prediction of the aggregate of others' beliefs.

The score for agent $i$:

$$s_i = \underbrace{D_{KL}(p_i \,\|\, \bar{m}_{-i}) - D_{KL}(p_i \,\|\, \bar{p}_{-i})}_{\text{information gain}} - \underbrace{D_{KL}(\bar{p}_{-i} \,\|\, m_i)}_{\text{prediction accuracy}}$$

where:
- $p_i$ is the agent's true belief
- $m_i$ is their prediction of others' aggregate beliefs
- $\bar{p}_{-i}$ is the geometric mean of others' actual beliefs
- $\bar{m}_{-i}$ is the geometric mean of others' predictions

The information gain term captures how much the agent's belief differed from what others predicted, corrected by what others actually believed. The prediction accuracy term rewards calibration about the collective.

Negative scores indicate noise -- the agent added distortion rather than signal. Stake redistributes from noise producers to signal producers proportional to scores.

---

## what BTS measures

BTS measures information contribution in bits. The [[KL divergence]] between the agent's belief and the predicted mean ($D_{KL}(p_i \| \bar{m}_{-i})$) measures the agent's surprise relative to the prior. The correction term ($D_{KL}(p_i \| \bar{p}_{-i})$) removes the portion attributable to consensus rather than private signal.

The net score is the agent's unique informational contribution: what they knew that the group did not already know and did not already expect.

---

## Prelec's equilibrium proof

Prelec proved that truthful reporting of $p_i$ (actual belief) and $m_i$ (actual prediction of others) is a Bayes-Nash equilibrium: no agent can improve their expected score by misreporting either quantity.

The mechanism is incentive-compatible because:
- inflating belief toward popularity loses the information gain component (the belief stops being more popular than predicted once the agent has predicted it into the crowd)
- deflating belief to seem contrarian loses the prediction accuracy component (the agent mispredicts the aggregate)
- the only strategy that consistently maximizes expected score is accurate reporting of both belief and meta-belief

This is why the mechanism is called a "serum" -- it does not rely on virtue. It makes honesty the dominant response through score structure alone.

---

## relation to the wisdom of the crowds

The [[wisdom of the crowds]] (Galton, 1907) aggregates raw beliefs. It works when errors are independent and cancel. It fails when beliefs are correlated -- when the crowd shares a common bias, errors compound rather than cancel ([[Condorcet]] jury theorem requires independence).

BTS corrects for correlated bias by using second-order beliefs (predictions about predictions) to detect and discount systematic distortions. It does not require independent beliefs -- it only requires that truthful agents' private signals are distributed around reality, even if all agents share a common prior.

---

## mapping to cyberlinks

In [[cyber]], the [[cyberlink]] IS the BTS input -- no separate submission step required:

| BTS concept | cyberlink field |
|---|---|
| first-order belief $p_i$ | link creation + stake $(\tau, a)$ -- the neuron asserts the connection and stakes on it |
| meta-prediction $m_i$ | valence $v \in \{-1, 0, +1\}$ -- the neuron's prediction of how the [[ICBS]] market on this edge will converge |
| agent identity | $\nu$ -- the signing neuron |

Every [[cyberlink]] is simultaneously a structural assertion and a BTS prediction, in one atomic act. The scoring engine computes $s_i$ for every [[neuron]] from the public graph without any additional input.

---

## karma

Karma is the accumulated BTS score history of a [[neuron]] -- the record of how much [[information]] a neuron has contributed to the collective over time. A neuron that repeatedly links things the market later validates has high karma. A neuron that links noise has low karma.

Karma is non-transferable. It cannot be bought with stake alone. It is earned by consistently being right before the crowd.

High karma means the network has observed a track record of genuine private signals. That track record enters [[effective adjacency]] as $\kappa(\nu)$ -- the trust multiplier that amplifies future contributions from consistently honest neurons.

---

## karma in effective adjacency

Karma weights every future link the neuron creates in the [[tri-kernel]] effective adjacency:

$$A^{\text{eff}}_{pq} = \sum_\ell \text{stake}(\ell) \times \underbrace{\text{karma}(\nu(\ell))}_{\text{BTS history}} \times f(\text{ICBS price}(\ell))$$

This makes karma an epistemic weight, not merely an economic one. Epistemic capital -- the form of wealth that can only be earned by being right before the crowd.

---

## honesty: three atomic acts

In the [[cybergraph]], honesty is expressed through three acts that form one atomic record:

1. Creating the [[cyberlink]] -- "I believe this connection exists"
2. Setting the stake -- "how strongly I believe it"
3. Setting [[valence]] -- "my honest prediction of where the market will settle"

Honesty and correctness are independent properties. A neuron is honest when it reports what it actually believes, regardless of whether that belief is accurate. A neuron is correct when its belief matches reality. Honesty is a property of the reporting; correctness is a property of the belief's relationship to the world. BTS does not require correctness -- it requires honesty.

---

## protocol honesty vs epistemic honesty

Protocol honesty: the [[neuron]] runs the correct software, signs valid transactions, and follows the [[consensus]] rules of [[nox]]. This is what the [[honest majority assumption]] requires -- more than half of staked weight does not deviate from the protocol. It is enforceable by cryptographic proof: a [[stark]] verifies that the state transition is correct. Dishonesty at this level is detectable.

Epistemic honesty: the [[neuron]] creates [[cyberlinks]] that reflect its actual beliefs -- that the source particle relates to the target particle, that the connection deserves the stake it receives, that [[valence]] $v$ accurately encodes its private prediction. This is what [[Bayesian Truth Serum]] targets. It is not directly verifiable -- only the outcome (whether the market confirmed the prediction) is observable after the fact.

Both are necessary. Protocol honesty guarantees the computation runs correctly. Epistemic honesty guarantees the computation produces knowledge rather than noise.

---

## why honesty is rational

[[Bayesian Truth Serum]] proves that epistemic honesty is a Bayes-Nash equilibrium: when a neuron believes other neurons are reporting honestly, honest reporting is the uniquely score-maximizing response.

The logic:
- a neuron that inflates [[valence]] toward what it expects the crowd to say loses its information gain (it is no longer more accurate than the predicted mean -- it has predicted itself into the crowd)
- a neuron that sets valence contrarian without genuine private signal loses prediction accuracy (the market does not move where it predicted)
- the only robust strategy is accurate reporting of both first-order belief (link + stake) and meta-belief (valence)

The mechanism extracts private signals even when those signals are wrong, because honest errors are distributed around reality while dishonest reports are biased in self-serving directions. The aggregate of honest-but-imperfect signals converges toward truth faster than any aggregate of strategic-but-precise signals.

---

## the compounding mechanism

Honesty compounds through [[karma]]. Each accurate BTS prediction adds to the neuron's accumulated score. High karma means the network has observed a track record of genuine private signals.

A neuron that consistently lies accumulates negative karma. Its future [[cyberlinks]] carry diminished weight in the [[tri-kernel]], regardless of stake. Epistemic dishonesty is therefore economically self-defeating in expectation: the mechanism does not punish dishonesty in a single round (a lie can go undetected once), but it punishes it in expectation across rounds, because the honest strategy dominates the dishonest one in expected score.

Consistently right before the crowd → high karma → more adjacency weight per link → more reward per contribution → more resources to stake on the next correct insight.

---

## honesty as the foundation of syntropy

The [[cybergraph]]'s information measure -- [[syntropy]] $J(\pi^*) = D_{KL}(\pi^* \| u)$ -- is produced entirely by the aggregate of honest epistemic acts. Each honest cyberlink is a bit of genuine signal. The tri-kernel converts honest signals into a sharper $\pi^*$. Dishonest links move $\pi^*$ toward noise, lowering syntropy.

A maximally honest graph is a maximally syntropy-generating machine. Honesty is the fuel.

see [[epistemic-markets]] for the ICBS market mechanism. see [[rewards]] for the reward functions. see [[knowledge-economy]] for the full economic design. see [[veritas]] for the continuous temporal extension of BTS. see [[honest majority assumption]] for the protocol-level complement.
