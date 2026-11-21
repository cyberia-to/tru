---
tags: cyber, tru, docs
alias: truth explained, staking on truth, a stake on truth, how truth works, truth from claim to payout
crystal-type: article
crystal-domain: cyber
---
# truth, from claim to payout

For a reader who knows how a prediction market works and asks: what is truth here, who asks and who answers, why would anyone stake on it, and who checks.

A prediction market waits for the world to settle its question. Most knowledge has no such day, so [[cyber]] settles inside itself, every epoch. A link is a claim. Of the links that answer one question, the one that turns out more common than the crowd itself predicted wins. Those who staked on the others pay it, and the network adds a premium for the knowledge the winners created. Every link stays open to a new stake forever.

---

## the machine you already know

| | a prediction market | cyber |
|---|---|---|
| the question | written by an organizer | a [[particle]]: any file |
| the answers | a fixed list of outcomes | the links that leave it, made by any [[neuron]] |
| the stake | on an outcome | on a link |
| the judge | the world, once | the crowd's own surprise, every epoch |
| who pays | the losing side pays the winning side | the losing side pays the winning side, and the network adds a premium |
| the end | the market closes | every link stays open; a later stake can overturn it |

A market needs an event that closes the question. "This paper answers that question" or "this file is the source of that claim" never get one, and people rely on them every day. That is the knowledge cyber is built for.

---

## the link is the claim

A [[cyberlink]] `p → q` joins two [[files]] and says one thing: from `p`, the file `q` deserves attention. Making the link is saying "this is true". If `p` is a question and `q` a text, the link is an answer.

Nobody is appointed to ask or to answer. Any neuron makes a link, and whoever reads `p` later sees its links ordered by [[focus]]. A neuron that disagrees with a link makes or backs a better one from the same `p`: the links that leave one particle compete for its attention, so there is no vote against, only a stronger answer.

A link with no stake is free speech: it carries no weight. To make it count a neuron adds two things.

| what the neuron commits | what it says | in the cyberlink |
|---|---|---|
| the link | this is my answer | the two particles |
| a stake | this much stands behind it | the amount locked |
| a prediction | this is what I expect of the crowd | the [[valence]]: `+1` the crowd will hold my answer, `−1` the crowd will hold another, `0` no prediction |

The link is what the neuron believes. The valence is what it expects of everyone else. A stake with valence `0` gives the link weight in the ranking, takes no part in the game, risks nothing and earns nothing ([rewards](../../specs/rewards.md) §5, §9).

---

## the process

<div class="truth-process">
<svg viewBox="0 0 900 250" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="the process: the claim, the weight, focus, the serum, settlement, and back" style="width:100%;height:auto;max-width:900px;font-family:inherit">
<defs><marker id="tp-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" fill="currentColor"/></marker></defs>
<g fill="none" stroke="currentColor" stroke-width="1.5">
<rect x="10" y="50" width="160" height="130" rx="10"/><rect x="190" y="50" width="160" height="130" rx="10"/><rect x="370" y="50" width="160" height="130" rx="10" stroke="#3fb950" stroke-width="2.5"/><rect x="550" y="50" width="160" height="130" rx="10" stroke="#d29922" stroke-width="2.5"/><rect x="730" y="50" width="160" height="130" rx="10" stroke="#f85149" stroke-width="2.5"/>
<path d="M170 115H188" marker-end="url(#tp-a)"/><path d="M350 115H368" marker-end="url(#tp-a)"/><path d="M530 115H548" marker-end="url(#tp-a)"/><path d="M710 115H728" marker-end="url(#tp-a)"/>
<path d="M810 50V30H90V46" stroke-dasharray="6 5" marker-end="url(#tp-a)"/>
</g>
<g fill="currentColor" font-size="15" text-anchor="middle">
<text x="450" y="20" font-size="13" opacity="0.75">every link stays open: anyone may stake again</text>
<text x="90" y="80" font-weight="600">1 · the claim</text><text x="270" y="80" font-weight="600">2 · the weight</text><text x="450" y="80" font-weight="600">3 · focus</text><text x="630" y="80" font-weight="600">4 · the serum</text><text x="810" y="80" font-weight="600">5 · settlement</text>
<g font-size="13.5">
<text x="90" y="112">a link p → q,</text><text x="90" y="134">a stake,</text><text x="90" y="156">a prediction</text>
<text x="270" y="112">stakes weigh the</text><text x="270" y="134">links; answers to one</text><text x="270" y="156">question compete</text>
<text x="450" y="112">attention settles</text><text x="450" y="134">over the whole</text><text x="450" y="156">weighted graph</text>
<text x="630" y="112">which answer came</text><text x="630" y="134">out more common</text><text x="630" y="156">than predicted?</text>
<text x="810" y="112">losers pay</text><text x="810" y="134">winners; the network</text><text x="810" y="156">adds a premium</text>
</g>
<g font-size="13" opacity="0.75">
<text x="180" y="205">the premise</text><text x="450" y="205">truth emerges</text><text x="630" y="205">validation</text><text x="810" y="205">payout</text>
<text x="180" y="226">any neuron, any time</text><text x="630" y="226">every node, once an epoch</text>
</g>
</g>
</svg>
</div>

