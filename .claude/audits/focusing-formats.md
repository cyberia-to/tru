# audit — focusing engine + format layer (M0–M2)

scope: `rs/arithmetic.rs`, `rs/focusing/`, `rs/vocab.rs`, `rs/model/`, `cli/`.
method: the 12 quality passes (`cyber/quality`). one high-severity finding, fixed; the rest clean or noted.

## findings

| # | pass | severity | finding | status |
|---|------|----------|---------|--------|
| 1 | 3 arithmetic | HIGH | `FocusingGraph::build` used `Fx::from_int(amount)`, which computes `amount << 32`. For stakes above `2^31` (realistic u64 token amounts) this exceeds the Goldilocks prime and wraps — silent wrong φ*, no crash. | FIXED — stakes normalized by max via `Fx::ratio_u128` (exact u128 ratio → fixed-point); weights land in (0,1]. Scale-invariance test: 10^15-scale stakes give bit-identical φ* to the proportional small graph. |
| 2 | 6 error handling | none | every `.unwrap()`/`try_into().unwrap()` in library code is either in `#[cfg(test)]` or on a length-guarded fixed-size slice (record decode, vocab/model parse bounds checked first). No unguarded panics in lib paths. | ok |
| 3 | 1 determinism | none | no float on any deterministic path (`to_f64` is display/offline only); field addition is associative so HashMap iteration order in the build cannot change φ*; step count `T(ε)` is fixed. Determinism proven by `deterministic_bit_identical` + `large_stakes_are_scale_invariant`. | ok |
| 4 | 5 type safety | none | `Fx` and `Encoding` are newtypes; `.model` tensor encoding is a closed enum; no invalid-state construction. | ok |
| 5 | 7 adversarial | low | vocab/model parsers bounds-check section sizes and per-entry lengths; frontmatter `index_sections` rejects sections past EOF. A `.model` `weights` `size` not a multiple of the encoding width would silently drop a trailing partial value (`chunks_exact`). | noted — validate `size % width == 0` when tensor content is real (M5) |
| 6 | 11 performance | low | `FocusingGraph::build` runs ~180 power-iteration matvecs (λ_max + λ₂) unconditionally, even when the caller passes an explicit step count. Wasteful for the benchmark's per-neuron ego graphs. | noted — lazy/cache λ if it shows up in profiling; correctness unaffected |
| 7 | 10 compactness / pass file-size | none | all source files < 500 lines; the `.cyb` section indexer was de-duplicated into `frontmatter::index_sections` (graph + vocab + model share it). | ok |

## verification

35 lib tests + 1 smoke, zero warnings. CLI exercised end-to-end on generated fixtures (`gen_demo`): `inspect`/`focus`/`vocab`/`model` all produce correct output; `focus` on a 5-edge demo graph reports cyberank, J, H, κ, λ₂, T(ε) consistent with the engine.
