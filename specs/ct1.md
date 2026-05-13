---
tags: cyber, tru, core, spec
crystal-type: spec
crystal-domain: cyber
alias: CT-1, compiled transformers spec, ct-spec, model compilation pipeline, tru compile, clifford, clifford extensions, geometric algebra extensions, wedge extensions
---
# Compiled Transformers Specification (CT-1)

formal contract for compiling a transformer from a [[cybergraph]] snapshot. companion to [[compiled transformers]] (the how-to article) and [[graph-native-transformer]] (the derivation). this page is what the rust crate implements; conformance is checked against the predicates in §12.

---

## 1. Scope

CT-1 specifies a deterministic function

$$\text{compile}: G \to \mathcal{M}$$

where $G$ is a cybergraph snapshot in [[cyb-graph|.graph format]] and $\mathcal{M}$ is a transformer checkpoint in [[cyb-model|.model format]]. Two implementations conforming to CT-1 must produce a byte-identical $\mathcal{M}$ given a byte-identical $G$ and the same compiler version.

---

## 2. Input Definitions

### 2.1 Snapshot

A snapshot is a `.graph` container (see [[cyb-graph]]) read into the tuple $G = (\mathcal{S}, h, \nu_{\text{compiler}})$ where:

- $\mathcal{S}$ — the `signals` records, ordered as written in the file (canonical chain order)
- $h$ — the `block` field of the `config` section
- $\nu_{\text{compiler}}$ — the compiler version string, always `"CT-1"`.

If the optional `proof` or `impulse` extension sections are present (see [[cyb-graph]] §extensions), conforming compilers verify proofs before compilation and may reuse impulses to skip power iteration (see §6.1). Snapshots without these extensions are accepted — the base `.graph` spec has no provenance layer.

### 2.2 Signal and cyberlink

Each $s \in \mathcal{S}$ is a signal per [[cyber/signal]]:

$$s = (\nu_s, t_s, \vec\ell_s) \quad \text{where} \quad \vec\ell_s = (\ell_{s,1}, \ldots, \ell_{s,n_s})$$

- $\nu_s$ — signing neuron (one per signal)
- $t_s$ — unix timestamp in seconds (one per signal), with $t_s \leq \text{config.captured\_at}$
- $\vec\ell_s$ — ordered vector of link records $\ell_{s,i} = (p, q, \tau, a, v)$, $1 \leq i \leq n_s$, where $p, q, \tau \in P$ (all three are particles, including the token denomination), $a \in \mathbb{F}_p$ (Goldilocks field element, $p = 2^{64} - 2^{32} + 1$), $v \in \{-1, 0, +1\}$

The seven-tuple cyberlink from [[cyber/link]] is reconstructed at iteration time. Note that $t_s$ in the snapshot is a unix timestamp; the chain's own link tuple carries a block height. Conversion happens at snapshot emission, not at compile time.

$$\ell = (\nu_s, p, q, \tau, a, v, t_s)$$

$a$ is in the smallest token unit (no floats). The set $L$ of all cyberlinks is $L = \bigcup_{s \in \mathcal{S}} \vec\ell_s$, concretely yielded by

```
fn links(S) -> Iterator<Cyberlink>:
    for s in S:
        for ℓ in s.links:
            yield (s.ν, ℓ.p, ℓ.q, ℓ.τ, ℓ.a, ℓ.v, s.t)
```

All passes that read "links" use this iterator. Passes that need per-signal grouping (6.1 impulse reuse, 8.3 walks) iterate $\mathcal{S}$ directly.

### 2.3 Particle and axon

A particle is a 32-byte hemera hash. The axon-particle of $(p, q)$ is

$$\text{axon}(p, q) = H(p \,\|\, q) \in P$$

where $H$ is hemera over the concatenation of the two 32-byte particles. This matches [[cybergraph]] axiom A6.

### 2.4 Effective stake

The effective stake of cyberlink $\ell = (\nu, p, q, \tau, a, v, t)$ is

$$w(\ell) = \begin{cases} a \cdot \rho_\tau & v = +1 \\ 0 & v = 0 \\ -a \cdot \rho_\tau & v = -1 \end{cases}$$

where $\rho_\tau \in \mathbb{Q}_{>0}$ is the token-denomination weight looked up by content match: the entry in `config.tokens` whose `particle` equals $\tau$ provides `weight`. Conforming compilers reject snapshots where any signal references a $\tau$ absent from the `config.tokens` table. Negative effective stake is clipped to zero before any matrix construction (see §4.4).

---

## 3. Clifford Extensions

Formal extension of selected [[cybergraph]] primitives from scalar to multivector form (scalar + bivector). Restores directional and structural information reduced to scalar weights in the base cybergraph spec. Strictly additive: the scalar part of every extended quantity reproduces the existing cybergraph specification byte-for-byte. When the input `.graph` carries no bivector data (all $w_2 = 0$), all Clifford terms are zero and CT-1 output is byte-identical to a scalar compile.

Reference: Ji, Z. *CliffordNet: All You Need is Geometric Algebra.* arXiv 2601.06793, 2026 — shifted geometric product as $O(N)$ primitive.

### 3.1 Multivector Representation

