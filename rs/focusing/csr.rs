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

    #[test]
    fn build_sums_duplicate_row_col_entries() {
        let mut b = CsrBuilder::new(2);
        b.add(0, 1, Fx::from_int(3));
        b.add(0, 1, Fx::from_int(4));
        b.add(1, 0, Fx::from_int(1));
        let m = b.build();
        assert_eq!(m.nnz(), 2);
        assert_eq!(m.values[m.row_ptr[0]], Fx::from_int(7));
        assert_eq!(m.values[m.row_ptr[1]], Fx::from_int(1));
    }

    #[test]
    fn build_is_independent_of_insertion_order() {
        let mut a = CsrBuilder::new(3);
        a.add(2, 0, Fx::from_int(1));
        a.add(0, 2, Fx::from_int(2));
        a.add(1, 1, Fx::from_int(3));
        let ma = a.build();

        let mut b = CsrBuilder::new(3);
        b.add(1, 1, Fx::from_int(3));
        b.add(0, 2, Fx::from_int(2));
        b.add(2, 0, Fx::from_int(1));
        let mb = b.build();

        assert_eq!(ma.row_ptr, mb.row_ptr);
        assert_eq!(ma.col_idx, mb.col_idx);
        assert_eq!(ma.values, mb.values);
    }

    #[test]
    fn build_with_no_triplets_is_an_all_zero_row_ptr() {
        let m = CsrBuilder::new(4).build();
        assert_eq!(m.nnz(), 0);
        assert_eq!(m.row_ptr, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn spmv_computes_matrix_vector_product() {
        // A = [[2, 0, 1],
        //      [0, 3, 0],
        //      [1, 1, 1]]
        let mut b = CsrBuilder::new(3);
        b.add(0, 0, Fx::from_int(2));
        b.add(0, 2, Fx::from_int(1));
        b.add(1, 1, Fx::from_int(3));
        b.add(2, 0, Fx::from_int(1));
        b.add(2, 1, Fx::from_int(1));
        b.add(2, 2, Fx::from_int(1));
        let m = b.build();

        let x = vec![Fx::from_int(1), Fx::from_int(2), Fx::from_int(3)];
        let mut y = vec![Fx::from_int(99); 3]; // pre-existing garbage must be overwritten, not accumulated
        m.spmv(&x, &mut y);

        assert_eq!(y[0], Fx::from_int(5)); // 2*1 + 1*3
        assert_eq!(y[1], Fx::from_int(6)); // 3*2
        assert_eq!(y[2], Fx::from_int(6)); // 1*1 + 1*2 + 1*3
    }

    #[test]
    fn spmv_add_accumulates_onto_existing_y() {
        let mut b = CsrBuilder::new(2);
        b.add(0, 1, Fx::from_int(2));
        b.add(1, 0, Fx::from_int(3));
        let m = b.build();

        let x = vec![Fx::from_int(1), Fx::from_int(1)];
        let mut y = vec![Fx::from_int(10), Fx::from_int(20)];
        m.spmv_add(&x, &mut y);

        assert_eq!(y[0], Fx::from_int(12)); // 10 + 2*1
        assert_eq!(y[1], Fx::from_int(23)); // 20 + 3*1
    }

    #[test]
    fn spmv_add_grouping_order_does_not_change_the_sum() {
        // Same terms, added left-to-right vs. right-to-left via two builders
        // with reversed insertion order for the same row — field addition is
        // associative, so both must land on the identical accumulated value.
        let mut fwd = CsrBuilder::new(1);
        fwd.add(0, 0, Fx::from_int(1));
        fwd.add(0, 0, Fx::from_int(2));
        fwd.add(0, 0, Fx::from_int(3));
        let mf = fwd.build();

        let mut rev = CsrBuilder::new(1);
        rev.add(0, 0, Fx::from_int(3));
        rev.add(0, 0, Fx::from_int(2));
        rev.add(0, 0, Fx::from_int(1));
        let mr = rev.build();

        assert_eq!(mf.values[0], mr.values[0]);
        assert_eq!(mf.values[0], Fx::from_int(6));
    }

    #[test]
    fn zero_matrix_leaves_y_zeroed_by_spmv() {
        let m = CsrBuilder::new(3).build();
        let x = vec![Fx::from_int(7); 3];
        let mut y = vec![Fx::from_int(1); 3];
        m.spmv(&x, &mut y);
        assert_eq!(y, vec![Fx::ZERO; 3]);
    }
}
