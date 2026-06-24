---
tags: cyber, cybics, core, reference, research
alias: strong truthfulness, joint mechanism, composite truthfulness, perpetual truthfulness, strongly truthful, surprisingly popular selection
crystal-type: proof
crystal-domain: cybics
---
# strong truthfulness

The proof that the fused mechanism of the [[cybergraph]] — [[ICBS]] market, [[Bayesian Truth Serum]] scoring, and [[valence]] privacy — extracts truth, including in the perpetual market where no external oracle resolves anything.

This page settles the two results left open by [[truth-scoring]] and [[market]]: the joint mechanism and the perpetual game. It states what is proved, the one correction the naive composite needs, the architecture that makes the parts correct, and the residual obligations.

---

## division of labor

The three layers carry different jobs, and the separation is what makes each one correct. The mistake to avoid is asking any single layer to do all three.

| layer | job | required property |
|---|---|---|
| [[ICBS]] market | liquidity, commitment, spam cost, coarse first-order price | costly and self-scaling — not required to be a proper scoring rule |
| serum ([[valence]] + surprisingly-popular) | incentive-compatible truth extraction | strictly proper in the meta-report; selects truth over coordinated consensus |
| privacy ([[ZKP]]) | removes the coordination channel that fabricates consensus | only aggregates public; individual positions and reports hidden |

The serum is the oracle. The market is a liquidity skin over it. Privacy is the third leg that lets the no-external-oracle case stand.

---

## the market is not a truth-scorer, and must not be asked to be one

[[ICBS]] has cost $C(s_Y, s_N) = \lambda\sqrt{s_Y^2 + s_N^2}$, so its spot prices satisfy $p_Y^2 + p_N^2 = \lambda^2$ — a circle of radius $\lambda$, not the probability simplex $p_Y + p_N = 1$. A risk-neutral trader with belief $\theta = P(\text{YES})$ buys YES while $\theta > p_Y$ and NO while $1-\theta > p_N$; the two optimality conditions $p_Y = \theta$ and $p_N = 1-\theta$ are simultaneously reachable only when $\theta^2 + (1-\theta)^2 = \lambda^2$, a measure-zero coincidence. Driving $p_Y$ to $\theta$ forces a reserve ratio (with $\lambda = 1$)

$$q = \frac{\theta}{\theta + \sqrt{1-\theta^2}}, \qquad \theta = 0.5 \;\Rightarrow\; q = \frac{0.5}{0.5 + 0.866} = 0.366 \neq 0.5.$$

The reserve-ratio report is a systematically biased function of belief. [[ICBS]] with unit settlement is not a proper scoring rule; the mark-to-market reading turns it into pure price speculation. Both readings agree: the market alone does not track truth.

This is the right behavior for a liquidity substrate. [[ICBS]] is kept for its self-scaling TVL, inverse coupling, and $\lambda$-range early-conviction payoff. Properness lives in the serum, where it holds: the meta-prediction's log-score term is strictly proper. The truthfulness guarantee routes through the serum, never through the market.

---

## the separability lemma

The serum score of [[truth-scoring]] separates additively across its two reports. Expanding the information-gain term over outcomes $k$:

$$D_{KL}(p_i \| \bar m_{-i}) - D_{KL}(p_i \| \bar p_{-i}) = \sum_k p_i(k)\,\log\frac{\bar p_{-i}(k)}{\bar m_{-i}(k)} \;=\; \langle p_i,\,\ell\rangle, \qquad \ell(k) := \log\frac{\bar p_{-i}(k)}{\bar m_{-i}(k)},$$

linear in the first-order report. The prediction term

$$-\,D_{KL}(\bar p_{-i} \| m_i) = \sum_k \bar p_{-i}(k)\,\log m_i(k) + \text{const}$$

is the strictly proper log scoring rule for the meta-report $m_i$, with expected maximum at the truthful meta-belief. So the serum is linear in the first-order report and strictly concave in the meta-report, and the two never couple.

---

## the correction the naive composite needs

The serum's first-order score is linear, so it cannot by itself pin a unique truthful report — a linear objective is maximized at a simplex vertex, not at the interior posterior. Two distinct settings resolve this, and they require different architectures.

Resolved markets. When a market settles against a real external outcome, the market profit becomes a strictly proper scoring rule (it is then a standard convex-cost maker over the simplex). The strictly-concave market term regularizes the serum's linear first-order term: the composite has a unique maximum at the true posterior. The market carries first-order truthfulness; the serum adds the meta channel. The factorization holds — route first-order signal through the market, score the [[valence]] with the serum.

Perpetual markets. With no external resolution the price is marked against its own converged value, which makes it a Keynesian beauty contest: any commonly-expected convergence point is self-fulfilling. The market cannot carry first-order truthfulness. The serum carries it instead, through the surprisingly-popular signal — proved next.

