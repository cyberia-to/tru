---
tags: cyber, tru, docs
alias: truth explained, staking on truth, a stake on truth, how truth works, truth from claim to payout
crystal-type: article
crystal-domain: cyber
---
# truth, from claim to payout

For a reader who knows how a prediction market works and asks: what is truth here, who asks and who answers, why would anyone stake on it, and who checks.

A prediction market waits for the world to settle its question. Most knowledge has no such day, so [[cyber]] settles inside itself, every epoch. Every link has two sides, true and false. The side that turns out more common than the crowd itself predicted wins. The other side pays it, and the network adds a premium for the knowledge the winners created. The link stays open to a new stake forever.

---

## the machine you already know

| | a prediction market | cyber |
|---|---|---|
| the question | written by an organizer | the link itself, made by any [[neuron]] |
| the stake | on an outcome of an event | on the link: true or false |
| the judge | the world, once | the crowd's own surprise, every epoch |
| who pays | the losing side pays the winning side | the losing side pays the winning side, and the network adds a premium |
| the end | the market closes | the link stays open; a later stake can overturn it |

A market needs an event that closes the question. "This paper answers that question" or "this file is the source of that claim" never get one, and people rely on them every day. That is the knowledge cyber is built for.

---

## the link, the stake and one signed number

A [[cyberlink]] `p → q` joins two [[files]] and says one thing: from `p`, the file `q` deserves attention. If `p` is a question and `q` a text, the link is an answer. Nobody is appointed to ask or to answer: any neuron makes the link, and whoever reads `p` later sees its links ordered by [[focus]].

A link with no stake is free speech: it carries no weight. To take part a neuron locks a stake on the link and sets one signed number, the activation.

| the activation | its sign says | its size says |
|---|---|---|
| above zero | this link is true | how much of the stake on this link I expect to stand with me |
| below zero | this link is false | the same |
| zero | I take no side | my stake weighs the link, risks nothing and earns nothing |

The size runs from 1 to 127 and is read as a share of 128. So one number answers two questions: what do I think, and how many will think the same.

| activation | reads as |
|---|---|
| `+115` | true, and nine in ten will agree with me |
| `+26` | true, and only one in five will agree: I know something the crowd does not |
| `−115` | false, and nine in ten will agree with me |
| `−26` | false, and only one in five will agree: I know something the crowd does not |

The second question is the unusual one. The rest of the page turns on it.

---

## the process

<div class="truth-process">
<svg viewBox="0 0 900 250" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="the process: the claim, the two sides, focus, the serum, settlement, and back" style="width:100%;height:auto;max-width:900px;font-family:inherit">
<defs><marker id="tp-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" fill="currentColor"/></marker></defs>
<g fill="none" stroke="currentColor" stroke-width="1.5">
<rect x="10" y="50" width="160" height="130" rx="10"/><rect x="190" y="50" width="160" height="130" rx="10"/><rect x="370" y="50" width="160" height="130" rx="10" stroke="#3fb950" stroke-width="2.5"/><rect x="550" y="50" width="160" height="130" rx="10" stroke="#d29922" stroke-width="2.5"/><rect x="730" y="50" width="160" height="130" rx="10" stroke="#f85149" stroke-width="2.5"/>
<path d="M170 115H188" marker-end="url(#tp-a)"/><path d="M350 115H368" marker-end="url(#tp-a)"/><path d="M530 115H548" marker-end="url(#tp-a)"/><path d="M710 115H728" marker-end="url(#tp-a)"/>
<path d="M810 50V30H90V46" stroke-dasharray="6 5" marker-end="url(#tp-a)"/>
</g>
<g fill="currentColor" font-size="15" text-anchor="middle">
<text x="450" y="20" font-size="13" opacity="0.75">the link stays open: anyone may stake on it again</text>
<text x="90" y="80" font-weight="600">1 · the claim</text><text x="270" y="80" font-weight="600">2 · two sides</text><text x="450" y="80" font-weight="600">3 · focus</text><text x="630" y="80" font-weight="600">4 · the serum</text><text x="810" y="80" font-weight="600">5 · settlement</text>
<g font-size="13.5">
<text x="90" y="112">a link p → q,</text><text x="90" y="134">a stake,</text><text x="90" y="156">an activation</text>
<text x="270" y="112">stakes fill the link's</text><text x="270" y="134">true side and its</text><text x="270" y="156">false side</text>
<text x="450" y="112">attention settles</text><text x="450" y="134">over the whole</text><text x="450" y="156">weighted graph</text>
<text x="630" y="112">which side came</text><text x="630" y="134">out more common</text><text x="630" y="156">than predicted?</text>
<text x="810" y="112">losers pay</text><text x="810" y="134">winners; the network</text><text x="810" y="156">adds a premium</text>
</g>
<g font-size="13" opacity="0.75">
<text x="180" y="205">the premise</text><text x="450" y="205">truth emerges</text><text x="630" y="205">validation</text><text x="810" y="205">payout</text>
<text x="180" y="226">any neuron, any time</text><text x="630" y="226">every node, once an epoch</text>
</g>
</g>
</svg>
</div>

