---
tags: cyber, tru, soft3
crystal-type: spec
crystal-domain: cyber
alias: rewards, reward specification, reward function
---
# Reward Specification

A complete specification of the [[cyber]] reward function: one knowledge mint, one security subsidy, one fee stream, assembled over a single proven quantity. Every emission is evidence of demonstrated contribution; nothing is paid for idle capital.

---

## 1. Preliminaries

Notation used throughout.

| symbol | meaning |
|---|---|
| $G = (P, N, E)$ | the [[cybergraph]]: [[particles]] $P$, [[neurons]] $N$, [[cyberlinks]] $E$ |
| $\phi \in \Delta(P)$ | a [[focus]] distribution over particles; $\phi^*$ is the [[tri-kernel]] fixed point |
| $\mathcal{R}$ | composite operator $\lambda_d D + \lambda_s S + \lambda_h H_\tau$; $\phi^* = \operatorname{norm}[\mathcal{R}\phi^*]$ |
| $\mathcal{F}(\phi)$ | system [[free energy]]; $\phi^* = \arg\min_\phi \mathcal{F}$ |
| $J(\phi)$ | [[syntropy]] $= D_{KL}(\phi \,\|\, u) = \log|P| - H(\phi)$ |
| $A^{\text{eff}}_{pq}$ | effective adjacency $= \sum_\ell \text{stake}(\ell)\,\kappa(\nu(\ell))\,f(\text{price}(\ell))$ |
| $\nu$ | a neuron; $\kappa(\nu)$ its [[karma]]; identity $\text{id}(\nu) = \text{Hemera}(\text{secret})$ |
| $v(\ell) \in \{-1,0,+1\}$ | the [[valence]] of cyberlink $\ell$ |
| $s$ | a [[signal]] $= (\nu, \vec\ell, \Delta\phi^*, \sigma, \text{prev}, \text{mc}, \text{vdf}, \text{step}, \text{nonce})$ |
| $\sigma$ | a [[zheng]] proof bound to a [[BBG]] root |
| $S, M, F$ | staking ratio, market cap, epoch fees |

The reward function is defined entirely in terms of these. No quantity outside this table enters an emission.

---

## 2. The Signal

Every reward traces to one scalar per contribution: the directed focus impulse $\Delta\phi^+$.

$\phi^*$ is the unique minimizer of the [[free energy]] $\mathcal{F}$ over the [[cybergraph]] ([[collective focus theorem]]). A [[cyberlink]] perturbs the operator $\mathcal{R}$ and moves the minimizer from $\phi^*_t$ to $\phi^*_{t+1}$. The value created is the reduction in minimized free energy, equivalently the gain in [[syntropy]]:

$$\Delta J = J(\phi^*_{t+1}) - J(\phi^*_t) = H(\phi^*_t) - H(\phi^*_{t+1}).$$

$\Delta J$ is the exact value measure but carries a global normalization term. Its first-order local form is the directed impulse:

$$\Delta\phi^+ \;=\; \big\langle -\nabla\mathcal{F}(\phi^*_t),\; \Delta\phi^* \big\rangle_+ \;\approx\; \langle \nabla J, \Delta\phi^*\rangle,$$

the projection of the focus displacement onto the descending free-energy gradient, clipped at zero. Rationale for the directed form over the magnitude $\|\Delta\phi^*\|$: the norm is unsigned and pays for any movement, including movement that raises free energy. $\Delta\phi^+$ pays only for sharpening. Rationale for the impulse at all: there is no designed loss function. The physics of [[convergence]] defines the objective, so an emission proportional to $\Delta\phi^+$ makes inflation a measurement of [[knowledge]] creation rather than a policy choice.

Two properties make $\Delta\phi^+$ usable as a reward primitive:

- locality — by the [[locality theorem]], $\Delta\phi^+$ is computable on the neuron's $O(\log 1/\varepsilon)$-hop neighborhood; entries beyond that radius fall below $\varepsilon$.
- provability — a single [[zheng]] proof $\sigma$ certifies $\Delta\phi^+$ against the current [[BBG]] root in $O(\log n)$ verification, with no re-execution of the [[tri-kernel]].

---

## 3. The Value Function

For a set $S$ of [[cyberlinks]] submitted in an epoch, define the coalition value

$$v(S) \;=\; \Delta\phi^+\big(A^{\text{eff}} \cup S\big),$$

