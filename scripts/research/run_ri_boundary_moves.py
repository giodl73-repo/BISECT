#!/usr/bin/env python3
"""Bounded connected single-vertex descent from a frozen RI seed."""
import argparse
import hashlib
import json
from fractions import Fraction
from pathlib import Path

from analyze_dfs_county_response import ROOT, ALPHAS, digest
from run_multiroot_county_diagnostic import write_new
from run_ri_population_band import tree_intervals, eligible
from run_ri_county_tree import edges_from


def connected(adjacency, labels, label):
    units = {u for u, value in enumerate(labels) if value == label}
    if not units:
        return False
    start = min(units)
    seen, stack = {start}, [start]
    while stack:
        for v, _ in adjacency[stack.pop()]:
            if v in units and v not in seen:
                seen.add(v)
                stack.append(v)
    return len(seen) == len(units)


def metrics(ids, pops, edges, labels, alpha):
    raw = internal = 0
    for (u, v), weight in edges.items():
        if labels[u] != labels[v]:
            raw += weight
            if ids[u][:5] == ids[v][:5]:
                internal += weight
    counties = {}
    for u, geoid in enumerate(ids):
        counties.setdefault(geoid[:5], set()).add(labels[u])
    return {"raw_cut": raw, "within_county_cut": internal,
            "weighted_cut": str(raw + Fraction(alpha) * internal),
            "deviation_numerator": abs(2 * sum(p for p, label in zip(pops, labels) if label) - sum(pops)),
            "split_counties": sum(len(values) > 1 for values in counties.values())}


def improve(context, seed, alpha, move_budget=50, check_budget=5000, band=1000):
    ids, pops = context["units"]["unit_ids"], context["populations"]
    edges = edges_from(context)
    adjacency = [[] for _ in ids]
    for (u, v), weight in edges.items():
        adjacency[u].append((v, weight))
        adjacency[v].append((u, weight))
    labels = bytearray(seed)
    total = sum(pops)
    pop_one = sum(p for p, label in zip(pops, labels) if label)
    if not eligible(abs(2 * pop_one - total), total, band) or not all(connected(adjacency, labels, label) for label in (0, 1)):
        raise ValueError("seed violates population or connectivity")
    initial = metrics(ids, pops, edges, labels, alpha)
    a = Fraction(alpha)
    trace, checks, status = [], 0, "move-budget"
    for _ in range(move_budget):
        options = []
        for u, neighbors in enumerate(adjacency):
            if not any(labels[v] != labels[u] for v, _ in neighbors):
                continue
            proposed = pop_one + (pops[u] if labels[u] == 0 else -pops[u])
            if not eligible(abs(2 * proposed - total), total, band):
                continue
            # Integer cost is scaled by alpha's denominator; no rounding.
            delta = sum((1 if labels[v] == labels[u] else -1) * weight *
                        (a.denominator + (a.numerator if ids[u][:5] == ids[v][:5] else 0))
                        for v, weight in neighbors)
            if delta < 0:
                options.append((delta, u, proposed))
        accepted = False
        for delta, u, proposed in sorted(options):
            if checks >= check_budget:
                status = "connectivity-check-budget"
                break
            donor = labels[u]
            labels[u] = 1 - donor
            checks += 1
            # Recipient gains an adjacent vertex; explicitly check both anyway.
            if all(connected(adjacency, labels, label) for label in (0, 1)):
                pop_one = proposed
                trace.append({"unit_index": u, "geoid": ids[u], "new_label": labels[u],
                              "scaled_cost_delta": delta})
                accepted = True
                break
            labels[u] = donor
        if not accepted:
            if status != "connectivity-check-budget":
                status = "no-improving-feasible-single-vertex-move"
            break
    final = metrics(ids, pops, edges, labels, alpha)
    if not eligible(final["deviation_numerator"], total, band) or not all(connected(adjacency, labels, label) for label in (0, 1)):
        raise ValueError("final invariant violation")
    if Fraction(final["weighted_cut"]) - Fraction(initial["weighted_cut"]) != Fraction(sum(t["scaled_cost_delta"] for t in trace), a.denominator):
        raise ValueError("incremental/full score disagreement")
    return {"alpha": alpha, "initial": initial, "final": final, "status": status,
            "moves": len(trace), "connectivity_checks": checks, "trace": trace,
            "assignment_sha256": hashlib.sha256(labels).hexdigest(),
            "assignment": list(labels), "final_connected": True}


def recover_seed(context, wanted):
    n = len(context["populations"])
    total = sum(context["populations"])
    for root in sorted({j * (n - 1) // 4 for j in range(5)}):
        entry, leave, subtree = tree_intervals(context["graph"]["adjacency"], context["populations"], root)
        for u in range(n):
            if u == root or not eligible(abs(2 * subtree[u] - total), total, 1000):
                continue
            labels = bytearray(entry[u] <= entry[v] < leave[u] for v in range(n))
            if labels[0]:
                labels = bytearray(1 - label for label in labels)
            if hashlib.sha256(labels).hexdigest() == wanted:
                return labels
    raise ValueError("seed identity not recovered")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "run"))
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    prior = ROOT / "docs/experiments/ri-population-band-2020/analysis.json"
    sources = [Path(__file__)] + [ROOT / f"scripts/research/{name}.py" for name in
        ("analyze_dfs_county_response", "run_multiroot_county_diagnostic", "run_ri_population_band", "run_ri_county_tree")]
    settings = {"alphas": list(ALPHAS), "band_ppm": 1000, "move_budget": 50,
                "connectivity_candidate_check_budget": 5000,
                "ordering": "best strictly negative scaled cost delta, then ascending unit index",
                "execution": "serial; no RNG; bounded moves/checks, no hard wall-time or memory cap",
                "scope": "RI two labels; original graph connectivity includes bridges"}
    if args.action == "prepare":
        previous = json.loads(prior.read_text())
        winners = previous["bands"][-1]["alpha_response"][0]["winners"]
        if len(winners) != 1:
            raise ValueError("requires unique frozen seed")
        write_new(args.manifest, {"schema": "ri-boundary-moves-v1", "settings": settings,
                  "seed_sha256": winners[0], "prior_sha256": digest(prior),
                  "input": json.loads((prior.parent / "manifest.json").read_text())["input"],
                  "sources": {str(p.absolute()): digest(p) for p in sources}})
        return
    if args.out is None or args.out.exists():
        raise ValueError("requires unused output")
    manifest = json.loads(args.manifest.read_text())
    if manifest["schema"] != "ri-boundary-moves-v1" or manifest["settings"] != settings or digest(prior) != manifest["prior_sha256"]:
        raise ValueError("manifest/prior mismatch")
    if set(manifest["sources"]) != {str(p.absolute()) for p in sources}:
        raise ValueError("source set mismatch")
    for path, expected in manifest["sources"].items():
        if digest(Path(path)) != expected:
            raise ValueError("source hash mismatch")
    path = Path(manifest["input"]["path"])
    if digest(path) != manifest["input"]["sha256"]:
        raise ValueError("input hash mismatch")
    context = json.loads(path.read_text())
    seed = recover_seed(context, manifest["seed_sha256"])
    results = []
    for alpha in ALPHAS:
        result = improve(context, seed, alpha)
        results.append(result)
        print(alpha, result["moves"], result["status"], json.dumps(result["final"]), flush=True)
    write_new(args.out, {"manifest_sha256": digest(args.manifest), "results": results})


if __name__ == "__main__":
    main()
