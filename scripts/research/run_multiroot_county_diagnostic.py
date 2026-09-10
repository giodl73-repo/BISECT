#!/usr/bin/env python3
"""Manifest-first, root-only DFS search diagnostic; not a full-plan bakeoff."""
import argparse
import hashlib
import json
import sys
import time
from fractions import Fraction
from pathlib import Path

from analyze_dfs_county_response import ALPHAS, ROOT, analyze, digest


def candidates(context, seats, root):
    ids, pops = context["units"]["unit_ids"], context["populations"]
    graph = context["graph"]["adjacency"]
    n = len(ids)
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
    children = [[] for _ in ids]
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
    population = list(pops)
    for u in reversed(order):
        if u != root:
            population[parent[u]] += population[u]
    total, best, selected = population[root], None, []
    for u in range(n):
        if u == root:
            continue
        deviation = min(abs(seats * p - (seats // 2) * total)
                        for p in (population[u], total - population[u]))
        if best is None or deviation < best:
            best, selected = deviation, []
        if deviation == best:
            selected.append(u)
    rows = []
    for u in selected:
        inside = bytearray(entry[u] <= entry[v] < leave[u] for v in range(n))
        # Canonical physical identity independent of tree root/orientation.
        if inside[0]:
            inside = bytearray(1 - value for value in inside)
        raw = internal = 0
        counties = {}
        for v, neighbors in enumerate(graph):
            counties.setdefault(ids[v][:5], set()).add(inside[v])
            for edge in neighbors:
                w = edge["to"]
                if v < w and inside[v] != inside[w]:
                    weight = edge.get("weight")
                    weight = 1 if weight is None else int(weight)
                    raw += weight
                    if ids[v][:5] == ids[w][:5]:
                        internal += weight
        rows.append({"partition_sha256": hashlib.sha256(inside).hexdigest(),
                     "minimum_deviation": best, "raw_cut": raw,
                     "within_county_cut": internal,
                     "root_split_counties": sum(len(labels) > 1 for labels in counties.values())})
    return rows


def score_pool(rows):
    best = min(r["minimum_deviation"] for r in rows)
    eligible = {r["partition_sha256"]: r for r in rows if r["minimum_deviation"] == best}
    response = []
    for alpha in ALPHAS:
        scores = {key: r["raw_cut"] + Fraction(alpha) * r["within_county_cut"]
                  for key, r in eligible.items()}
        minimum = min(scores.values())
        response.append({"alpha": alpha, "minimum_weighted_cut": str(minimum),
                         "winners": sorted(key for key, value in scores.items() if value == minimum)})
    return {"minimum_deviation": best, "physical_candidates_before_population_filter":
            len({r["partition_sha256"] for r in rows}),
            "physical_candidates_after_population_filter": len(eligible),
            "eligible_candidates": list(eligible.values()), "alpha_response": response,
            "winning_set_changed": any(r["winners"] != response[0]["winners"] for r in response)}


def write_new(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x", encoding="utf-8", newline="\n") as handle:
        handle.write(json.dumps(value, indent=2, sort_keys=True) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "run"))
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    sources = [Path(__file__), ROOT / "scripts/research/analyze_dfs_county_response.py"]
    baseline_path = ROOT / "docs/experiments/dfs-root-county-response-2020/analysis.json"
    if args.action == "prepare":
        baseline = json.loads(baseline_path.read_text())
        write_new(args.manifest, {"schema": "multiroot-county-diagnostic-manifest-v1",
                  "inputs": baseline["inputs"], "baseline_path": str(baseline_path),
                  "baseline_sha256": digest(baseline_path),
                  "sources": {str(p.absolute()): digest(p) for p in sources},
                  "states": ["RI", "IA", "NC", "TX"], "alphas": list(ALPHAS),
                  "root_rule": "sorted GEOID indices floor(j*(n-1)/4), j=0..4",
                  "population_rule": "minimum exact deviation across union of per-root minima",
                  "scope": "root-only diagnostic, not W1 full-plan experiment",
                  "execution": "serial; no RNG; no enforced memory cap; 1800-second cooperative deadline",
                  "python_version": sys.version})
        return
    if args.out is None or args.out.exists():
        raise ValueError("requires an unused --out path")
    manifest = json.loads(args.manifest.read_text())
    if (manifest["schema"] != "multiroot-county-diagnostic-manifest-v1"
            or manifest["states"] != ["RI", "IA", "NC", "TX"]
            or manifest["alphas"] != list(ALPHAS)
            or manifest["root_rule"] != "sorted GEOID indices floor(j*(n-1)/4), j=0..4"
            or manifest["population_rule"] != "minimum exact deviation across union of per-root minima"
            or manifest["execution"] != "serial; no RNG; no enforced memory cap; 1800-second cooperative deadline"
            or set(manifest["sources"]) != {str(p.absolute()) for p in sources}):
        raise ValueError("unsupported manifest settings")
    for path, expected in manifest["sources"].items():
        if digest(Path(path)) != expected:
            raise ValueError("source hash mismatch")
    if digest(baseline_path) != manifest["baseline_sha256"]:
        raise ValueError("baseline hash mismatch")
    baseline = json.loads(baseline_path.read_text())
    start, results = time.monotonic(), {}
    for state in manifest["states"]:
        path = Path(manifest["inputs"][state]["path"])
        if digest(path) != manifest["inputs"][state]["sha256"]:
            raise ValueError("context hash mismatch")
        context = json.loads(path.read_text())
        seats = baseline["states"][state]["seats"]
        control = analyze(context, seats)  # Validate input and reproduce frozen control.
        for key in ("candidate_scores", "minimum_deviation", "alpha_response"):
            if control[key] != baseline["states"][state][key]:
                raise ValueError("control replay mismatch")
        roots = sorted({j * (len(context["populations"]) - 1) // 4 for j in range(5)})
        rows, per_root = [], []
        for root in roots:
            if time.monotonic() - start > 1800:
                write_new(args.out, {"status": "timeout", "states": results,
                                    "manifest_sha256": digest(args.manifest)})
                return
            found = candidates(context, seats, root)
            per_root.append({"root_index": root, "root_geoid": context["units"]["unit_ids"][root],
                             "result": score_pool(found)})
            rows.extend(found)
        results[state] = {"control_replayed": True, "per_root": per_root,
                          "expanded": score_pool(rows)}
        print(state, json.dumps({k: v for k, v in results[state]["expanded"].items()
                                if k not in ("eligible_candidates", "alpha_response")}), flush=True)
        del context
    write_new(args.out, {"status": "complete", "manifest_sha256": digest(args.manifest),
                        "states": results, "claim_boundary": "Root-only fixed population-first search. "
                        "No final plans, final county fragments, legal compliance or optimality claim."})


if __name__ == "__main__":
    main()
