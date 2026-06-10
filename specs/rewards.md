---
tags: cyber, tru, soft3
crystal-type: process
crystal-domain: cyber
alias: rewards, reward specification
---
# Reward Specification

Reward signal, candidate functions, hybrid model, link valuation, attribution, self-minting, and token operations for the [[cybergraph]] knowledge economy.

The reward signal is the [[impulse]] — the proven change in [[focus]] Δφ* a [[neuron]] delivers via a [[signal]]. The impulse is computed locally on the neuron's neighborhood (see [[focus-flow]]), proven against the [[BBG]] root, and doubles as the reward claim: a valid proof of ‖Δφ*‖ > 0 self-mints [[$CYB]] proportional to the shift. This page specifies what that signal is worth; [[focus-flow]] specifies how it is computed.

Source material: [[learning incentives]], [[rewards]]

---

## The Signal: Δφ*

Every reward traces back to one quantity: how much did the action shift the [[tri-kernel]] fixed point φ*?

$$\text{reward}(v) \propto \Delta\phi^*(v)$$

φ* is the stationary distribution of the composite operator $\mathcal{R} = \lambda_d D + \lambda_s S + \lambda_h H_\tau$ — [[diffusion]] explores, [[springs]] enforce structure, [[heat kernel]] adapts. The [[collective focus theorem]] proves φ* exists, is unique, and is computable locally.

Δφ* is the gradient of system [[free energy]]. Creating valuable structure is literally creating [[value]]. No designed loss function — the physics of [[convergence]] defines what deserves to be optimized.

Per-link reward:

$$r(\ell) \;\propto\; \Delta\phi^*(q) \;=\; \phi^*_{t+1}(q) - \phi^*_t(q)$$

where $q$ is the target particle of link $\ell$.

---

## Reward Functions

Five candidates for measuring convergence contribution, each with trade-offs:

| Function | Formula | Strength | Weakness |
|---|---|---|---|
| Δφ* norm | $\sum_j \big|\phi^{*(t+1)}_j - \phi^{*(t)}_j\big|$ | simple, easy to verify | unsigned — pays for noise, not just oscillation |
| [[syntropy]] growth | $H(\phi^{(t)}) - H(\phi^{(t+1)})$ | rewards semantic sharpening | computationally heavier |
| spectral gap | $\lambda_2^t - \lambda_2^{t+1}$ | measures global convergence speedup | expensive, non-local |
| predictive alignment | $\text{align}(\phi^{(t+1)}, \phi^*_T)$ | favors early correct contributions | requires delayed validation |
| DAG weight | descendant blocks referencing this one | rewards foundational work | slow to accrue |

---

## Hybrid Model

The hybrid model combines the candidate functions:

$$R = \alpha \cdot \Delta\phi^* + \beta \cdot \Delta J + \gamma \cdot \text{DAGWeight} + \epsilon \cdot \text{AlignmentBonus}$$

where $\Delta J = H(\phi^{(t)}) - H(\phi^{(t+1)})$ is [[syntropy]] growth.

Fast local rewards use Δφ* and ΔJ. Checkpoints add alignment and spectral verification bonuses. Validators sample and verify blocks probabilistically.

New [[$CYB]] is minted only when $\Delta\phi^* > 0$. The protocol's inflation is literally evidence of [[knowledge]] creation — there is no emission without demonstrated contribution to collective [[focus]].

---

## Link Valuation

[[Cyberlinks]] are yield-bearing epistemic assets. They accrue rewards over time based on contribution to [[focus]] emergence:

$$R_{i \to j}(T) = \int_0^T w(t) \cdot \Delta\phi^*_j(t) \, dt$$

where $\Delta\phi^*_j(t)$ = change in [[focus]] on target [[particle]] $j$ attributable to the link, $w(t)$ = time-weighting function, $T$ = evaluation horizon.

### Four Trajectory Types

| Link Type | Characteristics | Reward Trajectory |
|---|---|---|
| viral | high Δφ* short-term | early peak, fast decay |
| foundational | low Δφ* early, grows later | slow rise, long reward |
| confirming | low individual Δφ*, strengthens axon weight | shared reward via attribution |
| semantic bridge | medium, cross-module | moderate, persistent |

---

## Discovery Premium

Early discovery is maximally rewarded. The first [[neuron]] to surface a valuable [[particle]] captures the largest $\Delta\phi^*$. Late consensus-following earns little — when many neurons have already linked a particle, the marginal focus gain shrinks toward zero.

This creates a race to discover genuine [[relevance]] rather than copy existing links.

---

## Self-Minting

Rewards are computed locally. Each [[neuron]] proves their own contribution and claims their own reward.

Every [[cyber/signal]] carries a $\Delta\phi^*$ — the neuron's locally computed focus shift for a batch of [[cyberlinks]]. This $\Delta\phi^*$ is proven correct by a single [[stark]] proof referencing a specific $\text{bbg\_root}$.

The four steps:

