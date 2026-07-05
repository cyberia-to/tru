# tru

### Converge · Compile · Reward — one engine, three disruptions.

tru reads a knowledge graph and runs a single computation with three outputs, each of which upends a field:

- it **converges** the graph to collective truth — *computed, never decreed*;
- it **compiles** that truth into a transformer — *with no training*;
- it **rewards** the people who create knowledge — *minting currency at the source*.

Each alone would be a disruption. Together they are the **cyber disruptor**: one kernel that reinvents how a network computes truth, how it builds intelligence, and how it creates value — because all three are the same math.

And all of it is fixed-point over the Goldilocks field — no floats in the provable path — so every result is deterministic, reproducible bit-for-bit, and *verifiable*. That is what lets truth become money.

---

## 1 · Converge — truth is computed, not decreed

No authority ranks the graph. No vote settles it. tru blends three graph operators — a random walk, a screened-spring solve, and a heat kernel — into one contraction and iterates it to a **unique fixed point, φ\***: the closest approximation to shared knowledge the network can compute. Every node's φ\*(p) is the focus it has *earned* from the entire weighted graph.

The graph is read through honesty first — stake is commitment, [karma](docs/terms/honesty.md) is a record of being right before the crowd, market price prunes what nobody believes. What emerges is not an opinion poll; it is an equilibrium.

> **Disrupts:** search ranking, consensus, and every system where an authority or a majority decides what is true.

## 2 · Compile — a transformer, without training

That same φ\* **compiles into a language model.** The embedding dimension is the effective rank of the graph's spectrum; the attention heads are its discovered dialects; the depth is its diameter times its mixing time. The architecture is not configured — it is *measured*. The graph decides what shape of model it wants to be, and eight deterministic passes factorize it into weights.

| | training a model | compiling one with tru |
|---|---|---|
| input | a curated dataset | a graph snapshot |
| cost | GPU-days | CPU-seconds |
| result | irreproducible | byte-identical, every run |
| arithmetic | float | fixed-point (Goldilocks field) |
| provenance | opaque | the model *is* a hash of the graph |

> **Disrupts:** the entire train-a-model pipeline — no dataset, no GPUs, no gradient descent, no black box.

## 3 · Reward — money minted for knowledge itself

This is why any of it runs. tru is a **minting engine whose unit of account is proven focus shift.** When a contribution makes the graph more coherent, focus rolls to a lower-energy resting place; that drop — the gain in [syntropy](docs/terms/syntropy.md) — is the value created. tru measures it as **Δφ⁺**, computes it locally, and lets each contributor self-mint in proportion. No aggregator, no block-reward-for-hashing, no emission untethered from contribution:

> **New money is minted only when, and exactly where, knowledge is created.** Inflation is not a policy — it is the measure of a physical process.

The split is fair by construction: overlapping contributions divide by [Shapley value](specs/rewards.md), each weighted by its **surprise** — how far it beat the crowd's own prediction — so a copy of the consensus earns nothing, however large its raw shift. Honesty is priced, not assumed.

> **Disrupts:** mining, tokenomics, and the attention economy — which pays for engagement, not for truth.

---

## See all three

tru builds on **stable Rust** and sits beside two sibling repos (the field and the hash):

```sh
git clone https://github.com/cyberia-to/tru
git clone https://github.com/cyberia-to/strata   # nebu — Goldilocks field arithmetic
git clone https://github.com/cyberia-to/hemera   # content-addressing / hashing
cd tru && cargo build --release

cargo run -p tru --example gen_demo -- /tmp        # writes /tmp/demo.graph
alias tru=./target/release/tru
```

