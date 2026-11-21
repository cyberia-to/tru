---
tags: cyber, tru, docs
alias: truth explained, staking on truth, a stake on truth, how truth works, truth from claim to payout
crystal-type: article
crystal-domain: cyber
---
# truth, from claim to payout

What a stake on a [[cyberlink]] claims, how the network decides whether it holds, who pays whom, and how far the result can be trusted. Written for a reader who knows how a prediction market works and asks four plain questions: what is truth here, who asks and who answers, why would anyone stake on it, and who checks.

The short answer. A prediction market waits for the world to settle its question. Most knowledge has no such day, so [[cyber]] settles inside itself, every epoch: the side of a link that turns out more common than the crowd itself predicted wins, the losing side pays it, and the network adds a premium for the knowledge the winners created. The link stays open to a new stake forever.

---

## the machine you already know

A prediction market has four parts. An organizer writes a question. Players stake on an outcome. The event happens in the world. The losers pay the winners, and the market closes.

It works because truth arrives from outside. That is also its limit. "This paper answers that question", "this file is the source of that claim", "this model is better at that task" — nothing ever happens in the world that closes these, yet people rely on them every day. This is the knowledge cyber is built for, and each of the four parts changes to fit it.

| | a prediction market | cyber |
|---|---|---|
| the question | written by an organizer | the link itself, made by any [[neuron]] |
| the stake | on the outcome of an event | on the link: for it or against it |
| the judge | the world, once | the crowd's own surprise, every epoch |
| who pays | the losing side pays the winning side | the losing side pays the winning side, and the network adds a premium to the winners |
| the end | the market closes | the link stays open; a later stake can overturn it |

One thing stays exactly as it is in the market: nobody is paid for holding tokens, and nobody is right for free. A stake that takes a side is at risk.

---

## the claim: who asks and who answers

A cyberlink `p → q` joins two [[particles]], the identities of two [[files]]. It says one thing: from `p`, the file `q` deserves attention. If `p` is a question and `q` a text, the link says "this answers it". If `p` is a claim and `q` a dataset, it says "this supports it". If `p` is the name of an event and `q` a report of it, it says "this is what happened".

Nobody is appointed to ask or to answer. The question is a file, the answer is a file, and the link between them is the answer given. Whoever reads `p` later sees every link that leaves it, ordered by how much [[focus]] the network has placed on each.

A link with no stake is free speech: it exists, carries no weight and earns nothing.

---

## one act, three commitments

A neuron that wants its link to count commits three things at once.

| commitment | what it says | what it is for |
|---|---|---|
| a side | this link holds, or this link fails | the first-order belief: what the neuron itself thinks |
| a stake | this much of my money stands behind that side | the cost of being wrong, and the weight of the voice |
| a prediction | this is where I expect the crowd to stand | the second-order belief: the part that lets the network tell knowledge from agreement |

A neuron may also stake passively: its money gives the link weight in the ranking and takes no side. Passive stake risks nothing, predicts nothing and earns nothing. It buys attention and that is its whole return ([rewards](../../specs/rewards.md) §9).

The third commitment is the unusual one, and the rest of the page turns on it.

---

## the process

```text
        ┌───────────── the link stays open: anyone may stake on it again ─────────────┐
        ▼                                                                              │
 1 the claim       2 the market      3 focus           4 the serum       5 settlement  │
┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │
│ link p → q  │  │ stakes for  │  │ attention   │  │ which side   │  │ losers pay   │  │
│ a side      │→ │ and against │→ │ settles over│→ │ came out more│→ │ winners; the │──┘
│ a stake     │  │ set the     │  │ the weighted│  │ common than  │  │ network adds │
│ a prediction│  │ link's price│  │ graph       │  │ predicted?   │  │ a premium    │
└─────────────┘  └─────────────┘  └─────────────┘  └──────────────┘  └──────────────┘
  the premise                       truth emerges     validation        payout
  any neuron, any time              every node, once an epoch
```

### 1 · the claim

A neuron makes the link and commits a side, a stake and a prediction.

### 2 · the market

Every staked link carries its own small market with two sides, true and false ([[inversely coupled bonding surface|ICBS]]). Stake for the link raises its price; stake against it lowers it; the two sides are coupled, so raising one presses the other down. The protocol itself is the market maker, so there is always someone to stake against.

The market does three jobs. It makes a link cost money, so spam is expensive. It turns the stakes into one number, the price, the network's running estimate of the link. And it makes a false link expensive to defend: whoever props it up must keep buying the losing side from everyone who sees the mistake. It never closes.

