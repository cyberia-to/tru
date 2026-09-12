#!/usr/bin/env python3
"""trusted spectrum for the eval_pussy_mix mirror.

the eval-side Rust subspace iteration collapses the singular tail (double
Gram-Schmidt on a full-width block drives trailing vectors into the null
space: sigma = [123, 13.7, 9.1, 0.48, 0.20, 0, 0.095, 0, 0, ...] with exact
zeros — lp_eval's scipy spectrum has a live tail at k=16). This script is
the reference SVD spine: it reads the graph + phi emitted by the Rust
mirror, builds the shipped mixed operator M = D (A + 0.5 A^2) D in the
same index space, runs scipy svds, writes U / sigma / V back.

usage: mirror_svd.py <graph.bin> <svd.bin>

graph.bin: u64 n, u64 nnz, nnz x (u32 i, u32 j, f64 w), n x f64 phi
svd.bin:   u64 k, k x f64 sigma, n*k f64 U row-major, n*k f64 V row-major
"""

import struct
import sys

import numpy as np
import scipy.sparse as sp
from scipy.sparse.linalg import svds


def read_graph(path):
    with open(path, "rb") as f:
        (n, nnz) = struct.unpack("<QQ", f.read(16))
        rec = np.frombuffer(f.read(nnz * 16), dtype=np.dtype([("i", "<u4"), ("j", "<u4"), ("w", "<f8")]))
        phi = np.frombuffer(f.read(8 * n), dtype="<f8").astype(np.float64)
    return n, rec["i"].astype(np.int64), rec["j"].astype(np.int64), rec["w"].astype(np.float64), phi


def main():
    gpath, spath = sys.argv[1], sys.argv[2]
    n, ii, jj, ww, phi = read_graph(gpath)
    a = sp.csr_matrix((ww, (ii, jj)), shape=(n, n))
    ds = np.sqrt(phi)
    m = sp.diags(ds) @ (a + 0.5 * (a @ a)) @ sp.diags(ds)
    k = min(64, n - 2)
    u, s, vt = svds(m.astype(np.float64), k=k, which="LM", maxiter=2000, tol=1e-10)
    order = np.argsort(-s)
    s = s[order]
    u = u[:, order]
    v = vt.T[:, order]
    print(f"mirror svd: sigma[:8]={np.array2string(s[:8], precision=4)}", file=sys.stderr)
    print(f"mirror svd: sigma[8:16]={np.array2string(s[8:16], precision=4)}", file=sys.stderr)
    with open(spath, "wb") as f:
        f.write(struct.pack("<Q", k))
        f.write(s.astype("<f8").tobytes())
        f.write(np.ascontiguousarray(u, dtype="<f8").tobytes())
        f.write(np.ascontiguousarray(v, dtype="<f8").tobytes())


if __name__ == "__main__":
    main()
