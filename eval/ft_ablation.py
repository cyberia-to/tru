#!/usr/bin/env python3
"""fine-tuning ablation — does the compiled init train better than random?

the init-time attention verdict (tru#5, eval/e2e_pussy.md) is capped by an
information ceiling: on structural walks the answer is never in the prefix.
training lifts the ceiling. this probe asks whether CT-0's compiled weights
are a better STARTING POINT than random — the product claim of compilation.

configs (all llama-semantics, d=64, L=4, tied head, no MLP — its
LayerScale is 1e-5 at init):
  random    E ~ N(0, 0.02), W random                — the training baseline
  structural  shipped Fx E + shipped attention weights (gain 8.77)
  quiet     structural but attention output gain 1e-3 (tru#5 shipping proposal)
  embed-only  shipped Fx E, random attention        — E's contribution isolated
  exact-E   scipy-spectrum E (tru#7), random attention — artifact vs exact

task: next-particle prediction on random walks over the train graph
(temporal 90% of pussy_links.jsonl). eval: the same 1181 held-out
walk-continuation queries as eval_pussy (MRR, full vocab).

usage: .venv/bin/python eval/ft_ablation.py [--steps 1500] [--out eval/ft_ablation.md]
"""

import argparse
import json
import struct
import time

import numpy as np
import torch

torch.manual_seed(0)

DATA = "eval/data/pussy_links.jsonl"
DUMP = "/tmp/e2e_dump.bin"
SVD = "/tmp/pussy_svd.bin"
D_MODEL = 64
L_TRAIN = 4
T = 10  # walk length
B = 32  # walks per step
EPS = 1e-5
ROPE_THETA = 1e6


# ---------- data ----------

def load_links():
    links = []
    with open(DATA) as f:
        for line in f:
            d = json.loads(line)
            links.append((d["h"], d["f"], d["t"]))
    links.sort(key=lambda x: x[0])
    return links


def load_dump():
    raw = open(DUMP, "rb").read()
    off = 0

    def u64():
        nonlocal off
        v = int.from_bytes(raw[off:off + 8], "little")
        off += 8
        return v

    n, d = u64(), u64()
    e = np.frombuffer(raw, "<f8", n * d, off).reshape(n, d).copy()
    off += n * d * 8
    nq = u64()
    queries = []
    for _ in range(nq):
        L = u64()
        seq = [u64() for _ in range(L)]
        queries.append((seq, u64()))
    L = u64()
    layers = []
    for _ in range(L):
        tensors = {}
        for name in ["wq", "wk", "wv", "wo"]:
            tensors[name] = np.frombuffer(raw, "<f8", d * d, off).reshape(d, d).copy()
            off += d * d * 8
        layers.append(tensors)
    return n, d, e, queries, layers


def load_scipy_e(n, d):
    raw = open(SVD, "rb").read()
    k = int.from_bytes(raw[0:8], "little")
    sig = np.frombuffer(raw, "<f8", k, 8)
    u = np.frombuffer(raw, "<f8", n * k, 8 + k * 8).reshape(n, k)
    assert k == d
    return (u * np.sqrt(sig)).astype(np.float64)


class WalkSampler:
    def __init__(self, links, split):
        self.adj = {}
        for _, f, t in links[:split]:
            self.adj.setdefault(f, []).append(t)
        self.nodes = [f for f, ts in self.adj.items() if ts]
        self.p2i = None  # interned by the caller's vocab
        self.rng = np.random.default_rng(7)

    def intern(self, pid_of):
        self.p2i = {p: pid_of(p) for p in set(self.nodes) | {t for ts in self.adj.values() for t in ts}}

    def batch(self, b, t_len):
        seqs = []
        for _ in range(b):
            node = self.nodes[self.rng.integers(0, len(self.nodes))]
            seq = [node]
            for _ in range(t_len - 1):
                nbrs = self.adj.get(node)
                if not nbrs:
                    break
                node = nbrs[self.rng.integers(0, len(nbrs))]
                seq.append(node)
            seqs.append(seq)
        return seqs


# ---------- model (llama semantics, mirrors eval_pussy forward) ----------

