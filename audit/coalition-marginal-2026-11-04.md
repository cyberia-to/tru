---
title: coalition marginal — the ticket sample on the epoch engine at pussy's scale
tags: tru, audit, focusing, chain, settlement-mining
date: 2026-11-04
---

# coalition marginal — measured 2026-11-04

Receipt for tru `feat/coalition-marginal` (unit P15a of the director's plan,
D20, soft3#680; the gap named in cybernode
`audit/chain/settlement-mining-state-2026-11-02.md` §2 "sample", §5 and §6
item 2: foculus computes a ticket's marginal with the legacy kernel
`tru::impulse`, a different function from the integer epoch engine the chain
settles by). Code: `rs/focusing/epoch/coalition.rs` at tru `d0a54b1`;
tests `rs/tests/coalition_marginal.rs`; harness
`rs/examples/coalition_bench.rs`. Spec: `specs/epoch.md` §8, rewards.md §4,
§7; soft3 `specs/chain-economy.md` §2.3 (beacon), §7.3 (ticket rule set).

## 1. what was built

`Game::new(prev, players, params)` takes the epoch's canonical input — the
axon list `A_prev` (chain-economy §4.2 net weights) and one `Change` per
position or link (`from, to, delta: i128`, a position's `Δs·σ`) — and
solves the base value once. Then:

| call | meaning | cost |
|---|---|---|
| `value(S)` | `v(S) = J_U(φ*(A_S)) − J_U(φ*(A_prev))`, `A_S = max(0, A_prev + Σ_S Δ)` | one build + one `PHI_STEPS` solve |
| `marginal(S, ℓ)` | `m(ℓ \| S) = v(S ∪ {ℓ}) − v(S)`, the signed `ΔJ` marginal | two solves (one when `S = ∅`) |
| `ordering(beacon, cluster, n, k)` | `π(n)`: Fisher–Yates on the hemera XOF of `D_ORDER ‖ beacon ‖ cluster ‖ n` | one hash |
| `sample(beacon, cluster, n)` | `m(n)`: every player's marginal along `π(n)`; `Σ m = v(N)` exactly | `\|N\|` solves |
| `samples(beacon, cluster, nonces, workers)` | the miner's loop, one sample per nonce, nonces in parallel | — |
| `shapley()` | the exact Shapley value by subset enumeration (`\|N\| ≤ 20`) | `2^\|N\|` solves |
| `Sample::preimage()` | `D_SAMPLE ‖ k ‖ m[0] … m[k−1]` (i128 LE), the `commit(m(n))` preimage | — |

`φ*` is `epoch::solve` on `EpochGraph::build_with(A_S, ground)` — the same
function, the same bits, as `epoch::phi` of that axon list; `J_U` is the
engine's syntropy with the epoch's particle count `|U|` held fixed, so a
change that adds a particle is valued on the same scale as one that moves
weight. Every quantity is an integer at the engine's scales
(`specs/arithmetic.md`; `specs/epoch.md` §3). The beacon is
`block_id(boundary(E))` (chain-economy §2.3), the cluster id and the tag
`D_ORDER = "soft3/chain/order/v1\0"` are the ticket rule set's (§7.3 leaves
the tags to it; the values here are proposed in the style of `D_PHI`).

## 2. tests (`cargo test -p cyber-tru --release --test coalition_marginal`)

12 tests, 0.4 s on galapot, 0.89 s on build-a. The whole crate: 168 + 7 +
16 + 1 + 1 passed, 3 ignored (chain-scale budgets).

