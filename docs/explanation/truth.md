---
tags: cyber, tru, docs
alias: truth explained, staking on truth, a stake on truth, how truth works, truth from claim to payout
crystal-type: article
crystal-domain: cyber
---
# truth, from claim to payout

What a stake on a [[cyberlink]] claims, how the network settles it, where the reward comes from, and how far the result can be trusted. Written for a reader who knows how a prediction market works and asks the four plain questions: what is truth here, who asks and who answers, why would anyone stake on it, and who checks.

The short answer: a prediction market settles on an event outside itself. [[cyber]] has no outside. It settles every epoch on its own state, and a link is true for as long as stake keeps holding it against anyone who could profit by pushing it down.

---

## the machine you already know

A prediction market has four parts: an organizer writes a question, players stake on an outcome, the event happens in the world, and the losers pay the winners. Truth arrives from outside, once, and the market closes.

Most knowledge never gets such a day. "This paper answers that question", "this file is the source of that claim", "this model is better at that task" — nothing happens in the world that closes them. They stay open, and people keep relying on them. That is the knowledge cyber is built for, so each of the four parts changes:

| | a prediction market | cyber |
|---|---|---|
| the question | written by an organizer | the link itself, made by any [[neuron]] |
| the stake | on an outcome of an event | on the link: for it, against it, or passive |
| the settlement | once, when the event happens outside | every epoch, on the state of the graph; it never closes |
| who pays | the losing side | at launch the network's emission; later the losing side as well |
| being right means | matching the event | holding what the network keeps holding |

---

## the claim: who asks and who answers

A cyberlink `p → q` joins two [[particles]], the identities of two [[files]]. It says one thing: from `p`, the file `q` deserves attention. If `p` is a question and `q` a text, the link says "this answers it". If `p` is a claim and `q` a dataset, it says "this supports it".

Nobody is appointed to ask or to answer. The question is a file, the answer is a file, and the link between them is the answer given. Whoever reads `p` later sees every link that leaves it, ordered by how much [[focus]] the network has placed on each.

A link with no stake is free speech: it exists, carries no weight and earns nothing.

---

## the stake: three sides

A neuron locks tokens on a link and chooses a side, the [[valence]] of the position.

| side | what the stake does to the link | what it can earn |
|---|---|---|
| `+1`, for | adds its weight | a reward, when adding it made the shared picture sharper |
| `−1`, against | removes weight, down to zero | a reward, when removing it made the shared picture sharper |
| `0`, passive | adds its weight | nothing: it moves the ranking and takes no part in the reward |

The stake stays locked for seven days. That lock is the whole risk at launch.

---

## the process

```text
   the claim           the premise          truth emerges         validation           reward
 ┌─────────────┐    ┌──────────────┐    ┌───────────────┐    ┌───────────────┐    ┌─────────────┐
 │ link p → q  │    │ every stake  │    │ focus is      │    │ whose added   │    │ a share of  │
 │ stake, side │ →  │ on every link│ →  │ recomputed    │ →  │ stake made    │ →  │ the focus   │
 │ +1 / 0 / −1 │    │ becomes its  │    │ over the      │    │ focus sharper,│    │ pool, by    │
 │             │    │ weight       │    │ whole graph   │    │ and by how    │    │ that measure│
 └─────────────┘    └──────────────┘    └───────────────┘    │ much?         │    └─────────────┘
   one neuron         all neurons          the chain         └───────────────┘      the neuron
                                         once an epoch          the chain
```

1. The claim. A neuron makes a link and locks stake on a side.
2. The premise. At the end of an epoch (240 blocks, about two hours) the chain takes every link with the net stake on it: stake for, minus stake against, never below zero. This weighted graph is everything the network currently asserts.
3. Truth emerges. Over that graph the chain computes [[focus]]: where attention settles when it flows along the links by their weights. The computation has one answer, every node gets the same one, and no neuron chooses it ([[collective focus theorem]]). The links that carry focus are what the network holds true this epoch.
4. Validation. The chain measures [[syntropy]], how sharply focus is concentrated, before and after the epoch, and attributes the change to the stake each position added. Stake that sharpened the picture has a positive share. Stake that blurred it has none.
5. Reward. The epoch's budget is divided by those shares and paid to the positions.

Then the next epoch begins on the new graph. A link is never settled for good: new stake for it or against it moves it again.

---

## what "true" means, in three layers

Each layer asks a different question about the same stake. The first is live at launch. The other two are rule sets that follow it, and the pages that describe them describe the general law.

| layer | the question it asks | what it catches | state |
|---|---|---|---|
| focus | did this stake make the shared picture sharper? | noise: stake that fits nothing | live at launch |
| the serum | did this neuron hold the view before the crowd came to it? | copying and herding | a later rule set |
| the market | can anyone profit by staking against this link? | errors that outlive their epoch | a later rule set |

Focus alone pays for coherence, and coherence is weaker than truth: a crowd can agree on a mistake, and stake that joins it sharpens the picture and is paid.