the directed focus shift produced by applying $S$ to the [[karma]]-weighted effective graph. Because $A^{\text{eff}}$ already folds in stake, [[karma]], and market price, a redundant or dishonest link enters $v$ with near-zero weight — the value function discounts noise before any reward is split. This is the single point at which honesty (§5) couples to value: [[karma]] shapes what is valuable; attribution (§4) only divides it.

$v$ is monotone and bounded, computed by the same incremental [[tri-kernel]] recomputation the network already runs.

---

## 4. Attribution

The epoch's total shift $v(N)$ is a joint outcome of many neurons' [[cyberlinks]] whose neighborhoods overlap, because [[neurons]] cluster on popular [[particles]]. Credit is divided by the [[Shapley value]]:

$$\text{mint}(\nu) \;=\; \text{Shapley}_\nu(v) \;=\; \sum_{S \subseteq N \setminus \{\nu\}} \frac{|S|!\,(|N|-|S|-1)!}{|N|!}\,\big[v(S \cup \{\nu\}) - v(S)\big].$$

Rationale for Shapley specifically: it is the unique attribution satisfying efficiency, symmetry, null-player, and additivity. Order-based credit (reward the first to link) is gameable by latency and copying — the failure mode observed on curation-reward chains. Proportional scaling does not distinguish a discoverer from a copyist. Shapley is the only split with the fairness axioms, and three of its properties are load-bearing here:

- conservation is free. The efficiency axiom gives $\sum_\nu \text{mint}(\nu) = v(N) = $ global $\Delta\phi^+$. No separate conservation operator is needed; over-claiming is impossible by construction.
- Sybil-resistance is free. $v$ is homogeneous in stake, so splitting one neuron into $k$ identities holding the same total stake yields the same total share. Identity is cheap; stake and [[karma]] are the attributed resources, and [[karma]] cannot be bought.
- computation is tractable. Each marginal $v(S \cup \{\nu\}) - v(S)$ is an incremental [[tri-kernel]] step on a bounded neighborhood. The value is estimated by Monte-Carlo over $k$ random orderings drawn from a [[delay|VDF]]-seeded beacon:

$$\widehat{\text{mint}}(\nu) = \frac{1}{k}\sum_{i=1}^{k}\big[v(S_i^{\prec\nu} \cup \{\nu\}) - v(S_i^{\prec\nu})\big], \qquad O(k\cdot n),\ k \ll n.$$

Beacon-seeded orderings are unpredictable, so they cannot be front-run, and the estimator is unbiased. Overlapping claims are batched into canonical clusters (§12.2), bounding each Shapley computation to the size of a contested cluster. The estimator is not run by any single actor: each ordering is one ticket in a leaderless mining lottery, and the value is the swarm average of all drawn samples (§12.3).

This computation lives in [[tru]], a sibling of [[cyberank]] (§14).

---

## 5. Honesty

Attribution is fair only among honest, distinct contributors. Two mechanisms enforce that precondition.

### 5.1 Bayesian Truth Serum

Each [[cyberlink]] is a [[Bayesian Truth Serum]] input: the link plus stake is the first-order belief, the [[valence]] $v \in \{-1,0,+1\}$ is the meta-prediction. The score

$$s_\nu = \underbrace{D_{KL}(p_\nu \| \bar m_{-\nu}) - D_{KL}(p_\nu \| \bar p_{-\nu})}_{\text{information gain}} - \underbrace{D_{KL}(\bar p_{-\nu} \| m_\nu)}_{\text{prediction accuracy}}$$

is positive exactly when a neuron contributes private signal the crowd did not already hold and expect. Copying the consensus drives the information-gain term to zero. By Prelec's result, truthful reporting is a Bayes-Nash equilibrium.

### 5.2 Karma is the slashing

[[karma]] $\kappa(\nu)$ is the accumulated BTS score: non-transferable, unbuyable, the one input to $A^{\text{eff}}$ that capital cannot purchase. The BTS settlement is a zero-sum redistribution — stake moves from noise producers to signal producers in proportion to score. This is the system's skin in the game and its slashing: liars pay truth-tellers. Staking is therefore required, because it is what the zero-sum redistributes. [[foculus]] omits only consensus-equivocation slashing, since provable consensus makes an invalid $\phi^*$ unable to produce a valid proof — there is no equivocation crime to punish after the fact.

