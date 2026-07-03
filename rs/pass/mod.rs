//! The CT-0 compile — eight passes from `.graph` to `.model` (`specs/ct0.md`).
//!
//! - phase 1: [`index`] (pass 1), [`dialect`] (pass 2), arch (pass 3)
//! - phase 2: embed (pass 4), attn (pass 5)
//! - phase 3: mlp (pass 6), norm (pass 7), pack (pass 8)
//!
//! Structural passes (index, dialect, arch, norm, pack) are implemented; the
//! SVD-derived weight passes (embed, attn, mlp) land with the fixed-point
//! randomized-SVD milestone.

pub mod arch;
pub mod dialect;
pub mod index;

pub use arch::Arch;
pub use dialect::{Dialects, BOTTOM};
pub use index::{axon, effective_stake, Adjacency, Edge, ParticleIndex};
