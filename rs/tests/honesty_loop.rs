//! The honesty loop, end to end: BTS reports → karma → effective adjacency → φ*.
//!
//! This is the seam items 1 (A_eff honesty-weighting) and 5 (truth-scoring)
//! close together. A neuron that contributed genuine private signal earns karma
//! from [`tru::accumulate`]; that karma, fed to [`FocusingGraph::build`], makes
//! the neuron's links weigh more in the tri-kernel — so its link targets draw
//! more collective focus than an equal-stake follower's.

use tru::{
    accumulate, compute_focusing, Context, FocusingGraph, FocusingParams, Fx, Karma, Link, Report,
};

fn hash(b: u8) -> [u8; 32] {
    let mut h = [0u8; 32];
    h[0] = b;
    h
}

fn r(id: u8, belief: (i64, i64), prediction: (i64, i64)) -> Report {
    Report {
        neuron: hash(id),
        belief: Fx::from_ratio(belief.0, belief.1),
        prediction: Fx::from_ratio(prediction.0, prediction.1),
    }
}

#[test]
fn reports_earn_karma_that_reweights_focus() {
    // 1. An epoch of BTS reports. Neurons 1–3 follow the crowd (believe and
    //    predict "valid"); neuron 4 is an informed contrarian — it reports
    //    "invalid" yet correctly predicts the crowd will say "valid", the
    //    surprisingly-popular signal.
    let reports = vec![
        r(1, (8, 10), (8, 10)),
        r(2, (8, 10), (8, 10)),
        r(3, (8, 10), (8, 10)),
        r(4, (15, 100), (8, 10)),
    ];
    let karma = accumulate(&Karma::none(), &reports, Fx::from_ratio(1, 2));
    assert!(
        karma.get(&hash(4)) > karma.get(&hash(1)),
        "the contrarian must out-earn the follower in karma"
    );

    // 2. A symmetric voter graph: voter V splits attention between two targets
    //    X and Y through two authors — the honest neuron 4 backs X, the follower
    //    neuron 1 backs Y, at equal stake. Everything else is symmetric.
    let (v, x, y, c) = (hash(5), hash(10), hash(11), hash(12));
    let links = vec![
        Link {
            neuron: hash(4),
            from: v,
            to: x,
            amount: 100,
            valence: 1,
            price: Fx::ONE,
        },
        Link {
            neuron: hash(1),
            from: v,
            to: y,
            amount: 100,
            valence: 1,
            price: Fx::ONE,
        },
        Link::stake(x, c, 100),
        Link::stake(y, c, 100),
        Link::stake(c, v, 100),
    ];

    // Neutral karma: X and Y are symmetric, so they share focus.
    let g0 = FocusingGraph::build(links.clone(), &Context::none());
    let f0 = compute_focusing(&g0, &FocusingParams::default());
    let idx0 = |h: &[u8; 32]| g0.node_ids().iter().position(|n| n == h).unwrap();
    let gap = (f0.focus[idx0(&x)].to_f64() - f0.focus[idx0(&y)].to_f64()).abs();
    assert!(
        gap < 1e-6,
        "with neutral karma X and Y are symmetric (Δ={gap})"
    );

    // 3. With the earned karma, the honest neuron's target X outranks Y — honesty
    //    computed from reports flows all the way into collective focus.
    let g1 = FocusingGraph::build(links, &Context::with_karma(karma));
    let f1 = compute_focusing(&g1, &FocusingParams::default());
    let idx1 = |h: &[u8; 32]| g1.node_ids().iter().position(|n| n == h).unwrap();
    assert!(
        f1.focus[idx1(&x)] > f1.focus[idx1(&y)],
        "the honest neuron's target should draw more focus than the follower's"
    );
}
