---
tags: cyber, tru, roadmap, rewards, economics
crystal-type: plan
crystal-domain: cyber
status: open
alias: GFP funding, hardware funding question, bootstrap funding
---
# GFP funding — can the mint and the floor fund the hardware before fees arrive?

status: open (the registry's words: proven · simulated · measured · open). Moved out of [[rewards]] §15 on 2026-11-12 under `~/cyber/AGENTS.md` § moving a page: a projection with a named assumption and an open question leaves the specification; the contract keeps the rule.

## the rule that stands (rewards §10, §16)

Issuance is one number. On the root chain the mint and the security budget are one subsidy — `SUBSIDY = ⌊S0 / 105,120,000⌋` per block, 1 % of genesis supply a year, split per epoch between the ticket and the focus pool by `min(½, θ^½)` — so the emission per epoch is `L · SUBSIDY` and supply is `S0 + h · SUBSIDY`, a function of height alone (cybernode `specs/chain-economy.md` §6; chain.md §3.5). In the general law the mint runs on its own budget beside the security budget `B` and their sum is capped by the §10 allocation; nothing in either leaves issuance open.

## the projection

The question: whether the early discovery-premium mint and the security floor, paid in the chain's own coin, fund the [[Goldilocks field processor|GFP]] hardware the work of [[rewards]] §7 wants — sparse matrix–vector passes over the Goldilocks field, `PHI_STEPS` dependent solves — before fees (rewards §8, cut from phase 1) arrive.

Named assumption: that the coin's price against the hardware's cost is such that the ticket's subsidy per block, `SUBSIDY − cut + fees` (chain.md §2.2), pays for the machine that mines it within the machine's life. The subsidy is fixed in coin; the hardware's price is in money; the ratio is the market's, not the protocol's.

What is measured: the ticket's cost on the hardware that exists — 35 ms per marginal on an M4 Max core, 127 ms on a Xeon core, 5.06 and 1.59 samples a second per machine (tru#46, `audit/coalition-marginal-2026-11-04.md` §3) — and the subsidy per block on pussy, 10,458,426,657 PUSSY, of which the ticket keeps at least half (chain-economy §6.1). What is not: the coin's price, the hardware's cost, the fee flow.

## what closes it

The canary record: the ticket's subsidy against the measured cost of the machines that mine pussy over phase 2 (cyber `launch.md`: the canary under real use; no date on the public pages). The question moves from `open` to `measured` when a receipt in `audit/` states the subsidy per machine-day in coin and the machine's cost in the same unit; what the owner does with a measured answer that says the floor does not pay for the hardware is his decision, not this page's.

See [[rewards]] §8 (the three roles, the GFP flywheel), §10 (the split and the floor) and §16 (the root chain's values).
