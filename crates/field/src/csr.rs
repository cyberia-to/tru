/// Compressed Sparse Row matrix (square, n×n).
pub struct CsrMatrix {
    pub n: usize,
    pub row_ptr: Vec<usize>,
    pub col_idx: Vec<usize>,
    pub values: Vec<f64>,
}

impl CsrMatrix {
    pub fn nnz(&self) -> usize {
        self.values.len()
    }

    /// y = A·x  (accumulating, does NOT zero y first)
    pub fn spmv_add(&self, x: &[f64], y: &mut [f64]) {
        for i in 0..self.n {
            let mut acc = 0.0;
            for k in self.row_ptr[i]..self.row_ptr[i + 1] {
                acc += self.values[k] * x[self.col_idx[k]];
            }
            y[i] += acc;
        }
    }

    /// y = A·x
    pub fn spmv(&self, x: &[f64], y: &mut [f64]) {
        y[..self.n].iter_mut().for_each(|v| *v = 0.0);
        self.spmv_add(x, y);
    }
}

/// Build a CSR from (row, col, value) triplets.
/// Duplicate (row, col) entries are summed.
pub struct CsrBuilder {
    n: usize,
    triplets: Vec<(usize, usize, f64)>,
}

impl CsrBuilder {
    pub fn new(n: usize) -> Self {
        Self { n, triplets: Vec::new() }
    }

    pub fn add(&mut self, row: usize, col: usize, val: f64) {
        debug_assert!(row < self.n && col < self.n);
        self.triplets.push((row, col, val));
    }

    pub fn build(mut self) -> CsrMatrix {
        self.triplets.sort_unstable_by_key(|&(r, c, _)| (r, c));

        let mut deduped: Vec<(usize, usize, f64)> = Vec::with_capacity(self.triplets.len());
        for (r, c, v) in self.triplets {
            match deduped.last_mut() {
                Some(last) if last.0 == r && last.1 == c => last.2 += v,
                _ => deduped.push((r, c, v)),
            }
        }

        let nnz = deduped.len();
        let mut row_ptr = vec![0usize; self.n + 1];
        let mut col_idx = vec![0usize; nnz];
        let mut values = vec![0.0f64; nnz];

        for (k, &(r, c, v)) in deduped.iter().enumerate() {
            row_ptr[r + 1] += 1;
            col_idx[k] = c;
            values[k] = v;
        }
        for i in 0..self.n {
            row_ptr[i + 1] += row_ptr[i];
        }

        CsrMatrix { n: self.n, row_ptr, col_idx, values }
    }
}
