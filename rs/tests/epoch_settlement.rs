//! Epoch settlement: determinism, order independence, the gradient against
//! finite differences, completeness of the integrated gradient, agreement with
//! exact Shapley on small graphs, the valence net weight, the φ digest
//! preimage, and blessed conformance vectors. Performance tests at chain scale
//! are `#[ignore]`d (they read the link files of `eval/links_bin.py`).

use tru::focusing::epoch::fixed::GRAD_BITS;
use tru::focusing::epoch::integrated::{AxonScore, integrated_gradient, midpoint_attribution};
use tru::focusing::epoch::{
    Axon, AxonKey, EpochGraph, EpochParams, Ground, PHI_STEPS, Position, canonical, gradient,
    midpoint, net_axons, path_gradient, phi, preimage, solve_steps, syntropy,
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

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// A base graph with one particle X (id 999) that has in-axons but no
/// out-axon, and `cur` = base with five changes: X gains its first out-axon,
/// plus four weight changes of ±30 %.
fn first_out_link_case(seed: u64) -> (Vec<Axon>, Vec<Axon>) {
    let mut prev = random_graph(500 + seed, 10, 34);
    let x = pid(999);
    for k in 0..2 {
        prev.push(Axon {
            from: pid((seed + 3 * k) % 10),
            to: x,
            weight: 2_000_000 + 1_000_000 * k as u128,
        });
    }
    let prev = canonical(&prev);
    let mut cur = prev.clone();
    let mut r = seed ^ 0x55;
    for _ in 0..4 {
        r = mix(r);
        let k = (r % cur.len() as u64) as usize;
        cur[k].weight = cur[k].weight * (70 + (r >> 20) % 61) as u128 / 100;
    }
    cur.push(Axon {
        from: x,
        to: pid(seed % 10),
        weight: 1_500_000,
    });
    (prev, canonical(&cur))
}

#[test]
fn path_gradient_tracks_exact_shapley() {
    // The protocol rule (ground γ_i = 1/4, four Gauss–Legendre nodes) against
    // exact Shapley, per class of change; acceptance in
    // audit/epoch-settlement-2026-10.md §6: median ≤ 1 % on ±30 % weight
    // changes, ≤ 5 % when a particle gains or loses its only out-axon.
    let p = params(256, 1);
    let protocol = |prev: &[Axon], cur: &[Axon], p: &EpochParams| {
        rel_err(
            &integrated_gradient(prev, cur, p.quadrature, p),
            &shapley(prev, cur, p),
        )
    };
    let mut smooth = Vec::new();
    for seed in 0..6 {
        let prev = random_graph(300 + seed, 10, 40);
        let mut cur = prev.clone();
        let mut r = seed;
        for _ in 0..5 {
            r = mix(r);
            let k = (r % cur.len() as u64) as usize;
            cur[k].weight = cur[k].weight * (70 + (r >> 20) % 61) as u128 / 100;
        }
        smooth.push(protocol(&prev, &cur, &p));
    }
    let (mut first, mut last, mut plain) = (Vec::new(), Vec::new(), Vec::new());
    let none = EpochParams {
        ground: Ground::NONE,
        ..p.clone()
    };
    for seed in 0..7 {
        let (prev, cur) = first_out_link_case(seed);
        first.push(protocol(&prev, &cur, &p));
        last.push(protocol(&cur, &prev, &p));
        plain.push(protocol(&prev, &cur, &none));
    }
    let (ms, mf, ml, mp) = (median(smooth), median(first), median(last), median(plain));
    eprintln!(
        "median error: ±30 % {ms:.4}, first out-axon {mf:.4}, last out-axon {ml:.4}, first out-axon without ground {mp:.4}"
    );
    assert!(ms <= 0.01, "±30 % weight changes: median {ms}");
    assert!(mf <= 0.05, "first out-axon: median {mf}");
    assert!(ml <= 0.05, "last out-axon: median {ml}");
    assert!(
        mp > 0.10,
        "without the ground the first out-axon is a jump: median {mp}"
    );
}

#[test]
fn focus_is_continuous_at_a_first_out_axon() {
    // X has in-axons and no out-axon; give it one of weight ε. With the ground
    // the focus moves by O(ε); without it, it jumps.
    let (prev, _) = first_out_link_case(1);
    let x = pid(999);
    let move_by = |eps: u128, ground: Ground| {
        let mut cur = prev.clone();
        cur.push(Axon {
            from: x,
            to: pid(1),
            weight: eps,
        });
        let p = EpochParams {
            ground,
            ..params(256, 1)
        };
        let (a, b) = (phi(&prev, &p), phi(&cur, &p));
        a.phi
            .iter()
            .zip(&b.phi)
            .map(|(&u, &v)| u.abs_diff(v) as f64)
            .sum::<f64>()
            / ONE
    };
    let (small, tiny) = (
        move_by(1_000, Ground::PROTOCOL),
        move_by(10, Ground::PROTOCOL),
    );
    assert!(
        small < 1e-3 && tiny < small / 20.0,
        "ground: ε=1000 moves {small:e}, ε=10 moves {tiny:e}"
    );
    let jump = move_by(10, Ground::NONE);
    assert!(
        jump > 1e-2,
        "without ground a weight-10 axon moves φ* by {jump:e}"
    );
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

/// The conformance path: axon (1,2) raised 50 %, (2,5) removed, (5,1) new
/// (particle 5 keeps (5,3)); a new particle 6 behind (4,6).
fn vector_path() -> (Vec<Axon>, Vec<Axon>) {
    let prev = vector_graph();
    let mut cur: Vec<Axon> = prev
        .iter()
        .filter(|a| a.from != [2; 32] || a.to != [5; 32])
        .copied()
        .collect();
    cur[0].weight = 1_500_000;
    let p = |b: u8| [b; 32];
    cur.push(Axon {
        from: p(5),
        to: p(1),
        weight: 800_000,
    });
    cur.push(Axon {
        from: p(4),
        to: p(6),
        weight: 600_000,
    });
    (prev, cur)
}

#[test]
fn conformance_vector_path_gradient() {
    let (prev, cur) = vector_path();
    let g = path_gradient(&prev, &cur, &EpochParams::default());
    assert_eq!(
        g.by_axon, VECTOR_PATH,
        "path gradient of the conformance path moved"
    );
}

#[test]
fn conformance_vector_gradient() {
    let g = gradient(&vector_graph(), &EpochParams::default());
    assert_eq!(
        g.by_axon, VECTOR_GRAD,
        "gradient of the conformance graph moved"
    );
}

// Blessed 2026-10-17 (tru feat/epoch-ground): `EpochParams::default()` —
// PHI_STEPS = 48, ground γ_i = 1/4, four Gauss–Legendre nodes. φ in ascending
// particle order ([1;32] … [5;32]); the gradient in input order; the path
// gradient in union order ([`vector_path`]). hemera 0.4 (7237a23) of the φ
// preimage: e0ff79997aa697bced38e2cdedb823b189892982bae411933ec623007de18fc8
const VECTOR_PHI: [u64; 5] = [
    219_310_425_963_150_397,
    244_918_065_555_344_491,
    314_071_252_939_911_789,
    227_905_484_039_409_407,
    146_716_275_947_897_466,
];
const VECTOR_J: i128 = 31_829_771_616_584_911;
const VECTOR_GRAD: [i128; 7] = [
    1_914_075_910_134_244_213_425_895,
    71_171_900_741_159_642_673_700_975,
    9_937_083_483_998_543_538_357_229,
    -17_125_362_367_786_076_875_988_209,
    11_207_443_854_475_690_734_085_092,
    -13_369_225_266_052_630_809_369_953,
    -101_821_770_616_628_008_646_342_672,
];
const VECTOR_PATH: [i128; 9] = [
    24_512_188_647_810_751_159_877_835,
    6_595_504_669_415_890_941_122_482,
    69_113_271_079_698_993_274_897_226,
    -129_462_933_411_104_598_769_260_652,
    52_682_207_631_725_481_215_888_292,
    63_409_899_129_634_271_710_099_828,
    -830_202_655_685_804_916_443_499_500,
    -68_394_074_836_277_878_255_736_175,
    -23_457_048_889_162_735_643_430_201,
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
    let g = path_gradient(&prev, &cur, &p);
    let el = t.elapsed();
    eprintln!(
        "bostrom settlement (solve 1 + {}-node path gradient), one thread: {el:.2?}",
        p.quadrature
    );
    assert!(f.syntropy > 0 && !g.by_axon.is_empty());
    assert!(
        el.as_secs() < 300,
        "settlement must stay within 300 s single-thread"
    );
}
