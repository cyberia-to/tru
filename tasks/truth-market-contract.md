---
tags: tru, task, rewards, truth, pussy
alias: truth market contract, T1 truth market
kind: task
status: open
owner: claude
serves: pussy · the truth market at genesis
lane: wide
date: 2026-11-21
---

# the truth market — one contract

T1 of [the truth market at genesis](../projects/truth-market-at-genesis.md).

## the owner's word (2026-11-21, translated)

"The losing side must always pay; the winning side must have additional benefits." "Those must be defined before launch — so something is literally wrong in our plans. Check and structure it using tasks and projects."

## the claim

One page of `specs/` states the truth market as the rule set a node computes at genesis: what a neuron commits in one act, what the chain records, how the link's market moves, how the serum scores an epoch, and how the settlement moves value — the losing side to the winning side, the premium from the focus pool, the annuity, karma — in the chain's integer arithmetic, with no part left to a later rule set. Every clause elsewhere that defers or contradicts it is listed old → new.

## outputs

1. the contract in `specs/`, as a completion of [rewards](../specs/rewards.md) §5, §9, §11, §12 for the root chain, with a trace of every rule to its source: an earlier clause, the owner's sentence, or a receipt;
2. the nine lines of the project page settled in the text, each with its reason; where the evidence does not choose, a prepared decision for the owner (apparatus `combine/decision.md`), one question each;
3. the amendment list, old text verbatim at origin heads: soft3 `specs/chain-economy.md` §3.7, §4.2, §5.4, §5.5, §7.4, §9, §12; rewards §16 (the rows of karma, price and will, surprise, the annuity); cyber `launch.md` (the cores' sentence, rows 24, 25, 28, 29); cyberia `projects/pussy.md` (the target table, the critical path); the explainers and term pages of `docs/` that read the valence two ways;
4. the implementation deltas for T3 and T4, per function and per record.

## method

Read whole, at origin heads, before a line is written: `specs/rewards.md`, `specs/truth-scoring.md`, `specs/strong-truthfulness.md`, `specs/epoch.md`, `specs/arithmetic.md`, `docs/terms/market.md`, `docs/explanation/` (honesty, valence, markets, epistemic-markets, incentives), `audit/coordinated-consensus-scoring.md`, `audit/manufactured-surprise-hardening.md`, `audit/annuity-accrual.md`, `audit/allocation-curve.md`; `rs/truth_scoring.rs`, `rs/icbs.rs`, `rs/annuity.rs`, `rs/tests/honesty_loop.rs`; soft3 `specs/chain-economy.md` and `proposals/rewards-at-genesis.md`; cyber `launch.md` (the cores, rows 24–29, the decision log); the closed plumb#15 and mudra#5 for what they found.

The standard the owner set on 2026-11-20 for a rule of the protocol holds here: one formula at every size and load, no threshold that switches a rule, hardness and safety by construction and never by a measurement.

## checks

- no deferral words in the contract, and no constant standing in for a mechanism: a value of 1 for karma, price or surprise is a deferral;
- conservation stated and checkable: the redistribution sums to zero; the premium and the annuity leave the focus pool and nothing else; supply stays a function of height;
- every row of the explainer's table of strategies follows from a clause of the contract;
- two review rounds by another body at most, findings with ids, each disposed; the owner approves.

## state

Open. Nothing is written. The project page holds the inventory this task starts from.