Each extended quantity is a graded multivector over the Goldilocks field $\mathbb{F}_p$, $p = 2^{64} - 2^{32} + 1$:

$$M = M_0 + M_2 \in \mathbb{F}_p \oplus \bigwedge\nolimits^{\!2}\!\mathbb{F}_p^D$$

where $M_0 \in \mathbb{F}_p$ is the scalar grade, $M_2 \in \bigwedge^2 \mathbb{F}_p^D$ is the bivector grade, and $D$ is the ambient dimension of the enclosing context (vocabulary size, embedding dimension, etc.). Grades 1 (vectors) and higher than 2 are not used in this spec.

The bivector is stored as a sparse list of coefficients indexed by ordered basis pairs:

$$M_2 = \sum_{1 \leq i < j \leq D} c_{ij}\, e_i \wedge e_j, \quad c_{ij} \in \mathbb{F}_p$$

Canonical encoding: ordered pairs $(i, j)$ with $i < j$. Wrap-around from shift operators (§3.5) is resolved by sign flip rather than reordering.

### 3.2 Legacy Scalar Form

For any quantity $X$ defined in [[cybergraph]] as a scalar, its extended form is:

$$X^{\star} = X^{\star}_0 + X^{\star}_2$$

with the conformance invariant $X^{\star}_0 \equiv X$: the scalar part equals the legacy scalar value exactly. All theorems in [[cybergraph]] (T1–T4), axiom A5 (conservation), and existing determinism predicates continue to hold on the scalar grade.

### 3.3 Extended Axon Weight

Legacy form from [[cybergraph]] §Raw Adjacency:

$$w(p, q) = \sum_{\ell : \mathrm{src}(\ell) = p,\, \mathrm{tgt}(\ell) = q} r(\tau(\ell)) \cdot a(\ell)$$

Extended form:

$$w^{\star}(p, q) = w_0(p, q) + w_2(p, q)$$

Scalar grade (unchanged): $w_0(p, q) = \sum_{\ell} r(\tau(\ell)) \cdot a(\ell)$

Bivector grade:

$$w_2(p, q) = \sum_{\ell} r(\tau(\ell)) \cdot a(\ell) \cdot v(\ell) \cdot (e_p \wedge e_q)$$

where $v(\ell) \in \{-1, 0, +1\}$ is the [[valence]] of the cyberlink and $e_p, e_q$ are basis vectors indexed by the particle indices of $p$ and $q$ in particle set $P$.

The bivector coefficient $c_{pq} = \sum_\ell r(\tau) a v$ encodes signed valence consensus on edge $(p, q)$: positive means affirmative consensus ($v = +1$), negative means contested ($v = -1$), zero means balanced or zero-valence throughout. The bivector is oriented: $w_2(p, q) = -w_2(q, p)$ as bivectors, capturing directional asymmetry the scalar form conflated.

### 3.4 Extended Effective Adjacency

Legacy form from [[cybergraph]] §Effective Adjacency:

$$A^{\mathrm{eff}}_{pq} = \sum_{\ell} a(\ell) \cdot \kappa(\nu(\ell)) \cdot f(m(\ell))$$

where $\kappa$ is [[karma]] and $m(\ell) \in [0, 1]$ is the [[inversely coupled bonding surface|ICBS]] market belief.

Extended form:

$$A^{\mathrm{eff}\star}_{pq} = A^{\mathrm{eff}}_{0,\,pq} + A^{\mathrm{eff}}_{2,\,pq}$$

Scalar grade (unchanged): $A^{\mathrm{eff}}_{0,\,pq} = \sum_{\ell} a(\ell) \cdot \kappa(\nu(\ell)) \cdot f_0(m(\ell))$

Bivector grade:

$$A^{\mathrm{eff}}_{2,\,pq} = \sum_{\ell} a(\ell) \cdot \kappa(\nu(\ell)) \cdot f_2(m(\ell)) \cdot \mathrm{sign}(v(\ell)) \cdot (e_p \wedge e_q)$$

where the confidence mapping $f_2(m) = |2m - 1|$ is zero at maximum uncertainty ($m = 0.5$) and one at full market confidence in either direction.

CT-1 scalar path reads $A^{\mathrm{eff}}_0$, discards $A^{\mathrm{eff}}_2$ (when `wedge_attention = false`). CT-1 Clifford path reads both grades; the bivector participates in wedge-augmented attention (§8.7).

### 3.5 Shifted Geometric Product

The primitive local operator for all downstream Clifford consumers. For feature tensors $H, C \in \mathbb{F}_p^{N \times D}$ and shift offset $s \in \{1, 2, \ldots, D-1\}$:

Shifted inner product:

$$\mathrm{Inner}_s(H, C)_{i, c} = \sigma\!\left( H_{i, c} \cdot C_{i,\, (c+s) \bmod D} \right)$$

where $\sigma$ is the SiLU activation over $\mathbb{F}_p$ via lookup table (see [[Goldilocks field processor]] LUT primitive).

Shifted wedge product (strictly anti-symmetric):

$$\mathrm{Wedge}_s(H, C)_{i, c} = H_{i, c} \cdot C_{i,\, (c+s) \bmod D} - H_{i,\, (c+s) \bmod D} \cdot C_{i, c}$$

