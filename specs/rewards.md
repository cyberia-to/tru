---
tags: cyber, tru, soft3
crystal-type: spec
alias: rewards, reward specification, reward function, learning incentives, learning rewards
---
# Reward Specification

One law runs the whole economy: new money is minted only when, and exactly where, [[knowledge]] is created. Knowledge here has a physical meaning — [[focus]] settling into a lower-energy, more coherent state — so inflation is the measure of a physical process. Everything below makes that one law precise: how the quantity is measured, how it is divided fairly among the [[neurons]] that produced it, how it is computed and secured without a leader, and what economy assembles around it.

The design follows from four requirements, in order of force:

1. compute locally, validate later — a [[neuron]] on a phone must be able to claim its reward from its own neighborhood, before any global agreement;
2. be fair — overlapping contributions must split by a principled rule of fair division;
3. pay for real value — emission must track demonstrated contribution, never idle capital;
4. optimize the network's own compute — the work done to earn is the work the network needs.

---

## 1. The Principle — pay for descent

[[focus]] $\phi$ is a distribution of attention over [[particles]]. Left alone on the [[cybergraph]], it flows downhill on a landscape — the [[free energy]] $\mathcal{F}$ — and settles at the unique low point $\phi^*$, the [[tri-kernel]] fixed point. A [[cyberlink]] reshapes the landscape; the focus rolls to a new resting place; the drop in free energy is the value created, equivalently the gain in [[syntropy]]:

$$\Delta J = J(\phi^*_{t+1}) - J(\phi^*_t) = H(\phi^*_t) - H(\phi^*_{t+1}).$$

There is no designed loss function, and this is the load-bearing choice, so state it exactly. A supervised loss points outside the system at a target somebody supplied ("this input should output cat"); it is one arbitrary goal per example, and it can be gamed by forging the answer key. $\mathcal{F}$ points only at the system's own internal consistency — there is no external target anywhere, so there is no answer key to design or to forge. The focus value $\phi^*$ is emergent: it falls out of the law and the data, the way a marble's resting spot falls out of the bowl, not out of anyone's wish.

One criterion is chosen, and honesty demands naming it: descent is value — focus concentrating, the graph agreeing with itself, is the thing worth paying for. This is a single uniform law, content-blind, applied identically to everything. And it is less invented than discovered — free-energy minimization is the same principle statistical inference, thermodynamics, and the Bayesian brain independently converge on. It is designed the way $F=ma$ is designed: a law one discovers, not a knob one tunes.

So inflation measures knowledge creation rather than expressing a policy. That is the whole foundation. The rest is consequence.

---

## 2. The Measure — the directed impulse

$\Delta J$ is the exact value, but it carries a global normalization term, so a neuron computes its first-order local form instead, the directed focus impulse:

$$\Delta\phi^+ \;=\; \big\langle -\nabla\mathcal{F}(\phi^*_t),\; \Delta\phi^* \big\rangle_+ \;\approx\; \langle \nabla J, \Delta\phi^*\rangle,$$

the projection of the focus displacement onto the descending free-energy gradient, clipped at zero. Two rationales:

- directed, not magnitude. The norm $\|\Delta\phi^*\|$ is unsigned — it pays for any movement, including movement that raises free energy (adding noise). $\Delta\phi^+$ pays only for the downhill component, only for sharpening.
- a gradient, not a loss. $\Delta\phi^+$ is a gradient — the slope of the intrinsic landscape of §1, the system's own energy. "No designed loss" never meant "no gradient."

Two properties make $\Delta\phi^+$ usable as the reward primitive, and they are exactly requirement 1:

- local. By the [[locality theorem]], $\Delta\phi^+$ is computable on the neuron's $O(\log 1/\varepsilon)$-hop neighborhood; the perturbation of a single edge decays exponentially with graph distance, and entries past that radius fall below $\varepsilon$.
- provable. A single [[zheng]] proof $\sigma$ certifies $\Delta\phi^+$ against the current [[BBG]] root in $O(\log n)$, with no re-execution of the [[tri-kernel]].

Everything paid by the protocol traces to this one scalar.

---

## 3. The Value Function

§2 priced one link. But [[neurons]] cluster on popular [[particles]], so two links into the same region partly create the same focus shift — value is joint, not additive. Paying each link its own $\Delta\phi^+$ would pay twice for shared work. So value is scored over sets of links.

For any coalition $S$ of an epoch's [[cyberlinks]], define the value it jointly produces:

$$v(S) \;=\; \Delta\phi^+\!\big(A^{\text{eff}} \cup S\big),$$

the directed focus shift from applying $S$ to the [[karma]]-weighted effective graph

