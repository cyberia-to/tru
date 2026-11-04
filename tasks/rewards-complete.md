---
tags: tru, task, rewards, specification, pussy
kind: task
status: doing
owner: claude
serves: pussy
lane: wide
date: 2026-11-04
---

# rewards — the specification complete, the proof of work samples the rewards

## the owner's word (2026-11-04, translated)

"for genesis tomorrow we are not ready. my decision does not change: the full implementation of the rewards specification is necessary — I do not understand what the limitation is. change every document that contradicts it. without it I will not launch the chain; this is the most important thing in the project: the proof of work must really sample values by the rewards distribution. launch agents on the servers to finish, test and carry it. a completely finished rewards specification, without notes about what is undone." Earlier, 2026-11-02, on chain-economy §7.1: "I would never agree — we have to supply complete mining rewards at genesis."

## the claim

`specs/rewards.md` is one consistent, complete contract: every hash attempt of the chain's proof of work is a Shapley sample of a link's marginal Δφ⁺ (§4, §7); the winning ticket is the block's work and the subsidy (§7, §8: "no separate proof-hash puzzle"); tickets fold per cluster; difficulty, deadline, k_min, withholding, the split (§10) and the mint are specified with no deferral, no "unspecified", no "not at genesis"; and every clause elsewhere that defers, replaces or contradicts it is amended with old → new text.

## outputs

1. `specs/rewards.md` complete (this branch): §7 finished (the region, the lottery, aggregation, deadline, k_min, difficulty, withholding), §8 and §10 consistent with it, §16 rewritten from "recompute-only" to the lottery as the rule of the root chain; a trace table of every rule to its source (earlier clause, the owner's sentence, a measured receipt).
2. the amendment list for the other documents, old → new verbatim: cybernode `specs/chain-economy.md` §7.1, §7.3, §6; `specs/chain.md` §1 (PoW rule, header), §2.2 (coinbase), the "bootstrap puzzle of rewards §8" labels; soft3 `proposals/steps-blocks-epochs.md` §5 (the ordering hash); `cyber/launch.md` decisions 2026-10-15/17 wording and registry rows 3, 4, 7, 35, 36; and any other page found by search.
3. the open design choices that the specification itself must settle (not defer): whether a separate ordering hash survives (rewards.md:175 says no; steps-blocks-epochs §5 says yes — settle by rewards.md unless a measured reason forbids), how the beacon is drawn, the ticket's verification at pussy's scale (recomputation; the proof later), the cap per epoch in place of the O(1) fold until the accumulator is sound (named in the spec as the rule, not as a gap).

## checks

- no deferral words remain in rewards.md: "unspecified", "not at genesis", "later rule set", "deferred", "open frontier" in the normative sections (§15's frontier list may stay as research, not as holes in the contract);
- every rule carries a clause, the owner's sentence or a receipt; a reviewer of another body re-opens each;
- the three design choices are settled in the text, with the reason;
- the amendments quote the current text verbatim at origin heads;
- two review rounds at most (deliver.md), findings with ids, each disposed; the owner approves (wide, a specification of the kernel).

## log

- 2026-11-04 · opened on D20 (soft3#680).