class TiedTransformer(torch.nn.Module):
    def __init__(self, n, d, layers, config):
        super().__init__()
        self.d = d
        self.n = n
        self.E = torch.nn.Parameter(torch.zeros(n, d))
        self.norm_g = torch.nn.Parameter(torch.ones(d))
        self.wq = torch.nn.ParameterList()
        self.wk = torch.nn.ParameterList()
        self.wv = torch.nn.ParameterList()
        self.wo = torch.nn.ParameterList()
        for _ in range(layers):
            self.wq.append(torch.nn.Parameter(torch.zeros(d, d)))
            self.wk.append(torch.nn.Parameter(torch.zeros(d, d)))
            self.wv.append(torch.nn.Parameter(torch.zeros(d, d)))
            self.wo.append(torch.nn.Parameter(torch.zeros(d, d)))
        self.n_layers = layers
        self.init_config = config

    def load(self, e=None, dump_layers=None, quiet=False, rng=None):
        with torch.no_grad():
            if e is not None:
                t = torch.tensor(e, dtype=torch.float32)
                # row-normalize: 96% of compiled rows are ~zero (the Fx SVD
                # keeps only the hub structure); rmsnorm then amplifies them
                # 1/sqrt(eps) ~ 316x per layer and the backward explodes
                # (~2^42 through 4 layers + final norm), and the tied head
                # sees a 1e4-scale spread of logits. unit rows put every
                # particle on equal footing for training; cold particles
                # start random and learn from walk co-occurrence.
                t = t / t.norm(dim=1, keepdim=True).clamp(min=1e-12)
                dead = t.norm(dim=1) < 1e-6
                if dead.any():
                    fill = torch.randn(int(dead.sum()), t.shape[1])
                    t[dead] = fill / fill.norm(dim=1, keepdim=True)
                self.E.copy_(t)
            else:
                self.E.normal_(0.0, 0.02)
                with torch.no_grad():
                    self.E /= self.E.norm(dim=1, keepdim=True).clamp(min=1e-12)
            g = 1e-3 if quiet else 1.0
            for l in range(self.n_layers):
                if dump_layers is not None:
                    for name, p in [("wq", self.wq[l]), ("wk", self.wk[l]),
                                    ("wv", self.wv[l]), ("wo", self.wo[l])]:
                        p.copy_(torch.tensor(dump_layers[l][name], dtype=torch.float32) * (g if name == "wo" else 1.0))
                else:
                    for p in (self.wq[l], self.wk[l], self.wv[l], self.wo[l]):
                        p.normal_(0.0, 1.0 / np.sqrt(self.d))

    def rms(self, x):
        return x * torch.rsqrt(x.pow(2).mean(-1, keepdim=True) + EPS)

    def rope(self, x):
        # x: (B, T, d); near-identity (theta = 1e6)
        d = x.shape[-1]
        pos = torch.arange(x.shape[1], dtype=torch.float32).unsqueeze(1)
        i = torch.arange(0, d, 2, dtype=torch.float32)
        f = ROPE_THETA ** (-i / d)
        ang = pos * f  # (T, d/2)
        c, s = torch.cos(ang), torch.sin(ang)
        x1, x2 = x[..., 0::2], x[..., 1::2]
        o1 = x1 * c - x2 * s
        o2 = x1 * s + x2 * c
        out = torch.empty_like(x)
        out[..., 0::2], out[..., 1::2] = o1, o2
        return out

    def forward(self, seqs):
        # seqs: list of token-id lists. returns logits (sum over positions of CE handled outside)
        B_ = len(seqs)
        T_ = max(len(s) for s in seqs)
        idx = torch.zeros(B_, T_, dtype=torch.long)
        mask = torch.zeros(B_, T_, dtype=torch.bool)
        for b, s in enumerate(seqs):
            idx[b, :len(s)] = torch.tensor(s, dtype=torch.long)
            mask[b, :len(s)] = True
        h = self.E[idx]  # (B,T,d) f64
        causal = torch.tril(torch.ones(T_, T_, dtype=torch.bool))
        for l in range(self.n_layers):
            hn = self.rms(h)
            q = self.rope(hn @ self.wq[l].T)
            k = self.rope(hn @ self.wk[l].T)
            v = hn @ self.wv[l].T
            att = q @ k.transpose(-1, -2) / np.sqrt(self.d)
            att = att.masked_fill(~causal, float("-inf"))
            w = torch.softmax(att, dim=-1)
            ctx = w @ v
            h = h + ctx @ self.wo[l].T
        h = self.rms(h) * self.norm_g
        logits = h @ self.E.T  # tied head
        return logits, idx, mask


def eval_mrr(model, queries, batch=256):
    model.eval()
    ranks = []
    with torch.no_grad():
        for i in range(0, len(queries), batch):
            chunk = queries[i:i + batch]
            seqs = [s for s, _ in chunk]
            logits, idx, mask = model.forward(seqs)
            for b, (_, gold) in enumerate(chunk):
                L = sum(mask[b]).item()
                lg = logits[b, L - 1]  # query at last position
                sg = lg[gold]
                ranks.append(1.0 / (1 + int(((lg > sg) & (torch.arange(lg.numel()) != gold)).sum())))
    model.train()
    return float(np.mean(ranks))


