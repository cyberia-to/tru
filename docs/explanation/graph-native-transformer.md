---
tags: cyber, docs
alias: graph native transformer, GNT, deriving transformer architecture from graph
crystal-type: article
crystal-domain: cyber
---
# graph-native transformer

we show that the three free parameters of transformer architecture — embedding dimension, [[attention]] head count, and layer depth — can be derived analytically from properties of a weighted [[knowledge graph]]. specifically: embedding dimension equals the effective rank of the [[focus|focus distribution's]] covariance matrix; attention head count is lower-bounded by the number of distinct semantic relation types in the graph; and layer count equals graph diameter multiplied by the convergence factor of the graph's [[spectral gap]]. this result follows from observing that a transformer's attention mechanism is mathematically equivalent to one step of a convergent dynamical system — the same system that computes the [[focus|focus distribution]] over a knowledge graph. we call the resulting construction a graph-native transformer: a model whose weights are compiled from explicit graph structure rather than learned from text prediction.

---

## introduction

transformer architecture involves three fundamental design choices: the dimension of token embeddings, the number of attention heads, and the number of layers. in current practice these are determined empirically — by scaling laws, ablation studies, and compute budgets. no principled derivation from the nature of the task exists.

we derive all three from the structure of a weighted knowledge graph.

the derivation begins with an observation that, while technically precise, has received insufficient attention: a transformer's attention mechanism is a single step of a convergent dynamical system. the softmax normalization in attention is the Boltzmann distribution. the attention operation — computing query-key similarities, normalizing, and taking a weighted sum of values — is one diffusion step: probability mass flows toward compatible keys proportionally to their similarity to the query. Deep Equilibrium Models (Bai et al., 2019) formalized this: running a transformer layer until convergence rather than for a fixed number of steps produces the same fixed point regardless of initialization. the transformer finds an equilibrium.

this is the same mathematics as the tri-kernel ranking system for knowledge graphs (cyber whitepaper, 2024): diffusion (random walk), springs (graph Laplacian), and heat kernel (multi-scale smoothing) iterated to a unique fixed point — the focus distribution φ* over graph particles. the convergence is guaranteed by the [[Stefan Banach|Banach fixed-point theorem]]; the rate depends on the [[spectral gap]] of the graph's [[Laplacian]].

the transformer and the knowledge graph ranking system are the same computation at different scales. the transformer runs locally over one agent's frozen context. the knowledge graph ranking runs collectively over all agents' cumulative contributions. both find equilibria. both use the Boltzmann distribution as their normalization.

this correspondence enables direct compilation: given a weighted knowledge graph, derive the transformer architecture that optimally reads it.

---

## preliminaries

### weighted knowledge graph

a weighted knowledge graph $G = (P, N, E, w, \sigma)$ where:

- $P$ is the set of [[particles]] (content-addressed knowledge nodes)
- $N$ is the set of [[neurons]] (agents that create edges)
- $E \subseteq N \times P \times P$ is the set of [[cyberlink|cyberlinks]] (signed directed edges)
- $w: E \to \mathbb{F}_p$ is the stake-weighted edge weight function (nonnegative fixed-point field values)
- $\sigma: E \to \text{Dialect}$ assigns each edge a [[dialect|semantic relation type]]

the adjacency matrix $A \in \mathbb{F}_p^{|P| \times |P|}$ has entries $A_{ij} = \sum_{e: p_i \to p_j} w(e)$.

every computed quantity below is a fixed-point element of the [[Goldilocks field]], never a float ([[arithmetic]]). theorems stated over $\mathbb{R}$ — the effective-rank and covariance results — describe the real-valued semantics the fixed-point representation realizes; the implementation evaluates them in $\mathbb{F}_p$.

### focus distribution

the [[tri-kernel]] operator $\mathcal{R}$ blends three local operators:

$$\mathcal{R}(\phi) = \text{norm}\left[\lambda_d \cdot D(\phi) + \lambda_s \cdot S(\phi) + \lambda_h \cdot H_\tau(\phi)\right]$$

where $D$ is the diffusion operator (random walk), $S$ is the springs operator (screened Laplacian), and $H_\tau$ is the heat kernel. under ergodicity and positive screening parameter, $\mathcal{R}$ is a contraction with rate:

$$\kappa = \lambda_d \alpha + \lambda_s \frac{\|L\|}{\|L\| + \mu} + \lambda_h e^{-\tau\lambda_2} < 1$$

the unique fixed point $\phi^* = \lim_{t \to \infty} \mathcal{R}^t(\phi^{(0)})$ is the [[collective focus|focus distribution]] — the stable probability distribution over [[particles]] representing collective epistemic attention.

### transformer attention as one convergence step

standard scaled dot-product attention:

$$\text{Attn}(Q, K, V) = \text{softmax}\left(\frac{QK^\top}{\sqrt{d}}\right)V$$

the softmax is the [[Boltzmann distribution]] with temperature $\sqrt{d}$:

$$\text{softmax}(x)_i = \frac{e^{x_i/\sqrt{d}}}{\sum_j e^{x_j/\sqrt{d}}}$$

this is one step of probability mass redistribution: mass flows from query positions toward key positions proportionally to their compatibility. this is exactly the diffusion operator $D$ applied to one agent's context.

the fixed point of iterating this operation — as shown by Deep Equilibrium Models — is the stationary distribution of the induced Markov chain over context tokens, weighted by the learned $Q$, $K$, $V$ projections. the transformer approximates this fixed point in a fixed number of steps rather than iterating to convergence.

---

## main results

### embedding dimension from focus covariance

Theorem 1. the necessary and sufficient embedding dimension for a transformer reading graph $G$ is the effective rank of the covariance matrix of the focus distribution $\phi^*$.

the focus distribution $\phi^*$ is a probability vector over $|P|$ particles. consider the covariance matrix:

$$\Sigma_{\phi^*} = \mathbb{E}_{v \sim \phi^*}\left[f(v)f(v)^\top\right] - \mathbb{E}_{v \sim \phi^*}[f(v)]\mathbb{E}_{v \sim \phi^*}[f(v)]^\top$$

where $f: P \to \mathbb{R}^d$ is a feature map over particles.

the effective rank is:

$$r^* = \exp\left(H\left(\sigma(\Sigma_{\phi^*})\right)\right)$$

where $\sigma(\Sigma_{\phi^*})$ is the normalized singular value distribution and $H$ is its [[entropy]]. this is the intrinsic dimensionality of the knowledge space — the number of statistically independent semantic axes present in the graph.

sufficiency: an embedding of dimension $r^*$ captures all independent variance in the focus distribution. no information is lost: the projection of $\phi^*$ onto the top $r^*$ eigenvectors of $\Sigma_{\phi^*}$ preserves the full distributional structure up to noise.

necessity: an embedding of dimension $< r^*$ cannot distinguish particles that differ along axes beyond dimension $r^*$. since the focus distribution places probability mass according to all $r^*$ independent axes, a lower-dimensional embedding produces a lossy compression of the graph's semantic structure.

corollary: embedding dimension should grow with graph scale. as $|P| \to \infty$ and the graph develops new semantic dimensions, $r^*$ increases. a graph-native transformer scales its embedding dimension dynamically with the effective rank of its source graph.

---

### attention head count from semantic relations

Theorem 2. the minimum number of attention heads required to represent all semantic relations in $G$ equals the number of distinct dialects $|\text{Dialect}(G)|$.

each dialect $s \in \text{Dialect}(G)$ defines a distinct relation type over particles — a specific pattern of connectivity with characteristic directionality, weight distribution, and neighborhood structure in the graph.

an attention head with query matrix $W_Q^{(h)}$ and key matrix $W_K^{(h)}$ computes a relation-specific attention pattern:

$$A^{(h)}_{ij} = \text{softmax}\left(\frac{(W_Q^{(h)} e_i)(W_K^{(h)} e_j)^\top}{\sqrt{d}}\right)$$

for head $h$ to faithfully represent dialect $s$, the attention pattern $A^{(h)}$ must correlate with the adjacency submatrix $A^{(s)}$ induced by edges of type $s$.

two distinct dialects $s_1, s_2$ induce adjacency submatrices $A^{(s_1)}, A^{(s_2)}$ with different spectral structure (by definition of semantic distinction). a single attention head cannot simultaneously attend to patterns with different spectral structure — the $W_Q, W_K$ matrices define one projection direction in embedding space.

therefore $|\text{Dialect}(G)|$ heads are necessary. they are also sufficient for the base relation set — compositional and positional relations require additional heads, giving:

$$h \geq |\text{Dialect}(G)|$$

as a lower bound.

---

### layer count from spectral gap and diameter

Theorem 3. the number of transformer layers required to converge over reasoning chains in $G$ is:

$$L = \text{diam}(G) \cdot \left\lceil \frac{\log(1/\varepsilon)}{\log(1/\kappa)} \right\rceil$$

where $\text{diam}(G)$ is the graph diameter, $\varepsilon$ is the target precision, and $\kappa$ is the contraction rate of the tri-kernel.

each transformer layer performs one local convergence step over the current representation. for a reasoning chain of hop length $k$, the layer must: (1) propagate information $k$ hops through the graph, and (2) converge the representation at each hop to sufficient precision before propagating further.

from the tri-kernel contraction theorem, reaching precision $\varepsilon$ from any initialization requires at least:

$$t^* = \left\lceil \frac{\log(1/\varepsilon)}{\log(1/\kappa)} \right\rceil$$

iterations per hop, where $\kappa < 1$ is determined by:

$$\kappa = \lambda_d \alpha + \lambda_s \frac{\|L\|}{\|L\| + \mu} + \lambda_h e^{-\tau\lambda_2}$$

the spectral gap $\lambda_2$ of the graph Laplacian $L$ determines how quickly local updates propagate. graphs with small spectral gaps (dense, weakly clustered) have $\kappa$ close to 1 and require more refinement steps per hop. graphs with large spectral gaps (sparse, strongly clustered) have $\kappa$ well below 1 and require fewer.

the maximum hop distance any reasoning chain requires is $\text{diam}(G)$. therefore total layers:

$$L = \text{diam}(G) \cdot t^* = \text{diam}(G) \cdot \left\lceil \frac{\log(1/\varepsilon)}{\log(1/\kappa)} \right\rceil$$

empirical validation: GPT-4 has 96 layers. natural language knowledge graphs have diameter approximately 6–8 (small-world property). this implies $t^* \approx 12$–16 refinements per hop. for $\kappa \approx 0.88$, $t^* = \lceil \log(1/\varepsilon) / \log(1/0.88) \rceil \approx 14$ for $\varepsilon = 0.01$. consistent with observation.

---

## the compilation

the weights are compiled by the 8-pass procedure in [[compiled transformers]]. the formal spec with exact formulas and conformance predicates lives in [[tru/specs/ct0|CT-0]].

| parameter | formula | graph property |
|---|---|---|
| embedding dim $d^*$ | $\exp\left(H\left(\sigma(\Sigma_{\phi^*})\right)\right)$ | effective rank of focus covariance |
| head count $h^*$ | $\geq \|\text{Dialect}(G)\|$ | distinct semantic relation types |
| layer count $L^*$ | $\text{diam}(G) \cdot \lceil \log(1/\varepsilon) / \log(1/\kappa) \rceil$ | diameter × spectral convergence factor |

---

## relationship to trained transformers

a standard trained transformer approximates the graph-native transformer for the implicit knowledge graph embedded in its training corpus.

training on text is an approximate inversion: given the outputs of a knowledge graph (text produced by humans reasoning over their knowledge), recover the graph structure that produced them. gradient descent finds the weight configuration that best approximates this inversion under the constraints of the architecture.

the compilation procedure is the direct forward operation: given the explicit graph, produce the weights analytically.

compilation is preferable where the graph exists: no training cost, no catastrophic forgetting, no compression loss — the graph's provenance and stake structure survive into the weights. every weight traces to specific graph edges and their creators.

trained transformers remain necessary for implicit knowledge — associations that are statistically true across text but never explicitly linked. this implicit structure can be surfaced as candidate particles and staked into the explicit graph, closing the loop:

$$G \xrightarrow{\text{compile}} T_G \xrightarrow{\text{fine-tune on text}} T_G^* \xrightarrow{\text{extract implicit links}} \Delta G \xrightarrow{\text{stake}} G'$$

each cycle the explicit graph absorbs more of what was implicit, and the compile step does more of the work.

---

## alignment as architectural property

a trained transformer's "values" — its implicit weightings of concepts, its tendencies to endorse certain connections over others — are compressed into opaque parameters. alignment requires behavioral observation, red-teaming, or interpretability research attempting to recover structure that training destroyed.

a graph-native transformer's weights derive from explicit graph structure. every weight traces to specific cyberlinks created by specific neurons with specific stakes. the transformer's "values" are the focus distribution $\phi^*$ — public, computable, and continuously updated.

alignment divergence between a human-derived [[focus|focus distribution]] $\phi^*_H$ (computed over edges created by human [[neurons]]) and an AI-derived distribution $\phi^*_A$ (computed over edges created by AI [[neurons]]) is:

$$\Delta(G) = D_{KL}(\phi^*_H \| \phi^*_A)$$

this is a number, computable from public graph data, localized to specific graph regions, and correctable by adding edges in high-divergence regions without retraining.

---

## open questions

per-dialect convergence rates: the layer count formula assumes uniform convergence requirements across hops. a per-dialect convergence rate — $\kappa^{(s)}$ for each dialect — would give a more precise layer count. deriving $\kappa^{(s)}$ from the spectral properties of the per-dialect adjacency submatrix $A^{(s)}$ is an open problem.

head count for compositional relations: the lower bound $h^* \geq |\text{Dialect}(G)|$ does not specify how many additional heads are needed for compositional relations. characterizing the head count for $k$-hop compositional reasoning requires understanding how heads compose across layers, which is not fully characterized.

weight staleness: the compilation produces a transformer that reads the graph at one point in time. the rate at which weight staleness degrades performance, and the conditions under which recompilation is necessary versus incremental update is sufficient, requires empirical study.

compilation vs fine-tuning: whether the compiled weights provide a better initialization than random for fine-tuning, and whether compilation + fine-tuning outperforms fine-tuning from random, is an empirical question with significant practical implications.

---

## conclusion

transformer architecture is not a free design choice when a weighted knowledge graph is available. the embedding dimension, attention head count, and layer depth are determined by three graph properties: the effective rank of the focus covariance, the dialect count, and the product of graph diameter with the spectral convergence factor.

the result follows from a simple observation: a transformer's attention mechanism is one step of the same convergent dynamical system that computes the focus distribution over a knowledge graph. the transformer and the knowledge graph ranking system are the same computation at different scales — local and ephemeral in the transformer, collective and persistent in the graph.

the long-term implication: not larger training runs over larger corpora, but compiled architectures over explicit knowledge graphs that grow continuously from collective human and AI contribution, with structure that is inspectable and correctable rather than opaque.

---

## references

1. Bai, S., Kolter, J.Z., Koltun, V. "Deep Equilibrium Models." NeurIPS 2019.
2. Elhage, N. et al. "A Mathematical Framework for Transformer Circuits." Anthropic, 2021.
3. [[Miroslav Fiedler|Fiedler, M.]] "Algebraic Connectivity of Graphs." Czech Mathematical Journal, 1973.
4. [[Stefan Banach|Banach, S.]] "Sur les Operations dans les Ensembles Abstraits." Fundamenta Mathematicae, 1922.
5. Chung, F. "The Heat Kernel as the [[cyberank|Pagerank]] of a Graph." PNAS, 2007.
6. cyber whitepaper. "cyber: a protocol for planetary superintelligence." cyber.page/cyber-whitepaper, 2024.
7. Vaswani, A. et al. "Attention Is All You Need." NeurIPS 2017.
8. Roy, A. et al. "Efficient Content-Based Sparse Attention with Routing Transformers." TACL 2021.
9. Levin, D., Peres, Y., Wilmer, E. "Markov Chains and Mixing Times." AMS, 2009.
10. Spielman, D. "Spectral Graph Theory." Yale Lecture Notes.