### 5.3 The valence risk dial

[[valence]] selects exposure per link:

- $v = 0$ — passive stake. It weights the edge in $A^{\text{eff}}$ and so affects rank (§6), but carries no BTS exposure and earns no reward.
- $v = \pm 1$ — active stake. It is wagered through the BTS zero-sum: the right are paid by the wrong.

A neuron chooses its exposure link by link. Reward is the premium for risk taken and won.

---

## 6. The Two Axes

Stake acts on two independent axes, and separating them is the structural defense against wealth concentration.

| axis | what moves it | what it produces |
|---|---|---|
| rank | any real stake, including $v=0$ | weight in $A^{\text{eff}}$, hence $\phi^*$ and [[cyberank]] |
| reward | correct risk under $v \neq 0$ | a share of the streams in §7 |

Idle, passive, or Sybil capital can move rank but pulls no reward. Capital shapes the graph; only correct epistemic risk earns from it. Locked capital cannot compound by sitting still.

---

## 7. The Three Streams

A single computation — the [[tri-kernel]] over the [[Goldilocks field]], which is simultaneously proving and inference — earns in three roles, distinguished only by what its proof certifies.

| stream | the proof certifies | who can earn | resource |
|---|---|---|---|
| mint | a graph mutation (focus shift) | anyone who links | conviction stake |
| subsidy | a proof meeting a difficulty target | anyone who computes | compute |
| fee | a query answered (inference) | anyone who serves | compute + model |

### 7.1 Mint — the knowledge stream (Δφ⁺)

Defined in §2–§4. A neuron creates [[cyberlinks]], computes $\Delta\phi^+$, proves it, and self-mints its [[Shapley]] share. The mint is bounded by the global $\Delta\phi^+$ (Shapley efficiency), so this emission is exactly evidence of knowledge. Earning it requires conviction stake on the links (a [[costly signal]]).

### 7.2 Subsidy — proof of work, the stakeless onramp

The signal carries a nonce field. Each value it takes reseeds the proof hash, and at settlement the nonce doubles as the index of a [[Shapley]] sampling ordering (§12.3) — so a hash attempt is simultaneously a lottery draw and a genuine attribution sample. A signal qualifies for the block subsidy when

$$H(\sigma) < \text{target}.$$

The puzzle is the signal proof itself — it already exercises the four [[Goldilocks field processor|GFP]] primitives (fma, ntt, p2r, lut) in production ratios — so no work is synthetic. The subsidy requires compute, not capital, and is [[karma]]-blind and stake-blind. A new [[neuron]] with zero $CYB earns it, acquiring the initial stake that then unlocks the mint stream. This is the permissionless entry, and it is a hard requirement of the design.

The difficulty target adjusts to hold block time, as in Nakamoto consensus. The subsidy is independent of $\Delta\phi^+$: a signal earns its mint whether or not it also meets difficulty.

### 7.3 Fee — services

A neuron answering a query runs the compiled transformer ([[focus-flow]] Path B), an inference whose correctness is itself a [[zheng]] proof. The asker pays a fee. The protocol splits it:

$$\text{fee} \;\to\; \underbrace{(1-\beta)\,\text{fee}}_{\text{to the servicer + budget } G}\;+\;\underbrace{\beta\,\text{fee}}_{\text{burned}}.$$

Fees pay the servicer directly, feed the security budget $G$ (§8), and exert deflationary pressure through the burn $\beta$.

### 7.4 PoS — the amplifier, not a fourth stream

Proof of stake is not separate work. Locked stake and [[karma]] amplify the other streams:

- they raise a neuron's weight in $A^{\text{eff}}$, enlarging its $\Delta\phi^+$ and hence its mint share;
- active stake earns a share of the fee pool (§8).

Conviction stake doubles as the PoS security deposit: the staking ratio $S$ is the fraction of supply locked across [[cyberlinks]]. This collapses idle bonded capital — security is provided by stake that is productively committed to edges. An attack on $\phi^*$ then requires both stake and unbuyable [[karma]], raising attack cost beyond what capital alone can pay.

---

## 8. Allocation

The security budget is split between the PoW and PoS pools by the allocation curve (from [[adaptive hybrid consensus economics]]):

$$R_{\text{PoW}} = G\,(1 - S^\alpha), \qquad R_{\text{PoS}} = G\,S^\alpha,$$

