---
tags: cyber, tru, docs
alias: truth explained, staking on truth, a stake on truth, how truth works, truth from claim to payout
crystal-type: article
crystal-domain: cyber
---
# truth, from claim to payout

A prediction market can tell you who wins an election, because one day the election happens. It cannot tell you whether a paper answers a question, because that day never comes.

[[cyber]] is built for the second kind of knowledge. It asks a crowd two things at once: what do you think, and what will the others think. Then it pays the side that turned out more common than the crowd itself expected. No outside judge is needed, and no question ever closes.

---

## the machine you already know

| | a prediction market | cyber |
|---|---|---|
| the question | written by an organizer | a link, made by any [[neuron]] |
| the stake | on an outcome of an event | on the link: true or false |
| the judge | the world, once | the crowd's own surprise, every epoch |
| who pays | losers pay winners | losers pay winners, and the network adds a premium |
| the end | the market closes | the link stays open: a later stake can overturn it |

---

## a link is a claim

A [[cyberlink]] `p → q` joins two [[files]] and says: from `p`, look at `q`. If `p` is a question and `q` is a text, the link is an answer. Anyone can make one. Nobody is appointed to ask or to answer.

A link alone is free speech and weighs nothing. To make it count, a neuron locks a stake on it and sets one number.

---

## the activation: one byte, two answers

| | the question | the answer |
|---|---|---|
| the sign | what do I think of this link? | plus, it is true; minus, it is false |
| the size | how many will think otherwise? | 1 to 127, a share of 128 of the stake on this link |
| zero | none | I take no side: my stake weighs the link, risks nothing, earns nothing |

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

Read the size as loudness. Near zero you say what everyone says. Far from zero you say: I am right, and almost nobody knows it yet. Loud is free to claim and costly to get wrong, because the size is a forecast and it gets checked.

Add up everyone's byte on one link and you get three numbers where a vote gives one.

| number | made from | used for |
|---|---|---|
| how the crowd answered | stake behind plus against stake behind minus | the weight of the link |
| how it expected to answer | the sizes, weighed by stake | the yardstick |
| the surprise | the first minus the second | who wins |

A link the crowd calls false gets lighter. Nothing flows backward.

---

## how one epoch runs

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

Neurons stake whenever they like. Once an epoch, about every two hours, every node does the same sum. It weighs each link by its two sides and lets attention flow along the links until it settles. That resting pattern is [[focus]]: what the network holds true right now. It is unique and every node gets the same one, which is the [[collective focus theorem]].

Focus alone would pay for agreement, and a crowd can agree on a mistake. So the node runs one more count on every link, moves the money, and opens everything again.

---

## the count that needs no judge

Ask a crowd whether Philadelphia is the capital of Pennsylvania. Most say yes, and they are wrong. Now ask each one: how many of the others will say yes?

Those who say yes expect nearly everyone to agree. Those who say no know the real capital, and they also know the mistake is common, so they expect to be few.

Count. "No" comes out more common than the crowd expected. That surprise marks the people who know. Dražen Prelec proved it in 2004 as the [[Bayesian Truth Serum]]. It works because knowledge is lopsided: whoever knows the fact also knows the popular error, and whoever holds the error has never heard of the fact.

Here is that crowd on one link, capital of Pennsylvania → Philadelphia. Every row says what that neuron expects on false. Weigh those by stake and you have what the crowd expected. The surprise is one subtraction:

surprise = what happened − what was expected

In the bar under the rows the red is what happened, the yellow mark is what was expected, and the striped piece between them is the surprise. Drag any activation and watch the striped piece, and who pays.