1 · the claim. A neuron makes or finds a link, locks a stake on it and sets its activation.

2 · two sides. Every link holds two reserves: the stakes that call it true and the stakes that call it false. The two are coupled: money on one side makes the other weigh less. A link the crowd calls true carries attention in full; a link it calls false is pressed toward nothing. A stake costs money, which makes spam expensive.

3 · focus. Once an epoch every node computes where attention settles when it flows along all the links by their weights. The answer is unique, every node gets the same one, and no neuron chooses it ([[collective focus theorem]]). The links that carry focus are what the network holds true this epoch.

4 · the serum. Focus alone would pay for agreement, and a crowd can agree on a mistake. The serum tells knowledge from agreement, by the sizes of the activations. Next section.

5 · settlement. The losing side pays the winning side, the network adds a premium, and the scores are recorded as [[karma]]. Then the link is open again.

---

## the serum: how a crowd is checked without a judge

Ask a crowd whether Philadelphia is the capital of Pennsylvania. Most say yes, and they are wrong. Ask each a second question: what share of the others will say yes? Those who say yes expect nearly everyone to agree. Those who say no know the real capital, and they know the mistake is common, so they expect to be few.

Count the answers. "No" comes out more common than the crowd predicted. The surprisingly popular answer is the correct one ([[Bayesian Truth Serum]], Prelec 2004; Prelec, Seung and McCoy 2017).

It works because knowledge is lopsided: whoever knows the fact also knows the popular error, and whoever holds the error has never heard of the fact. So the informed side is always underestimated, and that gap can be measured without anyone knowing the answer in advance.

On a link the two questions are the sign and the size of the activation. Take the link "capital of Pennsylvania → Philadelphia":

| neuron | stake | activation | reads as |
|---|---|---|---|
| A | 6,000 | `−26` | false, and one in five will agree with me |
| B | 2,000 | `−26` | the same |
| C | 9,000 | `+115` | true, and nine in ten will agree with me |
| D | 3,000 | `+115` | the same |

How the crowd answered: 12,000 true, 8,000 false. False holds 40 % of the stake.

How the crowd predicted it would answer: C and D expected a tenth to say false; A and B expected a fifth. Weighed by stake, the crowd predicted 14 % for false.

False came out 26 points more common than predicted. It wins, with less money, and the link is pressed down.

| neuron | pays or receives | premium | karma |
|---|---|---|---|
| A | + 232 | + 23 | up |
| B | + 77 | + 8 | up |
| C | − 232 | 0 | down |
| D | − 77 | 0 | down |

Why 309 and not 12,000: the losing side pays a share of its stake each epoch, in proportion to how surprising the winner was. One epoch is one count, and one count can be wrong, so it moves a part and leaves the rest standing. A side that stays wrong pays again next epoch, and the one after.

Now the same link attacked by a ring that knows nothing. It stakes 1,000 on false against 19,000 on true. The majority had said nine in ten would agree, so a tenth on the other side was expected, and the ring is only a twentieth. It comes out less common than predicted, and it pays. A small group cannot win by merely being small: it has to be larger than the crowd thought possible.

Move the sliders to try other crowds:

<div id="truth-epoch"></div>
<style>#truth-epoch{border:1px solid rgba(128,128,128,.4);border-radius:10px;padding:16px 18px;margin:18px 0;max-width:720px}#truth-epoch label{display:block;margin:0 0 12px;font-size:.95em}#truth-epoch label span{float:right;font-variant-numeric:tabular-nums}#truth-epoch input[type=range]{display:block;width:100%;margin-top:4px}#truth-epoch .bar{display:grid;grid-template-columns:15em 1fr 3.5em;gap:10px;align-items:center;margin:7px 0;font-size:.9em}#truth-epoch .bar i{display:block;height:12px;border-radius:6px;background:rgba(128,128,128,.22);overflow:hidden}#truth-epoch .bar em{display:block;height:100%}#truth-epoch .bar span:last-child{text-align:right;font-variant-numeric:tabular-nums}#truth-epoch p{margin:10px 0 0}#truth-epoch .v{font-size:1.05em}#truth-epoch .n{font-variant-numeric:tabular-nums;border-bottom:2px solid currentColor}#truth-epoch .note{font-size:.85em;opacity:.7}#truth-epoch h4{margin:14px 0 8px;font-size:.95em;font-weight:600}@media(max-width:560px){#truth-epoch .bar{grid-template-columns:1fr 3.5em}#truth-epoch .bar i{grid-column:1/3;grid-row:2}}</style>
<script>(function(){var r=document.getElementById('truth-epoch');if(!r)return;r.innerHTML='<label>stake that calls the link true <span id="te-av"></span><input id="te-a" type="range" min="0" max="20000" step="500" value="12000"></label><label>its activation: how much of the stake it expects on its side <span id="te-mav"></span><input id="te-ma" type="range" min="1" max="127" step="1" value="115"></label><label>stake that calls the link false <span id="te-bv"></span><input id="te-b" type="range" min="0" max="20000" step="500" value="8000"></label><label>its activation: how much of the stake it expects on its side <span id="te-mbv"></span><input id="te-mb" type="range" min="1" max="127" step="1" value="26"></label><div id="te-out"></div>';function g(i){return document.getElementById(i)}function n(x){return Math.round(x).toLocaleString('en-US')}function pc(x){return Math.round(x*100)+' %'}function bar(l,v,c){return '<div class="bar"><span>'+l+'</span><i><em style="width:'+Math.round(v*100)+'%;background:'+c+'"></em></i><span>'+pc(v)+'</span></div>'}function u(){var a=+g('te-a').value,b=+g('te-b').value,ma=+g('te-ma').value,mb=+g('te-mb').value;g('te-av').textContent=n(a);g('te-bv').textContent=n(b);g('te-mav').textContent='+'+ma+' · '+pc(ma/128);g('te-mbv').textContent='−'+mb+' · '+pc(mb/128);var t=a+b,o=g('te-out');if(t===0){o.innerHTML='<p class="v">No stake, no claim: the link carries no weight.</p>';return}var st=a/t,pt=(a*ma/128+b*(1-mb/128))/t,gap=st-pt,h='<h4>true</h4>'+bar('how the crowd answered',st,'#3fb950')+bar('how it predicted it would',pt,'#d29922')+'<h4>false</h4>'+bar('how the crowd answered',1-st,'#3fb950')+bar('how it predicted it would',1-pt,'#d29922'),v;if(Math.abs(gap)<0.03){v='<p class="v">The crowd answered as it expected to. No surprise: nobody pays, nobody earns.</p>'}else{var w=gap>0,win=w?'True':'False',ls=w?b:a,ws=w?a:b,pay=0.1*Math.abs(gap)*ls;v='<p class="v">'+win+' came out '+Math.round(Math.abs(gap)*100)+' points more common than predicted: the surprisingly popular side'+(ws<ls?', though it holds less money':'')+'.</p><p>The '+(w?'false':'true')+' side pays <span class="n">'+n(pay)+'</span>. The '+(w?'true':'false')+' side receives <span class="n">'+n(pay)+'</span> from it and a premium of <span class="n">'+n(pay*0.1)+'</span> in new tokens.</p>'}o.innerHTML=h+v+'<p class="note">The rates are invented to show the shape: the losing side pays a tenth of its stake times the surprise, and the premium is a tenth of that.</p>'}['te-a','te-b','te-ma','te-mb'].forEach(function(i){g(i).addEventListener('input',u)});u()})();</script>

Once the crowd expects the link to be false, calling it false shows no surprise: a neuron that arrives then pays nothing and earns nothing. Drag the last slider to the right and watch the payment vanish.

---

## what settlement pays

