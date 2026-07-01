---
tags: cyber, tru, roadmap, spec
crystal-type: plan
crystal-domain: cyber
alias: tru implementation plan, tru build plan, what to build next
---
# implementation plan

the full build order for every [[tru]] spec — from the one engine that is built (the [[focusing]] stub) to the eight-pass [[CT-0]] compiler and the cross-repo reward settlement. each milestone has an input, an output artifact, and a verifiable predicate. dependency order is strict: a later milestone may assume every earlier one passes.

this plan refines two existing maps: [specs/README.md](../specs/README.md) (layer/status view) and the [implementation steps table](../README.md) (step ids 0a–3). it adds the concrete module layout, the algorithm per spec, and the exact predicate names so an engineer can start at M1 and not stop.

one invariant cuts across every milestone: tru computes in fixed-point over the [[Goldilocks field]], never float ([[arithmetic]]). $\phi^*$ is a consensus object and $\Delta\phi^*$ is what [[zheng]] proves, so float is doubly excluded — non-deterministic and unprovable. M0 builds that arithmetic; M1 ports the `f64` stub onto it; M5 compiles in it. the only float anywhere is an external checkpoint quantized once at the [[model]] import boundary.

## state today

| layer | spec | status |
|-------|------|--------|
| focusing | [[tri-kernel]] | ✅ conformant engine built (M1: coupled iteration, fixed-point `Fx`, deterministic) |
| focusing | [[focusing]] | 🟡 φ* only (binary topology); cyberank + syntropy missing |
| focusing | attention, truth-scoring, impulse | ⬜ spec only |
| format | vocab, model | ⬜ / 🟡 writer scaffold (`unimplemented!`) |
| compile | ct0 (8 passes) | ⬜ spec only — the bulk of the work |
| economics | rewards | 📐 spec complete; settlement spans [[foculus]]/[[tok]] |

built: the `.graph` reader (`rs/graph/`), the fixed-point field type (`rs/arithmetic.rs`, M0), the conformant focusing engine + spectral contraction (`rs/focusing/`, M1 — coupled iteration, deterministic), and cyberank + syntropy + telemetry (M1.5). 23 tests green. Still doc-comment stubs: `rs/pass/mod.rs`, `rs/model/writer.rs`.

## critical path

```
M0 foundation ─┬─ M1 focusing conformance ─── M1.5 cyberank+syntropy ─┐
               │        (no external deps)                            │
               ├─ M2 format layer (vocab + model writer) ─────────────┤
               │                                                       ▼
               └─ M3 effective adjacency ── M4 impulse ──── M5 CT-0 passes 1–8 ── M6 conformance
                     (needs bbg reads)                          (the bulk)         (P-* harness)
                                                                                        │
                                                              M7 economics (measurement math; mint/settle = foculus/tok)
```

The single highest-value, lowest-cost, zero-external-dependency change is M1: replace the three-independent-solves-averaged form with one coupled iteration. Do it first.

---

## M0 — foundation wiring

the substrate the critical path needs. the field arithmetic is the real M0 — the rest of the old list was M2/M3 plumbing with no consumer yet, so it lands with its first consumer rather than as dead code.

| task | detail | status |
|------|--------|--------|
| field arithmetic | the representation contract ([[arithmetic]]): a thin fixed-point layer at scale $\Sigma = 2^{32}$ over [[nebu]]'s `Goldilocks` (`cyb-nebu`, `../strata/nebu/rs`). tru adds encode/decode, mul-then-rescale (i128 widen → round → reduce mod p), div/recip, integer `sqrt`, order on the balanced residue. no `f64` on any deterministic path. `rs/arithmetic.rs`, type `Fx` | ✅ done — 7 tests green |
| wire [[hemera]] | `cyber-hemera` for particle ids / axon hash / file particles | ↦ at first use (M2 formats) — avoid an unused dep |
| config `.tokens` | token_weight $\rho_\tau$ from `config.tokens` | ↦ M3 (its only consumer is $A^{eff}$) |
| generalize `.cyb` reader | extract a format-agnostic opener from `rs/graph/reader.rs` (drops the hardcoded `"graph"` assertion at `reader.rs:35`) | ↦ M2, when vocab + model become the second and third consumers (no premature abstraction) |
| `Serialize` on frontmatter | emit path for the writers | ↦ M2 (with the model/vocab writers) |

