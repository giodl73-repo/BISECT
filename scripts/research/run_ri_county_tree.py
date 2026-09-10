#!/usr/bin/env python3
"""Paired deterministic maximum-spanning-tree candidate diagnostic."""
import argparse
import hashlib
import json
from fractions import Fraction
from pathlib import Path

from analyze_dfs_county_response import ROOT, ALPHAS, digest
from run_multiroot_county_diagnostic import write_new
from run_ri_population_band import BANDS, eligible, tree_intervals


def edges_from(context):
    edges = {}
    for u, neighbors in enumerate(context["graph"]["adjacency"]):
        for edge in neighbors:
            v = edge["to"]
            weight = edge.get("weight")
            weight = 1 if weight is None else weight
            if not 0 <= v < len(context["populations"]) or weight <= 0 or int(weight) != weight:
                raise ValueError("invalid graph edge")
            if u < v:
                edges[u, v] = int(weight)
    return edges


def spanning_tree(ids, edges, county_first):
    parent = list(range(len(ids)))
    size = [1] * len(ids)
    def find(u):
        while parent[u] != u:
            parent[u] = parent[parent[u]]
            u = parent[u]
        return u
    ordered = sorted(edges, key=lambda pair: (
        int(county_first and ids[pair[0]][:5] != ids[pair[1]][:5]),
        -edges[pair], pair[0], pair[1]))
    graph = [[] for _ in ids]
    selected = []
    for u, v in ordered:
        a, b = find(u), find(v)
        if a == b:
            continue
        if size[a] < size[b]:
            a, b = b, a
        parent[b] = a
        size[a] += size[b]
        graph[u].append({"to": v, "weight": edges[u, v]})
        graph[v].append({"to": u, "weight": edges[u, v]})
        selected.append((u, v))
    if len(selected) != len(ids) - 1:
        raise ValueError("disconnected source graph")
    return graph, selected


def evaluate(context, county_first, bands=BANDS):
    ids, pops = context["units"]["unit_ids"], context["populations"]
    edges = edges_from(context)
    tree, selected = spanning_tree(ids, edges, county_first)
    entry, leave, subtree = tree_intervals(tree, pops, 0)
    total, candidates = sum(pops), {}
    for u in range(1, len(ids)):
        deviation = abs(2 * subtree[u] - total)
        if not eligible(deviation, total, max(bands)):
            continue
        labels = bytearray(entry[u] <= entry[v] < leave[u] for v in range(len(ids)))
        # Root zero is outside every proper subtree.
        raw = internal = 0
        for (v, w), weight in edges.items():
            if labels[v] != labels[w]:
                raw += weight
                if ids[v][:5] == ids[w][:5]:
                    internal += weight
        counties = {}
        for v, geoid in enumerate(ids):
            counties.setdefault(geoid[:5], set()).add(labels[v])
        candidates[hashlib.sha256(labels).hexdigest()] = {
            "deviation_numerator": deviation, "raw_cut": raw, "within_county_cut": internal,
            "split_counties": sum(len(values) > 1 for values in counties.values())}
    results = []
    for band in bands:
        choices = {key: r for key, r in candidates.items() if eligible(r["deviation_numerator"], total, band)}
        response = []
        for alpha in ALPHAS:
            costs = {key: r["raw_cut"] + Fraction(alpha) * r["within_county_cut"] for key, r in choices.items()}
            minimum = min(costs.values()) if costs else None
            response.append({"alpha": alpha, "minimum_weighted_cut": str(minimum) if minimum is not None else None,
                             "winners": sorted(key for key, cost in costs.items() if cost == minimum)})
        results.append({"band_ppm": band, "eligible_physical_cuts": len(choices),
                        "status": "feasible-in-family" if choices else "empty-in-family",
                        "alpha_response": response,
                        "winning_set_changed": any(r["winners"] != response[0]["winners"] for r in response)})
    return {"tree_edge_count": len(selected),
            "tree_edges_sha256": hashlib.sha256(json.dumps(sorted(selected), separators=(",", ":")).encode()).hexdigest(),
            "tree_cross_county_edges": sum(ids[u][:5] != ids[v][:5] for u, v in selected),
            "minimum_deviation_over_all_tree_cuts": min(abs(2 * subtree[u] - total) for u in range(1, len(ids))),
            "candidates": candidates, "bands": results}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "run"))
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    previous = ROOT / "docs/experiments/ri-population-band-2020/manifest.json"
    sources = [Path(__file__), ROOT / "scripts/research/analyze_dfs_county_response.py",
               ROOT / "scripts/research/run_multiroot_county_diagnostic.py",
               ROOT / "scripts/research/run_ri_population_band.py"]
    settings = {"bands_ppm": list(BANDS), "alphas": list(ALPHAS), "state": "RI", "year": 2020,
                "arms": ["geographic-maximum-spanning-tree", "county-first-maximum-spanning-tree"],
                "ordering": "cross-county-last in county arm only, descending weight, ascending endpoint indices",
                "scope": "one unrooted tree per arm, all tree edges; original graph scoring",
                "execution": "serial, deterministic, no enforced resource cap",
                "success_test": "county-first yields fewer split counties among winners at a common feasible band/alpha; report all ties"}
    if args.action == "prepare":
        write_new(args.manifest, {"schema": "ri-county-tree-v1", "settings": settings,
                  "input": json.loads(previous.read_text())["input"],
                  "sources": {str(p.absolute()): digest(p) for p in sources}})
        return
    if args.out is None or args.out.exists():
        raise ValueError("requires unused --out")
    manifest = json.loads(args.manifest.read_text())
    if manifest["schema"] != "ri-county-tree-v1" or manifest["settings"] != settings:
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
    arms = {name: evaluate(context, index == 1) for index, name in enumerate(settings["arms"])}
    write_new(args.out, {"manifest_sha256": digest(args.manifest), "arms": arms})
    for name, result in arms.items():
        print(name, "minimum deviation", result["minimum_deviation_over_all_tree_cuts"],
              "eligible counts", [b["eligible_physical_cuts"] for b in result["bands"]])


if __name__ == "__main__":
    main()
