---
title: epoch settlement — φ*, J and the syntropy gradient at chain scale
tags: tru, audit, focusing, chain
date: 2026-10-17
---

# epoch settlement — measured 2026-10-17

Receipt for tru `feat/epoch-settlement` (spec: [specs/epoch.md](../specs/epoch.md);
chain contract: soft3 `specs/chain-economy.md` §4–§5, rewards.md §16). Machine:
Apple M4 Max (12 P + 4 E cores, 48 GB), macOS 26.4, rustc 1.95 release.
Data: the burial snapshots' `cyberlinks_indexed.csv.gz` converted by
`eval/links_bin.py` — pussy 29,112 links / 16,271 particles, bostrom
2,949,732 links / 3,143,650 particles. Synthetic stake per link (no stake
exists on chain yet): uniform 10⁶, or Pareto(α = 1.16, scale 10⁶, cap 10¹⁵).
Particle ids are hashed (random order, as real hemera ids), except where noted.

Harnesses: `rs/examples/epoch_bench.rs` (modes `legacy`, `epoch`, `settle`,
`phi-only`, `gradient-only`), `rs/examples/epoch_shapley.rs`, and the ignored
tests `bostrom_phi_within_budget` / `bostrom_settlement_within_budget` in
`rs/tests/epoch_settlement.rs`. Peak memory is `/usr/bin/time -l` maximum RSS
(whole process, harness inputs included) and, for the API calls alone, a
counting allocator's heap peak above the live baseline.

## 1. before — the legacy engine (`FocusingGraph::build` + `compute_focusing`)

Every link at its stake, `Context::none()`, default `FocusingParams`.

| graph | stake | build | λ_max | T | heat sub-applications per step | one solve | peak RSS | Σφ |
|---|---|---|---|---|---|---|---|---|
| pussy | uniform | 0.18 s | 4,173.7 | 16 | 1,044 (×8 SpMV) | 86.2 s (5.39 s/step) | 18 MB | 1.000000119 |
| pussy | power-law | 0.15 s | 3.77 | 21 | 1 | 0.14 s | 19 MB | 0.999999988 |
| bostrom | power-law | 27.4 s | 2.00 | 19 | 1 | 24.6 s (1.29 s/step) | 2.04 GB | 0.999965696 |
| bostrom | uniform | 27.6–33.5 s | 36,026 | 15 | 9,007 (72,056 SpMV) | one step did not finish in 40 min; projected ≈ 2.6 h/step, ≈ 39 h/solve | 1.88 GB at build | — |

Findings:

- the combinatorial heat kernel's cost is `O(τ·λ_max)` and `λ_max` is the
  largest weighted degree: at uniform stake one bostrom solve is a day and a
  half of one core. Its cost depends on the stake distribution, so it cannot
  carry a consensus deadline.
- the λ₂ estimate (120 power iterations) is wrong on these graphs: bostrom
  uniform reports λ₂ = 37,253 > λ_max = 36,026, and pussy reports 1.96 for a
  disconnected graph whose λ₂ is 0. κ and T(ε) inherit the error. T is also
  order-dependent (nodes indexed by first appearance).
- 2⁻³² resolution: bostrom's φ sums to 0.999966 (n rounding errors), and a mean
  entry of 2⁻²¹·⁵ keeps about three significant digits.
- the soft3 spec's own baseline (chain-economy §7.2, genesis weights): one
  bostrom solve 48 s, 2.8 GB.
- deterministic across two runs (bit-identical) in every case.

## 2. after — the epoch engine (`tru::focusing::epoch`)

Positions: one per link, valence +1, every 13th passive (0), every 31st
against (−1, subtracting); `net_axons` → 2,848,062 axons over 3,061,768
particles on bostrom (25,878 / 15,965 on pussy). `PHI_STEPS = 48`, cold from
uniform.

### 2.1 φ*

| graph | stake | build | 48 steps, 1 thread | 4 threads | 12 threads | ℓ1 to 192-step reference | Σφ − 1 |
|---|---|---|---|---|---|---|---|
| pussy | uniform | 5.6 ms | 28.8 ms | 32 ms | 53 ms | 5.9 × 10⁻⁸ | −9.6 × 10⁻¹¹ |
| pussy | power-law | 5.7 ms | 29.5 ms | 30 ms | 52 ms | 9.2 × 10⁻⁸ | −1.2 × 10⁻¹⁰ |
| bostrom | power-law | 2.07 s | 6.94 s (145 ms/step) | 2.03 s | 1.04 s | 7.5 × 10⁻⁸ | −8.5 × 10⁻¹¹ |
| bostrom | uniform | 1.61 s | 6.23 s (130 ms/step) | 2.00 s | 0.87 s | 7.6 × 10⁻⁸ | −2.9 × 10⁻¹¹ |

