# tru

**A convergence engine that computes collective truth — and pays for creating it.**

tru turns a knowledge graph into **φ\***, the closest approximation to shared knowledge a network of agents can compute. From that one computation come two missions, and they are inseparable:

- **Truth** — rank every idea by the focus it has earned, and *compile* the graph directly into a transformer. No training: the model's architecture and weights are derived from the graph's own spectrum.
- **Reward** — mint currency for knowledge itself. New money appears only when, and exactly where, a contribution makes the graph more coherent. No aggregator decides who earned what; each contributor self-mints against the shift it provably caused.

Truth is the unit of account. Computing what the network knows and paying the people who add to it are the same event, priced in the same quantity.

Everything runs in fixed-point over the Goldilocks field — no floats in the provable path — so the result is deterministic, reproducible bit-for-bit, and therefore *verifiable*. That is what lets truth be money.

---

## Mission 1 — Truth: rank a graph, compile a model

tru reads a graph weighted by honesty — stake is commitment, [karma](specs/truth-scoring.md) is a track record of being right before the crowd, market price prunes links nobody believes — and iterates a three-operator kernel to its unique fixed point **φ\***. Every node's φ\*(p) is the focus it has earned from the entire weighted graph. No authority assigns it; it converges.

That same φ\* **compiles into a transformer.** The embedding dimension is the effective rank of the graph's spectrum; the attention heads are its discovered dialects; the layer depth is its diameter times its mixing time. The architecture is not configured — it is *measured*. The graph decides what shape of model it wants to be.

| | training a model | compiling one with tru |
|---|---|---|
| input | a curated dataset | a graph snapshot |
| cost | GPU-days | CPU-seconds |
| result | irreproducible | byte-identical, every run |
| arithmetic | float | fixed-point (Goldilocks field) |
| provenance | opaque | the model *is* a hash of the graph |

## Mission 2 — Reward: mint money for creating knowledge

This is why any of it runs. tru is a **minting engine whose unit of account is proven focus shift.**

When a contribution reshapes the graph, focus rolls to a new resting place and the drop in free energy — the gain in **syntropy** — is the value created. tru measures it as **Δφ⁺**, the directed focus impulse. A contributor computes Δφ⁺ locally, proves it, and self-mints in proportion. There is no central aggregator deciding who contributed what, and no emission untethered from contribution:

> **New money is minted only when, and exactly where, knowledge is created.** Inflation is not a policy — it is the measure of a physical process.

The division is fair by construction. Overlapping contributions split by [Shapley value](specs/rewards.md); each is weighted by its **surprise** — how far it beat the crowd's own prediction — so a copy of the consensus earns nothing however large its raw shift. Honesty is priced in, not assumed: the same Bayesian-truth-serum score that pays truth-tellers slashes noise.

`tru impulse` prices a single contribution live; `tru`'s reward layer computes the surprise-weighted value and its Shapley division. (The leaderless settlement that distributes the computation across the network, and the mint itself, live in the sibling consensus and token layers — see **Status** below.)

---

## Quick start

tru builds on **stable Rust** and depends on two sibling repos (the Goldilocks field and the hash), laid out beside it:

```sh
git clone https://github.com/cyberia-to/tru
git clone https://github.com/cyberia-to/strata   # nebu — Goldilocks field arithmetic
git clone https://github.com/cyberia-to/hemera   # content-addressing / hashing
cd tru && cargo build --release
```

Generate a demo graph and try all three:

```sh
cargo run -p tru --example gen_demo -- /tmp        # writes /tmp/demo.graph
alias tru=./target/release/tru
```

