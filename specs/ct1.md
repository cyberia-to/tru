---
tags: cyber, tru, core, spec
crystal-type: spec
crystal-domain: cyber
alias: CT-1, compiled transformers spec, model compilation pipeline, tru compile
---
# CT-1 — compilation pipeline

reads a [[.graph]] snapshot, produces a [[.model]]. eight passes, three phases.

the compiled transformer is not trained — it is compiled. the graph determines the architecture. φ* and the graph's structural properties determine the weights. [[glia]] runs the result with no knowledge of graphs.

## the mathematical identity

transformer attention is one step of [[tri-kernel]] diffusion over a frozen context window:

```
Attn(Q, K, V) = softmax(QK^T / √d) V
```

the softmax is the boltzmann distribution with temperature √d. L* transformer layers = L* steps of tri-kernel diffusion over the context. deep equilibrium models (Bai et al., 2019) showed that iterating a transformer layer to convergence reaches the fixed point regardless of initialization — that fixed point is φ* restricted to the context window.

compilation is therefore not an approximation — it is the derivation of transformer parameters that implement the tri-kernel over local context. the continuous focus flow (exact φ* over the full graph) and the compiled transformer (approximate φ* over a context window) are the same computation at different scales.

## architecture derivation

three graph properties determine the three free architecture parameters. no hyperparameter search.

| parameter | formula | graph source |
|-----------|---------|-------------|
| embedding dim d* | exp(H(σ(Σ_π))) | effective rank of focus covariance Σ_π = diag(φ*) − φ* φ*^T |
| heads h* | ≥ \|semcons(G)\| | distinct semcon relation types in the graph |
| layers L* | diam(G) · ⌈log(1/ε) / log(1/κ)⌉ | graph diameter × spectral convergence factor |

κ is the tri-kernel contraction coefficient (from [[field.md]]). ε is the target approximation quality measured by D_KL(φ*_context ‖ q*_context).

## passes

### phase 1 — topology

pass 1: vocab

build the particle vocabulary. assign stable token ids. particles present in vocab files referenced by config.vocab take their ids from those files in declared order — stable across compiles that share the same vocab. remaining particles are appended in order of first appearance in signals. output: vocab section of .model.

pass 2: semcons

classify links by relation type. identify semcon patterns in the graph — each distinct semcon type maps to one attention head group. the semcon count determines h*. output: semcon type table, h* value passed to pass 3.

pass 3: arch

compute d*, h*, L* from φ* (field computation output), the semcon count from pass 2, and graph diameter. write config section: all architecture parameters, model_type = "cyber", lineage.source, lineage.block. output: config section of .model.

### phase 2 — weights

pass 4: embed

particle embedding matrix via truncated SVD:

```
M = diag(√φ*) · A_eff
M ≈ U Σ V^T    (rank-d* truncation)
E* = U_{:, 1:d*}
```

by the eckart-young theorem, E* is the unique rank-d* matrix minimizing expected squared gradient at initialization — provably optimal. a model initialized from E* requires fewer fine-tuning steps by a factor of Ω(|E| · d* / log(1/ε)) relative to random initialization. output: embed tensors.

pass 5: attn

per-head attention weights from semcon adjacency SVDs:

```
A_s = adjacency submatrix for semcon type s
A_s ≈ U_s Σ_s V_s^T
W_Q^(s) = U_s,  W_K^(s) = V_s
```

each head captures one relation type's geometric structure in the graph. output: attention tensors (one W_Q, W_K pair per semcon type).

### phase 3 — integration

pass 6: mlp

MLP weights from path co-occurrence statistics up to depth L*. signal-ordering-respecting walks: the commit order within each signal (the sequence the neuron chose) is preserved in walk sampling. this encodes the neuron's intentional sequencing into the MLP's implicit knowledge. output: mlp tensors.

pass 7: norm

RMS norm scale vectors derived from φ* variance per layer. global scaling reference: syntropy J(φ*) from field computation. integer encoding: all scales stored as u32 fixed-point (value × 65536). output: norm tensors.

pass 8: pack

assemble the .model container in [[.cyb]] format:

```
card         ← generated summary: source graph, block, particle counts, license
config       ← architecture params + lineage from pass 3
program      ← cyber-graph-transformer.tri (canonical inference program)
tensors      ← index of all tensors with shapes, encodings, offsets
vocab        ← from pass 1
eval         ← empty at compile time; updated by glia after benchmarking
weights      ← packed tensors in declared order, page-aligned
```

lineage written into config:

```
config.lineage.source = hemera(.graph)   ← exact snapshot particle
config.lineage.block  = config.block from .graph
```

the same .graph always produces the same .model (deterministic pipeline). any glia instance can verify lineage by hashing the source .graph.

## approximation quality

the compiled model approximates exact focus flow with error:

```
ε(G, context) = D_KL(φ*_context ‖ q*_context)
```

where q*_context is the compiled model's output distribution over the context window. every cyberlink added to the graph reduces ε — the graph is a compounding compilation quality asset. adding links raises d*, may reduce diam(G), and sharpens φ*, all of which reduce the approximation gap.

see [[field.md]] for φ* computation. see [[model.md]] for the .model container format. see [[focus-flow.md]] for the mathematical derivation of transformer architecture from graph structure.