The anti-symmetry property $\mathrm{Wedge}_s(H, H) \equiv 0$ is the defining characteristic of the exterior product; see conformance predicate C3 in §3.7.

Full shifted geometric product:

$$[H C]_s = \mathrm{Inner}_s(H, C) \oplus \mathrm{Wedge}_s(H, C)$$

where $\oplus$ is channel-wise concatenation followed by a learnable linear projection back to $\mathbb{F}_p^D$. For a shift set $S = \{s_1, \ldots, s_k\}$:

$$\mathrm{Clifford}(H, C; S) = \mathrm{Linear}\!\left( \bigoplus_{s \in S} [HC]_s \right) \in \mathbb{F}_p^{N \times D}$$

Complexity: $O(N \cdot D \cdot |S|)$ time, $O(N \cdot D)$ space. Default shift set: $S = \{1, 2, 4, 8, 16\}$ — logarithmic, giving global channel mixing in $O(\log D)$ hops.

Jet mapping: compiles to two [[nox]] jets — `shifted_inner_product` (extending the existing `geometric_product` jet) and `shifted_wedge_product` (new, same arithmetic structure with subtraction).

### 3.6 Backward Compatibility

A scalar-only consumer reads $X^{\star}_0$ and discards $X^{\star}_2$ for every extended quantity. CT-1 scalar path (`wedge_attention = false`, `clifford_mlp = false`) produces output byte-identical to the unextended cybergraph specification.

Storage format: extended quantities serialize as two adjacent sections — scalar section (identical to legacy binary layout) and bivector section (sparse CSR of `(i, j, coefficient)` triples, ordered by `(i, j)` lexicographically, omitted entirely when empty). A `.graph` snapshot with an empty bivector section is bit-identical to a legacy snapshot.

Hash invariance: $H(G^{\star}) |_{G_2 = 0} = H(G)$ — the hash of an extended graph with zero bivector part equals the legacy hash, preserving content-addressing (axiom A1) for the legacy subset.

### 3.7 Clifford Conformance Predicates

These are local structural invariants verified during compilation; see §12.6 (P-CLIFFORD) for the compiled-model conformance check.

C1 — Scalar identity: $w^{\star}_0(p, q) \equiv w(p, q)$ and $A^{\mathrm{eff}\star}_0(p, q) \equiv A^{\mathrm{eff}}(p, q)$ for every $(p, q)$.

C2 — Bivector anti-symmetry: $w^{\star}_2(p, q) = -w^{\star}_2(q, p)$ when interpreted as bivectors.

C3 — Shifted product anti-symmetry: $\mathrm{Wedge}_s(H, H) \equiv 0$ for every $H, s$.

C4 — Determinism: two independent runs on the same $G^{\star}$ produce byte-identical extended quantities (same bivector coefficients in the same order).

C5 — Jet equivalence: the shifted geometric product via [[nox]] jets yields the same coefficients as a reference scalar-field implementation.

---

## 4. Pass 1 — Particle Index

### 4.1 Procedure

1. Initialize $V := \emptyset$, an ordered set.
2. Seed from vocab refs. For each `[[vocab]]` entry in `config` in declared order, load the referenced [[cyb-vocab|.vocab]] file (a particle dictionary). For each entry in the vocab file, in file order, insert its particle into $V$ if absent. Vocab data bytes (when present) are recorded for `vocab` section emission in §11.6 but do not affect id assignment.
3. Append from signals. Iterate $\mathcal{S}$ via the `links()` iterator. For each $\ell = (\nu, p, q, \ldots)$: insert $p$, then $q$, then $\text{axon}(p, q)$ into $V$ if absent.
4. Assign $\text{idx}: V \to \{0, 1, \ldots, |V|-1\}$ in insertion order.

### 4.2 Output

`vocab.json` — the JSON object $\{ \text{particle}_{\text{hex}} \mapsto \text{idx} \}$ with keys lowercase-hex-encoded. The compiled `.model`'s `vocab` section contains the same id assignment.

### 4.3 Determinism

Insertion order is fixed by (vocab refs in declared order) then (snapshot signal order). Two compilers seeing the same `.graph` and the same referenced `.vocab` files produce the same $\text{idx}$. Snapshots that share a `[[vocab]]` reference yield models with stable, comparable token id assignments — a particle has the same id across compiles that pull the same vocab.

### 4.4 Adjacency construction

Build $A \in \mathbb{Z}_{\geq 0}^{|V| \times |V|}$ in CSR with

$$A_{\text{idx}(p), \text{idx}(q)} = \sum_{\ell : (p, q) \in \ell, \, w(\ell) > 0} w(\ell)$$

stored as int128 to avoid overflow on long-running chains. $A$ is fed to passes 2 and 3.

---

## 5. Pass 2 — Semcon Discovery

### 5.1 Axon set

$$\Omega = \{ \text{axon}(p, q) : (\nu, p, q, \ldots) \in L \}$$

### 5.2 Label edges

A label edge is any $\ell = (\nu, p, q, \ldots)$ with $q \in \Omega$. The source $p$ is a candidate semcon.

### 5.3 Scoring

For each candidate $p$ appearing as the source of label edges:

