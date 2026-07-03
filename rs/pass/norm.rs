//! Pass 7 — norms and position (`specs/ct0.md` §9).
//!
//! Layer norms are identity (all ones); RoPE stores no tensor (inverse
//! frequencies are recomputed at load from θ₀ and head_dim); the output head is
//! tied to the embedding. So this pass emits only the layernorm weights.

use crate::arithmetic::Fx;
use crate::model::{Encoding, Tensor};

/// The layernorm tensors for a `d*`-wide, `L*`-layer model (§9.1): per-layer
/// input and post-attention norms plus the final `model.norm`, all ones, stored
/// as `u32` (§10.5).
pub fn layernorms(d: usize, l: usize) -> Vec<Tensor> {
    let ones = || vec![Fx::ONE; d];
    let mut tensors = Vec::with_capacity(2 * l + 1);
    for layer in 0..l {
        tensors.push(Tensor {
            name: format!("model.layers.{layer}.input_layernorm.weight"),
            shape: vec![d as u64],
            encoding: Encoding::U32,
            data: ones(),
        });
        tensors.push(Tensor {
            name: format!("model.layers.{layer}.post_attention_layernorm.weight"),
            shape: vec![d as u64],
            encoding: Encoding::U32,
            data: ones(),
        });
    }
    tensors.push(Tensor {
        name: "model.norm.weight".to_string(),
        shape: vec![d as u64],
        encoding: Encoding::U32,
        data: ones(),
    });
    tensors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_two_per_layer_plus_the_final_norm() {
        let t = layernorms(64, 4);
        assert_eq!(t.len(), 2 * 4 + 1);
        assert!(t.iter().all(|t| t.shape == vec![64] && t.data.iter().all(|&x| x.raw() == Fx::ONE.raw())));
        assert!(t.iter().any(|t| t.name == "model.norm.weight"));
        assert!(t.iter().any(|t| t.name == "model.layers.3.post_attention_layernorm.weight"));
    }
}
