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

## the link, the stake and the activation

A [[cyberlink]] `p → q` joins two [[files]] and says one thing: from `p`, the file `q` deserves attention. If `p` is a question and `q` a text, the link is an answer. Nobody is appointed to ask or to answer: any neuron makes the link, and whoever reads `p` later sees its links ordered by [[focus]].

A link with no stake is free speech: it carries no weight. To take part a neuron locks a stake on the link and sets one signed number, the activation. It is one byte, and it answers two questions at once.

| | the question | the answer |
|---|---|---|
| the sign | what do I think of this link? | plus: it is true. minus: it is false |
| the size | how many will think otherwise? | from 1 to 127, read as a share of 128 of the stake on this link |
| zero | — | I take no side: my stake weighs the link, risks nothing and earns nothing |

<div class="truth-scale">
<svg viewBox="0 0 900 190" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="the scale of the activation from minus 127 to plus 127" style="width:100%;height:auto;max-width:900px;font-family:inherit">
<defs><linearGradient id="ts-f" x1="1" x2="0"><stop offset="0" stop-color="#f85149" stop-opacity="0.15"/><stop offset="1" stop-color="#f85149" stop-opacity="0.9"/></linearGradient><linearGradient id="ts-t" x1="0" x2="1"><stop offset="0" stop-color="#3fb950" stop-opacity="0.15"/><stop offset="1" stop-color="#3fb950" stop-opacity="0.9"/></linearGradient></defs>
<rect x="40" y="70" width="400" height="26" rx="6" fill="url(#ts-f)"/><rect x="460" y="70" width="400" height="26" rx="6" fill="url(#ts-t)"/>
<g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M450 60V106"/><path d="M119 66V100"/><path d="M399 66V100"/><path d="M501 66V100"/><path d="M781 66V100"/></g>
<g fill="currentColor" text-anchor="middle" font-size="14">
<text x="240" y="30" font-weight="600">false</text><text x="660" y="30" font-weight="600">true</text><text x="450" y="50" font-size="13">0 · no side</text>
<text x="119" y="58" font-size="13">−102</text><text x="399" y="58" font-size="13">−13</text><text x="501" y="58" font-size="13">+13</text><text x="781" y="58" font-size="13">+102</text>
<text x="40" y="118" font-size="12" text-anchor="start">−127</text><text x="860" y="118" font-size="12" text-anchor="end">+127</text>
<g font-size="13" opacity="0.85">
<text x="129" y="140">false, and four in five</text><text x="129" y="158">will disagree with me</text>
<text x="350" y="140">false, as nearly</text><text x="350" y="158">everyone thinks</text>
<text x="550" y="140">true, as nearly</text><text x="550" y="158">everyone thinks</text>
<text x="771" y="140">true, and four in five</text><text x="771" y="158">will disagree with me</text>
</g>
<g font-size="12" opacity="0.6"><text x="129" y="180">loud: I bring news</text><text x="450" y="180">quiet: common knowledge</text><text x="771" y="180">loud: I bring news</text></g>
</g>
</svg>
</div>

The size reads as strength. Near zero a neuron says what everyone says, and that is quiet. Far from zero it says "I am right and almost nobody knows it yet", and that is loud. Shouting does not pay by itself: the size is a prediction, and it is checked against the count.

From this one byte, summed over everyone who staked, a link gets three quantities where a plain vote gives one:

| quantity | where it comes from | what it is for |
|---|---|---|
| how the crowd answered | the stakes behind plus against the stakes behind minus | the weight of the link: a link called false is damped, and nothing flows backward |
| how the crowd expected to answer | the sizes, weighed by stake | the yardstick the answer is measured against |
| the surprise | the first minus the second | who wins: the side that came out more common than expected |

The surprise is the unusual one. The rest of the page turns on it.

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

On a link the two questions are the sign and the size of the activation. Take the link "capital of Pennsylvania → Philadelphia" and six neurons, each with its own view of the crowd:

| neuron | stake | activation | reads as | so it expects on false |
|---|---|---|---|---|
| A | 5,000 | `−110` | false, and nearly nine in ten will disagree with me | 14 % |
| B | 3,000 | `−77` | false, and six in ten will disagree | 40 % |
| C | 7,000 | `+6` | true, and one in twenty will disagree | 5 % |
| D | 3,000 | `+26` | true, and one in five will disagree | 20 % |
| E | 2,000 | `+13` | true, and one in ten will disagree | 10 % |
| F | 4,000 | `0` | no side | — |

How the crowd answered: 12,000 behind true, 8,000 behind false. False holds 40 % of the stake that took a side. F only weighs the link.

How the crowd expected to answer: take what each neuron expects on false and weigh it by its stake. C's 7,000 expect 5 %, A's 5,000 expect 14 %, and so on down the last column. Together the crowd expected 15 % on false.

False came out 25 points more common than expected. It wins, with less money, and the link is damped.

| neuron | pays or receives | premium | karma |
|---|---|---|---|
| A | + 186 | + 19 | up |
| B | + 112 | + 11 | up |
| C | − 174 | 0 | down |
| D | − 74 | 0 | down |
| E | − 50 | 0 | down |
| F | 0 | 0 | — |

Why 298 and not 12,000: the losing side pays a share of its stake each epoch, in proportion to how surprising the winner was. One epoch is one count, and one count can be wrong, so it moves a part and leaves the rest standing. A side that stays wrong pays again next epoch, and the one after.

The size is scored on its own as well. B expected 40 % on false and exactly 40 % stood there; A expected 14 %. Both are on the winning side, and B also read the crowd right, which the score pays beside the side ([truth scoring](../../specs/truth-scoring.md)). That is why a neuron names the dissent it really expects: a size chosen to look informed is a wrong prediction.

Now a ring that knows nothing stakes 1,000 on false against 19,000 on true. The majority had expected about a tenth to disagree, and the ring is a twentieth. It comes out less common than expected, and it pays. A small group cannot win by merely being small: it has to be larger than the crowd thought possible.

Move the activations and watch each neuron's line:

<div id="truth-epoch"></div>
<style>#truth-epoch{border:1px solid rgba(128,128,128,.4);border-radius:10px;padding:16px 18px;margin:18px 0;max-width:720px}#truth-epoch label{display:block;margin:0 0 12px;font-size:.95em}#truth-epoch label span{float:right;font-variant-numeric:tabular-nums}#truth-epoch input[type=range]{display:block;width:100%;margin-top:4px}#truth-epoch .bar{display:grid;grid-template-columns:15em 1fr 3.5em;gap:10px;align-items:center;margin:7px 0;font-size:.9em}#truth-epoch .bar i{display:block;height:12px;border-radius:6px;background:rgba(128,128,128,.22);overflow:hidden}#truth-epoch .bar em{display:block;height:100%}#truth-epoch .bar span:last-child{text-align:right;font-variant-numeric:tabular-nums}#truth-epoch p{margin:10px 0 0}#truth-epoch .v{font-size:1.05em}#truth-epoch .n{font-variant-numeric:tabular-nums;border-bottom:2px solid currentColor}#truth-epoch .note{font-size:.85em;opacity:.7}#truth-epoch h4{margin:14px 0 8px;font-size:.95em;font-weight:600}@media(max-width:560px){#truth-epoch .bar{grid-template-columns:1fr 3.5em}#truth-epoch .bar i{grid-column:1/3;grid-row:2}}#truth-epoch .rows label{display:grid;grid-template-columns:9em 1fr;gap:2px 12px;align-items:center;margin:0 0 10px}#truth-epoch .rows label span{float:none}#truth-epoch .rows label .who{font-weight:600}#truth-epoch .rows label input{grid-column:1/3;margin:0}#truth-epoch table{border-collapse:collapse;margin-top:12px;font-size:.9em;width:auto}#truth-epoch td,#truth-epoch th{padding:3px 22px 3px 0;text-align:left;border:0;font-variant-numeric:tabular-nums}</style>
<script>(function(){ var r=document.getElementById('truth-epoch'); if(!r)return; var N=[['A',5000,-110],['B',3000,-77],['C',7000,6],['D',3000,26],['E',2000,13],['F',4000,0]]; function g(i){return document.getElementById(i);} function n(x){return Math.round(x).toLocaleString('en-US');} function pc(x){return Math.round(x*100)+' %';} function sg(x){return (x>0?'+ ':x<0?'− ':'')+n(Math.abs(x));} function rd(v){if(v===0)return '0 · no side';var m=Math.abs(v);return (v>0?'+':'−')+m+' · '+(v>0?'true':'false')+', '+pc(m/128)+' against me';} function bar(l,v,c){return '<div class="bar"><span>'+l+'</span><i><em style="width:'+Math.round(v*100)+'%;background:'+c+'"></em></i><span>'+pc(v)+'</span></div>';} var h='<div class="rows">'; N.forEach(function(x,i){h+='<label><span class="who">'+x[0]+' · stake '+n(x[1])+'</span><span id="te-r'+i+'"></span><input id="te-v'+i+'" type="range" min="-127" max="127" step="1" value="'+x[2]+'"></label>';}); r.innerHTML=h+'</div><div id="te-out"></div>'; function u(){ var T=0,F=0,y=0,V=[]; N.forEach(function(x,i){var v=+g('te-v'+i).value;V.push(v);g('te-r'+i).textContent=rd(v);var m=Math.abs(v)/128;if(v>0){T+=x[1];y+=x[1]*m;}else if(v<0){F+=x[1];y+=x[1]*(1-m);}}); var o=g('te-out'),t=T+F; if(t===0){o.innerHTML='<p class="v">Nobody takes a side: the link has weight and no verdict.</p>';return;} var xf=F/t,yf=y/t,gap=xf-yf; var out='<h4>false</h4>'+bar('how the crowd answered',xf,'#f85149')+bar('how it expected to answer',yf,'#d29922')+'<h4>true</h4>'+bar('how the crowd answered',1-xf,'#3fb950')+bar('how it expected to answer',1-yf,'#d29922'); var w=gap>0,L=w?T:F,W=w?F:T,pay=0.1*Math.abs(gap)*L,v; if(Math.abs(gap)<0.03){pay=0;v='<p class="v">The crowd answered as it expected to. No surprise: nobody pays, nobody earns.</p>';} else if(L===0||W===0){pay=0;v='<p class="v">Only one side holds stake, so there is nobody to pay and nobody to be paid.</p>';} else{v='<p class="v">'+(w?'False':'True')+' came out '+Math.round(Math.abs(gap)*100)+' points more common than expected: the surprisingly popular side'+(W<L?', though it holds less money':'')+'. The '+(w?'true':'false')+' side pays <span class="n">'+n(pay)+'</span>.</p>';} var tb='<table><tr><th>neuron</th><th>pays or receives</th><th>premium</th></tr>'; N.forEach(function(x,i){var s=V[i]>0,d=0,p=0;if(pay>0&&V[i]!==0){if(s===w){d=-pay*x[1]/L;}else{d=pay*x[1]/W;p=0.1*d;}}tb+='<tr><td>'+x[0]+'</td><td>'+(d===0?'0':sg(d))+'</td><td>'+(p===0?'0':sg(p))+'</td></tr>';}); o.innerHTML=out+v+tb+'</table><p class="note">The rates are invented to show the shape: the losing side pays a tenth of its stake times the surprise, and the premium is a tenth of that.</p>'; } N.forEach(function(x,i){g('te-v'+i).addEventListener('input',u);}); u(); })();</script>

When the crowd answers as it expected to, nobody is surprised, and joining a view everyone already expects pays nothing. Set every activation next to zero: each side then expects nearly everyone to agree with it, the expectation equals the count, and nothing is paid.

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
| takes the side it believes and names the dissent it really expects | the highest expected score when others do the same: Prelec's theorem |
| copies the visible majority | the majority is already expected: no surprise, nothing earned, and the stake is at risk if the majority is wrong |
| takes the unpopular side with no knowledge | it is as few as the crowd expected, or fewer: it pays |
| names a large dissent to look informed | the size is a prediction checked against the count: its side must then come out as small as it said, and a misprediction costs it |
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