---

## theorem — surprisingly-popular selection (perpetual market)

Claim. In the perpetual market the surprisingly-popular divergence selects the truthful equilibrium out of the beauty-contest manifold, provided reinforcement couples to that divergence rather than to the raw price.

Proof. A false focal point $p'$ is commonly expected by construction — agents coordinate on $p'$ because they predict everyone will report $p'$. So the meta-predictions also point there: $\bar m_{-i} \approx \bar p_{-i} = p'$, giving divergence

$$\Delta := \bar p_{-i} - \bar m_{-i} \approx 0.$$

This is the babbling case of [[truth-scoring]], which scores exactly zero. A belief backed by private evidence is more popular than predicted — the surprisingly-popular effect — giving $\bar p_{-i} > \bar m_{-i}$, hence $\Delta > 0$ in its direction. The divergence is the coordinate that separates truth from mere agreement (Prelec–Seung–McCoy, 2017): popular-and-expected scores zero, popular-beyond-expectation scores positive.

Let reinforcement act on the surprisingly-popular estimate $\hat\theta$ derived from $\Delta$ rather than on the raw price. Then false focal points ($\Delta \approx 0$) receive no reinforcement and are not stationary under the dynamics; the truthful answer ($\Delta > 0$) is reinforced and is selected. Under the standard peer-prediction model — common prior, conditionally independent signals, stochastic relevance — $\hat\theta$ equals the truth, so the selected equilibrium is the truthful one. ∎

This is why the second dimension is essential, not redundant: the price can sit at a false focal point; the meta-report reveals whether that point is genuinely supported or merely coordinated. The two-dimensional signal of [[market]] is load-bearing in the perpetual case.

The remaining attack is a coordinated inversion — agents agreeing to report against their signal rather than merely coordinating on a focal point. Surprisingly-popular selection does not address this; it reduces to the honest-majority-by-stake condition with the multi-task Correlated Agreement structure of [[truth-scoring]].

---

## privacy completes the selection

The beauty-contest equilibrium is sustained by observability: agents coordinate on $p'$ because they can see the consensus and follow it. [[ZKP]] privacy removes that channel.

- A pump cannot recruit followers — no one can see the position to copy it — so it is faded by independent informed flow and reverts. Privacy permits secret accumulation but denies the cascade, and the cascade is where the systemic damage lives.
- Faced with a price it cannot attribute (one whale or a thousand honest agents), a rational agent has no informative social signal and falls back on its own belief, trading toward truth.
- A cartel cannot verify member compliance, so members defect to truth.

Privacy and the serum are complementary anti-consensus-fabrication mechanisms: the serum distinguishes coordinated from true; privacy removes the observability needed to coordinate. Residual single-agent manipulation is bounded by honest-majority-by-stake. Privacy is a defense, not a manipulation vector.

---

## the two settlement modes

The mechanism is a multipurpose oracle: settlement is a free parameter, and the truth source differs by mode.

| mode | truth source | role of the serum |
|---|---|---|
| time-bounded, externally resolved | the market becomes proper at resolution | adds the meta channel; factorization holds |
| perpetual, no external oracle | the serum's surprisingly-popular signal | the load-bearing oracle; full serum essential |

This is the precise sense in which [[Bayesian Truth Serum]] is the fundamental low-level oracle: it works exactly where markets fail, on the unresolvable and perpetual questions. The market is a liquidity skin; the serum is the part that cannot be removed.

---

## the design rules this fixes

- Route truthfulness through the serum, never through [[ICBS]] — the market is liquidity and commitment, not a scoring rule.
- Couple reinforcement (rank, effective weight, reward) to the surprisingly-popular estimate $\hat\theta$, not to the raw price. This is what selects truth over coordinated consensus.
- Keep individual positions and reports behind [[ZKP]]; publish only aggregates. This removes the coordination that fabricates false consensus.

---

## what remains conjectural

- the Regime-B capital bound: the honest informed [[ICBS]] capital that makes a stake-fraction-$f$ cartel strictly unprofitable, as a function of $\lambda$, $f$, and the signal margin. The coordinated-inversion case of the selection theorem assumes it; the explicit bound is the remaining economic theorem.
- the surprisingly-popular selection rests on the standard peer-prediction model (common prior, conditional independence, stochastic relevance). Correlated evidence across agents weakens it; its interaction with the perpetual dynamics is unverified.

see [[truth-scoring]] for the static minority case, the babbling lemma, and the Correlated Agreement reduction. see [[market]] for the perpetual market and the liquidity damper. see [[epistemic-markets]] for the [[ICBS]] cost function. see [[serum]] for the original Prelec mechanism. see [[honest majority assumption]] for the stake condition.
