#!/usr/bin/env python3
"""Manifest-first RI population-band diagnostic, independent of production rules."""
import argparse
import hashlib
import json
from fractions import Fraction
from pathlib import Path

from analyze_dfs_county_response import ALPHAS, ROOT, digest
from run_multiroot_county_diagnostic import write_new

BANDS = (10, 100, 1000)
HYPOTHESIS = ROOT / "signals/discover/hypothesis/ri-population-band-hypothesis-2026-09-10.md"


def tree_intervals(graph, populations, root):
    n = len(populations)
    parent, order, stack = [-1] * n, [], [root]
    parent[root] = root
    while stack:
        u = stack.pop()
        order.append(u)
        for v in sorted((e["to"] for e in graph[u]), reverse=True):
            if parent[v] == -1:
                parent[v] = u
                stack.append(v)
    if len(order) != n:
        raise ValueError("disconnected graph")
    children = [[] for _ in populations]
    for u in range(n):
        if u != root:
            children[parent[u]].append(u)
    entry, leave, clock = [0] * n, [0] * n, 0
    stack = [(root, False)]
    while stack:
        u, leaving = stack.pop()
        if leaving:
            leave[u] = clock
        else:
            entry[u] = clock
            clock += 1
            stack.append((u, True))
            stack.extend((v, False) for v in reversed(children[u]))
    subtree = list(populations)
    for u in reversed(order):
        if u != root:
            subtree[parent[u]] += subtree[u]
    return entry, leave, subtree


def eligible(deviation, total, band):
    return deviation * 1000000 <= band * total


def evaluate(context, bands=BANDS):
    ids, populations = context["units"]["unit_ids"], context["populations"]
    graph, total = context["graph"]["adjacency"], sum(populations)
    n = len(ids)
    if n < 2 or total <= 0:
        raise ValueError("invalid population universe")
    roots = sorted({j * (n - 1) // 4 for j in range(5)})
    edges = {(u, e["to"]): int(e["weight"] if e.get("weight") is not None else 1)
             for u, neighbors in enumerate(graph) for e in neighbors if u < e["to"]}
    physical, counts = {}, []
    for root in roots:
        entry, leave, subtree = tree_intervals(graph, populations, root)
        admitted = 0
        for u in range(n):
            deviation = abs(2 * subtree[u] - total)
            if u == root or not eligible(deviation, total, max(bands)):
                continue
            admitted += 1
            labels = bytearray(entry[u] <= entry[v] < leave[u] for v in range(n))
            if labels[0]:
                labels = bytearray(1 - label for label in labels)
            key = hashlib.sha256(labels).hexdigest()
            if key in physical:
                continue
            raw = internal = 0
            for (v, w), weight in edges.items():
                if labels[v] != labels[w]:
                    raw += weight
                    if ids[v][:5] == ids[w][:5]:
                        internal += weight
            counties = {}
            for v, geoid in enumerate(ids):
                counties.setdefault(geoid[:5], set()).add(labels[v])
            physical[key] = {"deviation_numerator": deviation, "raw_cut": raw,
                             "within_county_cut": internal,
                             "split_counties": sum(len(labels) > 1 for labels in counties.values())}
        counts.append({"root_index": root, "eligible_at_largest_band": admitted})
    results = []
    for band in bands:
        choices = {key: row for key, row in physical.items()
                   if eligible(row["deviation_numerator"], total, band)}
        response = []
        for alpha in ALPHAS:
            scores = {key: r["raw_cut"] + Fraction(alpha) * r["within_county_cut"]
                      for key, r in choices.items()}
            minimum = min(scores.values()) if scores else None
            winners = sorted(key for key, score in scores.items() if score == minimum)
            response.append({"alpha": alpha, "minimum_weighted_cut": str(minimum), "winners": winners})
        results.append({"band_ppm": band, "eligible_physical_cuts": len(choices),
                        "status": "feasible-in-family" if choices else "empty-in-family",
                        "alpha_response": response,
                        "winning_set_changed": any(r["winners"] != response[0]["winners"] for r in response)})
    return {"total_population": total, "unit_count": n, "roots": counts,
            "candidates": physical, "bands": results,
            "hypothesis_survives": any(b["eligible_physical_cuts"] > 1 and b["winning_set_changed"] for b in results)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "run"))
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    sources = [Path(__file__), ROOT / "scripts/research/analyze_dfs_county_response.py",
               ROOT / "scripts/research/run_multiroot_county_diagnostic.py", HYPOTHESIS]
    settings = {"bands_ppm": list(BANDS), "alphas": list(ALPHAS), "seats": 2,
                "root_rule": "floor(j*(n-1)/4), j=0..4", "scope": "RI 2020 root only",
                "execution": "serial, no RNG, no enforced time or memory cap"}
    if args.action == "prepare":
        baseline = json.loads((ROOT / "docs/experiments/dfs-root-county-response-2020/analysis.json").read_text())
        write_new(args.manifest, {"schema": "ri-population-band-manifest-v1", "settings": settings,
                  "input": baseline["inputs"]["RI"],
                  "sources": {str(p.absolute()): digest(p) for p in sources}})
        return
    if args.out is None or args.out.exists():
        raise ValueError("requires unused --out path")
    manifest = json.loads(args.manifest.read_text())
    if manifest["schema"] != "ri-population-band-manifest-v1" or manifest["settings"] != settings:
        raise ValueError("unsupported manifest")
    if set(manifest["sources"]) != {str(p.absolute()) for p in sources}:
        raise ValueError("source set mismatch")
    for path, expected in manifest["sources"].items():
        if digest(Path(path)) != expected:
            raise ValueError("source hash mismatch")
    path = Path(manifest["input"]["path"])
    if digest(path) != manifest["input"]["sha256"]:
        raise ValueError("input hash mismatch")
    context = json.loads(path.read_text())
    if context["units"]["state"] != "RI" or context["units"]["year"] != 2020:
        raise ValueError("wrong State/year")
    report = evaluate(context)
    report["manifest_sha256"] = digest(args.manifest)
    write_new(args.out, report)
    print(json.dumps({"bands": report["bands"], "hypothesis_survives": report["hypothesis_survives"]}))


if __name__ == "__main__":
    main()
