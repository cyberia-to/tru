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

with `out_p = Σ_q A_pq`, `in_p = Σ_r A_rp`, `d_p = out_p + in_p` (undirected degree = node stake), `π = d/Σd` (the stake prior), the ground weight `w0_p = ⌊in_p/4⌋`, the out-row normaliser `N_p = out_p + w0_p`, its teleport share `s_p = w0_p/N_p` (`s_p = 1` when `N_p = 0` and `d_p > 0`), `P = A·diag(1/N)` and `W = (A + Aᵀ)·diag(1/d)` (column-stochastic, with `P` completed by `s`):

```
D(φ) = α·π·Σφ + (1−α)·(P φ + π·Σ_p s_p φ_p)
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
| ground `γ_i` (`w0_p = γ_i·in_p`) | 1/4 (amended 2026-10-17, §7) |
| attribution nodes `k` | 4 Gauss–Legendre nodes (amended 2026-10-17, §7) |
| steps | `PHI_STEPS = 48`, from the uniform vector `⌊2^60/n⌋` |

this is [[tri-kernel]] with the normalised (random-walk) Laplacian `I − W` for springs and heat, which tri-kernel §1.2 admits, and the stake prior as both the teleport target and the springs reference. three reasons, all measured: the combinatorial heat kernel `exp(−τ(D − A))` needs `O(τ·λ_max)` sparse passes per step and `λ_max` is the largest weighted degree (36,026 on bostrom at uniform stake, 9,007 Chebyshev sub-applications per step, about 2.6 h per step on one core); the normalised operators are scale-free, so one large stake does not change everyone's heat time; and every operator maps the simplex to itself, so the step is an affine contraction with the a-priori rate `κ = λ_d(1−α) + λ_s/(1+μ) + λ_h = 0.775` — a parameter function, no eigenvalue estimate. a particle without stake has zero prior, zero inflow and zero focus.

the ground weight (amended 2026-10-17). without it, a particle with focus and no out-axon sends all of its focus to the prior, and its first out-axon — at any positive weight — redirects all of it: φ* jumps, and no gradient sees a jump (§7). with it, every particle's out-row carries a virtual weight `w0_p = in_p/4` that teleports, so the share its out-axons carry, `out_p/(out_p + w0_p)`, grows continuously from zero. a particle without in-weight is unchanged; a particle without out-weight is pure teleport, as before. the rule is scale-free (a ratio of the particle's own weights) and keeps every operator column-stochastic, so κ is unchanged.

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
Mᵀλ = λ_d·[(1−α) f + α·c_t] + λ_s·[μ c_t + Wᵀλ]/(1+μ) + λ_h e^{−τ} Σ_k τ^k/k! (Wᵀ)^k λ,    c_t = πᵀλ,    f_i = (Pᵀλ)_i + s_i·c_t
```

(the constant parts of `∇J` vanish because `∂M/∂A` moves no mass; a constant shift of λ changes no gradient). per directed axon `p → q`, with `u_j = (Wᵀ)^j λ`, `v_i = W^i φ`:

```
∂J/∂A_pq = λ_d (1−α) φ_p (λ_q − f_p) / N_p
         + λ_d (1−α) γ_i φ_q (c_t − f_q) / N_q                                  (raising A_pq raises in_q, so w0_q)
         + (λ_d (α + (1−α) m_T) + λ_s μ/(1+μ)) (λ_p + λ_q − 2 c_t) / Σd      m_T = Σ_i s_i φ_i / Σφ
         + [λ_s/(1+μ) · φ_p (λ_q − u₁_p) + λ_h e^{−τ} Σ_{k=1}^{K} τ^k/k! Σ_{j<k} v_{k−1−j,p} (u_{j,q} − u_{j+1,p})] / d_p
         + [the same with p ↔ q] / d_q
```

each line is `∂(xᵀ C y)/∂C_·p = y_p (x_· − (Cᵀx)_p)/norm_p` for a column-normalised operator `C`. (the implementation carries a `γ_d·d_p` ground term as well, zero in the protocol.) an absent axon (`A_pq = 0`) has gradient 0. numerators are formed at 2^60 with the coefficients applied, then `num·2^52/den` by `(num/den)·2^52 + ((num mod den)·2^52)/den` (truncation toward zero), the four quotients summed (saturating), and the weight shift undone by a rounded right shift.