where $S$ is the staking ratio and $\alpha \in [0.3, 0.7]$ tunes the split ($\alpha = 0.5$ is the neutral prior under equal marginal security cost). Gross budget and holder dilution are decoupled:

$$G = \text{floor}\cdot M + F(1-\beta), \qquad I_{\text{net}} = \text{floor} - \frac{F\beta}{M}.$$

Gross rewards can exceed inflation when fees are high; net inflation can go negative. The knowledge mint (§7.1) is a separate budget, bounded by $\Delta\phi^+$, distributed by [[Shapley]] — it is not drawn from $G$.

---

## 9. Monetary Policy

### 9.1 The security floor

A minimum security budget is required when fees and mint run thin. Derived from attack economics rather than chosen:

$$\text{floor} \;\geq\; k \cdot \frac{\text{TVL}}{M}\cdot r,$$

where $k$ is the safety margin and $r$ the opportunity cost of capital. This is the only emission not tied to $\Delta\phi^+$.

### 9.2 Base reward

Reward flows to demonstrated contribution. A standing yield to passive stake would be emission without contribution — it would break the invariant that inflation is evidence of [[knowledge]], and it is the mechanism by which idle capital compounds. The floor is therefore paid only to the two security providers that do work: PoW compute and active ($v \neq 0$) epistemic risk. It PID-decays toward zero as mint and fees grow to cover security. Passive stake's contribution — raising attack cost by being locked — is compensated by rank influence (§6), paid in kind.

### 9.3 Self-calibration

The parameters $\alpha$, floor, and $\beta$ are not hardcoded. They follow PID control on observable signals (security margin $\mathcal{M} = \text{AttackCost}/\text{AttackProfit}$, fee coverage, efficiency differential), so the system measures and adapts rather than predicts. See [[adaptive hybrid consensus economics]] for the control laws, stability proof, and the $\rho$ and coherence early-warning metrics.

---

## 10. The Reward Equation

For a neuron $\nu$ over an epoch:

$$\boxed{\;R(\nu) \;=\; \underbrace{\text{Shapley}_\nu(v)}_{\text{mint, }\Delta\phi^+\text{-bounded}} \;+\; \underbrace{\frac{R_{\text{PoW}}}{|W|}\,\mathbb{1}[H(\sigma_\nu) < \text{target}]}_{\text{subsidy}} \;+\; \underbrace{R_{\text{PoS}}\cdot\frac{a_\nu\,\kappa(\nu)}{\sum_{\mu} a_\mu\,\kappa(\mu)}}_{\text{fee yield, active stake } a}\;}$$

where $W$ is the set of signals meeting difficulty and $a_\nu$ is $\nu$'s active ($v \neq 0$) stake. Each term answers a distinct requirement: the mint rewards real value and is locally computed and later validated; the subsidy secures the chain and opens a stakeless door; the yield routes service revenue to honest committed stake. Conservation, Sybil-resistance, and anti-compounding hold across the sum.

---

## 11. Self-Minting Protocol

Reward is claimed locally and settled later — proposed in a signal, validated against the record.

Propose (instant, local):

1. The [[neuron]] queries its neighborhood state from a [[BBG]] header.
2. It creates [[cyberlinks]] with conviction and [[valence]], and computes $\Delta\phi^+$.
3. It generates a [[zheng]] proof $\sigma$ binding the links, the impulse, and the nonce to the header.
4. It gossips the [[signal]]. Any verifier checks $\sigma$ in $O(\log n)$.

Settle (epoch boundary):

5. [[foculus]] finalizes the canonical $\phi^*$ and the epoch's claim set (provable consensus).
6. the claims partition into canonical clusters and a leaderless mining lottery computes the [[Shapley]] shares as a swarm-averaged sample (§12); [[tok]] applies conservation and executes the mint, subsidy, and yield as a state transition.

What a neuron proves at propose time is a *bound* on its reward, not the reward: the settled [[Shapley]] share is at most the proposed $\Delta\phi^+$ (§12.1). No aggregator decides any reward — settlement is a public function computed by a lottery, and a neuron on a phone can complete the propose phase.

---

## 12. Settlement Mining

