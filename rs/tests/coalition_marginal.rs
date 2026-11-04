//! The coalition marginal and the ticket sample on the epoch engine
//! (`rs/focusing/epoch/coalition.rs`): the one-change marginal against the
//! engine's own attribution, efficiency (the sample sums to the coalition's
//! value, exactly), determinism across runs and thread counts (a pinned
//! digest), the ordering as a beacon function, and the sample against the
//! exact Shapley value by enumeration — identically over all orderings for
//! `n ≤ 6`, by Monte Carlo for `n = 8`.

use tru::focusing::epoch::coalition::WEIGHT_BITS;
use tru::focusing::epoch::fixed::div_round_i;
use tru::focusing::epoch::integrated::integrated_gradient;
use tru::focusing::epoch::{Axon, Change, EpochParams, Game, canonical, ordering};

const ONE: f64 = (1u64 << 60) as f64;

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn pid(i: u64) -> [u8; 32] {
    let mut h = [0u8; 32];
    h[..8].copy_from_slice(&mix(i ^ 0xAB).to_be_bytes());
    h
}

/// A random graph: `n` particles, about `m` axons, weights 10^6..10^7.
fn random_graph(seed: u64, n: u64, m: u64) -> Vec<Axon> {
    let mut r = seed;
    let mut next = || {
        r = mix(r);
        r
    };
    let mut v = Vec::new();
    for _ in 0..m {
        let (a, b) = (next() % n, next() % n);
        if a != b {
            v.push(Axon {
                from: pid(a),
                to: pid(b),
                weight: 1_000_000 + (next() % 9_000_000) as u128,
            });
        }
    }
    canonical(&v)
}

fn params(steps: usize, threads: usize) -> EpochParams {
    EpochParams {
        steps,
        threads,
        ..EpochParams::default()
    }
}

