---
tags: cyber, tru, core, spec
crystal-type: spec
crystal-domain: cyber
alias: epoch engine, epoch focus, root chain focus, epoch gradient, PHI_STEPS
---
# epoch

the form of [[focusing]] the phase-1 root chain runs in consensus. every full node computes, at every epoch boundary, from chain state alone: φ* of the staked graph, its [[syntropy]] J, and the gradient ∂J/∂A of syntropy with respect to every axon weight, which the chain multiplies by stake changes to divide the epoch's budget ([[rewards]] §4, §16; soft3 `specs/chain-economy.md` §4–§5). this page is normative for the operators, the arithmetic, the step count, the transposes and the scales. the implementation is `rs/focusing/epoch/`; measurements are `audit/epoch-settlement-2026-10.md`.

## 1. input

a list of axons `(p, q, A_pq)`, `A_pq: u128`, the chain's net weight `max(0, G_pq + S⁺_pq − S⁻_pq)` (chain-economy §4.2). in any order: self-loops and zero weights are dropped, duplicates sum, particles are indexed by ascending 32-byte id, axons by `p ‖ q`. the result never depends on input order.

if `Σ A ≥ 2^62`, every weight is shifted right by the smallest `s` that brings the total below `2^62`; a positive weight never shifts to zero (it is kept at 1). gradients are reported per raw weight unit, the shift undone.

## 2. operators

with `out_p = Σ_q A_pq`, `d_p = out_p + Σ_r A_rp` (undirected degree = node stake), `π = d/Σd` (the stake prior), `P = A·diag(1/out)` and `W = (A + Aᵀ)·diag(1/d)` (both column-stochastic):

```
D(φ) = α·π·Σφ + (1−α)·(P φ + π·Σ_{out_p = 0} φ_p)
S(φ) = (μ·π·Σφ + W φ) / (1+μ)
H(φ) = e^{−τ} Σ_{k=0}^{K} τ^k/k! · W^k φ
φ'   = norm(λ_d·D(φ) + λ_s·S(φ) + λ_h·H(φ))
```

| parameter | value |
|---|---|
| α (teleport) | 0.15 |
| μ (screening) | 1 |
| τ (heat time) | 1 |
| λ_d, λ_s, λ_h | 0.5, 0.3, 0.2 |
| K (heat degree) | 9 — the smallest with `2·e^{−τ}·τ^{K+1}/(K+1)! ≤ ε = 10⁻⁶` |
| steps | `PHI_STEPS = 48`, from the uniform vector `⌊2^60/n⌋` |

this is [[tri-kernel]] with the normalised (random-walk) Laplacian `I − W` for springs and heat, which tri-kernel §1.2 admits, and the stake prior as both the teleport target and the springs reference. three reasons, all measured: the combinatorial heat kernel `exp(−τ(D − A))` needs `O(τ·λ_max)` sparse passes per step and `λ_max` is the largest weighted degree (36,026 on bostrom at uniform stake, 9,007 Chebyshev sub-applications per step, about 2.6 h per step on one core); the normalised operators are scale-free, so one large stake does not change everyone's heat time; and every operator maps the simplex to itself, so the step is an affine contraction with the a-priori rate `κ = λ_d(1−α) + λ_s/(1+μ) + λ_h = 0.775` — a parameter function, no eigenvalue estimate. a particle without stake has zero prior, zero inflow and zero focus.

the legacy combinatorial kernel (`compute_focusing`) remains the research reference; it is not the consensus object.

## 3. arithmetic

integers only, every value below 2^63 and so a canonical Goldilocks residue ([[arithmetic]] §2 with a scale per quantity):

| quantity | scale | storage |
|---|---|---|
| φ | 2^60, `Σφ ≈ 2^60` | u64 |
| P, W entries | 2^40, `round(w·2^126/x)` via the reciprocal `⌊2^126/x⌋` | u64 |
| parameters α, 1−α, λ, μ/(1+μ), 1/(1+μ), τ/k, τ^k/k!, λ_h·e^{−τ} | 2^32, from `Fx` | u64 |
| π | 2^60 | u64 |
| ln, adjoint λ | 2^40 | i64 |
| J | 2^60 nats | i128 |
| gradient g | 2^112 nats per weight unit | i128 |

products widen to u128/i128; every rescale rounds half up (unsigned) or half away from zero (signed). one step evaluates `W φ` once (shared by springs and the innermost Horner term), the heat polynomial by Horner (`g_K = φ + (τ/K)·Wφ`, `g_k = φ + (τ/k)·W g_{k+1}`, `H = λ_h e^{−τ}·g_1`), then `D`, `S`, the blend, and normalises with the factor `⌊2^92/Σ⌋` (`φ_i = round(φ'_i·f/2^32)`). each output entry is a function of its own row; every global quantity is an exact integer sum. the result is therefore identical for any thread count, any chunking and any internal relabelling of the particles (the implementation orders them breadth-first for memory locality and reports in id order).

