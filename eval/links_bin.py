#!/usr/bin/env python3
"""Convert a snapshot `cyberlinks_indexed.csv.gz` into the compact binary the
epoch-settlement benchmark reads (`rs/examples/epoch_bench.rs`).

Output: little-endian u32 records `(from, to, neuron, height)`, one per CSV
row, in file order. Particles and neurons are numbered by first appearance.
A 16-byte header carries `b"TRUL"`, version 1, link count, particle count.

    python3 -I eval/links_bin.py ~/.cache/cyber/snapshots/pussy/cyberlinks_indexed.csv.gz /tmp/tru-epoch/pussy.links

stdlib only (no Node, no third-party packages).
"""

import csv
import gzip
import io
import struct
import sys


def main(src: str, dst: str) -> None:
    particles: dict[str, int] = {}
    neurons: dict[str, int] = {}
    out = bytearray()
    rows = 0
    with gzip.open(src, "rb") as raw:
        reader = csv.reader(io.TextIOWrapper(raw, encoding="utf-8"))
        header = next(reader)
        col = {name: i for i, name in enumerate(header)}
        fi, ti, ni, hi = (col[k] for k in ("particle_from", "particle_to", "neuron", "height"))
        for row in reader:
            f = particles.setdefault(row[fi], len(particles))
            t = particles.setdefault(row[ti], len(particles))
            n = neurons.setdefault(row[ni], len(neurons))
            out += struct.pack("<IIII", f, t, n, int(row[hi]))
            rows += 1
    with open(dst, "wb") as fh:
        fh.write(b"TRUL" + struct.pack("<III", 1, rows, len(particles)))
        fh.write(out)
    print(f"{dst}: {rows} links, {len(particles)} particles, {len(neurons)} neurons")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: links_bin.py <cyberlinks_indexed.csv.gz> <out.links>")
    main(sys.argv[1], sys.argv[2])