| flow | from | to | by what measure |
|---|---|---|---|
| the redistribution | the losing side | the winning side | the serum's score; it sums to zero ([rewards](../../specs/rewards.md) §5) |
| the premium | new tokens, through the focus pool | the winners | the gain in [[syntropy]] their stake caused: money is minted only where knowledge was created (§1–§4) |
| the annuity | new tokens | links that keep carrying focus | the growth of focus on the link's target in later epochs: foundational links earn as the graph builds on them (§11–§12) |
| [[karma]] | — | the neuron | the running sum of its scores; it cannot be bought or transferred, and it weighs the neuron's future links |

Being right pays twice, from the losers and from the network. Being wrong costs once, to the winners.

---

## why honesty is the best play

| what a neuron does | what happens to it |
|---|---|
| takes the side it believes and names the share it really expects | the highest expected score when others do the same: Prelec's theorem |
| copies the visible majority | the majority is already expected: no surprise, nothing earned, and the stake is at risk if the majority is wrong |
| takes the unpopular side with no knowledge | it is as few as the crowd expected, or fewer: it pays |
| names a small share to look informed | its side must then come out larger than it said, which nothing it controls makes happen; the misprediction costs it |
| stakes both sides | pays itself; the fees and the lock are spent for nothing |
| splits its stake across many keys | the same total: every rule is linear in stake |
| joins a ring that agrees on a falsehood | every member expects the ring's answer, so it shows no surprise and scores nothing ([strong truthfulness](../../specs/strong-truthfulness.md)) |

The mechanism rewards honesty and stays silent on correctness. A neuron that honestly holds an error loses to those who know better, and that loss is how the error gets corrected.

The chain reads no news. The world enters through neurons: someone who saw the event or read the source stakes on what it knows, and wins when that turns out to be more than the crowd expected. A fact nobody stakes on is a fact the network does not hold.

---

## how far it can be trusted

| claim | standing |
|---|---|
| focus has one value and every node computes the same | proven, and checked by every node each epoch ([epoch](../../specs/epoch.md)) |
| telling the truth is the best reply when others do | proven for the serum's report (Prelec 2004) |
| the surprisingly popular side is the true one | proven when the stakers' evidence is independent ([strong truthfulness](../../specs/strong-truthfulness.md)); weaker when it all comes from one source |
| a minority that shares one error and expects to be few | reads as knowledge in a single count; the open link is the correction, epoch after epoch |
| an informed minority beats a coordinated majority | measured in the scorer at up to 200 agents ([receipt](../../audit/coordinated-consensus-scoring.md)) |
| a majority of all stake that agrees to invert its reports | wins: no scoring defends against it, as no chain defends against a majority of its work |
| the network converges on what is the case | the hypothesis the network exists to test, in public, under real stake |

---

## the four questions, answered

- What is truth? The side of a link that keeps coming out more common than the crowd predicted, and so keeps its weight and its focus. It is held, epoch after epoch, and it can be overturned.
- Who asks and who answers? Any neuron. A link from one file to another is a question answered; a stake on it says true or false.
- Why stake on it? Being right is paid by those who were wrong and by the network's premium, and it builds karma. Holding tokens idle is paid nothing.
- Who checks? The crowd checks itself: every staker's expectation of the others is the yardstick, every node recomputes the result, and anyone may stake on either side of any link at any time.

---

## where it stands, and where the rules are

This is the law of [rewards](../../specs/rewards.md), told with the record the stack is moving to. On the rehearsal network today the link carries a three-valued valence: `+1`, `0`, `−1`. That is the side without the share, so the serum has nothing to count yet. The signed activation described here is one of two records under decision in the project [activation](https://github.com/cyberia-to/soft3/blob/release/phase1/projects/activation.md); the other keeps the three values and puts the prediction on a second link. Focus and the focus pool run in consensus; the serum's scorer and the annuity's accrual are built and tested in this repository, and [the truth market at genesis](../../projects/truth-market-at-genesis.md) joins them to the chain before launch. The rates are fixed there; here they are in words.

- the reward law: [rewards](../../specs/rewards.md) — §1 the principle, §5 honesty, §9 the two axes of stake, §11–§12 the annuity, §15 security
- the serum: [truth scoring](../../specs/truth-scoring.md), [strong truthfulness](../../specs/strong-truthfulness.md)
- the general law in narrative: [honesty](honesty.md), [valence](valence.md), [incentives](incentives.md)