/// `k` players over `base`: raised axons, a lowered one, a new axon between
/// existing particles and an axon to a new particle (every kind of change).
fn players(base: &[Axon], k: usize, seed: u64) -> Vec<Change> {
    let m = base.len();
    (0..k)
        .map(|i| {
            let a = &base[(i * 7 + seed as usize) % m];
            match i % 4 {
                0 | 1 => Change {
                    from: a.from,
                    to: a.to,
                    delta: (a.weight as i128) * 3 / 10,
                },
                2 => Change {
                    from: a.from,
                    to: a.to,
                    delta: -((a.weight as i128) * 3 / 10),
                },
                _ => {
                    let b = &base[(i * 13 + 5) % m];
                    let to = if i % 8 == 3 {
                        pid(900 + i as u64)
                    } else {
                        b.to
                    };
                    Change {
                        from: a.from,
                        to,
                        delta: 2_000_000,
                    }
                }
            }
        })
        .collect()
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut cur: Vec<usize> = (0..n).collect();
    fn go(a: &mut Vec<usize>, k: usize, out: &mut Vec<Vec<usize>>) {
        if k == a.len() {
            out.push(a.clone());
            return;
        }
        for i in k..a.len() {
            a.swap(k, i);
            go(a, k + 1, out);
            a.swap(k, i);
        }
    }
    go(&mut cur, 0, &mut out);
    out
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// ── the marginal against the engine's attribution ─────────────────────────

#[test]
fn one_change_marginal_matches_the_engine_attribution() {
    // A one-change epoch: the chain attributes ΔA·ḡ (four-node path gradient,
    // the Aumann–Shapley limit); the game's marginal against ∅ is the exact
    // ΔJ. They agree to the quadrature's error of epoch.md §6–§7, which
    // depends on the kind of change: a raise or a lowering of an existing
    // axon is smooth (≈ 10⁻⁷ … 10⁻⁶); an axon to a new particle is the
    // class §7 measures at 0.49–1.77 % (its φ ln φ grows from zero).
    let p = params(256, 1);
    for (kind, bound) in [("raise", 1e-5), ("lower", 1e-4), ("new particle", 3e-2)] {
        let mut worst: f64 = 0.0;
        for seed in 0..6u64 {
            let prev = random_graph(300 + seed, 14, 40);
            let a = prev[(seed as usize * 5) % prev.len()];
            let change = match kind {
                "raise" => Change {
                    from: a.from,
                    to: a.to,
                    delta: (a.weight as i128) * 3 / 10,
                },
                "lower" => Change {
                    from: a.from,
                    to: a.to,
                    delta: -((a.weight as i128) * 3 / 10),
                },
                _ => Change {
                    from: a.from,
                    to: pid(900 + seed),
                    delta: (a.weight as i128) * 3 / 10,
                },
            };
            let game = Game::new(&prev, &[change], &p).unwrap();
            let marginal = game.marginal(&[], 0);
            let mut cur = prev.clone();
            if kind == "new particle" {
                cur.push(Axon {
                    from: change.from,
                    to: change.to,
                    weight: change.delta as u128,
                });
            } else {
                for x in cur.iter_mut() {
                    if (x.from, x.to) == (a.from, a.to) {
                        x.weight = (x.weight as i128 + change.delta) as u128;
                    }
                }
            }
            let ig = integrated_gradient(&prev, &cur, 4, &p);
            assert_eq!(ig.len(), 1);
            let (m, s) = (marginal as f64 / ONE, ig[0].score as f64 / ONE);
            let err = (m - s).abs() / s.abs().max(1e-300);
            eprintln!("{kind} seed {seed}: marginal {m:e} attribution {s:e} rel {err:e}");
            worst = worst.max(err);
        }
        // measured: raise 1.9 × 10⁻⁷, lower 1.4 × 10⁻⁷ (the reviewer's harness
        // saw 3 × 10⁻⁶), new particle 1.4 %
        // (audit/coalition-marginal-2026-11-04.md §2)
        eprintln!("{kind}: worst {worst:e}");
        assert!(
            worst < bound,
            "{kind}: one-change marginal vs path gradient {worst:e} ≥ {bound:e}"
        );
    }
}

// ── efficiency ────────────────────────────────────────────────────────────

#[test]
fn sample_sums_to_the_coalition_value_exactly() {
    let p = params(48, 1);
    for seed in 0..3u64 {
        let base = random_graph(400 + seed, 16, 48);
        let pl = players(&base, 7, seed);
        let game = Game::new(&base, &pl, &p).unwrap();
        let all: Vec<usize> = (0..pl.len()).collect();
        let total = game.value(&all);
        for nonce in 0..4u64 {
            let s = game.sample(&[seed as u8; 32], &[7; 32], nonce);
            assert_eq!(s.total, total, "seed {seed} nonce {nonce}: Σm ≠ v(N)");
            assert_eq!(s.marginals.iter().sum::<i128>(), s.total);
            // every entry is the marginal against its prefix
            for (j, &i) in s.order.iter().enumerate() {
                assert_eq!(s.marginals[i], game.marginal(&s.order[..j], i));
            }
        }
    }
}

// ── determinism ───────────────────────────────────────────────────────────

#[test]
fn sample_is_bit_identical_across_runs_and_thread_counts() {
    let base = random_graph(500, 40, 160);
    let pl = players(&base, 9, 1);
    let beacon = [0x5a; 32];
    let cluster = [3; 32];
    let one = Game::new(&base, &pl, &params(48, 1))
        .unwrap()
        .sample(&beacon, &cluster, 11);
    let again = Game::new(&base, &pl, &params(48, 1))
        .unwrap()
        .sample(&beacon, &cluster, 11);
    let four = Game::new(&base, &pl, &params(48, 4))
        .unwrap()
        .sample(&beacon, &cluster, 11);
    assert_eq!(one, again);
    assert_eq!(one, four);
    let digest = cyber_hemera::hash(&one.preimage());
    eprintln!("sample digest {}", digest.to_hex());
    // Pinned on the first run (galapot, M4 Max); build-a (x86-64) reproduces
    // it — audit/coalition-marginal-2026-11-04.md.
    assert_eq!(
        hex(digest.as_bytes()),
        "ba4bd05facf5580bfa6023ef0c5b467145de2f0582e04b731d8d2f5b0cd6f17b",
        "the sample's digest moved: a consensus change"
    );
}

#[test]
fn ordering_is_a_permutation_drawn_from_the_beacon() {
    let (b, c) = ([1u8; 32], [2u8; 32]);
    let o = ordering(&b, &c, 7, 12);
    let mut sorted = o.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..12).collect::<Vec<_>>());
    assert_eq!(o, ordering(&b, &c, 7, 12));
    assert_ne!(o, ordering(&b, &c, 8, 12), "the nonce selects the ordering");
    assert_ne!(o, ordering(&[9; 32], &c, 7, 12), "the beacon selects it");
    assert_ne!(o, ordering(&b, &[9; 32], 7, 12), "the cluster selects it");
    assert_eq!(ordering(&b, &c, 7, 1), vec![0]);
    assert!(ordering(&b, &c, 7, 0).is_empty());
}

// ── the exact Shapley value ───────────────────────────────────────────────