predicate (met): `Fx` round-trips encode/decode, mul rescales within one ULP of the rational, div/recip/sqrt correct, order respects sign+magnitude, division-by-zero degrades to zero (checked form reports it). the existing 6 focusing tests get ported onto `Fx` in M1.

---

## M1 — focusing engine conformance (math + arithmetic) — ✅ done

fully conformant — coupled iteration, fixed-point over Goldilocks, stake-weighted A_eff, Chebyshev heat on the combinatorial L, and a κ-derived step count from a real λ_max/λ₂ (`rs/focusing/spectral.rs`). no deferrals remain. 18 tests green (`rs/arithmetic.rs` 8 + `rs/focusing/` 10) including bit-identical determinism, κ<1 contraction, T(ε) convergence, and heat mass-conservation.

1. blend-of-attractors → fixed. `compute_focusing` was three independent solves averaged once ([[tri-kernel]] §2.4 / [[focusing]] forbid it — no single free energy, no single $\kappa$). now one coupled iteration: `diffusion_step`/`springs_step`/`heat_step` each apply once to the shared φ, blend `λ_d·D+λ_s·S+λ_h·H`, normalize, feed back, ×`iters`.
2. float → fixed. every `f64` replaced by `Fx` (M0). $\phi^*$ is now bit-identical across runs — the determinism [[arithmetic]] requires and `f64` could never give. field addition is associative, so the HashMap-order in the build no longer threatens determinism.

the settled form is one coupled iteration, run a fixed step count (no float-threshold loop):

```
φ ← u                                  (uniform or stake prior), fixed-point 𝔽_p
repeat exactly T(ε) = ⌈log(1/ε)/log(1/κ)⌉ times:
    d ← D_step(φ)                      one diffusion application:  α Pᵀφ + (1−α)u
    s ← S_step(φ)                      one springs relaxation step: toward (L+μI)⁻¹(μ·x₀)
    h ← H_step(φ)                      one bounded heat application: Chebyshev Σ cₖ(τ)Tₖ(L̃)φ
    φ ← norm(λ_d·d + λ_s·s + λ_h·h)     blend + simplex-normalize, all in fixed-point
```

| task | detail | size |
|------|--------|------|
| port to field type | replace every `f64` in `rs/focusing` with the M0 fixed-point $\mathbb{F}_p$ type; reciprocal/sqrt (the `norm` and degree divides) via fixed-point Newton; no float literals | M |
| operators → single-step | rewrite `operators.rs` `diffusion/springs/heat` from self-converging solves into `*_step(φ)` maps. heat via Chebyshev three-term recurrence on $\tilde L = 2L/\lambda_{max}-I$ (a polynomial in L — field-native, no matrix exponential) | M |
| outer coupled loop | rewrite `compute_focusing`: blend single-steps, simplex-normalize, feed φ back, run exactly $T(\varepsilon)$ steps computed from $\kappa$; `max_iter` is replaced by the derived step count | S |
| fix springs RHS | `operators.rs:68` uses `stake` as RHS; spec is $\mu\cdot x_0$ (TK §1.2 $(L+\mu I)x^*=\mu x_0$) | S |
| weighted P | diffusion transition currently binary $1/\text{outdeg}$ (`focusing.rs:138`); derive $P$ from $A^{eff}$ once M3 lands. until then run on stake-weighted $A^{eff}$ | S |

predicates (from [[tri-kernel]] §2.2 / [[focusing]] / [[arithmetic]]):
- determinism: two runs on the same input produce the bit-identical $\phi^*$ (this is the field-arithmetic payoff; impossible under `f64`)
- contraction: $\kappa = \lambda_d\alpha + \lambda_s\frac{\|L\|}{\|L\|+\mu} + \lambda_h e^{-\tau\lambda_2} < 1$ (test computing $\kappa$ from $\|L\|$ via power iteration and Fiedler $\lambda_2$), and $T(\varepsilon)$ steps reach the fixed point within $\varepsilon$
- simplex: $\sum_i \phi^*(i)=1$ (in fixed-point, to one ULP), $\phi^*(i)>0\ \forall i$ (port `focus_sums_to_one`)
- ranking sanity: port `high_in_stake_ranks_higher`, `well_linked_node_ranks_higher`

