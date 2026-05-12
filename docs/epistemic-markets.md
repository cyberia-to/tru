# Epistemic Markets Specification

The [[inversely coupled bonding surface]] (ICBS) market mechanism and [[market inhibition]] — the two components that give the [[cybergraph]] its inhibitory channel.

Source material: [[inversely coupled bonding surface]], [[market inhibition]]

---

## ICBS Cost Function

$$C(s_{YES},\, s_{NO}) = \lambda \sqrt{s_{YES}^2 + s_{NO}^2}$$

where $s_{YES}$ and $s_{NO}$ are token supplies and $\lambda$ is a scaling constant fixed at deployment. Iso-cost curves are circles in the $(s_{YES}, s_{NO})$ plane. Trading moves outward from the origin along the surface.

---

## Prices and Coupling

Prices emerge as partial derivatives of the cost function:

$$p_{YES} = \lambda \cdot \frac{s_{YES}}{\sqrt{s_{YES}^2 + s_{NO}^2}}, \quad p_{NO} = \lambda \cdot \frac{s_{NO}}{\sqrt{s_{YES}^2 + s_{NO}^2}}$$

The inverse coupling:

$$\frac{\partial p_{YES}}{\partial s_{NO}} = -\lambda \cdot \frac{s_{YES} \cdot s_{NO}}{(s_{YES}^2 + s_{NO}^2)^{3/2}} < 0$$

Buying NO lowers YES. Buying YES lowers NO. The two sides are genuine opposites on a shared geometric surface.

---

## ICBS Properties

| Property | Description |
|---|---|
| self-scaling | TVL $= C(s_{YES}, s_{NO})$ grows with trading volume |
| solvency | total value locked always equals the cost function |
| early conviction | prices range from 0 to $\lambda$, rewarding early discovery |
| geometric simplicity | only square roots — tractable on-chain computation |
| inverse coupling | buying one side directly suppresses the other |

---

## Role in the Cybergraph

Every [[cyber/link]] in the [[cybergraph]] can carry an ICBS market. The market price enters the [[tri-kernel]] as the effective edge weight:

$$w_{\text{eff}}(e) = \text{price}(e) \times \text{stake}(e)$$

At $p \to 1$: full [[focus]] flows through the edge. At $p \to 0$: the edge is deactivated. This is [[market inhibition]] — the mechanism by which collective epistemic assessment reshapes structural connectivity.

Self-scaling liquidity means trading volume automatically grows the market. The most-contested edges become the most liquid, yielding the most accurate prices. No external liquidity providers required.

---

## Market Inhibition

In a standard graph, every [[cyberlink]] contributes positively to [[attention]] flow. Market inhibition introduces a negative channel: when a prediction market on a link's validity resolves against the link, the market outcome scales down its weight.

$$w_{\text{eff}}(\ell) \;=\; w(\ell) \cdot \bigl(1 - \alpha \cdot m(\ell)\bigr)$$

where $w(\ell)$ is the original stake-weighted strength, $m(\ell) \in [0, 1]$ is the market's disbelief signal, and $\alpha$ is the inhibition coefficient.

---

## Effective Adjacency

The effective adjacency weight combines stake, karma, and ICBS price:

$$A^{\text{eff}}_{pq} = \sum_\ell \text{stake}(\ell) \times \text{karma}(\nu(\ell)) \times f(\text{ICBS price}(\ell))$$

---

## Neural Network Equivalence

Excitation alone produces a directed weighted graph. Adding inhibition makes the [[cybergraph]] computationally equivalent to a neural network:

| Biological | Cyber |
|---|---|
| excitatory synapse | staked [[cyberlink]] with positive weight |
| inhibitory synapse | market-suppressed [[cyberlink]] |
| neurotransmitter balance | stake vs. disbelief ratio |

The [[tri-kernel]] processes both signals simultaneously: [[diffusion]] spreads excitation, while market inhibition dampens unreliable paths.

---

## Economic Dynamics

Inhibition carries a cost. A [[neuron]] that inhibits a link must stake into the [[ICBS]] market against it. If the link turns out to be valid, the inhibitor loses stake. This symmetry ensures that both belief and disbelief are costly — cheap talk in either direction is eliminated.

---

See [[cyber/truth/coupling]] for the full specification. See [[valence]] for the ternary epistemic field on [[cyberlinks]]. See [[Bayesian Truth Serum]] for the scoring layer. See [[cyber/nomics]] for the broader economic design. See [[cyberlinks]], [[cybergraph]], [[tri-kernel]], [[attention]], [[tru]].
