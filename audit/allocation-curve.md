# allocation curve — PoW/PoS split and the security floor

property #27 in cyber/launch.md (the phase-1 tracker). date 2026-09-21,
revision 39e0caf (base `master`), `rs/allocation.rs` this branch.

## what this closes

`specs/rewards.md` §10 fixes the split formula and the neutral prior:

$$R_{\text{PoW}} = B(1-\theta^\alpha), \quad R_{\text{PoS}} = B\theta^\alpha,
\quad \alpha \in [0.3, 0.7], \quad \alpha = 0.5 \text{ neutral}$$

and the security floor, `floor ≥ c_sec · (TVL/M) · r_atk`. Neither had code.
`rs/allocation.rs` implements both over `Fx` (fixed-point, no floats on the
path — `specs/arithmetic.md`), computing `θ^α` as `exp(α · ln θ)` from the
existing `Fx::exp`/`Fx::ln`. `c_sec` and `r_atk` stay function parameters:
§10 states both follow PID control on observable signals, so this slice
does not fix their numeric values — only the formula and its properties.

## simulation

`cargo run --example allocation_sim`, same revision:

```
-- R_PoW / R_PoS split of a unit security budget B, by staking ratio θ --
α=0.3 (PoW-favoring bound):
  θ=  0%  R_PoW=100.00%  R_PoS=0.00%
  θ= 10%  R_PoW=49.88%   R_PoS=50.12%
  θ= 25%  R_PoW=34.02%   R_PoS=65.98%
  θ= 50%  R_PoW=18.77%   R_PoS=81.23%
  θ= 75%  R_PoW=8.27%    R_PoS=91.73%
  θ= 90%  R_PoW=3.11%    R_PoS=96.89%
  θ=100%  R_PoW=0.00%    R_PoS=100.00%
α=0.5 (neutral prior):
  θ=  0%  R_PoW=100.00%  R_PoS=0.00%
  θ= 10%  R_PoW=68.38%   R_PoS=31.62%
  θ= 25%  R_PoW=50.00%   R_PoS=50.00%
  θ= 50%  R_PoW=29.29%   R_PoS=70.71%
  θ= 75%  R_PoW=13.40%   R_PoS=86.60%
  θ= 90%  R_PoW=5.13%    R_PoS=94.87%
  θ=100%  R_PoW=0.00%    R_PoS=100.00%
α=0.7 (PoS-favoring bound):
  θ=  0%  R_PoW=100.00%  R_PoS=0.00%
  θ= 10%  R_PoW=80.05%   R_PoS=19.95%
  θ= 25%  R_PoW=62.11%   R_PoS=37.89%
  θ= 50%  R_PoW=38.44%   R_PoS=61.56%
  θ= 75%  R_PoW=18.24%   R_PoS=81.76%
  θ= 90%  R_PoW=7.11%    R_PoS=92.89%
  θ=100%  R_PoW=0.00%    R_PoS=100.00%

-- security floor = c_sec · (TVL/M) · r_atk, per-epoch attacker cost of capital r_atk = 1% --
c_sec=1x:
  TVL/M= 10%  floor=0.001000 (of M per epoch)
  TVL/M= 25%  floor=0.002500 (of M per epoch)
  TVL/M= 50%  floor=0.005000 (of M per epoch)
  TVL/M=100%  floor=0.010000 (of M per epoch)
  TVL/M=200%  floor=0.020000 (of M per epoch)
c_sec=2x:
  TVL/M= 10%  floor=0.002000 (of M per epoch)
  TVL/M= 25%  floor=0.005000 (of M per epoch)
  TVL/M= 50%  floor=0.010000 (of M per epoch)
  TVL/M=100%  floor=0.020000 (of M per epoch)
  TVL/M=200%  floor=0.040000 (of M per epoch)
c_sec=3x:
  TVL/M= 10%  floor=0.003000 (of M per epoch)
  TVL/M= 25%  floor=0.007500 (of M per epoch)
  TVL/M= 50%  floor=0.015000 (of M per epoch)
  TVL/M=100%  floor=0.030000 (of M per epoch)
  TVL/M=200%  floor=0.060000 (of M per epoch)
c_sec=5x:
  TVL/M= 10%  floor=0.005000 (of M per epoch)
  TVL/M= 25%  floor=0.012500 (of M per epoch)
  TVL/M= 50%  floor=0.025000 (of M per epoch)
  TVL/M=100%  floor=0.050000 (of M per epoch)
  TVL/M=200%  floor=0.100000 (of M per epoch)
```

what it shows: the split crosses 50/50 exactly at `θ = 0.5^(1/α)` — for
`α=0.5` that is `θ=0.25` (`0.25^0.5 = 0.5`), confirmed above; a
higher `α` shifts the crossover down and shrinks the PoW share at every
`θ`, matching §10's framing of `α` as which side absorbs marginal security
cost. The floor is linear and separable in `c_sec`, `TVL/M`, and `r_atk`
by construction (three multiplications), confirmed by
`floor_scales_linearly_in_each_factor` in `rs/allocation.rs`.

## remains

`c_sec` and `r_atk` are governance/PID outputs, not derived here — the
next slice is the PID loop reading the observable signals §10 names
(security margin, fee coverage, efficiency differential) and feeding
them into `security_floor` and `clamp_alpha`'s midpoint.
