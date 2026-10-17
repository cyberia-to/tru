//! Epoch settlement: determinism, order independence, the gradient against
//! finite differences, completeness of the integrated gradient, agreement with
//! exact Shapley on small graphs, the valence net weight, the φ digest
//! preimage, and blessed conformance vectors. Performance tests at chain scale
//! are `#[ignore]`d (they read the link files of `eval/links_bin.py`).

use tru::focusing::epoch::fixed::GRAD_BITS;
use tru::focusing::epoch::integrated::{AxonScore, integrated_gradient, midpoint_attribution};
use tru::focusing::epoch::{
    Axon, AxonKey, EpochGraph, EpochParams, PHI_STEPS, Position, canonical, gradient, midpoint,
    net_axons, phi, preimage, solve_steps, syntropy,
};

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

// ── determinism ──────────────────────────────────────────────────────────

#[test]
fn phi_is_bit_identical_across_runs_and_thread_counts() {
    let axons = random_graph(7, 3000, 9000);
    let a = phi(&axons, &params(PHI_STEPS, 1));
    for th in [1, 2, 3, 8] {
        let b = phi(&axons, &params(PHI_STEPS, th));
        assert_eq!(a.phi, b.phi, "φ differs at threads={th}");
        assert_eq!(a.syntropy, b.syntropy);
        assert_eq!(a.drift, b.drift);
    }
}

#[test]
fn gradient_is_bit_identical_across_thread_counts() {
    let axons = random_graph(8, 1500, 4500);
    let a = gradient(&axons, &params(PHI_STEPS, 1));
    for th in [2, 5, 8] {
        let b = gradient(&axons, &params(PHI_STEPS, th));
        assert_eq!(a.by_axon, b.by_axon, "gradient differs at threads={th}");
        assert_eq!(a.syntropy, b.syntropy);
    }
}

#[test]
fn input_order_does_not_matter() {
    let axons = random_graph(9, 800, 2400);
    let a = phi(&axons, &params(PHI_STEPS, 1));
    // reverse, rotate, and split one axon into two duplicate entries
    let mut shuffled: Vec<Axon> = axons.iter().rev().copied().collect();
    shuffled.rotate_left(17);
    let mut dup = shuffled[3];
    let half = dup.weight / 2;
    shuffled[3].weight -= half;
    dup.weight = half;
    shuffled.push(dup);
    let b = phi(&shuffled, &params(PHI_STEPS, 1));
    assert_eq!(a.particles, b.particles);
    assert_eq!(a.phi, b.phi);
    // the gradient follows its input index
    let ga = gradient(&axons, &params(PHI_STEPS, 1));
    let gb = gradient(&shuffled, &params(PHI_STEPS, 1));
    for (k, x) in shuffled.iter().enumerate() {
        let j = axons
            .binary_search_by(|y| (y.from, y.to).cmp(&(x.from, x.to)))
            .unwrap();
        assert_eq!(gb.by_axon[k], ga.by_axon[j]);
    }
}

#[test]
fn particles_ascend_and_focus_is_a_distribution() {
    let axons = random_graph(10, 500, 1500);
    let f = phi(&axons, &params(PHI_STEPS, 1));
    assert!(f.particles.windows(2).all(|w| w[0] < w[1]));
    assert!(
        f.phi.iter().all(|&x| x > 0),
        "every staked particle has focus"
    );
    let mass: u128 = f.phi.iter().map(|&x| x as u128).sum();
    let err = mass.abs_diff(1u128 << 60);
    assert!(
        err < 1 << 32,
        "Σφ = 2^60 within 2^-28 relative, off by {err}"
    );
    assert!(f.syntropy > 0);
}

#[test]
fn forty_eight_steps_reach_the_fixed_point() {
    let axons = random_graph(11, 2000, 6000);
    let g = EpochGraph::build(&axons);
    let p = params(PHI_STEPS, 1);
    let at = solve_steps(&g, &p, PHI_STEPS);
    let far = solve_steps(&g, &p, 4 * PHI_STEPS);
    let l1: f64 = at
        .phi
        .iter()
        .zip(&far.phi)
        .map(|(&a, &b)| a.abs_diff(b) as f64)
        .sum::<f64>()
        / ONE;
    assert!(l1 < 1e-6, "ℓ1 to the converged φ* after 48 steps = {l1:e}");
}