$$\text{usage}(p) = \sum_{\ell : \text{label edge}, \text{src}(\ell) = p} w(\ell)$$

$$\text{coverage}(p) = |\{ \text{tgt}(\ell) : \text{label edge}, \text{src}(\ell) = p \}|$$

$$\text{score}(p) = \text{usage}(p) \cdot \log_2(1 + \text{coverage}(p))$$

### 5.4 Registration

The registered semcon set $S \subseteq P$ is

$$S = \{ p : \text{score}(p) \geq \theta \cdot \max_{p'} \text{score}(p') \}$$

with $\theta = 10^{-3}$ (one-thousandth of the strongest semcon by score). Order $S$ by descending score; ties broken by ascending particle hash.

The default semcon is the reserved particle $0x00 \times 32$, denoted $\bot$. It is appended to $S$ at the highest index.

### 5.5 Assignment

For each $\ell = (\nu, p, q, \ldots) \in L$ compute $\alpha = \text{axon}(p, q)$ and

$$\sigma(\ell) = \arg\max_{s \in S \setminus \{\bot\}} \sum_{\ell' : \text{src}(\ell') = s, \text{tgt}(\ell') = \alpha} w(\ell')$$

If the argmax set is empty (no registered semcon labels $\alpha$), $\sigma(\ell) = \bot$. Argmax ties are broken by ascending position of $s$ in $S$.

### 5.6 Output

`semcons.json` — the ordered list $S$ with per-semcon edge count and aggregate stake.

### 5.7 Complexity

$O(|L|)$ time, $O(|S| + |\Omega|)$ extra space.

---

## 6. Pass 3 — Architecture Parameters

### 6.1 Focus distribution

Compute $\phi^* \in \Delta^{|V|}$ by power iteration of the column-stochastic transition matrix $P = A^\top D^{-1}$ (with $D = \text{diag}(A^\top \mathbf{1})$, treating zero-degree rows as teleport):

$$\phi^{(k+1)} = \alpha P \phi^{(k)} + (1 - \alpha) u, \quad \phi^{(0)} = u, \quad u_i = \frac{1}{|V|}$$

with $\alpha = 0.85$. Halt when $\|\phi^{(k+1)} - \phi^{(k)}\|_1 < \varepsilon_\pi$ with $\varepsilon_\pi = 10^{-8}$.

Impulse reuse. If the optional `impulse` extension is present, each signal $s$ carries a sparse focus delta $\Delta\phi^*^{(s)}$ that was proven on chain when the signal was accepted. The base distribution is then

$$\phi^*_{\text{chain}} = \phi^{(0)} + \sum_{s \in \mathcal{S}} \Delta\phi^*^{(s)}$$

where $\phi^{(0)}$ is the genesis prior from `config`. Power iteration is unnecessary for the set of signals covered by impulses; it runs only over the residual adjacency (signals without impulse). On a fully proof-carrying snapshot this skips the entire iteration.

### 6.2 Embedding dimension

Take the singular value spectrum $\Sigma = (\sigma_1, \ldots, \sigma_r)$ of the $\phi^*$-weighted adjacency

$$M = \text{diag}(\sqrt{\phi^*}) \cdot A \cdot \text{diag}(\sqrt{\phi^*})$$

via randomized SVD truncated to rank $r = 1024$ (oversampled). Normalize: $\hat{\sigma}_i = \sigma_i / \sum_j \sigma_j$. Then

$$d^* = \left\lceil \exp\left(- \sum_i \hat{\sigma}_i \log \hat{\sigma}_i\right) \right\rceil$$

Round to the nearest multiple of $h^*$ (see §6.3) and clamp to $[64, 4096]$.

### 6.3 Head count

$$h^* = |S|$$

(includes $\bot$).

### 6.4 Layer count

Compute the spectral gap $\lambda_2$ of the normalized Laplacian $\mathcal{L} = I - D^{-1/2} A D^{-1/2}$ via Lanczos with $k = 32$ iterations. Compute the contraction rate

$$\kappa = \alpha (1 - \lambda_2)$$

Estimate the diameter $\text{diam}(G)$ via BFS from the highest-degree node (lower bound; sufficient for our use). Then

$$L^* = \text{diam}(G) \cdot \left\lceil \frac{\log(1/\varepsilon_L)}{\log(1/\kappa)} \right\rceil$$

with $\varepsilon_L = 10^{-2}$. Clamp $L^* \in [4, 512]$.

### 6.5 Output

`arch.toml`:

```toml
compiler   = "CT-1"
block      = 12345678
particles  = 3143630
d          = 300
h          = 13
L          = 290
kappa      = 0.851
lambda2    = 0.0015
diameter   = 10
```

---

## 7. Pass 4 — Embedding Matrix

### 7.1 Computation

Continue the randomized SVD of $M$ from §6.2 to extract the top $d^*$ left singular vectors $U_{:, 1:d^*}$ and singular values $\Sigma_{1:d^*}$. Set

$$E = U_{:, 1:d^*} \cdot \text{diag}(\sqrt{\Sigma_{1:d^*}}) \in \mathbb{R}^{|V| \times d^*}$$

### 7.2 Determinism

