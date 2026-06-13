---
tags: cyber, tru, roadmap, spec
crystal-type: spec
crystal-domain: cyber
status: proposal
alias: motif-aware focus, motif-aware compilation, higher-order focus, CT-motif
---
# Motif-Aware Focus and Compilation

formal proposal. extends [[cyber/tri-kernel]] and [[ct0|CT-0]] so that [[focus]] and the compiled transformer perceive non-linear subgraph structure (motifs) that the current pairwise pipeline is provably blind to. companion to the neural frontier survey (`neural/docs/frontier.md`, finding §5).

## 1. The problem — the compile is motif-blind

[[ct0|CT-0]] derives the entire architecture from the raw pairwise adjacency $A$:

- φ\* by power iteration of $P = A^\top D^{-1}$ (CT-0 §5.1; [[cyber/tri-kernel]] §1.1)
- the embedding by SVD of the φ\*-weighted adjacency $M = \mathrm{diag}(\sqrt{\phi^*})\,A\,\mathrm{diag}(\sqrt{\phi^*})$ (CT-0 §5.2)
- the attention heads from per-dialect adjacency $A^{(s)}$ (CT-0 §7)

every one of these is a function of pairwise edges. message passing and diffusion of this form are bounded by the 1-Weisfeiler-Leman test (Xu et al. 2019; Morris et al. 2019): they provably cannot count triangles or cycles, and cannot distinguish a node closed in a triangle from a node on an open chain when their WL colours match. φ\* assigns them identical focus; the embedding gives them identical roles. the compiled transformer inherits the blindness — corroboration, shape, analogy, and foresight are absent from the model, and no amount of downstream training recovers structure that was never in the weights.

### 1.1 The completeness theorem is consistent with the blindness

[[cyber/tri-kernel]] §4 proves that $\{M, L, H_\tau\}$ span every local LINEAR operator on a graph. this is true and it is exactly the boundary: the span of local linear operators on a node signal IS the 1-WL class. motif counts (subgraph-participation features, e.g. the diagonal of $A^3$ read as a feature, graphlet orbit counts) are not local linear operators on a signal — they live outside $\{M, L, H_\tau\}$. so the theorem does not contradict the blindness; it locates it. extending the basis is the work.

## 2. What this buys, in plain terms

take two concepts $A$ and $B$, each with two neighbours. $A$'s neighbours are linked to each other (a triangle — mutual corroboration); $B$'s are not (a chain — a lone bridge). to 1-WL they are identical, so today they compile to the same focus and the same embedding role; the model cannot tell a corroborated cluster from a lone bridge.

motif-awareness fixes this two ways, which are complementary:

- reweighting (a sharper formula). compute φ\* on a motif-weighted graph, so a concept inside many triangles outranks an equally-connected concept that is not. the model attends to corroborated concepts, not merely popular ones.
- features (a motif vocabulary the model carries). give each token its motif-participation as part of its embedding ("in 5 triangles, hub of 1 star, on 2 cycles"). $A$ and $B$ now embed differently.

applied payoff at inference: confidence (corroborated vs lone), analogy (match concepts by structural role, not neighbour overlap), and foresight (predict the link that completes a near-finished motif — the Arrival engine).

## 3. The fit — CT-0 is already graded

CT-0 is not a scalar compile; it is geometric-algebra valued (CT-0 §2.5–2.6): a scalar grade (grade 0, stake-weighted sum) and a bivector grade (grade 2, $w_2 = \sum r\,a\,v\,(e_p \wedge e_q)$ — an oriented edge). a $k$-node motif is the next grade: a triad is a trivector $e_p \wedge e_q \wedge e_r$ (grade 3), a $k$-motif a $k$-vector. geometric algebra is the algebra of oriented simplices — the same higher-order / Hodge structure the literature points to. so motif-awareness is the grade-2 → grade-3+ extension of the algebra CT-0 already chose, and the degeneracy discipline already exists: CT-0 §13 / P-CLIFFORD-B already specify "when higher grades are zero, output is byte-identical to a scalar compile."

## 4. Integration points (by leverage)