#[test]
fn empty_and_degenerate_inputs() {
    let p = params(PHI_STEPS, 1);
    assert!(phi(&[], &p).phi.is_empty());
    let selfloop = [Axon {
        from: pid(1),
        to: pid(1),
        weight: 5,
    }];
    assert!(phi(&selfloop, &p).phi.is_empty());
    assert_eq!(gradient(&selfloop, &p).by_axon, vec![0]);
    let zero = [
        Axon {
            from: pid(1),
            to: pid(2),
            weight: 0,
        },
        Axon {
            from: pid(2),
            to: pid(3),
            weight: 9,
        },
    ];
    let f = phi(&zero, &p);
    assert_eq!(f.phi.len(), 2, "a zero-weight axon is absent");
    assert_eq!(gradient(&zero, &p).by_axon[0], 0);
    // weights near u128::MAX are shifted, not overflowed
    let huge = [
        Axon {
            from: pid(1),
            to: pid(2),
            weight: u128::MAX / 3,
        },
        Axon {
            from: pid(2),
            to: pid(1),
            weight: u128::MAX / 5,
        },
    ];
    let f = phi(&huge, &p);
    assert!(f.phi.iter().all(|&x| x > 0));
}

// ── the gradient ─────────────────────────────────────────────────────────

fn j_of(axons: &[Axon], p: &EpochParams) -> f64 {
    phi(axons, p).syntropy as f64 / ONE
}

#[test]
fn gradient_matches_finite_differences() {
    let p = params(300, 1);
    let mut checked = 0;
    for seed in 0..4 {
        let axons = random_graph(100 + seed, 14, 40);
        let g = gradient(&axons, &p);
        for k in (0..axons.len()).step_by(3) {
            let h = axons[k].weight / 50;
            let mut up = axons.clone();
            up[k].weight += h;
            let mut dn = axons.clone();
            dn[k].weight -= h;
            let fd = (j_of(&up, &p) - j_of(&dn, &p)) / (2.0 * h as f64);
            let an = g.by_axon[k] as f64 / 2f64.powi(GRAD_BITS as i32);
            let scale = fd.abs().max(an.abs()).max(1e-15);
            assert!(
                (fd - an).abs() / scale < 2e-3 || (fd - an).abs() < 1e-13,
                "seed {seed} axon {k}: finite difference {fd:e} vs adjoint {an:e}"
            );
            checked += 1;
        }
    }
    assert!(checked > 20);
}

#[test]
fn integrated_gradient_is_complete() {
    // Σ ΔA·∫g = J∪(cur) − J∪(prev) up to quadrature error.
    let p = params(256, 1);
    for seed in 0..3 {
        let prev = random_graph(200 + seed, 20, 60);
        let mut cur = prev.clone();
        for (k, a) in cur.iter_mut().enumerate() {
            if k % 4 == 0 {
                a.weight = a.weight * 13 / 10;
            } else if k % 7 == 0 {
                a.weight = a.weight * 8 / 10;
            }
        }
        let mut parts: Vec<[u8; 32]> = prev.iter().flat_map(|a| [a.from, a.to]).collect();
        parts.sort();
        parts.dedup();
        let nu = parts.len() as u64;
        let dj =
            (syntropy(&phi(&cur, &p).phi, nu) - syntropy(&phi(&prev, &p).phi, nu)) as f64 / ONE;
        let sum = |s: &[AxonScore]| s.iter().map(|x| x.score as f64).sum::<f64>() / ONE;
        let ig4 = integrated_gradient(&prev, &cur, 4, &p);
        let mid = midpoint_attribution(&prev, &cur, &p);
        let gross = ig4.iter().map(|x| (x.score as f64).abs()).sum::<f64>() / ONE;
        let (s4, s1) = (sum(&ig4), sum(&mid));
        eprintln!("seed {seed}: ΔJ={dj:e} Σ|a|={gross:e} GL-4 Σ={s4:e} midpoint Σ={s1:e}");
        assert!(
            (s4 - dj).abs() <= 1e-3 * gross,
            "GL-4 Σ={s4:e} vs ΔJ={dj:e}"
        );
        assert!(
            (s1 - dj).abs() <= 5e-2 * gross,
            "midpoint Σ={s1:e} vs ΔJ={dj:e}"
        );
    }
}