the attribution gradient is the path average over `k` Gauss–Legendre nodes `(t_i, w_i)` on `[0, 1]` (2^32 scale, the weights summing to exactly 2^32):

```
A_i  = A_prev + ⌊t_i·(A_cur − A_prev) / 2^32⌋      per axon of the union, floor (arithmetic shift)
ḡ_pq = Σ_i ⌊w_i·g_pq(A_i) / 2^32⌋                  (256-bit product, truncated toward zero)
```

`path_gradient(prev, cur, p)` returns `ḡ` in the union's ascending order; with one node `A_1` is exactly `⌊(A_prev + A_cur)/2⌋`. the chain's attribution `a(c) = Δs(c)·σ(c)·ḡ_pq` (chain-economy §5.2) is a 256-bit product at 2^112; `mul_wide_shr(a, g, 52)` gives it in nats at 2^60. nodes (2^32): `k = 4`: t = 298207500, 1417379916, 2877587380, 3996759796; w = 747012592, 1400471056, 1400471056, 747012592.

## 5. digest

`phi = hemera(D_PHI ‖ n: u32 ‖ φ[0]: u64 ‖ … ‖ φ[n−1]: u64)`, little-endian, particles ascending, `D_PHI = "soft3/chain/phi/v1\0"` (chain-economy §5.6). tru produces the preimage (`Phi::preimage`); the chain hashes it with its hemera (0.4). vector: `n = 2, φ = [2^59, 2^59]` → `3351bfa4…9a8a`.

## 6. conformance

`rs/tests/epoch_settlement.rs` pins, at the defaults: φ and J of a five-particle, seven-axon graph (φ preimage digest `e0ff79997aa697bced38e2cdedb823b189892982bae411933ec623007de18fc8` under hemera 0.4; `e9462109…` before the ground amendment), the gradient of each of its axons, the 4-node path gradient of a fixed change of it (an axon raised, one removed, one added, one new particle), the digest preimage of the chain vector, bit-identity across 1–8 threads and across input orders, and the gradient against central finite differences (0.2 %). any change of a bit of these is a consensus change (chain-economy §4.4) and moves the vectors.

## 7. continuity and the attribution rule (amended 2026-10-17)

the integrated gradient is the Aumann–Shapley value of a smooth game ([[rewards]] §4); it can only match the Shapley value where φ* is continuous in `A`. before the ground weight one change was not: a particle gaining its first out-axon (or losing its last). against exact Shapley on random small graphs the one-node midpoint then erred by a median 34–45 %, and more nodes did not help (a jump is invisible to every quadrature).

the ground weight and the node count were chosen together, by measurement against exact Shapley (audit `epoch-settlement-2026-10.md` §6), under the acceptance: median error ≤ 5 % in every class of change including first/last out-axon, ≤ 1 % for weight changes ≤ 30 %, settlement on bostrom ≤ 300 s on one core, `PHI_STEPS` still reaching ℓ1 ≤ 10⁻⁷. `γ_i = 1/4` with four nodes meets it with the widest margin measured: medians 0.65 / 1.49 / 3.03 % on first/last out-axon at changes of ≈ 5 / 30 / 100 %, 0.49 / 1.00 / 1.77 % on new particles, 0.10 / 0.35 / 1.26 % otherwise, 0.04 % on pure weight changes ≤ 30 %. the midpoint alone stays near 6 % on new particles at any ground (a new particle's `φ ln φ` grows from zero), so one node cannot meet it; two nodes meet it at `γ_i = 1/4` with less margin. smaller grounds move φ* less but leave less margin (`γ_i = 1/10` with four nodes: 4.73 % on first/last out-axon at ≈ 100 % changes, 0.3 points under the bar; with two nodes it misses).

the price is a change of φ*: on pussy the ground moves φ* by ℓ1 0.21 against the plain dangling rule and keeps 877 of the top 1,000 particles; it also raises the effective teleport, so 48 steps reach the rounding floor (ℓ1 ≈ 10⁻¹⁰).
