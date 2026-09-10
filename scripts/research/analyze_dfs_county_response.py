#!/usr/bin/env python3
"""Read-only root-cut diagnostic; not a complete-plan county-weight bakeoff."""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
from fractions import Fraction
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ALPHAS = ("0", "0.5", "1", "2", "4", "8")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def analyze(context, seats):
    ids = context["units"]["unit_ids"]
    populations = context["populations"]
    adjacency = context["graph"]["adjacency"]
    n = len(ids)
    if seats < 2 or n < 2 or len(populations) != n or len(adjacency) != n:
        raise ValueError("invalid dimensions or seat count")
    if ids != sorted(set(ids)) or any(len(x) != 15 or not x.isdigit() for x in ids):
        raise ValueError("requires unique sorted Census block GEOIDs")
    if any(type(p) is not int or p < 0 for p in populations):
        raise ValueError("invalid populations")
    parent = [-1] * n
    parent[0] = 0
    stack, order = [0], []
    edges = {}
    for u, neighbors in enumerate(adjacency):
        for edge in neighbors:
            v = edge["to"]
            if type(v) is not int or not 0 <= v < n:
                raise ValueError("edge target out of range")
            weight = edge.get("weight")
            weight = 1 if weight is None else weight
            if weight <= 0 or weight != int(weight) or weight >= 2**64:
                raise ValueError("requires positive integer u64 weights")
            if u < v:
                edges[u, v] = int(weight)
    # Mirror exact_cmd.rs: mark parents on push, not on pop.
    while stack:
        u = stack.pop()
        order.append(u)
        for v in sorted((e["to"] for e in adjacency[u]), reverse=True):
            if parent[v] == -1:
                parent[v] = u
                stack.append(v)
    if len(order) != n:
        raise ValueError("disconnected graph")
    children = [[] for _ in ids]
    for u in range(1, n):
        children[parent[u]].append(u)
    entry, leave = [0] * n, [0] * n
    stack, clock = [(0, False)], 0
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
        if u:
            subtree[parent[u]] += subtree[u]
    target = (seats // 2) * subtree[0]
    best, candidates = None, []
    for u in range(1, n):
        for complement in (False, True):
            pop = subtree[0] - subtree[u] if complement else subtree[u]
            deviation = abs(seats * pop - target)
            if best is None or deviation < best:
                best, candidates = deviation, []
            if deviation == best:
                candidates.append((u, complement))
    # Every proper subtree excludes root 0. Distinct subtree roots therefore
    # identify distinct unoriented physical partitions (also with zero-pop units).
    physical = sorted({u for u, _ in candidates})
    scores = []
    for u in physical:
        inside = [entry[u] <= entry[v] < leave[u] for v in range(n)]
        raw = internal = 0
        for (v, w), weight in edges.items():
            if inside[v] != inside[w]:
                raw += weight
                if ids[v][:5] == ids[w][:5]:
                    internal += weight
        scores.append({"subtree_root_geoid": ids[u], "raw_cut": raw,
                       "within_county_cut": internal})
    sweep = []
    for alpha in ALPHAS:
        a = Fraction(alpha)
        values = [Fraction(s["raw_cut"]) + a * s["within_county_cut"] for s in scores]
        minimum = min(values)
        winners = [s["subtree_root_geoid"] for s, value in zip(scores, values) if value == minimum]
        sweep.append({"alpha": alpha, "minimum_weighted_cut": str(minimum),
                      "winning_physical_partitions": winners})
    baseline = set(sweep[0]["winning_physical_partitions"])
    return {"unit_count": n, "seats": seats, "minimum_deviation": best,
            "minimum_deviation_candidates": len(candidates),
            "minimum_deviation_physical_partitions": len(physical),
            "minimum_deviation_cut_candidates": sum(ids[u] in baseline for u, _ in candidates),
            "minimum_deviation_cut_partitions": len(baseline),
            "candidate_scores": scores, "alpha_response": sweep,
            "winning_set_changed": any(set(s["winning_physical_partitions"]) != baseline for s in sweep)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contexts", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    if args.out.exists():
        raise FileExistsError(args.out)
    census = ROOT / "docs/experiments/nrs-v0.3-complete-tree-dfs-census-2020/node-results.csv"
    with census.open(newline="", encoding="utf-8") as handle:
        roots = {r["state"].upper(): r for r in csv.DictReader(handle) if r["path"] == "root"}
    results, inputs = {}, {}
    for state in ("RI", "IA", "NC", "TX"):
        path = args.contexts / f"{state.lower()}_blocks_2020.rctx"
        inputs[state] = {"path": str(path.absolute()), "sha256": digest(path)}
        context = json.loads(path.read_text(encoding="utf-8"))
        if context["units"]["state"] != state or context["units"]["year"] != 2020:
            raise ValueError("wrong context State/year")
        result = analyze(context, int(roots[state]["seats"]))
        for field in ("unit_count", "minimum_deviation_candidates",
                      "minimum_deviation_cut_candidates", "minimum_deviation_cut_partitions"):
            if result[field] != int(roots[state][field]):
                raise ValueError(f"{state}: baseline census mismatch: {field}")
        result["baseline_census_counters_match"] = True
        results[state] = result
        print(f"{state}: {result['minimum_deviation_physical_partitions']} physical candidates; "
              f"winning set changed={result['winning_set_changed']}", flush=True)
        del context
    report = {"schema_version": "dfs-root-county-response-v1", "inputs": inputs,
              "source_sha256": digest(Path(__file__)), "census_sha256": digest(census),
              "selector_source_sha256": digest(ROOT / "crates/bisect-cli/src/exact_cmd.rs"),
              "states": results,
              "claim_boundary": "Root-only fixed-DFS candidate rescoring, exact rational additive county penalties. "
              "Not a full engine sweep, METIS rerun, tie-break replay, final assignment comparison, "
              "county-fragment outcome, or optimality/neutrality/legal claim. Includes retained zero-population units and graph bridges."}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("x", encoding="utf-8", newline="\n") as handle:
        handle.write(json.dumps(report, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
