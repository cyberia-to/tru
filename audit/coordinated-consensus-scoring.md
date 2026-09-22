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
tested; the coordinated majority is never rewarded for its count.

## measurement, read honestly (review 2026-09-22)

the lone-contrarian margin is real but thin, and it is the followers'
penalty, not the contrarian's reward. against a majority that predicts
itself perfectly the crowd's actual and predicted means coincide, so the
contrarian's information gain is exactly zero: its score is `0` to within
one fixed-point ULP (`raw = 2`, i.e. `4.7e-10`) at every size. it wins
because its presence pulls the leave-one-out mean off the fixed point and
every follower pays `−2·KL` for that, a term that shrinks like `1/n²`:

| majority size | contrarian score | follower score | margin |
|---|---|---|---|
| 4 | 0 | −1.19e-1 | 1.19e-1 |
| 20 | 0 | −4.08e-3 | 4.08e-3 |
| 100 | 0 | −1.57e-4 | 1.57e-4 |
| 200 | 0 | −3.91e-5 | 3.91e-5 |

under the mint gate `ρ = clip(s / s_max)` a score of zero mints nothing,
so this case shows the majority cannot out-earn the contrarian, not that
the contrarian earns. the fixed-share case is what [[strong truthfulness]]
actually needs, and it is the second test,
`informed_minority_share_keeps_positive_margin_at_scale`: an informed
minority holding 5% of the reports, sharing belief 0.15 and predicting the
crowd accurately (the geometric mean of everyone else's belief), against a
majority reporting and predicting 0.8. every informed neuron scores
strictly positive, every follower strictly negative, and the informed
margin does not collapse as the population grows tenfold — asserted in
`Fx`, no float comparison:

| n followers | k informed (5%) | informed score (min) | follower score (max) |
|---|---|---|---|
| 100 | 5 | +7.38e-2 | −3.76e-3 |
| 200 | 10 | +8.21e-2 | −3.72e-3 |
| 1000 | 50 | +8.86e-2 | −3.69e-3 |

```
cargo test -p cyber-tru --lib truth_scoring
test result: ok. 8 passed; 0 failed
```

this is evidence at the [[truth-scoring]] layer only — [[BTS]] leave-one-out
scores over synthetic reports. it does not exercise the on-chain reward
path, the [[ICBS]] market coupling, or a real adversarial network; those
remain open per the registry's implementation note.

## remains

- wire the score into the actual mint gate (surprise ρ) end to end
- test against the "coordinated inversion" case (agents report against their
  own signal, not merely coordinate on a focal point) — reduces to the
  honest-majority-by-stake condition, not yet exercised here
- adversarial network test with real message delay and partial information
