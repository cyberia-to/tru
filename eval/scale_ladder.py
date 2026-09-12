#!/usr/bin/env python3
"""the scale ladder — at what graph size does trainable generativity emerge?

doctrine (black box): no content, topology only. the agent's head IS its
graph; semantics migrate into topology by stake. the open question: at
what size does training on structure start generalizing to UNSEEN links
(eval/e2e_pussy.md, ft_ablation.md showed it does NOT at 32k nodes)?

design: temporal prefixes of bostrom (natural growth curve), rungs at
~100k / 500k / 1M / 3.1M train particles. per rung, identical protocol:
  1. temporal 90/10 split inside the prefix
  2. phi* (preferential pagerank, tru#1 fix)
  3. exact spectrum of the shipped mixed operator M = D(A + 0.5 A^2) D
     via scipy svds on a LinearOperator (A^2 never materialized)
  4. E = U sqrt(Sigma), d = 64 (fixed across rungs for comparability)
  5. probes on the same held-out walk-continuation queries:
     - floor: compiled E, no training
     - compiled E + BPR metric training (E-only: score = rmsnorm(E[f]).E[t])
     - random E + same training (the baseline the ladder is about)
     plus the add-k bigram floor for reference.

BPR touches only E[f], E[t+], E[t-] per pair — sparse gradients, no
full-vocab projection in training; eval ranks the full vocab once per
checkpoint. if a rung's trained curve crosses its floor, generativity
emerged at that scale.

usage: .venv/bin/python eval/scale_ladder.py [--rungs 100000 500000 1000000 3100000] [--steps 2000]
"""

import argparse
import json
import time

import numpy as np
import scipy.sparse as sp
from scipy.sparse.linalg import LinearOperator, svds

DATA = "eval/data/bostrom_links.jsonl"
GAMMA = 0.5
D = 64
K_SM = 0.1


def load_links():
    links = []
    with open(DATA) as f:
        for line in f:
            d = json.loads(line)
            links.append((d["h"], d["f"], d["t"]))
    links.sort(key=lambda x: x[0])
    return links


def pagerank(A, alpha=0.85, iters=300, tol=1e-12):
    n = A.shape[0]
    out = np.asarray(A.sum(axis=1)).ravel()
    dang = out == 0
    osafe = np.where(out == 0, 1.0, out)
    P = (sp.diags(1.0 / osafe) @ A).T
    phi = np.full(n, 1.0 / n)
    for _ in range(iters):
        prev = phi.copy()
        dmass = phi[dang].sum()
        phi = alpha * (P @ phi + dmass * phi) + (1 - alpha) / n
        phi /= phi.sum()
        if np.abs(phi - prev).sum() < tol:
            break
    return phi


def mixed_svd(A, phi, k):
    n = A.shape[0]
    ds = np.sqrt(phi)
    At = A.T.tocsr()

    def mv(x):
        t = ds * x
        ax = A @ t
        a2x = A @ ax
        return ds * (ax + GAMMA * a2x)

    def mvt(x):
        t = ds * x
        atx = At @ t
        at2x = At @ atx
        return ds * (atx + GAMMA * at2x)

    op = LinearOperator((n, n), matvec=mv, rmatvec=mvt, dtype=np.float64)
    u, s, vt = svds(op, k=k, which="LM", maxiter=500, tol=1e-8)
    order = np.argsort(-s)
    return u[:, order], s[order], vt.T[:, order]


def rms(x, eps=1e-5):
    return x / np.sqrt((x * x).mean() + eps)


def mrr_queries(E, queries):
    r = 0.0
    for seq, gold in queries:
        f = seq[-1]
        s = E @ rms(E[f])
        r += 1.0 / (1 + int(np.sum(s > s[gold])))
    return r / len(queries)


def bigram_mrr(links_train, queries, n):
    from collections import defaultdict
    cnt = defaultdict(int)
    outc = defaultdict(int)
    for f, t in links_train:
        cnt[(f, t)] += 1
        outc[f] += 1
    r = 0.0
    for seq, gold in queries:
        f = seq[-1]
        s = np.array([cnt.get((f, j), 0) + K_SM for j in range(n)])
        s /= outc.get(f, 0) + K_SM * n
        r += 1.0 / (1 + int(np.sum(s > s[gold])))
    return r / len(queries)


def build_queries(test_pairs, preds, max_q=800, seed=0x9E3779B97F4A7C15, per_src=3):
    rng = np.random.default_rng(seed)
    pool, seen, src_n = [], set(), {}
    for f, t in test_pairs:
        if f not in preds or not preds[f]:
            continue
        if src_n.get(f, 0) >= per_src:
            continue
        b = preds[f][rng.integers(0, len(preds[f]))]
        if b in preds and preds[b]:
            a = preds[b][rng.integers(0, len(preds[b]))]
            seq = (a, b, f)
        else:
            seq = (b, f)
        if seq in seen:
            continue
        seen.add(seq)
        src_n[f] = src_n.get(f, 0) + 1
        pool.append((list(seq), t))
    if len(pool) > max_q:
        take = rng.choice(len(pool), max_q, replace=False)
        pool = [pool[i] for i in take]
    return pool


