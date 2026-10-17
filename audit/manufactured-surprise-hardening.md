---
tags: tru, audit, launch
crystal-type: report
crystal-domain: cyber
---
# manufactured-surprise crowd-reference hardening

date: 2026-09-23 · revision: 8941629 (origin/master at worktree creation) ·
property: registry row 24 · command: `RUSTC_BOOTSTRAP=1 cargo test -p cyber-tru --lib truth_scoring`

`specs/rewards.md` §5 requires the crowd reference `m̄_−ν` to be "taken over
distinct karma-bearing predictors who have themselves staked BTS exposure on
the cluster ... with v_ℓ=0 reports excluded," and names the reason: without
that filter "a stake-rich actor manufactures surprise by seeding unslashable
v_ℓ=0 decoy predictions that depress the reference." §15 lists this as
"manufactured surprise ... blunted, not closed."

`rs/truth_scoring.rs`'s `bts_scores` (and its `geo_mean` crowd reference) took
every other report unconditionally — no staked-exposure filter existed in
code, so the attack §5 describes was unmitigated regardless of the spec text.

This pass adds `bts_scores_hardened(reports, staked)`, an additive sibling of
`bts_scores` that excludes `staked[k] == false` reports from the crowd
reference `others_b`/`others_m` while still scoring every report on the
left-hand side. `Report` and `bts_scores` are unchanged — `foculus/src/
rewards.rs::contributions_with_rho` and every other external caller keep
building against the existing signatures.

## measured

`decoy_reports_depress_the_plain_crowd_reference_but_not_the_hardened_one`
(`rs/truth_scoring.rs`):

- baseline: an informed contrarian (belief 0.15, accurate meta-prediction 0.8)
  against 3 genuine followers (belief/prediction 0.8) scores `s ≈
  -2.3e-10` under plain `bts_scores` — effectively zero, since its
  meta-prediction (0.8) matches the crowd exactly and its own belief matches
  the crowd's leave-one-out belief reference too (all three followers report
  0.8/0.8, so `p̄_−ν = 0.8`, cancelling most of the KL terms; the residual is
  floating/fixed-point noise).
- attacked: 3 unstaked decoys added, each reporting belief/prediction 0.15
  (the contrarian's own belief) to pull the crowd reference toward it — plain
  `bts_scores` on the 7-report set scores the same contrarian at `s ≈
  -0.287`, confirming the attack drives a genuinely neutral score sharply
  negative under the unfiltered scorer.
- hardened: `bts_scores_hardened` on the identical 7-report set with the 3
  decoys marked `staked = false` scores the contrarian at `s ≈ -2.3e-10`,
  bit-for-bit equal to the undepressed baseline (the filtered `others_b`/
  `others_m` slices are computed over the same 3 genuine indices either way)
  — the decoys contribute nothing to the crowd reference.

```
$ RUSTC_BOOTSTRAP=1 cargo test -p cyber-tru --lib truth_scoring
running 7 tests
test truth_scoring::tests::surprise_clips_to_unit_interval ... ok
test truth_scoring::tests::karma_accumulates_up_on_signal_and_floors_at_zero ... ok
test truth_scoring::tests::surprisingly_popular_report_beats_a_follower ... ok
test truth_scoring::tests::accumulate_closes_the_loop_to_a_karma_table ... ok
test truth_scoring::tests::accurate_meta_prediction_beats_an_inaccurate_one ... ok
test truth_scoring::tests::consensus_copier_scores_near_zero_information ... ok
test truth_scoring::tests::decoy_reports_depress_the_plain_crowd_reference_but_not_the_hardened_one ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 79 filtered out
```

## remains

`bts_scores_hardened` takes the staked mask as an argument; nothing yet
supplies a real one. The caller (`foculus::rewards::contributions_with_rho`,
or a successor) needs to source `staked[i]` from each report's actual ICBS
position size (`v_ℓ ≠ 0`) and switch from `bts_scores` to
`bts_scores_hardened` — a foculus-side wiring slice, out of scope here. The
coordinated-ring case §15 also names ("a coordinated ring still can [depress
the reference]") stays open: this hardening defeats a single stake-rich actor
seeding unslashable decoys, not a cartel of genuinely staked, colluding
reporters.