$$A^{\text{eff}}_{pq} \;=\; \sum_\ell \text{stake}(\ell)\,\kappa(\nu(\ell))\,f(\text{price}(\ell)).$$

This is the move the whole specification turns on: $v$ makes the economy a cooperative game. As a set-function $v : 2^N \to \mathbb{R}$ it is that game's characteristic function — the worth of every possible coalition of contributors — and the reward (§4) is its fair division. Once value is a set-function, every result downstream is a property of $v$.

Three of those properties are load-bearing, each invoked by name later:

- honesty enters here, and only here. $A^{\text{eff}}$ folds in stake, [[karma]], and market price, so a low-reputation or market-doubted link joins $v$ with reduced weight. But [[karma]] is accumulated reputation and price is the standing market belief; neither sees whether the contribution at hand carried information the crowd had not already expected. Since a copy and an original produce the same $\Delta\phi^+$, Shapley's symmetry alone would split their credit equally — so the per-contribution surprise gate (§5) supplies the missing asymmetry. Karma and surprise shape what is valuable; attribution (§4) only divides it.
- submodular among substitutes, supermodular among complements. Overlapping links on a saturating particle have diminishing returns (substitutes), so within a contested cluster the standalone marginal is the largest a link contributes — the regime where the propose bound (§6) holds and settlement is an honest sampling problem (§7). But links that form a path or build on a foundation are complementary: together they shift focus more than the sum of their parts, so a foundational link's Shapley share can exceed its standalone marginal. $v$ is therefore neither globally sub- nor supermodular, and the only universal guarantee is conservation, not a per-link ceiling.
- monotone and bounded — a true contribution never lowers value and value never runs away, so the game has a well-defined, finite [[Shapley]] solution (§4).

$v$ is the same $\Delta\phi^+$ (§2) read over sets, by the incremental [[tri-kernel]] recomputation the network already runs.

For the reward, $v$ is read in its surprise-weighted form $v^\star$: each contribution is scaled by its [[Bayesian Truth Serum|BTS]] surprise $\sigma_\ell \in [0,1]$ (§5), so a copy contributes nothing and the mint divides surprising syntropy — the focus shift the crowd did not predict. Rank reads the unweighted graph (§9), so a copy's capital still ranks even as it mints nothing.

---

## 4. Fair Division — Shapley, and why it is integrated gradient

The epoch's total shift $v(N)$ must be split among contributors. The split is the [[Shapley value]]:

$$\text{mint}(\nu) \;=\; \text{Shapley}_\nu(v) \;=\; \sum_{S \subseteq N \setminus \{\nu\}} \frac{|S|!\,(|N|-|S|-1)!}{|N|!}\,\big[v(S \cup \{\nu\}) - v(S)\big].$$

Shapley is the unique attribution satisfying efficiency, symmetry, null-player, and additivity. The alternatives fail concretely: order-based credit ("reward whoever links first") is gameable by latency and copying — the curation-reward death spiral seen on social chains; proportional scaling cannot tell a discoverer from a copyist. Three of Shapley's properties are load-bearing, and each answers a requirement for free:

- conservation is the efficiency axiom. $\sum_\nu \text{mint}(\nu) = v(N) = $ global $\Delta\phi^+$. No separate conservation operator is needed; over-claiming is impossible by construction. This is requirement 3.
- Sybil-resistance is stake conservation. $\phi^*$ is the normalized fixed point, so it is invariant to scaling $A^{\text{eff}}$; the resistance therefore rests not on degree-1 homogeneity but on per-edge stake conservation. Split a neuron into $k$ identities that hold the same total stake on the same links, and $A^{\text{eff}}$ — and so every $\Delta\phi^+$ — is unchanged, while the $k$ Sybil shares sum to the one original share. Fragmenting identity multiplies nothing because stake, not identity count, is the attributed resource and cannot be fabricated by splitting. Karma compounds the defense: a fresh identity starts at zero karma, so fragmenting only dilutes the one input capital cannot buy.
- tractability is locality. Each marginal $v(S \cup \{\nu\}) - v(S)$ is an incremental [[tri-kernel]] step on a bounded neighborhood, estimated by Monte-Carlo over $k$ random orderings seeded by a [[delay|VDF]] beacon — $O(k\cdot n)$, $k \ll n$. The beacon is drawn after submission, so orderings cannot be front-run.

### The gradient correspondence

There is a reason Shapley is the right tool and not merely a fair one, and it ties this reward to how brains learn. A deep network assigns credit by the chain rule — backpropagation. The brain almost certainly does not run backprop (a synapse cannot read itself backward — the weight-transport problem), but it plausibly does gradient learning by energy-based means: predictive coding and equilibrium propagation show that the same relaxation that settles a network to its answer also computes the gradient of its energy, locally, with no separate backward pass. The objective is intrinsic free energy — exactly §1.

