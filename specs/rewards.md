---
tags: cyber, tru, soft3
crystal-type: spec
alias: rewards, reward specification, reward function, learning incentives, learning rewards
---
# Reward Specification

One law runs the whole economy: **new money is minted only when, and exactly where, [[knowledge]] is created.** Knowledge has a physical meaning here — focus settling into a lower-energy, more coherent state — so inflation is not a policy, it is a *measurement*. Everything below is that one law made precise: how the quantity is measured, how it is divided fairly among the [[neurons]] that produced it, how it is computed and secured without a leader, and what economy assembles around it.

The design follows from four requirements, in order of force:

1. **compute locally, validate later** — a [[neuron]] on a phone must be able to claim its reward from its own neighborhood, before any global agreement;
2. **be fair** — overlapping contributions must split by a principled rule, not a race;
3. **pay for real value** — emission must track demonstrated contribution, never idle capital;
4. **optimize the network's own compute** — the work done to earn must be the work the network needs.

---

## 1. The Principle — pay for descent

[[focus]] $\phi$ is a distribution of attention over [[particles]]. Left alone on the [[cybergraph]], it flows downhill on a landscape — the [[free energy]] $\mathcal{F}$ — and settles at the unique low point $\phi^*$, the [[tri-kernel]] fixed point ([[collective focus theorem]]). A [[cyberlink]] reshapes the landscape; the focus rolls to a new resting place; the **drop in free energy is the value created**, equivalently the gain in [[syntropy]]:

$$\Delta J = J(\phi^*_{t+1}) - J(\phi^*_t) = H(\phi^*_t) - H(\phi^*_{t+1}).$$

There is no designed loss function — and this is the load-bearing choice, so state it exactly. A supervised loss points *outside* the system at a target somebody supplied ("this input should output *cat*"); it is one arbitrary goal per example, and it can be gamed by forging the answer key. $\mathcal{F}$ points *only at the system's own internal consistency* — there is no external target anywhere, so there is no answer key to design or to forge. The focus value $\phi^*$ is not designed either; it falls out of the law and the data, the way a marble's resting spot falls out of the bowl, not out of anyone's wish.

One criterion *is* chosen, and honesty demands naming it: **descent is value** — focus concentrating, the graph agreeing with itself, is the thing worth paying for. But this is a single uniform law, content-blind, applied identically to everything; it is not a per-case target. And it is less invented than discovered — free-energy minimization is the same principle statistical inference, thermodynamics, and the Bayesian brain independently converge on. It is designed the way $F=ma$ is designed: a law that appears to be the right one, not a knob tuned to taste.

So inflation measures knowledge creation rather than expressing a policy. That is the whole foundation. The rest is consequence.

---

## 2. The Measure — the directed impulse

$\Delta J$ is the exact value but carries a global normalization term, so it is not what a neuron computes. Its first-order local form is the **directed focus impulse**:

$$\Delta\phi^+ \;=\; \big\langle -\nabla\mathcal{F}(\phi^*_t),\; \Delta\phi^* \big\rangle_+ \;\approx\; \langle \nabla J, \Delta\phi^*\rangle,$$

the projection of the focus displacement onto the descending free-energy gradient, clipped at zero. Two rationales:

- **directed, not magnitude.** The norm $\|\Delta\phi^*\|$ is unsigned — it pays for *any* movement, including movement that *raises* free energy (adding noise). $\Delta\phi^+$ pays only for the downhill component, only for sharpening.
- **a gradient, not a loss.** $\Delta\phi^+$ *is* a gradient — the slope of the intrinsic landscape of §1, not of an external scorecard. "No designed loss" never meant "no gradient." It means the gradient is of the system's own energy.

Two properties make $\Delta\phi^+$ usable as the reward primitive — and they are exactly requirement 1:

- **local.** By the [[locality theorem]], $\Delta\phi^+$ is computable on the neuron's $O(\log 1/\varepsilon)$-hop neighborhood; the perturbation of a single edge decays exponentially with graph distance, and entries past that radius fall below $\varepsilon$.
- **provable.** A single [[zheng]] proof $\sigma$ certifies $\Delta\phi^+$ against the current [[BBG]] root in $O(\log n)$, with no re-execution of the [[tri-kernel]].