| test | checks | measured |
|---|---|---|
| `one_change_marginal_matches_the_engine_attribution` | `marginal(∅, ℓ)` against `integrated_gradient(prev, cur, 4)` for a one-change epoch (six graphs of 14 particles, 256 steps), per kind of change | relative error: raise of an axon by 30 % 1.9 × 10⁻⁷ (bound 10⁻⁵), lowering by 30 % 1.4 × 10⁻⁷ (bound 10⁻⁴; the reviewer's harness 3 × 10⁻⁶), an axon to a new particle 1.4 % (bound 3 %; epoch.md §7 measures 0.49–1.77 % on that class) — the quadrature's error, not a bit mismatch |
| `sample_sums_to_the_coalition_value_exactly` | `Σ m(n) = v(N)` and `m[i] = marginal(prefix, i)` for 3 graphs × 4 nonces, 7 players of every kind (raised, lowered, new axon, new particle) | equal as integers |
| `sample_is_bit_identical_across_runs_and_thread_counts` | two runs, 1 and 4 solve threads; hemera digest of the preimage pinned | `ba4bd05f…f17b` on galapot (arm64) and build-a (x86-64) |
| `ordering_is_a_permutation_drawn_from_the_beacon` | a permutation; changes with nonce, beacon and cluster; `n ≤ 1` trivial | — |
| `permutation_mean_equals_shapley_by_enumeration_up_to_six_players` | the rounded mean of `sample_along` over all `n!` orderings equals `shapley()` for `n = 3, 4, 5, 6` | equal as integers |
| `beacon_samples_converge_to_shapley_for_eight_players` | 800 beacon-drawn samples, 8 players, against exact Shapley (256 coalitions) | `Σ\|mean − Shapley\| / Σ\|Shapley\| = 8.2 × 10⁻⁴`; bound 10⁻² |
| `coalition_axons_sum_deltas_and_clip_at_zero` | `A_S`: deltas sum per axon, clip at 0, a repeated index counts once, `v(∅) = 0` | — |

The first row is the one-link identity the unit asked for: the game's
marginal is the exact `ΔJ`; the engine's attribution is its four-node
Aumann–Shapley integral; they agree to the quadrature's error of epoch.md
§6–§7, not bit for bit (a gradient integral is not a difference of two
solves). The error is a function of the kind of change: smooth weight
changes agree to ≈ 10⁻⁷, an axon to a new particle to ≈ 1 % (a new
particle's `φ ln φ` grows from zero; the class §7 chose the ground weight
and the four nodes against).

## 3. pussy's scale

Data: the genesis export of pussy-rc (`soft3 genesis build --chain pussy-rc
… --export`, 2026-11-02, the frames of `/var/lib/soft3/pussy-rc/genesis.tade`
sha256 `d20bab4b…76f5`), read from edge-b
`/srv/edge/data/explorer/genesis/{links,balances}.tsv` (sha256
`78caedb0388c0ff6366db151b349ef167d96171f210982f93e174c7d10761ed3` and
`241828dd38e63cde74ba6d2e1e5deb5a6dbb62aee18291b8ef9594da3d636661`):
29,113 link frames
→ 26,805 axons over 16,273 particles, weighted by the chain's rule
`g(f) = max(1, ⌊B(author)/L(author)⌋)` (chain-economy §4.1), `Σ G =
4.18 × 10¹⁶` (shift 0). Base `J = 3.605474` nats. Players: 64 (or 256)
deterministic changes — three in four raise an existing axon by 30 %, one
in four adds a new axon between existing particles. Beacon `0x42…`, cluster
`0x01…`, nonces from 0.

Machines: build-a (Akash `provider.zencloud.eu`, 2 × Intel Xeon Platinum
8168 @ 2.7 GHz, 96 hardware threads on 48 cores, 2 NUMA nodes, 376 GB;
Ubuntu 24.04, rustc 1.99.1) — built in `/srv/build/scratch/coalition-marginal/`
from the branch's tree with the sibling checkouts hemera `b58ae54`, strata
`5ef0c84`, honeycrisp (acpu, a macOS-only dev-dependency, manifest only);
galapot (Apple M4 Max, 12 P + 4 E, rustc 1.99.1) for comparison.

```
cargo build -p cyber-tru --release --example coalition_bench
./target/release/examples/coalition_bench DIR <players> <solve_threads> <workers> <nonces>
```

### 3.1 one marginal and one sample

| machine | solve threads | `Game::new` (build + base solve) | one marginal, warm (prefix known: one solve) | one marginal, cold (two solves) | one sample, 64 players | per marginal |
|---|---|---|---|---|---|---|
| build-a | 1 | 153–170 ms | 127–133 ms | 254 ms | 8.15–8.33 s | 127–130 ms |
| build-a | 32 | 706 ms | 676 ms | 1.30 s | 40.9 s | 639 ms |
| galapot | 1 | 38 ms | 35 ms | 70 ms | 2.16–2.26 s | 34–35 ms |
| galapot | 12 | — | 61 ms | 92 ms | 3.77 s | 59 ms |

A sample of 256 players on build-a, one thread: 32.3 s (126 ms per
marginal) — linear in the players, as the construction says (one solve per
prefix).

Per-solve threads are a loss at pussy's size: the kernel spawns a scoped
thread per row chunk per pass (≈ 12 passes × 48 steps), and at 16 k
particles the spawn costs more than the rows; the epoch audit saw the same
(`epoch-settlement-2026-10.md` §2.1: 12 threads 53 ms against 29 ms on
one). The right parallelism for tickets is across nonces.

### 3.2 samples per second (64 players; one solve thread per worker)

| machine | workers | nonces | wall | samples/s | marginals/s | scaling |
|---|---|---|---|---|---|---|
| build-a | 1 | 4 | 34.1 s | 0.117 | 7.5 | 1 |
| build-a | 32 | 64 | 40.3 s | 1.59 | 101.7 | 13.6× |
| build-a | 96 | 192 | 184 s | 1.04 | 66.6 | 8.9× |
| galapot | 8 | 8 | 2.26 s | 3.54 | 227 | 7.9× (of 8) |
| galapot | 12 | 24 | 4.74 s | 5.06 | 324 | 11.3× (of 12) |

Every sample of every run summed to `v(N)` exactly (`k/k sum to v(N)` in the
harness), and the sample of nonce 0 has the same hemera digest on both
machines and at every thread count: `9c3e34db…abcf` (64 players),
`64085e31…b275` (256 players).

Reading. On build-a one marginal costs 127 ms and a 64-player sample 8 s on
one core; 32 workers give 1.6 samples/s (102 marginals/s), 96 workers less
(hyper-threads and two NUMA nodes sharing the graph's ≈ 2 MB arrays; the
host also reports a 38 % clock scaling). The M4 Max is 3.7× faster per core
(35 ms per marginal) and scales nearly linearly to its 12 P-cores
(324 marginals/s). The cybernode audit's estimate of ≈ 0.07 s per marginal
at pussy's size was of this order; the measured value on the chain's own
engine is 35 ms (M4 Max) to 127 ms (a shared Xeon core). A whole pussy
epoch with 64 changed positions is one sample of 8 s on build-a's core,
against the 3,600 s settlement window; the exact recompute of the epoch
(four-node path gradient) is 0.12 s, so one ticket costs ≈ 70 recomputes —
the lottery pays for sampling what every node can recompute until the graph
outgrows the recompute (chain-economy §7.3), which is the trade the
cybernode audit §5 named.

What a faster sample would take (not done here): the build (≈ 20 ms of the
127 on build-a) can be kept across the prefixes of one ordering by updating
the weights in place — bit-identical only if the start vector and the
particle count follow the coalition's own graph; and a marginal could reuse
the prefix's φ* as a warm start, which the consensus definition forbids
(every solve is cold, `PHI_STEPS` steps).

## 4. determinism

Every digest here is hemera 0.3.1's (the `D_PHI` vector's hemera as well);
when `feat/hemera-0.4` lands, the pinned test digest and the pussy digests
move with it, and the pins are re-blessed from the new run.