1. φ\* on a motif-weighted adjacency. run the [[cyber/tri-kernel]] update and CT-0 §5.1 power iteration on $W_M$ (the motif-weighted graph: edge weight = co-participation in a chosen motif; Benson, Gleich, Leskovec, Science 2016) instead of raw $A$. focus then concentrates on motif-rich rather than degree-rich nodes. foundational — φ\* feeds everything below.
2. higher Clifford grades (CT-0 §2.5–2.6). extend the graded weight $w = w_0 + w_2$ to $w_0 + w_2 + w_3 + \dots$; the trivector grade is triadic consensus. wedge-augmented attention (§7.7) and the Clifford MLP (§8) then carry motif structure natively, making the compiled transformer super-WL.
3. motif-orbit features in the embedding (CT-0 §5.2 / Pass 4). concatenate per-node graphlet-orbit counts to the SVD input (Graph Substructure Networks, Bouritsas et al. 2022) — provably exceeds WL; each token carries its motif role.
4. higher-order Laplacian in the MLP context (CT-0 §8). the graph-Laplacian action $\mathcal{L}H$ becomes the Hodge Laplacian $L_1 / L_2$ on the simplicial lift, so the model's context sees triangles and cycles.
5. conformance predicate P-MOTIF (CT-0 §11). the compile must give distinct output on a pair of graphs that 1-WL cannot distinguish (a triangle-count or cycle-basis check). this is the formal guarantee that the blindness is removed.

minimal high-leverage set: (1) + (2). the rest are consequences.

## 5. The formal extension

generalize the operator basis. [[cyber/tri-kernel]] currently spans $\{M, L, H_\tau\}$ on grade-1 signals (1-WL). the extension adds the higher-order operators on the simplicial lift:

$$\{M_k, L_k, H_{\tau,k}\} \quad \text{for } k = 0, 1, 2, \dots$$

where $L_k$ is the $k$-th Hodge Laplacian on $k$-cells (motifs), and φ\* becomes the fixed point of the composite operator over the lift. equivalently, at grade 1, replace $A$ with the motif-weighted $W_M$. the contraction/convergence guarantees of §2.2 must be re-derived on the lift (open question §8).

which motifs are first-class is discovered, not hand-set: the canonical motif set is the one that most compresses the [[cybergraph]] (minimum description length; Liu et al. 2024) and/or the motif-conductance communities (Benson 2016). a small bootloader motif set (triad, star, chain, diamond, cycle, feed-forward) may be seeded, mirroring the bootloader dialects.

## 6. Backward compatibility

when the input `.graph` carries no higher-grade or motif data, all $M_{k\geq 1}$, $w_{k\geq 3}$, and orbit features vanish and the output is byte-identical to the current CT-0 compile. same discipline as the existing bivector degeneracy (P-CLIFFORD-B). motif-awareness is additive.

## 7. Scope and touched specs

a CT-2 architecture feature, deliverable additively on CT-0. it touches:

- `tru/specs/ct0.md` — graded weights §2.5–2.6 (add trivector+), φ\* §5.1 (motif-weighted), embedding §5.2 (orbit features), MLP §8 (Hodge Laplacian), conformance §11 (P-MOTIF)
- `tru/specs/tri-kernel.md` — the operator basis §4 (extend to higher-order); convergence §2.2 on the lift
- [[cybergraph]] / `.graph` — whether motif grades are stored in the snapshot or computed at compile time
- [[nox]] jets — `shifted_wedge_product` exists for grade 2; a grade-3 wedge jet is needed
- [[mc]] — the reference rust compiler

## 8. Resolutions

the four design questions, mostly closed.

### 8.1 the motif set — a pipeline, not a choice

the three criteria answer different questions; compose them. persistence GATES (real vs noise — a homology class born early and surviving scale; zigzag gives birth/death = the foresight signal). MDL SELECTS the vocabulary (the motif set that most compresses the graph — parameter-free, the same eigenvector as φ\*). motif-conductance (Benson) PARTITIONS into dialects (downstream of selection). pipeline: `candidates → persistence gate → MDL ranking → conductance clustering`.

the $k$-vector blow-up is illusory. the graph is sparse, so the grade-$k$ structure is supported only on the $O(|\text{lexicon}_k|)$ discovered motifs, never $\binom{|P|}{k}$. carry grades up to 3 first-class — link (1), axon-bivector (2), triad-trivector (3); the motif zoo is triad-dominated (feed-forward loop, bi-fan). grade 4+ are compositions of grade-$\leq 3$ via the motif algebra (operadic substitution), not dense grades. assembly-index × copy-number bounds which higher motifs are worth keeping.

### 8.2 convergence — preserved, with a coupling bound