Every run: bit-identical across two runs, across 1/4/12 threads, and with the
input list reversed (and, in the tests, rotated and with an axon split in two).
The blessed conformance vectors computed in a release build pass in the debug
build. No zero entry; bostrom's smallest φ is 1.7 × 10⁹ units of 2⁻⁶⁰
(1.5 × 10⁻⁹).

Convergence from uniform, bostrom power-law, ℓ1 to the 192-step reference:

| steps | 8 | 16 | 24 | 32 | 40 | **48** | 56 | 64 |
|---|---|---|---|---|---|---|---|---|
| ℓ1 | 1.5e-2 | 8.9e-4 | 7.7e-5 | 7.5e-6 | 7.4e-7 | **7.5e-8** | 7.6e-9 | 8.0e-10 |

Observed rate ≈ 0.75 per step against the a-priori κ = 0.775; pussy and
uniform stake match within a factor 1.3. `PHI_STEPS = 48` is pinned on this:
below 10⁻⁷ in ℓ1 on both graphs and stakes, two orders under the ε = 10⁻⁶ the
heat degree is set to. The rounding floor (reference last-step drift) is
4 × 10⁻¹¹.

Memory (bostrom, power-law): `phi(axons)` heap peak 645 MB above its input
(the axon list itself is 217 MB; the returned φ + ids 116 MB); `gradient`
960 MB. Whole-process max RSS of the `phi-only` harness 1.49 GB, `gradient-only`
1.89 GB, both including the harness's 2.9 M positions and their net-weight
pass. The graph's arrays are ≈ 406 MB.

Locality (bostrom, per step, one thread): particles in hashed-id order
450 ms; in first-appearance (chain-height) order 228 ms; breadth-first
relabelling (shipped) 131 ms. The relabelling changes no bit (each output entry
is an exact integer function of its row).

### 2.2 settlement (chain-economy §5.1: solve 1 on A_cur, solves 2 + 3 on A_mid)

Previous boundary: all but the last 10,000 positions; current: all, every 97th
earlier position doubled — 40,307 changed positions.

| bostrom | solve 1 φ*(A_cur) incl. build | solves 2 + 3 gradient(A_mid) | attribution of 40,307 positions | total | gradient change 48 → 96 steps |
|---|---|---|---|---|---|
| power-law, 1 thread | 11.4 s | 22.7 s | 50 ms | 34.2 s | 8.6 × 10⁻⁸ (relative ℓ1, all axons) |
| power-law, 12 threads | 2.7 s | 4.2 s | 39 ms | 7.0 s | — |
| uniform, 1 thread | 8.3 s | 16.5 s | 66 ms | 25.0 s | 9.8 × 10⁻⁸ |
| uniform, 12 threads | 2.7 s | 4.0 s | 41 ms | 6.8 s | — |

The ignored test `bostrom_settlement_within_budget` measured 26.1 s for one
thread. Against `SETTLE_BUDGET` = 900 s that is a 26–35× margin on one
core and ≈ 130× on twelve; pussy settles in 0.11 s. (The single-thread wall
times carry ±25 % noise: a legacy benchmark held one core throughout.)

### 2.3 before / after

| bostrom, one solve | legacy | epoch engine | factor |
|---|---|---|---|
| power-law stake, 1 thread | 27.4 s build + 24.6 s (19 steps) = 52 s | 2.1 s + 6.9 s (48 steps) = 9.0 s | 5.8× (with 2.5× the steps) |
| uniform stake, 1 thread | ≈ 39 h (projected) | 7.8 s | ≈ 18,000× |
| 12 threads | — (single-threaded) | 0.9–1.0 s + build | — |
| resolution | 2⁻³² (Σφ off by 3.4 × 10⁻⁵) | 2⁻⁶⁰ (Σφ off by 10⁻¹⁰) | — |
| step count | derived from a wrong λ₂, order-dependent | fixed 48 | — |
| peak memory | 2.04 GB RSS | 645 MB call heap (1.49 GB process) | — |

## 3. attribution against exact Shapley