Randomized SVD uses ChaCha20 seeded with $\text{hemera}(L \,\|\, \nu_{\text{compiler}})$ truncated to 32 bytes. Singular vector signs are normalized so the entry of largest absolute value in each column is positive (sign convention SC-1).

### 7.3 Output tensor

`embed.weight` of shape $(|V|, d^*)$, dtype `float32`, row-major.

---

## 8. Pass 5 — Attention Weights

For each layer $l \in \{0, \ldots, L^* - 1\}$ and each semcon $s \in S$ at head index $h_s$:

### 8.1 Per-semcon adjacency

$$A^{(s)}_{ij} = \sum_{\ell : \text{idx}(\text{src}) = i, \text{idx}(\text{tgt}) = j, \sigma(\ell) = s} w(\ell)$$

### 8.2 Layer-specific power

The layer-$l$ semcon adjacency is

$$A^{(s, l)} = (A^{(s)})^{l_{\text{eff}}}, \quad l_{\text{eff}} = 1 + \lfloor l \cdot \text{diam}(G) / L^* \rfloor$$

computed by repeated sparse-times-dense multiplication; never materialized as dense.

### 8.3 Projection into embedding space

$$P^{(s, l)} = E^\top A^{(s, l)} E \in \mathbb{R}^{d^* \times d^*}$$

### 8.4 SVD per head

$$P^{(s, l)} = U^{(s,l)} \Sigma^{(s,l)} V^{(s,l)\top}$$

Truncate to rank $d_h = d^* / h^*$:

$$W_Q^{(l, h_s)} = U^{(s,l)}_{:, 1:d_h} \cdot \sqrt{\Sigma^{(s,l)}_{1:d_h}}$$

$$W_K^{(l, h_s)} = V^{(s,l)}_{:, 1:d_h} \cdot \sqrt{\Sigma^{(s,l)}_{1:d_h}}$$

$$W_V^{(l, h_s)} = E^\top \cdot \text{diag}(\phi^*) \cdot A^{(s)} \cdot E_{:, h_s \cdot d_h : (h_s+1) \cdot d_h}$$

Sign convention SC-1 applied to $U^{(s,l)}, V^{(s,l)}$.

### 8.5 Output projection

$$W_O^{(l)} = (W_V^{(l, 0)} \,\|\, \cdots \,\|\, W_V^{(l, h^*-1)})^\dagger$$

(Moore-Penrose pseudoinverse of the concatenated values, giving the optimal aggregation back to $d^*$.)

### 8.6 Output tensors

Per layer $l$:

- `layers.{l}.attn.q_proj.weight` of shape $(d^*, d^*)$ — concatenation of $W_Q^{(l, h)}$ over $h$
- `layers.{l}.attn.k_proj.weight` of shape $(d^*, d^*)$
- `layers.{l}.attn.v_proj.weight` of shape $(d^*, d^*)$
- `layers.{l}.attn.o_proj.weight` of shape $(d^*, d^*)$

dtype `float32`, row-major.

### 8.7 Wedge-Augmented Attention

CT-1 extends the scalar score $Q K^\top$ with a bivector magnitude term capturing orientation mismatch between query and key. The per-head score at inference time is

$$\mathrm{score}_{ij}^{(l, h)} = \alpha \cdot \frac{Q_i^{(l, h)} \cdot K_j^{(l, h)}}{\sqrt{d_h}} + \beta \cdot \frac{\|Q_i^{(l, h)} \wedge K_j^{(l, h)}\|}{\sqrt{d_h}}$$

where the bivector norm uses the shifted wedge product from §3.5 with the default shift set $S = \{1, 2, 4, 8, 16\}$:

$$\|Q \wedge K\|^2 = \sum_{s \in S} \sum_c \left( Q_c K_{(c+s) \bmod d_h} - Q_{(c+s) \bmod d_h} K_c \right)^2$$

The scalars $\alpha, \beta \in \mathbb{F}_p$ are per-layer learnable (2 additional parameters per layer), initialized at $\alpha = 1, \beta = 0$. When the input `.graph` has no bivector extensions, wedge terms are zero and $\beta$ receives no gradient — the score degenerates to standard dot-product attention with no performance penalty.

The attention weights $W_Q, W_K, W_V, W_O$ in §8.4–8.5 are unchanged. Only the score function adds the wedge magnitude term.

Emitted tensors per layer:
- `layers.{l}.attn.alpha_beta.weight` of shape $(2,)$, dtype `float32`

See §3.7 C3 for the wedge anti-symmetry conformance check.

---

## 9. Pass 6 — MLP Weights

CT-1 uses a Clifford-block MLP per CliffordNet (Ji, 2026). The shifted geometric product from §3.5 followed by a single learnable projection replaces a conventional FFN — no expanded inner dimension, no FFN ratio. When the input `.graph` has no bivector extensions, all wedge terms are zero and the Clifford block degenerates to a standard gated linear unit with no performance penalty.

Per-layer computation:

$$H_{\mathrm{out}} = H + \gamma \odot \left[ \sigma(H) + \mathrm{gate}(H, G) \odot G \right]$$

