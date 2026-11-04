//! The coalition marginal and the ticket sample on the epoch engine
//! (`rs/focusing/epoch/coalition.rs`): the one-change marginal against the
//! engine's own attribution, efficiency (the sample sums to the coalition's
//! value, exactly), determinism across runs and thread counts (a pinned
//! digest), the ordering as a beacon function, and the sample against the
//! exact Shapley value by enumeration — identically over all orderings for
//! `n ≤ 6`, by Monte Carlo for `n = 8`.

use tru::focusing::epoch::D_SAMPLE;
use tru::focusing::epoch::coalition::WEIGHT_BITS;
use tru::focusing::epoch::fixed::div_round_i;
use tru::focusing::epoch::integrated::integrated_gradient;
use tru::focusing::epoch::{Axon, Change, EpochParams, Game, canonical, ordering, phi, probes};

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
    // it — audit/coalition-marginal-2026-11-04.md. The digest is hemera
    // 0.3.1's (as the D_PHI vector's); when feat/hemera-0.4 lands it moves,
    // and the pin moves with it.
    assert_eq!(
        hex(digest.as_bytes()),
        "ba4bd05facf5580bfa6023ef0c5b467145de2f0582e04b731d8d2f5b0cd6f17b",
        "the sample's digest moved: a consensus change"
    );
}

#[test]
fn ordering_is_uniform_over_the_permutations_of_three() {
    // 6,000 nonces over the 6 orderings of three players: every bucket near
    // 1,000 (±15 %; a biased draw or a fixed point would show at once).
    let mut buckets = std::collections::BTreeMap::new();
    for nonce in 0..6_000u64 {
        *buckets
            .entry(ordering(&[4; 32], &[5; 32], nonce, 3))
            .or_insert(0u32) += 1;
    }
    assert_eq!(buckets.len(), 6);
    for (o, c) in &buckets {
        assert!(
            (850..=1_150).contains(c),
            "ordering {o:?} drawn {c} times of 6,000"
        );
    }
}

#[test]
fn syntropy_of_an_axon_list_is_the_engine_s_own() {
    // J_U with U = the list's own particles is epoch::phi's syntropy, bit
    // for bit: the game's value is the engine's function, not a sibling.
    let p = params(48, 1);
    let base = random_graph(900, 30, 120);
    let game = Game::new(&base, &[], &p).unwrap();
    assert_eq!(game.base_syntropy(), phi(&base, &p).syntropy);
    assert_eq!(game.syntropy_of(&base), phi(&base, &p).syntropy);
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

// ── the ticket: the window and the probes (rewards.md §7) ─────────────────

/// The chain's rank draw, `r = u64_le(hemera(template)[8..16]) mod N`, with
/// the nonce bytes standing in for the template.
fn rank_of(nonce: u64, n: usize) -> usize {
    let h = cyber_hemera::hash(&nonce.to_le_bytes());
    (u64::from_le_bytes(h.as_bytes()[8..16].try_into().unwrap()) % n as u64) as usize
}

#[test]
fn window_covers_each_player_with_probability_w_over_n() {
    // N = 10 players, W_TICKET = 4 (three marginals, one probe): over 3,000
    // tickets each player is covered ≈ 900 times (3/10; ±15 %), the window
    // is cyclic (ranks 8, 9, 0 when r = 8) and every ticket has 4 entries.
    let p = params(16, 1);
    let base = random_graph(1000, 14, 40);
    let pl = players(&base, 10, 2);
    let game = Game::new(&base, &pl, &p).unwrap();
    let (b, c) = ([6u8; 32], [7u8; 32]);
    let mut covered = [0u32; 10];
    let mut wrapped = false;
    for nonce in 0..3_000u64 {
        let t = game.ticket(&b, &c, nonce, rank_of(nonce, 10), 4);
        assert_eq!(t.width(), 4);
        assert_eq!(t.covered.len(), 3);
        assert_eq!(t.probes.len(), 1);
        assert!(t.covered.windows(2).all(|w| w[0] < w[1]));
        let ranks: Vec<usize> = (0..3).map(|k| t.order[(t.rank + k) % 10]).collect();
        let mut sorted = ranks.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, t.covered);
        wrapped |= t.rank == 8;
        for &i in &t.covered {
            covered[i] += 1;
        }
    }
    assert!(wrapped);
    for (i, &k) in covered.iter().enumerate() {
        assert!(
            (765..=1_035).contains(&k),
            "player {i} covered {k} of 3,000"
        );
    }
}