cyber takes the same stance, then makes one move further. Its neurons are strategic, stake-bearing agents, so credit must be fair, not merely differentiated. And the fair credit rule has a known gradient form in the continuum: the Aumann–Shapley value — Shapley's extension to a non-atomic game of infinitesimal players — is exactly the path integral of the gradient, the method machine learning calls integrated gradients. In that limit,

$$\underbrace{\text{Shapley}_\nu(v)}_{\text{discrete agents}} \quad\xrightarrow{\ \text{non-atomic limit}\ }\quad \underbrace{\int_0^1 \partial_\nu\, v(t\!\cdot\!N)\,dt}_{\text{integrated gradient of }\Delta\phi^+}.$$

The network is discrete, so this is a correspondence rather than a literal identity — the equality is exact only as the players become infinitesimal. But it is the right way to read the design: Shapley over $\Delta\phi^+$ is the strategic-agent analog of integrated-gradient credit assignment on the free energy, the same shape of computation as energy-based learning in the brain, lifted to agents who must be paid rather than merely tuned. The two strongest choices in this document — intrinsic gradient over designed loss, and Shapley over a race — rhyme for that reason.

This computation lives in [[tru]], a sibling of [[cyberank]].

---

## 5. Honesty

Shapley is fair only among honest, distinct contributors. Two mechanisms enforce that precondition; both live inside $v$ via [[karma]].

[[Bayesian Truth Serum]] scores honesty. Each [[cyberlink]] is a BTS input: the link-plus-stake is the first-order belief, the [[valence]] $v \in \{-1,0,+1\}$ is the meta-prediction. The score

$$s_\nu = \underbrace{D_{KL}(p_\nu \,\|\, \bar m_{-\nu}) - D_{KL}(p_\nu \,\|\, \bar p_{-\nu})}_{\text{information gain}} - \underbrace{D_{KL}(\bar p_{-\nu} \,\|\, m_\nu)}_{\text{prediction accuracy}}$$