Everything paid by the protocol traces to this one scalar.

---

## 3. The Value Function

Contributions are not independent — [[neurons]] cluster on popular [[particles]], so their neighborhoods overlap and credit must be shared. Define the value of a coalition $S$ of [[cyberlinks]] submitted in an epoch:

$$v(S) \;=\; \Delta\phi^+\!\big(A^{\text{eff}} \cup S\big),$$

the directed focus shift produced by applying $S$ to the **[[karma]]-weighted effective graph** $A^{\text{eff}}_{pq} = \sum_\ell \text{stake}(\ell)\,\kappa(\nu(\ell))\,f(\text{price}(\ell))$.

This is the single point where honesty couples to value. Because $A^{\text{eff}}$ already folds in stake, [[karma]], and market price, a redundant or dishonest link enters $v$ with near-zero weight — **[[karma]] shapes what is valuable; attribution (§4) only divides it.** $v$ is monotone, bounded, and submodular: overlapping links on a saturating particle have diminishing returns. That submodularity is used twice below — it caps the propose-time claim (§6) and it makes settlement an honest sampling problem (§7).

---

## 4. Fair Division — Shapley, and why it is integrated gradient

The epoch's total shift $v(N)$ must be split among contributors. The split is the [[Shapley value]]:

$$\text{mint}(\nu) \;=\; \text{Shapley}_\nu(v) \;=\; \sum_{S \subseteq N \setminus \{\nu\}} \frac{|S|!\,(|N|-|S|-1)!}{|N|!}\,\big[v(S \cup \{\nu\}) - v(S)\big].$$

Shapley is not one option among several — it is the **unique** attribution satisfying efficiency, symmetry, null-player, and additivity. The alternatives fail concretely: order-based credit ("reward whoever links first") is gameable by latency and copying — the curation-reward death spiral seen on social chains; proportional scaling cannot tell a discoverer from a copyist. Three of Shapley's properties are load-bearing, and each answers a requirement for free:

- **conservation is the efficiency axiom.** $\sum_\nu \text{mint}(\nu) = v(N) = $ global $\Delta\phi^+$. No separate conservation operator exists or is needed; over-claiming is impossible by construction. This is requirement 3.
- **Sybil-resistance is homogeneity.** $v$ is homogeneous in stake, so splitting one neuron into $k$ identities holding the same total stake yields the same total share. Identity is cheap; stake and [[karma]] are the attributed resources, and [[karma]] cannot be bought.
- **tractability is locality.** Each marginal $v(S \cup \{\nu\}) - v(S)$ is an incremental [[tri-kernel]] step on a bounded neighborhood, estimated by Monte-Carlo over $k$ random orderings seeded by a [[delay|VDF]] beacon — $O(k\cdot n)$, $k \ll n$. The beacon is drawn *after* submission, so orderings cannot be front-run.

### The deep identity

There is a reason Shapley is the *right* tool and not merely a fair one, and it ties this reward to how brains learn. A deep network assigns credit by the chain rule — backpropagation. The brain almost certainly does not run backprop (a synapse cannot read itself backward — the weight-transport problem), but it plausibly does *gradient* learning by energy-based means: predictive coding and equilibrium propagation show that the same relaxation that settles a network to its answer also computes the gradient of its energy, locally, with no separate backward pass. The objective is intrinsic free energy — exactly §1.

cyber takes the same stance, then makes one move further. Its "neurons" are strategic, stake-bearing agents, so credit must be *fair*, not merely differentiated. And the fair generalization is already gradient credit: the **Aumann–Shapley value** — the continuous limit of Shapley — is precisely the path integral of the gradient, the method known in machine learning as *integrated gradients*. So:

$$\underbrace{\text{Shapley}_\nu(v)}_{\text{discrete agents}} \quad\xrightarrow{\ \text{non-atomic limit}\ }\quad \underbrace{\int_0^1 \partial_\nu\, v(t\!\cdot\!N)\,dt}_{\text{integrated gradient of }\Delta\phi^+}.$$

Dividing the reward by Shapley over $\Delta\phi^+$ *is* integrated-gradient credit assignment on the free energy — the same mathematics as energy-based learning in the brain, lifted to agents who must be paid rather than merely tuned. The two boldest choices in this document — *intrinsic gradient, not designed loss* and *Shapley, not a race* — are therefore one choice seen at two resolutions.

