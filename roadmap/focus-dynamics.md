---
tags: cyber, tru, roadmap, spec
crystal-type: spec
crystal-domain: cyber
status: proposal
alias: focus dynamics, focus trajectory, focus velocity, homeostatic adaptation, tire-out, waxing and waning
---
# Focus Dynamics — trajectory readout and homeostatic adaptation

formal proposal. extends [[focusing]] so that focus is treated as a **moving quantity across epochs**, not only a per-epoch fixed point. two additive layers that share one piece of persisted state:

- **Part A — trajectory readout** (read-only, cheap, safe). expose how $\phi^*$ *moves* epoch to epoch: velocity, attention-shift, trailing baseline, dwell. no change to the tri-kernel; pure post-processing of the $\phi^*$ sequence.
- **Part B — homeostatic adaptation** ("tire-out"). a slow, per-particle excitability gain that damps a particle *because* it has held high focus, so dominance decays and other structure surfaces. produces the waxing/waning + up/down repertoire, and is a first-principles anti-monopoly force.

companion to the neural-frontier survey (the dynamical-systems / low-rank-RNN lens): rich behaviour comes from **structure + adaptation + noise**. cyber has structure (the tri-kernel) and, with Part B, adaptation. it stays deterministic — the "noise" ingredient, if ever added, belongs in the same slow layer (VRF/block-hash pseudo-entropy), **never** in the contraction.

## 0. The invariant this proposal must not break

[[focusing]] runs the tri-kernel to a **unique** fixed point $\phi^*_t$ each epoch, in a fixed step count $T(\varepsilon)$, byte-identical across all neurons and validators. uniqueness is what makes the [[cybergraph]] a shared memory; determinism is what [[reference_zheng_not_stark|zheng]] proves.

**both parts obey one rule: the per-epoch contraction stays untouched.** all dynamics live *between* epochs, in state carried forward like [[cyberank|karma]] already is. within epoch $t$ every new quantity is either (A) a function of already-final $\phi^*$ vectors, or (B) a **constant** multiplier fixed before the iteration starts. so $\phi^*_t$ is still the unique contraction fixed point, still byte-identical, still provable. the repertoire emerges in the *sequence* $\{\phi^*_t\}$, not inside any one solve.

---

## Part A — trajectory readout

### A.1 The gap

today [[focusing]] emits only snapshots of the current epoch (`φ*`, `cyberank`, spectral positions, `syntropy J`, and the within-batch `Δφ*`). nothing carries across epochs, so nothing answers *is this concept rising or falling? did the network's attention just switch? how long has this held?* — the questions the trajectory answers. the information is free; it is discarded every epoch.

### A.2 The one piece of new state

persist the **previous epoch's focus** $\phi^*_{t-1}$ (one $|P|$-length vector) and a **trailing focus** EMA:

$$\bar\phi_t(p) = (1-\beta)\,\bar\phi_{t-1}(p) + \beta\,\phi^*_t(p), \qquad \beta \in (0,1)$$

$\bar\phi$ is the particle's own baseline — its "usual" focus over a window $\sim 1/\beta$ epochs. both vectors live in [[bbg]] and update once per epoch, same discipline as karma. cost: two $|P|$ vectors and one EMA pass. that is the entire footprint of Part A.

### A.3 Concrete new outputs

new rows for the [[focusing]] outputs table. each is a deterministic field-arithmetic function of proven $\phi^*$ vectors, so each is byte-identical and provable exactly like `Δφ*`.

| output | definition | answers | consumer |
|--------|-----------|---------|----------|
| **focus velocity** $\dot\phi^*(p)$ | $\phi^*_t(p) - \phi^*_{t-1}(p)$ | rising or falling, this epoch | [[cyb]] "trending", [[foculus]], rewards |
| **attention shift** $\sigma_t$ (scalar) | $1 - \cos(\phi^*_t,\ \phi^*_{t-1})$ | did the collective's focus just switch? | network health (sits beside `syntropy J`); regime-switch alarm |
| **deviation** $\delta_t(p)$ | $\phi^*_t(p) - \bar\phi_t(p)$ | is $p$ above/below its own baseline (up/down state) | anomaly surfacing; input to Part B |
| **dwell** $w_t(p)$ | consecutive epochs $p$ has stayed in top-$k$ focus | waxing vs waning; maturity | ranking, foresight, decay policy |

**minimal first step (ship this alone):** persist $\phi^*_{t-1}$ and emit $\dot\phi^*$ and $\sigma_t$. that is "keep last epoch's focus and subtract" — a few lines in the focusing pass — and it already turns focus from a photograph into a velocity field. everything else in this doc builds on that one vector.

### A.4 Why it matters immediately

- **ranking becomes temporal.** [[cyb]] can show *momentum* (what is gaining focus), not just the frozen top-$k$. a new, corroborated concept climbing fast is more interesting than a stale concept sitting high.
- **regime-switch is now observable.** $\sigma_t$ spiking = the network changed its mind. that is a first-class health/consensus signal (a large $\sigma_t$ is exactly a metastable transition), currently invisible.
- **rewards can pay for direction, not just position.** the [[impulse]] reward is Shapley-of-$\Delta\phi^*$ within an epoch; velocity lets a neuron stake on the *direction* $\phi^*$ will move (foresight) — the Arrival framing from [[motif-awareness]] §8.4, now with a measured target to score against.

---

## Part B — homeostatic adaptation ("tire-out")

### B.1 The gap