1 · the claim. A neuron makes a link, locks a stake on it and sets its valence.

2 · the weight. Every stake adds weight to its link. The links that leave one particle share that particle's attention in proportion to their weights, so backing one answer takes attention from the others. A link costs money, which makes spam expensive.

3 · focus. Once an epoch every node computes where attention settles when it flows along all the links by their weights. The answer is unique, every node gets the same one, and no neuron chooses it ([[collective focus theorem]]). The links that carry focus are what the network holds true this epoch.

4 · the serum. Focus alone would pay for agreement, and a crowd can agree on a mistake. The serum tells knowledge from agreement, by the valences. Next section.

5 · settlement. The losing answers pay the winning one, the network adds a premium, and the scores are recorded as [[karma]]. Then every link is open again.

---

## the serum: how a crowd is checked without a judge

Ask a crowd for the capital of Pennsylvania. Most say Philadelphia, and they are wrong. Ask each a second question: what will most people answer? Those who say Philadelphia expect everyone to agree. Those who say Harrisburg know the real capital, and they know the mistake is common, so they too predict that most will say Philadelphia.

Count the answers. Harrisburg comes out more common than the crowd predicted. The surprisingly popular answer is the correct one ([[Bayesian Truth Serum]], Prelec 2004; Prelec, Seung and McCoy 2017).

It works because knowledge is lopsided: whoever knows the fact also knows the popular error, and whoever holds the error has never heard of the fact. So the informed answer is always underestimated, and that gap can be measured without anyone knowing the answer in advance.

On the graph the question is a particle, the answers are its links, the stakes say how the crowd answered and the valences say how it expected to answer. The same question as two links:

| neuron | its link | stake | valence | what the valence says |
|---|---|---|---|---|
| A | capital → Harrisburg | 6,000 | `−1` | I know the answer, and I know the crowd will hold the other |
| B | capital → Harrisburg | 2,000 | `−1` | the same |
| C | capital → Philadelphia | 9,000 | `+1` | everyone will answer as I do |
| D | capital → Philadelphia | 3,000 | `+1` | the same |

Nobody predicted that the crowd would hold Harrisburg, and 40 % of the stake stands on it. It is 40 points more common than predicted. It wins, with less money.

| neuron | pays or receives | premium | karma |
|---|---|---|---|
| A | + 360 | + 36 | up |
| B | + 120 | + 12 | up |
| C | − 360 | 0 | down |
| D | − 120 | 0 | down |

Why 480 and not 12,000: the losing answer pays a share of its stake each epoch, in proportion to how surprising the winner was. One epoch is one count, and one count can be wrong, so it moves a part and leaves the rest standing. An answer that stays wrong pays again next epoch, and the one after. C and D lose 4 % now and keep losing while they hold.

Move the sliders to try other crowds:

<div id="truth-epoch"></div>
<style>#truth-epoch{border:1px solid rgba(128,128,128,.4);border-radius:10px;padding:16px 18px;margin:18px 0;max-width:720px}#truth-epoch label{display:block;margin:0 0 12px;font-size:.95em}#truth-epoch label span{float:right;font-variant-numeric:tabular-nums}#truth-epoch input[type=range]{display:block;width:100%;margin-top:4px}#truth-epoch .bar{display:grid;grid-template-columns:15em 1fr 3.5em;gap:10px;align-items:center;margin:7px 0;font-size:.9em}#truth-epoch .bar i{display:block;height:12px;border-radius:6px;background:rgba(128,128,128,.22);overflow:hidden}#truth-epoch .bar em{display:block;height:100%}#truth-epoch .bar span:last-child{text-align:right;font-variant-numeric:tabular-nums}#truth-epoch p{margin:10px 0 0}#truth-epoch .v{font-size:1.05em}#truth-epoch .n{font-variant-numeric:tabular-nums;border-bottom:2px solid currentColor}#truth-epoch .note{font-size:.85em;opacity:.7}#truth-epoch h4{margin:14px 0 8px;font-size:.95em;font-weight:600}@media(max-width:560px){#truth-epoch .bar{grid-template-columns:1fr 3.5em}#truth-epoch .bar i{grid-column:1/3;grid-row:2}}</style>
<script>(function(){var r=document.getElementById('truth-epoch');if(!r)return;r.innerHTML='<label>stake on the link to Philadelphia <span id="te-av"></span><input id="te-a" type="range" min="0" max="20000" step="500" value="12000"></label><label>of it, with valence +1: the crowd will hold my answer <span id="te-pav"></span><input id="te-pa" type="range" min="0" max="100" step="5" value="100"></label><label>stake on the link to Harrisburg <span id="te-bv"></span><input id="te-b" type="range" min="0" max="20000" step="500" value="8000"></label><label>of it, with valence +1: the crowd will hold my answer <span id="te-pbv"></span><input id="te-pb" type="range" min="0" max="100" step="5" value="0"></label><div id="te-out"></div>';function g(i){return document.getElementById(i)}function n(x){return Math.round(x).toLocaleString('en-US')}function pc(x){return Math.round(x*100)+' %'}function bar(l,v,c){return '<div class="bar"><span>'+l+'</span><i><em style="width:'+Math.round(v*100)+'%;background:'+c+'"></em></i><span>'+pc(v)+'</span></div>'}function u(){var a=+g('te-a').value,b=+g('te-b').value,pa=+g('te-pa').value/100,pb=+g('te-pb').value/100;g('te-av').textContent=n(a);g('te-bv').textContent=n(b);g('te-pav').textContent=pc(pa);g('te-pbv').textContent=pc(pb);var t=a+b,o=g('te-out');if(t===0){o.innerHTML='<p class="v">No stake, no claim: the links carry no weight.</p>';return}var sb=b/t,qb=(b*pb+a*(1-pa))/t,gap=sb-qb,h='<h4>Harrisburg</h4>'+bar('how the crowd answered',sb,'#3fb950')+bar('how it predicted it would',qb,'#d29922')+'<h4>Philadelphia</h4>'+bar('how the crowd answered',1-sb,'#3fb950')+bar('how it predicted it would',1-qb,'#d29922'),v;if(Math.abs(gap)<0.03){v='<p class="v">The crowd answered as it expected to. No surprise: nobody pays, nobody earns.</p>'}else{var w=gap>0,win=w?'Harrisburg':'Philadelphia',los=w?'Philadelphia':'Harrisburg',ls=w?a:b,ws=w?b:a,pay=0.1*Math.abs(gap)*ls;v='<p class="v">'+win+' came out '+Math.round(Math.abs(gap)*100)+' points more common than predicted: the surprisingly popular answer'+(ws<ls?', though it holds less money':'')+'.</p><p>The stake on '+los+' pays <span class="n">'+n(pay)+'</span>. The stake on '+win+' receives <span class="n">'+n(pay)+'</span> from it and a premium of <span class="n">'+n(pay*0.1)+'</span> in new tokens.</p>'}o.innerHTML=h+v+'<p class="note">The rates are invented to show the shape: the losing answer pays a tenth of its stake times the surprise, and the premium is a tenth of that.</p>'}['te-a','te-b','te-pa','te-pb'].forEach(function(i){g(i).addEventListener('input',u)});u()})();</script>