def bigram_floor(links, split, pid_of, queries, n):
    cnt = {}
    outc = {}
    for _, f, t in links[:split]:
        fi, ti = pid_of(f), pid_of(t)
        cnt[(fi, ti)] = cnt.get((fi, ti), 0) + 1
        outc[fi] = outc.get(fi, 0) + 1
    vocab_n = n
    r = 0.0
    for seq, gold in queries:
        f = seq[-1]
        s = np.array([cnt.get((f, j), 0) + 0.1 for j in range(vocab_n)])
        s /= outc.get(f, 0) + 0.1 * vocab_n
        r += 1.0 / (1 + int(np.sum(s > s[gold])))
    return r / len(queries)


def bigram_floor_seen(links, split, cid2idx, seen_queries, n):
    cnt, outc = {}, {}
    for _, f, t in links[:split]:
        fi, ti = cid2idx[f], cid2idx[t]
        cnt[(fi, ti)] = cnt.get((fi, ti), 0) + 1
        outc[fi] = outc.get(fi, 0) + 1
    r = 0.0
    for seq, gold in seen_queries:
        f = seq[-1]
        s = np.array([cnt.get((f, j), 0) + 0.1 for j in range(n)])
        s /= outc.get(f, 0) + 0.1 * n
        r += 1.0 / (1 + int(np.sum(s > s[gold])))
    return r / len(seen_queries)


