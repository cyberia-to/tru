---
tags: cyber, docs
alias: valence framework, epistemic accounting, debit credit cybergraph, valence explained
crystal-type: article
crystal-domain: cyber
---
# valence

valence is $v \in \{-1, 0, +1\}$ — the fifth field of a [[cyberlink]]. it is simultaneously an accounting primitive (debit/credit/hold), an epistemic prediction (challenge/void/affirm), and the seed that initializes the ICBS market on a link. these are not different interpretations. they are the same thing seen from three angles.

---

## binary topology, ternary economics

every known system that produces collective intelligence uses the same two-layer architecture: binary topology, ternary economics. mycelium, neurons, markets, ecosystems — all of them.

the binary layer answers "with whom?" — a synapse either exists or doesn't, a hypha either connects two trees or doesn't, a cyberlink either exists or doesn't. connection topology is always discrete.

the ternary layer answers "how?" — through an existing connection, flow is: give (+1), receive (−1), or maintain the channel with no net flow (0). the neutral state is not the absence of a connection. it is active standby: a different kind of nothing than no-edge.

the separation of these two questions is not a modeling choice. it is a fundamental property of efficient distributed computation. binary and ternary are irreducible — $2^m \neq 3^n$ for any natural numbers — so neither subsumes the other. any system that conflates them into one layer loses computational power.

| system | binary (topology) | ternary (economics) |
|---|---|---|
| mycelium | hypha exists / doesn't | give / maintain / receive |
| neuron | synapse exists / doesn't | excitatory / modulatory / inhibitory |
| market | counterparty relationship | buy / hold / sell |
| [[cybergraph]] | [[cyberlink]] exists / doesn't | affirm / void / challenge |

the cyberlink structure encodes all three layers of the pattern in one atomic record:

| layer | field | type |
|---|---|---|
| structural | $(p, q)$ | binary |
| economic | $(\tau, a)$ | continuous |
| epistemic | $v$ | ternary |

---

## the accounting frame

every cyberlink is a double-entry journal entry. the two accounts are already present: the source particle $p$ and the target particle $q$.

| $v$ | accounting | epistemic |
|---|---|---|
| $+1$ | credit $q$, debit $p$ | affirm — I predict the market converges TRUE |
| $-1$ | debit $q$, credit $p$ | challenge — I predict the market converges FALSE |
| $0$ | open account, no entry | void — this path exists; no directional bet posted |

the source particle always posts to one side; the target always posts to the other. the link is the transaction. this gives the system a conservation law: total epistemic assets equal total epistemic liabilities across the whole graph. $\sum_p \phi^*(p) = 1$ is not just a normalization constraint — it is the balance sheet identity.

[[semcons]] are the chart of accounts. `cites`, `contradicts`, `is-a`, `extends` are account types with different debit/credit semantics. `contradicts` edges always debit the target. `cites` edges typically credit. the [[knowledge graph]] partitioned by semcon is a set of sub-ledgers, each recording a different kind of epistemic transaction.

---

## what each value does

### v = +1 — affirm

the neuron credits the target particle's truth account and debits its own epistemic capital. it is simultaneously placing a bet: "I predict this market converges toward TRUE."

in the [[inversely coupled bonding surface|ICBS]] market, the affirm signal pushes the market price $m(\ell)$ toward 1. if many neurons affirm, the price rises, and effective adjacency $A^{\text{eff}}_{pq}$ grows. the link gains structural weight in the graph. focus accumulates at the target.

### v = -1 — challenge

the neuron debits the target particle's truth account. the bet: "I predict this market converges toward FALSE."

negative valence is not contradiction of the link's existence — the neuron has still created the link, paying stake. the neuron is asserting: "this connection exists and is worth marking, but the collective will judge it false." this is rational when the neuron knows something the market has not yet priced.

in effective adjacency, dominant negative valence drives $m(\ell) \to 0$, suppressing the link's weight toward zero. this is [[market inhibition]] — the epistemic immune system of the [[cybergraph]]. without inhibition, the graph cannot learn a boundary; it can only accumulate. excitation without inhibition produces a blob, not a mind.

### v = 0 — void