[[focusing]] has plasticity — karma, [[ICBS|price]], conviction reshape `A_eff` — but no **homeostasis**: nothing reduces a particle's pull *because* it is already dominant. a well-connected, well-staked concept can hold the top of $\phi^*$ indefinitely (rich-get-richer). the picture's "adaptation" is specifically the *activity-dependent excitability that stabilizes dynamics* — the tired-neuron reflex. cyber lacks it, and its absence is both a missing dynamical ingredient and a centralization pressure.

### B.2 The mechanism

give each particle a slow **excitability gain** $g_t(p) \in [g_{\min}, 1]$ that adapts toward a homeostatic set-point $\phi_\star$ using the trailing focus $\bar\phi$ from Part A:

$$g_{t+1}(p) = \mathrm{clip}\Big(g_t(p) + \eta\big(\phi_\star - \bar\phi_t(p)\big),\ g_{\min},\ 1\Big)$$

a particle whose trailing focus sits **above** target loses gain (it *tires*); **below** target, gain recovers. $\eta$ is the adaptation rate; $g_{\min}>0$ guarantees no particle is ever fully silenced (preserves ergodicity). the gain enters the effective adjacency as a per-particle multiplier — one factor added to the existing product:

$$A_{\text{eff}}(p,q) = g_t(q)\cdot\!\!\sum_{\ell:\,p\to q}\!\! \text{stake}(\ell)\times \text{karma}(\nu(\ell))\times f(\text{price}(\ell))$$

### B.3 Why this is consensus-safe

$g_t$ is computed from epoch $t{-}1$'s already-final $\bar\phi$ and carried in [[bbg]] like karma. so at epoch $t$ it is a **constant** input to the tri-kernel, exactly as karma and price already are. the iteration $R = \lambda_d D + \lambda_s S + \lambda_h H_\tau$ is unchanged; $A_{\text{eff}}$ stays non-negative and (with teleport $\alpha$) ergodic, so the §fixed-point contraction $\kappa<1$ holds verbatim and $\phi^*_t$ is still the unique, byte-identical fixed point. the adaptation lives purely epoch-to-epoch. the $g$-update itself is a field-arithmetic EMA+clip on a proven vector — a small [[nox]] program, provable and byte-identical, maintained by [[plumb]] alongside karma.

### B.4 What it buys

- **waxing and waning, for free.** a concept rises, tires, recedes; freed focus flows to the next structure; that may in turn tire — the network's attention *cycles* through its repertoire instead of freezing. this is the picture's "emergent repertoire," at epoch scale, fully deterministic.
- **anti-monopoly is now structural, not bolted on.** tire-out is a negative feedback on dominance — the first-principles form of the anti-compounding intent (`v=0` affects rank not reward) and a decentralization force that needs no special-case rule.
- **a criticality knob.** $\phi_\star$ and $\eta$ set how "hot" the network runs. weak/absent → ossified single-winner (dead). strong → fast churn. tuned near the edge → maximally expressive. `syntropy J` and $\sigma_t$ are the dials that tell you where you sit.

### B.5 Degeneracy / backward compatibility

with $\eta = 0$ (or $g \equiv 1$) every $g_t(p)=1$ and $A_{\text{eff}}$, $\phi^*$, and all outputs are **byte-identical to today's [[focusing]]**. homeostasis is additive and off by default, same discipline as [[motif-awareness]] §6.

---

## Scope and touched specs

- `tru/specs/focusing.md` — Part A outputs table (add $\dot\phi^*$, $\sigma_t$, $\delta_t$, $w_t$ and the persisted $\phi^*_{t-1}$, $\bar\phi$); Part B `A_eff` gains the $g_t(q)$ factor and a homeostasis subsection.
- [[bbg]] — two new $|P|$ vectors ($\phi^*_{t-1}$, $\bar\phi$) plus $g_t$; carried like karma.
- [[plumb]] — writes $\bar\phi$ and $g_t$ each epoch (it already writes karma); tru only reads them.
- `tru/specs/rewards.md` / [[impulse]] — optional: velocity/foresight as a stake-able, scorable target.
- [[cyb]] / [[foculus]] — surface momentum and attention-shift, not just static rank.

## Open questions

Part A is settled (pure read-out). Part B has real design work:

1. **the set-point $\phi_\star$** — uniform $1/|P|$ (pure equalization, likely too flat), a soft cap (tire only above a threshold), or stake-/karma-relative (tire relative to earned standing). the soft cap is the conservative default.
2. **stability of the slow loop.** tire-out is negative feedback *with delay* (via the $\bar\phi$ window). that is precisely what produces waxing/waning — but a delayed negative feedback with too-high gain $\eta$ can grow into an unbounded epoch-scale oscillation. needs a control-theoretic bound on $\eta$ vs. ($\beta$, loop gain) that keeps the *inter-epoch* dynamics bounded while still expressive. this is the one genuinely open constant, analogous to [[motif-awareness]]'s $C_\partial$.
3. **karma vs. homeostasis interaction.** both multiply into `A_eff`. confirm they compose rather than fight (karma rewards honest track record; homeostasis damps dominance — a high-karma neuron's *concept* can still tire without penalizing the neuron). may want $g$ to act on particle excitability but leave the karma factor untouched, as written.
4. **per-particle vs. per-neuron.** B.2 tires *particles* (concepts). a per-neuron variant (a neuron's whole output tires) is a different, stronger lever — probably wrong (penalizes productivity), but worth stating the choice.

see [[focusing]] for the focus computation this extends, [[tri-kernel]] for the contraction it preserves, and [[motif-awareness]] for the companion additive-extension proposal.
