#!/usr/bin/env python3
"""fetch cyberlinks from a tendermint node's tx index via tx_search.

each MsgCyberlink tx emits one `cyberlink` event per link with
particleFrom/particleTo attributes (base64). we paginate
"cyberlink.neuron EXISTS" and record (height, particle_from, particle_to).

usage: fetch_links.py <rpc_url> <out_jsonl>
"""

import base64
import json
import sys
import time
import urllib.parse
import urllib.request

RPC = sys.argv[1]
OUT = sys.argv[2]
PER_PAGE = 100

def get(url):
    for attempt in range(5):
        try:
            with urllib.request.urlopen(url, timeout=60) as r:
                return json.load(r)
        except Exception as e:
            print(f"  retry {attempt}: {e}", file=sys.stderr)
            time.sleep(2 * (attempt + 1))
    raise RuntimeError("giving up: " + url)

q = urllib.parse.quote('"cyberlink.neuron EXISTS"')
probe = f"{RPC}/tx_search?query={q}&per_page=1&page=1"
total = int(get(probe)["result"]["total_count"])
pages = (total + PER_PAGE - 1) // PER_PAGE
print(f"{total} txs, {pages} pages", file=sys.stderr)

n_links = 0
with open(OUT, "w") as out:
    for page in range(1, pages + 1):
        url = f"{RPC}/tx_search?query={q}&per_page={PER_PAGE}&page={page}"
        res = get(url).get("result", {})
        for tx in res.get("txs", []):
            height = int(tx["height"])
            for ev in tx.get("tx_result", {}).get("events", []):
                if ev.get("type") != "cyberlink":
                    continue
                attrs = {}
                for a in ev.get("attributes", []):
                    k = base64.b64decode(a["key"]).decode()
                    v = base64.b64decode(a["value"]).decode()
                    attrs[k] = v
                if "particleFrom" in attrs and "particleTo" in attrs:
                    out.write(json.dumps({
                        "h": height,
                        "f": attrs["particleFrom"],
                        "t": attrs["particleTo"],
                    }) + "\n")
                    n_links += 1
        if page % 20 == 0:
            print(f"page {page}/{pages} · {n_links} links", file=sys.stderr)
print(f"done: {n_links} links -> {OUT}", file=sys.stderr)