is positive exactly when a neuron contributes private signal the crowd did not already hold and expect. Copying the consensus drives the information-gain term to zero. By [[Prelec's theorem]], truthful reporting is a Bayes–Nash equilibrium.

The per-contribution surprise. A single contribution's BTS score, normalized to $\sigma_\ell \in [0,1]$, measures how far its report diverged from what the crowd predicted — its information gain. It is the instantaneous counterpart of [[karma]]: karma is a contributor's accumulated surprise (the prior, its reputation), $\sigma_\ell$ is this contribution's surprise (the likelihood).

$\sigma$ and the [[inversely coupled bonding surface|ICBS]] price answer different questions, and the mint requires both. Price is a validity gate: $f(\text{price}) \in [0,1]$ weights the edge by whether the market believes the link is true, so a market-rejected link carries little weight however surprising. $\sigma$ is a novelty score: it asks whether the report beat the crowd's prediction, so a true link nobody found surprising mints nothing however large its $\Delta\phi^+$. The reward is directed syntropy that is at once valid and surprising — surprising syntropy.

The two pull against each other early, and that tension is real. A genuinely novel link has low price before the market catches on, so the validity gate under-weights it exactly when $\sigma$ is highest. That lag is the same one §12 names the discovery leak: the formula trusts the market, and the market is late. Closing it is open.

[[karma]] is the slashing. $\kappa(\nu)$ is the accumulated BTS score: non-transferable, unbuyable, the one input to $A^{\text{eff}}$ that capital cannot purchase. The BTS settlement is a zero-sum redistribution — stake moves from noise producers to signal producers in proportion to score. This is the skin in the game and the slashing: liars pay truth-tellers. Staking is therefore required, because it is what the zero-sum redistributes. [[foculus]] omits only consensus-equivocation slashing — provable consensus makes an invalid $\phi^*$ unable to produce a valid proof, so there is no equivocation crime to punish.

[[valence]] is the risk dial. Exposure is chosen per link: $v = 0$ is passive stake — it weights the edge in $A^{\text{eff}}$ and so moves rank (§9), but takes no BTS exposure and earns no reward; $v = \pm 1$ is active stake, wagered through the zero-sum. Reward is the premium for risk taken and won.

---

## 6. Propose and Settle

Requirement 1 — local now, validated later — and requirement 2 — Shapley fairness — appear to conflict: a neuron's Shapley share is a function of the other contenders, who do not exist when it acts alone. The resolution is forced. Propose computes a bound; settle computes the share. They are two phases because they must be.

### Propose — instant, agent-local

A neuron computes its own standalone marginal $\Delta\phi^+_\nu = v(\{\nu\}) - v(\emptyset)$ against the [[BBG]] header it observed, proves it with $\sigma$, and gossips the [[signal]]. The propose proof certifies one fact: the contribution's value in isolation. Among substitutes (§3) — the clustered pile-on that is the common case — this standalone marginal is the largest a link contributes, so the settled share obeys

$$\text{Shapley}_\nu(v^\star) \;\le\; \Delta\phi^+_\nu \qquad \text{(substitutes; surprise } \sigma \le 1 \text{ only tightens it).}$$

So in the saturating regime the proposed marginal is a ceiling, which is what conviction stake escrows against. Complementary contributions are the exception: a link that enables others can settle for more than it proposed, and that surplus is granted by the protocol's own Shapley computation, never claimed by the neuron — so a contributor still cannot inflate its own number, only earn what its complementarity is worth. The universal bound is conservation, not the per-link ceiling: the cluster's shares sum to its realized $\Delta\phi^+$ (§4), so no claim escapes the value that was actually created. A phone completes this phase.

### Settle — epoch boundary

[[foculus]] finalizes the canonical $\phi^*$ and the epoch's claim set; the claims partition into clusters (§7); a leaderless lottery computes the Shapley shares (§7); [[tok]] applies conservation and executes the result as a state transition. The settled reward is the surprise-weighted Shapley share:

$$R(\nu) \;=\; \text{Shapley}_\nu(v^\star), \qquad v^\star(S) = \Delta\phi^+\big(A^{\text{eff}} \cup S\big)\ \text{scaled per contribution by}\ \sigma_\ell.$$

The surprise $\sigma_\ell$ can only be fixed here, because it compares the contribution to the crowd's collected predictions — which, like the contender set, exist only once the epoch closes. So the overlap division (Shapley) and the surprise gate (BTS) finalize together. Conservation tightens to $\sum_\nu R(\nu) = v^\star(N) \le \Delta\phi^+(N)$; the slack is predictable or copied syntropy, left unminted.

The two phases certify different facts against different states: propose proves "my marginal in isolation was $X$" (a fact, ceiling among substitutes); settle proves "the division of the real joint $\Delta\phi^+$ is correct" (the share). The settlement beacon is drawn after propose closes — which is what makes the orderings un-front-runnable. The distinction that dissolves the apparent conflict: agent-local (one actor, alone — possible for the bound, impossible for the share) versus graph-local (a bounded neighborhood — true for both).

---

## 7. Settlement Mining

Settlement is computed with no neuron, leader, or aggregator deciding it. This is the document's structural core, and it satisfies requirement 4: the work that secures the chain is the work that computes the fair division.

### The region

Locality is in graph distance because $\phi^*$ is a heat-kernel fixed point with exponential spatial decay. The region a claim touches is its $\varepsilon$-support — every node whose contribution to $\Delta\phi^+$ is $\ge \varepsilon$, the protocol precision floor:

- radius $r = O(\log 1/\varepsilon)$ hops, $r \approx \log(1/\varepsilon)/\log(1/\lambda_{\text{local}})$;
- content-dependent — wide around a hub (slow local mixing), tiny on the sparse fringe;
- canonical — the superlevel set is a deterministic function of the edges and $\varepsilon$, so no miner can draw a self-serving boundary. The settlement proof commits to the support and certifies that boundary nodes fall below $\varepsilon$, the anti-cheat against excluding a node to inflate a marginal.

A cluster is a connected component of overlapping $\varepsilon$-supports. The partition of an epoch's claims into clusters is therefore canonical, and clusters are independent — non-overlapping regions leave each other's Shapley values untouched, so settlement parallelizes across them.

### The lottery

A deterministic "first to compute the settlement wins" is not progress-free — the fastest machine finishes first every time, electing a de facto leader and centralizing. The fix reuses randomness already present: Shapley estimation is a sampling process, and each sample becomes a lottery ticket — the entropy the lottery needs and the variance the estimator needs are the same randomness.

For a cluster with beacon seed $\mathrm{b}$, a miner:

1. picks a nonce $n$; the ordering is $\pi(n) = \mathrm{VRF}(\mathrm{b} \,\|\, n)$ — public and miner-independent;
2. computes the marginal sample $m(n)$ under $\pi(n)$ on the surprise-weighted value $v^\star$ (§6) — so the same draw that settles attribution also applies the BTS gate, with no separate pass; a genuine draw of the §4 estimator, and the useful work;
3. holds a winning ticket iff $H(\mathrm{b} \,\|\, n \,\|\, \mathrm{id}(\nu)) < \text{target}$, claimed by publishing $(n, m(n), \sigma)$.

Step 3 is a per-miner Poisson test: progress-free, leaderless, poolable on the same terms as Nakamoto consensus, and random in proportion to throughput. The settlement itself is the average of every published sample — more mining means more independent draws and a tighter estimate (Hoeffding). No actor produces the answer; it converges out of the swarm, and security spend converts directly into attribution precision with zero synthetic work.

This collapses the proof-of-work subsidy (§8) into the same act. The nonce a miner grinds to reseed a proof hash is the ordering index $n$ — so every hash attempt is a real Shapley sample. Securing the chain and computing the fair division become one computation; settlement mining is the content of the PoW subsidy.

Three properties of this lottery are asserted here and not yet proven. Each published sample carries its own [[zheng]] proof that $m(n)$ is a correct $v^\star$ marginal, so the proof cost per ticket must stay below the reward per ticket for mining to be rational — an economic condition, unmodeled. Progress-freedom holds per cluster but not uniformly across clusters, since a large cluster costs more per sample than a small one, so the clean Poisson picture is an approximation that the difficulty schedule has to correct for. And the analysis below covers only withholding; beacon manipulation and the verifier's aggregate cost are open. Settlement mining is the boldest construction in this document and the least adversarially tested.

### Residual: withholding

The lottery is not fully closed against a miner that is also a contender in the cluster it settles: it can compute $m(n)$, see that the sample lowers its own share, and decline to publish even a winning ticket. It cannot lie — claiming any ticket requires publishing the verified $m(n)$ — so its only freedom is to abstain from a nonce, and a withheld nonce stays a valid ticket for other miners (their threshold is keyed to their own identity), who re-cover it with probability proportional to their throughput. The injectable bias is therefore bounded by the attacker's share of settlement compute — negligible for a minority, and a majority already breaks consensus. The cheap deterrents are to price it (a withheld ticket forfeits its subsidy; calibrate so the forfeit exceeds the share-gain) and to separate roles (a miner does not settle a cluster it contends in). A commit-to-$n$-before-learning-$m(n)$ round drives the bias to zero in expectation by forcing a non-adaptive adversary, at the cost of a synchronous commit–reveal assumption foreign to the lottery; it is the escalation, not the default. This sits alongside collusion (§15) as a bounded, open frontier.

---

## 8. The Three Roles

A single computation — the [[tri-kernel]] over the [[Goldilocks field]], simultaneously learning, proving, and inference — earns in three roles, distinguished only by what its proof certifies. This is requirement 4 at the economic level: one chip, one kind of work, three economic faces. What each is paid for differs in kind — committed stake, expended work, a received fee — while the chip and the proof stay one.

| role | the proof certifies | who earns | basis |
|---|---|---|---|
| mint | a graph mutation (focus shift) | anyone who links | stake |
| subsidy | a proof meeting a difficulty target | anyone who computes | work |
| fee | a query answered (inference) | anyone who serves | fee |

### Mint — the knowledge stream

A neuron links, computes $\Delta\phi^+$, proves it, and self-mints its Shapley share, settled by the lottery and bounded by global $\Delta\phi^+$. Earning it requires conviction stake — a [[costly signal]]. This is a budget of its own, held separate from the security budget $G$ below.

### Subsidy — proof of work, the stakeless onramp

The [[signal]] carries a nonce; a signal qualifies for the block subsidy when $H(\sigma) < \text{target}$. The puzzle is the signal proof itself — it exercises the four [[Goldilocks field processor|GFP]] primitives (fma, ntt, p2r, lut) in production ratios, and at settlement the nonce is the Shapley ordering index (§7), so the work is real throughout. The subsidy is [[karma]]-blind and stake-blind: a new [[neuron]] with zero [[$CYB]] earns it and acquires the stake that unlocks the mint. The mint it then earns is karma-light at first, since karma is earned rather than granted; the sparse early graph offsets this with a large discovery premium — every link moves an uncrowded $\phi^*$ a long way — and karma accrues as those early links are validated. So the cold-start path runs subsidy → stake → discovery-premium mint → karma → amplified mint. This permissionless entry is a hard requirement. Difficulty adjusts to hold block time; the subsidy is independent of $\Delta\phi^+$.

### Fee — services

A neuron answering a query runs the compiled transformer ([[focus-flow]] Path B), an inference whose correctness is itself a [[zheng]] proof. The asker pays; the protocol splits the fee to the servicer and the budget $G$, and burns a fraction $\beta$.

### PoS — the amplifier

Proof of stake adds no fourth role; locked stake and [[karma]] amplify the other three. They raise a neuron's weight in $A^{\text{eff}}$ — enlarging its $\Delta\phi^+$ and mint share — and active stake earns a share of the fee pool. Conviction stake doubles as the security deposit: the staking ratio $S$ is the fraction of supply locked across [[cyberlinks]], so the bonded capital is always productively committed. An attack on $\phi^*$ then needs both stake and unbuyable [[karma]].

---

## 9. Two Axes

Stake acts on two independent axes; separating them is the structural defense against wealth concentration, and the answer to requirement 3.

| axis | what moves it | what it produces |
|---|---|---|
| rank | any real stake, including $v=0$ | weight in $A^{\text{eff}}$, hence $\phi^*$ and [[cyberank]] |
| reward | correct risk under $v \neq 0$ | a share of the streams in §8 |

Idle, passive, or Sybil capital can move rank but pulls no reward. Capital shapes the graph; only correct epistemic risk earns from it. Locked capital cannot compound by sitting still — the precise structural fix for the wealth-compounding failure of stake-weighted systems.

A $v=0$ link earns nothing by category, not by penalty. It is a purchase, not an investment: the time-value of staked [[$CYB]] spent to buy weight over $\phi^*$. This is rational for a [[neuron]] whose use-value of that influence exceeds its capital cost, and unattractive to rent-seekers, who have none — a monetary yield would convert the purchase into an investment and reopen compounding. The protocol also cannot pay it in principle: minting must separate signal from copying through BTS information-gain (§5), which needs the meta-prediction that $v=0$ declines, so a passive link's $\Delta\phi^+$ is real movement that stays unverifiable as knowledge. Influence over $\phi^*$ is the entire return, paid in kind and unpriceable by design.

---

## 10. Supply and Allocation

The security budget splits between PoW and PoS by the allocation curve of [[adaptive hybrid consensus economics]]:

$$R_{\text{PoW}} = G\,(1 - S^\alpha), \qquad R_{\text{PoS}} = G\,S^\alpha, \qquad \alpha \in [0.3, 0.7],$$

with $\alpha = 0.5$ the neutral prior under equal marginal security cost. Gross budget and holder dilution are decoupled:

$$G = \text{floor}\cdot M + F(1-\beta), \qquad I_{\text{net}} = \text{floor} - \frac{F\beta}{M}.$$

Gross rewards can exceed inflation when fees are high; net inflation can go negative. The security floor is derived from attack economics rather than chosen — $\text{floor} \ge k\cdot(\text{TVL}/M)\cdot r$, the one emission untied to $\Delta\phi^+$.

Base emission goes to work and risk only. A standing yield to passive stake would be emission without contribution — it would break the invariant that inflation is [[knowledge]], and it is the mechanism by which idle capital compounds. The floor is paid only to the two providers that do work: PoW compute and active ($v \neq 0$) epistemic risk. It PID-decays toward zero as mint and fees grow to cover security. The parameters $\alpha$, floor, and $\beta$ follow PID control on observable signals (security margin, fee coverage, efficiency differential), so the system measures and adapts rather than predicts.

---

## 11. The Reward Equation

For a neuron $\nu$ over an epoch, the whole specification assembles into one line:

$$\boxed{\;R(\nu) \;=\; \underbrace{\text{Shapley}_\nu(v^\star)}_{\text{mint}} \;+\; \underbrace{R_{\text{PoW}}\cdot\frac{w_\nu}{\sum_\mu w_\mu}}_{\text{subsidy}} \;+\; \underbrace{\sum_{q\,\in\,Q_\nu}(1-\beta)\,\text{fee}_q}_{\text{service fees}} \;+\; \underbrace{R_{\text{PoS}}\cdot\frac{a_\nu\,\kappa(\nu)}{\sum_{\mu} a_\mu\,\kappa(\mu)}}_{\text{stake yield}}\;}$$

where $w_\nu$ is $\nu$'s proven settlement work this epoch, $Q_\nu$ the queries it answered, and $a_\nu$ its active ($v \neq 0$) stake. Four terms, four requirements: the mint rewards real value, locally computed and later validated; the subsidy secures the chain and opens a stakeless door, shared in proportion to work done so splitting a signal into many earns nothing extra; the service fees pay whoever served a query directly; the stake yield routes the fee pool to honest committed stake. Conservation, Sybil-resistance, and anti-compounding hold across the sum.

A single mint underpays foundational work, which starts at low $\Delta\phi^+$ and grows as the graph builds around it. So an active ($v \neq 0$) link also earns a yield stream — the delayed mint of that foundational work, the time-integral of the target particle's [[cyberank]] growth attributable to the link. Passive ($v=0$) stake earns no part of it; the annuity is realized value, not rent on locked capital:

$$R_{i \to j}(T) = \int_0^T w(t)\,\Delta\phi^*_j(t)\,dt.$$

The mint is the pulse; the yield is the annuity. Viral links earn the pulse and decay; foundational links earn the long-rising annuity; confirming links strengthen [[axon]] weight, shared by attribution. Together they pay both discovery and infrastructure.

---

## 12. Timing and Accrual

A contribution's worth is rarely settled the epoch it is made. A foundational link starts at tiny $\Delta\phi^+$ and grows over a hundred epochs as the graph builds around it; whether a link was surprising or correct is known only once the crowd and the [[inversely coupled bonding surface|ICBS]] market converge, many epochs later. The reward runs on three timescales, and no actor ever reaches back to re-grade the past.

| timescale | what is scored | who scores it |
|---|---|---|
| instant | the structural bound $\Delta\phi^+$ | the [[neuron]] (propose, §6) |
| epoch | surprise-weighted Shapley | the settlement-mining swarm (§7) |
| continuous | maturing value, resolving truth | the per-epoch [[focusing]] pass, the ICBS market, and [[karma]] accrual |

The present re-scores itself. [[focusing]] runs every epoch over the current graph, which still holds every historical link at its current weight. So an old contribution is scored now, at now's value, by now's settlement — the graph is the state, and the current $\phi^*$ already encodes all of history. A foundational link keeps earning because it stays a live player in its cluster's coalition: Shapley's complementarity gives it a slice of each epoch's new value precisely because the new links' worth depends on the foundation they build on. Distant-in-time value is collected as the integral of present-tense settlements (§11's annuity), through the contribution staying in the game rather than any re-opening of the past.

The market resolves distant-in-time truth. Each epoch [[focusing]] re-reads a link's current ICBS price and the contributor's current [[karma]]. When a link earlier thought false is later vindicated, its price rises, its weight rises, its $\Delta\phi^+$ contribution rises, and it earns again; when a link is falsified, its price falls to zero, its weight vanishes, and it stops earning and stops moving focus. The market is the time-resolution mechanism; karma is its reputational integral. The weights update and the present settlement reflects them, with no audit reaching backward.

Finality and accuracy are split. The pulse (instant mint, §11) is settled at the epoch boundary on provisional information — modest, conservative, final, paid now. The annuity (the yield stream) accrues as truth reveals itself, computed by the same per-epoch pass, and it is self-correcting: a link later falsified simply stops drawing it. So the protocol pays a small final pulse now and lets the annuity carry the long-horizon correction.

Open — the discovery leak. An early contrarian-correct link looks wrong at settlement (consensus disagrees, ICBS price low, surprise low because the crowd predicted failure), so it earns almost no pulse — exactly when the discovery premium should be largest. It begins earning only once the market turns, far later, through the annuity. A retroactive discovery bonus, paid when a long-dormant link's cluster finally ignites, is the natural fix and is unbuilt. A second asymmetry compounds it: a paid pulse is irreversible, so a contribution later revealed dishonest can have its future earnings stopped (price → 0) but not its pulse reversed — which is the reason to keep the pulse conservative and most value in the self-correcting annuity.

---

## 13. Token Operations

- Mint — prove $\Delta\phi^+$, receive the Shapley share; emission bounded by global $\Delta\phi^+$.
- Burn — destroy [[$CYB]] for permanent $\phi^*$-weight on [[eternal particles]] or [[eternal cyberlinks]]; the fee burn $\beta$ is the protocol-level form.
- Lock — stake on [[particles]] or [[cyberlinks]]; active stake earns fee yield, passive stake earns rank.

---

## 14. Positioning

Rewards are an emergent binding of four layers, and the separation keeps monetary policy out of consensus safety.

| concern | layer |
|---|---|
| value magnitude ($\Delta\phi^+$, [[karma]], [[syntropy]]) | [[tru]] |
| finality, canonical $\phi^*$, settlement lottery | [[foculus]] |
| conservation, allocation, mint | [[tok]] |
| identity, anonymity | [[mudra]] |

[[foculus]] decides what is real; the reward function decides what it is worth. Economic parameters change without touching consensus safety.

---

## 15. Security and Open Frontiers

| property | guarantee |
|---|---|
| conservation | $\sum_\nu \text{mint}(\nu) = $ global $\Delta\phi^+$, by Shapley efficiency |
| Sybil-resistance | stake-weighting makes identity-splitting reward-neutral |
| honest reporting | BTS makes truthful [[valence]] a Bayes–Nash equilibrium |
| stakeless entry | the PoW subsidy is karma- and stake-blind |
| no idle rent | only $v \neq 0$ risk earns; passive stake earns rank, not income |
| attack cost | $\phi^*$ manipulation needs stake and unbuyable [[karma]] |
| leaderless settlement | attribution is a swarm-averaged sampling lottery; no producer decides it |

Collusion remains open. Stake-weighting closes Sybil splitting, but a cartel of distinct, real-stake actors coordinating [[valence]] and links is not closed — BTS is incentive-compatible only against unilateral deviation. Partial defenses: the conservation cap (a ring on a saturated [[particle]] splits near-zero $\Delta\phi^+$), [[karma]] non-transferability, and [[identity]] cost.

Withholding remains open. A contender-miner can bias the settlement average by declining to publish unfavorable winning tickets (§7). It cannot lie, only abstain, so the bias is bounded by its share of settlement compute; pricing and role-separation tighten it, commit-before-marginal closes it at a synchrony cost. Both biases are bounded and still open.

The discovery leak remains open (§5, §12). The validity gate under-weights a novel link while its market price lags, and the pulse under-pays the early contrarian, deferring the discovery premium into the slow annuity. A retroactive discovery bonus is the candidate fix and is unbuilt.

Two economic questions are deferred to a standalone monetary-policy pass: total issuance — the mint runs on its own budget beside the security budget $G$, with no invariant yet capping their sum per epoch — and bootstrap funding, whether the early discovery-premium mint and the security floor can fund the [[Goldilocks field processor|GFP]] hardware before fees arrive.

---

## Appendix — Notation

| symbol | meaning |
|---|---|
| $G = (P, N, E)$ | the [[cybergraph]]: [[particles]] $P$, [[neurons]] $N$, [[cyberlinks]] $E$ |
| $\phi \in \Delta(P)$ | a [[focus]] distribution; $\phi^*$ is the [[tri-kernel]] fixed point |
| $\mathcal{R}$ | composite operator $\lambda_d D + \lambda_s S + \lambda_h H_\tau$; $\phi^* = \operatorname{norm}[\mathcal{R}\phi^*]$ |
| $\mathcal{F}(\phi)$ | system [[free energy]]; $\phi^* = \arg\min_\phi \mathcal{F}$ |
| $J(\phi)$ | [[syntropy]] $= D_{KL}(\phi \,\|\, u) = \log|P| - H(\phi)$ |
| $\Delta\phi^+$ | directed focus impulse, the reward primitive (§2) |
| $\sigma_\ell \in [0,1]$ | per-contribution [[Bayesian Truth Serum\|BTS]] surprise (§5) |
| $v^\star(S)$ | surprise-weighted value: $v$ scaled per contribution by $\sigma$ (§6) |
| $A^{\text{eff}}_{pq}$ | effective adjacency $= \sum_\ell \text{stake}(\ell)\,\kappa(\nu(\ell))\,f(\text{price}(\ell))$ |
| $\nu,\ \kappa(\nu)$ | a [[neuron]] and its [[karma]]; $\text{id}(\nu) = \text{Hemera}(\text{secret})$ |
| $v(\ell) \in \{-1,0,+1\}$ | the [[valence]] of a cyberlink |
| $v(S)$ | coalition value function (§3) |
| $s$ | a [[signal]] $= (\nu, \vec\ell, \Delta\phi^*, \sigma, \text{prev}, \text{mc}, \text{vdf}, \text{step}, \text{nonce})$ |
| $\sigma$ | a [[zheng]] proof bound to a [[BBG]] root |
| $S, M, F$ | staking ratio, market cap, epoch fees |

---

See [[focus-flow]] for how $\phi^*$ and $\Delta\phi^+$ are computed, [[truth-scoring]] for BTS and [[karma]], [[adaptive hybrid consensus economics]] for the PoW/PoS allocation and PID control, [[unified mining]] for the subsidy-as-signal-proof construction, and [[provable-consensus]] for epoch finalization. See whitepaper §6.9 and §14 for the surrounding economics.

The energy-based-learning grounding of §1 and §4 — that intrinsic free-energy descent, not a designed loss, is how the brain plausibly learns, and that Shapley credit is the integrated-gradient (Aumann–Shapley) generalization of it — follows Lillicrap, Santoro, Marris, Akerman & Hinton, *Backpropagation and the brain* (Nature Reviews Neuroscience, 2020); Scellier & Bengio, *Equilibrium Propagation* (2017); and Sundararajan, Taly & Yan, *Axiomatic Attribution / Integrated Gradients* (2017).
