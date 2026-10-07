---
tags: cyber, tru, core, spec
crystal-type: spec
crystal-domain: cyber
alias: proving focus, provable focusing, tri-kernel circuit, φ* circuit
---
# proving — the tri-kernel as a zheng circuit

[[focusing]] computes $\phi^*$; this page costs proving it. one [[zheng]] proof that the composite [[tri-kernel]] operator converged to the claimed $\phi^*$ over the committed graph replaces every voting round — that consequence is [[foculus]]'s ([[foculus/specs/provable-consensus|provable consensus]]); the circuit, its constraint budget and its feasibility argument are tru's, because they are the cost of tru's own computation.

the content moved here from `foculus/specs/provable-consensus.md` on 2026-10-07 so the kernel is derived once ([[soft3/roadmap/component-boundaries|component boundaries]]).

## status

[[zheng]] implements the domain-local core — `prove_phi_star` / `verify_phi_star`, multi-row CCS for sparse matvec over a bounded neighbourhood ([[zheng/specs/phi-spmv|phi-spmv]]). the full-graph circuit below is an estimate; no full-graph proof has been produced.

## the kernel in circuit

$\phi^*$ is the fixed point of the composite operator ([[tri-kernel]] §2.4):

$$\phi^{(t+1)} = \text{norm}[\lambda_d \cdot \mathcal{D}(\phi^t) + \lambda_s \cdot \mathcal{S}(\phi^t) + \lambda_h \cdot \mathcal{H}_\tau(\phi^t)]$$

- $\mathcal{D}$ (diffusion) — random walk on weighted adjacency
- $\mathcal{S}$ (springs) — screened Laplacian: mean neighbour focus
- $\mathcal{H}_\tau$ (heat) — multi-scale smoothing: 2-hop context

two ingredients make the proof possible:

1. $\phi^*$ is deterministic given the graph — same adjacency, same $\phi^*$ — hence verifiable
2. [[algebraic state commitments]] let the circuit read the graph as polynomial evaluations instead of hash paths — O(|E|) field ops instead of O(|E| · log n) [[hemera]] hashes

### iteration

each iteration applies three sparse matrix operations and combines them:

```
D_step:  d[j] += α × w_ij × φ[i] / out_degree[i]    # diffusion (PageRank)
         d[i] += (1-α) / |P|                           # teleport

S_step:  s[i] = Σ_j (A_ij + A_ji) × φ[j] / degree[i] # springs (mean neighbour)

H_step:  h = A × (A × φ)                               # heat (2-hop)
         h[i] /= degree²[i]                            # normalise

combine: φ_new = normalize(λ_d × d + λ_s × s + λ_h × h)
```

default weights: $\lambda_d = 0.5$, $\lambda_s = 0.3$, $\lambda_h = 0.2$ ([[foculus/specs/parameters|parameters]]). all arithmetic is fixed-point over Goldilocks ([[arithmetic]]).

### cost per iteration

each operator is a sparse matrix-vector multiply (SpMV). heat needs two ($A^2\phi = A(A\phi)$). figures are for [[bostrom]]: 2.9M particles, 2.7M edges.

| operator | SpMV count | edge ops | particle ops | constraints |
|---|---|---|---|---|
| D (diffusion) | 1 | \|E\| muls + divs | \|P\| adds (teleport) | ~8.3M |
| S (springs) | 1 | \|E\| muls + divs (symmetric) | \|P\| divs | ~8.3M |
| H (heat) | 2 | 2×\|E\| muls + divs | \|P\| divs | ~16.6M |
| combine | 0 | 0 | 3×\|P\| muls + \|P\| adds + normalise | ~14.6M |
| total per iteration | 4 SpMV | | | ~47.8M |

### convergence

bostrom converges in 23 iterations (measured contraction $\kappa = 0.74$, $\lambda_2 = 0.13$ — [[spectral gap from convergence]]). the composite's contraction rate is $\kappa = \lambda_d \alpha + \lambda_s\frac{\|L\|}{\|L\|+\mu} + \lambda_h e^{-\tau_{\text{heat}}\lambda_2}$ ([[convergence]]) — a convex combination of the operators' own rates, not their max; a convex combination of terms each below 1 is itself below 1, which is the whole $\kappa < 1$ proof and only holds for the weighted sum. the measured 0.74 is the actual composite, so 23 iterations stand under the correct formula.

constraints for $\phi^*$: 23 × 47.8M = **1.1B**.

### zheng capacity

[[zheng]] (SuperSpartan + Brakedown) handles up to $2^{32}$ = 4.3B constraints. 1.1B / 4.3B = 25.6 % — the full tri-kernel uses a quarter of what zheng can prove. the remaining 74.4 % holds graph reads, finalisation checks, nullifier verification and state-transition application.

## reading the graph

### through NMT: infeasible