---

## M1.5 — focusing outputs: cyberank + syntropy — ✅ done

`rs/focusing/measures.rs`; 4 tests green. `syntropy` is emitted in `FocusingResult` and printed by the CLI.

| task | detail | status |
|------|--------|--------|
| cyberank | `cyberank(g, result, particle) → φ*(p)` by hash, zero if absent ([[cyberank]]) | ✅ |
| syntropy | $J(\phi^*)=\sum_j \phi^*(j)\ln(|V|\phi^*(j)) = D_{KL}(\phi^*\|u)$, a fixed-point field on `FocusingResult`; $\ln$ is fixed-point range-reduce + atanh series (`Fx::ln`), never `f64::ln` ([[syntropy]]) | ✅ |
| telemetry | `Telemetry { particles, syntropy, entropy H, lambda_2, kappa, steps }` (TK §6.3, the cheap monitors). ℓ₁-drift + locality-radius are per-signal → M4 (impulse) | ✅ |

predicates (met): $J(u)=0$ at uniform; $J\ge 0$; $J$ grows with concentration; cyberank over all particles sums to 1; emitted `result.syntropy` equals recomputed $J$ bit-for-bit.

---

## M1.6 — superadditivity benchmark (measure collective intelligence) — ✅ done

the empirical validation that the engine produces collective intelligence, not just a fixed point — the measurement method of [[superadditivity]]. needs M1 (φ*) and M1.5 (J). produces the first real numbers, so it doubles as the engine's correctness witness.

| task | detail | size |
|------|--------|------|
| ego baseline | for each neuron ν, build its ego-net (radius r) and run the same coupled iteration → $\phi^*_\nu$ | M |
| task scorers | link-prediction (hide edges → ROC-AUC, average precision) and retrieval@10 over focus-ranked particles | M |
| σ metric | $\sigma_{\text{mean}} = Q(\phi^*) - \text{mean}_\nu Q(\phi^*_\nu)$, $\sigma_{\text{best}} = Q(\phi^*) - \max_\nu Q(\phi^*_\nu)$ | S |
| connectivity sweep | add edges in $\lambda_2$ order; record $\sigma$ and $J$ vs $\lambda_2$ to test the generalized-CFT monotonicity | M |

datasets: Zachary Karate Club (34 particles) as the smallest sanity instance, then a real cybergraph snapshot. predicate: $\sigma_{\text{best}} > 0$ and rising with $\lambda_2$ — the collective strictly beats its strongest neuron, by more as the graph connects. report measured figures only (no targets baked into the spec); they land in a benchmark output + `docs/explanation/superadditivity` once run on the conformant engine.

harness: `rs/examples/superadditivity.rs` (`cargo run -p tru --example superadditivity`).

measured on the fully-conformant engine (M1: coupled iteration, fixed-point over Goldilocks, Chebyshev heat, κ-derived T(ε), bit-identical across runs) — Karate Club, 80/20 split, predictor φ(p)·φ(q): collective AUC 0.725 vs best-ego 0.589, mean-ego 0.511 → σ_best(AUC) +0.135, σ_mean(AUC) +0.214; J(φ*) 0.084. On AP the collective beats the average (σ_mean +0.069) but not the single best neuron (σ_best −0.091). So superadditivity holds clearly for global ranking (AUC) — the collective strictly outranks its strongest neuron — and on-average for AP; a well-placed neuron still wins locally on AP.

λ₂ connectivity sweep (fixed 34-node spanning tree + k edges, so λ₂ is Fiedler-monotone) — the generalized-CFT test, and it partly refutes the conjecture, honestly: **σ rises with λ₂** (Pearson +0.5 mean; σ_best > 0 at every level — the collective beats its strongest neuron throughout), but **J falls with λ₂** (Pearson −0.7) — densifying spreads focus toward uniform, lowering syntropy. So connectivity buys collective *advantage* but costs *syntropy*; they are distinct axes. Spec ([[superadditivity]]) + tri-kernel §3 + syntropy term updated to the measured result. Still pending: retrieval@k (needs personalized focus, not the single global φ*).