$$G = \mathrm{Linear}_{\mathrm{proj}} \!\left( \bigoplus_{s \in S} \left[ \mathrm{Wedge}_s(H, C) \,\|\, \mathrm{Inner}_s(H, C) \right] \right)$$

$$C = \mathrm{DWConv}_{3 \times 3} \!\left( \mathrm{DWConv}_{3 \times 3}(H) \right) - \lambda H$$

with:
- shift set $S = \{1, 2, 4, 8, 16\}$ (default, stored in `config`)
- self-energy suppression $\lambda \in \{0, 1\}$ (default $\lambda = 1$, "differential mode")
- $\sigma$ is SiLU over $\mathbb{F}_p$ via LUT (see [[Goldilocks field processor]])
- $\gamma$ is a learnable LayerScale vector of shape $(d^*,)$
- $\mathrm{gate}$ is a sigmoid over a concatenation-then-linear of $(H, G)$

For graph-native compiles, the "DWConv" is replaced by a local graph convolution over the [[cybergraph]] 1-hop neighborhood — the graph Laplacian action $\mathcal{L} H$ — computed by SpMV against the cybergraph adjacency. This preserves spatial-topological fidelity since the cybergraph is the native topology.

### 9.1 Weights emitted

Per layer $l$:

- `layers.{l}.mlp_clifford.proj.weight` of shape $(|S| \cdot 2 d^*, d^*)` — the projection $\mathrm{Linear}_{\mathrm{proj}}$
- `layers.{l}.mlp_clifford.gate.weight` of shape $(2 d^*, d^*)` — gate linear layer
- `layers.{l}.mlp_clifford.gamma` of shape $(d^*,)$ — LayerScale
- `layers.{l}.mlp_clifford.context.weight_1` of shape $(d^*, 3, 3)$ — first DWConv (or graph-conv kernel)
- `layers.{l}.mlp_clifford.context.weight_2` of shape $(d^*, 3, 3)$ — second DWConv

Total MLP parameters per layer: $2 |S| d^{*2} + 2 d^{*2} + d^* + 18 d^*$.

At $|S| = 5$ and $d^* = 300$: $2 \cdot 5 \cdot 300^2 + 2 \cdot 300^2 + 5700 = 1{,}085{,}700$ params per layer.

The Clifford block achieves SwiGLU-equivalent capability at fewer layers ($L^*/2$ to $L^*/3$ in CIFAR-class experiments per the paper), so the total param budget drops proportionally. Run §6.4 with $\lambda_2$ and $\kappa$ recomputed against the Clifford layer contraction rate — the emitted $L^*$ scales down automatically.

### 9.2 Compile determinism

Context DWConv / graph-conv weights are initialized by seeded ChaCha20 per §7.2 with salt `"mlp_clifford"`. The LayerScale $\gamma$ initializes to $10^{-5}$ (fp32). All other weights initialized by He-normal seeded from hemera hash of $(L, \nu_{\mathrm{compiler}}, l)$. Sign convention SC-1 applies to the projection SVD where factorization is used for initialization.

---

## 10. Pass 7 — Norms and Position

### 10.1 Layer norms

For every layer $l$:

- `layers.{l}.input_layernorm.weight` of shape $(d^*,)$, all entries $1.0$
- `layers.{l}.post_attention_layernorm.weight` of shape $(d^*,)$, all entries $1.0$
- `model.norm.weight` of shape $(d^*,)$, all entries $1.0$

### 10.2 Position encoding

RoPE with base $\theta_0 = 10000$, max sequence length 8192. Inverse frequencies are computed at load time from $(\theta_0, d^* / h^*)$; no tensor is stored.

### 10.3 Output head

`lm_head.weight` is tied to `embed.weight` (no separate tensor written).

---

## 11. Pass 8 — Packaging as `.model`

The output of CT-1 is a single `.model` file (see [[cyb-model]]) loadable by the cyb-llm runtime at `~/git/cyb/llm`. The runtime mmaps the file, parses the TOML frontmatter, jumps to the binary `weights` section, and starts inference — no extraction step.

### 11.1 Container layout

`.cyb` three-rule contract: TOML frontmatter, `~~~name` delimiters, `size` for binary sections.

```toml
[cyb]
types = ["model"]
name = "bostrom-23195000-ct1"

[[files]]
name = "card"
format = "md"

[[files]]
name = "config"
format = "toml"

[[files]]
name = "program"
format = "rs"

[[files]]
name = "tensors"
format = "toml"

[[files]]
name = "vocab"
format = "toml"

[[files]]
name = "eval"
format = "toml"

[[files]]
name = "weights"
format = "tensors"
size = 16823492608
```

### 11.2 `card` section

Markdown. Auto-generated from compile inputs:

```markdown
~~~card
# bostrom-23195000-ct1

Compiled from bostrom-23195000.graph at 2026-03-23 14:42 UTC.
Spec: CT-1. d=300, h=13, L=290, params=4.19B.

snapshot particle: hemera:9f3c...
model particle:    hemera:1a2b...
```

### 11.3 `config` section

Compile parameters and architecture, integers only per cyb-model convention.