def bpr_train(E, edge_pool, edge_set, n, steps, B=256, lr=3e-4, seed=5,
              log=(), eval_fn=None, eval_every=400):
    rng = np.random.default_rng(seed)
    E = E.copy()
    m = np.zeros_like(E)
    v = np.zeros_like(E)
    b1, b2, eps = 0.9, 0.999, 1e-8
    t0 = time.time()
    history = [(0, eval_fn(E) if eval_fn else 0.0)]
    for step in range(1, steps + 1):
        idx = rng.integers(0, len(edge_pool), size=B)
        grads = {}
        loss = 0.0
        for i in idx:
            f, tp = edge_pool[i]
            tn = int(rng.integers(0, n))
            if (f, tn) in edge_set:
                continue
            sf = rms(E[f])
            sp_ = float(sf @ E[tp])
            sn = float(sf @ E[tn])
            d = sp_ - sn
            sig = 1.0 / (1.0 + np.exp(min(d, 50)))  # d(sigmoid) wrt d
            loss += np.log1p(np.exp(-min(d, 50)))
            g = -sig
            for j, coef in ((f, 0.0), (tp, 0.0), (tn, 0.0)):
                pass
            # gradients (normalized-embedding convention: the rms jacobian
            # projection is dropped — standard for metric learning on
            # normalized vectors; keeps the pair update O(d))
            acc = grads.setdefault
            acc(f, np.zeros(E.shape[1]))[:] += g * (E[tp] - E[tn])
            acc(tp, np.zeros(E.shape[1]))[:] += g * sf
            acc(tn, np.zeros(E.shape[1]))[:] += -g * sf
        # AdamW on touched rows
        for j, g in grads.items():
            m[j] = b1 * m[j] + (1 - b1) * g
            v[j] = b2 * v[j] + (1 - b2) * g * g
            mh = m[j] / (1 - b1 ** step)
            vh = v[j] / (1 - b2 ** step)
            E[j] -= lr * mh / (np.sqrt(vh) + eps)
        if step % eval_every == 0:
            val = eval_fn(E) if eval_fn else 0.0
            history.append((step, val))
            print(f"      step {step:5}  MRR {val:.4f}  loss {loss/ B:.4f}  ({time.time()-t0:.0f}s)", flush=True)
    return E, history


def run_rung(links, target_particles, steps):
    # temporal prefix by particle count
    seen = set()
    cut = len(links)
    for i, (_, f, t) in enumerate(links):
        seen.add(f)
        seen.add(t)
        if len(seen) >= target_particles:
            cut = i + 1
            break
    prefix = links[:cut]
    split = int(len(prefix) * 0.9)
    train, test = prefix[:split], prefix[split:]
    print(f"\n=== rung target {target_particles:,} -> prefix {len(prefix):,} links "
          f"(train {len(train):,}, test {len(test):,}) ===", flush=True)

    pid = {}
    def intern(c):
        if c not in pid:
            pid[c] = len(pid)
        return pid[c]
    tr_pairs = []
    preds = {}
    for _, f, t in train:
        fi, ti = intern(f), intern(t)
        tr_pairs.append((fi, ti))
        preds.setdefault(ti, []).append(fi)
    n = len(pid)
    print(f"  particles {n:,}", flush=True)

    test_pairs = [(intern(f), intern(t)) for _, f, t in test if f in pid and t in pid]
    queries = build_queries(test_pairs, preds)
    print(f"  queries {len(queries)}", flush=True)

    rows, cols = [], []
    for f, t in tr_pairs:
        rows.append(f)
        cols.append(t)
    A = sp.csr_matrix((np.ones(len(rows)), (rows, cols)), shape=(n, n))
    phi = pagerank(A)
    t0 = time.time()
    U, s, V = mixed_svd(A, phi, D)
    print(f"  svds k={D} sigma1={s[0]:.4f} sigma_k={s[-1]:.6f} ({time.time()-t0:.0f}s)", flush=True)
    E = U * np.sqrt(s)

    floor = mrr_queries(E, queries)
    bg = bigram_mrr(tr_pairs, queries, n)
    print(f"  floor MRR {floor:.4f} · bigram {bg:.4f}", flush=True)

    edge_pool = tr_pairs
    edge_set = set(tr_pairs)
    eval_fn = lambda X: mrr_queries(X, queries)

    rng = np.random.default_rng(3)
    E_rand = rng.normal(0, 1, (n, D))
    E_rand /= np.linalg.norm(E_rand, axis=1, keepdims=True)

    print("  compiled E + BPR:", flush=True)
    _, h_comp = bpr_train(E, edge_pool, edge_set, n, steps, eval_fn=eval_fn)
    print("  random E + BPR:", flush=True)
    _, h_rand = bpr_train(E_rand, edge_pool, edge_set, n, steps, eval_fn=eval_fn)

    return {
        "n_particles": n, "n_train": len(train), "n_test": len(test),
        "n_queries": len(queries), "sigma": s.tolist(),
        "floor": floor, "bigram": bg,
        "compiled_bpr": h_comp, "random_bpr": h_rand,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rungs", type=int, nargs="+",
                    default=[100_000, 500_000, 1_000_000, 3_100_000])
    ap.add_argument("--steps", type=int, default=2000)
    args = ap.parse_args()

    links = load_links()
    print(f"bostrom: {len(links):,} links loaded", flush=True)
    results = {}
    for r in args.rungs:
        results[r] = run_rung(links, r, args.steps)
        with open("eval/scale_ladder_result.json", "w") as f:
            json.dump(results, f)

    print("\n=== ladder summary ===")
    print(f"{'rung':>10} {'particles':>10} {'floor':>8} {'bigram':>8} {'comp+BPR':>20} {'rand+BPR':>20}")
    for r, res in results.items():
        cf = " ".join(f"{v:.3f}" for _, v in res["compiled_bpr"])
        rf = " ".join(f"{v:.3f}" for _, v in res["random_bpr"])
        print(f"{r:>10,} {res['n_particles']:>10,} {res['floor']:8.4f} {res['bigram']:8.4f}   {cf:18}   {rf:18}")


if __name__ == "__main__":
    main()
