---
tags: cyber, cybernomics, reference
alias: knowledge economy, epistemic economy, cyber knowledge economy, knowledge markets, syntropy
---
# knowledge economy

The mechanisms that make contributing to the [[cybergraph]] more profitable than free-riding -- and that make epistemic accuracy the unit of wealth. Includes the five epistemic asset classes, the focus reward, attribution mechanics, honest signaling, the GFP flywheel, and [[syntropy]] as the metabolic measure of collective intelligence.

---

## epistemic asset classes

The [[cybergraph]] creates a new category of financial asset. An epistemic asset is a claim on the [[knowledge]] economy's flow. Unlike financial assets (claims on future cash flows) or utility tokens (access rights to service capacity), epistemic assets yield returns proportional to the [[information]] contributed to collective [[intelligence]].

### cyberlinks

Yield-bearing [[knowledge]] claims. Every [[cyberlink]] accrues rewards over time as a function of the [[focus]] shift it generates:

$$R_{i \to j}(T) = \int_0^T w(t) \cdot \Delta\pi_j(t) \, dt$$

where $\Delta\pi_j(t)$ is the change in [[focus]] on target [[particle]] $j$ attributable to the link and $w(t)$ is the time-weighting function. Four reward trajectories: viral (high Δπ early, fast decay), foundational (low early, grows as graph builds around it), confirming (shared reward via [[Shapley]] attribution), semantic bridge (moderate, persistent, cross-module).

### eternal particles

Positions burned into permanence. Burning [[$CYB]] permanently anchors a [[particle]]'s π-weight -- the particle cannot be archived or deprioritized below the burn-weighted floor. The graph's long-term assertions: the claims whose importance the market cannot undo.

### eternal cyberlinks

Edges burned into permanence. The link cannot be forgotten by stake dynamics or [[ICBS]] market collapse. The graph's highest-conviction structural commitment.

### ICBS market positions

YES/NO bets on the epistemic market attached to every [[cyberlink]]. Position value grows as the market converges. Early conviction rewards are unbounded -- prices range from $0$ to $\lambda$. Capital flows from incorrect beliefs to correct ones. See [[epistemic-markets]] for the full ICBS specification.

### karma

The accumulated [[Bayesian Truth Serum|BTS]] score history of a [[neuron]]. Non-transferable. Structurally determinant: karma weights every future link the neuron creates in the [[tri-kernel]] effective adjacency. Epistemic capital -- the form of wealth that can only be earned by being right before the crowd.

---

## the focus reward

Every reward traces back to one quantity: how much did your action shift the [[tri-kernel]] fixed point $\pi^*$?

$$\text{reward}(v) \propto \Delta\pi(v)$$

$\Delta\pi$ is the gradient of the system's [[free energy]]. Creating valuable structure literally creates [[value]]. No designed loss function -- the physics of [[convergence]] defines what deserves to be optimized.

The hybrid reward function:

$$R = \alpha \cdot \Delta\pi + \beta \cdot \Delta J + \gamma \cdot \text{DAGWeight} + \epsilon \cdot \text{AlignmentBonus}$$

New [[$CYB]] is minted only when $\Delta\pi > 0$. The protocol's inflation is literally evidence of [[knowledge]] creation -- there is no emission without demonstrated contribution to collective [[focus]].

---

## attribution

Multiple [[neurons]] contribute [[cyberlinks]] in the same epoch. The total $\Delta\pi$ shift is a joint outcome. The [[Shapley value]] distributes fair credit: each agent's reward equals their average marginal contribution across all possible orderings.

Two approaches:

Conservative (scale factor): $R_i = \alpha \cdot \Delta\mathcal{F}_i + (1-\alpha) \cdot \hat{S}_i$ where $\Delta\mathcal{F}_i$ is the fast local estimate. $\alpha$ balances speed against fairness.

Shapley (Monte Carlo approximation): sample $k$ random orderings, measure marginal contributions, distribute proportionally.

Complexity: $O(k \cdot n)$ with $k \ll n$, feasible for $10^6+$ transactions per epoch.

---

