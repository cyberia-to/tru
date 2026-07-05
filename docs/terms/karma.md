---
tags: cyber, tru, core, spec
crystal-type: property
crystal-domain: cyber
alias: karma, kappa, trust multiplier, epistemic capital, reputation
---
# karma

the accumulated honesty record of a [[neuron]]: how much [[information]] it has contributed to the collective over time. karma is the non-transferable trust multiplier $\kappa(\nu)$ the [[tri-kernel]] reads — the one input to effective adjacency that capital cannot buy. it is earned by being right before the crowd, and by nothing else.

## how it is earned

karma is the running integral of a neuron's [[Bayesian Truth Serum|BTS]] scores (see [[truth-scoring]]). each [[cyberlink]] is a BTS report; the score is positive exactly when the neuron contributed private signal the crowd did not already hold and expect. karma rises on genuine signal, falls on noise, and is floored at zero — a neuron that repeatedly links things the market later validates accumulates karma; a neuron that links noise does not.

$$\kappa'(\nu) = \max\big(0,\; \kappa(\nu) + \eta \cdot s^{\text{BTS}}_\nu\big)$$

karma is [[serum|the serum]]'s memory. where the serum scores one report, karma is the reputation those reports compound into.

## what it weighs

karma is one of the three factors in the [[focusing|effective adjacency]] the tri-kernel runs on:

$$A^{\text{eff}}_{pq} = \sum_{\ell} a(\ell)\cdot\underbrace{\kappa(\nu(\ell))}_{\text{karma}}\cdot f\big(m(\ell)\big)$$

$a(\ell)$ is the [[conviction]] staked, $\kappa(\nu)$ the author's karma, $f(m)$ the [[market inhibition]] multiplier. all three must align: heavy conviction behind a low-karma neuron moves [[focus]] little. karma is what makes a link's weight depend on *who* asserted it, not only on how much they staked.

## why it cannot be bought

karma is non-transferable and unbuyable — it is the form of wealth that exists only as a track record. this is what closes [[Sybil]] attacks at the epistemic layer: a fresh identity starts at zero karma, so splitting stake across new identities *dilutes* rather than preserves influence. stake can be fabricated by splitting; karma cannot. it is [[epistemic capital]] — earned by prediction, not purchase.

karma is written by [[plumb]] from BTS scores and stored in [[bbg]]; [[tru]] reads it each epoch as an input. tru does not mint karma — it consumes it.

see [[serum]] for the per-report score · [[truth-scoring]] for accumulation · [[honesty]] for why it makes truth-telling rational · [[conviction]] for the capital it multiplies · [[focus]] for what it shapes.

discover all [[concepts]]