---

## M2 — format layer (containers)

the two on-disk formats. prerequisites for the compiler: vocab feeds pass 1, model is the output of pass 8. authoritative byte layout for the writer is [[ct0]] §10.

### vocab (`specs/vocab.md`) — step 0a

`.cyb` container, two sections: `card` (md) + `particles` (binary). `particles` layout:

```
[0..4]  n  (u32 LE)
×n:  [0..32] particle (hemera hash)  [32..40] len (u64 LE)  [40..40+len] data
```

entry index = vocab id. `len=0` valid (registers existence). self-consistency: `hemera(data)==particle` when `len>0`. file identity: `particle(.vocab)=hemera(file bytes)`.

| task | size |
|------|------|
| `Vocab` + `VocabEntry` structs, new `rs/vocab/` module | S |
| `ParticleEntryIter` over mmap'd `particles` slice (mirror `graph::record::CyberlinkIter`) | S |
| writer: frontmatter + binary serialize, precompute section `size` $=4+\Sigma(40+\text{len}_i)$ | S–M |
| multi-vocab composition with first-hit-wins dedup (CT-0 pass 1 §3.1) | M |

predicate: round-trip — parse then re-emit yields byte-identical file and same file particle.

### model (`specs/model.md` + ct0 §10) — step 0b, 2g

`.cyb` container, seven sections: card, config, program, tensors, vocab, eval, weights. `weights` is binary, 4096-byte page-aligned per tensor (zero-copy mmap), integer encodings only — CT-0 emits u16 (projections, `round(v·256)`) and u32 (norms, `round(v·65536)`). `config` and `tensors` are integers-only TOML (e.g. `rms_norm_eps=1000000` for 1e-6).

| task | size |
|------|------|
| extend `Model` to hold card/config/program/tensors/vocab/eval + weights blob | M |
| `TensorEntry { name, shape, encoding, offset, size }`, `enum Encoding` (U16/U32 now; Q8/Q4/Ternary deferred to CT-2) | S |
| weights blob assembly with per-tensor 4096-byte alignment + u16/u32 encode | M |
| section emitters: `rs/emit/{card,config,tensors,vocab,eval,weights}.rs` | M |
| file particle `hemera(bytes)` + `certificate.toml` sidecar (§12) | S |

predicate: P-DET — two runs produce byte-identical `.model` (same particle, §10.9). Full P-LOAD waits on cyb-llm runtime.

current scaffold: `model/writer.rs` is `unimplemented!("Model::write")`. The whole write path is new; the read side (`graph/`) is the working half to mirror.

---

## M3 — effective adjacency, truth-scoring, attention input

the weighted graph the focusing engine actually runs on. partially blocked: karma and price are bbg reads.

$$A^{eff}_{pq} = \sum_{\ell:\,p\to q} \text{stake}(\ell)\cdot\kappa(\nu(\ell))\cdot f(m(\ell))$$

- stake$(\ell)=a(\ell)\cdot\text{token\_weight}(\tau(\ell))$ — normalizes denominations from `config.tokens`
- $\kappa(\nu)$ = karma — accumulated BTS history, read from [[bbg]], written by [[plumb]]. tru reads, never writes
- $f(m(\ell))$ = ICBS price → edge multiplier in fixed-point $[0,1]$ — [[market inhibition]]. valence $v$ does not enter $A^{eff}$ directly; it acts through $f(\text{price})$
- all inputs are field-native: stake, karma, and price arrive as fixed-point $\mathbb{F}_p$ elements and $A^{eff}$ is assembled in fixed-point — there is no lift to float on entry ([[arithmetic]]). a read path through inf (the cybergraph query layer) returns field elements directly
- attention ([[attention]]): a neuron's per-edge weight = will-share (broad [[will]] lock auto-distributed across its links) + per-link conviction ([[box]] $(\tau,a)$). this is one summand $a\cdot\kappa\cdot f(m)$
- truth-scoring ([[truth-scoring]]): BTS score accumulates into karma; the cyberlink IS the BTS input (belief = $(\tau,a)$, meta = $v$, identity = $\nu$)

