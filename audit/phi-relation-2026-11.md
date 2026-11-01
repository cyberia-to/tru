---
title: the φ* relation, measured first — what proving tru's epoch φ* costs
tags: tru, zheng, audit, focusing, proving
date: 2026-11-01
---

# the φ* relation — measured 2026-11-01

Receipt for soft3 `release/director/plan.md` row **P7** ("the φ* relation, measured first": tru's epoch engine φ* as a relation zheng proves, at pussy's size, cycles or constraints counted, one prove on Metal; the aggregation decided from the receipt). Measured on the stand's revisions: tru `464104c` (`release/phase1`), zheng `166a14e`, nox `41ba24f`, hemera `8559b52`, lens `d6dcc3f`. Nothing here changes tru or zheng: both harnesses live on diagnostic branches that are never merged.

| what | branch · commit | run | legs |
|---|---|---|---|
| exact operation counts of the epoch kernel | tru `diag/p7-phi-count` `3b8b9fd` (cargo feature `opcount`, off by default; `rs/tests/p7_phi_count.rs`) | [tru 41300179245](https://github.com/cyberia-to/tru/actions/runs/41300179245) | linux-x86_64 ✅, macos-aarch64 ✅ — the `P7` lines are byte-identical on both |
| the cost of its primitives as nox programs under zheng's sound prover | zheng `diag/p7-phi-relation` `84c50c3` (`rs/tests/phi_relation.rs`) | [zheng 41300181812](https://github.com/cyberia-to/zheng/actions/runs/41300181812) | linux-aarch64 CPU ✅ (3/3 tests); macOS Metal ×2: L ≤ 64 proved and verified, L = 512 hit the CI GPU's timeout (below); linux-x86_64 and macOS CPU: see § open |

## 1. how big φ* is — exact counts (tru)

`phi`, `gradient` and `path_gradient` with `EpochParams::default()` (48 forward steps, heat Taylor degree K = 9). Counters are relaxed atomics behind `opcount`. The counts are identical across two runs, at 1 thread and at 4 threads, and on both platforms, and φ's output is identical across the runs.

Graphs: **pussy** is `eval/data/pussy_links.jsonl`: 48,370 lines, 39,818 unique (from, to) pairs, 32,682 particles, one position per pair at uniform stake 10⁶. **synthetic_burial** is deterministic at the burial figures of [epoch-settlement-2026-10](epoch-settlement-2026-10.md): 29,112 links over 16,271 particles. The jsonl is a larger pussy than the burial CSV the October audit used, so both are reported.

| graph · entry | n | nnz sym / dir | steps | sparse entries (sym + dir) | qmul | shr_u | ln_q40 |
|---|---|---|---|---|---|---|---|
| pussy · `phi` | 32,682 | 71,508 / 39,818 | 48 fwd | 30,891,456 + 1,911,264 = **32.80 M** | **17.26 M** | 39.22 M | 32,683 |
| pussy · `gradient` | | | 48 fwd + 48 adj | 63.0 M + 3.86 M | 17.26 M + 17.26 M (`qmul_i`) | 39.48 M | 65,365 |
| pussy · `path_gradient` | | | 192 + 192 | 252.0 M + 15.4 M | 69.0 M + 69.0 M | 157.9 M | 261,460 |
| burial · `phi` | 16,271 | 58,220 / 29,112 | 48 fwd | 25,151,040 + 1,397,376 = **26.55 M** | **8.59 M** | 19.53 M | 16,272 |

Per forward step at pussy: 643,572 sym entries (≈ 9 SpMV over the 71,508-entry symmetric matrix: the heat series), 39,818 directed entries (one SpMV), 359,502 qmul. A φ solve is about 33 M multiply-accumulates and 17 M rounded Q32 products; the gradient doubles it, and the path gradient (Gauss–Legendre, 4 points) is four gradients.

## 2. what one primitive costs under zheng's sound prover (zheng)

The programs are written as nox nouns and proven by the **nox machine** (`zheng::machine`: `execute_hinted`, `prove_run` `rs/src/machine/mod.rs:192`, `verify` `:264`). Not the `legacy` feature, nothing under `phi`/`fold`/`decide`. The machine's relation is "tested, not proven" (zheng `specs/soundness.md:54`). zheng's CCS execution provers were rejected on size: they refuse programs over 4096 nodes (`rs/src/execution/relation.rs:179-181`), and a row of 64 entries already exceeds that.

- **P1 `qmul(a, q)`** = `(a·q + 2³¹) >> 32`, tru `kernel.rs:200`. The prover supplies the limbs `ah` and `c1` through pattern 16 (`call`). Every limb is range-checked by a 32-bit word `and`. The `+1` checks (`ah + 1 < 2³²`, `c1 + 1 < 2³²`) shut out the second representative mod p, so each equation holds over the integers.
- **P2 one sparse row of length L**, exactly `sym_fwd_row` (`kernel.rs:161-167`): `Σ w_k·x_k` in u128, then `(acc + 2³⁹) >> 40`. Each weight is split into 20-bit limbs and each entry into 30-bit limbs, with four accumulators and a carry chain of five range checks. The output is the same integer for every valid witness.
- **Correctness:** both programs' outputs are bit-equal to a port of tru's formula and to native `nox::reduce`. P1 was tested on 84 edge pairs, 4 rounding boundaries and 60 seeded random pairs. P2 was tested on 10 edge rows and random rows of L ∈ {1, 2, 3, 5, 8, 13}.
- **Soundness, tested:** 696 forged P1 witnesses and 1,160 forged P2 witnesses, **0 accepted with a different output**, on every leg that ran the tests. A forged output or cycle count does not verify.

linux-aarch64, CPU, 4-vCPU runner (job 124829313151):

| program | trace rows | segments | machine steps | nox cycles | proof | prove (median) | verify (median) |
|---|---|---|---|---|---|---|---|
| P1 qmul | 512 | 1 | 340 | 195 | 89,980 B | 16.1 s | 9.1 ms |
| P2 L = 1 | 1,024 | 1 | 673 | 362 | 96,936 B | 43.5 s | 10.0 ms |
| P2 L = 8 | 2,048 | 1 | 1,363 | 775 | 109,360 B | 85.3 s | 11.4 ms |
| P2 L = 64 | 16,384 | 1 | 7,399 | 4,079 | 133,044 B | 199.4 s | 16.4 ms |
| P2 L = 512 | 81,920 | 5 | 58,413 | 30,511 | 487,312 B | 639.9 s | 71.9 ms |

Linear fits over L (`P7-fit`): **cycles = 303 + 59·L** (residual 0), **steps ≈ 399 + 113.3·L**, **trace rows ≈ 2,540 + 155.9·L**. At L = 512 that is **160 trace rows per multiply-accumulate entry**. In the measured row, weights and entries are quoted constants; a relation that reads them from committed state pays for that fetch in addition.

**One prove on Metal (macos-15, Apple Paravirtual device), jobs 124829312931 (1 thread) and 124829313121 (multi-thread):** P1 proved in 5.86 s and verified in 8.1 ms. P2 at L = 1, 8 and 64 proved in 14.4 s, 26.3 s and 126.7 s. The rows, steps, cycles and proof bytes are identical to the CPU leg. At L = 512 (81,920 rows) the CI's virtual GPU returned `kIOGPUCommandBufferCallbackErrorTimeout` (lens `gpu/src/metal.rs:846`). That is a limit of the runner's paravirtual device, the class G16a named, and not of the relation. The correctness and forgery tests passed on both Metal legs.

## 3. φ* as a machine relation — the product

| | sparse entries × 160 rows | qmul × 340–512 rows | one φ solve |
|---|---|---|---|
| pussy (32,682 particles) | 5.25 G | 5.9–8.8 G | **≈ 11–14 G rows** |
| burial (16,271 particles) | 4.25 G | 2.9–4.4 G | **≈ 7–9 G rows** |

At the measured 7.8 ms per row (640 s for 81,920 rows on the CI runner), one pussy φ solve is about 2.7 years of that runner. The gradient doubles it. **Proving φ* by the general nox machine is not viable at any graph size the chain will see, and more runners do not change that.** For scale: tru `specs/proving.md` (the owner's draft, tru#29) estimates 47.8 M field constraints per iteration at bostrom's 3.1 M particles. The exact integer kernel at pussy is about 250 M machine rows per step. The difference is the integer rounding and the 32-row range checks, not the graph.

zheng's existing `prove_phi_star` / `verify_phi_star` (`specs/phi-spmv.md`) is under the `legacy` feature (zheng `specs/README.md:47`), which is the unsound path, and it is field arithmetic, not tru's integer kernel. It is a skeleton, not a prover of this φ*.

## 4. the decision — the aggregation

The director's decision, checked with fable (2026-11-01, 2nd question of the day):

**(c) a direct CCS relation for one exact integer kernel step, aggregated by (a) IVC over the 48 steps. Not (b).** Parallel segments of the machine trace (b) already exist: L = 512 ran in 5 segments of 2¹⁴. They spread 11 G rows across machines but cannot shrink them. The cost is per entry, about 160 rows where a custom relation spends a few constraints. The attack order:

1. **Range checks.** Replace the 32-row bit-peel with a lookup (16-bit tables). Range-check each φ entry once per particle and each weight once per edge, not once per use. Only each row's rounded output needs a fresh check.
2. **qmul and the row rounding.** Prove them as witnessed divisions (`a·q + 2³¹ = 2³²·hi + lo`, `lo < 2³²`, `hi` bounded): one constraint and two lookups. P1 already proves this shape, but with 32-row checks.
3. **The 48 steps.** A fixed factor, absorbed by folding.

**What phase 1 must not do: replace the integer kernel with field arithmetic.** A field circuit proves a different function than the one consensus computes. Consensus would then have to run the field version and lose the fixed-point semantics the heat series relies on. The integer function is proven with limbs: φ at 2⁶⁰ times a weight at 2⁴⁰ is about a 100-bit product, and Goldilocks wraps at 64 bits. This is the decomposition §2 measured.

**Phase 1 is unaffected.** At pussy's genesis φ* is "computed by every full node at every boundary" and settled "by recomputation on every node" (soft3 `specs/chain-economy.md` §4, §7.1), not proven. Proving φ* is post-genesis work. Its first unit is the step relation of (c), and zheng's CCS cap of 4096 nodes is the engineering gap it has to close.

## open

- zheng 41300181812's linux-x86_64 and macOS-CPU legs were still running when this receipt was written; they add platforms, not quantities (rows, cycles and bytes are deterministic and equal on the three legs that finished).
- P3 (`mul_shr(a, b, 60)`, a 124-bit product of two runtime operands) is not measured: it needs a three-level carry chain. P1 and P2 price its two ingredients.