not agnostic in a weak sense. void is the structural primitive that predates accounting. it opens the account relationship on both sides without posting any transactions.

a void-valence link costs [[will]] to create — it is still a costly signal — but the signal is "this path exists" rather than "this path is true or false." the channel is open. signaling molecules flow through neutral synapses. the market on this link initializes at 50/50 with no directional pressure.

void-valence links serve as scaffolding: structural relationships established before the community has formed a view. as evidence accumulates and neurons post affirm or challenge links, the market price moves away from 0.5. the void link becomes the substrate on which the epistemic process runs.

---

## Bayesian Truth Serum

valence IS the [[Bayesian Truth Serum]] meta-prediction. no separate submission step is needed.

| BTS concept | cyberlink field |
|---|---|
| first-order belief $p_i$ | link creation + stake $(\tau, a)$ |
| meta-prediction $m_i$ | valence $v$ |
| agent identity | signing [[neuron]] $\nu$ |

the BTS score for neuron $i$:

$$s_i = D_{KL}(p_i \| \bar{m}_{-i}) - D_{KL}(p_i \| \bar{p}_{-i}) - D_{KL}(\bar{p}_{-i} \| m_i)$$

Prelec (2004) proved truthful reporting is a Bayes-Nash equilibrium: no neuron improves their expected score by misreporting either belief or meta-belief. the optimal strategy is to post what you actually believe and predict what you actually expect the collective to conclude. gaming the system produces lower expected karma than honesty.

---

## karma as the audit trail

the accounting cycle closes through karma:

| accounting step | cybergraph equivalent |
|---|---|
| journal entry | cyberlink posted with valence |
| sub-ledger update | ICBS price adjusts |
| trial balance | tri-kernel → $\phi^*$ |
| reconciliation | BTS score → karma update |

karma is the accumulated record of whether your posted entries were confirmed by market settlement. consistent accurate posting builds karma; bad entries are written off. high karma amplifies future link weight through $\kappa(\nu)$ in effective adjacency, so the audit trail compounds: honest bookkeeping earns authority to post larger entries.

the compounding loop: honest valence → correct BTS prediction → karma accrues → higher $\kappa(\nu)$ → more effective weight per link → more $\phi^*$ shift per contribution → higher [[CYB]] reward.

---

## the full truth stack

$$\text{binary} \;\to\; \text{ternary} \;\to\; \text{continuous} \;\to\; \phi^*$$

| layer | what | type |
|---|---|---|
| structural | link exists / doesn't | binary |
| epistemic seed | $v \in \{-1, 0, +1\}$ | ternary |
| market | ICBS price $m(\ell) \in (0,1)$ | continuous |
| field | focus distribution $\phi^*$ | convergent |

each layer requires the one below it. the market cannot initialize without the ternary seed. $\phi^*$ cannot converge without the market weights. the ternary is not redundant with the continuous layer — it is the discrete prior that the continuous layer refines.

---

## implications for CT-0

the bivector grade of effective adjacency in [[CT-0]] is the accumulated signed valence consensus:

$$A^{\text{eff}}_2(p, q) = f_2(m) \cdot \sum_\ell a(\ell) \cdot v(\ell) \cdot (e_p \wedge e_q)$$

where $f_2(m) = |2m - 1|$ is zero at maximum market uncertainty and one at full confidence. the bivector grade is the orientation of collective agreement: when neurons agree on valence, it accumulates; when they disagree, the opposing entries cancel. this is the double-entry property operating at the graph level — conflicting journal entries net to zero.

the bivector feeds into the wedge-augmented attention score (§7.7) and the Clifford-block MLP (§8). when all bivector grades are zero — either because all links are void-valence, or because affirm and challenge have perfectly cancelled — every Clifford term vanishes and CT-0 output is byte-identical to a scalar compile.

---

see [[valence]] for the field specification. see [[cyberlink]] for the full 5-tuple. see [[Bayesian Truth Serum]] for the scoring mechanism. see [[inversely coupled bonding surface]] for market mechanics. see [[binary topology ternary economics]] for the architectural principle. see [[CT-0]] for the multivector compile.