| task | detail | size | blocker |
|------|--------|------|---------|
| extend `Link` | add `neuron`, `token`; carry/look up `karma`, `price` | S | — |
| `adjacency.rs` | $A^{eff}$ assembly: token_weight, $f(\text{price})$ map $[0,1]$, karma/price join | M | needs bbg shape |
| BTS + karma magnitudes | $s^{BTS}$, karma accumulation as a computation (substrate may live in a sibling) | M | — |
| wire bbg reads | karma $\kappa(\nu)$ and ICBS price $m(\ell)$ per epoch, joined by $\nu$ and $\ell$ | L | bbg accessor |

predicate: adjacency matches [[focusing]] §effective-adjacency; market-doubted edges ($m\to0$) suppressed not deleted; until bbg lands, engine runs on stake-only $A^{eff}$ (the M1 fallback).

note: the README step table's "Pearson ≥ 0.7" and "§3.4 / §3" labels belong to ct0 §11 predicates, not to the focusing specs — track them under M6, not here.

---

## M4 — impulse Δφ*

the per-signal focus shift, the unit a neuron mints against.

$\Delta\phi^* = \phi^*_{after} - \phi^*_{before}$ for a neuron's link batch — a sparse $(particle, \Delta\phi^*)$ vector. computed locally: run the coupled iteration on the $O(\log(1/\varepsilon))$-hop neighborhood only. the locality theorem (TK §2.2) bounds out-of-radius effects below $\varepsilon$, so $N_h$ recompute gives global error $\le\varepsilon$. conservation (impulse.md): per-epoch minting bounded by actual global $\Delta\phi^*$; overlapping claims scaled proportionally.

| task | size |
|------|------|
| neighborhood extraction $N_h$, $h=O(\log 1/\varepsilon)$ | M |
| `compute_impulse(graph, batch) -> Vec<(particle, Fp)>` — local coupled solve (fixed-point field, same $T(\varepsilon)$ as the global pass), before/after diff, sparse pack | L |
| directed form $\Delta\phi^+ = [J(\phi^*_{t+1})-J(\phi^*_t)]_+$ for rewards | S |

predicate: locality — recomputing on $N_h$ vs full graph agrees within $\varepsilon$; sparse support (most entries zero).

doc fix: `impulse.md` (lines 21, 25) says "stark proof" — per repo convention these are [[zheng]] proofs (see [[reference_zheng_not_stark]]). correct the wording.

new module: `rs/focusing/impulse.rs`.

---

## M5 — CT-0 compiler (8 passes) — the bulk

deterministic `compile: G → M`. every pass computes in fixed-point over $\mathbb{F}_p$ ([[arithmetic]]) — the $\phi^*$-weighted adjacency, the randomized SVD, the embedding, the projections, the Clifford block, the norms — and emits integer-encoded field tensors. byte-identity across machines (P-DET) follows from there being no float to diverge. the two heavy passes (3, 5) are the randomized-SVD / matrix-power numeric core; the rest are linear scans, constant fills, or serialization. proposed layout:

```
rs/
  input/   graph.rs · stake.rs (eff stake w(ℓ), clip<0) · multivector.rs (AxonWeight{w0,w2}, EffAdj{a0,a2})
  pass/    pass1..pass8.rs
  geometry/ wedge.rs · inner.rs        (mirror nox jets shifted_wedge_product / shifted_inner_product)
  numeric/ rsvd.rs (ChaCha20-seeded) · lanczos.rs (λ2) · power_iter.rs (φ* + impulse reuse)
  emit/    card·config·tensors·vocab·eval·weights
  verify/  p_embed · p_attn · p_layer · p_det · p_load · p_clifford_{a,b,c}
```

