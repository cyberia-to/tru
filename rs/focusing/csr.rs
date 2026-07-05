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
