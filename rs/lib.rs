//! mc — model compilation.
//!
//! Reference implementation of the [CT-0 spec](https://cyber.page/compiled-transformers-spec):
//! read a `.graph` cybergraph snapshot, produce a `.model` transformer
//! checkpoint per the [cyb-model](https://cyber.page/cyb-model) format.
//!
//! ```ignore
//! use tru::Graph;
//!
//! let g = Graph::open("bostrom-23195000.graph")?;
//! println!("snapshot {}: {} cyberlinks", g.name(), g.cyberlinks()?.count());
//! ```

pub mod arithmetic;
pub mod error;
pub mod focusing;
pub mod graph;
pub mod model;
pub mod pass;
pub mod rewards;
pub mod truth_scoring;
pub mod vocab;

pub use arithmetic::Fx;
pub use error::{McError, Result};
pub use focusing::{compute_focusing, impulse, Context, FocusingGraph, FocusingParams, FocusingResult, Impulse, Karma, Link, Will};
pub use graph::{Cyberlink, Graph};
pub use model::Model;
pub use rewards::{shapley, value, Contribution};
pub use truth_scoring::{accumulate, bts_scores, surprise, Report};
