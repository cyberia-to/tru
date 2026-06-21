---
tags: cyber, tru, core, spec
crystal-type: spec
crystal-domain: cyber
alias: valence, valences, epistemic valence, link valence, v field, ternary signal
---
# valence

the ternary epistemic field of a [[cyberlink]]: $v \in \{-1,\,0,\,+1\}$. [[cybergraph]] carries the field; [[tru]] runs the dynamics it seeds. valence is the [[neuron]]'s meta-prediction, fixed at link creation, of where the [[inversely coupled bonding surface|ICBS]] market on that edge will settle — the coarse, human-readable quantization of belief before the market produces a continuous probability.

| value | name | meaning |
|-------|------|---------|
| $+1$ | affirm | the neuron predicts the ICBS market will converge toward TRUE |
| $\phantom{+}0$ | void | the link exists; no epistemic bet is posted — the channel is held open |
| $-1$ | challenge | the neuron predicts the market will converge toward FALSE |

## the third layer

valence is the epistemic dimension of the cyberlink, orthogonal to the other two:

| layer | field(s) | type | what it encodes |
|-------|----------|------|-----------------|
| structural | $(\text{from}, \text{to})$ | binary | connection exists or does not |
| economic | [[box]] $(\tau, a)$ | continuous | conviction depth |
| epistemic | $v$ | ternary | prediction of collective judgment |

$v = -1$ is not contradiction. a neuron can assert a structural connection while predicting the network will judge the link false — rational when the neuron knows something the market has not yet priced, or deliberately adds anti-[[knowledge]] for others to refute. [[Bayesian Truth Serum|BTS]] rewards this exactly when the prediction proves correct.

## binary topology, ternary economics

valence is the ternary economic layer over a binary topology — the same architecture mycorrhizal networks, neural synapses, and markets all run:

| domain | $+1$ | $0$ | $-1$ |
|--------|------|-----|------|
| [[cybergraph]] | affirm: market → TRUE | void: no prediction | challenge: market → FALSE |
| neurobiology | excitatory synapse | neuromodulation | inhibitory synapse |
| [[mycelium]] | give resources | hold channel | receive / take |
| [[market]] | buy TRUE | hold | buy FALSE |

the zero state carries information without carrying directional belief: a void-valence link holds the structural channel open — the connection exists, the topic is raised — without forcing an epistemic commitment. for the full accounting frame of $v$ as debit / credit / hold, see [[tru/docs/explanations/valence|valence as accounting]].

## effect on focusing

valence does not enter effective adjacency directly. it seeds the market whose price $m(\ell)$ weights the edge:

$$A^{\text{eff}}_{pq} = \sum_{\substack{\ell \in L \\ \text{src}(\ell)=p,\;\text{tgt}(\ell)=q}} a(\ell)\cdot\kappa(\nu(\ell))\cdot f\big(m(\ell)\big)$$

$v$ is the input to [[Bayesian Truth Serum]] scoring — it is $m_i$ in the BTS formula. the score accumulates into [[karma]] $\kappa$ and drives the ICBS price $m(\ell)$, which $f:[0,1]\to[0,1]$ maps to an edge multiplier. edges the collective disbelieves converge to $m \approx 0$, so $f(m)\approx 0$ — suppressed in the [[tri-kernel]] without being deleted from the record. this is [[market inhibition]]: the graph-theoretic analog of inhibitory transmission. without negative valence, $A^{\text{eff}}$ is purely excitatory — the graph could reinforce but never suppress.

## connection to syntropy

valence-seeded prices, once resolved, raise [[syntropy]] when predictors are accurate — $\phi^*$ sharpens around true structure, $J(\phi^*)$ rises — and lower it when markets stay divided. a neuron whose $v$ predictions proved correct earned positive BTS score and raised the graph's order.

see [[Bayesian Truth Serum]] for the scoring formula · [[truth-scoring]] for karma accumulation · [[inversely coupled bonding surface]] for the market substrate · [[focusing]] for effective adjacency · [[syntropy]] for what accurate valence grows.

discover all [[concepts]]
