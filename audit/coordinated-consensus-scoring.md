---
tags: tru, audit, cybernomics
crystal-type: report
crystal-domain: cyber
---
# coordinated-consensus scoring at scale

date: 2026-09-19
revision: 39e0caf (base), plus `informed_minority_beats_coordinated_majority_at_scale` in `rs/truth_scoring.rs`
property: launch #24 — [[cyber/launch]] registry — "the serum is strictly proper in the meta-report and selects truth over coordinated consensus"

## claim under test

[[strong truthfulness]] requires the serum to select truth by score, not by
vote count: a coordinated majority repeating the same fixed point offers no
private signal and must not out-earn an informed minority, at any scale.

## command

```
cargo test -p cyber-tru --lib truth_scoring::tests::informed_minority_beats_coordinated_majority_at_scale
```

## result

```
running 1 test
test truth_scoring::tests::informed_minority_beats_coordinated_majority_at_scale ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## measurement

one informed contrarian (belief 0.15, accurate meta-prediction 0.8) against
a coordinated majority at four sizes, all reporting and predicting the fixed
point 0.8:

| majority size | contrarian beats every follower |
|---|---|
| 4 | yes |
| 20 | yes |
| 100 | yes |
| 200 | yes |

the contrarian's score margin over every follower holds at every scale
tested; the coordinated majority is never rewarded for its count. this is
evidence at the [[truth-scoring]] layer only — [[BTS]] leave-one-out scores
over synthetic reports. it does not exercise the on-chain reward path, the
[[ICBS]] market coupling, or a real adversarial network; those remain open
per the registry's implementation note.

## remains

- wire the score into the actual mint gate (surprise ρ) end to end
- test against the "coordinated inversion" case (agents report against their
  own signal, not merely coordinate on a focal point) — reduces to the
  honest-majority-by-stake condition, not yet exercised here
- adversarial network test with real message delay and partial information