1. [[Neuron]] creates [[cyber/signal]] with one or more [[cyberlinks]], $\Delta\phi^*$, and [[stark]] proof.
2. Proof demonstrates: applying these links to the graph at $\text{bbg\_root}_t$ shifts φ* by $\Delta\phi^*$.
3. Any verifier checks the proof against the header — O(log n), no recomputation.
4. If valid and Δφ* > 0, the neuron mints [[$CYB]] proportional to the proven shift.

No aggregator decides the reward. The proof IS the mining. A [[neuron]] on a phone: buy a header, query neighborhood state, create [[cyberlinks]], prove Δφ*, bundle into a [[cyber/signal]], mint tokens.

### Conservation

Total minting per epoch is bounded by the actual global Δφ*, verifiable from consecutive headers. If the sum of individual claims exceeds the actual shift (overlapping neighborhoods), all claims are scaled proportionally.

---

## Attribution

Multiple [[neurons]] contribute [[cyberlinks]] in the same epoch. The total Δφ* shift is a joint outcome — credit must be divided fairly.

The [[Shapley value]] answers: each agent's reward equals their average marginal contribution across all possible orderings. The coalition's total value is the [[free energy]] reduction $\Delta\mathcal{F}$, and each agent's marginal contribution is how much φ* shifts when their [[cyberlinks]] are added to the graph.

Exact computation is infeasible ($O(n!)$). [[Probabilistic Shapley attribution]] approximates:

1. Local marginal — compute each transaction's individual $\Delta\mathcal{F}$ (add link, measure φ* shift).
2. Monte Carlo sampling — sample $k$ random orderings of the epoch's transactions, measure marginal contributions in each ordering.
3. Hierarchical batching — cluster transactions by affected neighborhood, distribute within clusters.
4. Final reward:

$$R_i = \alpha \cdot \Delta\mathcal{F}_i + (1-\alpha) \cdot \hat{S}_i$$

where $\Delta\mathcal{F}_i$ is the fast local estimate and $\hat{S}_i$ is the sampled Shapley approximation. $\alpha$ balances speed (local marginal) against fairness (Shapley).

Complexity: $O(k \cdot n)$ with $k \ll n$. Feasible for $10^6+$ transactions per epoch.

---

## Three Token Operations

- Mint: [[neurons]] prove Δφ* via [[stark]] and self-mint [[$CYB]] proportional to their contribution.
- Burn: [[neurons]] destroy [[$CYB]] for permanent φ*-weight on [[particles]] ([[eternal particles]]) or [[cyberlinks]] ([[eternal cyberlinks]]).
- Lock: [[neurons]] stake [[$CYB]] on [[particles]] or [[cyberlinks]], earning from fee pools proportional to [[attention]] attracted.

---

## Costly Signals

Learning incentives and [[costly signal]] mechanics work together: the staking cost filters out noise, while the reward function amplifies signal. A neuron must risk real [[tokens]] (cost) to earn rewards (incentive), ensuring alignment between economic interest and [[knowledge]] production.

---

## The Game

[[Knowledge]] creation is costly, but its benefits are [[collective]]. Without incentives, rational agents free-ride on others' [[cyberlinks]] — the discoverer bears the cost, the network reaps the reward, and the graph stagnates into an epistemic tragedy of the commons. The reward mechanism makes contributing profitable and free-riding unprofitable.

The rules produce a game where early and accurate wins:

- early, accurate links to important [[particles]] earn the most — the [[attention]] yield curve
- confirming links strengthen [[axon]] weight: repeated signals build consensus, not noise
- [[neurons]] build long-term reputation through accumulated [[karma]]
- [[focus]] as cost makes every [[cyberlink]] a [[costly signal]] — you stake real [[$CYB]] to play

The evolutionary loop: contribute accurately → Δφ* reward → accumulate [[$CYB]] → stake on more links → accumulate [[karma]] → links carry more adjacency weight → earlier Δφ* attribution → more [[$CYB]] per contribution. The flywheel rewards sustained accuracy, not one-time luck.

---

## Settled Design

The synthesis from working through [[GFP]], the [[impulse]], [[Shapley]], and [[Bayesian Truth Serum]] together (2026-06-10). The five candidate functions above do not coexist as a hybrid — they collapse into a small core. This section records what survives and why.

### The collapse

| candidate | fate | reason |
|---|---|---|
| Δφ* norm | core, refined | becomes directed Δφ⁺, stake-weighted, [[Shapley]]-attributed |
| [[syntropy]] growth ΔJ | justification only | Δφ⁺ is the first-order proxy for ΔJ — not a separate term |
| spectral gap | dropped | non-local, violates local computation |
| predictive alignment | becomes [[karma]] | the validate-later honesty axis; enters the value function |
| DAG weight | becomes link valuation | the time-integral yield stream for foundational links |

The four-term hybrid measures the same thing four ways. The settled form is one mint ([[Shapley]] of Δφ⁺), one yield stream (link valuation over time), one honesty gate ([[karma]] inside the value function). Same coverage, far fewer knobs.

### The core formula

$$\text{mint}(\nu) = \text{Shapley}_\nu(v), \qquad v(S) = \Delta\phi^*\big(\text{effective graph} + S\big)$$

