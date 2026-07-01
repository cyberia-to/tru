---
tags: cyber, tru, core, spec
crystal-type: measure
crystal-domain: cyber
alias: superadditivity, collective intelligence, emergence, sigma-mean, sigma-best, collective advantage
---
# superadditivity

collective intelligence is the claim that the whole sees what no part can. tru makes it a number. superadditivity is the advantage of the collective [[focus]] $\phi^*$ — the [[tri-kernel]] over the whole [[cybergraph]] — over the focus an individual [[neuron]] computes from its own local view. when it is positive, the network is provably more than the sum of its neurons.

this is the operational test of the [[collective focus theorem]]: that theorem proves $\phi^*$ exists, is unique, and converges; superadditivity measures that it is emergent. [[syntropy]] is the task-free version of the same fact — superadditivity grounds it in a task.

## the ego baseline

each [[neuron]] $\nu$ holds a local view: the sub-cybergraph of its own [[cyberlinks]] and their endpoints, the ego-net at radius $r$. running the same [[tri-kernel]] on that view alone yields the ego focus $\phi^*_\nu$ — the sharpest picture $\nu$ can form without the rest of the network. the collective focus $\phi^*$ is the tri-kernel over the full weighted graph. superadditivity compares the two.

## the metric

fix a task with a quality score $Q \in [0,1]$, higher better (see tasks). let $Q(\phi)$ be the score when [[particles]] are ranked by focus $\phi$. define

$$\sigma_{\text{mean}} = Q(\phi^*) - \tfrac{1}{|N|}\sum_{\nu} Q(\phi^*_\nu), \qquad \sigma_{\text{best}} = Q(\phi^*) - \max_\nu Q(\phi^*_\nu).$$

$\sigma_{\text{mean}} > 0$ says the collective beats the typical neuron. $\sigma_{\text{best}} > 0$ is the strong claim — the collective beats every individual, the signature of genuine emergence. ($\sigma$ here is superadditivity, local to this page; it is not the signal proof $\sigma$ nor the dialect map $\sigma(\ell)$ of [[ct0]].)

## tasks

two task families ground $Q$ using only the graph, no external labels:

- link prediction — hide a fraction of [[cyberlinks]], score candidate pairs $(p,q)$ by a focus-derived affinity, measure ROC-AUC and average precision. tests whether $\phi^*$ captures latent structure.
- retrieval@k — for a query particle, rank the rest by focus-weighted proximity and measure the precision of the top $k$ against held-out neighbours. tests whether $\phi^*$ concentrates attention on the right particles.

both are self-contained, consistent with tru's epistemics: the graph grades itself.

## connection to syntropy

[[syntropy]] $J(\phi^*) = D_{\mathrm{KL}}(\phi^* \,\|\, u)$ is the task-free measure — how far collective focus departs from uniform noise. superadditivity is its task-grounded sibling. a sharper $\phi^*$ (higher $J$) concentrates attention on real structure, which is what raises $Q$, so $\sigma$ tracks $J$. $J$ answers "is the focus structured?"; $\sigma$ answers "is that structure useful, and more useful than any single neuron's?" both rise together as the graph accumulates honest [[cyberlinks]].

## the generalized collective focus theorem

the [[collective focus theorem]] ([[tri-kernel]] §3) proves $\phi^*$ exists, is unique, and converges. the generalization concerns its quality. let $\lambda_2$ be the algebraic connectivity (Fiedler value) of the weighted graph. adding a non-redundant [[cyberlink]] — one bridging otherwise weakly-connected regions — raises $\lambda_2$.

claim, as measured: $\sigma$ increases with $\lambda_2$ — the collective advantage grows with connectivity. $J(\phi^*)$ does the opposite: it *falls* as $\lambda_2$ rises.

the same $\lambda_2$ already governs convergence: the heat term contracts at rate $e^{-\tau\lambda_2}$ inside $\kappa$ ([[tri-kernel]] §2.2). so algebraic connectivity sets how fast $\phi^*$ converges and how much collective *advantage* $\sigma$ it carries. but it does not raise syntropy — adding edges spreads focus toward uniform, and a more uniform $\phi^*$ has *less* $J$. the original conjecture ("$J$ and $\sigma$ both rise with $\lambda_2$") conflated two different things: connectivity grows the collective's edge over any individual, while sharpness (syntropy) comes from structure being concentrated, which dense connectivity dilutes.

status (benchmark-tested, Karate Club, fixed-vertex spanning-tree sweep — [`rs/examples/superadditivity.rs`](../rs/examples/superadditivity.rs)):
- $\lambda_2$–convergence: proven (§2.2).
- $\sigma$–$\lambda_2$: supported. Pearson $+0.5$ ($\sigma_{\text{mean}}$), and $\sigma_{\text{best}} > 0$ at every connectivity level — the collective beats its strongest neuron throughout.
- $J$–$\lambda_2$: refuted. Pearson $\approx -0.7$; $J$ decreases monotonically as edges are added. The sparse spanning tree is the *most* syntropic state; densification lowers $J$.

open: whether the $\sigma$–$\lambda_2$ rise holds on larger graphs and every operator blend $(\lambda_d,\lambda_s,\lambda_h)$; and the right way to state the corrected law (syntropy rewards concentration, superadditivity rewards connectivity — they are distinct axes).

## benchmark

the measurement harness, smallest sanity instance first:

1. take a graph — Zachary's Karate Club (34 particles) is the minimal sanity check; the target is the live cybergraph at $10^6+$ cyberlinks.
2. compute the collective $\phi^*$ and $J(\phi^*)$.
3. for each neuron, compute its ego $\phi^*_\nu$.
4. score $Q$ on link-prediction and retrieval@10 for $\phi^*$ and every $\phi^*_\nu$.
5. report $\sigma_{\text{mean}}$, $\sigma_{\text{best}}$, $J(\phi^*)$, and their growth as edges are added in connectivity order ($\lambda_2$ rising).

a passing result is $\sigma_{\text{best}} > 0$ that grows with $\lambda_2$ — the collective strictly outperforming its strongest member, by more as the graph connects. concrete figures are benchmark output, reported only once measured on the conformant engine ([[arithmetic]], fixed-point), never asserted in this contract.

## why it matters

emergence stops being a slogan and becomes a number that runs on a phone. on sparse graphs — the realistic regime for a decentralized network of millions of cyberlinks — the locality-bounded tri-kernel measures superadditivity at $O(\deg)$ cost per update, where a dense graph transformer pays $O(n^2)$ and centralizes. the collective wins not by being a bigger model but by wiring many small local views into one focus none of them held alone.

> many small lights, once wired together, see farther than a single sun.

see [[syntropy]] for the task-free measure · [[collective focus theorem]] for existence and uniqueness · [[tri-kernel]] §2.2 for the $\lambda_2$ that ties convergence to intelligence · [[rewards]] for why non-redundant links are what pays.
