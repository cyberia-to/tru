---
tags: cyber, tru, core, spec
crystal-type: spec
crystal-domain: cyber
alias: focusing, field computation, focus computation, tri-kernel computation, cyberank computation
---
# focusing

focusing is tru's continuous computation — the act whose output is [[focus]] ($\phi^*$). every epoch: read signals from [[cybergraph]], build effective adjacency, run [[tri-kernel]], emit $\phi^*$ and derivatives.

## inputs

every [[signal]] contributes a header (ν, t) and one or more [[cyberlink]] tuples (p, q, τ, a, v):

| field | type | meaning |
|-------|------|---------|
| ν | 32-byte particle | neuron — the sender |
| p | 32-byte particle | from-particle |
| q | 32-byte particle | to-particle |
| τ | 32-byte particle | token denomination |
| a | u64 goldilocks | stake amount in smallest denomination units |
| v | i8 ∈ {-1, 0, +1} | valence — neuron's prediction of link validity |
| t | u64 | unix timestamp |

tru reads two additional quantities from [[bbg]] per epoch:

- karma(ν): accumulated [[BTS]] score history for each neuron — the long-run record of honest signaling
- price(ℓ): [[ICBS]] market price per link — the market's collective epistemic assessment of link validity

stake is economic commitment. karma is epistemic track record. price is collective verdict. φ* is always computed from these three weighted inputs, never stored independently.

## effective adjacency

raw stake is not the edge weight. tru constructs effective adjacency from the full truth-weighted signal:

```
A_eff(p, q) = Σ_{ℓ: from=p, to=q}  stake(ℓ) × karma(ν(ℓ)) × f(price(ℓ))
```

where stake(ℓ) = a(ℓ) × token_weight(τ(ℓ)) normalizes across denominations using the weights declared in config.tokens.

f(price) maps ICBS price to an edge multiplier in [0, 1]. a link the market believes (price → λ) carries full weight. a link the market doubts (price → 0) carries diminished weight. this is [[market inhibition]] — collective epistemic assessment prunes false connections structurally.

valence v ∈ {-1, 0, +1} does not directly enter A_eff. its effect is mediated through price(ℓ): valence is the BTS meta-prediction, and BTS scoring accumulates into karma and drives ICBS market convergence.

## tri-kernel

the composite operator:

```
R = λ_d · D + λ_s · S + λ_h · H_τ      (λ_d + λ_s + λ_h = 1)
```

### diffusion

column-stochastic transition matrix P = A_eff · diag(1 / col_sum(A_eff)):

```
φ^(t+1) = α · P^T φ^(t) + (1 - α) · u
```

α ∈ (0, 1): teleport parameter. u: prior (uniform or stake-weighted). teleport ensures ergodicity — probability mass occasionally restarts from the prior, preventing trapping in dense subgraphs.

locality: geometric decay with rate α. a local edit's effect reaches ε precision within O(log(1/ε)) hops.

answers: where does probability flow?

### springs

screened laplacian solve. let L = diag(col_sum(A_eff)) − A_eff:

```
(L + μI) x* = μ x_0
```

μ > 0: screening parameter. x_0: reference state (often uniform). the screened green's function (L + μI)^−1 decays exponentially with graph distance — locality with exponential tail. larger μ pulls harder toward x_0; smaller μ lets structure dominate.

locality: exponential decay with rate O(exp(−μ^{1/2} · d)).

answers: what satisfies structural constraints?

### heat

heat kernel approximated by chebyshev polynomial truncation at degree K. let L̃ = 2L / λ_max − I:

```
H_τ ≈ Σ_{k=0}^{K} c_k(τ) T_k(L̃)
```

c_k(τ) are the chebyshev coefficients of exp(−τ·). τ ≥ 0: temperature. high τ smooths broadly across the graph (annealing). low τ focuses locally (crystallization). the ability to adjust τ lets tru operate simultaneously across multiple scales.

locality: gaussian tail decay, O(log(1/ε)) hops.

answers: what does the graph look like at scale τ?

## fixed point

the [[collective focus theorem]] guarantees: under ergodicity of P, μ > 0, and bounded τ, the composite operator R is a contraction:

```
‖Rφ − Rψ‖ ≤ κ ‖φ − ψ‖,   κ = λ_d α + λ_s ‖L‖/(‖L‖+μ) + λ_h e^{−τλ_2} < 1
```

by the banach fixed-point theorem, φ^(t) → φ* at linear rate. the fixed point is unique and satisfies:

```
φ* = norm[R(φ*)]        Σ_i φ*(i) = 1        φ*(i) > 0 ∀ i
```

computation is one coupled iteration: each step applies D, S, and H_τ to the same current φ, blends with weights λ, and normalizes — repeat to the fixed point. tru does not solve the three operators independently to their own fixed points and average the results; that is a different, weaker object that minimizes no single free energy and has no single κ (see [[tri-kernel]] §2.4). the contraction κ < 1 governs this coupled iteration, and the [[ct0]] architecture parameters read κ from it.

φ* is the boltzmann equilibrium minimizing the free energy functional:

```
F(φ) = λ_s E_spring(φ) + λ_h E_heat(φ) + λ_d D_KL(φ ‖ D(φ))
```

every cyberlink shifts φ*. learning and knowledge state are the same operation.

## eigensolver

tru runs LOBPCG (locally optimal block preconditioned conjugate gradient) on the screened laplacian (L + μI) to extract the k leading eigenvectors V_k.

each particle receives a position in k-dimensional spectral space: row i of V_k is particle i's coordinate. particles that are structurally similar (densely interconnected) cluster in spectral space. these positions are emitted to [[mir]] every epoch as the geometric substrate of the R-1.0 world.

## outputs

| output | definition | consumer |
|--------|-----------|---------|
| φ* | tri-kernel fixed point, Σ φ*(i) = 1 | [[foculus]], self-minting proofs, CT-0 compilation |
| cyberank(p) | φ*(p) — focus per particle | [[glia]] routing, [[cyb]] ranking, [[cybernode]] queries |
| spectral positions | top-k eigenvectors of (L + μI) | [[mir]] world geometry |
| syntropy J | Σ_j φ*(j) · log(|V| · φ*(j)) | network health, norm pass in CT-0 |
| Δφ*(ν, batch) | φ*_after − φ*_before for neuron ν's link batch | self-minting proof input to [[zheng]] |

karma is not written by tru's focusing pass. karma is accumulated by [[plumb]] from BTS scores and read from [[bbg]] as input. tru reads karma; [[plumb]] writes it.

see [[collective focus theorem]] for the convergence proofs. see [[ct0.md]] for how φ* feeds into model compilation. see [[tri-kernel]] for why these three operators are the minimal sufficient basis.
