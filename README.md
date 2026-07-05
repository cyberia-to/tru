# tru

**Compile a transformer from a knowledge graph — no training, no dataset, no gradient descent.**

The architecture and every weight fall out of the graph's own spectrum. The compile is a pure function: the same graph produces a byte-identical model, in seconds, on a CPU. No floats anywhere in the path — fixed-point over the Goldilocks field — so two machines get the same result, and the result is verifiable.

tru does two things, and they are the same thing at different scales:

- **focus** — turn a weighted graph into **φ\***, a collective-attention distribution: a truth-weighted ranking of every node, computed by iterating a three-operator kernel to a fixed point.
- **compile** — freeze that φ\* into a Llama-shaped transformer. Embedding dimension, head count, layer depth, and all weight matrices are *derived* from the φ\*-weighted adjacency. Not trained. Compiled.

Focusing is the continuous limit; the compiled model is that limit frozen at finite depth.

---

## Why compile instead of train

A trained model needs a dataset, GPUs, days of compute, and lands at weights you cannot reproduce. tru's compile is deterministic and cheap:

| | training | tru compile |
|---|---|---|
| input | a curated dataset | a graph snapshot |
| cost | GPU-days | CPU-seconds |
| result | irreproducible | byte-identical, every run |
| arithmetic | float | fixed-point (Goldilocks field) |
| provenance | opaque | the model *is* a hash of the graph |

The architecture is not configured — it is measured. The embedding dimension is the effective rank of the graph's spectrum; the head count is the number of discovered dialects; the layer depth is the graph's diameter times its mixing time. The graph decides what shape of model it wants to be.

---

## Quick start

tru builds on **stable Rust** and depends on two sibling repos (the Goldilocks field and the hash), laid out beside it:

```sh
git clone https://github.com/cyberia-to/tru
git clone https://github.com/cyberia-to/strata   # nebu — Goldilocks field arithmetic
git clone https://github.com/cyberia-to/hemera   # content-addressing / hashing
cd tru && cargo build --release
```

Generate a demo graph and try the three commands:

```sh
cargo run -p tru --example gen_demo -- /tmp        # writes /tmp/demo.graph
alias tru=./target/release/tru
```

**Rank a graph** — compute φ\* and each node's spectral position:

```
$ tru focus /tmp/demo.graph
focus demo-graph
  particles 4 · cyberlinks 5
  syntropy J 0.0484 · entropy H 1.3379
  κ 0.412 · λ₂ 0.604 · T(ε) 16

cyberank φ*(p) · position (x,y) — top 4
  0100000000000000…  0.337544  (+0.2471, +0.0675)
  0300000000000000…  0.304874  (+0.1290, +1.0000)
  0200000000000000…  0.212581  (-1.0000, -0.3204)
  0400000000000000…  0.145001  (+0.6239, -0.7471)
```

**Price a contribution** — how much a new link sharpens collective focus (Δφ⁺):

```
$ tru impulse /tmp/demo.graph --from 02 --to 01 --stake 8000
impulse 0200000000000000… → 0100000000000000…
  stake 8000
  Δφ⁺ reward 0.072235 · ΔJ +0.072235
  entropy drop +0.072235 · discovery +0.000000 · ‖Δφ*‖₁ 0.324732
```

**Compile a model** — derive a transformer from the graph:

```
$ tru compile /tmp/demo.graph -o /tmp/demo.model
compile demo-graph → demo-graph-ct0
  d 64 · h 1 · L 4
  particles 9 · params 268168
  tensors 50 · particle 429704ff8514fc02…
  wrote /tmp/demo.model
```

