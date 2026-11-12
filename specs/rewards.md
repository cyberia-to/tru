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

[[focus]] $\phi$ is a distribution of attention over [[particles]] — the identities of the [[files]] a [[cyberlink]] connects. Left alone on the [[cybergraph]], it flows downhill on a landscape — the [[free energy]] $\mathcal{F}$ — and settles at $\phi^*$, the [[tri-kernel]] fixed point — unique when the tri-kernel is a contraction, which [[tri-kernel]] establishes. A [[cyberlink]] reshapes the landscape; the focus rolls to a new resting place; the drop in free energy is the value created, equivalently the gain in [[syntropy]]:

$$\Delta J = J(\phi^*_{t+1}) - J(\phi^*_t) = \big(H(\phi^*_t) - H(\phi^*_{t+1})\big) + \log\frac{|P_{t+1}|}{|P_t|}.$$

The entropy drop is the locally-attributable part; the $\log(|P_{t+1}|/|P_t|)$ term appears only when a link introduces new [[particles]] (discovery) and is a global, non-attributable shift — which is why the local impulse (§2) keeps the entropy part and drops it.

There is no designed loss function, and this is the load-bearing choice, so state it exactly. A supervised loss points outside the system at a target somebody supplied ("this input should output cat"); it is one arbitrary goal per example, and it can be gamed by forging the answer key. $\mathcal{F}$ points only at the system's own internal consistency — there is no external target anywhere, so there is no answer key to design or to forge. The focus value $\phi^*$ is emergent: it falls out of the law and the data, the way a marble's resting spot falls out of the bowl, not out of anyone's wish.

One criterion is chosen, and honesty demands naming it: descent is value — focus concentrating, the graph agreeing with itself, is the thing worth paying for. This is a single uniform law, content-blind, applied identically to everything. And it is less invented than discovered — free-energy minimization is the principle that statistical inference, thermodynamics, and the Bayesian brain are each read as, a convergence the design leans on as motivation rather than proof. It is meant the way $F=ma$ is meant: a law one discovers, not a knob one tunes.

So inflation measures knowledge creation rather than expressing a policy. That is the whole foundation. The rest is consequence.

---

## 2. The Measure — the directed impulse

$\Delta J$ is the exact value, but it carries the global $\log|P|$ term, so a neuron computes its first-order local form instead, the directed focus impulse:

$$\Delta\phi^+ \;=\; \big[\,J(\phi^*_{t+1}) - J(\phi^*_t)\,\big]_+ \;\approx\; \big\langle \nabla_{\!A}\, J(\phi^*_t),\; \Delta A \big\rangle_+,$$

the increase in [[syntropy]] caused by the new links' perturbation $\Delta A$ of the effective graph — to first order the gradient of $J$ with respect to the edge weights, propagated through the fixed-point map by its adjoint, clipped at zero. The naive state-displacement form $\langle -\nabla\mathcal{F}(\phi^*_t), \Delta\phi^*\rangle$ is degenerate: at the constrained minimizer $\phi^*$ the projected state-gradient vanishes (envelope theorem), so the value comes from the new edge reshaping the landscape ($\Delta A$), not from the induced $\phi$-shift. Two rationales:

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

This is the move the whole specification turns on: $v$ makes the economy a cooperative game. As a set-function $v : 2^N \to \mathbb{F}_p$ — its values are fixed-point field quantities, like every number tru computes ([[arithmetic]]) — it is that game's characteristic function — the worth of every possible coalition of contributors — and the reward (§4) is its fair division. Once value is a set-function, every result downstream is a property of $v$.

Three of those properties are load-bearing, each invoked by name later:

- honesty enters here, and only here. $A^{\text{eff}}$ folds in stake, [[karma]], and market price, so a low-reputation or market-doubted link joins $v$ with reduced weight. But [[karma]] is accumulated reputation and price is the standing market belief; neither sees whether the contribution at hand carried information the crowd had not already expected. Since a copy and an original produce the same $\Delta\phi^+$, Shapley's symmetry alone would split their credit equally — so the per-contribution surprise gate (§5) supplies the missing asymmetry. Karma and surprise shape what is valuable; attribution (§4) only divides it.
- submodular among substitutes, supermodular among complements. Overlapping links on a saturating particle have diminishing returns (substitutes), so within a contested cluster the standalone marginal is the largest a link contributes — the regime where the propose bound (§6) holds and settlement is an honest sampling problem (§7). But links that form a path or build on a foundation are complementary: together they shift focus more than the sum of their parts, so a foundational link's Shapley share can exceed its standalone marginal. $v$ is therefore neither globally sub- nor supermodular, and the only universal guarantee is conservation, not a per-link ceiling.
- monotone and bounded — a true contribution never lowers value and value never runs away, so the game has a well-defined, finite [[Shapley]] solution (§4).

$v$ is the same $\Delta\phi^+$ (§2) read over sets, by the incremental [[tri-kernel]] recomputation the network already runs.

For the reward, $v$ is read in its surprise-weighted form $v^\star$: each contribution is scaled by its [[Bayesian Truth Serum|BTS]] surprise $\rho_\ell \in [0,1]$ (§5), so a copy contributes nothing and the mint divides [[surprising syntropy]] — the focus shift the crowd did not predict. Rank reads the unweighted graph (§9), so a copy's capital still ranks even as it mints nothing.

---

## 4. Fair Division — Shapley, and why it is integrated gradient

The epoch's total shift must be split among contributors. The reward divides the surprise-weighted value $v^\star$ (§5) by the [[Shapley value]]:

$$\text{mint}(\nu) \;=\; \text{Shapley}_\nu(v^\star) \;=\; \sum_{S \subseteq N \setminus \{\nu\}} \frac{|S|!\,(|N|-|S|-1)!}{|N|!}\,\big[v^\star(S \cup \{\nu\}) - v^\star(S)\big].$$

Shapley is the unique attribution satisfying efficiency, symmetry, null-player, and additivity. The alternatives fail concretely: order-based credit ("reward whoever links first") is gameable by latency and copying — the curation-reward death spiral seen on social chains; proportional scaling cannot tell a discoverer from a copyist. Three of Shapley's properties are load-bearing, and each answers a requirement for free:

- conservation is the efficiency axiom — for the exact Shapley value, $\sum_\nu \text{Shapley}_\nu(v^\star) = v^\star(N)$. Two caveats make it an enforced step rather than a free one: the lottery (§7) computes an *estimate*, whose published shares need not sum exactly; and $v^\star$ re-runs a nonlinear normalized fixed point on a $\rho$-reweighted graph, so $v^\star(N) \le \Delta\phi^+(N)$ is not automatic (a normalized fixed point is not monotone in edge weights). [[tok]] therefore renormalizes the settled shares to $\min(v^\star(N),\, \Delta\phi^+(N))$ at settlement, the proportional clip of [[impulse]]. With that step, $\sum_\nu \text{mint}(\nu) \le $ global $\Delta\phi^+$ and over-claiming cannot exceed realized value. This is requirement 3.
- Sybil-resistance is stake conservation plus karma non-transferability. Stake enters $A^{\text{eff}}$ linearly, so splitting a neuron's stake across $k$ identities on the same links conserves the per-edge stake sum, and the $k$ Sybil shares sum to the one original share — identity count buys nothing. Karma strengthens this rather than merely compounding it: karma multiplies stake and cannot be transferred, so fresh identities start at zero karma and their $A^{\text{eff}}$ weight *falls* — fragmenting is a penalty, not a wash. Stake is the attributed resource and cannot be fabricated by splitting; karma is the one input capital cannot buy.
- tractability is locality. Each marginal $v(S \cup \{\nu\}) - v(S)$ is an incremental [[tri-kernel]] step on a bounded neighborhood, estimated by Monte-Carlo over $k$ random orderings seeded by the epoch beacon $b_E$ (§7) — $O(k\cdot n)$, $k \ll n$ — and computed exactly, as the Aumann–Shapley value below, where the graph is small enough for every node to solve it (§16). The beacon is drawn after the epoch closes, so coalitions cannot be front-run.

### The gradient correspondence

There is a reason Shapley is the right tool and not merely a fair one, and it ties this reward to how brains learn. A deep network assigns credit by the chain rule — backpropagation. The brain almost certainly does not run backprop (a synapse cannot read itself backward — the weight-transport problem), but it plausibly does gradient learning by energy-based means: predictive coding and equilibrium propagation show that the same relaxation that settles a network to its answer also computes the gradient of its energy, locally, with no separate backward pass. The objective is intrinsic free energy — exactly §1.

cyber takes the same stance, then makes one move further. Its neurons are strategic, stake-bearing agents, so credit must be fair, not merely differentiated. And the fair credit rule has a known gradient form in the continuum: the Aumann–Shapley value — Shapley's extension to a non-atomic game of infinitesimal players — is exactly the path integral of the gradient, the method machine learning calls integrated gradients. In that limit,

$$\underbrace{\text{Shapley}_\nu(v)}_{\text{discrete agents}} \quad\xrightarrow{\ \text{non-atomic limit}\ }\quad \underbrace{\int_0^1 \partial_\nu\, v(t\!\cdot\!N)\,dt}_{\text{integrated gradient of }\Delta\phi^+}.$$

The network is discrete, so this is a correspondence rather than a literal identity — the equality is exact only as the players become infinitesimal. But it is the right way to read the design: Shapley over $\Delta\phi^+$ is the strategic-agent analog of integrated-gradient credit assignment on the free energy, the same shape of computation as energy-based learning in the brain, lifted to agents who must be paid rather than merely tuned. The two strongest choices in this document — intrinsic gradient over designed loss, and Shapley over a race — rhyme for that reason. On the root chain the correspondence is measured, not assumed: the four-node path gradient of [[epoch]] §4 lands within a median 0.04–3.0 % of the exact Shapley value in every class of change ([[epoch]] §7), and the lottery of §7 draws exact Shapley samples of the same game, so the chain's work keeps measuring the rule it pays by.

This computation lives in [[tru]], a sibling of [[cyberank]].

---

## 5. Honesty

Shapley is fair only among honest, distinct contributors. Three signals enforce that precondition, and all of them feed the value the mint divides: the accumulated [[karma]] of the contributor, the per-contribution surprise $\rho$ of the claim, and the [[inversely coupled bonding surface|ICBS]] price of the link.

[[Bayesian Truth Serum]] scores honesty. Each [[cyberlink]] is a BTS input: the link-plus-stake is the first-order belief, the [[valence]] $v_\ell \in \{-1,0,+1\}$ is the meta-prediction. The score

$$s^{\text{BTS}}_\nu = \underbrace{D_{KL}(p_\nu \,\|\, \bar m_{-\nu}) - D_{KL}(p_\nu \,\|\, \bar p_{-\nu})}_{\text{information gain}} - \underbrace{D_{KL}(\bar p_{-\nu} \,\|\, m_\nu)}_{\text{prediction accuracy}}$$