`ln` is integer: `v = 2^e·(1 + i/256)·(1 + δ)`, a compile-time table of `ln(1 + i/256)` at 2^62 (by the `atanh` series) and five terms of `ln(1+δ)`, `δ < 2^-8`; error below 2^-41.

`J(φ) = Σ_i φ_i (ln φ_i − 60·ln 2 + ln n) / 2^40` at 2^60, over the graph's `n` particles.

## 4. the gradient

at the fixed point `φ = M φ` (M the step before normalisation, column-stochastic on the simplex) the envelope theorem gives `dJ = λᵀ (∂M/∂A) φ` with the adjoint

```
λ ← Mᵀ λ + g,    g_i = ln φ_i − Σ_j φ_j ln φ_j / Σφ,    λ₀ = 0,    PHI_STEPS steps
Mᵀλ = λ_d·[(1−α) Pᵀλ + (α + (1−α)·[out_i = 0])·c_t] + λ_s·[μ c_t + Wᵀλ]/(1+μ) + λ_h e^{−τ} Σ_k τ^k/k! (Wᵀ)^k λ,    c_t = πᵀλ
```

(the constant parts of `∇J` vanish because `∂M/∂A` moves no mass; a constant shift of λ changes no gradient). per directed axon `p → q`, with `u_j = (Wᵀ)^j λ`, `v_i = W^i φ`:

```
∂J/∂A_pq = λ_d (1−α) φ_p (λ_q − (Pᵀλ)_p) / out_p
         + (λ_d (α + (1−α) m_0) + λ_s μ/(1+μ)) (λ_p + λ_q − 2 c_t) / Σd      m_0 = Σ_{out_i=0} φ_i / Σφ
         + [λ_s/(1+μ) · φ_p (λ_q − u₁_p) + λ_h e^{−τ} Σ_{k=1}^{K} τ^k/k! Σ_{j<k} v_{k−1−j,p} (u_{j,q} − u_{j+1,p})] / d_p
         + [the same with p ↔ q] / d_q
```

each line is `∂(xᵀ C y)/∂C_·p = y_p (x_· − (Cᵀx)_p)/norm_p` for a column-normalised operator `C`. the dangling set is piecewise constant and has no derivative. an absent axon (`A_pq = 0`) has gradient 0. numerators are formed at 2^60 with the coefficients applied, then `num·2^52/den` by `(num/den)·2^52 + ((num mod den)·2^52)/den` (truncation toward zero), the four quotients summed (saturating), and the weight shift undone by a rounded right shift.

the chain's attribution `a(c) = Δs(c)·σ(c)·g_pq` (chain-economy §5.2) is a 256-bit product at 2^112; `mul_wide_shr(a, g, 52)` gives it in nats at 2^60.

## 5. digest

`phi = hemera(D_PHI ‖ n: u32 ‖ φ[0]: u64 ‖ … ‖ φ[n−1]: u64)`, little-endian, particles ascending, `D_PHI = "soft3/chain/phi/v1\0"` (chain-economy §5.6). tru produces the preimage (`Phi::preimage`); the chain hashes it with its hemera (0.4). vector: `n = 2, φ = [2^59, 2^59]` → `3351bfa4…9a8a`.

## 6. conformance

`rs/tests/epoch_settlement.rs` pins, at the defaults: φ and J of a five-particle, seven-axon graph (φ preimage digest `e9462109f934da2baca3aef7f903033d829bb245f404c203c7a7352cec2ab1e0` under hemera 0.4), the gradient of each of its axons, the digest preimage of the chain vector, bit-identity across 1–8 threads and across input orders, and the gradient against central finite differences (0.2 %). any change of a bit of these is a consensus change (chain-economy §4.4) and moves the vectors.

## 7. limits of the gradient form

the integrated gradient is the Aumann–Shapley value of a smooth game ([[rewards]] §4). where the game is smooth — weights rising or falling on axons whose endpoints keep their out-axons — the chain's midpoint rule agrees with the exact Shapley value to a median 0.01 % (changes ≤ 5 % of an axon), 0.2 % (≤ 30 %) and 2.7 % (≤ 100 %); new particles are continuous and land at a median 5–6.5 %. one change is not smooth: a particle gaining its first out-axon (or losing its last) switches its focus from the teleport prior to that axon at any positive weight — a jump in φ* the gradient does not see. on such epochs the midpoint attribution differs from exact Shapley by a median 34–45 % (audit). the jump is not paid to anyone (the budget is divided by attribution, so the mint stays conserved); it is an under-payment of first out-links. closing it needs a continuous dangling rule — an open question for [[rewards]] §16.