# ---------- ablation ----------

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--steps", type=int, default=1500)
    ap.add_argument("--eval-every", type=int, default=100)
    ap.add_argument("--lr", type=float, default=1e-4)
    ap.add_argument("--objective", choices=["lm", "lp"], default="lm",
                    help="lm = walk next-token; lp = contrastive link "
                    "prediction (rank a train edge above sampled non-edges "
                    "— the objective aligned with the unseen-link eval)")
    args = ap.parse_args()

    links = load_links()
    split = len(links) * 9 // 10
    n, d, e_ship, queries, dump_layers = load_dump()

    pid_map = {}
    def pid_of(cid):
        if cid not in pid_map:
            import hashlib
            pid_map[cid] = len(pid_map)
        return pid_map[cid]

    # queries already carry compiled indices; the sampler needs the same
    # interning. reproduce tru's order: from, then to, then axon per link.
    # NOTE: instead of replicating hemera interning, we sample walks over
    # cids and map them through the same id-space as the dump: the dump's
    # queries use compiled indices, so we intern cids by first appearance
    # in the SAME loop tru uses (from, to, axon). we rebuild that here.
    import hashlib
    def h32(b):
        return hashlib.sha256(b).digest()
    order = []
    seen = set()
    def intern(p):
        if p not in seen:
            seen.add(p)
            order.append(p)
    for _, f, t in links[:split]:
        ax = h32(h32(f.encode()) + h32(t.encode()))
        intern(f)
        intern(t)
        intern(ax)
    cid2idx = {p: i for i, p in enumerate(order)}
    assert len(order) == n, f"intern count {len(order)} != dump n {n}"

    sampler = WalkSampler(links, split)
    sampler_adapted = {}
    for f, ts in sampler.adj.items():
        sampler_adapted[cid2idx[f]] = [cid2idx[t] for t in ts]
    walk_nodes = [cid2idx[p] for p in sampler.nodes]

    def sample_batch(b, t_len, rng):
        seqs = []
        for _ in range(b):
            node = walk_nodes[rng.integers(0, len(walk_nodes))]
            seq = [node]
            for _ in range(t_len - 1):
                nbrs = sampler_adapted.get(node)
                if not nbrs:
                    break
                node = nbrs[rng.integers(0, len(nbrs))]
                seq.append(node)
            seqs.append(seq)
        return seqs

    e_exact = load_scipy_e(n, d)

    configs = {
        "random": dict(e=None, dump_layers=None),
        "structural": dict(e=e_ship, dump_layers=dump_layers),
        "quiet": dict(e=e_ship, dump_layers=dump_layers, quiet=True),
        "embed-only": dict(e=e_ship, dump_layers=None),
        "exact-E": dict(e=e_exact, dump_layers=None),
    }

    # seen-query control: same prefix construction over TRAIN edges — tracks
    # memorization separately from generalization.
    preds = {}
    for a, bs in sampler_adapted.items():
        for b in bs:
            preds.setdefault(b, []).append(a)
    rng_s = np.random.default_rng(11)
    seen_queries = []
    seen_used = set()
    for r in links[:split]:
        f, t = cid2idx[r[1]], cid2idx[r[2]]
        if f not in preds:
            continue
        b = preds[f][rng_s.integers(0, len(preds[f]))]
        if b in preds:
            a = preds[b][rng_s.integers(0, len(preds[b]))]
            seq = [a, b, f]
        else:
            seq = [b, f]
        key = (tuple(seq), t)
        if key in seen_used:
            continue
        seen_used.add(key)
        seen_queries.append((list(seq), t))
        if len(seen_queries) >= 1181:
            break
    rng = np.random.default_rng(99)

    bg = bigram_floor(links, split, lambda c: cid2idx[c], queries, n)
    bg_seen = bigram_floor_seen(links, split, cid2idx, seen_queries, n)
    print(f"bigram floor: unseen {bg:.4f} seen {bg_seen:.4f}", flush=True)

    curves = {}
    for name, cfg in configs.items():
        torch.manual_seed(1234)
        rng = np.random.default_rng(99)
        # LP objective data: train edges with endpoints, for t- sampling
        edge_pool = [(cid2idx[f], cid2idx[t]) for _, f, t in links[:split]]
        edge_set = set(edge_pool)
        model = TiedTransformer(n, d, L_TRAIN, name).float()
        model.load(**cfg)
        opt = torch.optim.AdamW(model.parameters(), lr=args.lr, weight_decay=0.01)
        curve = [(0, eval_mrr(model, queries), eval_mrr(model, seen_queries))]
        t0 = time.time()
        for step in range(1, args.steps + 1):
            if args.objective == "lp":
                # contrastive: prefix -> f ranks t+ above t-
                seqs, t_pos, t_neg = [], [], []
                tries = 0
                while len(seqs) < B and tries < B * 20:
                    tries += 1
                    f, tp = edge_pool[rng.integers(0, len(edge_pool))]
                    if f not in preds:
                        continue
                    b = preds[f][rng.integers(0, len(preds[f]))]
                    seq = [b, f] if b not in preds else [preds[b][rng.integers(0, len(preds[b]))], b, f]
                    tn = int(rng.integers(0, n))
                    if (f, tn) in edge_set or tn == f:
                        continue
                    seqs.append(seq)
                    t_pos.append(tp)
                    t_neg.append(tn)
                logits, idx, mask = model.forward(seqs)
                # score at the last position (query f) via tied head
                last = []
                for b_, s in enumerate(seqs):
                    L = len(s)
                    last.append(logits[b_, L - 1])
                last = torch.stack(last)
                tp = torch.tensor(t_pos, dtype=torch.long)
                tn = torch.tensor(t_neg, dtype=torch.long)
                loss = torch.nn.functional.softplus(-(last.gather(1, tp.unsqueeze(1)) - last.gather(1, tn.unsqueeze(1)))).mean()
                opt.zero_grad()
                loss.backward()
                torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
                opt.step()
                if step % args.eval_every == 0:
                    curve.append((step, eval_mrr(model, queries), eval_mrr(model, seen_queries)))
                    print(f"{name:12} step {step:5}  unseen {curve[-1][1]:.4f}  seen {curve[-1][2]:.4f}  ({time.time()-t0:.0f}s)", flush=True)
                continue
            seqs = sample_batch(B, T, rng)
            logits, idx, mask = model.forward(seqs)
            # next-token CE at every valid position
            loss = 0.0
            ntok = 0
            for b, s in enumerate(seqs):
                L = len(s)
                if L < 2:
                    continue
                pred = logits[b, :L - 1]
                gold = idx[b, 1:L]
                loss = loss + torch.nn.functional.cross_entropy(pred, gold)
                ntok += L - 1
            loss = loss / max(ntok, 1)
            opt.zero_grad()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            opt.step()
            if step % args.eval_every == 0:
                curve.append((step, eval_mrr(model, queries), eval_mrr(model, seen_queries)))
                print(f"{name:12} step {step:5}  unseen {curve[-1][1]:.4f}  seen {curve[-1][2]:.4f}  ({time.time()-t0:.0f}s)", flush=True)
        curves[name] = curve

    print("\n=== summary — unseen-link MRR (generalization) / seen (memorization) ===")
    print(f"{'config':12} {'init':>8} {'final':>8}   unseen curve")
    for name, curve in curves.items():
        pts = " ".join(f"{v:.3f}" for _, v, _ in curve)
        print(f"{name:12} {curve[0][1]:8.4f} {curve[-1][1]:8.4f}   {pts}")
    print(f"{'config':12} {'':>8} {'':>8}   seen curve")
    for name, curve in curves.items():
        pts = " ".join(f"{s:.3f}" for _, _, s in curve)
        print(f"{name:12} {'':>8} {'':>8}   {pts}")
    print(f"\nbigram floor: unseen {bg:.4f}  seen {bg_seen:.4f}")

    with open("eval/ft_ablation_result.json", "w") as f:
        json.dump({"bigram_unseen": bg, "bigram_seen": bg_seen,
                   "curves": {k: v for k, v in curves.items()}}, f)


if __name__ == "__main__":
    main()