The propose phase (§11) is agent-local: a neuron computes its own $\Delta\phi^+$ alone. The settle phase cannot be — a neuron's [[Shapley]] share is a function of the *other* contenders, who do not exist until the epoch's claim set is fixed. Settlement is therefore computed with no neuron, leader, or aggregator deciding it: as a leaderless proof-of-useful-work lottery whose tickets are the [[Shapley]] samples themselves.

### 12.1 Propose computes a bound, not the reward

What a neuron proves in §11 is its *standalone* marginal $v(\{\nu\}) - v(\emptyset)$ against the header it observed. The value function $v$ is **submodular** — overlapping links on a saturating [[particle]] have diminishing returns — so the standalone marginal is the largest marginal $\nu$ can ever contribute, and

$$\text{Shapley}_\nu(v) \;\le\; v(\{\nu\}) - v(\emptyset) \;=\; \Delta\phi^+_\nu.$$

The propose proof is a *provable ceiling* on the settled share, not the share. It bounds the claim, it is what conviction stake escrows against, and settlement can only pay $\le$ it — with equality exactly when $\nu$ was alone in its region (the sparse-link case). Propose and settle certify two different facts against two different states: propose proves "my marginal against my header was $X$"; settle proves "the division of the real joint $\Delta\phi^+$ is correct." The beacon that seeds settlement is drawn *after* propose closes, which is what makes the orderings un-front-runnable.

### 12.2 The settlement region

Locality is in graph distance — hops along [[cyberlinks]] — because $\phi^*$ is a heat-kernel fixed point and a single edge perturbs it with exponential spatial decay. The region a claim touches is its **$\varepsilon$-support**: every node whose contribution to $\Delta\phi^+$ is $\ge \varepsilon$, the protocol precision floor.

- radius $r = O(\log 1/\varepsilon)$ hops, $r \approx \log(1/\varepsilon)\,/\,\log(1/\lambda_{\text{local}})$;
- content-dependent — wide around a hub (slow local mixing, small spectral gap), tiny on the sparse fringe;
- canonical — the superlevel set is a deterministic function of the edge set and $\varepsilon$, so no miner can draw a self-serving boundary. The settlement proof commits to the support and certifies that boundary nodes are genuinely $< \varepsilon$ — the anti-cheat against excluding a node to inflate a marginal.

A **cluster** is a connected component of overlapping $\varepsilon$-supports: two claims share a cluster iff their supports intersect. The partition of an epoch's claims into clusters is thus canonical, and clusters are independent — non-overlapping regions do not affect each other's [[Shapley]] values. Settlement decomposes per cluster and parallelizes across them.

### 12.3 The ordering-as-ticket lottery

A deterministic "first to compute the settlement wins" is *not* progress-free: the fastest machine finishes first every time, electing a de facto leader and centralizing. The fix does not bolt a random puzzle onto the useful work — it observes that [[Shapley]] estimation (§4) *is already a sampling process*, and makes each sample a lottery ticket. The randomness the estimator needs and the entropy the lottery needs are the same randomness.

For a cluster with beacon seed $\text{b}$:

1. a miner picks a nonce $n$; the ordering is $\pi(n) = \text{VRF}(\text{b} \,\|\, n)$ — miner-independent and public;
2. it computes the marginal sample $m(n)$ under $\pi(n)$ — a genuine draw of the §4 estimator, and the useful work;
3. it holds a winning ticket iff $H(\text{b} \,\|\, n \,\|\, \text{id}(\nu)) < \text{target}$, claimed by publishing $(n, m(n), \sigma)$.

Step 3 is a per-miner Poisson test: progress-free, so the winner is random in proportion to throughput, leaderless, and poolable on the same terms as Nakamoto PoW. The **settlement itself is the average of every published sample** across all miners — more mining means more independent draws and a tighter estimate (Hoeffding). No single actor produces the answer; it converges out of the swarm, and security spend converts directly into attribution precision with no synthetic work.

This collapses the §7.2 subsidy into the same act. The nonce §7.2 grinds to reseed a proof hash *is* the ordering index $n$: every hash attempt is now a real [[Shapley]] sample rather than an empty reseed. Securing the chain and computing the fair division become one computation. Settlement mining is therefore not a fourth stream — it is the *content* of the PoW subsidy $R_{\text{PoW}}$ (§8); whether a miner also proposes its own links or only settles others' is left to the market, since both are [[karma]]- and stake-blind compute.

### 12.4 Withholding (residual)