#[test]
fn permutation_mean_equals_shapley_by_enumeration_up_to_six_players() {
    // Over all n! orderings the samples' mean is the Shapley value, and both
    // sides are the same integers: the identity holds exactly.
    let p = params(48, 1);
    for (seed, k) in [(1u64, 3usize), (2, 4), (3, 5), (4, 6)] {
        let base = random_graph(600 + seed, 12, 30);
        let pl = players(&base, k, seed);
        let game = Game::new(&base, &pl, &p).unwrap();
        let sh = game.shapley();
        let mut acc = vec![0i128; k];
        let perms = permutations(k);
        for o in &perms {
            for (i, m) in game.sample_along(o).into_iter().enumerate() {
                acc[i] += m;
            }
        }
        let fact = perms.len() as i128;
        let mean: Vec<i128> = acc.iter().map(|&a| div_round_i(a, fact)).collect();
        assert_eq!(mean, sh, "k = {k}");
        assert!(sh.iter().any(|&x| x != 0), "a degenerate game");
    }
}

#[test]
fn beacon_samples_converge_to_shapley_for_eight_players() {
    let p = params(48, 1);
    let base = random_graph(700, 18, 54);
    let pl = players(&base, 8, 3);
    let game = Game::new(&base, &pl, &p).unwrap();
    let sh = game.shapley();
    let nonces: Vec<u64> = (0..800).collect();
    let samples = game.samples(&[0x77; 32], &[1; 32], &nonces, 4);
    let mut acc = [0i128; 8];
    for s in &samples {
        for (a, m) in acc.iter_mut().zip(&s.marginals) {
            *a += m;
        }
    }
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for (a, &s) in acc.iter().zip(&sh) {
        let mean = *a as f64 / nonces.len() as f64;
        num += (mean - s as f64).abs();
        den += (s as f64).abs();
    }
    let err = num / den;
    eprintln!(
        "8 players, {} samples: Σ|mean − Shapley| / Σ|Shapley| = {err:e}",
        nonces.len()
    );
    // measured 8.2 × 10⁻⁴ (deterministic: the nonces are fixed)
    assert!(err < 1e-2, "Monte Carlo vs exact Shapley: {err:e}");
}

// ── the coalition's graph ─────────────────────────────────────────────────

#[test]
fn coalition_axons_sum_deltas_and_clip_at_zero() {
    let base = vec![
        Axon {
            from: pid(1),
            to: pid(2),
            weight: 100,
        },
        Axon {
            from: pid(2),
            to: pid(3),
            weight: 50,
        },
    ];
    let pl = vec![
        Change {
            from: pid(1),
            to: pid(2),
            delta: 25,
        },
        Change {
            from: pid(1),
            to: pid(2),
            delta: -200,
        },
        Change {
            from: pid(3),
            to: pid(1),
            delta: 7,
        },
    ];
    let game = Game::new(&base, &pl, &params(8, 1)).unwrap();
    assert_eq!(game.particles(), 3);
    let w = |ax: &[Axon], f: u64, t: u64| {
        ax.iter()
            .find(|a| a.from == pid(f) && a.to == pid(t))
            .map(|a| a.weight)
    };
    let a = game.axons(&[0]);
    assert_eq!(w(&a, 1, 2), Some(125));
    let a = game.axons(&[0, 1, 0]);
    assert_eq!(
        w(&a, 1, 2),
        Some(0),
        "clipped at zero, a repeated index counts once"
    );
    assert_eq!(w(&a, 2, 3), Some(50));
    let a = game.axons(&[2]);
    assert_eq!(w(&a, 3, 1), Some(7));
    assert_eq!(game.value(&[]), 0);
    assert_eq!(game.axons(&[]), canonical(&base));
}

#[test]
fn weights_beyond_the_engine_range_are_an_error_not_a_panic() {
    let base = random_graph(800, 8, 20);
    let huge = |i: u64| Change {
        from: pid(i),
        to: pid(i + 1),
        delta: i128::MAX,
    };
    let p = params(8, 1);
    assert!(Game::new(&base, &[huge(1), huge(2), huge(3)], &p).is_err());
    // the bound is on Σ A_prev + Σ |Δ|: a negative delta counts by magnitude
    let neg = Change {
        from: pid(1),
        to: pid(2),
        delta: -(1i128 << (WEIGHT_BITS - 1)),
    };
    assert!(Game::new(&base, &[neg, neg], &p).is_err());
    // inside the bound the game solves (a shift absorbs the large weights)
    let big = Change {
        from: pid(1),
        to: pid(2),
        delta: 1i128 << 100,
    };
    let game = Game::new(&base, &[big, big, big], &p).unwrap();
    let all: Vec<usize> = (0..3).collect();
    assert_eq!(
        game.sample_along(&all).iter().sum::<i128>(),
        game.value(&all)
    );
}