the §2.2 Banach proof is adjacency-agnostic: it holds for any non-negative ergodic operator. so φ\* on the motif-weighted $W_M$ (PSD Laplacian, ergodic via teleport $\alpha$) contracts unchanged. cost: only the heat term's rate $e^{-\tau\lambda_2}$ degrades, since $W_M$ often has a smaller spectral gap (cleaner communities = slower intra-mixing) — compensate by raising $\lambda_d/\lambda_s$ or $\tau$, as diffusion and springs contract at $\alpha$ and $\|L\|/(\|L\|+\mu)$ independent of $\lambda_2$.

the Hodge lift, two-sided coupling — short proof. write the joint state $\phi = \bigoplus_k \phi_k$ over grades and the joint operator $R = D + \beta C$, where $D$ is block-diagonal (the per-grade tri-kernels, each contracting: $\|D\| \le \kappa_{\max} = \max_k \kappa_k < 1$) and $C$ is the off-diagonal boundary coupling ($\phi_k \leftarrow \partial_k\phi_{k-1}$ and $\phi_k \leftarrow \partial_{k+1}^\top\phi_{k+1}$) at strength $\beta$. the boundary maps are bounded: $\|\partial_k\|^2 = \|L_k^{\mathrm{down}}\| \le (k+1)\Delta$ for max coface degree $\Delta$, so $\|C\| \le C_\partial < \infty$. then

$$\|R\phi - R\psi\| \le (\kappa_{\max} + \beta\,C_\partial)\,\|\phi - \psi\|,$$

so $R$ is a contraction whenever

$$\beta < \frac{1 - \kappa_{\max}}{C_\partial}.$$

the per-grade margin $1-\kappa_{\max}$ is the budget; any two-sided coupling within it keeps the Banach fixed point, unique and converging linearly at rate $\kappa_{\max}+\beta C_\partial$. ∎ (one-sided read-out coupling is the $\beta\to 0$ limit and needs no condition.)

### 8.3 storage vs compute — compute, with an impulse cache

motifs are derived from links, so storing grades in `.graph` is redundant and staleness-prone. compute them at compile (triad counting is $O(|E|\cdot\deg)$, comparable to the existing power iteration + SVD). but extend CT-0's impulse reuse (§5.1): a signal carries its proven motif-delta, and by the §2.2 locality radius a new link changes only the motifs within $O(\log(1/\varepsilon))$ hops — so motif maintenance is local and cheap, carried as a proven delta like an impulse. snapshots stay light; the compiler skips the higher-order pass on proof-carrying snapshots.

### 8.4 proof — foresight as a verifiable query

yes, via the [[reference_zheng_not_stark|zheng]] verifiable-query compiler (CT-0 §12.8). claim: "link $(p,q)$ has predicted score $s$ under the current motif structure." witness: the local motif counts around $(p,q)$, the dialect's transition kernel $P(m'|m)$, the score computation. proof: the whole prediction is a [[nox]] program — `look` the local motifs (state-read + Brakedown opening), count them (a CCS-expressible polynomial in the local adjacency), look up the transition and compute $s$ (field ops) — so its trace IS the zheng witness (240-byte proof). honest scope: zheng proves the score is correctly computed from the committed graph, not that the future occurs — it certifies the present structure's implication, exactly the Arrival framing. this makes foresight unfakeable and stake-able (predict a link before it forms, provably), tying into the Shapley-of-Δφ\* reward.

### still open

the cross-grade soundness CONSTANT: a tight, graph-dependent bound on $C_\partial$ (hence the admissible $\beta$) for real cybergraph topologies, and whether the motif representation (hypergraph vs simplicial) shifts φ\*-dynamics (Nat. Commun. 2023 says it can).

## References

- Xu, Hu, Leskovec, Jegelka, "How Powerful are Graph Neural Networks?" (ICLR 2019); Morris et al., "Weisfeiler and Leman Go Neural" (AAAI 2019) — the 1-WL bound.
- Bouritsas, Frasca, Zafeiriou, Bronstein, "Improving Graph Neural Network Expressivity via Subgraph Isomorphism Counting" (GSN, TPAMI 2022) — motif counts exceed WL.
- Benson, Gleich, Leskovec, "Higher-order organization of complex networks" (Science 2016) — motif-weighted adjacency, motif conductance.
- Schaub, Barbarossa, Bianconi et al. — Hodge Laplacians / topological signal processing on simplicial complexes.
- Liu et al., "Compression-based inference of network motif sets" (PLOS Comp Bio 2024) — MDL motif discovery.

see [[ct0]] for the compile contract, [[cyber/tri-kernel]] for the focus definition, and `neural/docs/frontier.md` §5 for the survey finding this proposal acts on.