<div id="truth-epoch"></div>
<style>#truth-epoch{border:1px solid rgba(128,128,128,.4);border-radius:10px;padding:14px 18px;margin:18px 0}#truth-epoch .row{display:grid;grid-template-columns:6.5em minmax(120px,1fr) 3em 12.5em 6em 12.5em;gap:12px;align-items:center;padding:5px 0;font-size:.92em}#truth-epoch .head span{font-size:.85em;opacity:.65}#truth-epoch .who{font-weight:600}#truth-epoch .num{font-variant-numeric:tabular-nums}#truth-epoch .ex{color:#d29922}#truth-epoch .up{color:#3fb950}#truth-epoch .dn{color:#f85149}#truth-epoch input[type=range]{-webkit-appearance:none;appearance:none;width:100%;height:14px;margin:0;border-radius:7px;outline:none;background:linear-gradient(90deg,#f85149 0%,rgba(248,81,73,.12) 49.6%,rgba(160,160,160,.9) 49.6%,rgba(160,160,160,.9) 50.4%,rgba(63,185,80,.12) 50.4%,#3fb950 100%)}#truth-epoch input[type=range]::-webkit-slider-thumb{-webkit-appearance:none;appearance:none;width:18px;height:18px;border-radius:50%;background:#fff;border:2px solid #24292f;cursor:pointer}#truth-epoch input[type=range]::-moz-range-thumb{width:14px;height:14px;border-radius:50%;background:#fff;border:2px solid #24292f;cursor:pointer}#truth-epoch input[type=range]::-moz-range-track{background:transparent}#truth-epoch #te-out{margin-top:10px;border-top:1px solid rgba(128,128,128,.25);padding-top:14px}#truth-epoch .track{position:relative;height:22px;border-radius:6px;background:rgba(128,128,128,.22)}#truth-epoch .track em{position:absolute;top:0;bottom:0;display:block}#truth-epoch .track .ans{left:0;background:#f85149;border-radius:6px 0 0 6px;opacity:.85}#truth-epoch .track .gap{background:repeating-linear-gradient(135deg,#fff 0,#fff 4px,rgba(255,255,255,.25) 4px,rgba(255,255,255,.25) 8px);outline:2px solid #fff}#truth-epoch .track b{position:absolute;top:-5px;bottom:-5px;width:3px;margin-left:-1px;background:#d29922;border-radius:2px}#truth-epoch .lg{display:flex;flex-wrap:wrap;gap:4px 22px;margin-top:9px;font-size:.9em}#truth-epoch .lg .a{color:#f85149}#truth-epoch .lg .e{color:#d29922}#truth-epoch .lg .s{font-weight:600}#truth-epoch p{margin:10px 0 0}#truth-epoch .v{font-size:1.05em}#truth-epoch .note{font-size:.82em;opacity:.65}@media(max-width:860px){#truth-epoch .row{grid-template-columns:5.5em 1fr 3em}#truth-epoch .row span:nth-child(n+4){grid-column:1/4;padding-left:5.5em;margin-top:-6px}#truth-epoch .head span:nth-child(n+3){display:none}}</style>
<script>(function(){ var r=document.getElementById('truth-epoch'); if(!r)return; var N=[['A',5000,-110],['B',3000,-77],['C',7000,6],['D',3000,26],['E',2000,13],['F',4000,0]]; function g(i){return document.getElementById(i);} function n(x){return Math.round(x).toLocaleString('en-US');} function pc(x){return Math.round(x*100)+' %';} function sg(x){return (x>0?'+':'−')+n(Math.abs(x));} function rd(v){if(v===0)return 'no side';return (v>0?'true':'false')+' · '+pc(Math.abs(v)/128)+' will disagree';} var h='<div class="row head"><span>neuron · stake</span><span>activation: false ← 0 → true</span><span></span><span>reads as</span><span>so expects on false</span><span>pays or receives</span></div>'; N.forEach(function(x,i){h+='<div class="row"><span class="who">'+x[0]+' · '+n(x[1])+'</span><input id="te-v'+i+'" type="range" min="-127" max="127" step="1" value="'+x[2]+'" aria-label="activation of '+x[0]+'"><span class="num" id="te-n'+i+'"></span><span id="te-r'+i+'"></span><span class="num ex" id="te-e'+i+'"></span><span class="num" id="te-p'+i+'"></span></div>';}); r.innerHTML=h+'<div id="te-out"></div>'; function u(){ var T=0,F=0,y=0,V=[]; N.forEach(function(x,i){var v=+g('te-v'+i).value;V.push(v);g('te-n'+i).textContent=(v>0?'+':v<0?'−':'')+Math.abs(v);g('te-r'+i).textContent=rd(v);var m=Math.abs(v)/128,e=v>0?m:1-m;g('te-e'+i).textContent=v===0?'—':pc(e);if(v>0){T+=x[1];y+=x[1]*m;}else if(v<0){F+=x[1];y+=x[1]*(1-m);}}); var o=g('te-out'),t=T+F,pay=0,w=false,L=0,W=0,v; if(t===0){v='<p class="v">Nobody takes a side: the link has weight and no verdict.</p>';} else{ var xf=F/t,yf=y/t,gap=xf-yf,lo=Math.min(xf,yf),pts=Math.round(Math.abs(gap)*100);w=gap>0;L=w?T:F;W=w?F:T; v='<div class="sp"><div class="track"><em class="ans" style="width:'+(xf*100)+'%"></em><em class="gap" style="left:'+(lo*100)+'%;width:'+(Math.abs(gap)*100)+'%"></em><b style="left:'+(yf*100)+'%"></b></div><div class="lg"><span class="a">happened: '+pc(xf)+' of the stake said false</span><span class="e">expected: '+pc(yf)+', the last column weighed by stake</span><span class="s">surprise: '+pc(xf)+' − '+pc(yf)+' = '+(gap>=0?'+':'−')+pts+' points for false</span></div></div>'; if(Math.abs(gap)<0.03){v+='<p class="v">The crowd answered as it expected to. No surprise, so nobody pays and nobody earns.</p>';} else if(L===0||W===0){v+='<p class="v">Only one side holds stake: nobody to pay, nobody to be paid.</p>';} else{pay=0.1*Math.abs(gap)*L;v+='<p class="v">'+(w?'False got more than the crowd expected':'False got less than the crowd expected, so true got more')+(W<L?', with less money behind it':'')+'. The surprising side wins: '+(w?'true':'false')+' pays '+n(pay)+'.</p>';} } N.forEach(function(x,i){var c=g('te-p'+i),d=0,p=0;if(pay>0&&V[i]!==0){if((V[i]>0)===w){d=-pay*x[1]/L;}else{d=pay*x[1]/W;p=0.1*d;}}c.textContent=d===0?'0':sg(d)+(p>0?' and '+sg(p)+' premium':'');c.className='num '+(d>0?'up':d<0?'dn':'');}); o.innerHTML=v+'<p class="note">The rates are invented to show the shape: the losing side pays a tenth of its stake times the surprise, and the network adds a tenth of that as premium.</p>'; } N.forEach(function(x,i){g('te-v'+i).addEventListener('input',u);}); u(); })();</script>