- the same digest for the 9-player test game on arm64 and x86-64, at 1 and
  4 solve threads (pinned in the test);
- the same digest for the pussy game, 64 and 256 players, on both machines,
  at 1, 12 and 32 solve threads and under 1–96 workers;
- `ordering` is a pure function of `(beacon, cluster, nonce, n)`; the sample
  of a nonce is a pure function of the game and that ordering.

## 5. what foculus must change to use it

`foculus/src/settlement.rs` and `tickets.rs` (master and `release/phase1`):

1. `settlement::value` / `marginals` build `tru::Link`s and call
   `tru::impulse(base, batch, ctx, params).directed` (the legacy
   combinatorial kernel, `Fx` at 2⁻³²); replace with `Game::new(A_prev,
   changes, &EpochParams::default())` over the chain's net-weight axons
   (`epoch::net_axons`) and `game.sample_along(order)` / `game.marginal`;
   a `Contribution` becomes one or more `Change`s with `delta = Δs·σ·ρ`
   (ρ = 1 at genesis, `RHO_LAUNCH`); marginals are `i128` nats at 2^60,
   not `Fx`;
2. `settlement::ordering(n, beacon, nonce)` hashes `beacon ‖ nonce` without
   a tag or the cluster; replace with `epoch::ordering(beacon, cluster,
   nonce, n)` (`D_ORDER`, chain-economy §7.3);
3. `tickets::commit_marginals` hashes `Fx` words under `foculus-marginal-v0`;
   replace with `hemera(Sample::preimage())` (`D_SAMPLE`), so the win test
   `hemera(D_TICKET ‖ beacon ‖ cluster ‖ n ‖ miner ‖ commit)` binds the
   engine's sample;
4. `try_settlement_ticket` / `grind_settlement` take `(base, contribs, ctx,
   params)`; take a `&Game` instead, and grind nonces with `game.samples`
   (one solve thread per worker, §3.1);
5. `shapley_exact` and `ticket_proof.rs`'s relation `m[π_j] = v_j − v_{j−1}`
   keep their shape with `v_j = game.value(prefix)`; the per-ticket proof
   (foculus#61) still has no constraint system for the solve — unchanged by
   this unit;
6. the `directed` clip `[ΔJ]₊` of `tru::impulse` is not applied per marginal
   here: a marginal is the signed `ΔJ` (a position that lowers syntropy
   carries a negative marginal, as the chain's attribution `a(c)` does,
   chain-economy §5.2) and the ticket's sample carries it signed (rewards.md
   §7, `m_j(n)`); the clips apply after the sample — per position,
   `max(0, a(c))` (chain-economy §5.3), and on the total, `min(v*(N),
   Δφ⁺(N))` (rewards.md §4) — in the settlement, not in tru.