The value function $v$ is the [[focus]] shift of a coalition $S$ of [[cyberlinks]], computed on the [[karma]]-weighted effective adjacency $A^{\text{eff}} = \text{stake} \times \text{karma} \times f(\text{price})$. [[karma]] enters $v$, so copies and noise carry near-zero marginal before attribution runs. [[Shapley]] splits $v(N)$ fairly.

Three properties come for free:

- conservation = Shapley's efficiency axiom: $\sum$ shares $= $ global Δφ*. No separate scaling operator.
- Sybil-resistance = stake-weighting: $v$ is homogeneous in stake, so splitting one neuron into many with the same total stake yields the same total share. Identity is cheap; stake and [[karma]] are not, and karma cannot be bought.
- tractability = locality: each Shapley marginal is an incremental [[tri-kernel]] recompute on an $O(\log 1/\varepsilon)$-hop neighborhood. Monte-Carlo over beacon-seeded random orderings is feasible — and random orderings cannot be front-run, which is the fix for the Steem curation-race failure.

### Two axes

Stake acts on two independent axes:

- rank — any real stake, including passive ([[valence]] 0), weights $A^{\text{eff}}$ and moves φ*/[[cyberank]]. Capital shapes the graph.
- reward — only correct risk under non-zero [[valence]] earns. Capital alone cannot extract; it must be right.

Idle, passive, or Sybil capital can move rank but pulls no reward. This is the structural answer to wealth compounding: locked capital cannot earn by sitting still.

### The valence risk dial

[[valence]] $v \in \{-1, 0, +1\}$ is a per-link choice of exposure:

- $v = 0$ — passive stake. Weights the edge, affects rank, earns no reward. Its compensation is influence over [[focus]], paid in kind.
- $v = \pm 1$ — active epistemic bet. Wagered through the BTS zero-sum: the right are paid by the wrong.

Reward is the premium for risk taken and won, not rent for capital parked.

### Base reward

Reward flows to demonstrated contribution. A standing yield to passive stake would be emission without contribution — it breaks the invariant that inflation is evidence of [[knowledge]], and it is the mechanism by which idle capital compounds. The one sanctioned non-Δφ* emission is the security floor.

A network needs a minimum security budget when [[fees]] and Δφ* mint run thin (the argument in [[adaptive hybrid consensus economics]]). The floor $\approx k \cdot \text{TVL}/\text{MarketCap} \cdot r$ is derived from attack economics, not chosen. It pays the two security providers that do work — PoW compute and active ($v \neq 0$) epistemic risk — and PID-decays toward zero as mint and fees grow to cover security. Passive stake's security service (raising attack cost by being locked) is compensated by rank influence, paid in kind.

### Two access paths: PoW and PoS

Reward has a stakeless onramp and a staked amplifier, combined by the allocation curve $\text{staking\_share} = S^\alpha$ from [[adaptive hybrid consensus economics]]:

- PoW — stakeless onramp. Produce the [[zheng]] proof of the [[impulse]], hit a difficulty target, earn the block subsidy. Needs compute, not capital. The subsidy is [[karma]]-blind, so a new [[neuron]] with zero stake can mine. This is a hard requirement, not an option.
- PoS — amplifier. Stake and [[karma]] raise $A^{\text{eff}}$ weight and earn fee yield. Never required to start.

[[karma]] multiplies Δφ⁺ always; [[karma]] never multiplies the block subsidy. A high-karma miner earns more because its links are worth more, not because its proofs are worth more — the onramp stays open to everyone.

### BTS is the slashing

The BTS zero-sum is the skin in the game: liars pay truth-tellers, stake redistributes from noise producers to signal producers proportional to score. Staking is required — it is what BTS redistributes. [[foculus]] drops only consensus-equivocation slashing, because provable consensus makes an invalid φ* unable to produce a valid proof; there is no equivocation crime to punish after the fact. Epistemic slashing stays.

### Positioning

Rewards are not a module — they bind four layers:

| concern | layer |
|---|---|
| value magnitude (Δφ⁺, [[karma]], [[syntropy]]) | [[tru]] |
| finality / canonical φ* | [[foculus]] |
| conservation + mint | [[tok]] |
| identity | [[mudra]] |

[[Shapley]] attribution computes in [[tru]], a sibling of [[cyberank]]. Economics stays out of [[foculus]] so monetary policy never couples to consensus safety.

### Open: collusion

Stake-weighting closes Sybil splitting, but a cartel of distinct, real-stake actors coordinating [[valence]] and links is not closed — BTS is incentive-compatible only against unilateral deviation. Partial defenses: the conservation cap (a ring on a saturated [[particle]] splits near-zero Δφ*), [[karma]] non-transferability, and [[identity]] cost. This is the live frontier (the likely shape of Steem's failure mode).

---

See §6.9 and §14.2 of the whitepaper for the full specification. See [[cyber/tokenomics]] for the system-level economics (monetary policy, allocation curve, GFP flywheel). See [[collective learning]] for the group-level dynamics. See [[cyberlink]], [[focus]], [[neuron]], [[particle]], [[costly signal]], [[convergence vm]].