### 3 · focus

Once an epoch every node takes the whole graph, each link weighed by the stake on it and by the standing the market and the serum give it, and computes where attention settles when it flows along the links. The computation has exactly one answer, every node gets the same one, and no neuron chooses it ([[collective focus theorem]]). The links that carry focus are what the network holds true this epoch. This is where truth emerges: as the resting state of everyone's stakes taken together.

### 4 · the serum

Focus alone would pay for agreement, and a crowd can agree on a mistake. The serum is what tells knowledge from agreement, using the third commitment.

The textbook case. Ask a crowd whether Philadelphia is the capital of Pennsylvania. Most say yes, and they are wrong. Now ask each person a second question: what share of the others will say yes? Those who answer yes expect nearly everyone to agree. Those who answer no know the real capital, and they also know the mistake is common, so they too predict that most will say yes. Count the answers: "no" comes out more common than the crowd predicted. The surprisingly popular answer is the correct one ([[Bayesian Truth Serum]], Prelec 2004; Prelec, Seung and McCoy 2017).

The reason it works is that knowledge is lopsided. Someone who knows the fact also knows the popular error; someone who holds the error has never heard of the fact. So the informed side is always underestimated by the crowd, and that gap is measurable without anyone knowing the answer in advance.

On a link it is the same count. Each staker has taken a side and predicted the crowd. The serum compares, per link, how the stake actually stands with how the stakers predicted it would stand. A side that is common and was expected to be common shows no gap: that is plain agreement, and it scores nothing. A side that is more common than predicted shows the gap: someone knew something. Each neuron's score has two parts, the surprise of its side and the accuracy of its prediction of the crowd ([truth scoring](../../specs/truth-scoring.md)).

This is the validation. It needs no outside judge: the question it asks is whether the people who stood behind the link knew more than the crowd expected them to.

### 5 · settlement

Three flows of value leave the epoch, and one record.

| flow | from | to | by what measure |
|---|---|---|---|
| the redistribution | stakes with a negative score | stakes with a positive score | the serum's score; it sums to zero, so the losing side always pays ([rewards](../../specs/rewards.md) §5) |
| the premium | the network's emission, through the focus pool | the winners | the gain in [[syntropy]] their stake caused, weighed by its surprise: new money is minted only where knowledge was created ([rewards](../../specs/rewards.md) §1–§4) |
| the annuity | the network's emission | links that keep carrying focus | the growth of focus on the link's target in each later epoch: foundational links earn as the graph builds on them ([rewards](../../specs/rewards.md) §11–§12) |
| [[karma]] | — | the neuron | the running sum of its scores: it cannot be bought or transferred, and it weighs the neuron's future links |

So being right pays twice, from the losers and from the network, and being wrong costs once, to the winners. The premium is what makes staking on knowledge better than a zero-sum game; the redistribution is what makes it a game with something to lose.

### and again

The link stays open. A later neuron who knows better stakes against it; the price moves; focus moves; the next epoch's serum scores the new stand. A link is true for as long as it survives this, and that is the only sense of the word the network uses.

---

## why honesty is the best play

| what a neuron does | what happens to it |
|---|---|
| stakes what it believes and predicts the crowd as it really expects | the highest expected score of any strategy, when others do the same: this is Prelec's theorem |
| copies the visible majority | the majority is already expected, so its side shows no surprise: nothing earned, and the stake stands at risk if the majority turns out wrong |
| goes against the crowd with no knowledge | has nothing that puts it on the surprising side, mispredicts where the crowd stands, and pays for the misprediction |
| stakes both sides | pays itself: the two positions cancel, and the fees and the lock are spent for nothing |
| splits its stake across many keys | the same total: every rule is linear in stake |
| stakes passively | no risk, no reward: it bought ranking |
| joins a ring that agrees on a falsehood | every member predicts the ring's answer, so the answer shows no surprise and scores nothing; to move the price as well, the ring must keep buying the losing side from everyone who sees the mistake, a cost that runs for as long as the lie stands ([strong truthfulness](../../specs/strong-truthfulness.md)) |

The mechanism rewards honesty and stays silent on correctness. A neuron that honestly holds an error loses to those who know better, and that loss is how the error gets corrected. Honest mistakes scatter around the fact; strategic reports lean one way; the sum of the first converges and the sum of the second does not.

---

## two epochs, with numbers

The rates are invented to show the shape. The rules are the network's.

A question `p` has a link to an answer `q`. In the first epoch three neurons take sides.