multivector foundation (§2.5–2.6): axon weight $w=w_0+w_2$ and effective adjacency $A^{eff}=A^{eff}_0+A^{eff}_2$ each carry a scalar grade and an oriented bivector grade ($w_2(p,q)=-w_2(q,p)$, valence-oriented consensus, sparse CSR with $i<j$). degeneracy: when $w_2=0$ and $A^{eff}_2=0$ everywhere, all Clifford terms vanish and output is byte-identical to a scalar compile. determinism: every seeded init is ChaCha20 from `hemera(L‖ν_compiler)`; sign convention SC-1 (largest-abs entry per singular column made positive).

| pass | § | builds | step | size |
|------|---|--------|------|------|
| 1 particle index | §3 | ordered $V$ + idx + CSR adjacency $A$ | 2a | M |
| 2 dialect discovery | §4 | dialect set $S$ + per-link $\sigma(\ell)$; $h^*=|S|$ | 2b | S |
| 3 architecture | §5 | $\phi^*$, $d^*$ (spectral entropy of φ-weighted $M$), $h^*$, $L^*$, $\kappa$, $\lambda_2$; partial rSVD of $M$ | 2c | L |
| 4 embedding | §6 | $E=U_{:,1:d^*}\sqrt{\Sigma_{1:d^*}}$ (Eckart-Young optimal) | 2d | M |
| 5 attention | §7 | per layer/head $W_Q,W_K,W_V,W_O$ + `alpha_beta` | 2e | L |
| 5+ wedge | §7.7 | shifted-wedge score term (Clifford); $\beta$ init 0, no gradient when $A^{eff}_2=0$ | 2e | M |
| 6 MLP | §8 | Clifford block: `proj`, `gate`, `gamma`, `context.weight_{1,2}` | 2f | M |
| 7 norms | §9 | unit-vector RMSNorms; RoPE meta ($\theta_0{=}10000$, computed at load); lm_head tied to embed | 2g | S |
| 8 package | §10 | the `.model` file + certificate | 2g | M |

pass-5 detail: per dialect $s$, layer $l$: $A^{(s,l)}=(A^{(s)})^{l_{eff}}$ (sparse matpow, never densified) → project $P^{(s,l)}=E^\top A^{(s,l)}E$ → SVD truncated to $d_h=d^*/h^*$ → $W_Q,W_K$ from $U,V$; $W_O$ = Moore-Penrose pseudoinverse of concatenated $W_V$.

the shifted geometric product is the shared primitive of passes 5 and 6: `Inner_s(H,C)=σ(H·shift_s C)` (SiLU over $\mathbb{F}_p$ via LUT) and `Wedge_s(Q,K)=Q·shift_s K − shift_s Q·K` (strictly anti-symmetric). `geometry/` must agree with the nox jets within $\varepsilon_j=10^{-9}$ (P-CLIFFORD-C).

deps: the numeric kernels (randomized SVD, power iteration, Lanczos, pseudoinverse) are fixed-point $\mathbb{F}_p$ implementations on the M0 field type — not nalgebra's `f64` routines; sprs/ndarray serve as sparse/dense containers only, parameterized over `Fp`. rand_chacha (deterministic seeding), hemera. one to confirm in the [[nox]] crate: the `shifted_inner_product` / `shifted_wedge_product` jets — a fixed-point reference impl is required regardless for P-CLIFFORD-C.

arithmetic note: one representation end to end — fixed-point over the [[Goldilocks field]], compile and inference and proof alike ([[arithmetic]]). there is no float-vs-field split; P-CLIFFORD-C checks the `geometry/` kernels against the nox jets, both fixed-point. the `.model` tensors are integer encodings of those field elements (ct0 §10.8). float exists only outside CT-0, at the [[model]] import boundary.

---

## M6 — conformance harness (ct0 §11)

every predicate, stored per-mille/boolean in the `.model` `eval` section + `certificate.toml`.