`epoch_shapley`: random graphs of 8–15 particles and 16–60 axons (weights
10⁶–10⁷); five players — axons whose weight changes between the snapshots;
`v(S) = J∪(φ*(A_prev + Δ_S)) − J∪(φ*(A_prev))` over the union particle set,
φ* at 256 steps, exact Shapley over all 2⁵ subsets. Error = Σ|est − Shapley| /
Σ|Shapley| per case.

Weight changes on existing axons only (120 cases each):

| change per axon | midpoint (chain rule) median / p90 / max | GL-4 median / p90 / max |
|---|---|---|
| ≤ 5 % | 0.01 % / 0.02 % / 0.17 % | 0.00 % / 0.00 % / 0.04 % |
| ≤ 30 % | 0.21 % / 0.58 % / 5.7 % | 0.05 % / 0.18 % / 1.9 % |
| ≤ 100 % | 2.7 % / 9.1 % / 53 % | 0.76 % / 2.8 % / 33 % |

All kinds of change (new axons, raised, lowered, removed; 200 cases each),
split by structure:

| change | class (cases) | midpoint median / p90 | GL-4 median / p90 |
|---|---|---|---|
| ≈ 5 % | no structural change (132) | 3.3 % / 23 % | 0.14 % / 3.3 % |
| ≈ 5 % | particles appear or vanish (34) | 5.1 % / 23 % | 0.63 % / 5.0 % |
| ≈ 5 % | a particle gains its first / loses its last out-axon (34) | 45 % / 103 % | 44 % / 101 % |
| ≈ 30 % | no structural change (132) | 3.5 % / 22 % | 0.50 % / 4.3 % |
| ≈ 30 % | particles appear or vanish (34) | 6.5 % / 20 % | 1.4 % / 4.4 % |
| ≈ 30 % | first / last out-axon (34) | 34 % / 89 % | 31 % / 89 % |

Reading:

- on smooth changes the gradient form converges to Shapley as the change
  shrinks (third order for the midpoint); GL-4 is 3–4× tighter than the
  midpoint but costs four gradient solves instead of one.