| neuron | side and stake | predicts the share staked for | |
|---|---|---|---|
| A | 10,000 for | 40 % | knows the answer, and knows few do |
| B | 5,000 for | 55 % | read the source |
| C | 8,000 against | 30 % | holds the common error |

The stake stands 15,000 for and 8,000 against: 65 % for. The crowd predicted about 40 %. "For" is the surprisingly popular side.

| neuron | the redistribution | the premium | the record |
|---|---|---|---|
| A | + 267 | + 20 | karma up |
| B | + 133 | + 10 | karma up |
| C | − 400 | 0 | karma down |

C pays 400 and A and B receive exactly 400 between them, in proportion to their stakes. The premium of 30 is new money on top, shared by what each stake added to the network's syntropy.

In the second epoch D sees that the link is winning and adds 6,000 for it, predicting 75 % for, as everyone now does. The stake stands at about 73 % for, close to what was predicted. There is no surprise left on this link, so D's score is near zero: nothing lost, nothing earned. Arriving after the crowd is a purchase of ranking at the price of locked money.

Had C been right, the same table would run the other way: "against" would have come out more common than predicted, and A and B would have paid C.

---

## where the outside world comes in

The chain reads no news. The world enters through neurons: someone who saw the event, ran the experiment or read the source stakes on what it knows, and is paid when that turns out to be more than the crowd expected. The network therefore knows what its neurons know and are willing to stake on. A fact nobody stakes on is a fact the network does not hold.

A claim about an event fits the same machine. "This happened" is a link like any other; those who watched stake for it, and they are few against a crowd that has yet to hear. That is exactly the lopsided case the serum is built to detect, and the market's price is the running forecast before the crowd catches up.

---

## how far it can be trusted

| claim | standing |
|---|---|
| focus has one value and every node computes the same | proven, and checked by every node each epoch ([[collective focus theorem]], [epoch](../../specs/epoch.md)) |
| telling the truth is the best reply when others do | proven for the serum's report (Prelec 2004) |
| the surprisingly popular side is the true one when no outside judge exists | proven under the standard model of peer prediction: a common prior and independent private evidence ([strong truthfulness](../../specs/strong-truthfulness.md)); weaker when everyone's evidence comes from one source |
| an informed minority beats a coordinated majority | measured in the scorer at up to 200 agents ([receipt](../../audit/coordinated-consensus-scoring.md)) |
| holding a false price costs a ring more than it gains | a model, with the threshold of honest capital stated ([strong truthfulness](../../specs/strong-truthfulness.md) § the capital bound); awaits a network |
| a majority of all stake that agrees to invert its reports | wins: no scoring defends against it, as no chain defends against a majority of its work |
| the whole payoff taken together keeps honesty the best reply | assumed, and bounded where measured ([rewards](../../specs/rewards.md) §15) |
| the network converges on what is the case | the hypothesis the network exists to test, in public, under real stake |

---

## the four questions, answered

- What is truth? The side of a link that keeps coming out more common than the crowd predicted, and so keeps its price and its focus. It is held, epoch after epoch, and it can be overturned.
- Who asks and who answers? Any neuron. A link from one file to another is a question answered.
- Why stake on it? Being right is paid twice, by those who were wrong and by the network's premium for created knowledge, and it builds karma that weighs every later link. Holding tokens idle is paid nothing.
- Who checks? The crowd checks itself: every staker's prediction of the others is the yardstick the stand is measured against, every node recomputes the result, and every neuron may stake against any link at any time. There is no outside judge and no closing day.

---

## where it stands

The mechanism above is the law of [rewards](../../specs/rewards.md). Its parts are at different distances from a running network: focus, attribution and the focus pool run in consensus on the rehearsal network; the serum's scorer, the market's cost function and the annuity's accrual are implemented and tested in this repository and are yet to be joined to the chain. The project that joins them before genesis is [the truth market at genesis](https://github.com/cyberia-to/tru/blob/master/projects/truth-market-at-genesis.md).

---

## where the rules are

- the reward law: [rewards](../../specs/rewards.md) — §1 the principle, §5 honesty, §9 the two axes of stake, §11 the reward equation, §12 timing, §15 security
- the serum: [truth scoring](../../specs/truth-scoring.md), [strong truthfulness](../../specs/strong-truthfulness.md)
- the market: [epistemic markets](epistemic-markets.md), [markets](markets.md), and the protocol of one link, [market](../terms/market.md)
- the epoch's computation: [epoch](../../specs/epoch.md)
- the general law in narrative: [honesty](honesty.md), [valence](valence.md), [incentives](incentives.md)