| predicate | § | checks |
|-----------|---|--------|
| P-EMBED | §11.1 | $\|EE^\top-M\|_F/\|M\|_F \le 0.05$ |
| P-ATTN | §11.2 | $\forall l,s:\ \text{Pearson}(\text{flat}(W_QW_K^\top),\text{flat}(P^{(s,l)}))\ge 0.7$ |
| P-LAYER | §11.3 | fixed-seed len-128 seq: layer-to-layer change monotonically nonincreasing $\forall l\ge1$ |
| P-DET | §11.4 | two runs → byte-identical `.model` (same particle) |
| P-LOAD | §11.5 | cyb-llm loads, mmaps, one forward pass → finite logits; HF export round-trips |
| P-CLIFFORD-A | §11.6 | wedge anti-symmetry $\text{Wedge}_s(X,X)=0$ within $\varepsilon_w=10^{-6}$ |
| P-CLIFFORD-B | §11.6 | zero-bivector graph → byte-identical to scalar-only compile |
| P-CLIFFORD-C | §11.6 | nox jets match scalar reference within $\varepsilon_j=10^{-9}$ on 64-element fixed set |

P-LOAD and HF export need the cyb-llm runtime (`~/git/cyb/llm`) — the only external runtime dependency. P-DET, P-EMBED, P-ATTN, P-LAYER, P-CLIFFORD-A/B/C are self-contained.

---

## M7 — economics (measurement math only)

tru ships the value-magnitude layer; minting and settlement are cross-repo. the §14 boundary:

| concern | repo |
|---------|------|
| value magnitude ($\Delta\phi^+$, karma, syntropy, Shapley shares) | tru |
| finality, canonical φ*, settlement lottery | [[foculus]] |
| conservation, allocation, mint execution | [[tok]] |
| proofs $\sigma$ | [[zheng]] |
| identity | [[mudra]] |

implementable now inside tru: $J$ and $\Delta J$ syntropy (S); $A^{eff}$ assembly (S, shares M3); $\Delta\phi^+$ impulse (M, shares M4); the value set-function $v(S)=\Delta\phi^+(A^{eff}\cup S)$ and surprise-weighted $v^*(S)=\Delta\phi^+(A^{eff}\cup\rho S)$ (M); BTS surprise $\rho_\ell$ + karma $\kappa$ magnitudes (M); ε-support cluster geometry (M); Shapley Monte-Carlo estimator over $k$ orderings (L, with a mock beacon).

v=0 rule (rewards §9): a void-valence link is passive stake — it weights $A^{eff}$ so it moves rank, but earns nothing, by category not penalty. enforced in tru's attribution layer: the surprise gate $\rho$ and the active-stake ($v\ne0$) selectors admit it to rank but not to reward; the BTS crowd reference excludes $v=0$ reports.

invariants: conservation = Shapley efficiency $\sum_\nu \text{mint}(\nu)=v^*(N)\le\Delta\phi^+(N)$; substitutes ceiling $\text{Shapley}_\nu(v^*)\le\Delta\phi^+_\nu$; sybil-neutrality by stake-weighting; stakeless PoW onramp.

hard cross-repo blockers (out of v0.1): the VDF beacon $b$ (foculus) for un-front-runnable orderings; zheng proof generation + the unbuilt aggregation/accumulator; tok conservation clip + mint execution + allocation PID; foculus canonical φ* + finality depth. tru's Shapley math runs against an injected/mock beacon until foculus lands.

---

## sequencing summary

1. M0 foundation — ✅ done (`Fx` fixed-point over nebu::Goldilocks); unblocked all
2. M1 focusing conformance — ✅ done (coupled iteration in `Fx`, stake-weighted, deterministic); 8 tests green
3. M1.5 cyberank + syntropy — ✅ done (deterministic J, cyberank accessor, telemetry)
4. M1.6 superadditivity benchmark — ✅ done (σ>0 confirmed; λ₂ sweep: σ rises with λ₂, J falls — conjecture half-refuted)
5. M2 format layer — vocab + model writer (parallelizable with M1)
6. M3 effective adjacency — partial until bbg reads land
7. M4 impulse — needs M1
8. M5 CT-0 passes 1–8 — the bulk; needs M1–M3 + M2 model writer
9. M6 conformance harness — needs M5; P-LOAD needs cyb-llm
10. M7 economics — measurement math now; mint/settle blocked on foculus/tok/zheng

built so far: M0 (field arithmetic) + M1 (conformant focusing engine, deterministic) + M1.5 (cyberank, syntropy, telemetry). the whole focusing layer is done. next M1.6 (superadditivity λ₂ sweep) + M2 (formats), then M5 is where the volume of work lives.