#[test]
fn full_window_ticket_is_the_sample_and_sums_to_v_n() {
    // W_TICKET − 1 ≥ N: the marginals are the whole ordering (= `sample`),
    // Σ = v(N) exactly, whatever the drawn rank; the rest are probes.
    let p = params(48, 1);
    let base = random_graph(1100, 16, 48);
    let pl = players(&base, 5, 4);
    let game = Game::new(&base, &pl, &p).unwrap();
    let (b, c) = ([8u8; 32], [9u8; 32]);
    let all: Vec<usize> = (0..5).collect();
    let total = game.value(&all);
    for nonce in 0..6u64 {
        let s = game.sample(&b, &c, nonce);
        for rank in 0..5 {
            let t = game.ticket(&b, &c, nonce, rank, 8);
            assert_eq!(t.covered, all);
            assert_eq!(
                t.marginals.iter().sum::<i128>(),
                total,
                "nonce {nonce} rank {rank}"
            );
            assert_eq!(t.probes.len(), 3);
            // a wrapped window re-walks the ordering from ∅: the same marginals
            assert_eq!(t.marginals, s.marginals, "nonce {nonce} rank {rank}");
        }
    }
}

#[test]
fn probe_is_the_engine_s_finite_difference_at_the_drawn_node() {
    // m^probe = J_U(φ*(A_E + u·e_a)) − J_U(φ*_E), u = max(1, ⌊A_a·t/2^32⌋):
    // exactly the game's own value of that change, and the engine's four-node
    // attribution of it to the quadrature's error.
    let p = params(256, 1);
    let base = random_graph(1200, 14, 40);
    let game = Game::new(&base, &[], &p).unwrap();
    let draws = probes(&[1; 32], &[2; 32], 5, 4, base.len());
    assert_eq!(draws.len(), 4);
    for &(a, t) in &draws {
        let pr = game.probe(a, t);
        assert_eq!(pr.u, ((base[a].weight * t as u128) >> 32).max(1));
        let mut cur = base.clone();
        cur[a].weight += pr.u;
        assert_eq!(pr.value, game.syntropy_of(&cur) - game.base_syntropy());
        let ig = integrated_gradient(&base, &cur, 4, &p);
        assert_eq!(ig.len(), 1);
        let (m, s) = (pr.value as f64 / ONE, ig[0].score as f64 / ONE);
        let err = (m - s).abs() / s.abs().max(1e-300);
        eprintln!(
            "probe axon {a} t {t} u {}: {m:e} vs attribution {s:e} rel {err:e}",
            pr.u
        );
        assert!(err < 1e-4, "probe vs path gradient: {err:e}");
    }
    // the floor: t = 0 adds one unit
    assert_eq!(game.probe(0, 0).u, 1);
    // at the weight bound the product A·t does not overflow (R46-11)
    let heavy = vec![
        Axon {
            from: pid(1),
            to: pid(2),
            weight: (1u128 << 123) - 1,
        },
        Axon {
            from: pid(2),
            to: pid(3),
            weight: 1 << 122,
        },
    ];
    let g = Game::new(&heavy, &[], &params(8, 1)).unwrap();
    let pr = g.probe(0, u32::MAX);
    assert_eq!(
        pr.u,
        (((1u128 << 123) - 1) >> 32) * u32::MAX as u128
            + ((0xFFFF_FFFFu128 * u32::MAX as u128) >> 32)
    );
    assert_eq!(g.probe(0, 1 << 31).u, 1 << 122);
    assert_eq!(g.probe(1, 1 << 31).u, 1 << 121);
    // an empty epoch's ticket is all probes, W solves
    let t = game.ticket(&[1; 32], &[2; 32], 5, 0, 3);
    assert!(t.covered.is_empty() && t.probes.len() == 3 && t.width() == 3);
    assert_eq!(t.probes[0].value, game.probe(draws[0].0, draws[0].1).value);
}

