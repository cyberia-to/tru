# the annuity — per-epoch accrual of the yield stream

property #28 in cyber/launch.md (the phase-1 tracker). date 2026-09-21,
revision 39e0caf (base `master`), `rs/annuity.rs` this branch.

## what this closes

`specs/rewards.md` §11–12 define the annuity as a continuous integral,
`R_{i→j}(T) = ∫₀ᵀ ω(t)·Δφ*_j(t) dt`, and describe it as self-correcting: a
link later falsified stops drawing it, with no audit reaching backward
(§12). Neither had code. `rs/annuity.rs` discretizes the integral as a
per-epoch Riemann sum (§12: "the present re-scores itself" every epoch,
so a discrete sum over epoch samples is the right shape, not a
closed-form integral): `accrue` returns the total through epoch `T`,
`accrue_running` exposes the running total after each epoch for a caller
that pays out per epoch rather than as one lump sum.

`ω(t)` (current ICBS price × karma) and `Δφ*_j(t)` are computed upstream
by the market and focusing passes; this module takes them as an
`EpochSample` and does not compute them, matching how `allocation.rs`
(#27) leaves `c_sec`/`r_atk` as inputs rather than re-deriving them.

## verified properties

- an empty history accrues nothing.
- a single epoch's contribution is exactly `ω·Δφ*` for growth (the
  integrand). the integrand is read as the directed impulse of §2,
  `ω·[Δφ*]₊`: an epoch in which the target's focus fell draws zero, never
  a negative amount — a paid epoch is final (§12), so a claw-back through
  a negative term would be the backward re-grading the spec forbids.
  (review 2026-09-22: the first cut multiplied the raw signed `Δφ*`, under
  which the "never drops" claim below was false for negative growth.)
- accrual sums correctly across epochs and is commutative for the final
  total, but the intermediate running totals are not — a payer must feed
  samples in epoch order, matching "the present re-scores itself" rather
  than a global re-sort.
- a falsified link (`ω → 0`) stops drawing from that epoch on, and the
  running total never drops — through zero-growth and negative-growth
  epochs alike; past accrual is never reversed, per §12.

## remains

this is the accrual primitive only. "per-epoch re-scoring on real graph"
(the registry's stated evidence bar) needs `ω(t)` and `Δφ*_j(t)` wired to
the actual ICBS price and focusing output over pussy-rc or bostrom
genesis data — unclaimed. the discovery leak (§12, §5) is unrelated and
stays an accepted open frontier for phase 1 per the registry.