```toml
~~~config
model_type = "llama"
parameters = 4192804864
license = "cyber license"
languages = []  # graph-native, vocabulary is particles

[architecture]
hidden_size = 300
num_attention_heads = 13
num_key_value_heads = 13
head_dim = 24            # = 300 / 13, rounded
num_hidden_layers = 290
intermediate_size = 1200  # 4 × hidden_size
vocab_size = 3143630
context_length = 8192
max_position_embeddings = 8192
rope_theta = 10000
rms_norm_eps = 1000000   # 1/ε convention; 1e-6

[tokenizer]
type = "particle"        # particle hashes as token ids, not BPE
bos_id = 0
eos_id = 0
pad_id = 0

[sampling]
temperature = 700        # 0.7
top_p = 900              # 0.9
scale = 1000

[clifford]
shift_set              = [1, 2, 4, 8, 16]   # S, see §3.5
self_energy_suppression = 1                 # λ ∈ {0, 1}; 1 = differential mode

[lineage]
spec          = "CT-1"
source        = "hemera:9f3c..."
source_kind   = ".graph"
chain_id      = "bostrom-1"
block         = 23195000
arch_hash     = "hemera:..."
vocab_hash    = "hemera:..."
semcons_hash  = "hemera:..."
```

### 11.4 `program` section

The standard Llama transformer-decoder program from cyb-model.md applies unchanged. CT-1 emits the trident form by default; the `.rs` form is acceptable when proof is not required.

```trident
~~~program
module model.pipeline
use std.nn.transformer_llama  # standard library

pub fn forward(input: Field, output: Field, seq: Field, cfg: Config) {
    transformer_llama.forward(input, output, seq, cfg)
}
```

CT-1 does not emit a custom program. The architecture parameters in `config` parameterize the standard one. Custom programs (e.g. for graph-walk inference instead of token-sequence inference) are reserved for CT-2.

### 11.5 `tensors` section

TOML index keyed by HuggingFace LlamaForCausalLM tensor names. Encoding is `u16` for projections and `u32` for norms by default; cyb-model encoding rules apply (no floats on disk).

```toml
~~~tensors
["model.embed_tokens.weight"]
shape    = [3143630, 300]
encoding = "u16"
offset   = 0
size     = 1886178000

["model.layers.0.self_attn.q_proj.weight"]
shape    = [300, 300]
encoding = "u16"
offset   = 1886178000
size     = 180000

# ... attn k/v/o, mlp up/down, layer norms × 290 layers
```

Tensor names match those listed in §7.3, §8.6, §9.1, §10.1. Storage order: embedding first, then layer 0 through layer $L^*-1$ in struct order, then `model.norm.weight`. `lm_head.weight` is omitted (tied to `embed_tokens`).

### 11.6 `vocab` section

For graph-native compiles the tokenizer type is `particle`: every token id is a particle. The vocab section is the particle index from pass 1 written as a flat table.

```toml
~~~vocab
[tokens]
0 = "0x1a2b3c4d..."
1 = "0x5e6f7a8b..."
2 = "0x9c0d1e2f..."
# ...
```

For particle vocabularies there are no merge rules; the `[merges]` table is omitted.

### 11.7 `eval` section

CT-1 conformance scores per §12, plus optional downstream metrics. Per-mille integers.

```toml
~~~eval
[ct1_conformance]
P_EMBED = 31         # reconstruction error × 1000; 0.031
P_ATTN_min = 810     # min Pearson × 1000
P_ATTN_mean = 890
P_LAYER_max_ratio = 930
P_DET = 1000         # 1 if deterministic, 0 if not
P_LOAD = 1000

[focus]
top_concentration = 1040  # top particle's focus, per-mille of total
```

Updatable by the runtime after benchmark runs, same convention as cyb-model.

### 11.8 `weights` section

Raw tensor data, 4096-byte page-aligned per tensor for zero-copy mmap and `unimem` integration. Encodings follow cyb-model §weights:

| from CT-1 internal | to disk encoding | conversion |
|---|---|---|
| float32 projections | u16 | `round(value * 256)` |
| float32 norms | u32 | `round(value * 65536)` |

Future quantization passes (`q4`/`q8`) are planned for CT-2 and remain u16 in CT-1.

### 11.9 Reproducibility particle

The compiled `.model` file is itself a particle. Its identity is

$$\text{particle}(\mathcal{M}) = \text{hemera}(\text{model file bytes})$$

over the entire `.model` file including frontmatter. Two CT-1 conforming implementations on the same `.graph` snapshot must produce the same particle.

---

## 12. Conformance Predicates

A compile $\mathcal{M}$ is CT-1 conforming on snapshot $G$ iff all the following hold.

### 12.1 Reconstruction (P-EMBED)

$$\frac{\|E E^\top - M\|_F}{\|M\|_F} \leq 0.05$$

### 12.2 Head specialization (P-ATTN)

For every layer $l$ and semcon $s$:

$$\text{Pearson}(\text{flatten}(W_Q^{(l, h_s)} W_K^{(l, h_s)\top}), \text{flatten}(P^{(s, l)})) \geq 0.7$$

### 12.3 Layer contraction (P-LAYER)

For a fixed pseudo-random seed and a length-128 random embedding sequence, layer-to-layer change is monotonically nonincreasing for all $l \geq 1$.

### 12.4 Determinism (P-DET)