Run it twice — the `particle` (the model's hash) is identical. That is the determinism guarantee.

---

## The three capabilities

| command | computes | use |
|---|---|---|
| `tru focus <graph>` | φ\*, cyberank, syntropy, spectral positions | rank a graph; a truth-weighted PageRank with geometry |
| `tru impulse <graph> --from P --to Q` | Δφ⁺, the directed focus shift of one link | price a contribution's information gain |
| `tru compile <graph> -o <model>` | a Llama-shaped `.model` (CT-0) | turn a graph into an inference-ready transformer |
| `tru inspect / vocab / model <file>` | container summaries | read `.graph` / `.vocab` / `.model` files |

---

## How it works

```
                          .graph
                            │
                     ┌──────▼──────┐
                     │  FOCUSING   │   tri-kernel: diffusion + springs + heat,
                     │  → φ*       │   blended and iterated to a fixed point,
                     └──────┬──────┘   honesty-weighted (karma · market · surprise)
              φ*            │
        ┌─────────────┬─────┴──────┐
        ▼             ▼            ▼
   ┌─────────┐  ┌──────────┐  ┌─────────┐
   │ COMPILE │  │ IMPULSE  │  │  RANK   │
   │ → .model│  │ → Δφ⁺    │  │ cyberank│
   └─────────┘  └────┬─────┘  └─────────┘
   CT-0, 8 passes    │ Δφ⁺ → surprise-weighted value → Shapley
                     ▼
                  rewards
```

**Focusing** runs the tri-kernel — three graph operators (a random walk, a screened-spring solve, and a heat kernel) blended into one contraction and iterated to its unique fixed point φ\*. The graph is weighted by honesty before it is read: stake is commitment, karma is a track record of being right before the crowd, and market price prunes links nobody believes.

**Compilation (CT-0)** is eight passes from `.graph` to `.model`: index the particles, discover dialects (which become attention heads), derive the architecture from the spectrum, and factorize the φ\*-weighted adjacency into an embedding, per-head attention projections, and a Clifford MLP. When the graph carries no geometric (bivector) data, the output is byte-identical to a plain scalar transformer.

**Impulse** measures Δφ⁺ — the gain in [syntropy](docs/terms/syntropy.md) one signal produces — the quantity a contributor is paid against. `tru`'s reward layer divides it fairly by Shapley value; the leaderless settlement that distributes the computation lives in sibling repos (`foculus`, `tok`).

Every number is fixed-point over the Goldilocks field (p = 2⁶⁴ − 2³² + 1). No float touches the provable path, so the whole pipeline is deterministic and reproducible bit-for-bit.

---

## Status — 0.1

Reference implementation. The full pipeline runs end-to-end and is deterministic:

- ✅ focusing engine (φ\*, cyberank, syntropy, spectral positions, Δφ⁺)
- ✅ honesty weighting (BTS → karma, will + conviction, market price, surprise ρ)
- ✅ CT-0 compiler — all 8 passes, `.graph → .model`, byte-identical across runs
- ✅ reward magnitude (value, Shapley attribution)
- ✅ builds on stable Rust · 87 tests · clippy-clean · fixed-point throughout

**What's next (post-0.1):** scale — the SVD is exact subspace iteration, correct but not yet the randomized/GPU form needed for million-node graphs. Runtime interop (loading a `.model` into an inference engine) and cross-repo settlement (foculus/tok) are outside this crate by design.

See [`specs/README.md`](specs/README.md) for the per-spec status map.

---

## Where it fits

tru is the intelligence layer of a larger stack. It is the only component that understands graph structure: it computes φ\* and compiles `.model` files; the inference engine, the renderer, and the consensus layer consume its outputs and stay graph-agnostic. The graph snapshots come from `cybergraph`; finality comes from `foculus`.

---

## Learn more

- **[specs/README.md](specs/README.md)** — the build map: every spec, what it produces, and its status
- **[specs/ct0.md](specs/ct0.md)** — the CT-0 compile, all 8 passes, conformance predicates
- **[specs/focusing.md](specs/focusing.md)** · **[specs/tri-kernel.md](specs/tri-kernel.md)** — how φ\* is computed
- **[specs/rewards.md](specs/rewards.md)** — surprising-syntropy minting and Shapley settlement
- **[docs/explanation/overview.md](docs/explanation/overview.md)** — what tru computes and why

## License

[Cyber](LICENSE).