## the 2|3 architecture

Each [[cyberlink]] carries three simultaneous signals:

1. Topology (binary): the edge exists -- the neuron asserts this structural connection
2. Market (continuous): the [[ICBS]] price -- the collective epistemic assessment of the link's validity
3. Meta-prediction (ternary): [[valence]] $v \in \{-1, 0, +1\}$ -- the neuron's prediction of market convergence

This produces a two-dimensional epistemic signal: price encodes magnitude, meta-score encodes collective confidence.

$$A^{\text{eff}}_{pq} = \sum_\ell \text{stake}(\ell) \times \text{karma}(\nu(\ell)) \times f(\text{ICBS price}(\ell))$$

---

## honest signaling via BTS

The [[cybergraph]] achieves honest markets through [[Bayesian Truth Serum]] (Prelec, 2004). The valence field in every [[cyberlink]] is the BTS meta-prediction -- no separate submission needed. Honesty is a Bayes-Nash equilibrium: no [[neuron]] can improve their expected score by misreporting belief or meta-belief.

[[Karma]] compounds the trust multiplier: consistently right before the crowd → high karma → more adjacency weight per link → more reward per contribution → more resources to stake on the next correct insight.

---

## the GFP flywheel

The optimal mining hardware and the optimal proving hardware are the same chip. The [[Goldilocks field processor]] exercises four primitives (fma, ntt, p2r, lut) for both PoUW mining and real workloads ([[stark]] proving, focus computation, neural inference). Mining rewards bootstrap chip development. Chips accelerate proving. Proving serves users. Users pay fees. Fees replace emission. No stranded assets.

---

## the evolutionary loop

contribute accurately → $\Delta\pi$ reward → accumulate [[$CYB]] → stake on more links → accumulate [[karma]] → links carry more adjacency weight → earlier $\Delta\pi$ attribution → more [[$CYB]] per contribution

The burn layer: burn on high-conviction [[particles]] → [[eternal particles|eternal weight]] → long-term yield floor → reduces risk premium for foundational contributions.

The result: the unit of wealth is provably epistemic accuracy. The only sustainable path to large [[$CYB]] balances, high [[karma]], and consistent ICBS returns is being right about what matters before the crowd recognizes it.

---

## syntropy

The pulse of the [[cybergraph]]. Syntropy measures [[order]] in [[bits]] -- the [[key metabolic factor]] of [[superintelligence]].

$$J(\pi) = \log|V| + \sum_j \pi_j \cdot \log(\pi_j)$$

This is the aggregate [[KL divergence]] from the uniform distribution -- the information gain of the focus distribution over maximum entropy. High syntropy means the graph is structured, connected, useful. Low syntropy means noise dominates.

Meaningful [[cyberlinks]] raise it. Spam and [[noise]] lower it. [[Tru]] computes syntropy every block in [[consensus]].

### per-neuron BTS scoring

Syntropy is aggregate [[KL divergence|information gain]] across all [[neurons]] in an epoch. A [[neuron]] whose [[cyberlinks]] sharpen collective certainty contributes positive syntropy. A [[neuron]] whose [[cyberlinks]] add [[noise]] contributes negative syntropy. The [[Bayesian Truth Serum|BTS]] score $s_i$ is syntropy measured at the level of one [[neuron]]: how many bits of information that neuron added to the collective picture.

### syntropy as metabolic factor

The [[approximation quality metric]] in [[focus flow computation]] uses $D_{KL}(\pi^*_c \| q^*_c)$ -- the same divergence measure -- to quantify how much the compiled [[transformer]] deviates from the exact [[focus]] distribution. The same mathematical object measures epistemic quality at three scales: individual [[neuron]] (BTS score), compiled model (approximation gap), and collective knowledge state (π* convergence).

see [[rewards]] for the detailed reward function specification. see [[epistemic-markets]] for the ICBS market mechanism. see [[truth-scoring]] for the BTS scoring layer. see [[cyber/tokenomics]] for the monetary plumbing. see [[functions of superintelligence]] for how the autonomous neuron participates in the same economy.
