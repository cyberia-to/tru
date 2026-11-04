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

## design (node 2 of deliver.md)

Read in full at origin heads on 2026-11-04 (method, non-negotiable): tru `specs/rewards.md` (409 lines), `epoch.md`, `arithmetic.md`, `locus.md`, `README.md`, `focusing.md`, `impulse.md`, `truth-scoring.md`, `superadditivity.md`, `strong-truthfulness.md`, `vocab.md`, `attention.md` (`tri-kernel.md`, `ct0.md`, `model.md` grepped for reward clauses: none), `audit/epoch-settlement-2026-10.md`, `audit/phi-relation-2026-11.md` (release/phase1), `audit/allocation-curve.md`, `audit/annuity-accrual.md`, `audit/coordinated-consensus-scoring.md`; foculus `specs/fold-mining.md`, `specs/protocol.md`, `specs/beacon.md`, `audit/sound-settlement-proofs-2026-10.md` and `audit/accumulated-settlement-2026-10.md` (on `feat/accumulated-settlement` and `release/phase1`; absent from master), `audit/withholding-bias.md`, `audit/progress-freedom-banding.md`, `audit/hardware-profile.md`, `audit/ticket-proof-cost.md`; cybernode `specs/chain-economy.md` (971 lines), `specs/chain.md` §0–§3, §5, §7, §10, `audit/chain/economy-cost-2026-10-17.md`, `audit/chain/economy-2026-10-17.md`, `audit/chain/settlement-mining-state-2026-11-02.md` (branch `audit/settlement-mining-state`); soft3 `proposals/steps-blocks-epochs.md`, `goldilocks-parano1d.md`, `proven-present.md`; cyber `launch.md` (the decisions of 10-15, 10-16, 10-17, 10-30, the four cores, rows 1–45, the decisions log — no line of 11-02 or 11-04 exists there), `whitepaper.md` §14; uhash `docs/viewing-economy.md`; warriors `mining.md`; hemera `specs/encoding.md` (the padding rule). The concept → clause trace is the "Trace" section of `specs/rewards.md`.

What §7 now fixes, and from what:

- the region: on the consensus engine the ε-support of every change is the whole staked graph (stake prior, normalization, the global gradient term of epoch.md §4) → one cluster per epoch, `cluster(E)` = the hash of the players;
- the game: players = (card, side, Δs) between two boundaries; `v(S) = J_U(φ*(A^S)) − J_U(φ*(A^∅))` on the epoch engine — the game the exact-Shapley measurement of epoch-settlement §3 already used;
- the sample: `seed = hemera(D_ORDER ‖ template[0..168])`; the drawn player, the inclusion threshold and the coalition from the seed (Owen's multilinear extension: the expectation is exact Shapley); `m = v(P ∪ {c}) − v(P)`, two solves; the empty epoch samples the gradient at a drawn axon, one solve; the whole template seeds the draw, so no header field is free to grind;
- the ticket: `hemera(template ‖ m: i128) < target` — the header gains `sample: i128` at offset 168 (184 bytes, four permutations); the block id is the commitment, the work, the ordering weight and the subsidy; no second hash, no kinds 8/9;
- difficulty: ASERT on the interval over tickets; `GENESIS_BITS = 0x1f100000` (2^244, 2^12 attempts ≈ 25 s for a twelve-thread machine at 0.036 s per pussy solve), `POW_LIMIT = 2^252` (16 attempts);
- verification: recomputation, two solves per block (0.072 s at pussy, ≈ 2 s on twelve threads at bostrom); the cap is one ticket per block, 240 per epoch; the accumulator replaces it under (a) a proof relation for the epoch engine's step and (b) a constant-size fold with the verifier inside the step, `K_BLOCK = 0` while false;
- deadline and k_min: the window of epoch E's game is epoch E + 1; the pulse at `settle(E) = boundary(E) + 120`, final; the payout is the exact Shapley value (the four-node path gradient) while it fits `SETTLE_BUDGET` (rule 1, pussy and bostrom), the same over a longer epoch (rule 2), the swarm mean with `k_min = 4,096` per player and a carried claim under it (rule 3);
- withholding: a withheld ticket is a withheld block and forfeits `SUBSIDY − cut + fees`; the gain is zero under the exact payout and bounded, published and priced by the miner floor under the sampling payout; role separation is not a rule.

The three design choices, in `specs/rewards.md` §16 with their reasons: (1) no separate ordering hash — the ticket is the block id; (2) the beacon is the boundary block id (chain-economy §2.3); (3) verification is recomputation, the accumulator by condition.

Amendments to the other documents: `audit/rewards-amendments-2026-11-04.md` — 58 entries (cybernode chain-economy 14, chain.md 13; soft3 7; cyber 13; foculus 7; uhash 2; tru 2, applied here: `specs/README.md`, `specs/epoch.md`).

## self-check (three passes, 2026-11-04)

1. clause fidelity — every rule of §7 and §16 traces to an earlier clause, the owner's sentence or a receipt: the Trace table of `specs/rewards.md` (23 rows), re-read against the sources above; the owner's prose of §1–§3, §9, §13 is byte-identical to origin/master, §4–§6, §8, §10–§12, §14–§15 edited only where a clause contradicted §7.
2. no deferral words — `grep -nE 'unspecified|not at genesis|later rule set|deferred|replaced by|until the graph outgrows|open frontier' specs/rewards.md` → (empty, exit 1); the wider sweep `grep -nE -i 'unbuilt|not yet proven|unmodeled|undone|shadow|deferr|later slice|next slice|not at launch|when it lands|TODO' specs/rewards.md` → (empty, exit 1).
3. nothing superfluous — no new section beyond the Trace; no new parameter without a derivation (`GENESIS_BITS`, `POW_LIMIT`, `k_min`, `K_BLOCK`, `D_ORDER`, `D_CLUSTER` each carry theirs); the fold's mechanics stay in foculus; the ε-support partition is stated once as the general law's object and resolved on the consensus engine, not duplicated.

## log

- 2026-11-04 · opened on D20 (soft3#680).
- 2026-11-04 · design and the completed specification on this branch; amendments listed; self-check passes 1–3 logged above; pull request to tru master opened (link in the first comment of the PR and below).