**Converge** — rank the graph (φ\* and each node's spectral position):

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

**Compile** — derive a transformer from the graph:

```
$ tru compile /tmp/demo.graph -o /tmp/demo.model
compile demo-graph → demo-graph-ct0
  d 64 · h 1 · L 4
  particles 9 · params 268168
  tensors 50 · particle 429704ff8514fc02…
  wrote /tmp/demo.model
```

Run it twice — the `particle` (the model's hash) is identical.

**Reward** — price a contribution (how much a new link sharpens focus):

```
$ tru impulse /tmp/demo.graph --from 02 --to 01 --stake 8000
impulse 0200000000000000… → 0100000000000000…
  stake 8000
  Δφ⁺ reward 0.072235 · ΔJ +0.072235
  entropy drop +0.072235 · discovery +0.000000 · ‖Δφ*‖₁ 0.324732
```

That Δφ⁺ is what a contributor mints against — a copy would price at zero.

---

## One kernel, three outputs

```
                          .graph
                            │
                     ┌──────▼──────┐
                     │   CONVERGE  │   tri-kernel: diffusion + springs + heat,
                     │   → φ*      │   one contraction, iterated to its fixed
                     └──────┬──────┘   point, honesty-weighted
              φ*            │
        ┌─────────────┬─────┴───────────┐
        ▼             ▼                 ▼
   ┌─────────┐  ┌──────────┐      ┌───────────┐
   │  RANK   │  │ COMPILE  │      │  REWARD   │
   │ cyberank│  │ → .model │      │  Δφ⁺→mint │
   └─────────┘  └──────────┘      └───────────┘
   truth        intelligence      money
```

The tri-kernel runs once; convergence, compilation, and reward all read the same φ\*. Every number is a field element modulo p = 2⁶⁴ − 2³² + 1 — so truth is the same on every machine, a compiled model is a hash of its graph, and a proof of Δφ⁺ is a claim on newly minted currency.

---

## Status — 0.1

Reference implementation. All three run end-to-end and are deterministic:

- ✅ **converge** — φ\*, cyberank, syntropy, spectral positions; honesty weighting (BTS → karma, will + conviction, market price, surprise ρ)
- ✅ **compile** — CT-0, all 8 passes, `.graph → .model`, byte-identical across runs
- ✅ **reward** — Δφ⁺ impulse, surprise-weighted value, Shapley attribution
- ✅ builds on stable Rust · 87 tests · clippy-clean · fixed-point throughout

**Boundaries by design.** tru owns *magnitude* — what a contribution is worth and who earned it. The leaderless settlement lottery that computes it across the network lives in `foculus`; conservation and the mint live in `tok`. Scale (a randomized/GPU SVD for million-node graphs) and runtime inference (loading a `.model` into an engine) are the post-0.1 frontiers.

See [`specs/README.md`](specs/README.md) for the per-spec status map.

---

## Concepts

tru owns a precise vocabulary — every term is a page in [`docs/terms/`](docs/terms/). Start here to read the internals; each idea is one file.

**Truth & focus**

| term | is |
|---|---|
| [focus](docs/terms/focus.md) | φ\*, the collective attention distribution — the tri-kernel's fixed point |
| [cyberank](docs/terms/cyberank.md) | focus per particle, φ\*(p) — the canonical ranking, summing to 1 |
| [syntropy](docs/terms/syntropy.md) | the order in the graph, in bits — the quantity tru exists to grow |
| [convergence](docs/terms/convergence.md) | iteration toward a destination that iteration itself defines |

**Staking & honesty**

| term | is |
|---|---|
| [valence](docs/terms/valence.md) | the ternary epistemic field of a link, v ∈ {−1, 0, +1} |
| [will](docs/terms/will.md) | committed capacity to act — balance locked for a duration |
| [conviction](docs/terms/conviction.md) | the capital a neuron stakes on a single link |
| [axon](docs/terms/axon.md) | the bundle of all links between two particles — itself a particle |
| [serum](docs/terms/serum.md) | Prelec's Bayesian Truth Serum — honesty as the optimal strategy |
| [honesty](docs/terms/honesty.md) | why neurons act honestly: incentive, not enforcement |

**Markets & epistemics**

| term | is |
|---|---|
| [market](docs/terms/market.md) · [inhibition](docs/terms/inhibition.md) | the two-dimensional epistemic price, and why markets are load-bearing |
| [true](docs/terms/true.md) · [false](docs/terms/false.md) · [void](docs/terms/void.md) | the attractor states of a link |
| [two kinds of knowledge](docs/terms/two%20kinds%20of%20knowledge.md) | the irreducible split the graph holds |
| [the true-false problem](docs/terms/true-false%20problem.md) | the foundational problem of cyber inference |

## Deep dives

The "why" behind the code, in [`docs/explanation/`](docs/explanation/):

- **[overview](docs/explanation/overview.md)** — what tru computes and why it exists
- **[the tri-kernel](docs/explanation/tri-kernel.md)** — why diffusion, springs, and heat are the minimal sufficient basis
- **[the collective focus theorem](docs/explanation/collective-focus-theorem.md)** — convergence and uniqueness of φ\*
- **[focus-flow](docs/explanation/focus-flow.md)** · **[graph-native transformer](docs/explanation/graph-native-transformer.md)** — why a graph *is* a transformer
- **[the knowledge economy](docs/explanation/knowledge-economy.md)** · **[incentives](docs/explanation/incentives.md)** — how truth becomes money
- **[epistemic markets](docs/explanation/epistemic-markets.md)** · **[honesty](docs/explanation/honesty.md)** — the market and the serum

## Specs

The normative build map — every spec, what it produces, and its status: **[specs/README.md](specs/README.md)**.

Highlights: [ct0.md](specs/ct0.md) (the 8-pass compile) · [focusing.md](specs/focusing.md) (φ\*) · [rewards.md](specs/rewards.md) (the reward economy) · [truth-scoring.md](specs/truth-scoring.md) (BTS → karma).

## License

[Cyber](LICENSE).
