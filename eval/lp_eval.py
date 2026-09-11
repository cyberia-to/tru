#!/usr/bin/env python3
"""link prediction on a live cybergraph — space-pussy / bostrom data.

task: given links up to height T (90% of links by time), rank held-out links
(10%) above sampled non-edges. temporal split — the honest version: predict
what the graph will learn, not what it already knows.

models:
  pa        preferential attachment phi_i * phi_j        (tru baseline)
  aa        Adamic-Adar over directed out(i) x in(j)
  tru-E     <E_i, E_j>, E = U sqrt(Sigma) of diag(sqrt(phi)) A diag(sqrt(phi))  (current CT-0 pass 4)
  directed  <U_i sqrt(Sigma), V_j sqrt(Sigma)> on the same matrix (V side used)
  ppmi      directed factorization of PPMI(A)

phi* is computed with the preferential dangling fix (cyberia-to/tru#1):
dangling mass returns to the prior, not uniformly.

usage: lp_eval.py links.jsonl [--k 64] [--random-split]
"""

import argparse
import json

import numpy as np
import scipy.sparse as sp
from scipy.sparse.linalg import svds


def load(path):
    links = []
    with open(path) as f:
        for line in f:
            d = json.loads(line)
            links.append((d["h"], d["f"], d["t"]))
    return links


def pagerank(A, alpha=0.85, iters=200, tol=1e-12):
    """preferential PageRank: dangling mass returns to the prior."""
    n = A.shape[0]
    out = np.asarray(A.sum(axis=1)).ravel()
    dang = out == 0
    osafe = np.where(out == 0, 1.0, out)
    Dinv = sp.diags(1.0 / osafe)
    P = (Dinv @ A).T  # column-stochastic-ish; (P phi)_i = sum_j w_ji phi_j / out_j
    phi = np.full(n, 1.0 / n)
    for _ in range(iters):
        prev = phi.copy()
        dmass = phi[dang].sum()
        phi = alpha * (P @ phi + dmass * phi) + (1 - alpha) / n
        # renormalize (mass conservation guard)
        phi /= phi.sum()
        if np.abs(phi - prev).sum() < tol:
            break
    return phi


def ppmi(A):
    out = np.asarray(A.sum(axis=1)).ravel()
    inc = np.asarray(A.sum(axis=0)).ravel()
    total = A.sum()
    C = A.tocoo()
    denom = np.maximum(out[C.row] * inc[C.col], 1e-300)
    vals = np.log(C.data * total / denom)
    vals = np.maximum(vals, 0.0)
    return sp.coo_matrix((vals, (C.row, C.col)), shape=A.shape).tocsr()


def svd_embed(M, k):
    U, S, Vt = svds(M.astype(np.float64), k=k, which="LM", maxiter=1000)
    order = np.argsort(-S)
    S = S[order]
    U = U[:, order]
    V = Vt.T[:, order]
    return U * np.sqrt(S), V * np.sqrt(S), S


def auc(scores, labels):
    order = np.argsort(scores, kind="mergesort")
    ranks = np.empty(len(scores))
    ranks[order] = np.arange(1, len(scores) + 1)
    # average ranks for ties
    s_sorted = scores[order]
    i = 0
    while i < len(s_sorted):
        j = i
        while j + 1 < len(s_sorted) and s_sorted[j + 1] == s_sorted[i]:
            j += 1
        if j > i:
            ranks[order[i : j + 1]] = (i + 1 + j + 1) / 2
        i = j + 1
    pos = labels == 1
    npos, nneg = pos.sum(), (~pos).sum()
    return (ranks[pos].sum() - npos * (npos + 1) / 2) / (npos * nneg)