/// Exact Shapley over changed axons, `v(S) = J∪(A_prev + Δ_S) − J∪(A_prev)`.
fn shapley(prev: &[Axon], cur: &[Axon], p: &EpochParams) -> Vec<(AxonKey, f64)> {
    let mid = midpoint(prev, cur); // the union, ascending
    let w = |l: &[Axon], k: ([u8; 32], [u8; 32])| {
        l.iter()
            .find(|a| (a.from, a.to) == k)
            .map_or(0, |a| a.weight)
    };
    let all: Vec<_> = mid
        .iter()
        .map(|m| {
            (
                (m.from, m.to),
                w(prev, (m.from, m.to)),
                w(cur, (m.from, m.to)),
            )
        })
        .collect();
    let mut parts: Vec<[u8; 32]> = mid.iter().flat_map(|a| [a.from, a.to]).collect();
    parts.sort();
    parts.dedup();
    let nu = parts.len() as u64;
    let pl: Vec<usize> = (0..all.len()).filter(|&i| all[i].1 != all[i].2).collect();
    let v: Vec<f64> = (0..1usize << pl.len())
        .map(|mask| {
            let ax: Vec<Axon> = all
                .iter()
                .enumerate()
                .map(|(i, &(k, a, b))| {
                    let on = pl
                        .iter()
                        .position(|&x| x == i)
                        .is_some_and(|j| mask >> j & 1 == 1);
                    Axon {
                        from: k.0,
                        to: k.1,
                        weight: if on { b } else { a },
                    }
                })
                .collect();
            syntropy(&phi(&ax, p).phi, nu) as f64 / ONE
        })
        .collect();
    let f = |k: usize| (1..=k).map(|x| x as f64).product::<f64>();
    let np = pl.len();
    pl.iter()
        .enumerate()
        .map(|(j, &i)| {
            let s: f64 = (0..1usize << np)
                .filter(|m| m >> j & 1 == 0)
                .map(|m| {
                    let z = m.count_ones() as usize;
                    f(z) * f(np - z - 1) / f(np) * (v[m | 1 << j] - v[m])
                })
                .sum();
            (all[i].0, s)
        })
        .collect()
}

fn rel_err(est: &[AxonScore], sh: &[(AxonKey, f64)]) -> f64 {
    let (mut num, mut den) = (0.0, 0.0);
    for (k, s) in sh {
        let e = est
            .iter()
            .find(|x| x.axon == *k)
            .map_or(0.0, |x| x.score as f64 / ONE);
        num += (e - s).abs();
        den += s.abs();
    }
    num / den.max(1e-300)
}

#[test]
fn gradient_attribution_tracks_exact_shapley_on_smooth_changes() {
    // Stake changes on existing axons (no particle appears, no particle loses
    // or gains its only out-axon): the regime where the gradient form is the
    // Aumann–Shapley value of a smooth game. Bound measured in
    // audit/epoch-settlement-2026-10.md.
    let p = params(256, 1);
    let mut worst = (0.0f64, 0.0f64);
    for seed in 0..6 {
        let prev = random_graph(300 + seed, 10, 40);
        let mut cur = prev.clone();
        let mut r = seed;
        for _ in 0..5 {
            r = mix(r);
            let k = (r % cur.len() as u64) as usize;
            let pct = 70 + (r >> 20) % 61; // ×0.70 … ×1.30
            cur[k].weight = cur[k].weight * pct as u128 / 100;
        }
        let sh = shapley(&prev, &cur, &p);
        let e_mid = rel_err(&midpoint_attribution(&prev, &cur, &p), &sh);
        let e_gl4 = rel_err(&integrated_gradient(&prev, &cur, 4, &p), &sh);
        worst = (worst.0.max(e_mid), worst.1.max(e_gl4));
    }
    assert!(
        worst.0 < 0.10,
        "midpoint vs Shapley relative ℓ1 = {}",
        worst.0
    );
    assert!(worst.1 < 0.10, "GL-4 vs Shapley relative ℓ1 = {}", worst.1);
}

