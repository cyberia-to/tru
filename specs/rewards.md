# Reward Specification

Reward signal, candidate functions, hybrid model, link valuation, attribution, self-minting, and token operations for the [[cybergraph]] knowledge economy.

Source material: [[learning incentives]], [[rewards]]

---

## The Signal: Δφ*

Every reward traces back to one quantity: how much did the action shift the [[tri-kernel]] fixed point φ*?

$$\text{reward}(v) \propto \Delta\phi^*(v)$$

φ* is the stationary distribution of the composite operator $\mathcal{R} = \lambda_d D + \lambda_s S + \lambda_h H_\tau$ — [[diffusion]] explores, [[springs]] enforce structure, [[heat kernel]] adapts. The [[collective focus theorem]] proves φ* exists, is unique, and is computable locally.

Δφ* is the gradient of system [[free energy]]. Creating valuable structure is literally creating [[value]]. No designed loss function — the physics of [[convergence]] defines what deserves to be optimized.

Per-link reward:

$$r(\ell) \;\propto\; \Delta\phi^*^*(q) \;=\; \phi^*_{t+1}(q) - \phi^*_t(q)$$

where $q$ is the target particle of link $\ell$.

---

## Reward Functions

Five candidates for measuring convergence contribution, each with trade-offs:

| Function | Formula | Strength | Weakness |
|---|---|---|---|
| Δφ* norm | $\sum_j \|\phi^*_j^{(t+1)} - \phi^*_j^t\|$ | simple, easy to verify | gameable by oscillation |
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

Early discovery is maximally rewarded. The first [[neuron]] to surface a valuable [[particle]] captures the largest $\Delta\phi^*^*$. Late consensus-following earns little — when many neurons have already linked a particle, the marginal focus gain shrinks toward zero.

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

See §6.9 and §14.2 of the whitepaper for the full specification. See [[cyber/tokenomics]] for the system-level economics (monetary policy, allocation curve, GFP flywheel). See [[collective learning]] for the group-level dynamics. See [[cyberlink]], [[focus]], [[neuron]], [[particle]], [[costly signal]], [[convergence vm]].
