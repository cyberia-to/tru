//! Operation counters for the epoch kernel (feature `opcount`, diagnostic only).
//!
//! Every arithmetic primitive of [`super::fixed`] and [`super::kernel`] bumps a
//! relaxed atomic; the sums are exact integers whatever the thread count.
//! Without the feature the `count!` macro expands to nothing.

#![allow(non_upper_case_globals)]

use std::sync::atomic::{AtomicU64, Ordering};

macro_rules! counters {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: AtomicU64 = AtomicU64::new(0);)*

        /// A snapshot of every counter.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct Counts {
            $(pub $name: u64,)*
        }

        impl Counts {
            /// `(name, value)` pairs in declaration order.
            pub fn fields(&self) -> Vec<(&'static str, u64)> {
                vec![$((stringify!($name), self.$name),)*]
            }
        }

        /// Zero every counter.
        pub fn reset() {
            $($name.store(0, Ordering::Relaxed);)*
        }

        /// Read every counter.
        pub fn snapshot() -> Counts {
            Counts { $($name: $name.load(Ordering::Relaxed),)* }
        }
    };
}

counters!(
    // fixed.rs primitives
    mul_shr,
    shr_u,
    shr_i,
    div_round_i,
    mul_shift_div,
    mul_wide_shr,
    mul_wide_floor,
    ln_q40,
    ln2_q40,
    // kernel.rs wrappers (each is one u128/i128 multiply plus one shr_u/shr_i)
    qmul,
    qmul_i,
    // 128-bit divisions outside fixed.rs (checked_div, i128 `/`)
    wide_div,
    // structural work: rows visited and CSR entries visited, per kind
    sym_fwd_rows,
    sym_fwd_entries,
    sym_adj_rows,
    sym_adj_entries,
    dir_fwd_rows,
    dir_fwd_entries,
    dir_adj_rows,
    dir_adj_entries,
    row_value,
    forward_step,
    adjoint_step,
    // gradient.rs: edge_gradient
    grad_edges,
    grad_q,
    grad_heat_calls,
    grad_heat_mul,
);