Once the crowd expects Harrisburg, staking on it shows no surprise: a neuron that arrives then pays nothing and earns nothing. Drag the last slider to the right and watch the payment vanish.

---

## what settlement pays

| flow | from | to | by what measure |
|---|---|---|---|
| the redistribution | the losing answers | the winning answer | the serum's score; it sums to zero ([rewards](../../specs/rewards.md) §5) |
| the premium | new tokens, through the focus pool | the winners | the gain in [[syntropy]] their stake caused: money is minted only where knowledge was created (§1–§4) |
| the annuity | new tokens | links that keep carrying focus | the growth of focus on the link's target in later epochs: foundational links earn as the graph builds on them (§11–§12) |
| [[karma]] | — | the neuron | the running sum of its scores; it cannot be bought or transferred, and it weighs the neuron's future links |

Being right pays twice, from the losers and from the network. Being wrong costs once, to the winners.

---

## why honesty is the best play

| what a neuron does | what happens to it |
|---|---|
| links what it believes, predicts the crowd as it really expects | the highest expected score when others do the same: Prelec's theorem |
| copies the visible majority | the majority is already expected: no surprise, nothing earned, and the stake is at risk if the majority is wrong |
| backs an unpopular answer with no knowledge | mispredicts where the crowd stands, and pays for it |
| stakes on two competing answers | pays itself; the fees and the lock are spent for nothing |
| splits its stake across many keys | the same total: every rule is linear in stake |
| joins a ring that agrees on a falsehood | every member predicts the ring's answer, so it shows no surprise and scores nothing ([strong truthfulness](../../specs/strong-truthfulness.md)) |

The mechanism rewards honesty and stays silent on correctness. A neuron that honestly holds an error loses to those who know better, and that loss is how the error gets corrected.

The chain reads no news. The world enters through neurons: someone who saw the event or read the source links what it knows, and wins when that turns out to be more than the crowd expected. A fact nobody stakes on is a fact the network does not hold.

---

## how far it can be trusted

| claim | standing |
|---|---|
| focus has one value and every node computes the same | proven, and checked by every node each epoch ([epoch](../../specs/epoch.md)) |
| telling the truth is the best reply when others do | proven for the serum's report (Prelec 2004) |
| the surprisingly popular answer is the true one | proven when the stakers' evidence is independent ([strong truthfulness](../../specs/strong-truthfulness.md)); weaker when it all comes from one source |
| a minority that shares one error and knows itself a minority | reads as knowledge in a single count; the open links are the correction, epoch after epoch |
| an informed minority beats a coordinated majority | measured in the scorer at up to 200 agents ([receipt](../../audit/coordinated-consensus-scoring.md)) |
| a majority of all stake that agrees to invert its reports | wins: no scoring defends against it, as no chain defends against a majority of its work |
| the network converges on what is the case | the hypothesis the network exists to test, in public, under real stake |

---

## the four questions, answered

- What is truth? The link that keeps coming out more common than the crowd predicted, and so keeps its focus. It is held, epoch after epoch, and it can be overturned.
- Who asks and who answers? Any neuron. A particle is a question; a link that leaves it is an answer.
- Why stake on it? Being right is paid by those who were wrong and by the network's premium, and it builds karma. Holding tokens idle is paid nothing.
- Who checks? The crowd checks itself: every staker's prediction of the others is the yardstick, every node recomputes the result, and anyone may back a better answer at any time.

---

## where it stands, and where the rules are

This is the law of [rewards](../../specs/rewards.md). Focus and the focus pool run in consensus on the rehearsal network. The serum's scorer and the annuity's accrual are built and tested in this repository and are yet to be joined to the chain: the project [the truth market at genesis](../../projects/truth-market-at-genesis.md) joins them before launch, and its first task fixes the rates and the arithmetic this page leaves in words.

- the reward law: [rewards](../../specs/rewards.md) — §1 the principle, §5 honesty, §9 the two axes of stake, §11–§12 the annuity, §15 security
- the serum: [truth scoring](../../specs/truth-scoring.md), [strong truthfulness](../../specs/strong-truthfulness.md)
- the general law in narrative: [honesty](honesty.md), [valence](valence.md), [incentives](incentives.md)
