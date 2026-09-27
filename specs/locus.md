---
tags: cyber, tru, core, spec
crystal-type: spec
crystal-domain: cyber
alias: locus, neuron locus, hyperbolic coordinate, hyperbolic position, locus measure
---
# locus

the locus of a [[neuron]] is its coordinate on the hyperbolic disk, computed every epoch from the follow graph. it is a measure like [[cyberank]] and the spectral positions: tru computes it, other layers read it. [[cybergraph]] carries it as the fourth link of the [[address|address record]]; the [[soft3]] node routes by it ([[routing|the wire obeys the graph]]); [[foculus]] picks fan-out peers by it. tru never routes and never dials.

## why a coordinate

a scale-free graph is a hyperbolic space in disguise: popularity is a radius, similarity is an angle, and the graph's own links almost always lead closer to any target in that geometry. greedy forwarding over existing links then delivers without routing tables (Krioukov, Papadopoulos, Kitsak, Vahdat, Boguñá, *hyperbolic geometry of complex networks*, Phys. Rev. E 82, 2010; Boguñá, Papadopoulos, Krioukov, *sustaining the internet with hyperbolic mapping*, Nat. Commun. 1, 2010). the point for cyber: the address of a neuron can be a function of the graph, not a random identifier, so there is no second network to maintain and no free identifiers to sybil. a neuron earns its place near the centre the way it earns rank: by being followed.

## input · the follow graph

$$F = (V_F, E_F),\qquad V_F = \{\text{name particles of neurons}\},\qquad E_F = \{(\nu \to \mu) : \nu \text{ published } \mathrm{FOLLOW} \to \mu\}$$

only FOLLOW [[cyberlink|cyberlinks]] enter. they are address records a neuron publishes on its own book by choice, so the locus leaks nothing of P1: knowledge links, whose author is private, never enter $F$. weights are the follower's effective stake as [[truth-scoring]] defines it for $A^{\text{eff}}$, so a thousand empty followers weigh less than one bonded one.

$$k_\nu = \sum_{(\mu \to \nu) \in E_F} w_\mu,\qquad N = |V_F|$$

## the coordinate

$$\mathrm{locus}(\nu) = (r_\nu,\ \theta_\nu)$$

**radius from popularity.** the standard hyperbolic map places degree $k$ at radius $2\ln N - 2\ln k$; nodes nobody follows sit on the rim.

$$r_\nu = \operatorname{clip}\big(2\ln N - 2\ln k_\nu,\ 0,\ R\big),\qquad R = 2\ln N$$

**angle from similarity.** the spectral positions [[focusing]] already emits are the top-$k$ eigenvectors of $L + \mu I$; restricted to $F$, the second and third eigenvectors $(x, y)$ are the similarity plane, and the angle is their argument.

$$\theta_\nu = \operatorname{atan2}(y_\nu, x_\nu)$$

**gauge.** eigenvectors are defined up to sign and rotation, so the angle is anchored: the neuron of highest $k$ has $\theta = 0$, and orientation is chosen so that the neuron of second-highest $k$ has $\theta \in (0, \pi)$. with the anchor the coordinate is deterministic: every node that holds $F$ at the epoch boundary computes the same locus for every neuron.

**distance.**

$$d(\nu, \mu) = \operatorname{arccosh}\big(\cosh r_\nu \cosh r_\mu - \sinh r_\nu \sinh r_\mu \cos \Delta\theta\big)$$

routing only compares distances, and for $r_\nu + r_\mu \gg 1$ the comparison is decided by the standard approximation $x = r_\nu + r_\mu + 2\ln\sin(\Delta\theta/2)$; a consumer may use $x$ where only ordering matters.

## arithmetic

every quantity is fixed point over Goldilocks per [[arithmetic]]: $r$ and $\theta$ carry the declared scale, $\ln$ is a table over the admitted range of $k$, $\operatorname{atan2}$ is CORDIC in the field, $\cos$ and $\sin$ are tables. no float anywhere in the path, so two nodes never disagree on a locus by rounding. the [[foculus]] epoch boundary fixes the $F$ the computation reads; the locus of epoch $E$ is a function of $\mathcal{S}_E$ and nothing else.

## properties

- **earned, not chosen.** $r$ falls only as $k$ rises; a sybil with no bonded followers sits at $r = R$ where nobody forwards to it.
- **local.** a neuron's angle moves continuously with its follow neighbourhood; the anchor keeps the whole disk from rotating between epochs unless the top two neurons change.
- **verifiable.** a published locus is a claim; a full node recomputes it from $F$ and rejects a mismatch. partial nodes trust claims within their namespaces and verify on demand.
- **cheap.** the follow graph is small next to the knowledge graph (one link per subscription, not one per thought); the spectral pass is the one focusing already runs.

## outputs and consumers

| output | form | read by |
|---|---|---|
| locus(ν) | $(r, \theta)$ fixed point, per epoch | [[cybergraph]] address record (LOCUS link, claim check) · [[soft3]] node ([[routing]]) · [[foculus]] fan-out and reconciliation peer choice |
| d(ν, μ) | fixed point, or the ordering surrogate $x$ | routing comparisons only |

## open

- the eigenvector count $k$ for partial nodes that hold only a namespace of $F$: they compute a local locus that agrees with the global one only up to the gauge; whether the anchor should be per namespace or per root is decided when [[routing]] is measured on the fleet.
- the refresh rule (how far a locus may move before the neuron republishes its LOCUS link) is the address record's business ([[address]]), not this page's.
- no code yet: `rs/focusing/` emits spectral positions; the radius, gauge and tables are the next slice.