This computation lives in [[tru]], a sibling of [[cyberank]].

---

## 5. Honesty

Shapley is fair only among honest, distinct contributors. Two mechanisms enforce that precondition; both are already inside $v$ via [[karma]].

**Bayesian Truth Serum.** Each [[cyberlink]] is a [[Bayesian Truth Serum]] input: the link-plus-stake is the first-order belief, the [[valence]] $v \in \{-1,0,+1\}$ is the meta-prediction. The score

$$s_\nu = \underbrace{D_{KL}(p_\nu \,\|\, \bar m_{-\nu}) - D_{KL}(p_\nu \,\|\, \bar p_{-\nu})}_{\text{information gain}} - \underbrace{D_{KL}(\bar p_{-\nu} \,\|\, m_\nu)}_{\text{prediction accuracy}}$$

is positive exactly when a neuron contributes private signal the crowd did not already hold and expect. Copying the consensus drives the information-gain term to zero. By Prelec's theorem, truthful reporting is a Bayes–Nash equilibrium.

**Karma is the slashing.** [[karma]] $\kappa(\nu)$ is the accumulated BTS score: non-transferable, unbuyable, the one input to $A^{\text{eff}}$ that capital cannot purchase. The BTS settlement is a zero-sum redistribution — stake moves from noise producers to signal producers in proportion to score. *This is the skin in the game and the slashing: liars pay truth-tellers.* Staking is therefore required, because it is what the zero-sum redistributes. [[foculus]] omits only consensus-equivocation slashing — provable consensus makes an invalid $\phi^*$ unable to produce a valid proof, so there is no equivocation crime to punish.

**Valence is the risk dial.** Exposure is chosen per link: $v = 0$ is passive stake — it weights the edge in $A^{\text{eff}}$ and so moves rank (§9), but takes no BTS exposure and earns no reward; $v = \pm 1$ is active stake, wagered through the zero-sum. Reward is the premium for risk taken and won.

---

## 6. Propose and Settle

Requirement 1 — local now, validated later — and requirement 2 — Shapley fairness — appear to conflict: a neuron's Shapley share is a function of *the other contenders*, who do not exist when it acts alone. The resolution is not a compromise; it is forced. **Propose computes a bound; settle computes the share.** They are two phases because they must be.

**Propose (instant, agent-local).** A neuron computes its own standalone marginal $\Delta\phi^+_\nu = v(\{\nu\}) - v(\emptyset)$ against the [[BBG]] header it observed, proves it with $\sigma$, and gossips the [[signal]]. By submodularity (§3) this standalone marginal is the *largest* marginal the neuron can ever contribute, so

$$\text{Shapley}_\nu(v) \;\le\; \Delta\phi^+_\nu.$$

The propose proof is a **provable ceiling** on the reward, not the reward. It bounds the claim, it is what conviction stake escrows against, and settlement can only ever pay $\le$ it — with equality exactly when the neuron was alone in its region (the sparse-link case). A phone completes this phase.

**Settle (epoch boundary).** [[foculus]] finalizes the canonical $\phi^*$ and the epoch's claim set; the claims partition into clusters (§7); a leaderless lottery computes the Shapley shares (§7); [[tok]] applies conservation and executes the result as a state transition.

The two phases certify *different facts against different states*: propose proves "my marginal against my header was $X$" (the ceiling); settle proves "the division of the real joint $\Delta\phi^+$ is correct" (the share). The settlement beacon is drawn after propose closes — which is exactly what makes the orderings un-front-runnable. The distinction that dissolves the apparent conflict: *agent-local* (one actor, alone — possible for the bound, impossible for the share) versus *graph-local* (bounded neighborhood — true for both).

---

## 7. Settlement Mining

Settlement is computed with no neuron, leader, or aggregator deciding it. This is the document's structural core, and it satisfies requirement 4: the work that secures the chain *is* the work that computes the fair division.

### The region

Locality is in graph distance because $\phi^*$ is a heat-kernel fixed point with exponential spatial decay. The region a claim touches is its **$\varepsilon$-support** — every node whose contribution to $\Delta\phi^+$ is $\ge \varepsilon$, the protocol precision floor:

- radius $r = O(\log 1/\varepsilon)$ hops, $r \approx \log(1/\varepsilon)/\log(1/\lambda_{\text{local}})$;
- content-dependent — wide around a hub (slow local mixing), tiny on the sparse fringe;
- canonical — the superlevel set is a deterministic function of the edges and $\varepsilon$, so no miner can draw a self-serving boundary. The settlement proof commits to the support and certifies that boundary nodes are genuinely $< \varepsilon$, the anti-cheat against excluding a node to inflate a marginal.

A **cluster** is a connected component of overlapping $\varepsilon$-supports. The partition of an epoch's claims into clusters is therefore canonical, and clusters are independent — non-overlapping regions do not affect each other's Shapley values, so settlement parallelizes across them.

### The lottery

A deterministic "first to compute the settlement wins" is *not* progress-free — the fastest machine finishes first every time, electing a de facto leader and centralizing. The fix does not bolt a random puzzle onto useful work. It observes that **Shapley estimation is already a sampling process**, and makes each sample a lottery ticket: the entropy the lottery needs and the variance the estimator needs are the same randomness.

For a cluster with beacon seed $\mathrm{b}$, a miner:

1. picks a nonce $n$; the ordering is $\pi(n) = \mathrm{VRF}(\mathrm{b} \,\|\, n)$ — public and miner-independent;
2. computes the marginal sample $m(n)$ under $\pi(n)$ — a genuine draw of the §4 estimator, and the useful work;
3. holds a winning ticket iff $H(\mathrm{b} \,\|\, n \,\|\, \mathrm{id}(\nu)) < \text{target}$, claimed by publishing $(n, m(n), \sigma)$.

Step 3 is a per-miner Poisson test: progress-free, leaderless, poolable on the same terms as Nakamoto consensus, and random in proportion to throughput. **The settlement itself is the average of every published sample** — more mining means more independent draws and a tighter estimate (Hoeffding). No actor produces the answer; it converges out of the swarm, and security spend converts directly into attribution precision with zero synthetic work.

This collapses the proof-of-work subsidy (§8) into the same act. The nonce a miner grinds to reseed a proof hash *is* the ordering index $n$ — so every hash attempt is a real Shapley sample, not an empty reseed. Securing the chain and computing the fair division become one computation; settlement mining is not a separate stream but the **content** of the PoW subsidy.

### Residual: withholding

The lottery is not fully closed against a miner that is *also a contender* in the cluster it settles: it can compute $m(n)$, see that the sample lowers its own share, and decline to publish even a winning ticket. It cannot lie — claiming any ticket requires publishing the verified $m(n)$ — so the only freedom is *not playing* a nonce, and a withheld nonce is still a valid ticket for other miners (their threshold is keyed to their own identity), who re-cover it with probability proportional to their throughput. The injectable bias is therefore **bounded by the attacker's share** of settlement compute — negligible for a minority, and a majority already breaks consensus. The cheap deterrents are to *price* it (a withheld ticket forfeits its subsidy; calibrate so the forfeit exceeds the share-gain) and to *separate roles* (a miner does not settle a cluster it contends in). A commit-to-$n$-before-learning-$m(n)$ round drives the bias to zero in expectation by forcing a non-adaptive adversary, but it imports a synchronous commit–reveal assumption foreign to the lottery; it is the escalation, not the default. This sits alongside collusion (§14) as a bounded, not-yet-closed frontier.

---

## 8. The Three Roles

A single computation — the [[tri-kernel]] over the [[Goldilocks field]], simultaneously learning, proving, and inference — earns in three roles, distinguished only by what its proof certifies. This is requirement 4 at the economic level: one chip, one kind of work, three economic faces.

| role | the proof certifies | who earns | resource |
|---|---|---|---|
| **mint** | a graph mutation (focus shift) | anyone who links | conviction stake |
| **subsidy** | a proof meeting a difficulty target | anyone who computes | compute |
| **fee** | a query answered (inference) | anyone who serves | compute + model |

**Mint — the knowledge stream.** §1–§7. A neuron links, computes $\Delta\phi^+$, proves it, and self-mints its Shapley share, settled by the lottery and bounded by global $\Delta\phi^+$. Earning it requires conviction stake — a [[costly signal]]. This is a budget of its own, *not* drawn from the security budget $G$ below.