- new particles are continuous (a new particle's focus grows from zero) and
  stay within single-digit percent.
- the **dangling switch is a jump**: a particle with focus but no out-axon
  sends its mass to the teleport prior; its first out-axon, at any positive
  weight, redirects all of it. φ* is discontinuous there, so no gradient form —
  midpoint or GL-4 — sees the jump, and exact Shapley pays it to whoever made
  the switch. The budget is divided by attribution, so nothing is minted
  unattributed; first out-links are under-paid. Closing it needs a continuous
  dangling rule (for example, every particle keeps a teleport share
  `c_p/(out_p + c_p)` with `c_p > 0`) — a change of φ*, so a spec decision
  (rewards.md §16, chain-economy §12).

Completeness on the path (`integrated_gradient_is_complete`): GL-4 sums to
`J∪(cur) − J∪(prev)` within 3 × 10⁻⁸ relative; the midpoint within 0.35 % of
the gross attribution `Σ|a|`.

The gradient against central finite differences
(`gradient_matches_finite_differences`, 4 graphs, ≈ 50 axons): within 0.2 %.

## 4. what the chain gets

```rust
use tru::focusing::epoch::{Axon, EpochParams, Phi, Gradient, phi, gradient, midpoint, net_axons, PHI_STEPS};

let p = EpochParams { threads: 8, ..EpochParams::default() };   // threads never change a bit
let now: Phi = phi(&a_cur, &p);              // solve 1: particles ascending, φ at 2^60, J at 2^60
let digest_input = now.preimage();           // D_PHI ‖ n ‖ φ…, LE — chain hashes with hemera 0.4
let mid = midpoint(&a_prev, &a_cur);         // ⌊(A_prev + A_cur)/2⌋, union, ascending
let g: Gradient = gradient(&mid, &p);        // solves 2 + 3: g.by_axon[k] for mid[k], nats/unit at 2^112
// a(c) = Δs(c)·σ(c)·g.by_axon[k]  (256-bit; integrated::score_of → nats at 2^60)
```

`J` is i128 at 2⁶⁰; the epoch record's `syntropy: i64` cannot hold it at that
scale (J ≤ ln n ≈ 15 nats needs 64 bits plus sign) — the record should carry
i128, or `J >> 8` (2⁵², ±2,048 nats) as an i64.

## 5. open

- the dangling switch (§3) — spec decision.
- the legacy uniform-stake solve was not run to completion (projected from
  the measured sub-application count and per-pass cost).
- cross-architecture determinism is checked by CI (ubuntu x86-64 runs the
  blessed vectors); the deimos/jupiter 24 h gate is the chain's.

## 6. amendment 2026-10-17 — the ground weight and the attribution nodes

Coordinator order: fix the dangling switch in φ*, then choose the ground and
the quadrature by measurement. Acceptance: median attribution error against
exact Shapley ≤ 5 % in every class of change including first/last out-axon,
≤ 1 % for weight changes ≤ 30 %; settlement on bostrom ≤ 300 s on one thread;
`PHI_STEPS` still reaching ℓ1 ≤ 10⁻⁷; the determinism suite unchanged.

Rule: every out-row carries a teleporting ground weight
`w0_p = ⌊(γ_d·d_p + γ_i·in_p)/2^32⌋`, normalised by `N_p = out_p + w0_p`
(spec [epoch.md](../specs/epoch.md) §2, §4; the adjoint and the gradient
carry the matching terms, checked against central finite differences within
0.2 % at γ_d = 1/8, γ_i = 1/4). With γ = 0 the engine reproduces §3's numbers
exactly.

### 6.1 the sweep (`epoch_shapley`, 5 players, 400 cases per scale; error = Σ|est − Shapley| / Σ|Shapley|)

At ≈ 30 % changes, 200 cases, median: ground on degree (γ_d) vs in-strength
(γ_i):

| ground | dangling switch: midpoint / GL-2 / GL-4 | new particle: midpoint / GL-4 | smooth: midpoint / GL-4 |
|---|---|---|---|
| none | 34.4 / 29.9 / 30.8 % | 6.5 / 1.4 % | 3.5 / 0.50 % |
| γ_d 5 % | 20.2 / 7.7 / 5.7 % | 6.4 / 1.3 % | 2.9 / 0.47 % |
| γ_d 10 % | 17.3 / 5.3 / 3.2 % | 6.2 / 1.2 % | 2.7 / 0.42 % |
| γ_d 25 % | 11.1 / 2.9 / 1.9 % | 6.1 / 1.1 % | 2.0 / 0.29 % |
| γ_d 50 % | 6.0 / 1.5 / 1.4 % | 6.1 / 0.9 % | 1.5 / 0.25 % |
| γ_i 5 % | 19.0 / 8.4 / 5.5 % | 6.2 / 1.4 % | 3.0 / 0.47 % |
| γ_i 10 % | 17.0 / 5.8 / 3.4 % | 6.2 / 1.2 % | 2.6 / 0.39 % |
| γ_i 25 % | 10.6 / 2.8 / 2.1 % | 6.0 / 1.0 % | 1.9 / 0.29 % |
| γ_i 50 % | 6.2 / 1.7 / 1.2 % | 6.1 / 1.0 % | 1.5 / 0.25 % |
| γ_i 100 % | 4.5 / 1.1 / 1.1 % | 6.0 / 0.9 % | 1.1 / 0.21 % |

γ_i (in-strength) does as well as γ_d at the same γ and moves less focus (a
particle without in-weight is untouched; at in = out, γ_i teleports γ/(1+γ)
against γ_d's 2γ/(1+2γ)). The midpoint stays at ≈ 6 % on new particles at
every ground — a new particle's `φ ln φ` grows from zero — so one node cannot
meet the acceptance.

γ_i at 400 cases, all classes, median (p90) in %:

| γ_i | change | class | midpoint | GL-2 | GL-3 | GL-4 |
|---|---|---|---|---|---|---|
| 1/10 | ≈ 5 % | first/last out-axon | 10.5 (71) | 4.1 (28) | 2.6 (17) | 1.7 (12) |
| 1/10 | ≈ 30 % | first/last out-axon | 11.5 (51) | 5.0 (20) | 3.3 (15) | 2.7 (14) |
| 1/10 | ≈ 100 % | first/last out-axon | 17.1 (42) | 7.1 (22) | 5.2 (23) | 4.7 (18) |
| 1/4 | ≈ 5 % | first/last out-axon | 7.6 (37) | 1.9 (8.8) | 0.87 (6.6) | **0.65** (5.8) |
| 1/4 | ≈ 30 % | first/last out-axon | 7.6 (31) | 2.7 (9.9) | 1.6 (6.8) | **1.49** (6.1) |
| 1/4 | ≈ 100 % | first/last out-axon | 11.8 (33) | 3.7 (12) | 3.2 (11) | **3.03** (9.5) |
| 1/4 | ≈ 5 % | new particle | 4.4 (14) | 1.2 (3.9) | 0.68 (2.4) | **0.49** (2.4) |
| 1/4 | ≈ 30 % | new particle | 6.0 (12) | 2.2 (4.2) | 1.3 (2.8) | **1.00** (2.7) |
| 1/4 | ≈ 100 % | new particle | 9.9 (19) | 3.5 (6.8) | 2.2 (4.9) | **1.77** (4.5) |
| 1/4 | ≈ 5 % | no structural change | 2.3 (16) | 0.14 (2.4) | 0.11 (1.8) | **0.10** (1.8) |
| 1/4 | ≈ 30 % | no structural change | 2.2 (14) | 0.38 (3.3) | 0.35 (3.2) | **0.35** (3.2) |
| 1/4 | ≈ 100 % | no structural change | 3.9 (16) | 1.3 (7.0) | 1.3 (6.6) | **1.26** (6.9) |
| 1/4 | ≤ 5 %, weights only | — | 0.00 | 0.00 | 0.00 | **0.00** |
| 1/4 | ≤ 30 %, weights only | — | 0.18 (0.50) | 0.04 | 0.04 | **0.04** (0.16) |
| 1/4 | ≤ 100 %, weights only | — | 2.3 (7.3) | 0.61 | 0.57 | **0.60** (2.2) |

(γ_i = 15 % and 20 % fall between; full logs reproduce with
`epoch_shapley <cases> 5 <scale> 0 <γ_i %>`, `TRU_MODIFY_ONLY=1` for the
weights-only rows.)

Choice: **γ_i = 1/4, four nodes.** It is the widest margin measured in every
row (worst median 3.03 %, the first/last out-axon at ≈ 100 % changes), a
dyadic constant, and four nodes cost 75 s of one core on bostrom (§6.3), a
quarter of the 300 s bar. Two nodes at 1/4 pass too, with less margin (3.7 %);
γ_i = 1/10 passes only at four nodes, 0.3 points under the bar.

The test `path_gradient_tracks_exact_shapley` pins the protocol rule on
constructed first/last-out-axon cases: median 0.30 % / 0.30 % (0.07 % on
±30 % weight changes), against 21.7 % for the same cases without the ground.
`focus_is_continuous_at_a_first_out_axon`: an ε-weight first out-axon moves φ*
by O(ε) with the ground, by more than 10⁻² without it.

### 6.2 what the ground does to φ*

ℓ1 between φ* with and without the ground, and the overlap of the 1,000
highest-focus particles:

| graph | γ_i 1/8 | 3/16 | **1/4** | 1/2 |
|---|---|---|---|---|
| pussy, power-law | 0.166 (904) | 0.191 (889) | **0.210 (877)** | 0.257 (852) |
| bostrom, power-law | 0.180 (847) | 0.191 (841) | **0.199 (838)** | 0.217 (826) |
| bostrom, uniform | 0.141 (809) | 0.157 (794) | **0.169 (784)** | 0.201 (774) |

Most of the shift is already there at 1/8: it is the cost of having a ground
at all (in-heavy particles stop hoarding focus), and rises slowly with γ.
J falls from 3.831 to 3.734 nats on bostrom (power-law).

### 6.3 bostrom, re-measured (`EpochParams::default()`: ground 1/4, 4 nodes)

| | before (§2) | with the ground |
|---|---|---|
| φ*, 48 steps, 1 thread | 6.2–6.9 s | 6.0–6.3 s |
| φ*, 12 threads | 0.87–1.04 s | 0.83–1.08 s |
| ℓ1 to the 192-step reference after 48 steps | 7.5 × 10⁻⁸ | 0.9–1.3 × 10⁻¹⁰ (rounding floor) |
| settlement (φ*(A_cur) + path gradient + 40,307 positions), 1 thread | 25–34 s (1 node) | 75 s (4 nodes) |
| same, 12 threads | 6.8–7.0 s | 17.2–19.5 s |
| path-gradient change 48 → 96 steps | 8.6 × 10⁻⁸ | 2.4 × 10⁻¹⁰ |
| call heap peak: φ / gradient / 4-node path gradient | 645 / 960 / — MB | 677 / 1,019 / 1,323 MB |

Bit-identical across 1/4/12 threads and with the input reversed; the debug
and release builds agree on the re-blessed vectors (φ digest
`e0ff7999…8fc8` under hemera 0.4).
