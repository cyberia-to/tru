---
tags: cyber, tru, project, pussy, launch
alias: truth market at genesis, the truth market at genesis, serum and market at genesis
kind: project
status: proposed
owner: master
serves: launch spacepussy ($PUSSY)
part of: launch spacepussy
lane: wide
date: 2026-11-21
---

# the truth market at genesis

On pussy's chain at genesis every staked cyberlink carries the whole truth mechanism of [rewards](../specs/rewards.md) §5: a side, a stake and a prediction of the crowd; a market on the link; the serum's score every epoch; the losing side paying the winning side; the mint as the winners' premium; karma.

What a reader is told this means is [truth, from claim to payout](https://github.com/cyberia-to/tru/blob/docs/truth-explainer/docs/explanation/truth.md), in review as [tru#47](https://github.com/cyberia-to/tru/pull/47). This page is the distance between that page and the chain.

## the owner's word · 2026-11-21

Translated from the owner's messages (no link exists). On the explainer's line "at launch the network's emission; later the losing side as well": "I do not think it is the correct framing — the losing side must always pay; the winning side must have additional benefits." On the serum and the market marked as a later rule set: "I can say for sure that those must be defined before launch — so something is literally wrong in our plans. Check and structure it using tasks and projects. I think it is a project for launching spacepussy; it cannot be done by a simple order."

Earlier words this one stands on: 2026-09-18, the decision log of `cyber/launch.md`: "hybrid economics is a core: truth markets (ICBS + serum + valence) and staking on two axes ship in phase 1"; 2026-11-01: "tru and foculus must be full"; 2026-11-04: "the full implementation of the rewards specification is necessary"; 2026-11-16: "either the mechanism exists or it does not; the key mechanism must be complete at once — rewards are the key mechanism".

## what is wrong in the plans

The decision of 2026-09-18 puts the truth market in phase 1. On 2026-10-17 the chain's economy went into consensus with the serum, the market, karma and the annuity set to constants, and every plan since has carried that as settled. Read on 2026-11-21:

| where | what it says | effect |
|---|---|---|
| soft3 `specs/chain-economy.md` §3.7 | karma = 1, f(price) = 1, will = 0 at genesis | no reputation, no market |
| the same, §5.4 | surprise ρ = 1: "there are no reports on chain at genesis" | no serum; a copy is paid like a discovery within an epoch |
| the same, §4.2 | valence is the direction of a position's push on the axon | the law reads valence as the prediction of the crowd ([rewards](../specs/rewards.md) §5); the chain has no prediction at all |
| the same, §5.5, §7.4 | payouts come from the focus pool alone; the redistribution "that is the slashing of rewards.md §5" belongs to the reports rule set | a stake on the losing side loses nothing |
| cyber `launch.md`, the cores | "the ticket lottery, fold, the serum and market, and privacy follow by scheduled fork" | the core the page itself calls the game changer is after genesis |
| the same, rows 24, 25, 28 | "deferred: reports rule set" | the serum, the market and the annuity have no date |
| cyberia `projects/pussy.md` | "serum, ICBS, annuity, lottery exist as unwired building blocks"; the reports rule set among "the proof layers as rule sets", open | the launch project does not carry them on its critical path |
| soft3 `release/director/plan.md` D20, P14, P15 | the gate of genesis is rewards §7–§8, the work | "the full rewards specification" was narrowed to the lottery |
| [tru#45](https://github.com/cyberia-to/tru/pull/45), rewards §16 at `028e5e5` | "karma κ, price f, will: 1, 1, 0 — the chain carries no reports, no market and no will"; "surprise ρ: 1"; "annuity: the pulse only" | the page passes its own search for deferral words because the deferral is written as a parameter |

One sibling already corrects a part: soft3 `proposals/rewards-at-genesis.md` (2026-11-16, proposed, on the branch `proposal/rewards-at-genesis`) brings the annuity and the whole allocation curve to genesis.

## what exists

Verified on the default branches on 2026-11-21.

| piece | where | state |
|---|---|---|
| the serum's scorer: `bts_scores`, `bts_scores_hardened`, `surprise`, `karma_step`, `accumulate` | tru `rs/truth_scoring.rs` | merged, 9 tests; a report is a belief and a prediction, both fractions; the epoch engine does not call it |
| an informed minority against a coordinated majority, to 200 agents | tru `audit/coordinated-consensus-scoring.md`, `audit/manufactured-surprise-hardening.md` | receipts |
| karma into the effective graph and so into focus | tru `rs/tests/honesty_loop.rs` | one test, on the general engine, outside consensus |
| the market's cost function and prices | tru `rs/icbs.rs` | merged, 6 tests; no reserves, no positions |
| the market's reserves and positions under conservation | plumb#15 | closed without merging |
| the annuity's per-epoch accrual | tru `rs/annuity.rs` | merged, 6 tests; not joined to price or focus |
| the allocation curve | tru `rs/allocation.rs`; soft3 chain-economy §6.1 | in consensus, clamped at a half |
| positions, sides, the pulse, the focus pool | soft3 and cybernode, the alpha | in consensus on the rehearsal network |
| the contracts | tru `specs/truth-scoring.md`, `specs/strong-truthfulness.md`; the protocol of one link, `docs/terms/market.md`, a draft | the law is written; no page states it as integers a node computes |
| privacy of positions and reports | mudra#5 closed (the audit: valence leaks on the wire); mudra#22 open (private authorship) | no relation for valence |
| staking from cyb | stopped; the chain's CLI only | — |

## what the contract has to settle

The pieces disagree with one another, so the first task is one contract. Each line becomes a prepared decision for the owner when its evidence is in; none is asked here.

1. The prediction. The law reads the valence as the prediction of the crowd (rewards §5); the scorer takes a fraction; the chain's draft adds a kind of its own with basis points (chain-economy §7.4). A three-valued prediction cannot say "a third of the crowd", which the count needs after the first epoch.
2. The side. The chain reads the valence as the side of the stake. In the law the side is the position in the link's market and the valence is the prediction.
3. What the losing side pays: the base and the rate of the redistribution (rewards §5 gives the proportion and no rate), and how it stands beside the market's own profit and loss.
4. What focus reads. Stake times price, or the surprisingly popular estimate that strong-truthfulness makes the design rule, and either one in integers.
5. Where karma attaches when the author of a link is private: chain-economy §3.7 names positions or payout keys.
6. The market's parameters: the depth λ, the damper δ, and whose money seeds a new link's market.
7. The annuity's weight ω(t) = price × karma, with the sibling proposal.
8. Whether positions and predictions are public at genesis or veiled (launch row 29).
9. Will: in the launch rule set or out of the law.

## the tasks

| id | task | repository | gate | waits for |
|---|---|---|---|---|
| T1 | the contract: the truth market as one complete rule set — the act, the records, the market, the score, the settlement, in integers; the nine lines above settled; every clause that contradicts it listed old → new | tru `specs/`, with the amendment list for soft3, cyber, cyberia | no deferral left in the page; two review rounds by another body; the owner approves — [task page](../tasks/truth-market-contract.md) | — |
| T2 | the mechanism under adversaries before it is consensus: the honest crowd, the copier, the hedger, the ring, the coordinated majority, the mistaken minority; the redistribution conserves | simulation, tru | a receipt in which every row of the explainer's table of strategies is reproduced with numbers | the draft of T1 |
| T3 | the engine: score, surprise, karma, redistribution, the market's books and the annuity on the consensus epoch engine, integers only | tru | tests, digests equal on two machines, the cost of an epoch at pussy's size | T1 |
| T4 | the chain: the kinds and records of the act, the order of settlement, conservation, chain-economy amended, conformance vectors | cybernode, soft3 | vectors blessed; adversary tests green | T1, T3 |
| T5 | cyb: the stake screen shows the side, the stake, the prediction and what is at risk; it replays after a revert | cyb | a neuron stakes, loses and wins from cyb on the alpha | T4 |
| T6 | the rehearsal: 24 hours on two machines with stake on both sides of a link; a losing side pays a winning side; the explainer's epoch reproduced on chain | cybernode | a receipt in `audit/chain/` | T4 |
| T7 | the registry and the pages: launch rows 24, 25, 28, 29 carry this project; pussy's target table and critical path; the older explainers follow the contract; spacepussy.ai says how truth works | cyber, cyberia, tru | no page calls a part of this project a later rule set | T1 |

T1 starts now and is the only task that can. T2 runs beside it. The size of T3–T6 is unknown until T1 settles the nine lines: the building blocks exist, the joins between them do not.

## beside tru#45

[tru#45](https://github.com/cyberia-to/tru/pull/45) completes the work of §7: what a miner computes and how it is paid. Its §16 still fixes karma, price, surprise and the annuity at the constants above. T1 replaces those rows. Until then tru#45 is complete for the work and silent on the honesty of §5.

## the measure

A neuron on the public pussy chain stakes on the losing side of a link and sees its stake pay the winning side at the next settlement; the winner sees the same amount and a premium on top; both see their karma move; every node agrees on each number.
