//! Pass 6 — Clifford-block MLP weights (`specs/ct0.md` §8).
//!
//! The MLP weights are initialized, not factorized: the compile emits a
//! deterministic seeded initialization (§8.2) that the runtime uses as-is (the
//! Clifford block reaches SwiGLU capability at init depth). Per layer it emits
//! the geometric-product projection, the gate, the LayerScale γ, and the two
//! context DWConv/graph-conv kernels.
//!
//! Determinism comes from a hemera-seeded PRNG keyed by `(salt, layer, "CT-0")`
//! — the spec names ChaCha20; a hemera counter stream serves the same role and
//! is byte-identical within this implementation (cross-impl parity with the
//! exact ChaCha seeding is a conformance refinement).

use crate::arithmetic::Fx;
use crate::model::{Encoding, Tensor};

const SHIFT_SET_LEN: usize = 5; // |S| (§8 default shift set {1,2,4,8,16})

/// Deterministic uniform stream seeded from `hemera(salt ‖ layer ‖ "CT-0")`.
/// Each digest yields four u64 draws, so one hash serves four weights.
struct Rng {
    seed: [u8; 32],
    ctr: u64,
    buf: [u64; 4],
    have: usize,
}

impl Rng {
    fn new(salt: &str, layer: usize) -> Self {
        let mut b = Vec::new();
        b.extend_from_slice(salt.as_bytes());
        b.extend_from_slice(&(layer as u64).to_le_bytes());
        b.extend_from_slice(b"CT-0");
        let mut seed = [0u8; 32];
        seed.copy_from_slice(cyber_hemera::hash(&b).as_bytes());
        Rng { seed, ctr: 0, buf: [0; 4], have: 0 }
    }

    fn next_u64(&mut self) -> u64 {
        if self.have == 0 {
            let mut b = [0u8; 40];
            b[..32].copy_from_slice(&self.seed);
            b[32..].copy_from_slice(&self.ctr.to_le_bytes());
            self.ctr += 1;
            let h = cyber_hemera::hash(&b);
            let d = h.as_bytes();
            for (i, slot) in self.buf.iter_mut().enumerate() {
                *slot = u64::from_le_bytes(d[i * 8..i * 8 + 8].try_into().unwrap());
            }
            self.have = 4;
        }
        self.have -= 1;
        self.buf[self.have]
    }

    /// Next uniform in `[-1, 1)`.
    fn unit(&mut self) -> Fx {
        let frac = Fx::ratio_u128(self.next_u64() as u128, u64::MAX as u128); // [0,1]
        Fx::from_int(2) * frac - Fx::ONE
    }

    /// He-uniform sample: uniform in `[-limit, limit]`, `limit = √(6/fan_in)`.
    fn he(&mut self, fan_in: usize) -> Fx {
        let limit = Fx::from_ratio(6, fan_in.max(1) as i64).sqrt();
        self.unit() * limit
    }
}

fn seeded(name: String, shape: Vec<u64>, count: usize, fan_in: usize, rng: &mut Rng) -> Tensor {
    let data = (0..count).map(|_| rng.he(fan_in)).collect();
    Tensor { name, shape, encoding: Encoding::U16, data }
}

/// Pass 6: the Clifford-MLP tensors for every layer (§8.1).
pub fn mlp(d: usize, l: usize) -> Vec<Tensor> {
    let mut tensors = Vec::new();
    for layer in 0..l {
        let mut rng = Rng::new("mlp_clifford", layer);

        // proj: (|S|·2d, d) geometric-product projection. gate: (2d, d).
        tensors.push(seeded(
            format!("model.layers.{layer}.mlp_clifford.proj.weight"),
            vec![(SHIFT_SET_LEN * 2 * d) as u64, d as u64],
            SHIFT_SET_LEN * 2 * d * d,
            d,
            &mut rng,
        ));
        tensors.push(seeded(
            format!("model.layers.{layer}.mlp_clifford.gate.weight"),
            vec![(2 * d) as u64, d as u64],
            2 * d * d,
            d,
            &mut rng,
        ));

        // γ LayerScale: (d,), the fixed-point element nearest 1e-5, as u32.
        tensors.push(Tensor {
            name: format!("model.layers.{layer}.mlp_clifford.gamma"),
            shape: vec![d as u64],
            encoding: Encoding::U32,
            data: vec![Fx::from_ratio(1, 100_000); d],
        });

        // Two context kernels: (d, 3, 3) each, seeded (salt "mlp_clifford").
        for which in 1..=2 {
            tensors.push(seeded(
                format!("model.layers.{layer}.mlp_clifford.context.weight_{which}"),
                vec![d as u64, 3, 3],
                d * 9,
                9,
                &mut rng,
            ));
        }
    }
    tensors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_five_tensors_per_layer_with_right_shapes() {
        let (d, l) = (16, 3);
        let t = mlp(d, l);
        assert_eq!(t.len(), 5 * l);
        let proj = t.iter().find(|t| t.name == "model.layers.0.mlp_clifford.proj.weight").unwrap();
        assert_eq!(proj.shape, vec![(SHIFT_SET_LEN * 2 * d) as u64, d as u64]);
        assert_eq!(proj.data.len(), SHIFT_SET_LEN * 2 * d * d);
        let gamma = t.iter().find(|t| t.name == "model.layers.0.mlp_clifford.gamma").unwrap();
        assert_eq!(gamma.shape, vec![d as u64]);
        assert_eq!(gamma.encoding, Encoding::U32);
        let ctx = t.iter().find(|t| t.name == "model.layers.2.mlp_clifford.context.weight_2").unwrap();
        assert_eq!(ctx.shape, vec![d as u64, 3, 3]);
    }

    #[test]
    fn init_is_deterministic() {
        let a = mlp(16, 2);
        let b = mlp(16, 2);
        assert!(a.iter().zip(&b).all(|(x, y)| x.data.iter().zip(&y.data).all(|(p, q)| p.raw() == q.raw())));
    }

    #[test]
    fn layers_get_distinct_seeds() {
        let t = mlp(16, 2);
        let p0 = &t.iter().find(|t| t.name == "model.layers.0.mlp_clifford.proj.weight").unwrap().data;
        let p1 = &t.iter().find(|t| t.name == "model.layers.1.mlp_clifford.proj.weight").unwrap().data;
        assert!(p0.iter().zip(p1).any(|(a, b)| a.raw() != b.raw()), "layers must not share weights");
    }
}