Two independent runs of the conforming implementation on the same `.graph` produce byte-identical `.model` files (same particle per §11.9).

### 12.5 Runtime load (P-LOAD)

The cyb-llm runtime at `~/git/cyb/llm` loads the `.model` file via the `.cyb` parser, mmaps the `weights` section, and performs one forward pass of context length 1. The pass returns finite logits and respects the architecture parameters declared in `config`. Reference command:

```
cyb-llm load <output.model> --warmup 1 --check-finite
```

A round-trip extraction to a HuggingFace directory (config.json + model.safetensors) is also supported via `cyb-llm export hf <output.model>` and must succeed for the file to be CT-1 conforming. This guarantees the compiled model is consumable by both the cyb stack and the wider ecosystem.

### 12.6 Clifford Conformance (P-CLIFFORD)

P-CLIFFORD decomposes into three sub-checks. All must pass.

P-CLIFFORD-A — wedge anti-symmetry. For every layer $l$, the shifted wedge operator satisfies $\mathrm{Wedge}_s(X, X) = 0$ numerically to within $\varepsilon_w = 10^{-6}$ on a fixed-seed length-128 random embedding sequence, for every $s \in S$. Corresponds to §3.7 C3.

P-CLIFFORD-B — zero-bivector degeneracy. On an input `.graph` with no bivector extensions ($w_2 = 0$ everywhere), the CT-1 output is byte-identical to a scalar-only compile. Enforces that the Clifford path degenerates correctly when the graph carries no geometric data.

P-CLIFFORD-C — jet equivalence. The shifted geometric product output computed via the [[nox]] jets (`shifted_inner_product`, `shifted_wedge_product`) matches a reference scalar-field implementation within $\varepsilon_j = 10^{-9}$ on a 64-element fixed test vector set emitted by the compiler alongside the `.model`. Corresponds to §3.7 C5.

Stored in the `eval` section (§11.7) as:

```toml
[ct1_conformance_clifford]
P_CLIFFORD_A = 1      # wedge antisymmetry
P_CLIFFORD_B = 1      # zero-bivector degeneracy
P_CLIFFORD_C = 1      # jet-vs-reference equivalence
```

P-CLIFFORD is `1` (pass) if and only if all three sub-checks pass.

---

## 13. Reference Implementation

The reference is [[mc]] (model compilation) at `~/git/mc` — rust, sprs + ndarray, writes `.model` directly via the cyb-format crate from `~/git/cyb/llm`. It depends on no Python and produces no intermediate safetensors — the `.model` file is the only artifact.

Build and run:

```
cd ~/git/mc
cargo build --release
./target/release/mc bostrom-23195000.graph -o bostrom-23195000-ct1.model
```

The certificate is embedded in the `.model`'s `eval` section (§11.7). The CLI also writes a sidecar `certificate.toml` for human inspection:

```toml
# certificate.toml
spec        = "CT-1"
snapshot    = "hemera:..."
output      = "hemera:..."   # the model's particle
P-EMBED     = { value = 0.031, pass = true }
P-ATTN      = { min = 0.81, mean = 0.89, pass = true }
P-LAYER     = { contracting = true, max_ratio = 0.93, pass = true }
P-DET       = { runs = 2, identical = true, pass = true }
P-LOAD      = { cyb_llm_load = true, hf_export = true, finite_logits = true, pass = true }
P-CLIFFORD  = { A_antisym = true, B_zero_bivec = true, C_jet_equiv = true, pass = true }
```

End-to-end pipe from go-cyber to a loaded model in one command:

```
curl -s https://node.bostrom.cybernode.ai/cyber/graph/snapshot?block=23195000 \
  | mc - -o bostrom-latest.model \
  && cyb-llm load bostrom-latest.model
```

---

## 14. Versioning

CT-1 is the current spec. The compiler version string (§2.1) is always `"CT-1"`. The `[clifford]` config block holds structural parameters (`shift_set`, `self_energy_suppression`); there are no on/off feature flags — Clifford geometry is part of the CT-1 architecture. When the input `.graph` carries no bivector data, Clifford terms are zero and the output is byte-identical to a scalar compile.

Backward-incompatible changes increment to CT-2. Changes that are strictly additive and backward-compatible remain CT-1 with updated patch notes here.

Future work:

- multi-label semcon assignment (split-weight variant of §5.5)
- ε-incremental recompile when only $\Delta L$ is supplied
- decoupled shift sets $S_{\mathrm{inner}} \neq S_{\mathrm{wedge}}$ per CliffordNet future-work §6
- learned shift offsets (adaptive geometric topology)
- rotor-RoPE extension to 4D rotors via the quaternion slot of $G(3, 0, 0)$
- q4/q8 quantization passes (CT-2 candidate)

---

see [[compiled transformers]] for the readable how-to. see [[graph-native-transformer]] for the mathematical derivation. see [[cyb-graph]] for the input file format. see [[cyb-model]] for the output file format. see [[cyber/link]] for the cyberlink seven-tuple. see [[cyber/tri-kernel]] for the focus computation. see [[cybergraph]] for the underlying axioms. see [[mir/specs/render]] for the T∞ rendering tier that runs inference on CT-1 output. see [[mc]] for the reference rust implementation.