// ── valence: the chain's net weight ──────────────────────────────────────

#[test]
fn valence_net_weight_adds_subtracts_and_clips() {
    let (a, b, c) = (pid(1), pid(2), pid(3));
    let pos = |from, to, stake, valence| Position {
        from,
        to,
        stake,
        valence,
    };
    let genesis = [Axon {
        from: a,
        to: b,
        weight: 10,
    }];
    let net = net_axons(
        &genesis,
        &[
            pos(a, b, 5, 1),  // +5
            pos(a, b, 3, 0),  // passive adds like +1
            pos(a, b, 7, -1), // subtracts
            pos(b, c, 4, 1),
            pos(b, c, 9, -1), // clipped: axon absent
            pos(c, a, 2, 0),
        ],
    );
    assert_eq!(
        net,
        vec![
            Axon {
                from: a,
                to: b,
                weight: 11
            },
            Axon {
                from: c,
                to: a,
                weight: 2
            },
        ]
    );
}

#[test]
fn opposing_stake_on_a_syntropy_raising_axon_has_the_opposite_sign() {
    // a(c) = Δs·σ·g: on an axon with g > 0 a +1 position adding stake scores
    // positive and a −1 position adding stake scores negative, and vice versa.
    let axons = random_graph(400, 12, 36);
    let g = gradient(&axons, &params(PHI_STEPS, 1));
    let k = (0..axons.len()).max_by_key(|&k| g.by_axon[k]).unwrap();
    assert!(g.by_axon[k] > 0);
    let plus = tru::focusing::epoch::integrated::score_of(1_000, g.by_axon[k]);
    let minus = tru::focusing::epoch::integrated::score_of(-1_000, g.by_axon[k]);
    assert!(plus > 0 && minus == -plus);
}

// ── digest and conformance vectors ───────────────────────────────────────

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

#[test]
fn phi_digest_preimage_matches_the_chain_vector() {
    // chain-economy §10: hemera(preimage) for n = 2, φ = [2^59, 2^59] is
    // 3351bfa40d6eaa24c8e2b8755d900dbcd3c3015730a5efcfdf9b6e14c7019a8a
    // (cyber-hemera 0.4 at 7237a23; checked 2026-10-17). tru pins hemera 0.3,
    // so the hash is the chain's; the preimage bytes are tru's.
    assert_eq!(
        hex(&preimage(&[1 << 59, 1 << 59])),
        "736f6674332f636861696e2f7068692f76310002000000\
         00000000000000080000000000000008"
    );
}

/// The conformance graph: five particles, seven axons.
fn vector_graph() -> Vec<Axon> {
    let p = |b: u8| [b; 32];
    let ax = |f: u8, t: u8, w: u128| Axon {
        from: p(f),
        to: p(t),
        weight: w,
    };
    vec![
        ax(1, 2, 1_000_000),
        ax(2, 3, 2_500_000),
        ax(3, 1, 700_000),
        ax(1, 4, 4_000_000),
        ax(4, 2, 1_200_000),
        ax(5, 3, 3_000_000),
        ax(2, 5, 900_000),
    ]
}

#[test]
fn conformance_vector_phi() {
    let f = phi(&vector_graph(), &EpochParams::default());
    assert_eq!(f.phi, VECTOR_PHI, "φ* of the conformance graph moved");
    assert_eq!(f.syntropy, VECTOR_J);
}