is positive exactly when a neuron contributes private signal the crowd did not already hold and expect. Copying the consensus drives the information-gain term to zero. By [[Prelec's theorem]], truthful reporting is a Bayes–Nash equilibrium.

The per-contribution surprise. A single contribution's BTS score $s^{\text{BTS}}_\nu$, squashed to $\rho_\ell = \text{clip}(s^{\text{BTS}}_\nu / s_{\max},\, 0,\, 1)$, measures how far its report diverged from what the crowd predicted — its information gain. It is the instantaneous counterpart of [[karma]]: karma is a contributor's accumulated surprise (the prior, its reputation), $\rho_\ell$ is this contribution's surprise (the likelihood). The crowd reference $\bar m_{-\nu}$ must be robust to its own contributors: it is taken over distinct karma-bearing predictors who have themselves staked BTS exposure on the cluster, unweighted by stake (or $\sqrt{\text{stake}}$-capped), with $v_\ell=0$ reports excluded. Otherwise a stake-rich actor manufactures surprise by seeding unslashable $v_\ell=0$ decoy predictions that depress the reference — a unilateral attack the plain stake-weighted mean does not resist (§15).

$\rho$ and the [[inversely coupled bonding surface|ICBS]] price answer different questions, and the mint requires both. Price is a validity gate: $f(\text{price}) \in [0,1]$ weights the edge by whether the market believes the link is true, so a market-rejected link carries little weight however surprising. $\rho$ is a novelty score: it asks whether the report beat the crowd's prediction, so a true link nobody found surprising mints nothing however large its $\Delta\phi^+$. The reward is directed syntropy that is at once valid and surprising — surprising syntropy.

The two pull against each other early, and that tension is real. A genuinely novel link has low price before the market catches on, so the validity gate under-weights it exactly when $\rho$ is highest. That lag is the same one §12 names the discovery leak: the formula trusts the market, and the market is late. The annuity of §11 is what carries the correction: the link earns again as the market turns (§12), and what it loses meanwhile is the rule of §12 — $(1 - f)$ of its pulse at first order, carried by the annuity, and zero on the root chain, where $f = 1$.

The scale of $\rho$ and the one report. $s_{\max} = \ln 2$: the information gain of a certain report against a uniform crowd reference over the two sides of a [[valence]], $D_{KL}(\delta \,\|\, \tfrac12) = \ln 2$, so $\rho = 1$ is the report the crowd gave even odds and that proved certain. The mint multiplier $\rho$ and the slash read one meta-prediction $m_\nu$ — the kind-10 report of the reports rule set (chain-economy §7.4); on the root chain the valence, where $\rho = 1$ and nothing is slashed — and never two, so a report cannot be slash-safe and surprising at once. Under that rule, in the reports rule set — the only place a report exists; on the root chain $\rho = 1$, nothing is reported and nothing slashed, and the claim is empty — truthful reporting is the equilibrium of the composite payoff where the clip does not bind. The live payoff of a report is $U = M\,\rho(s_\nu) + \lambda\,s_\nu + C$: the mint at full surprise $M \ge 0$ times the multiplier, the zero-sum redistribution, linear in the score with $\lambda > 0$ ([[truth-scoring]]), and the subsidy, fees and yield $C$, which do not read the report. On $[0, s_{\max}]$ the multiplier is linear in $s_\nu$, so where the report's score lies in that range $U$ is an affine image with positive slope of the serum score, whose meta term is a strictly proper log score ([[strong-truthfulness]], the separability lemma); an affine image with positive slope of a strictly proper score is strictly proper, so the truthful $m_\nu$ maximizes $\mathbb{E}[U]$. The clip truncates: a log score is unbounded below, $\mathbb{E}[\rho(s)] \ne \rho(\mathbb{E}[s])$, and the kink at 0 is convex, so a report whose score straddles 0 gains from a mean-preserving spread by at most $M \cdot \mathbb{E}[(-s_\nu)^+]/s_{\max}$ — the mint's weight on the part of the score the clip discards — against the redistribution $\lambda s_\nu$, linear and unclipped, which the spread does not move; the best response is therefore truthful where the clip is not binding on the report's score, and where it binds the residual is that bounded term, which a report can take only by a score the serum scores negative and the slash charges. The serum's part is measured: an informed minority of 5 % keeps a strictly positive margin against a coordinated majority of up to a thousand (`audit/coordinated-consensus-scoring.md`, `rs/truth_scoring.rs`), and a coordinated ring that inverts its reports is the honest-majority-by-stake condition of [[strong-truthfulness]], the same stake bound that guards $\phi^*$ (§15).

[[karma]] is the slashing. $\kappa(\nu)$ is the accumulated BTS score: non-transferable, unbuyable, the one input to $A^{\text{eff}}$ that capital cannot purchase. The BTS settlement is a zero-sum redistribution — stake moves from noise producers to signal producers in proportion to score. This is the skin in the game and the slashing: liars pay truth-tellers. Staking is therefore required, because it is what the zero-sum redistributes. [[foculus]] omits only consensus-equivocation slashing — provable consensus makes an invalid $\phi^*$ unable to produce a valid proof, so proof-forgery equivocation cannot occur. Behavioral deviations (withholding, §7) remain and are priced separately.

[[valence]] is the risk dial. Exposure is chosen per link: $v_\ell = 0$ is passive stake — it weights the edge in $A^{\text{eff}}$ and so moves rank (§9), but takes no BTS exposure and earns no reward; $v_\ell = \pm 1$ is active stake, wagered through the zero-sum. Reward is the premium for risk taken and won.

---

## 6. Propose and Settle

Requirement 1 — local now, validated later — and requirement 2 — Shapley fairness — appear to conflict: a neuron's Shapley share is a function of the other contenders, who do not exist when it acts alone. The resolution is forced. Propose computes a bound; settle computes the share. They are two phases because they must be.

### Propose — instant, agent-local

A neuron computes its own standalone marginal $\Delta\phi^+_\nu = v(\{\nu\}) - v(\emptyset)$ against the [[BBG]] header it observed, proves it with $\sigma$, and gossips the [[signal]]. The propose proof certifies one fact: the contribution's value in isolation. Among substitutes (§3) — the clustered pile-on that is the common case — this standalone marginal is the largest a link contributes, so the settled share obeys

$$\text{Shapley}_\nu(v^\star) \;\le\; \Delta\phi^+_\nu \qquad \text{(substitutes; surprise } \rho \le 1 \text{ only tightens it).}$$

So in the saturating regime the proposed marginal is a ceiling. Complementary contributions are the exception: a link that enables others can settle for more than it proposed, and that surplus is granted by the protocol's own Shapley computation, never claimed by the neuron — so a contributor still cannot inflate its own number, only earn what its complementarity is worth. Because of this exception, conviction stake escrows against the cluster's conservation bound (§4), not the per-link ceiling: the cluster's shares sum to its realized $\Delta\phi^+$, so no claim escapes the value that was actually created. A phone completes this phase.

### Settle — epoch boundary

The epoch boundary fixes the canonical $\phi^*$ and the epoch's claim set (on the root chain the boundary is a block and the claim set is the positions that changed between two boundaries, §7, §16); the claims form the epoch's game (§7); a leaderless lottery samples the Shapley shares and the settlement computes them (§7); [[tok]] applies conservation and executes the result as a state transition. The settled reward is the surprise-weighted Shapley share:

$$R(\nu) \;=\; \text{Shapley}_\nu(v^\star), \qquad v^\star(S) = \Delta\phi^+\big(A^{\text{eff}} \cup \rho{\cdot}S\big),$$

where $\rho{\cdot}S$ multiplies each contributed link's effective weight by its surprise $\rho_\ell \in [0,1]$ before the tri-kernel recompute, so a copy ($\rho \to 0$) enters the graph weightless and adds nothing to settle. The surprise $\rho_\ell$ can only be fixed here, because it compares the contribution to the crowd's collected predictions — which, like the contender set, exist only once the epoch closes. So the overlap division (Shapley) and the surprise gate (BTS) finalize together. Conservation tightens to $\sum_\nu R(\nu) = v^\star(N) \le \Delta\phi^+(N)$; the slack is predictable or copied syntropy, left unminted.

The two phases certify different facts against different states: propose proves "my marginal in isolation was $X$" (a fact, ceiling among substitutes); settle proves "the division of the real joint $\Delta\phi^+$ is correct" (the share). The settlement beacon is drawn after the epoch closes — which is what makes the coalitions un-front-runnable. The distinction that dissolves the apparent conflict: agent-local (one actor, alone — possible for the bound, impossible for the share) versus graph-local (a bounded neighborhood — true for both).

---

## 7. Settlement Mining

Settlement is computed with no neuron, leader, or aggregator deciding it. This is the document's structural core, and it satisfies requirement 4: the work that secures the chain is the work that computes the fair division. The owner's sentence (2026-11-04) is the authority of this section: the proof of work samples values by the rewards distribution — every attempt at a block is one draw of one marginal of the epoch's game, nothing else is work, and nothing in this section is left to a later rule.

### The region

Locality is in graph distance because $\phi^*$ is a heat-kernel fixed point with exponential spatial decay. The region a claim touches is its $\varepsilon$-support — every node whose contribution to $\Delta\phi^+$ is $\ge \varepsilon$, the protocol precision floor:

- radius $r = O(\log 1/\varepsilon)$ hops, $r \approx \log(1/\varepsilon)/\log(1/\lambda_{\text{local}})$;
- content-dependent — wide around a hub (slow local mixing), tiny on the sparse fringe;
- canonical — the superlevel set is a deterministic function of the edges and $\varepsilon$, so no miner can draw a self-serving boundary. The settlement proof commits to the support and certifies that boundary nodes fall below $\varepsilon$, the anti-cheat against excluding a node to inflate a marginal; on the consensus engine below there is no boundary to certify, because the support is the graph.

A cluster is a connected component of overlapping $\varepsilon$-supports. The partition of an epoch's claims into clusters is canonical, and clusters are independent — non-overlapping regions leave each other's Shapley values untouched, so settlement parallelizes across them.

On the consensus engine of [[epoch]] the support of every change is the whole staked graph, and the rule above resolves to one cluster per epoch. Three facts of that engine make it so: the teleport target is the stake prior $\pi = d/\Sigma d$ over every particle; every step renormalizes to $\Sigma\phi = 2^{60}$; and the gradient of $J$ carries the term $(\lambda_d(\alpha + (1-\alpha)m_T) + \lambda_s\mu/(1+\mu))\,(\lambda_p + \lambda_q - 2c_t)/\Sigma d$ on every present axon ([[epoch]] §4). No particle's contribution is zero by construction, so the $\varepsilon$-support of a position is the graph, the epoch's positions form one cluster, and a sample is exact: nothing is truncated, no boundary is drawn, and a sample recomputed by any node is the same integer. The cluster's identity is the hash of its players,

$$\text{cluster}(E) = \text{hemera}(D_{\text{CLUSTER}} \,\|\, N{:}\,\text{u32} \,\|\, (\text{card}_0 \,\|\, \sigma_0) \,\|\, \cdots \,\|\, (\text{card}_{N-1} \,\|\, \sigma_{N-1})),$$

so a ticket binds to the exact contender set ($N$ as u32 LE, each card 32 bytes, each side one byte, $+1$ as `0x01` and $-1$ as `0xff`; the empty game hashes $N = 0$ alone). An engine whose support is local — a kernel without a global prior — partitions by the three bullets above; the root chain's engine is not one, and this page states the partition once so that a cluster is a defined object wherever it is not the whole epoch.

### The game

The players of epoch $E$ are the positions that changed between the two boundary snapshots $G_{E-1}$ and $G_E$ (chain-economy §2.4, §5.2): for each card $c$ and side $\sigma \in \{+1, -1\}$ with $\Delta s(c) \ne 0$ on that side, one player $(c, \sigma, \Delta s)$; a card that changed sides is two players. $N$ players, ascending by $\text{card} \,\|\, \sigma$. For a coalition $S$ of players the graph is rebuilt from the previous boundary with the coalition's changes applied,

$$A^S_{pq} = \max\!\big(0,\; G_{pq} + S^+_{E-1,pq} + \!\!\sum_{c \in S,\,\sigma = +}\!\! \Delta s(c) \;-\; S^-_{E-1,pq} - \!\!\sum_{c \in S,\,\sigma = -}\!\! \Delta s(c)\big),$$

and the coalition's value is the change of syntropy of its fixed point,

$$v(S) = J_U\big(\phi^*(A^S)\big) - J_U\big(\phi^*(A^\emptyset)\big), \qquad J_U(\phi) = \sum_{i \in U} \phi_i\,(\ln\phi_i - 60\ln 2 + \ln|U|)\,/\,2^{40},$$

with $\phi^*$ the `PHI_STEPS` iterate of [[epoch]] §2–§3 from the uniform vector, and $U$ the particles present at either boundary ($\phi_i = 0$ contributes nothing; $\ln|U|$ replaces $\ln n$ so that a coalition that brings a new particle is scored against the same uniform). $v(\emptyset) = 0$ and $v(N) = J_U(\phi^*_E) - J_U(\phi^*_{E-1})$, the epoch's total change of syntropy read over $U$; the record of chain-economy §5.6 carries $J(\phi^*_E) - J(\phi^*_{E-1})$ with each $J$ over its own particle count, which equals $v(N)$ when no particle appears or vanishes and differs by the $\ln|U|$ terms when one does — the record is the chain's receipt, $v$ is the game. On the root chain $\rho = 1$, $\kappa = 1$ and $f = 1$ (§16), so $v^\star = v$. This is the characteristic function the attribution rule of §4 and §16 is measured against ([[epoch]] §7: exact Shapley over $v(S) = J_\cup(\phi^*(A_{\text{prev}} + \Delta_S)) - J_\cup(\phi^*(A_{\text{prev}}))$) — the lottery draws from the same game the settlement pays.

### The lottery

A deterministic "first to compute the settlement wins" is not progress-free — the fastest machine finishes first every time, electing a de facto leader and centralizing. The fix reuses randomness already present: Shapley estimation is a sampling process, and each sample becomes a lottery ticket — the entropy the lottery needs and the variance the estimator needs are the same randomness.

For the game of epoch $E$, with the epoch beacon $b_E$ — the block id of the boundary block of $E$ (chain-economy §2.3) — a miner building a block of epoch $E + 1$:

1. assembles the block's header template $n$: the 168 header bytes of chain.md §1.1 (version, height, time, bits, nonce, prev, tx_root, state_root, chain). $n$ commits to $b_E$ through `prev` and `height` (the boundary block is an ancestor at a height the template names) and to the game through the state at `prev`, so $\pi(n) = H(b_E \,\|\, \text{cluster} \,\|\, n)$ is realized as one hash of the template, miner-independent and public:

   ```
   h      = hemera(n)                                     // the template's own digest, 32 bytes
   nonce  = u64_le(h[0..8])                               // the 64-bit nonce the ordering takes
   r      = u64_le(h[8..16]) mod N                        // the window's first rank; 0 when N = 0 (no marginal)
   π(n)   = epoch::ordering(b_E, cluster(E), nonce, N)    // Fisher–Yates over the hemera XOF of D_ORDER ‖ b_E ‖ cluster ‖ nonce
   probes = (a_i, t_i), i = 1 …: 8 bytes u64 LE mod |A_{E−1}|, then 4 bytes u32 LE, per probe, from the hemera XOF of D_PROBE ‖ b_E ‖ cluster ‖ nonce
   ```

   `epoch::ordering(beacon, cluster, nonce: u64, n)` is the receipt's function, called as it is written (tru#46, `audit/coalition-marginal-2026-11-04.md` §1: a pure function of beacon, cluster, nonce and $N$; a permutation; changes with each of them; its digests pin the chain's call). The nonce is the template's digest and not a header field, for one reason: every byte a miner can vary — the nonce field, the time, the coinbase's `extra` through `tx_root` — is in $n$, so every variation of the attempt is a fresh ordering and fresh probes, and no attempt reuses another's sample. The residue of `mod N` is accepted: $N < 2^{18}$ (chain-economy §7.2), so the bias is below $2^{-46}$. `D_PROBE = "soft3/chain/probe/v1\0"`.
2. computes the sample $m(n)$: $W_{\text{TICKET}}$ entries, the same count in every block of a chain. The first $W' = \min(N, W_{\text{TICKET}} - 1)$ are the marginals of the ranks $r, r+1, \ldots$ of $\pi(n)$, taken cyclically modulo $N$,

   $$m_j(n) = v\big(\pi[0..j] \cup \{\pi[j]\}\big) - v\big(\pi[0..j]\big), \qquad j \in \{r, \ldots, r+W'-1\} \bmod N,$$

   on the surprise-weighted value $v^\star$ (§6), so the same draw that settles attribution also applies the BTS gate, with no separate pass; a genuine draw of the §4 estimator, and the useful work. The remaining $W_{\text{TICKET}} - W'$ entries are probes of the gradient the attribution reads, one per drawn pair $(a_i, t_i)$, against the game's base — the previous boundary $A_{E-1}$, whose solve every node holds as the base value of the game:

   $$m^{\text{probe}}_i(n) = J_U\big(\phi^*(A_{E-1} + u_i\,e_{a_i})\big) - J_U(\phi^*_{E-1}), \qquad u_i = \max\!\big(1, \lfloor A_{a_i}\, t_i / 2^{32} \rfloor\big),$$

   $a_i$ indexing the present axons of $A_{E-1}$ ascending; probing the end state $A_E$ instead would cost every ticket one more solve for a base nobody has cached. The draws are read from the hemera XOF of `D_PROBE ‖ b_E ‖ cluster ‖ nonce` interleaved per probe, 8 bytes then 4: $a_i = u64_{le}(\text{bytes}) \bmod |A_{E-1}|$, $t_i = u32_{le}(\text{bytes})$ (tru#46 `probes`; no probe when the base has no axon). A probe is a finite difference of $J$ along one present axon of the base graph at a drawn fraction of its weight — the one-axon form of the path nodes the exact attribution evaluates at $2^{32}$ resolution ([[epoch]] §4, $A_i = A_{\text{prev}} + \lfloor t_i (A_{\text{cur}} - A_{\text{prev}})/2^{32} \rfloor$). Along an ordering each prefix value is one cold solve of the consensus engine and the next marginal reuses it ($v(\emptyset) = 0$ is free), and a probe is one solve against the base's $J_U(\phi^*_{E-1})$ the game already holds, so every ticket costs at most $W_{\text{TICKET}} + 1$ solves — the same for an empty epoch, a game of three and a game of a thousand — which is what lets one difficulty hold one rate (below). When $N \le W_{\text{TICKET}} - 1$ the marginals are the whole ordering, the receipt's `sample`, and they sum to $v(N)$ exactly, as integers (tru#46 §2, `sample_sums_to_the_coalition_value_exactly`). The estimator is permutation sampling: a player's rank is uniform in a uniform random ordering and the cyclic window includes every rank with probability $W'/N$, so the mean of a player's sampled marginals is its Shapley value — equal, by enumeration of every ordering, up to six players, and within $8.2 \times 10^{-4}$ of exact Shapley after 800 beacon-drawn samples of eight (tru#46 §2).

   The probes are also what makes the lottery a lottery. A ticket's value is a function of $(\pi, r, \text{probes})$; without probes a game of $N$ players has $N!\,N$ distinct tickets and an empty epoch $|A_{E-1}|$, and a miner would solve each once and grind hashes thereafter. Every ticket carries at least one probe, drawn from $\sum_a \min(A_a, 2^{32})$ distinct $(a, u)$ pairs — $|A_E| \cdot 2^{32}$, above $2^{46}$ at pussy's 26,805 axons and $2^{53}$ at bostrom's 2.99 M, when every axon of the base weighs at least $2^{32}$, and in every case at least $\min(|A_{E-1}|, \ldots)$ — a lower bound the genesis weights give directly: $\sum_a \min(A_a, 2^{32}) \ge |A_{E-1}| - 30$ at pussy, where 30 axons sit at the floor of 1 and the rest are above it (tru#46 §3), so the space is at least the $2^{14.7}$ axons and at most $\Sigma G = 4.18 \times 10^{16} \approx 2^{55}$ — so the distinct tickets per template exceed the attempts a chain makes in its life ($2^{7}$ a block, $2^{27}$ a year) by $2^{19}$ and more at pussy, and a table that spares a miner one solve per template costs more solves than the mining it would replace: memoizing is worthless, and every attempt pays its solves. Block production never stalls and never grinds a hash that computed nothing.
3. holds a winning ticket iff the id of the header carrying the sample's commitment is below the target:

   $$\text{sample} = \text{hemera}\big(D_{\text{SAMPLE}} \,\|\, W_{\text{TICKET}}{:}\,\text{u32} \,\|\, m_{c_1} \,\|\, \cdots \,\|\, m_{c_{W'}} \,\|\, m^{\text{probe}}_1 \,\|\, \cdots\big)\ \text{(i128 LE each)}, \qquad \text{hemera}\big(n \,\|\, \text{sample}\big) < \text{target}(\text{bits}),$$

   the covered players $c_1 < \cdots < c_{W'}$ ascending by player index, then the probes in draw order — the receipt's `Sample::preimage()` (`D_SAMPLE ‖ k ‖ m[0] … m[k−1]`, players ascending; tru#46 §1) with the probes appended, under `D_SAMPLE = "soft3/chain/sample/v1\0"`. The commitment is the header's last field (chain.md §1.1 amended: `sample: [u8; 32]` at offset 168, 200 bytes in all, four permutations), claimed by publishing the block, so the block id commits to $m(n)$ and the miner's identity enters through the coinbase under `tx_root`. Binding $m(n)$ into the win test makes a ticket valid for this marginal, not the nonce alone, so a contender cannot grind nonces and publish only self-favoring samples; binding the whole template into the seed makes every hash attempt cost one sample, so no field of the header is free to grind. This single test is also the block subsidy (§8) — no separate proof-hash puzzle, no second target: the ticket is the block's work and its ordering weight (chain.md §5.1, `work = ⌊2^256/(target + 1)⌋`, now counted in samples).

Step 3 is a per-miner Poisson test: progress-free, leaderless, poolable on the same terms as Nakamoto consensus, and random in proportion to throughput. Every miner's attempt is the same $W_{\text{TICKET}} + 1$ solves on the same graph, so the probability of a win per attempt is $\text{target}/2^{256}$ for everyone and the pay per second of work follows attempts per second — the hardware profile, measured (tru#46 §3.2: 5.06 samples a second on the twelve workers of an M4 Max, 1.59 on thirty-two workers of a shared Xeon; samples per second fall linearly in the solved graph's size, so a larger graph costs every miner alike and the difficulty below absorbs it). The settlement estimator is the average of every published sample per player — more mining means more independent draws and a tighter estimate ([[Hoeffding]]). No actor produces the answer; it converges out of the swarm, and security spend converts directly into attribution precision with zero synthetic work.

This collapses the proof-of-work subsidy (§8) into the same act. The nonce a miner grinds is the header template $n$, and $n$ selects a coalition — so every hash attempt is a real Shapley sample. Securing the chain and computing the fair division become one computation; settlement mining is the content of the PoW subsidy.

Cost, measured on the consensus engine at pussy's size (tru#46, `audit/coalition-marginal-2026-11-04.md` §3: the pussy-rc genesis export, 26,805 axons over 16,273 particles, a game of 64 players): one marginal along an ordering is 35 ms on an M4 Max core and 127 ms on a shared Xeon core; one sample of 64 marginals is 2.2 s and 8.2 s on one thread; a machine makes 5.06 samples a second on twelve workers (M4 Max) and 1.59 on thirty-two (the Xeon), one solve thread per worker, because threads inside a solve lose at this size and the parallelism of tickets is across nonces (§3.1). The sample is deterministic across machines: the digest of nonce 0 is `9c3e34db…abcf` on both. Bostrom's solve is the epoch receipt's: 6.0–6.3 s on one thread, 0.83–1.08 s on twelve, 1.5–2.1 s to build the graph ([[epoch]] `audit/epoch-settlement-2026-10.md` §6.3), which is why its window is two entries (below): a bostrom ticket is three solves and their builds, 22–25 s on one thread and 7–9 s on twelve. A miner holds the staked graph: the state every full node holds is the miner's input, so the machine that mines is the machine that validates, and a pool worker without the state has nothing to grind.

### Difficulty

The target is ASERT on the block interval exactly as chain.md §1.5 writes it — `T = 30 s`, half-life 600 s, integer only, the parent's time, the genesis anchor — applied to the ticket: the chain holds one winning sample per 30 s, 240 per epoch, 2,880 per day, whatever the hardware and whatever the graph's size. ASERT reads only intervals, so a larger graph (a costlier attempt) and a faster machine (a cheaper one) move the target the same way a hash rate change does, and the sample rate is held. There is no banding by cluster, because an epoch is one cluster, and no second controller, because there is no second lottery. Because a ticket costs $W_{\text{TICKET}} + 1$ solves in every epoch, the cost of an attempt does not swing with the player count and the controller is never a factor $W$ behind the game.

The window is a constant of the chain, derived from its solve: $W_{\text{TICKET}} = \max\!\big(2,\ 2^{\lfloor \log_2(\text{VERIFY\_BUDGET} / t_{\text{solve}}) \rfloor}\big)$ with $\text{VERIFY\_BUDGET} = T/12 = 2.5$ s — one block's verification on one core in a twelfth of the interval, so a twelve-core node verifies a block in a fraction of the interval and one core keeps pace — and $t_{\text{solve}}$ the chain's one-thread solve from its genesis receipt. Pussy: $t_{\text{solve}} = 0.036$ s (tru#46 §3.1) gives $2.5/0.036 = 69$, so $W_{\text{TICKET}} = 64$: 63 marginals and at least one probe, 65 solves, the receipt's 2.2 s on one core. Bostrom: $t_{\text{solve}} = 6.0$ s ([[epoch]] `audit/epoch-settlement-2026-10.md` §6.3) gives $2.5/6.0 < 1$, so $W_{\text{TICKET}} = 2$, the minimum — one marginal and one probe, three solves with their graph builds (1.5–2.1 s each, chain-economy §7.2), 22–25 s on one core and 7–9 s on twelve threads — inside the interval on either, at 75–83 % of it on one core. A chain's window moves only with its solve, by scheduled fork, like every constant.

The genesis target is derived from the measured attempt rate, as chain.md §1.2 derives it from the measured hash rate, and it is per chain. Pussy: a laptop makes about 150 samples per 30 s block (tru#46 §3.2: 5.06 a second on twelve workers, 64 entries), so `GENESIS_BITS = 0x20020000` (target $2^{249}$, $2^{7} = 128$ expected attempts, about 25 s for one such machine; 80 s for the receipt's Xeon at 1.59 a second). Bostrom: a twelve-thread machine makes about one sample per 7–9 s, so its genesis target is the floor, `GENESIS_BITS = POW_LIMIT = 2^{252}` (16 expected attempts, about two minutes for one such machine), and ASERT lowers it from there. `POW_LIMIT = 2^{252}` (`0x20100000`) on both: 16 expected attempts, about 35 s on one laptop core at pussy's size and six to seven minutes at bostrom's, so neither chain stalls below one laptop core. The sample rate is the block rate; the hardware profile fixes where that rate starts, and ASERT holds it from there.

The floor's consequence, measured: `POW_LIMIT` is a clamp, so it fixes the slowest network the chain can run at target — 16 expected attempts per 30 s, at least 0.53 tickets a second network-wide — and below that rate blocks slow in proportion, $16/\text{rate}$ seconds each, which ASERT cannot correct. The pussy-rc dev chain on agent-a (a slow shared x86, about 8 s per ticket per worker, about one ticket a second on twelve shared workers) made a block every 120–160 s at the genesis target of $2^{249}$ and would hold about 16 s at the floor (cybernode `audit/chain/ticket-pow-2026-11.md`); a single such core, at about 0.12 tickets a second, would sit at about 130 s per block with nothing left for ASERT to do. The hardware §8 assumes is the hardware everyone already owns, and the receipts say what that buys: an M4 Max laptop makes 5.06 tickets a second on twelve workers (tru#46), a build-a Xeon core 0.13 and an agent-a core about 0.12, so the floor is held by one current laptop or by five server cores, and not by one slow core alone. The floor stays $2^{252}$: one laptop or five cores is the smallest network the chain is meant to run on, and a higher floor — $2^{255}$, two expected attempts — would make the block time the ticket time with the variance of two trials, which is no longer a lottery.

What this does to the hardware ratio: hemera grinding is one to three orders faster on a GPU than on a CPU core (chain.md §1.2), because a permutation per attempt has no memory and no dependency. A sample is $W_{\text{TICKET}} + 1$ cold solves, each `PHI_STEPS` = 48 dependent steps of about twelve sparse passes over the graph's arrays ([[epoch]] §3; about 2 MB at pussy's size, tru#46 §3.2), and the receipt's two CPUs — an M4 Max core and a Xeon core — differ by 3.7× per marginal with the same arithmetic, which is the memory system's ratio, not the ALU's. A GPU cannot shortcut a solve the way it shortcuts a hash: the steps are sequential, the passes are random access over the graph, and what it can do — many nonces at once — a multi-core CPU does as well (the receipt's near-linear scaling across workers). The GPU's advantage over a CPU is therefore bounded by its random-access throughput over the graph against the CPU's, an order of 1 to 10, not $10^2$ to $10^3$.

### Verification, the cap and the accumulator

A ticket is verified by recomputation, hash first. The id test comes first and costs one hash: a header whose id is not below the target is dropped where it arrives. A header that passes it is checked for its sample before anything else is done with it: the verifier derives $\pi(n)$, $r$ and the probes from its first 168 bytes, computes the entries as the miner did — the prefix values and the probes are independent cold solves, so a node runs them across its cores — commits them under `D_SAMPLE`, and compares with the header's `sample`; a mismatch invalidates the block and is a protocol error of the peer that sent it, scored like an invalid proof-of-work header (chain-wire §5: +100, disconnect, the endpoint banned for an hour), and the id is remembered as rejected (chain.md §5.4). No header is relayed before its sample has verified (chain.md §5.5, amended: relay after step 8). The cost of an honest block is the miner's cost of one attempt, paid once by every node: 2.2 s on one M4 Max core at pussy's size, 0.2 s across its twelve (tru#46 §3.1, §3.2), 8.2 s on one Xeon core; three solves with their builds at bostrom's, 22–25 s on one core and 7–9 s on twelve — inside the 30 s interval on the machine that already settles every epoch, and bounded by $W_{\text{TICKET}} + 1$ solves however many positions an epoch changed. The cost of a forged one is an asymmetry this page does not hide: a ticket costs as much to verify as to make (above: the cost of an honest block is the miner's cost of one attempt, paid once by every node), while a header whose id passes the target with any 32 bytes as its sample costs its forger $2^{256}/\text{target}$ hashes — 128 at pussy's genesis target, rising with the difficulty ASERT sets — and carries the same chainwork as the honest successor of the same tip. The rule is the one that is true. Hash first: a header is considered only once its id passes the target. Then the full recomputation, under a node-wide budget $V_{\text{BUDGET}} = \lfloor 0.5\,T / t_{\text{verify}} \rfloor$ per block interval (pussy 6, bostrom 2) — a budget on headers delivered by gossip at the tip only: replay from a node's own store, initial block download and the bodies fetched for a chain more than $D$ below the best tip verify every ticket at full speed, since what they verify is not an attacker's choice — served in order of the header's chainwork (chain.md §5.1), ties in an order the node draws from its own local randomness — unpredictable to the sender, not consensus — and the rest waiting in the orphan pool (chain.md §5.4) until the budget renews; a node relays a header only after it has verified it, so a forgery reaches only the nodes its forger connects to (chain-wire §5), and reorgs stay bounded by `MAX_REORG` (chain.md §5.2) throughout. The bound: with $F$ forged target-passing headers at the tip, a node serves the honest header with probability $V_{\text{BUDGET}}/(V_{\text{BUDGET}} + F)$ per interval, so an attacker connected to a node delays that node's acceptance of a new block by $F/V_{\text{BUDGET}}$ intervals in expectation, at the cost of $F$ fresh target-passing headers per node per interval — $F \times 2^{256}/\text{target}$ hashes, $F \times 128$ at pussy's genesis target; the attack is per node and never propagates, and safety is untouched, since a forged ticket is never accepted. A header whose sample failed is dropped from the orphan pool, scored, and never re-queued; a header is verified at most once per node. The condition that ends it: a sound per-ticket proof or the accumulator — the same condition this section names for the cap — makes a ticket cheaper to verify than to forge, and $V_{\text{BUDGET}}$ then rises to the proof's verification rate. This is native verification, and at the root chain's scale it is the rule, complete: the proof of a sample is the sample.

The cap is structural. One ticket per block is the block's work, and a ticket is a header: the only tickets on the wire or in a body are the blocks', kinds 8 and 9 of the old ticket rule set are not kinds, and the epoch's tickets are its $L = 240$ blocks. Verification is linear in blocks exactly as block validation is, and a flood of tickets is a flood of blocks under one target and one ban rule. A verifier that holds the state checks every sample the chain carries, without a proof it would have to trust.

The accumulator is the rule that replaces the cap, under a condition. A block admits $K_{\text{BLOCK}}$ tickets beyond its own, each a winning sample of the same game under the same target, folded with the block's own into the epoch's accumulator — the commutative monoid $(\sum_n m(n),\, k,\, \pi_{\text{acc}})$ over the tickets' statements, order-independent, one decider per epoch — when both hold: (a) the consensus engine's step is a relation [[zheng]] proves, a direct CCS relation of one exact integer step aggregated by IVC over the `PHI_STEPS` steps (tru `audit/phi-relation-2026-11.md` §4; the general nox machine is not viable — 11–14 G rows for one pussy solve), so a ticket's solves leave the verifier; and (b) the accumulation is constant-size with the verifier inside the step ([[foculus]] `audit/accumulated-settlement-2026-10.md` §4; the measured accumulated profile without it is 27–46× larger than the per-ticket proofs and verifies slower than recomputation). The condition is a predicate on the proof stack, checked by its receipts, and $K_{\text{BLOCK}} = 0$ while it is false; the rule set that sets $K_{\text{BLOCK}} > 0$ carries the relation, as every rule set must before it activates ([[proven-present]] §2.11). Under it a block's subsidy divides equally among its tickets, so emission stays a function of height, the samples per epoch rise by up to $K_{\text{BLOCK}} + 1$, and the fold's own work exercises the same four [[Goldilocks field processor|GFP]] primitives as the sample — the work is real throughout. Duplicate samples are excluded structurally: the monoid counts each header template once, and two tickets with one template are one ticket.

### Deadline, $k_{\min}$ and the payout

The window of epoch $E$'s game is epoch $E + 1$: every block at height $h$ with $\text{boundary}(E) < h \le \text{boundary}(E+1)$ samples the game of $E$, and no block samples any other game. Epoch 0 is the genesis block alone and its game is empty ($G_{-1} := G_0$, $N = 0$), so the blocks of epoch 1 carry probes of the genesis graph. The settlement of $E$ is a step of the block at

$$\text{settle}(E) = \text{boundary}(E+1) + D, \qquad D = 120,$$

where every ticket of the window is in and the window's last block is final by the reorg limit (chain-economy §2.5 amended: the settle height of epoch $E$ is $D$ past the boundary of $E + 1$, one height per epoch as before; the first payout, of epoch 1's game, lands at height 600, five hours after genesis; `MIN_LOCK ≥ 2L + D` keeps stake added in $E$ locked through its settlement). The pulse is paid there and is final, which is the escrow §12 asks for — the snapshots and the tickets it reads can no longer move.

The settlement pays each player by the swarm. For player $c$ with $k_c$ sampled marginals in the window and their mean $\bar m_c$,

$$a(c) = \begin{cases} \bar m_c & k_c \ge K_{\min} \\ a^{\text{exact}}(c) & k_c < K_{\min} \end{cases}, \qquad a^+(c) = \max(0, a(c)), \qquad \hat a(c) = \left\lfloor a^+(c)\cdot\frac{v(N)}{\sum_{c'} a^+(c')} \right\rfloor \ \text{ when } \sum_{c'} a^+(c') > 0 \text{ and } v(N) > 0,$$

where $a^{\text{exact}}(c)$ is the four-node path gradient of [[epoch]] §4 (chain-economy §5.2), the Aumann–Shapley value of this game, measured within a median 0.04–3.0 % of exact Shapley ([[epoch]] §7), computed by the consensus epoch engine on the game's base $A_{E-1}$ — the one engine that verifies the tickets (tru#46's `Game`), so the floor, the cap and the samples are one function. A consequence of the rule, not a case of its own: $k_c \ge K_{\min}$ needs $L(W_{\text{TICKET}} - 1)/N \ge 64$, so on bostrom ($W_{\text{TICKET}} = 2$) every player of a game of more than three is paid its exact gradient, and on pussy every player of a game of more than 236 — the payout at launch, by the same rule. The rescaling is the conservation clip of §4 in the game's own unit, with its conditions: a negative share is clipped to zero first (chain-economy §5.3 pays $\max(0, a(c))$; §4 clips the total), the factor is applied only when the clipped sum and $v(N)$ are both positive, so no share changes sign and the factor is bounded by $v(N)$ over a positive sum; when $\sum_{c'} a^+(c') = 0$ or $v(N) \le 0$ nothing is paid and the pool carries forward (chain-economy §5.5: a quiet epoch carries its accrual). The mean $\bar m_c$ is the integer quotient of the sum of the player's sampled marginals by $k_c$, rounded toward zero, and every rescaled share is floored, in the chain's integer arithmetic ([[arithmetic]]; the product in 256 bits as chain-economy §5.2 forms $a(c)$); the dust $v(N) - \sum_c \hat a(c)$ is the pool's, the rule of chain-economy §5.5. So the shares never exceed what the epoch created, and the exact gradient is the cap and the floor of the mean, never the payout where the swarm has spoken. From $a(c)$ the chain's own rules follow unchanged — the new-high rule, the budget, the cap of 1 % of the stake added, floor rounding, dust to the pool (chain-economy §5.3–§5.5) — with $x(c) = \lfloor \hat a(c) \cdot N^\sigma / R^\sigma \rfloor$. The exact gradient is computed by every node in any case (0.12 s at pussy, 75 s at bostrom on one thread) and the served estimate (chain-economy §11, `GET /epoch/{E}/lottery`: per player, $k_c$, $\bar m_c$, its standard error, $a^{\text{exact}}(c)$) is the chain's running measurement of the rule against the game.

$K_{\min}$ is what the chain can hold and what the receipt's variance needs. An epoch delivers $L \cdot (W_{\text{TICKET}} - 1)$ marginals, $k_c = L\,(W_{\text{TICKET}} - 1)/N$ per player: at pussy $15{,}120$ an epoch, so every player of a game of up to 236 players is sampled at least 64 times; at bostrom 240 an epoch, so games of up to three players. The precision of 64 samples follows from the receipt's one measured point by the Monte-Carlo law: 800 samples of an eight-player game reached $8.2 \times 10^{-4}$ of $\Sigma|\text{Shapley}|$ (tru#46 §2), so 64 samples reach about $8.2 \times 10^{-4}\sqrt{800/64} = 2.9 \times 10^{-3}$ — an order below the 5 % acceptance of the exact rule it replaces ([[epoch]] §7). So $K_{\min} = 64$: the count one epoch of 240 blocks delivers to every player of a game the window covers, and a count whose error is below the rule's own. The variance is the game's, so the chain publishes the realized standard error per player every epoch beside the mean; the bound is checked, not assumed. A player below $K_{\min}$ is paid its exact gradient — a game of more players than the window covers in one epoch is paid by the rule the swarm's mean converges to — so no claim waits and no settlement is empty: one rule, with its condition in the count.

Two ways to bias the mean are bounded by the same share. A contender that is also a miner sees $\pi(n)$, $r$ and the probes before any solve, so it may solve only templates whose window and prefixes favor its own players (pre-selection), and it may decline to publish a winning ticket whose marginals disfavor them (withholding, below). Both move the attacker's own players' means by at most its share $q$ of the window's tickets times the spread of their marginals, $|\Delta \bar m_c| \le q\,(m^{\max}_c - \mu_c)$ for pre-selection and $(\mu_c - m^{\min}_c)\,q/(1-q)$ for withholding ([[foculus]] `specs/fold-mining.md` § withholding bias, measured at 60–70 % of the bound in `audit/withholding-bias.md`); the rescaling to $v(N)$ makes a gain on one's own players a loss on every other, the cap of 1 % of the stake added bounds what the gain can take in tokens, and withholding forfeits a block's subsidy besides. Pre-selection has no forfeit: it is bounded by $q$ alone and is why a minority of the chain's work cannot move its own payout by more than its share of the spread.

The cartel bound. A cartel of $k$ real-stake colluders that locks $R$ in an epoch, under the rules above, extracts at most

$$\min\!\big(\lfloor R/100 \rfloor,\ \lfloor F/4 \rfloor\big),$$

the per-position cap of 1 % of the stake added and the epoch's budget, a quarter of the focus pool $F$ at the settlement (chain-economy §5.5, `POOL_RELEASE = 4`). The pool is fed per block by $\text{cut}(h) = \text{SUBSIDY} \cdot \text{focus\_bps}(\text{epoch}(h) - 1)/10^4 \le \text{SUBSIDY}/2$, read from the previous epoch's active stake (chain-economy §6.1), so a cartel's stake cannot raise its own epoch's budget — it raises the next epoch's inflow, at most to half the subsidy, and the miners fund it, since emission is fixed; after $q$ epochs that paid nothing the pool holds $(4 + q)$ epochs of inflow (the steady state is four, one paid), so the bound after a quiet stretch is the pool's quarter, not one epoch's inflow. Relative to its stake the take is at most $\lfloor F/4 \rfloor / R$, falling as $1/R$; it is paid once per new high (chain-economy §5.3) on stake locked seven days, so at most 52 times that a year for a cartel that rotates its stake to fresh axons every lock; the conservation clip of §4 and the rescaling to $v(N)$ (above) leave it no more than the epoch created, and [[identity]] cost and [[karma]] non-transferability leave it no cheaper identity than its stake. Measured on the pussy genesis graph (`audit/ring-extraction-2026-11-12.md`: 26,805 axons, a swarm of 64 and of 256 honest raises active in the previous epoch, rings of $k \in \{2, \ldots, 64\}$ at $10^{11}$–$10^{14}$ per colluder, the pool at its steady state and after ten quiet epochs): the take is at most 0.056 % of the cartel's stake in the steady state and 0.197 % after ten quiet epochs ($k = 2$, $10^{13}$ each, $H = 5.1 \times 10^{13}$), 0.0003 % and 0.0009 % at $R = 6.5 \times 10^{15}$, and 99.0–99.99 % of the budget whenever it earns at all; a ring of fresh particles with no link from the existing graph earns nothing within the sweep ($R \le 6.5 \times 10^{15}$, 0.6 % of $S_0$) because its attribution is $\le 0$ — new particles spread the focus and $J$ falls, a property of the measure in that range and not a rule; a ring of the order of $\Sigma G$ concentrates focus and is then bounded by the line above. The honest swarm's pulse is diluted pro rata by attribution — to 7 % of its lone value at $R = 5.4 H$ — which is the law paying descent: a stake that dwarfs the swarm's moves the focus more than the swarm does. Collusion is bounded by capital, once per lock, under a budget the miners fund: that is the rule, with its receipt.

An external-attention gate — scaling a position's attribution by the weight others had placed into its region before the epoch — was carried by this page on 2026-11-12, measured, and withdrawn (receipt §2, the gated columns): with one new axon to any old particle a ring's region has external weight and the ring keeps 88–99 % of the budget; a spoiler's new axons into an honest endpoint make the endpoint's own weight internal and scale the honest raise toward zero; and no on-chain test separates distinct neurons' earlier attention from a cartel's placed a week before. The cartel bound holds with and without it, so the gate is not a rule of this page, and self-dealing is bounded by the cartel bound alone.

### Withholding, priced

The lottery is not fully closed against a miner that is also a contender in the game it settles: it can compute $m(n)$, see that the sample lowers its own share, and decline to publish a winning ticket. It cannot lie — a ticket is a block whose sample every node recomputes — so its only freedom is to abstain, and an abstained template is a block it found and did not publish. The injectable bias is the one bounded above, $(\mu - m_{\min})\,q/(1-q)$ — negligible for a minority, and a majority already breaks consensus.

The forfeit is the price. A withheld ticket is a withheld block: the miner forfeits its subsidy $R_{\text{ticket}} = \text{SUBSIDY} - \text{cut}(h) + \text{fees}$ (chain.md §2.2) and the rivals' next block takes the height. The gain over an epoch is at most the withholder's own payouts times the relative bias, $G \le \text{pay}_w \cdot \frac{q}{1-q}\cdot\frac{\mu - m_{\min}}{\mu}$, against a forfeit of $|D| \cdot R_{\text{ticket}}$ for the $|D| \le qL$ withheld blocks; with the budget $B \le L \cdot \text{cut}$ and $\text{cut} \le \text{SUBSIDY}/2 \le R_{\text{ticket}}$ (the miner floor of §10, chain-economy §6.1), $\text{pay}_w \le B \le L \cdot R_{\text{ticket}}$, so the forfeit exceeds the gain for every withholder whose share satisfies $(1 - q)\,\mu \ge \mu - m_{\min}$ — every minority withholder of a game whose samples are not dominated by one outlier — and the bound is published with the estimate every epoch. The two further deterrents of the general law stand as the owner wrote them: to separate roles (a miner does not settle a cluster it contends in), which the root chain cannot read because positions are paid to payout owners a miner need not be, and a commit-to-$n$-before-learning-$m(n)$ round, which drives the bias to zero in expectation by forcing a non-adaptive adversary at the cost of a synchronous commit–reveal assumption foreign to the lottery; it is the escalation, not the default, and the root chain runs the forfeit.

---

## 8. The Three Roles

A single computation — the [[tri-kernel]] over the [[Goldilocks field]], simultaneously learning, proving, and inference — earns in three roles, distinguished only by what its proof certifies. This is requirement 4 at the economic level: one chip, one kind of work, three economic faces. What each is paid for differs in kind — committed stake, expended work, a received fee — while the chip and the proof stay one.

| role | the proof certifies | who earns | basis |
|---|---|---|---|
| mint | a graph mutation (focus shift) | anyone who links | stake |
| subsidy | a sample meeting a difficulty target | anyone who computes | work |
| fee | a query answered (inference) | anyone who serves | fee |

### Mint — the knowledge stream

A neuron links, computes $\Delta\phi^+$, proves it, and self-mints its Shapley share, settled by the lottery and bounded by global $\Delta\phi^+$. Earning it requires conviction stake — a [[costly signal]]. This is a budget of its own, held separate from the security budget $B$ below.

### Subsidy — proof of work, the stakeless onramp

The subsidy and the settlement lottery are one act: a miner earns the block subsidy by the same winning ticket of §7, $\text{hemera}(n \,\|\, \text{sample}) < \text{target}$, whose work is a real Shapley sample of the epoch's game — the marginals of a window of one ordering, one cold solve of the consensus engine each, sparse matrix–vector passes over the [[Goldilocks field]], the same work every full node does to settle — so the work is real throughout, with no separate proof-hash puzzle. When the epoch has no player, the same ticket samples the gradient the attribution reads (§7, step 2), so block production and the stakeless onramp never stall. The subsidy is [[karma]]-blind and stake-blind: a new [[neuron]] with zero [[$CYB]] earns it and acquires the stake that unlocks the mint. What the onramp asks of a newcomer is exactly what a full node has — the public staked graph and a machine that solves it — and nothing a holder has: no stake, no karma, no position, no permission. The mint it then earns is karma-light at first, since karma is earned rather than granted; the sparse early graph offsets this with a large discovery premium — every link moves an uncrowded $\phi^*$ a long way — and karma accrues as those early links are validated. So the cold-start path runs subsidy → stake → discovery-premium mint → karma → amplified mint. This permissionless entry is a hard requirement. Difficulty adjusts to hold block time; the subsidy is independent of $\Delta\phi^+$.

The ticket is the work from the first block. At genesis the subsidy's product is distribution and attribution at once: the first block after the first boundary already samples the first epoch's game, and before any position exists it samples the gradient of the genesis graph, which is the attribution rule's own input. No synthetic puzzle precedes the lottery, inside the chain or beside it. [[UniversalHash]] — the device-democratic memory-hard hash under which a phone stands at roughly one quarter of a desktop and a GPU stands nowhere — is the puzzle of the viewing economy (uhash `docs/viewing-economy.md`; [[lytics]] binds its work quanta to attention windows measured on-device), a distribution lane of its own, with its own envelope and its own ledger, outside the chain's consensus. Its work is synthetic and that is its honesty there: where the product is a fair raffle, neutral work on universal hardware is what makes the handout even ([[warriors/mining]]). It never enters the chain's work: synthetic work inside the settlement lottery would dissolve the zero-synthetic-work property of §7, and hardware built for AES serves no [[tri-kernel]]. One puzzle, two verdicts, decided by what the work is for.

### Fee — services

A neuron answering a query runs the compiled transformer ([[focus-flow]] Path B), an inference whose correctness is itself a [[zheng]] proof. The asker pays; the protocol burns a fraction $\beta$, pays the servicer $\gamma(1-\beta)$, and routes the remaining $(1-\gamma)(1-\beta)$ into the security budget $B$ — one fee, three disjoint slices.

### PoS — the amplifier

Proof of stake adds no fourth role; locked stake and [[karma]] amplify the other three. They raise a neuron's weight in $A^{\text{eff}}$ — enlarging its $\Delta\phi^+$ and mint share — and active stake earns a share of the fees routed through $B$ (§10). Conviction stake doubles as the security deposit: the staking ratio $\theta$ is the fraction of supply locked across [[cyberlinks]], so the bonded capital is always productively committed. An attack on $\phi^*$ then needs both stake and unbuyable [[karma]].

---

## 9. Two Axes

Stake acts on two independent axes; separating them is the structural defense against wealth concentration, and the answer to requirement 3.

| axis | what moves it | what it produces |
|---|---|---|
| rank | any real stake, including $v_\ell=0$ | weight in $A^{\text{eff}}$, hence $\phi^*$ and [[cyberank]] |
| reward | correct risk under $v_\ell \neq 0$ | a share of the streams in §8 |

Idle, passive, or Sybil capital can move rank but pulls no reward. Capital shapes the graph; only correct epistemic risk earns from it. Locked capital cannot compound by sitting still — the precise structural fix for the wealth-compounding failure of stake-weighted systems.

A $v_\ell=0$ link earns nothing by category, not by penalty. It is a purchase, not an investment: the time-value of staked [[$CYB]] spent to buy weight over $\phi^*$. This is rational for a [[neuron]] whose use-value of that influence exceeds its capital cost, and unattractive to rent-seekers, who have none — a monetary yield would convert the purchase into an investment and reopen compounding. The protocol also cannot pay it in principle: minting must separate signal from copying through BTS information-gain (§5), which needs the meta-prediction that $v_\ell=0$ declines, so a passive link's $\Delta\phi^+$ is real movement that stays unverifiable as knowledge. Influence over $\phi^*$ is the entire return, paid in kind and unpriceable by design.

---

## 10. Supply and Allocation

The security budget splits between PoW and PoS by the allocation curve of [[adaptive hybrid consensus economics]]:

$$R_{\text{PoW}} = B\,(1 - \theta^\alpha), \qquad R_{\text{PoS}} = B\,\theta^\alpha, \qquad \alpha \in [0.3, 0.7],$$

with $\alpha = 0.5$ the neutral prior under equal marginal security cost. Gross budget and holder dilution are decoupled:

$$B = \text{floor}\cdot M + (1-\gamma)(1-\beta)F, \qquad I_{\text{net}} = \text{floor} - \frac{F\beta}{M}.$$

Gross rewards can exceed inflation when fees are high; net inflation can go negative. The security floor is derived from attack economics rather than chosen — $\text{floor} \ge c_{\text{sec}}\cdot(\text{TVL}/M)\cdot r_{\text{atk}}$, where $c_{\text{sec}}$ is the safety margin and $r_{\text{atk}}$ an attacker's cost of capital per epoch (distinct from the §7 hop radius $r$ and the §4 sample count $k$); here $M$ is circulating supply. This is the one emission untied to $\Delta\phi^+$.

Base emission goes to work and risk only. A standing yield to passive stake would be emission without contribution — it would break the invariant that inflation is [[knowledge]], and it is the mechanism by which idle capital compounds. The floor is paid only to the two providers that do work: PoW compute and active ($v_\ell \neq 0$) epistemic risk. It PID-decays toward zero as mint and fees grow to cover security. The parameters $\alpha$, floor, and $\beta$ follow PID control on observable signals (security margin, fee coverage, efficiency differential), so the system measures and adapts rather than predicts.

On the root chain the split is the whole emission (§16): $B$ is the block subsidy, one constant per block; $\theta$ is active stake over supply; $\alpha = \tfrac12$ in integers; the PoS share $B \cdot \min(\tfrac12, \theta^{1/2})$ is the focus pool that funds the mint by attribution, so the mint and the stake yield of §11 are one stream paid to the active stake that added weight; the PoW share $B\,(1 - \min(\tfrac12, \theta^{1/2}))$ pays the ticket, one per block; $\beta = 0$ and $\gamma = 1$, every fee to the miner of the block. The three PID loops run on `tru::Pid` (`rs/pid.rs`, a clamped anti-windup controller over `Fx`, integral frozen while saturated); their inputs are the fee flow $F$, the market, and the attack-cost estimate, none of which the root chain carries, so on it the three parameters are the constants of §16 and the loops hold them there — a loop with a zero input holds its setpoint. Issuance is one number: the mint's budget and $B$ are one subsidy on the root chain, $L \cdot \text{SUBSIDY}$ an epoch, and in the general law their sum is the allocation above; whether that floor funds the hardware the work wants before fees arrive is a projection with a named assumption and lives in `roadmap/gfp-funding.md`, outside this contract.

---

## 11. The Reward Equation

For a neuron $\nu$ over an epoch, the whole specification assembles into one line:

$$\boxed{\;R(\nu) \;=\; \underbrace{\text{Shapley}_\nu(v^\star)}_{\text{mint}} \;+\; \underbrace{R_{\text{PoW}}\cdot\frac{w_\nu}{\sum_\mu w_\mu}}_{\text{subsidy}} \;+\; \underbrace{\sum_{q\,\in\,Q_\nu}\gamma(1-\beta)\,\text{fee}_q}_{\text{service fees}} \;+\; \underbrace{R_{\text{PoS}}\cdot\frac{a_\nu\,\kappa(\nu)}{\sum_{\mu} a_\mu\,\kappa(\mu)}}_{\text{stake yield}}\;}$$

where $w_\nu$ is $\nu$'s proven settlement work this epoch — its winning tickets, the blocks it mined — $Q_\nu$ the queries it answered, and $a_\nu$ its active ($v_\ell \neq 0$) stake. Four terms, four requirements: the mint rewards real value, locally computed and later validated; the subsidy secures the chain and opens a stakeless door, shared in proportion to work done so splitting a signal into many earns nothing extra; the service fees pay whoever served a query directly; the stake yield routes the fee pool to committed stake, weighted by [[karma]] so the share still tracks a record of being right. Sybil-resistance and anti-compounding hold across the sum; conservation binds the mint term specifically, $\sum_\nu \text{Shapley}_\nu(v^\star) = v^\star(N)$.

A single mint underpays foundational work, which starts at low $\Delta\phi^+$ and grows as the graph builds around it. So an active ($v_\ell \neq 0$) link also earns a yield stream — the delayed mint of that foundational work, the time-integral of the target particle's [[cyberank]] growth attributable to the link. Passive ($v_\ell=0$) stake earns no part of it; the annuity is realized value, not rent on locked capital:

$$R_{i \to j}(T) = \int_0^T \omega(t)\,\Delta\phi^*_j(t)\,dt.$$

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

Finality and accuracy are split. The pulse (instant mint, §11) is settled at the epoch boundary on provisional information — modest, conservative, final, paid now. The annuity (the yield stream) accrues as truth reveals itself, computed by the same per-epoch pass, and it is self-correcting: a link later falsified simply stops drawing it. So the protocol pays a small final pulse now and lets the annuity carry the long-horizon correction. The discovery leak is bounded by the rule: a link gated at $f(\text{price}) < 1$ is paid $f$ times the pulse it earns at $f = 1$, to first order in its weight — attribution is linear in a position's weight at a fixed path gradient (chain-economy §5.2, $a(c) = \Delta s\,\sigma\,\bar g$; measured: one author's 100 pays what 60 + 40 pays, cybernode `audit/chain/economy-2026-10-17.md`), while $f$ also enters $A^{\text{eff}}$ through the path the gradient is averaged on, whose second-order part is the quadrature's, under 5 % in every measured class of change ([[epoch]] §7) — so what the early contrarian-correct link loses is $(1 - f)$ of its pulse at first order, and the annuity of §11 pays it back as $\omega(t) = \text{price} \times \text{karma}$ rises with the market that turns; on the root chain $f = \rho = 1$ (chain-economy §3.7) and the leak is zero. A paid pulse is irreversible, so a contribution later revealed dishonest can have its future earnings stopped (price → 0) but not its pulse reversed, which is the reason to keep the pulse conservative and most value in the self-correcting annuity.

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
| value magnitude ($\Delta\phi^+$, [[karma]], [[syntropy]]), the consensus engine and its gradient | [[tru]] |
| the epoch boundary, the beacon, the ticket as the block's work, finality | the root chain (cybernode `specs/chain-economy.md`), with [[foculus]] as its settlement library |
| conservation, allocation, mint | [[tok]] |
| identity, anonymity | [[mudra]] |

The chain decides what is real; the reward function decides what it is worth. Economic parameters change without touching consensus safety.

---

## 15. Security

Every row is a rule of this page, with its clause; the bounds carry their receipts there.

| property | guarantee |
|---|---|
| conservation | $\sum_\nu \text{mint}(\nu) = v^\star(N) \le $ global $\Delta\phi^+$, by Shapley efficiency (§4); in tokens on the root chain, payouts $\le$ budget $\le$ pool, the shares rescaled to the exact $v(N)$ (§7, §16) |
| Sybil-resistance | stake-weighting makes identity-splitting reward-neutral (§4); on the root chain attribution, the cap and the split are linear in stake (chain-economy §5.2, §5.5) |
| honest reporting | in the reports rule set, truthful [[valence]] is a Bayes–Nash equilibrium of the composite payoff where the clip of $\rho$ does not bind: $\rho$ linear in the serum score on $[0, s_{\max}]$, $s_{\max} = \ln 2$, the slash linear in the same score, both reading one meta-prediction; where the clip binds the residual is bounded by $M\,\mathbb{E}[(-s)^+]/s_{\max}$ (§5); on the root chain there is no report and the row is empty; the serum's margin against a coordinated majority measured (`audit/coordinated-consensus-scoring.md`) |
| collusion | a cartel locking $R$ extracts at most $\min(\lfloor R/100 \rfloor, \lfloor F/4 \rfloor)$ an epoch, $F$ the pool fed at the previous epoch's `focus_bps`, once per new high; measured 0.056 % of its stake at most in the steady state, 0.197 % after ten quiet epochs, falling as $1/R$ (§7, `audit/ring-extraction-2026-11-12.md`) |
| self-dealing, manufactured surprise | a ring on fresh particles with no link from the graph earns nothing within the sweep (its attribution is $\le 0$); a ring linked to the graph is bounded by the cartel bound; the crowd reference of $\rho$ is taken over distinct karma-bearing predictors with their own exposure (§5, §7) |
| the discovery leak | a link gated at $f < 1$ loses $(1 - f)$ of its pulse at first order in its weight, and the annuity pays it back as the market turns; zero on the root chain, $f = \rho = 1$ (§12, chain-economy §3.7) |
| issuance | one subsidy, $L \cdot \text{SUBSIDY}$ an epoch, split by $\min(\tfrac12, \theta^{1/2})$; supply $= S_0 + h \cdot \text{SUBSIDY}$ (§10, §16, chain-economy §6) |
| stakeless entry | the PoW subsidy is karma- and stake-blind; its work needs the public state and a machine, nothing a holder has (§8) |
| no idle rent | only $v_\ell \neq 0$ risk earns; passive stake earns rank, not income (§9) |
| attack cost | $\phi^*$ manipulation needs stake and unbuyable [[karma]] (§5, §8); on the root chain a rewrite costs at least half the subsidy's work (chain-economy §6.1) |
| leaderless settlement | attribution is the swarm's mean of sampled marginals, capped and floored by the exact value every node computes; no producer decides it |
| settlement soundness | every sample is verified by recomputation, hash first, under a node-wide budget per interval served by chainwork (§7); the beacon is the boundary block id (§7, §16); the subsidy and the lottery share one preimage (§7–§8); the cap is one ticket per block; every ticket carries a probe, so no table of values replaces a solve |
| settlement liveness | every block carries a sample of the same cost, the empty epoch carries probes, the settlement block is always valid, and a player under $K_{\min}$ is paid its exact gradient (§7) |
| fork-safety | the pulse is paid at the block $D = 120$ deep past the window's last block, below the reorg limit, so no payout is minted against a snapshot or a ticket that can move (§7, chain-economy §2.5) |
| withholding, pre-selection | bounded by the attacker's share of the window's tickets, zero-sum under the rescaling to $v(N)$, capped at 1 % of the stake added; withholding forfeits a block's subsidy besides (§7) |

---

## 16. On the Root Chain — What Phase 1 Fixes

The phase-1 root chains (pussy, bostrom) run this law in consensus from genesis, on Nakamoto proof of work whose work is the settlement lottery of §7, with the parameters below; the normative page is cybernode `specs/chain-economy.md` (owner order 2026-10-17: "the goal is foculus, stakes with valence, and mining"; owner order 2026-11-04: "the proof of work must really sample values by the rewards distribution"). Each line names the section of this page it instantiates.

| quantity | root-chain value | here |
|---|---|---|
| epoch | 240 blocks (2 h at 30 s); the boundary is a block, $b_E$ is its block id | §6, §7 |
| the game | the positions that changed between two boundaries, one cluster, $v(S) = J_U(\phi^*(A^S)) - J_U(\phi^*(A^\emptyset))$ on the consensus engine | §7 |
| the work | every block of epoch $E + 1$ carries one sample of epoch $E$'s game — $W_{\text{TICKET}}$ entries: the marginals of a cyclic window of the ordering $\pi(n)$ (the whole ordering when the game is smaller than the window) and at least one gradient probe — committed in its header (`sample: [u8; 32]`, 200-byte header); the block id below target is the ticket, the ordering weight and the subsidy; the nonce is the template's digest; tags `D_ORDER`, `D_PROBE`, `D_SAMPLE`, `D_CLUSTER`; the ordering and the commitment are tru `epoch::ordering` and `Sample::preimage` (tru#46) | §7, §8 |
| the window | $W_{\text{TICKET}} = \max(2, 2^{\lfloor\log_2(2.5\,\text{s}/t_{\text{solve}})\rfloor})$: pussy 64 ($t_{\text{solve}}$ 0.036 s, tru#46), bostrom 2 ($t_{\text{solve}}$ 6.0 s, [[epoch]] receipt §6.3) | §7 |
| difficulty | ASERT on the block interval (chain.md §1.5) over tickets; `POW_LIMIT = 2^{252}` (`0x20100000`) on both chains; `GENESIS_BITS` pussy `0x20020000` ($2^{249}$, from 5.06 samples a second, tru#46 §3.2), bostrom `0x20100000` (the floor, from one sample per 3 s on twelve threads) | §7 |
| verification | hash first, then every sample recomputed by every node, at most $W_{\text{TICKET}} + 1$ solves: pussy 65 (2.2 s on one M4 Max core, 0.2 s across twelve, 8.2 s on one Xeon core, tru#46 §3.1–§3.2), bostrom 3 with their builds (22–25 s on one core, 7–9 s on twelve); at most $V_{\text{BUDGET}} = \lfloor 0.5\,T/t_{\text{verify}} \rfloor$ verifications per node per interval (pussy 6, bostrom 2), by chainwork, ties by the node's local draw, the rest in the orphan pool (chain-wire §5, chain.md §5.1, §5.4); $F$ forgeries at the tip delay one node's acceptance by $F/V_{\text{BUDGET}}$ intervals in expectation, never propagating, never safety; a failed header is dropped, scored, never re-queued; the asymmetry ends with the proof or the accumulator; nothing relays before the check; one ticket per block, 240 per epoch; $K_{\text{BLOCK}} = 0$ while the accumulator's condition is false | §7 |
| settlement | at $\text{settle}(E) = \text{boundary}(E+1) + 120$, by every full node: each player paid the mean of its sampled marginals when it has $K_{\min} = 64$ of them, its exact four-node path gradient when it has fewer (0.12 s at pussy, 75 s at bostrom on one thread), the shares rescaled to the exact $v(N)$; the estimate and the exact value served per epoch | §4, §7 |
| $\Delta\phi^+$ attribution | integrated gradient along the epoch's straight path, four Gauss–Legendre nodes, one adjoint solve each: $a_\ell = \Delta s_\ell\,\sigma_\ell\,\bar g_{pq}$, $\bar g = \sum_i w_i\,\partial J/\partial A_{pq}(A_{t_i})$ — the Aumann–Shapley form of §4 (amended 2026-10-17 from the one-node midpoint: measured against exact Shapley, [[epoch]] §7) | §2, §4; operators, transposes and scales: [[epoch]] |
| φ* continuity | every particle's out-row carries a ground weight $w_0 = \text{in}/4$ to the teleport prior, so a first out-link redirects focus continuously (amended 2026-10-17; without it φ* jumps and no gradient form can attribute the jump) | §4, [[epoch]] §2 |
| $A^{\text{eff}}$ | $\max(0,\ G_{pq} + S^+_{pq} - S^-_{pq})$: stake of $v \in \{0,+1\}$ adds, $v=-1$ subtracts, clipped per axon; $G$ = passive genesis weight | §3, §9 |
| karma $\kappa$, price $f$, will | 1, 1, 0 — the chain carries no reports, no market and no will; each is a parameter of its rule set, and a rule set that carries them carries its relation first ([[proven-present]] §2.11) | §3, §5 |
| surprise $\rho$ | 1 for every active position, the chain's `RHO_LAUNCH`; copying is priced by the gradient itself — a copy on the same axon in the same epoch splits that axon's credit pro rata; $s_{\max} = \ln 2$ and the one report are the general law's | §5 |
| the cartel bound | $\min(\lfloor R/100 \rfloor, \lfloor F/4 \rfloor)$ an epoch, once per new high, on seven-day locks; measured 0.056 % of the cartel's stake at most in the steady state, 0.197 % after ten quiet epochs, on the pussy genesis graph | §7, §15 |
| reward axis | only $v_\ell \neq 0$ stake that added weight in the epoch earns; passive stake and genesis weight earn nothing | §9 |
| recycling | a side earns only above its decayed high-water mark (half-life 30 days); stake locked 7 days | §12, §15 |
| emission | one subsidy (1 %/yr of genesis supply) split per epoch: the focus pool takes $\min(1/2,\ \theta^{1/2})$ of it, $\theta$ = active stake / supply, $\alpha = 1/2$; the ticket takes the rest and all fees | §8, §10 |
| mint budget | the pool releases a quarter of its balance per epoch, divided by attribution; a position takes at most 1 % of its added stake; dust stays in the pool | §4, §10 |
| conservation | in tokens: payouts $\le$ budget $\le$ pool, supply $= S_0 + h \cdot \text{SUBSIDY}$; the price of a nat of syntropy floats with the epoch | §4 |
| withholding, pre-selection | a withheld ticket forfeits $\text{SUBSIDY} - \text{cut} + \text{fees}$; both biases bounded by the attacker's share $q$ of the window's tickets and zero-sum under the rescaling to $v(N)$ | §7 |
| annuity | the pulse only; the annuity of §11 integrates $\omega(t) = \text{price} \times \text{karma}$, both 1 on the root chain, so its stream is the pulse's own history and nothing is owed beyond it | §11 |

One departure from the general law is deliberate. The mint is a fixed token budget divided by $\Delta\phi^+$, not $\Delta\phi^+$ converted at a set price, so the clip of §4 to $\min(v^\star(N), \Delta\phi^+(N))$ becomes conservation in tokens, and an epoch without positively attributed active stake pays nothing. The lottery is not a departure: the chain's work is the lottery of §7 from block 1, and the settlement pays by the swarm's mean, with the exact Aumann–Shapley value as its cap and as the floor of a player the window did not reach — §7's one rule, with the counts that decide it in the receipts.

Three design choices this page settles, each with its reason:

1. No separate ordering hash. The block id of the ticket-carrying header is the chain's one work function; steps-blocks-epochs §5's "the ticket is a gate, the grind is a race" and chain-economy §7.3's "the header keeps a hash target for ordering" are amended. Reason: ordering needs exactly one progress-free lottery and the ticket's win test is one (a Poisson trial per attempt, §7); a second lottery is synthetic work by definition, dissolves the zero-synthetic-work property of §7, and needs a subsidy nobody pays (cybernode `audit/chain/settlement-mining-state-2026-11-02.md` §4); and the whole-template seed closes the only way a hash grind could hide inside the ticket. The cost is at most $W_{\text{TICKET}} + 1$ solves of verification per block — 0.2 s across twelve cores at pussy's size, 7–9 s at bostrom's — inside the interval.
2. The beacon is the boundary block id (chain-economy §2.3). Reason: it is a consensus object every node has from headers alone, it is fixed after the epoch's last position is in, and its one bias — a boundary miner discarding a found block to redraw — costs that block's subsidy and buys a different sample set, whose effect on any mean is bounded by the redrawer's share like every other bias of §7; the VDF beacon of [[foculus beacon]] stays the general law's instrument for a chain without blocks.
3. Verification is recomputation. Reason, measured: a sound ticket proof discloses its witness and costs what recomputation costs (foculus `audit/sound-settlement-proofs-2026-10.md` §3); the φ* relation on the general nox machine is 11–14 G rows per pussy solve, about 2.7 years of the CI runner (tru `audit/phi-relation-2026-11.md` §3); the accumulated cluster profile is 27–46× larger than per-ticket proofs and verifies no faster (foculus `audit/accumulated-settlement-2026-10.md` §4). At most $W_{\text{TICKET}} + 1$ solves per block — 2.2 s on one core, 0.2 s across twelve at pussy's size (tru#46); three solves with their builds, 7–9 s on twelve threads at bostrom's — on the machine that already settles is cheaper than every proof measured, and the accumulator rule of §7 names the condition under which that stops being true.

---

## Trace — every rule to its source

| rule | clause, sentence or receipt |
|---|---|
| the proof of work is a Shapley sample; nothing else is work | the owner, 2026-11-04 ("the proof of work must really sample values by the rewards distribution"); §7 "The lottery" (the owner's prose, 2026-10); whitepaper §14.3 |
| one cluster per epoch on the consensus engine | [[epoch]] §2 (stake prior, normalization), §4 (the global gradient term); cybernode `audit/chain/settlement-mining-state-2026-11-02.md` §2 (the live path is one cluster) |
| the game $v(S)$ and $J_U$ | chain-economy §4.2, §5.2 (players, $A$, $\Delta s$ per side); tru `audit/epoch-settlement-2026-10.md` §3 (exact Shapley over $J_\cup$); tru#46 `Game::value` (`A_S = max(0, A_prev + Σ_S Δ)`, $J_U$ with $|U|$ fixed; the one-change marginal equals the engine's attribution to $10^{-7}$) |
| the ordering $\pi(n)$ by `epoch::ordering(beacon, cluster, nonce, N)` | §7 step 1 (the owner's $\pi(n) = H(b_E \,\|\, \text{cluster} \,\|\, n)$); tru#46 `epoch::ordering`, `audit/coalition-marginal-2026-11-04.md` §1, §4 |
| the nonce is the template's digest | §7 step 3 (the owner: a contender cannot grind nonces); chain.md §1.1 (every miner-variable field is in the header) |
| the probes, $(a, t)$ at $2^{32}$ resolution | [[epoch]] §4 (the path nodes at $2^{32}$); §8 (the owner: never stall); the count $\sum_a \min(A_a, 2^{32})$ against $2^{27}$ attempts a year |
| permutation sampling is the Shapley estimator; the window of $W_{\text{TICKET}} = 64$ | §4 (the owner: "Monte-Carlo over $k$ random orderings"); tru#46 §2 (`permutation_mean_equals_shapley_by_enumeration_up_to_six_players`, `beacon_samples_converge_to_shapley_for_eight_players`, `sample_sums_to_the_coalition_value_exactly`); §3.1 (one solve per marginal along an ordering); the window bounds the attempt at the receipt's measured sample |
| the ticket is the block id below target | §7 step 3, §8 (the owner: "this single test is also the block subsidy — no separate proof-hash puzzle"); chain.md §1.2 |
| `sample = hemera(D_SAMPLE ‖ W ‖ m…)`, `[u8; 32]`, 200 bytes, four permutations | tru#46 `Sample::preimage` (`D_SAMPLE ‖ k ‖ m[0] … m[k−1]`, i128 LE); hemera `specs/encoding.md` (one `0x01 ∥ 0x00*` absorption); chain.md §1.1 (168 = 3 × 56) |
| ASERT over tickets | chain.md §1.5; the owner's prose §7 ("the difficulty schedule has to correct for" cluster cost) |
| $W_{\text{TICKET}}$ per chain from $T/12$ and the solve | tru#46 §3.1 (0.036 s); [[epoch]] `audit/epoch-settlement-2026-10.md` §6.3 (6.0–6.3 s, 0.83–1.08 s on twelve) |
| `GENESIS_BITS` per chain, `POW_LIMIT` from the sample rate | tru#46 §3.2 (5.06 and 1.59 samples a second; 2.2 s per sample on one core); the bostrom rate from three solves on twelve threads; chain.md §1.2–§1.3 (the derivation shape) |
| hash first, $V_{\text{BUDGET}}$ by chainwork, ties by a local draw, the bound and its condition | chain-wire §5 (relay after verification, limits); chain.md §5.1 (chainwork), §5.2 (`MAX_REORG`), §5.4 (the orphan pool); this section (verify cost = make cost; the accumulator's condition); the owner's prose §7 (the asymmetric-cost DoS) |
| the GPU ratio is bounded by random-access throughput | tru#46 §3.1 (3.7× between two CPU cores at the same arithmetic; threads inside a solve lose; scaling across nonces); [[epoch]] §3 (48 dependent steps); chain.md §1.2 (hemera on a GPU: one to three orders) |
| verification by recomputation | foculus `audit/sound-settlement-proofs-2026-10.md` §3; tru `audit/phi-relation-2026-11.md` §3–§4; foculus `audit/accumulated-settlement-2026-10.md` §4 |
| one ticket per block, 240 per epoch | chain-economy §2.1 ($L = 240$); the owner's prose §7 (the flood-of-cheap-tickets DoS) |
| the accumulator under conditions (a), (b) | tru `audit/phi-relation-2026-11.md` §4 (decision c); foculus `audit/accumulated-settlement-2026-10.md` §4; [[proven-present]] §2.11 |
| the window is the next epoch; $\text{settle}(E) = \text{boundary}(E+1) + D$ | chain-economy §2.5 ($D$ = the reorg limit, finality), amended to the window's end |
| the payout is the swarm's mean | the owner's §7 ("the settlement itself is the average of every published sample", "converges out of the swarm"); the coordinator's reading of 2026-11-04 for the owner |
| the exact gradient as cap and floor | §4 (the conservation clip to the realized value); chain-economy §5.1–§5.2; [[epoch]] §7 (≤ 5 % median error) |
| $K_{\min} = 64$ | $L\,(W_{\text{TICKET}} - 1)/N$ per player; tru#46 §2 (800 samples → $8.2 \times 10^{-4}$) and the $1/\sqrt{k}$ law; [[epoch]] §7 (the 5 % acceptance) |
| withholding and pre-selection bounds, the forfeit | foculus `specs/fold-mining.md` § withholding bias, `audit/withholding-bias.md`; the owner's prose §7 ("a withheld ticket forfeits its subsidy; calibrate so the forfeit exceeds the share-gain"); chain-economy §6.1 (the miner floor) |
| the beacon is the boundary block id | chain-economy decision 2 and §2.3; cyber `launch.md` decision 2026-10-17 |
| the cartel bound | chain-economy §5.3 (once per new high), §5.5 (the 1 % cap, $B = \lfloor F/4 \rfloor$), §6.1 (`focus_bps` of the previous epoch, the half cap); `audit/ring-extraction-2026-11-12.md` (build-a, tru `dc874b9`, `rs/examples/ring_extraction.rs`) |
| the external-attention gate measured and withdrawn | the owner's prose §15 ("gating the discovery premium on external attention"); `audit/ring-extraction-2026-11-12.md` §2 (the bypass by one old link, the spoiler, the gated columns) |
| the discovery leak's bound | chain-economy §5.2 (linear in Δs), §3.7 ($f = \rho = 1$); cybernode `audit/chain/economy-2026-10-17.md` (linearity: 100 pays what 60 + 40 pays); [[epoch]] §7 (the quadrature's second order) |
| $s_{\max} = \ln 2$, one report, the composite equilibrium | [[strong-truthfulness]] (the separability lemma, the meta term strictly proper); [[truth-scoring]] (the redistribution linear in the score); `audit/coordinated-consensus-scoring.md`, `rs/truth_scoring.rs` |
| issuance is one number; the GFP question in `roadmap/gfp-funding.md` | chain-economy §6, chain.md §3.5; `~/cyber/AGENTS.md` § moving a page |
| UniversalHash in the viewing economy, never the chain | uhash `docs/viewing-economy.md`; cyber `launch.md` ("UniversalHash and the viewing economy: out of scope; a separate network"); [[warriors/mining]] (a distribution game inverts the values) |
| the split, the pool, the cap | chain-economy §5.5, §6.1; tru `audit/allocation-curve.md` |
| ρ, κ, f, will = 1, 1, 1, 0 | chain-economy §3.7, §5.4 |
| the pulse is final at depth 120 | chain-economy §2.5; the owner's prose §12 ("a paid pulse is irreversible") |

---

## Appendix — Notation

| symbol | meaning |
|---|---|
| $G = (P, N, E)$ | the [[cybergraph]]: [[particles]] $P$, [[neurons]] $N$, [[cyberlinks]] $E$ |
| $\phi \in \Delta(P)$ | a [[focus]] distribution; $\phi^*$ is the [[tri-kernel]] fixed point |
| $\mathcal{R}$ | composite operator $\lambda_d D + \lambda_s S + \lambda_h H_\tau$; $\phi^* = \operatorname{norm}[\mathcal{R}\phi^*]$ |
| $\mathcal{F}(\phi)$ | system [[free energy]]; $\phi^* = \arg\min_\phi \mathcal{F}$ |
| $J(\phi)$ | [[syntropy]] $= D_{KL}(\phi \,\|\, u) = \log|P| - H(\phi)$; $J_U$ reads it over the union particle set of an epoch (§7) |
| $\Delta\phi^+$ | directed focus impulse, the reward primitive (§2) |
| $\rho_\ell \in [0,1]$ | per-contribution [[Bayesian Truth Serum\|BTS]] surprise (§5) |
| $v^\star(S)$ | surprise-weighted value: $v$ with each contributed link's $A^{\text{eff}}$ weight multiplied by its surprise $\rho$ (§6) |
| $A^{\text{eff}}_{pq}$ | effective adjacency $= \sum_\ell \text{stake}(\ell)\,\kappa(\nu(\ell))\,f(\text{price}(\ell))$ |
| $\nu,\ \kappa(\nu)$ | a [[neuron]] and its [[karma]]; $\text{id}(\nu) = \text{Hemera}(\text{secret})$ |
| $v_\ell \in \{-1,0,+1\}$ | the [[valence]] of a cyberlink |
| $v(S)$ | coalition value function (§3, §7) |
| $n$ | a header template, the 168 bytes a miner varies; the lottery's nonce (§7) |
| $\pi(n),\ r,\ W'$ | the ordering of an attempt, the first rank of its window, the marginals it carries, $\min(N, W_{\text{TICKET}} - 1)$ (§7) |
| $m(n)$ | the sample of an attempt: the marginals $m_j = v^\star(\pi[0..j] \cup \{\pi[j]\}) - v^\star(\pi[0..j])$ of the window's ranks and the probes $m^{\text{probe}}_i$, i128 at $2^{60}$ nats each (§7) |
| $W_{\text{TICKET}},\ K_{\text{BLOCK}},\ K_{\min}$ | the entries of a ticket, a constant per chain (pussy 64, bostrom 2); folded tickets a block admits beyond its own (0 while the accumulator's condition is false); 64, the samples a player needs to be paid by its mean (§7) |
| $s$ | a [[signal]] $= (\nu, \vec\ell, \Delta\phi^*, \sigma, \text{prev}, \text{mc}, \text{vdf}, \text{step}, \text{nonce})$ |
| $\sigma$ | a [[zheng]] proof bound to a [[BBG]] root; also a player's side $\pm 1$ in §7 |
| $\theta, M, F$ | staking ratio, market cap, epoch fees |
| $B$ | security budget (PoW + PoS pools, §10), distinct from the cybergraph $G$ |
| $s^{\text{BTS}}_\nu,\ s_{\max}$ | raw [[Bayesian Truth Serum]] score (§5); $\rho_\ell$ is its clip to $[0,1]$ at the scale $s_{\max} = \ln 2$ |
| $R,\ H,\ F$ | a cartel's stake locked in an epoch; the honest swarm's active stake; the focus pool at the settlement (§7, §15) |
| $a_\nu,\ w_\nu$ | $\nu$'s active ($v_\ell \neq 0$) stake; its winning tickets this epoch (§11) |
| $\gamma,\ \beta$ | servicer fee share; fee burn fraction (§8) |
| $\varepsilon$ | precision floor — contributions below it are dropped (§2, §7); a protocol constant |
| $c_{\text{sec}},\ r_{\text{atk}}$ | security-floor safety margin; attacker cost of capital per epoch (§10) |

---

See [[focus-flow]] for how $\phi^*$ and $\Delta\phi^+$ are computed, [[epoch]] for the consensus engine the root chain's game and samples run on, [[truth-scoring]] for BTS and [[karma]], [[adaptive hybrid consensus economics]] for the PoW/PoS allocation and PID control, [[unified mining]] for the subsidy-as-signal-proof construction, and [[provable-consensus]] for epoch finalization. See whitepaper §6.9 and §14 for the surrounding economics.

The energy-based-learning grounding of §1 and §4 — that intrinsic free-energy descent, not a designed loss, is how the brain plausibly learns, and that Shapley credit is the integrated-gradient (Aumann–Shapley) generalization of it — follows Lillicrap, Santoro, Marris, Akerman & Hinton, *Backpropagation and the brain* (Nature Reviews Neuroscience, 2020); Scellier & Bengio, *Equilibrium Propagation* (2017); Sundararajan, Taly & Yan, *Axiomatic Attribution / Integrated Gradients* (2017). The permutation-sampling estimator of §7 is Mann & Shapley, *Values of large games IV* (RAND, 1960), as Castro, Gómez & Tejada, *Polynomial calculation of the Shapley value based on sampling* (2009) state it; its measured convergence on the consensus engine is tru#46, `audit/coalition-marginal-2026-11-04.md`.