each edge read is an NMT inclusion proof — O(log n) hemera hashes in-circuit:

```
per edge:    32 hemera calls × 736 constraints = 23,552 constraints
2.7M edges:  2.7M × 23,552 = 63.6 BILLION constraints

zheng capacity: 4.3B constraints
overflow: 15× over capacity
```

### through algebraic NMT: feasible

each edge read is a PCS evaluation — O(1) field operations:

```
per edge:    ~100 field operations ≈ 100 constraints
2.7M edges:  2.7M × 100 = 270M constraints

plus tri-kernel computation: 1,100M constraints
plus finalisation:            50M constraints

total:                       1,420M constraints
zheng capacity:              4,300M constraints
utilisation:                 33 %
```

feasible with 67 % headroom. the tri-kernel reads edges three times per iteration (D once, S once, H twice via $A^2\phi$); once the first read is in the witness the later reads are free. effective read cost: 270M once.

## the circuit — six sections

```
PROVABLE_FOCUSING_CIRCUIT:

inputs:
  BBG_commitment  — polynomial commitment to graph state (32 bytes)
  φ_claimed       — claimed tri-kernel fixed point (focus distribution)
  finality_set    — particles claimed final (φ_i > τ)

witness:
  A_edges         — all edge weights (opened from BBG_commitment)
  out_degrees     — per-particle out-degree
  sym_edges       — symmetric adjacency (A + Aᵀ) for springs
  φ_iterations    — intermediate φ vectors, one per iteration

constraints:

  // SECTION 1: graph read (270M)
  for each edge (i, j, w) in A_edges:
    assert PCS_eval(BBG_commitment, (axons_out, i, j)) == w
    // one polynomial evaluation check per edge; cached for D, S, H

  // SECTION 2: degree and symmetry consistency (5.8M)
  for each particle i:
    assert sum(w_ij for j in outgoing[i]) == out_degree[i]
    assert sym_degree[i] == in_degree[i] + out_degree[i]

  // SECTION 3: tri-kernel iteration (1,100M)
  for t in 0..22:
    // D: diffusion
    for each edge (i → j, w):   d[j] += α × w × φ_t[i] / out_degree[i]
    for each particle i:        d[i] += (1-α) / |P|
    // S: springs
    for each edge (i, j, w_sym): s[i] += w_sym × φ_t[j] / sym_degree[i]
    // H: heat — h_temp = A × φ_t, then h = A × h_temp
    for each edge (i → j, w):   h_temp[j] += w × φ_t[i]
    for each edge (i → j, w):   h[j] += w × h_temp[i]
    for each particle i:        h[i] /= degree²[i]
    // combine
    for each particle i:        φ_next[i] = λ_d × d[i] + λ_s × s[i] + λ_h × h[i]
    φ_next /= sum(φ_next)
    assert φ_iterations[t+1] == φ_next

  // SECTION 4: convergence (2.9M)
  assert ||φ_iterations[22] - φ_iterations[21]||_1 < ε

  // SECTION 5: finality (|F| × 3)
  for each particle i in finality_set:
    assert φ_claimed[i] > τ(t)
    assert i has valid nullifiers (not in the spent set)

  // SECTION 6: output binding
  assert φ_claimed == φ_iterations[22]

total: ~1,420M constraints (33 % of zheng capacity)
```

sections 5 and 6 are the consensus consequence; the τ rule and nullifiers are specified in [[foculus/specs/protocol|protocol]].

## times

**prover.** 1.42B constraints at ~1 µs per constraint = 1,420 s ≈ 24 minutes on a CPU. per epoch, not per block: one proof covers the full tri-kernel over the whole graph, and blocks inside the epoch inherit the proven $\phi^*$. SuperSpartan's SpMV is embarrassingly parallel — ~2–3 minutes on a GPU, practical at epoch boundaries (every 100 blocks ≈ 500 s).

**verifier.** O(log N) = O(log 1.42B) = 31 hemera hashes + field ops ≈ 50 µs. the complete tri-kernel over 2.9M particles, checked on a phone, without downloading the graph.

## dependency chain

```
nebu           Goldilocks field arithmetic
  ↓
hemera         Poseidon2 — signal identity, NOT state reads
  ↓
zheng          SuperSpartan + Brakedown — proves the φ* computation
  ↓
nox            patterns: compute + call + look — SpMV as an execution trace
  ↓
algebraic NMT  polynomial state — O(1) graph reads in-circuit
  ↓
proving        φ* proven correct
```

remove algebraic NMT and graph reads cost O(log n) hemera each — 15× over zheng capacity. algebraic NMT is the prerequisite: without it focus must be voted; with it focus can be proven.

[[tru]] · [[focusing]] · [[tri-kernel]] · [[impulse]] · [[zheng/specs/phi-spmv|phi-spmv]] · [[foculus/specs/provable-consensus|provable consensus]]