**Subsidy — proof of work, the stakeless onramp.** The [[signal]] carries a nonce; a signal qualifies for the block subsidy when $H(\sigma) < \text{target}$. The puzzle is the signal proof itself — it exercises the four [[Goldilocks field processor|GFP]] primitives (fma, ntt, p2r, lut) in production ratios, and at settlement the nonce is the Shapley ordering index (§7), so no work is synthetic. The subsidy is [[karma]]-blind and stake-blind: a new [[neuron]] with zero [[$CYB]] earns it and acquires the stake that unlocks the mint. This permissionless entry is a hard requirement. Difficulty adjusts to hold block time; the subsidy is independent of $\Delta\phi^+$.

**Fee — services.** A neuron answering a query runs the compiled transformer ([[focus-flow]] Path B), an inference whose correctness is itself a [[zheng]] proof. The asker pays; the protocol splits the fee to the servicer and the budget $G$, and burns a fraction $\beta$.

**PoS is the amplifier, not a fourth role.** Locked stake and [[karma]] raise a neuron's weight in $A^{\text{eff}}$ — enlarging its $\Delta\phi^+$ and mint share — and active stake earns a share of the fee pool. Conviction stake doubles as the security deposit: the staking ratio $S$ is the fraction of supply locked across [[cyberlinks]], so there is no idle bonded capital. An attack on $\phi^*$ then needs both stake and unbuyable [[karma]].

---

## 9. Two Axes

Stake acts on two independent axes; separating them is the structural defense against wealth concentration, and the answer to requirement 3.

| axis | what moves it | what it produces |
|---|---|---|
| **rank** | any real stake, including $v=0$ | weight in $A^{\text{eff}}$, hence $\phi^*$ and [[cyberank]] |
| **reward** | correct risk under $v \neq 0$ | a share of the streams in §8 |

Idle, passive, or Sybil capital can move **rank** but pulls no **reward**. Capital shapes the graph; only correct epistemic risk earns from it. Locked capital cannot compound by sitting still — the precise structural fix for the wealth-compounding failure of stake-weighted systems.

A $v=0$ link earns nothing by *category*, not by penalty. It is not capital seeking yield but a **purchase**: the time-value of staked [[$CYB]] spent to buy weight over $\phi^*$. This is rational for a [[neuron]] whose use-value of that influence exceeds its capital cost, and unattractive to rent-seekers, who have none — a monetary yield would convert the purchase into an investment and reopen compounding. Nor could it be paid even in principle: minting must separate signal from copying through BTS information-gain (§5), which needs the meta-prediction that $v=0$ declines, so a passive link's $\Delta\phi^+$ is real movement yet unverifiable as knowledge. Influence over $\phi^*$ is the entire return, paid in kind and unpriceable by design.

---

## 10. Supply and Allocation

The security budget splits between PoW and PoS by the allocation curve of [[adaptive hybrid consensus economics]]:

$$R_{\text{PoW}} = G\,(1 - S^\alpha), \qquad R_{\text{PoS}} = G\,S^\alpha, \qquad \alpha \in [0.3, 0.7],$$

with $\alpha = 0.5$ the neutral prior under equal marginal security cost. Gross budget and holder dilution are decoupled:

$$G = \text{floor}\cdot M + F(1-\beta), \qquad I_{\text{net}} = \text{floor} - \frac{F\beta}{M}.$$

Gross rewards can exceed inflation when fees are high; net inflation can go negative. The **security floor** is derived from attack economics, not chosen — $\text{floor} \ge k\cdot(\text{TVL}/M)\cdot r$, the only emission not tied to $\Delta\phi^+$.

**No base yield to idle stake.** A standing yield to passive stake would be emission without contribution — it breaks the invariant that inflation is [[knowledge]], and it is the mechanism by which idle capital compounds. The floor is paid only to the two providers that do work: PoW compute and active ($v \neq 0$) epistemic risk. It PID-decays toward zero as mint and fees grow to cover security. The parameters $\alpha$, floor, and $\beta$ are not hardcoded; they follow PID control on observable signals (security margin, fee coverage, efficiency differential), so the system measures and adapts rather than predicts.