#[test]
fn conformance_vector_gradient() {
    let g = gradient(&vector_graph(), &EpochParams::default());
    assert_eq!(
        g.by_axon, VECTOR_GRAD,
        "gradient of the conformance graph moved"
    );
}

// Blessed 2026-10-17 (tru feat/epoch-settlement): `EpochParams::default()`,
// PHI_STEPS = 48. φ in ascending particle order ([1;32] … [5;32]); the
// gradient in input order. hemera 0.4 (7237a23) of the φ preimage:
// e9462109f934da2baca3aef7f903033d829bb245f404c203c7a7352cec2ab1e0
const VECTOR_PHI: [u64; 5] = [
    264_668_384_119_641_678,
    257_361_480_420_213_991,
    283_365_895_498_496_502,
    222_988_084_795_569_635,
    124_537_659_616_327_972,
];
const VECTOR_J: i128 = 38_984_667_269_542_703;
const VECTOR_GRAD: [i128; 7] = [
    25_131_023_120_481_710_517_417_596,
    66_161_466_516_476_074_186_608_474,
    39_963_303_351_423_591_141_803_536,
    2_637_987_610_521_681_915_732_732,
    1_169_308_602_267_229_865_745_330,
    -35_266_280_524_199_669_297_466_447,
    -138_516_978_833_138_047_665_348_459,
];

// ── chain scale (ignored; needs /tmp/tru-epoch/*.links or TRU_EPOCH_LINKS) ─

fn load(name: &str) -> Option<Vec<Axon>> {
    let path =
        std::env::var("TRU_EPOCH_LINKS").unwrap_or_else(|_| format!("/tmp/tru-epoch/{name}.links"));
    let bytes = std::fs::read(path).ok()?;
    let u = |o: usize| u32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
    let count = u(8) as usize;
    let mut pos = Vec::with_capacity(count);
    for i in 0..count {
        let o = 16 + 16 * i;
        let id = |x: u32| {
            let mut h = [0u8; 32];
            h[..8].copy_from_slice(&mix(x as u64 ^ 0xC0FF_EE00).to_be_bytes());
            h[8..12].copy_from_slice(&x.to_be_bytes());
            h
        };
        let r = mix(i as u64);
        let stake = (1e6 * (((r >> 11) as f64 + 1.0) / (1u64 << 53) as f64).powf(-1.0 / 1.16))
            .min(1e15) as u128;
        pos.push(Position {
            from: id(u(o)),
            to: id(u(o + 4)),
            stake,
            valence: 1,
        });
    }
    Some(net_axons(&[], &pos))
}

#[test]
#[ignore = "chain-scale benchmark; run with --ignored --release"]
fn bostrom_phi_within_budget() {
    let Some(axons) = load("bostrom") else {
        eprintln!("no bostrom link file; skipped");
        return;
    };
    for threads in [1, 12] {
        let t = std::time::Instant::now();
        let f = phi(&axons, &params(PHI_STEPS, threads));
        let el = t.elapsed();
        eprintln!("bostrom φ* threads={threads}: {el:.2?}, n={}", f.phi.len());
        assert!(el.as_secs() < if threads == 1 { 20 } else { 5 });
    }
}

#[test]
#[ignore = "chain-scale benchmark; run with --ignored --release"]
fn bostrom_settlement_within_budget() {
    let Some(cur) = load("bostrom") else {
        eprintln!("no bostrom link file; skipped");
        return;
    };
    let prev: Vec<Axon> = cur[..cur.len() - 10_000].to_vec();
    let t = std::time::Instant::now();
    let p = params(PHI_STEPS, 1);
    let f = phi(&cur, &p);
    let g = gradient(&midpoint(&prev, &cur), &p);
    let el = t.elapsed();
    eprintln!("bostrom settlement (solve 1 + gradient), one thread: {el:.2?}");
    assert!(f.syntropy > 0 && !g.by_axon.is_empty());
    assert!(
        el.as_secs() < 900 / 10,
        "settlement must stay well under SETTLE_BUDGET"
    );
}
