---
tags: cyber, tru, core, spec
crystal-type: spec
crystal-domain: cyber
alias: arithmetic, field arithmetic, fixed-point, tru arithmetic, no floats, deterministic iteration
---
# arithmetic

tru computes entirely in the [[Goldilocks field]] $\mathbb{F}_p$, $p = 2^{64} - 2^{32} + 1$. every quantity it reads, every iterate it forms, and every weight it emits is a field element. float appears nowhere in the provable path. this page is the representation contract every other tru spec inherits — [[tri-kernel]], [[focusing]], [[impulse]], [[ct0]], and the value function in [[rewards]] all operate under it.

two properties force it, and both are load-bearing:

- determinism — $\phi^*$ is a consensus object. every [[neuron]] and validator must reach the byte-identical distribution for [[foculus]] to finalize it and for `P-DET` (two compiles of one `.graph` produce the same `.model` particle) to hold. float is non-deterministic across hardware — fused-multiply-add contraction, reassociation, rounding mode, denormals — so a float $\phi^*$ could never be agreed on.
- provability — $\Delta\phi^*$ is the quantity a neuron self-mints against and [[zheng]] proves. SuperSpartan and sumcheck operate over field elements; there is no proof system over IEEE-754. the tri-kernel iteration has to be a [[nox]] trace over $\mathbb{F}_p$ or the mint is unprovable.

## 1. the field

the substrate is the [[Goldilocks field]] $\mathbb{F}_p$ shared across the whole stack — the same field [[hemera]] hashes in, [[nox]] computes in, [[zheng]] proves over, and [[bbg]] commits. an element is one canonical residue in $[0, p)$. tru introduces no arithmetic of its own; it spends the field's.

## 2. fixed-point semantics

linear algebra needs rationals — $\phi^*$ is a probability vector, the embedding $E$ carries singular weights. tru represents a rational $x$ as the field element $X = \mathrm{round}(x \cdot \Sigma) \bmod p$ at a fixed scale $\Sigma$ (the stack [[fixed-point]] convention; `rs/core` provides the reference `FixedPoint`). $\Sigma$ is a compile-time constant recorded in `config`, so two runs share it and agree bit for bit.

the distinction that resolves the apparent paradox: tru does fixed-point rational arithmetic in which every value is stored as an $\mathbb{F}_p$ element — not abstract finite-field algebra. there is no "SVD over $\mathbb{F}_p$" — singular values and ordering are undefined in a bare finite field. magnitude and order are the order of the fixed-point representatives, which is total, canonical, and deterministic. the numerics are real-valued in meaning; the field is the storage and proof substrate.

signedness uses the balanced range: residues above $p/2$ read as negative, so $-a \equiv p - a$. negative stake (a $v=-1$ link) is clipped to zero before matrix construction ([[ct0]] §3.4), so adjacency stays in the nonnegative cone; signed values appear only in derived quantities (bivector grades, embeddings).

## 3. operations

every numerical step is one of these, and each is an $\mathbb{F}_p$ constraint exercising the four [[Goldilocks field processor|GFP]] primitives (fma, ntt, p2r, lut):

- add, subtract — field add. exact, no rescale.
- multiply — field multiply, then rescale by $\Sigma$. the rescale is the one nontrivial op: a division by $\Sigma$ realized as a witnessed quotient and remainder with a range check on the remainder (the p2r / lut primitives), never a field inverse. two scaled values multiply to scale $\Sigma^2$; the truncation brings it back to $\Sigma$.
- compare, max, threshold — on the canonical representative; total and deterministic.
- reciprocal, square root — the normalizations $1/\!\sum\phi$ and $\mathrm{diag}(\sqrt{\phi^*})$ are fixed-point Newton iterations to a fixed depth, each step an fma + rescale. no transcendental is ever evaluated directly.

transcendentals are forbidden and replaced by polynomials: the heat kernel $\exp(-\tau L)$ is a Chebyshev polynomial in $L$ ([[focusing]], [[tri-kernel]] §1.3) — pure fma over $\mathbb{F}_p$. this is why the spec mandates Chebyshev truncation rather than a matrix exponential: a polynomial is field-native, a matrix exponential is not.

## 4. deterministic iteration

a provable, agreed trace cannot loop "until $\|\phi^{(t)} - \phi^{(t-1)}\| < \varepsilon$" — the stopping point is data-dependent, so the trace length varies by machine and the proof shape is not fixed. tru iterates a constant number of steps instead. the composite contraction $\kappa < 1$ ([[tri-kernel]] §2.2) gives the bound directly:

$$T(\varepsilon) = \left\lceil \frac{\log(1/\varepsilon)}{\log(1/\kappa)} \right\rceil$$

every iterative method — the coupled [[tri-kernel]] iteration for $\phi^*$, the power iteration and Lanczos in the [[ct0]] architecture pass, the per-head SVD in the attention pass, the local recompute for [[impulse]] — runs exactly $T(\varepsilon)$ steps, a compile-time constant. the result is bit-exact across all machines, and the trace length is known before the computation runs (which is also what [[ct0]] reads $\kappa$ to set $L^*$).

## 5. the no-float invariant

float occurs nowhere tru produces or proves:

- focusing — operators, blend, normalization, $\phi^*$, cyberank, syntropy: all fixed-point $\mathbb{F}_p$.
- impulse — $\Delta\phi^*$ is a sparse vector of fixed-point field entries.
- ct0 compile — the $\phi^*$-weighted adjacency, randomized SVD, embedding, attention projections, Clifford block, and norms are all computed in fixed-point over $\mathbb{F}_p$; the emitted tensors are integer encodings of field elements (§10.8), and the on-disk format carries no floats.
- rewards — the value function $v$, the surprise weight $\rho$, karma $\kappa$, and the Shapley shares are fixed-point field quantities.

the single boundary where a float may be named is import: [[model]] §import quantizes an external checkpoint (a HuggingFace or GGUF model trained elsewhere in float) to integer encodings once, on the way in. that float dies at the boundary and never enters a tru computation. a model CT-0 compiles is field-native from the first pass and never crosses that boundary.

## 6. where each spec stands

| spec | quantity | representation |
|------|----------|----------------|
| [[tri-kernel]], [[focusing]] | operators, $\phi^*$, cyberank, syntropy | fixed-point $\mathbb{F}_p$, $T(\varepsilon)$ steps |
| [[impulse]] | $\Delta\phi^*$ | sparse fixed-point $\mathbb{F}_p$ |
| [[ct0]] | $M$, SVD, $E$, attention, Clifford, norms | fixed-point $\mathbb{F}_p$; tensors stored as integer encodings |
| [[model]] | weights on disk | integer encodings (u32/u16/q8/q4/ternary); import is the only float boundary |
| [[rewards]] | $v$, $\rho$, $\kappa$, Shapley shares | fixed-point $\mathbb{F}_p$ |
| inf reads (karma, price, signals) | inputs to $A^{\text{eff}}$ | field elements; no lift to float on entry |

discover the machine that runs this arithmetic in hardware: [[Goldilocks field processor]].