---

## 11. The Reward Equation

For a neuron $\nu$ over an epoch, the whole specification assembles into one line:

$$\boxed{\;R(\nu) \;=\; \underbrace{\text{Shapley}_\nu(v)}_{\text{mint, }\Delta\phi^+\text{-bounded}} \;+\; \underbrace{\frac{R_{\text{PoW}}}{|W|}\,\mathbb{1}[H(\sigma_\nu) < \text{target}]}_{\text{subsidy}} \;+\; \underbrace{R_{\text{PoS}}\cdot\frac{a_\nu\,\kappa(\nu)}{\sum_{\mu} a_\mu\,\kappa(\mu)}}_{\text{fee yield, active stake } a}\;}$$

where $W$ is the set of signals meeting difficulty and $a_\nu$ is $\nu$'s active ($v \neq 0$) stake. Each term answers a distinct requirement: the **mint** rewards real value, locally computed and later validated; the **subsidy** secures the chain and opens a stakeless door; the **yield** routes service revenue to honest committed stake. Conservation, Sybil-resistance, and anti-compounding hold across the sum.

A single mint underpays foundational work, which starts at low $\Delta\phi^+$ and grows as the graph builds around it. So an active ($v \neq 0$) link also earns a **yield stream** — the delayed mint of that foundational work, the time-integral of the target particle's [[cyberank]] growth attributable to the link. Passive ($v=0$) stake earns no part of it; the annuity is realized value, not rent on locked capital:

$$R_{i \to j}(T) = \int_0^T w(t)\,\Delta\phi^*_j(t)\,dt.$$

The mint is the pulse; the yield is the annuity. Viral links earn the pulse and decay; foundational links earn the long-rising annuity; confirming links strengthen [[axon]] weight, shared by attribution. Together they pay both discovery and infrastructure.

---

## 12. Token Operations

- **Mint** — prove $\Delta\phi^+$, receive the Shapley share; emission bounded by global $\Delta\phi^+$.
- **Burn** — destroy [[$CYB]] for permanent $\phi^*$-weight on [[eternal particles]] or [[eternal cyberlinks]]; the fee burn $\beta$ is the protocol-level form.
- **Lock** — stake on [[particles]] or [[cyberlinks]]; active stake earns fee yield, passive stake earns rank.

---

## 13. Positioning

Rewards are not a module. They bind four layers, and the separation keeps monetary policy out of consensus safety.

| concern | layer |
|---|---|
| value magnitude ($\Delta\phi^+$, [[karma]], [[syntropy]]) | [[tru]] |
| finality, canonical $\phi^*$, settlement lottery | [[foculus]] |
| conservation, allocation, mint | [[tok]] |
| identity, anonymity | [[mudra]] |

[[foculus]] decides what is real; the reward function decides what it is worth. Economic parameters change without touching consensus safety.

---

## 14. Security and Open Frontiers

| property | guarantee |
|---|---|
| conservation | $\sum_\nu \text{mint}(\nu) = $ global $\Delta\phi^+$, by Shapley efficiency |
| Sybil-resistance | stake-weighting makes identity-splitting reward-neutral |
| honest reporting | BTS makes truthful [[valence]] a Bayes–Nash equilibrium |
| stakeless entry | the PoW subsidy is karma- and stake-blind |
| no idle rent | only $v \neq 0$ risk earns; passive stake earns rank, not income |
| attack cost | $\phi^*$ manipulation needs stake and unbuyable [[karma]] |
| leaderless settlement | attribution is a swarm-averaged sampling lottery; no producer decides it |

**Open — collusion.** Stake-weighting closes Sybil splitting, but a cartel of distinct, real-stake actors coordinating [[valence]] and links is not closed — BTS is incentive-compatible only against unilateral deviation. Partial defenses: the conservation cap (a ring on a saturated [[particle]] splits near-zero $\Delta\phi^+$), [[karma]] non-transferability, and [[identity]] cost.

**Open — withholding.** A contender-miner can bias the settlement average by declining to publish unfavorable winning tickets (§7). It cannot lie, only abstain, so the bias is bounded by its share of settlement compute; pricing and role-separation tighten it, commit-before-marginal closes it at a synchrony cost. Both frontiers are bounded, not yet closed.

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