**Truth — rank the graph** (φ\* and each node's spectral position):

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

**Truth — compile a model** (a transformer derived from the graph):

```
$ tru compile /tmp/demo.graph -o /tmp/demo.model
compile demo-graph → demo-graph-ct0
  d 64 · h 1 · L 4
  particles 9 · params 268168
  tensors 50 · particle 429704ff8514fc02…
  wrote /tmp/demo.model
```

Run it twice — the `particle` (the model's hash) is identical. That reproducibility is the whole point.

**Reward — price a contribution** (how much a new link sharpens collective focus):

```
$ tru impulse /tmp/demo.graph --from 02 --to 01 --stake 8000
impulse 0200000000000000… → 0100000000000000…
  stake 8000
  Δφ⁺ reward 0.072235 · ΔJ +0.072235
  entropy drop +0.072235 · discovery +0.000000 · ‖Δφ*‖₁ 0.324732
```

That Δφ⁺ is what a contributor mints against — a copy would price at zero.

---

## How it works

```
                          .graph
                            │
                     ┌──────▼──────┐
                     │  FOCUSING   │   tri-kernel: diffusion + springs + heat,
                     │  → φ*       │   blended into one contraction, iterated to
                     └──────┬──────┘   its fixed point, honesty-weighted
              φ*            │
        ┌─────────────┬─────┴───────────┐
        ▼             ▼                 ▼
   ┌─────────┐  ┌──────────┐      ┌───────────┐
   │  RANK   │  │ COMPILE  │      │  IMPULSE  │
   │ cyberank│  │ → .model │      │  → Δφ⁺    │
   └─────────┘  └──────────┘      └─────┬─────┘
     truth        truth                 │ surprise-weighted value → Shapley
                  CT-0, 8 passes        ▼
                                     REWARD → mint
```

One kernel, computed once, feeds all three. **Focusing** blends a random walk, a screened-spring solve, and a heat kernel into a single contraction and iterates to φ\*. **Compilation (CT-0)** factorizes the φ\*-weighted adjacency into an embedding, per-head attention, and a Clifford MLP — eight deterministic passes. **Impulse** measures the [syntropy](docs/terms/syntropy.md) gain one signal produces, and the reward layer divides it by Shapley over surprising contributions.

Every number is a field element modulo p = 2⁶⁴ − 2³² + 1. No float touches the provable path, so truth is the same on every machine — and a proof of Δφ⁺ is a claim on newly minted currency.

---

## Status — 0.1

Reference implementation. Both missions run end-to-end and are deterministic:

- ✅ **truth** — φ\*, cyberank, syntropy, spectral positions; honesty weighting (BTS → karma, will + conviction, market price, surprise ρ)
- ✅ **model** — CT-0 compiler, all 8 passes, `.graph → .model`, byte-identical across runs
- ✅ **reward** — Δφ⁺ impulse, surprise-weighted value, Shapley attribution (the mintable quantity and its fair division)
- ✅ builds on stable Rust · 87 tests · clippy-clean · fixed-point throughout

**Boundaries by design.** tru owns *magnitude* — what a contribution is worth and who earned it. The leaderless settlement lottery that computes it across the network lives in `foculus`; conservation and the actual mint live in `tok`. Scale (a randomized/GPU SVD for million-node graphs) and runtime inference (loading a `.model` into an engine) are the post-0.1 frontiers.

See [`specs/README.md`](specs/README.md) for the per-spec status map.

---

## Where it fits

tru is the intelligence-and-economics layer of the cyber stack — the only component that understands graph structure. It computes φ\*, compiles `.model` files, and prices Δφ⁺; the inference engine, the renderer, the consensus layer, and the token layer consume its outputs and stay graph-agnostic. Graph snapshots come from `cybergraph`; finality from `foculus`; minting from `tok`.

---

## Learn more

- **[specs/README.md](specs/README.md)** — the build map: every spec, what it produces, and its status
- **[specs/rewards.md](specs/rewards.md)** — surprising-syntropy minting, Shapley settlement, the whole reward economy
- **[specs/ct0.md](specs/ct0.md)** — the CT-0 compile, all 8 passes, conformance predicates
- **[specs/focusing.md](specs/focusing.md)** · **[specs/tri-kernel.md](specs/tri-kernel.md)** — how φ\* is computed
- **[docs/explanation/overview.md](docs/explanation/overview.md)** — what tru computes and why

## License

[Cyber](LICENSE).
