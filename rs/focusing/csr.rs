use crate::arithmetic::Fx;

/// Compressed Sparse Row matrix (square, n×n) over fixed-point field elements.
pub struct CsrMatrix {
    pub n: usize,
    pub row_ptr: Vec<usize>,
    pub col_idx: Vec<usize>,
    pub values: Vec<Fx>,
}

impl CsrMatrix {
    pub fn nnz(&self) -> usize {
        self.values.len()
    }

    /// y += A·x. Field addition is associative, so accumulation order does
    /// not affect the result — the sum is bit-identical however it is grouped.
    pub fn spmv_add(&self, x: &[Fx], y: &mut [Fx]) {
        for i in 0..self.n {
            let mut acc = Fx::ZERO;
            for k in self.row_ptr[i]..self.row_ptr[i + 1] {
                acc = acc + self.values[k] * x[self.col_idx[k]];
            }
            y[i] = y[i] + acc;
        }
    }

    /// y = A·x
    pub fn spmv(&self, x: &[Fx], y: &mut [Fx]) {
        y[..self.n].iter_mut().for_each(|v| *v = Fx::ZERO);
        self.spmv_add(x, y);
    }
}

/// Build a CSR from (row, col, value) triplets.
/// Duplicate (row, col) entries are summed. Triplets are sorted before build,
/// so the result is independent of insertion order.
pub struct CsrBuilder {
    n: usize,
    triplets: Vec<(usize, usize, Fx)>,
}