The serum is the answer to that. Each staker also reports what share of the stake it expects on its own side. An answer that turns out more common than the crowd predicted is the one held by those who know something the rest did not expect ([[Bayesian Truth Serum]], Prelec 2004). The textbook case: asked whether Philadelphia is the capital of Pennsylvania, most people say yes. Those who say no also predict that most will say yes. "No" comes out more common than predicted, and "no" is correct. The score that pays this accumulates into [[karma]], and it is taken from those who added noise.

The market is the answer to time. Every link carries a price between false and true that anyone can push by staking, and it stays open. When a link once believed turns out wrong, the neuron who sees it first stakes against it and earns as the rest follow. See [epistemic markets](epistemic-markets.md).

---

## why anyone stakes

The reward is the only source of staking income in the network. A share of the block subsidy flows into the focus pool; the share grows with the fraction of supply that is actively staked. Each epoch the pool releases a quarter of its balance to the positions that earned, in proportion to what each contributed.

What a position can earn and lose at launch:

| | |
|---|---|
| it earns | when it is active (`+1` or `−1`), it added stake this epoch, and that stake has a positive share of the epoch's gain |
| once | only stake that lifts a link's side above its previous high is paid; unlocking and locking again to the same height earns nothing |
| at most | 1 % of the stake added, per epoch |
| it risks | the fee of the link, seven days of lock, and earning nothing |
| it loses | nothing else: a stake on the wrong side comes back whole. Losses for being wrong arrive with the serum |

Tokens enter the network through work and through this reward. A neuron that mined can stake, and mining is tuned for ordinary hardware ([rewards](../../specs/rewards.md) §8).

---

## one epoch, with numbers

The numbers are invented to show the shape; the rules are the chain's.

A question `p` has two links: to a good answer `q` and to a noise file `z`. In one epoch four neurons act, and the epoch's budget is 40 tokens.

| neuron | position | share of the epoch's gain | paid |
|---|---|---|---|
| A | 10,000 for `p → q` | 50 % | 20 |
| B | 5,000 for `p → q`, the same link, the same epoch | 25 % | 10 |
| C | 8,000 for `p → z` | none: focus got blurrier | 0 |
| D | 4,000 against `p → z` | 25 % | 10 |

A and B share one link's credit in proportion to what each added: copying A in the same epoch takes half as much with half the stake, and copying an epoch later meets a link that is already heavy and gains little. C is paid nothing and gets 8,000 back after the lock. D is paid for removing weight from the noise.

---

## where the outside world comes in

The chain reads no news. The world enters through neurons: someone who knows that a claim failed stakes against it, and is paid when that makes the picture sharper. So the network knows what its neurons know and are willing to stake on, and a fact nobody stakes on is a fact the network does not hold.

That is also the limit. A majority of stake that agrees on a falsehood holds it, the way a majority of work holds a chain. The design aims at making that expensive to keep up and profitable to break.

---

## how far it can be trusted

| claim | standing |
|---|---|
| focus has one value and every node computes the same | proven, and checked by every node each epoch ([[collective focus theorem]], [epoch](../../specs/epoch.md)) |
| splitting a stake across many keys earns the same | holds by the arithmetic: every rule is linear in stake |
| telling the truth is the best reply when others do | proven for the serum's report taken alone (Prelec); for the whole payoff together it is assumed ([rewards](../../specs/rewards.md) §15, [strong truthfulness](../../specs/strong-truthfulness.md)) |
| a ring of real stakers that agrees among itself gains little | bounded by capital: 1 % of the stake added, once per new high, seven days locked; it is bounded, and it is open |
| the network converges on what is the case | a hypothesis; the test is a public network under real stake |

The honest summary is the last row. The theorem says what honest players do; whether a live network of strategic ones behaves that way is what the launch measures.

---

## the four questions, answered

- What is truth? What the network keeps holding: the links that carry focus after every stake for and against them is counted, recomputed each epoch.
- Who asks and who answers? Any neuron. A link from one file to another is a question answered.
- Why stake on it? Active stake that sharpens the shared picture is the one thing the network pays for holding tokens.
- Who checks? Every node recomputes the same result each epoch, and every neuron may stake against any link at any time. There is no outside judge and no closing day.

---

## where the rules are

- the reward law: [rewards](../../specs/rewards.md) — §1 the principle, §5 honesty, §9 the two axes of stake, §15 security, §16 what the root chain fixes
- the epoch's computation: [epoch](../../specs/epoch.md)
- the serum's contract: [truth scoring](../../specs/truth-scoring.md), [strong truthfulness](../../specs/strong-truthfulness.md)
- positions, sides, the new-high rule, the cap and the budget on the root chain: chain-economy §3–§6 (cybernode `specs/chain-economy.md`)
- the general law in narrative: [honesty](honesty.md), [valence](valence.md), [epistemic markets](epistemic-markets.md), [incentives](incentives.md)