def average_precision(scores, labels):
    order = np.argsort(-scores)
    lab = labels[order]
    tp = np.cumsum(lab)
    prec = tp / np.arange(1, len(lab) + 1)
    return float((prec * lab).sum() / lab.sum())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("links")
    ap.add_argument("--k", type=int, default=64)
    ap.add_argument("--random-split", action="store_true")
    ap.add_argument("--neg-mult", type=int, default=5)
    args = ap.parse_args()

    rng = np.random.default_rng(7)
    links = load(args.links)
    heights = np.array([h for h, _, _ in links])
    order = np.argsort(heights, kind="stable")
    heights = heights[order]
    pairs = [(links[i][1], links[i][2]) for i in order]

    n = len(pairs)
    split = int(n * 0.9)
    if args.random_split:
        perm = rng.permutation(n)
        train_idx, test_idx = perm[:split], perm[split:]
    else:
        train_idx, test_idx = np.arange(split), np.arange(split, n)

    # particle index from TRAIN particles only
    pid = {}
    def intern(p):
        if p not in pid:
            pid[p] = len(pid)
        return pid[p]

    rows, cols, data = [], [], []
    cnt = {}
    for i in train_idx:
        a, b = pairs[i]
        ia, ib = intern(a), intern(b)
        key = (ia, ib)
        cnt[key] = cnt.get(key, 0) + 1
    for (ia, ib), c in cnt.items():
        rows.append(ia)
        cols.append(ib)
        data.append(float(c))
    nn = len(pid)
    A = sp.csr_matrix((data, (rows, cols)), shape=(nn, nn))
    print(f"train: {len(train_idx)} links · {A.nnz} unique pairs · {nn} particles")
    print(f"test:  {len(test_idx)} links (split={'random' if args.random_split else 'temporal by height'})")

    # ---- test pairs: both endpoints must be in train vocab ----
    full_pairs = set()
    for a, b in pairs:
        full_pairs.add((a, b))
    test_pairs, novel = [], 0
    for i in test_idx:
        a, b = pairs[i]
        if a in pid and b in pid:
            ia, ib = pid[a], pid[b]
            test_pairs.append((ia, ib))
            if (ia, ib) not in cnt and (ib, ia) not in cnt:
                novel += 1
    print(f"scorable test pairs: {len(test_pairs)} ({novel} novel in either direction)")

    # negatives: both endpoints in train vocab, strict non-edge in FULL graph
    inv = [None] * nn
    for p, i in pid.items():
        inv[i] = p
    negs = set()
    while len(negs) < len(test_pairs) * args.neg_mult:
        i = int(rng.integers(0, nn))
        j = int(rng.integers(0, nn))
        if i == j:
            continue
        if (inv[i], inv[j]) in full_pairs or (inv[j], inv[i]) in full_pairs:
            continue
        if (i, j) in cnt:
            continue
        negs.add((i, j))
    negs = list(negs)

    # ---- models ----
    phi = pagerank(A)
    out_deg = np.asarray(A.sum(axis=1)).ravel()
    in_deg = np.asarray(A.sum(axis=0)).ravel()

    sq = np.sqrt(phi)
    M_tru = sp.diags(sq) @ A @ sp.diags(sq)
    E_tru, _, _ = svd_embed(M_tru, args.k)
    E_src, E_tgt, S = svd_embed(M_tru, args.k)
    P = ppmi(A)
    P_src, P_tgt, _ = svd_embed(P, args.k)
    print(f"svd k={args.k} · sigma_top={S[0]:.4f} · sigma_k={S[-1]:.4f}")

    At = A.T.tocsr()
    def aa_score(i, j):
        # sum over z in out(i) ∩ in(j) of 1/log deg(z), deg = out+in strength
        nz = set(A.indices[A.indptr[i] : A.indptr[i + 1]])
        cand = At.indices[At.indptr[j] : At.indptr[j + 1]]
        s = 0.0
        for z in cand:
            if z in nz:
                dz = out_deg[z] + in_deg[z]
                if dz > 1:
                    s += 1.0 / np.log(dz)
        return s

    models = {
        "pa (phi_i*phi_j)": lambda i, j: phi[i] * phi[j],
        "tru-E (sym)": lambda i, j: float(E_tru[i] @ E_tru[j]),
        "directed (U,V)": lambda i, j: float(E_src[i] @ E_tgt[j]),
        "ppmi (U,V)": lambda i, j: float(P_src[i] @ P_tgt[j]),
    }

    for name, f in models.items():
        pos_s = np.array([f(i, j) for i, j in test_pairs])
        neg_s = np.array([f(i, j) for i, j in negs])
        scores = np.concatenate([pos_s, neg_s])
        labels = np.concatenate([np.ones(len(pos_s)), np.zeros(len(neg_s))])
        a = auc(scores, labels)
        ap_ = average_precision(scores, labels)
        # novel-only slice
        nov_pairs = [(i, j) for (i, j) in test_pairs
                     if (i, j) not in cnt and (j, i) not in cnt]
        if nov_pairs:
            ns = int(min(len(negs), len(nov_pairs) * args.neg_mult))
            ps = np.array([f(i, j) for i, j in nov_pairs])
            ss = np.concatenate([ps, neg_s[:ns]])
            ll = np.concatenate([np.ones(len(ps)), np.zeros(ns)])
            an = auc(ss, ll)
        else:
            an = float("nan")
        print(f"{name:22} AUC {a:.4f} · AP {ap_:.4f} · novel-AUC {an:.4f} (n={len(nov_pairs)})")

    # hybrids: does spectral carry signal beyond popularity?
    def zsc(x):
        s = x.std()
        return (x - x.mean()) / (s if s > 0 else 1.0)
    for k2 in [16, args.k]:
        P2 = ppmi(A)
        Ps2, Pt2, _ = svd_embed(P2, k2)
        pos_pa = np.array([phi[i] * phi[j] for i, j in test_pairs])
        neg_pa = np.array([phi[i] * phi[j] for i, j in negs])
        pos_sp = np.array([float(Ps2[i] @ Pt2[j]) for i, j in test_pairs])
        neg_sp = np.array([float(Ps2[i] @ Pt2[j]) for i, j in negs])
        for hname, hs in [
            ("pa*ppmi", (pos_pa * pos_sp, neg_pa * neg_sp)),
            ("pa+ppmi(z)", (zsc(pos_pa) + zsc(pos_sp), zsc(neg_pa) + zsc(neg_sp))),
        ]:
            scores = np.concatenate(hs)
            labels = np.concatenate([np.ones(len(hs[0])), np.zeros(len(hs[1]))])
            print(f"hybrid k={k2:<3} {hname:12} AUC {auc(scores, labels):.4f} · AP {average_precision(scores, labels):.4f}")

    # ---- 2-hop mixing experiment (tru#2): M <- diag(sq) (A + gamma A^2) diag(sq)
    # fine sibling structure lives at graph distance 2; the 1-step matrix
    # gives structurally equivalent tokens parallel embeddings.
    A2 = (A @ A).tocsr()
    for gamma in [0.25, 0.5, 1.0]:
        M2 = sp.diags(sq) @ (A + gamma * A2) @ sp.diags(sq)
        E2s, E2t, _ = svd_embed(M2, args.k)
        for name, f in {
            f"tru-E2 g={gamma} (sym)": lambda i, j: float(E2s[i] @ E2s[j]),
            f"directed2 g={gamma}": lambda i, j: float(E2s[i] @ E2t[j]),
        }.items():
            pos_s = np.array([f(i, j) for i, j in test_pairs])
            neg_s = np.array([f(i, j) for i, j in negs])
            scores = np.concatenate([pos_s, neg_s])
            labels = np.concatenate([np.ones(len(pos_s)), np.zeros(len(neg_s))])
            a = auc(scores, labels)
            ap_ = average_precision(scores, labels)
            nov_pairs = [(i, j) for (i, j) in test_pairs
                         if (i, j) not in cnt and (j, i) not in cnt]
            if nov_pairs:
                ns = int(min(len(negs), len(nov_pairs) * args.neg_mult))
                ps = np.array([f(i, j) for i, j in nov_pairs])
                ss = np.concatenate([ps, neg_s[:ns]])
                ll = np.concatenate([np.ones(len(ps)), np.zeros(ns)])
                an = auc(ss, ll)
            else:
                an = float("nan")
            print(f"{name:22} AUC {a:.4f} · AP {ap_:.4f} · novel-AUC {an:.4f} (n={len(nov_pairs)})")

    # AA is slow in pure python; run on a subsample for a reference point
    sub = test_pairs[: min(800, len(test_pairs))]
    pos_s = np.array([aa_score(i, j) for i, j in sub])
    ns = len(sub) * args.neg_mult
    neg_s = np.array([aa_score(i, j) for i, j in negs[:ns]])
    scores = np.concatenate([pos_s, neg_s])
    labels = np.concatenate([np.ones(len(pos_s)), np.zeros(len(neg_s))])
    print(f"{'adamic-adar (sub)':22} AUC {auc(scores, labels):.4f} · AP {average_precision(scores, labels):.4f} (n={len(sub)})")


if __name__ == "__main__":
    main()