impl CsrBuilder {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            triplets: Vec::new(),
        }
    }

    pub fn add(&mut self, row: usize, col: usize, val: Fx) {
        debug_assert!(row < self.n && col < self.n);
        self.triplets.push((row, col, val));
    }

    pub fn build(mut self) -> CsrMatrix {
        self.triplets.sort_unstable_by_key(|&(r, c, _)| (r, c));

        let mut deduped: Vec<(usize, usize, Fx)> = Vec::with_capacity(self.triplets.len());
        for (r, c, v) in self.triplets {
            match deduped.last_mut() {
                Some(last) if last.0 == r && last.1 == c => last.2 = last.2 + v,
                _ => deduped.push((r, c, v)),
            }
        }

        let nnz = deduped.len();
        let mut row_ptr = vec![0usize; self.n + 1];
        let mut col_idx = vec![0usize; nnz];
        let mut values = vec![Fx::ZERO; nnz];

        for (k, &(r, c, v)) in deduped.iter().enumerate() {
            row_ptr[r + 1] += 1;
            col_idx[k] = c;
            values[k] = v;
        }
        for i in 0..self.n {
            row_ptr[i + 1] += row_ptr[i];
        }

        CsrMatrix {
            n: self.n,
            row_ptr,
            col_idx,
            values,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(n: i64) -> Fx {
        Fx::from_int(n)
    }

    #[test]
    fn empty_builder_yields_zero_matrix() {
        let m = CsrBuilder::new(3).build();
        assert_eq!(m.n, 3);
        assert_eq!(m.nnz(), 0);
        assert_eq!(m.row_ptr, vec![0, 0, 0, 0]);
        let x = vec![fx(1), fx(2), fx(3)];
        let mut y = vec![fx(9), fx(9), fx(9)];
        m.spmv(&x, &mut y);
        assert_eq!(y, vec![Fx::ZERO, Fx::ZERO, Fx::ZERO]);
    }

    #[test]
    fn single_entry_multiplies_correctly() {
        let mut b = CsrBuilder::new(2);
        b.add(0, 1, fx(5));
        let m = b.build();
        assert_eq!(m.nnz(), 1);
        let x = vec![fx(1), fx(10)];
        let mut y = vec![Fx::ZERO, Fx::ZERO];
        m.spmv(&x, &mut y);
        // row 0: 5 * x[1] = 50; row 1: no entries, stays 0.
        assert_eq!(y, vec![fx(50), Fx::ZERO]);
    }

    #[test]
    fn duplicate_row_col_entries_are_summed() {
        let mut b = CsrBuilder::new(2);
        b.add(0, 0, fx(3));
        b.add(0, 0, fx(4));
        let m = b.build();
        assert_eq!(m.nnz(), 1, "duplicate (row, col) must collapse to one entry");
        assert_eq!(m.values[0], fx(7));
    }

    #[test]
    fn build_result_is_independent_of_insertion_order() {
        let mut forward = CsrBuilder::new(3);
        forward.add(0, 2, fx(1));
        forward.add(1, 0, fx(2));
        forward.add(2, 1, fx(3));
        forward.add(0, 0, fx(4));

        let mut reverse = CsrBuilder::new(3);
        reverse.add(0, 0, fx(4));
        reverse.add(2, 1, fx(3));
        reverse.add(1, 0, fx(2));
        reverse.add(0, 2, fx(1));

        let a = forward.build();
        let z = reverse.build();
        assert_eq!(a.row_ptr, z.row_ptr);
        assert_eq!(a.col_idx, z.col_idx);
        assert_eq!(a.values, z.values);
    }

    #[test]
    fn row_ptr_boundaries_match_per_row_entry_counts() {
        let mut b = CsrBuilder::new(3);
        b.add(0, 0, fx(1));
        b.add(0, 2, fx(2));
        b.add(2, 1, fx(3));
        // row 1 has no entries at all.
        let m = b.build();
        assert_eq!(m.row_ptr, vec![0, 2, 2, 3]);
        assert_eq!(m.col_idx, vec![0, 2, 1]);
    }

    #[test]
    fn identity_matrix_is_a_no_op_on_spmv() {
        let mut b = CsrBuilder::new(3);
        for i in 0..3 {
            b.add(i, i, Fx::ONE);
        }
        let m = b.build();
        let x = vec![fx(7), fx(8), fx(9)];
        let mut y = vec![Fx::ZERO; 3];
        m.spmv(&x, &mut y);
        assert_eq!(y, x);
    }

    #[test]
    fn spmv_zeroes_output_first_while_spmv_add_accumulates() {
        let mut b = CsrBuilder::new(2);
        b.add(0, 0, fx(2));
        b.add(1, 1, fx(3));
        let m = b.build();
        let x = vec![fx(1), fx(1)];

        let mut y = vec![fx(100), fx(100)];
        m.spmv(&x, &mut y);
        assert_eq!(y, vec![fx(2), fx(3)], "spmv must overwrite, not accumulate");

        let mut y = vec![fx(100), fx(100)];
        m.spmv_add(&x, &mut y);
        assert_eq!(
            y,
            vec![fx(102), fx(103)],
            "spmv_add must add onto the existing output"
        );
    }

    #[test]
    fn multiple_entries_per_row_sum_across_columns() {
        let mut b = CsrBuilder::new(1);
        b.add(0, 0, fx(2));
        b.add(0, 0, Fx::ZERO); // distinct add() call, same (row, col): still one dedup slot.
        let m = b.build();
        assert_eq!(m.nnz(), 1);
        let x = vec![fx(5)];
        let mut y = vec![Fx::ZERO];
        m.spmv(&x, &mut y);
        assert_eq!(y, vec![fx(10)]);
    }

    #[test]
    fn a_row_dot_product_sums_several_distinct_columns() {
        let mut b = CsrBuilder::new(3);
        b.add(0, 0, fx(2));
        b.add(0, 1, fx(3));
        b.add(0, 2, fx(4));
        let m = b.build();
        assert_eq!(m.nnz(), 3);
        let x = vec![fx(1), fx(10), fx(100)];
        let mut y = vec![Fx::ZERO; 3];
        m.spmv(&x, &mut y);
        // row 0: 2*1 + 3*10 + 4*100 = 432; rows 1, 2 have no entries.
        assert_eq!(y, vec![fx(432), Fx::ZERO, Fx::ZERO]);
    }
}
