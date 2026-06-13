---
tags: cyber, tru, core, spec
crystal-type: process
crystal-domain: cyber
alias: convergence, field convergence
---
# convergence

the process by which iteration approaches a destination that iteration itself defines. the [[tri-kernel]] iterates until [[focus]] stabilizes; [[neurons]] approach [[knowledge]]; the protocol approaches [[intelligence]]. convergence is the execution model of [[tru]] — what [[nox]] is to derivation, tru is to convergence.

a system applies the same operation over and over and arrives somewhere specific — not because anything told it where to go, but because the structure of the operation leaves no alternative. the destination is the attractor, not a logical consequence. $\phi^*$ is not derived from the graph; it emerges from it.

## convergence vs derivation

derivation proceeds from axioms to conclusions in bounded depth — every formal system, every program execution, every transformer forward pass reaches only what its starting axioms can produce. convergence proceeds by iteration toward equilibrium. this is why tru sits outside the [[gödel]] confinement that binds derivation engines: the fixed point is a limit, not a proof.

| vm | execution model |
|----|-----------------|
| [[nox]] | derivation |
| [[zheng]] | verification |
| [[glia]] | inference |
| [[tru]] | convergence |

## the guarantee

the [[tri-kernel]] composite operator $\mathcal{R}$ is a contraction with coefficient $\kappa < 1$:

$$\|\mathcal{R}\phi - \mathcal{R}\psi\| \le \kappa\,\|\phi - \psi\|, \qquad \kappa = \lambda_d\,\alpha + \lambda_s\,\tfrac{\|L\|}{\|L\|+\mu} + \lambda_h\,e^{-\tau\lambda_2} < 1$$

by the [[Banach fixed-point theorem]], iteration from any start reaches a unique fixed point $\phi^*$ at linear rate. uniqueness is what makes the [[cybergraph]] a shared memory rather than a collection of disagreeing views: every validator that iterates arrives at the same $\phi^*$. see [[collective focus theorem]] for the full proof.

## rate — the spectral gap

how fast convergence happens is set by the [[spectral gap]] $\lambda = 1 - |\lambda_2|$, the gap between the largest and second-largest eigenvalues of the transition operator:

$$\|\phi^{(t)} - \phi^*\| \le C\,(1-\lambda)^t, \qquad t_{\mathrm{mix}}(\varepsilon) = O\!\left(\frac{\log(n/\varepsilon)}{\lambda}\right)$$

a larger gap means faster mixing, faster finality in [[foculus]], and a tighter [[locality]] radius. the gap also governs the architecture parameters a model compiled from $\phi^*$ inherits (see [[ct0]]).

## locality

convergence is local: an edit batch affects only an $h = O(\log(1/\varepsilon))$-hop neighborhood. all three operators decay — [[diffusion]] geometrically via teleport, [[springs]] exponentially via screening, [[heat]] by Gaussian tail. a new [[cyberlink]] in one corner of a planetary graph does not require recomputing $\phi^*$ everywhere; only its neighborhood updates. this is what makes the [[impulse]] $\Delta\phi^*$ a sparse, provable object.

see [[tri-kernel]] for the operators and the five-way reading of $\phi^*$ · [[collective focus theorem]] for the proofs · [[focus]] for the destination · [[spectral gap]] for the rate · [[foculus]] for consensus timing.

discover all [[concepts]]