What the opening position shows:

- More money says true, and false still wins. The crowd expected 15 % on false and 40 % stood there: a surprise of 25 points. The link gets lighter.
- The losers pay 298, and their stake is 12,000. One epoch is one count, and one count can be wrong, so it moves a slice in proportion to the surprise. Stay wrong and you pay again next epoch.
- F takes no side, pays nothing and earns nothing.

Three things to try:

- Drag every activation next to zero. Each side now expects everyone to agree with it, the yellow mark lands on the end of the red, the striped piece is gone, and nothing is paid. Agreement that everyone expected is worth nothing.
- Drag C to the far right. One loud voice on the big side expects a great deal of dissent, the crowd's expectation of false rises above the count, and the result flips: now true is the surprise.
- Move B from the false side to the true side. The surprise shrinks with it: a small group wins only while it is larger than the crowd thought possible. A gang of a twentieth facing a crowd that expected a tenth of dissent pays.

The toy scores only the side. The full score also pays the forecast: B expected two fifths on false and exactly two fifths stood there, so B read the crowd better than A. That is why a neuron names the dissent it really expects.

---

## what settlement moves

| | from | to | for |
|---|---|---|---|
| redistribution | the losing side | the winning side | the surprise; it sums to zero |
| premium | new tokens | the winners | the knowledge their stake added |
| annuity | new tokens | links that keep carrying focus | every later epoch they still matter |
| [[karma]] | nobody | the neuron | its running score; it cannot be bought |

Being right pays twice, from the losers and from the network. Being wrong costs once.

---

## why honesty wins

| if a neuron | then |
|---|---|
| says what it believes and names the dissent it expects | it has the best expected score when others do the same |
| copies the visible majority | the majority was expected: no surprise, nothing earned |
| takes the unpopular side knowing nothing | it is as few as expected, or fewer: it pays |
| claims great dissent to look informed | its forecast misses, and the miss costs it |
| stakes both sides | it pays itself |
| splits its stake across many keys | same total: every rule is linear in stake |
| joins a ring that agrees on a lie | every member expects the ring's answer, so there is no surprise |

The count rewards honesty and stays silent on correctness. A neuron that honestly holds an error loses to those who know better, and that loss is how the error gets fixed.

The chain reads no news. The world enters through neurons: someone who saw the event or read the source stakes on what it knows. A fact nobody stakes on is a fact the network does not hold.

---

## how far to trust it

| claim | standing |
|---|---|
| focus has one value and every node computes the same | proven, and rechecked by every node each epoch |
| honesty is the best reply when others are honest | proven by Prelec for the serum's report |
| the surprising side is the true one | proven when the stakers' evidence is independent; weaker when it all comes from one source |
| a small group that shares one error and expects to be few | looks like knowledge for one epoch; the open link corrects it over the next ones |
| an informed minority beats a coordinated majority | measured in the scorer with up to 200 agents |
| most of the stake agrees to lie | it wins, as most of the work wins on any chain |
| the network converges on what is the case | the hypothesis the network exists to test, in public, under real stake |

The proofs and their conditions are in [strong truthfulness](../../specs/strong-truthfulness.md); the measurement is the [coordinated consensus receipt](../../audit/coordinated-consensus-scoring.md).

---

## the four questions

- What is truth? The side of a link that keeps coming out more common than the crowd expected. It is held, epoch after epoch, and it can be overturned.
- Who asks and who answers? Any neuron. A link is a question answered, and a stake on it says true or false.
- Why stake? Being right is paid by those who were wrong and by the network, and it builds karma. Idle tokens earn nothing.
- Who checks? The crowd checks itself. Every staker's forecast of the others is the yardstick, every node repeats the count, and anyone may stake on either side of any link at any time.

---

## where it stands

The mechanism is the law of the [rewards specification](../../specs/rewards.md): honesty is its section 5, the two kinds of stake its section 9, the annuity its sections 11 and 12. The serum's score is defined in [truth scoring](../../specs/truth-scoring.md).

The record is still being chosen. The rehearsal network carries a three-valued valence today: plus one, zero, minus one. That is the side without the size, so its count has no surprise yet. The activation described here is the leading of two candidates in the project [activation](https://github.com/cyberia-to/soft3/blob/release/phase1/projects/activation.md). Focus and the focus pool already run in consensus; the project [the truth market at genesis](../../projects/truth-market-at-genesis.md) joins the score, the payments and the annuity to the chain before launch, and fixes the rates this page leaves in words.