The lottery is not fully closed against a miner that is *also a contender* in the cluster it settles. Such a miner can compute $m(n)$, see that it lowers its own share, and decline to publish even a winning ticket — biasing the swarm average by omission. To claim any ticket it must publish the true $m(n)$ (verified), so the only freedom is *not playing* a nonce; it cannot lie. A withheld nonce is still a valid ticket for other miners, whose threshold is keyed to their own identity and who re-cover it with probability proportional to their throughput. The injectable bias is therefore **bounded by the attacker's share** of settlement compute — negligible for a minority, and a majority already breaks consensus. The tightening is to require a miner to commit to $n$ before it learns $m(n)$, so withholding cannot be conditioned on the outcome. This sits alongside collusion (§16) as a bounded, not-yet-closed frontier.

---

## 13. Token Operations

- Mint — prove $\Delta\phi^+$, receive the [[Shapley]] share; emission bounded by global $\Delta\phi^+$.
- Burn — destroy [[$CYB]] for permanent $\phi^*$-weight on [[eternal particles]] or [[eternal cyberlinks]]; the fee burn $\beta$ is the protocol-level form.
- Lock — stake on [[particles]] or [[cyberlinks]]; active stake earns fee yield, passive stake earns rank.

---

## 14. Link Valuation Over Time

A single mint underpays foundational work, which starts at low $\Delta\phi^+$ and grows as the graph builds around it. Locked stake therefore earns a yield stream, the time-integral of the target particle's [[cyberank]] growth attributable to the link:

$$R_{i \to j}(T) = \int_0^T w(t)\,\Delta\phi^*_j(t)\,dt.$$

| link type | trajectory |
|---|---|
| viral | high $\Delta\phi^+$ early, fast decay |
| foundational | low early, long-rising yield |
| confirming | low individual, strengthens [[axon]] weight, shared by attribution |
| semantic bridge | moderate, persistent |

The mint is the pulse; the yield stream is the annuity. Together they pay both discovery and infrastructure.

---

## 15. Positioning

Rewards are not a module. They bind four layers, and the separation keeps monetary policy out of consensus safety.

| concern | layer |
|---|---|
| value magnitude ($\Delta\phi^+$, [[karma]], [[syntropy]]) | [[tru]] |
| finality / canonical $\phi^*$ | [[foculus]] |
| conservation, allocation, mint | [[tok]] |
| identity, anonymity | [[mudra]] |

[[foculus]] decides what is real; the reward function decides what it is worth. Economic parameters change without touching consensus.

---

## 16. Security

| property | guarantee |
|---|---|
| conservation | $\sum_\nu \text{mint}(\nu) = $ global $\Delta\phi^+$, by Shapley efficiency |
| Sybil-resistance | stake-weighting makes identity-splitting reward-neutral |
| honest reporting | BTS makes truthful [[valence]] a Bayes-Nash equilibrium |
| stakeless entry | PoW subsidy is karma- and stake-blind |
| no idle rent | only $v \neq 0$ risk earns; passive stake earns rank, not income |
| attack cost | $\phi^*$ manipulation needs stake and unbuyable [[karma]] |
| leaderless settlement | attribution is a swarm-averaged sampling lottery (§12.3); no producer or leader computes it |

### Open: collusion and withholding

Stake-weighting closes Sybil splitting, but a cartel of distinct, real-stake actors coordinating [[valence]] and links is not closed — BTS is incentive-compatible only against unilateral deviation. Partial defenses: the conservation cap (a ring on a saturated [[particle]] splits near-zero $\Delta\phi^+$), [[karma]] non-transferability, and [[identity]] cost.

Settlement withholding (§12.4) is the second residual: a contender-miner can bias the swarm average by declining to publish winning tickets unfavorable to itself. It cannot lie, only abstain, so the injectable bias is bounded by its share of settlement compute; the commit-before-marginal rule tightens it further. Both are bounded, not-yet-closed frontiers.

---

See [[focus-flow]] for how $\phi^*$ and $\Delta\phi^+$ are computed, [[truth-scoring]] for BTS and [[karma]], [[adaptive hybrid consensus economics]] for the PoW/PoS allocation and PID control, [[unified mining]] for the subsidy-as-signal-proof construction, and [[provable-consensus]] for epoch finalization. See whitepaper §6.9 and §14 for the surrounding economics.