#[test]
fn probes_change_with_the_nonce_and_the_seed() {
    let d = |b: u8, c: u8, n: u64| probes(&[b; 32], &[c; 32], n, 2, 1_000);
    assert_eq!(d(1, 2, 3), d(1, 2, 3));
    assert_ne!(
        d(1, 2, 3),
        d(1, 2, 4),
        "a different template digest draws different probes"
    );
    assert_ne!(d(1, 2, 3), d(9, 2, 3));
    assert_ne!(d(1, 2, 3), d(1, 9, 3));
    assert!(probes(&[1; 32], &[2; 32], 3, 2, 0).is_empty());
}

#[test]
fn window_mean_per_player_equals_shapley_by_enumeration_for_n_above_the_width() {
    // N = 5 > W_TICKET − 1 = 2: over every ordering and every rank each
    // player's covered marginals average to its Shapley value, exactly.
    let p = params(48, 1);
    let base = random_graph(1300, 12, 30);
    let pl = players(&base, 5, 6);
    let game = Game::new(&base, &pl, &p).unwrap();
    let sh = game.shapley();
    let (b, c) = ([3u8; 32], [4u8; 32]);
    let (mut acc, mut cnt) = ([0i128; 5], [0i128; 5]);
    for o in permutations(5) {
        for rank in 0..5 {
            // the window over an explicit ordering (a ticket's ordering comes
            // from the beacon, so the enumeration uses `sample_along`; the
            // ticket is compared with it on drawn orderings below)
            let m = game.sample_along(&o);
            for k in 0..2 {
                let i = o[(rank + k) % 5];
                acc[i] += m[i];
                cnt[i] += 1;
            }
        }
    }
    let mean: Vec<i128> = (0..5).map(|i| div_round_i(acc[i], cnt[i])).collect();
    assert_eq!(mean, sh);
    // the ticket's window is that window: compare on drawn orderings
    for nonce in 0..4u64 {
        let t = game.ticket(&b, &c, nonce, rank_of(nonce, 5), 3);
        let m = game.sample_along(&t.order);
        for (i, v) in t.covered.iter().zip(&t.marginals) {
            assert_eq!(*v, m[*i]);
        }
    }
}

#[test]
fn ticket_is_bit_identical_across_runs_and_thread_counts() {
    let base = random_graph(500, 40, 160);
    let pl = players(&base, 9, 1);
    let (b, c) = ([0x5a; 32], [3; 32]);
    let one = Game::new(&base, &pl, &params(48, 1))
        .unwrap()
        .ticket(&b, &c, 11, 4, 8);
    let again = Game::new(&base, &pl, &params(48, 1))
        .unwrap()
        .ticket(&b, &c, 11, 4, 8);
    let four = Game::new(&base, &pl, &params(48, 4))
        .unwrap()
        .ticket(&b, &c, 11, 4, 8);
    assert_eq!(one, again);
    assert_eq!(one, four);
    assert_eq!(one.covered.len(), 7);
    assert_eq!(one.probes.len(), 1);
    let digest = cyber_hemera::hash(&one.preimage());
    eprintln!("ticket digest {}", digest.to_hex());
    assert_eq!(one.preimage().len(), D_SAMPLE.len() + 4 + 16 * 8);
    // hemera 0.3.1's digest; moves with feat/hemera-0.4 (re-blessed then)
    assert_eq!(
        hex(digest.as_bytes()),
        "331ed36850cd2a82d26fa783e6db8417ebd0f6d4bdfaf47dce801c14f32e9730",
        "the ticket's digest moved: a consensus change"
    );
}
