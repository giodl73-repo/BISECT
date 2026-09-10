#!/usr/bin/env python3
"""Paired adjacent-block swaps from frozen single-block stopping states."""
import argparse
import hashlib
import json
from fractions import Fraction
from pathlib import Path
from analyze_dfs_county_response import ROOT, digest
from run_multiroot_county_diagnostic import write_new
from run_ri_boundary_moves import connected, metrics
from run_ri_county_tree import edges_from
from run_ri_population_band import eligible

BASE = ROOT / "docs/experiments/ri-boundary-budget-2020"


def swap_delta(ids, adjacency, labels, u, v, alpha):
    a = Fraction(alpha)
    # The u-v edge stays cut; exclude it from both single-flip deltas.
    return sum((1 if labels[w] == labels[x] else -1) * weight *
               (a.denominator + (a.numerator if ids[x][:5] == ids[w][:5] else 0))
               for x, other in ((u, v), (v, u)) for w, weight in adjacency[x] if w != other)


def improve_swaps(context, seed, alpha, move_budget=50, check_budget=5000, band=1000):
    ids, pops = context["units"]["unit_ids"], context["populations"]
    edges = edges_from(context)
    adjacency = [[] for _ in ids]
    for (u, v), weight in edges.items():
        adjacency[u].append((v, weight))
        adjacency[v].append((u, weight))
    labels = bytearray(seed)
    total = sum(pops)
    if len(labels) != len(ids) or any(label not in (0, 1) for label in labels):
        raise ValueError("invalid seed labels")
    initial = metrics(ids, pops, edges, labels, alpha)
    if not eligible(initial["deviation_numerator"], total, band) or not all(connected(adjacency, labels, label) for label in (0, 1)):
        raise ValueError("infeasible seed")
    pop_one = sum(p for p, label in zip(pops, labels) if label)
    trace, checks, status = [], 0, "swap-budget"
    for _ in range(move_budget):
        options = []
        for u, v in edges:
            if labels[u] == labels[v]:
                continue
            proposed = pop_one + (pops[u] - pops[v] if labels[u] == 0 else pops[v] - pops[u])
            if not eligible(abs(2 * proposed - total), total, band):
                continue
            delta = swap_delta(ids, adjacency, labels, u, v, alpha)
            if delta < 0:
                options.append((delta, u, v, proposed))
        accepted = False
        for delta, u, v, proposed in sorted(options):
            if checks >= check_budget:
                status = "connectivity-check-budget"
                break
            labels[u], labels[v] = labels[v], labels[u]
            checks += 1
            if all(connected(adjacency, labels, label) for label in (0, 1)):
                pop_one = proposed
                trace.append({"units": [u, v], "geoids": [ids[u], ids[v]],
                              "scaled_cost_delta": delta})
                accepted = True
                break
            labels[u], labels[v] = labels[v], labels[u]
        if not accepted:
            if status != "connectivity-check-budget":
                status = "no-improving-feasible-adjacent-swap"
            break
    final = metrics(ids, pops, edges, labels, alpha)
    if (not eligible(final["deviation_numerator"], total, band)
            or not all(connected(adjacency, labels, label) for label in (0, 1))
            or Fraction(final["weighted_cut"]) - Fraction(initial["weighted_cut"]) !=
            Fraction(sum(t["scaled_cost_delta"] for t in trace), Fraction(alpha).denominator)):
        raise ValueError("final invariants or full/incremental score mismatch")
    return {"alpha": alpha, "initial": initial, "final": final, "swaps": len(trace),
            "status": status, "connectivity_checks": checks, "trace": trace,
            "assignment": list(labels), "assignment_sha256": hashlib.sha256(labels).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "run"))
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    base = json.loads((BASE / "manifest.json").read_text())
    settings = {"alphas": base["settings"]["alphas"], "band_ppm": 1000, "swap_budget": 50,
                "connectivity_check_budget": 5000, "ordering": "strict negative scaled delta, ascending endpoint indices",
                "move": "exchange labels of endpoints of an original cut edge; no single-block moves",
                "execution": "serial deterministic; no hard wall-time or memory cap"}
    sources = dict(base["sources"])
    sources[str(Path(__file__).absolute())] = digest(Path(__file__))
    if args.action == "prepare":
        write_new(args.manifest, {"schema": "ri-adjacent-swaps-v1", "settings": settings,
                  "input": base["input"], "sources": sources,
                  "baseline_manifest_sha256": digest(BASE / "manifest.json"),
                  "baseline_analysis_sha256": digest(BASE / "analysis.json")})
        return
    if args.out is None or args.out.exists():
        raise ValueError("requires unused output")
    manifest = json.loads(args.manifest.read_text())
    if (manifest["schema"] != "ri-adjacent-swaps-v1" or manifest["settings"] != settings
            or manifest["sources"] != sources or manifest["input"] != base["input"]
            or manifest["baseline_manifest_sha256"] != digest(BASE / "manifest.json")
            or manifest["baseline_analysis_sha256"] != digest(BASE / "analysis.json")):
        raise ValueError("manifest/baseline mismatch")
    for path, expected in sources.items():
        if digest(Path(path)) != expected:
            raise ValueError("source hash mismatch")
    path = Path(manifest["input"]["path"])
    if digest(path) != manifest["input"]["sha256"]:
        raise ValueError("input hash mismatch")
    context = json.loads(path.read_text())
    baseline = json.loads((BASE / "analysis.json").read_text())["results"]
    if [r["alpha"] for r in baseline] != settings["alphas"]:
        raise ValueError("baseline alpha coverage")
    results = []
    for old in baseline:
        if hashlib.sha256(bytes(old["assignment"])).hexdigest() != old["assignment_sha256"]:
            raise ValueError("seed hash mismatch")
        result = improve_swaps(context, old["assignment"], old["alpha"])
        if result["initial"] != old["final"]:
            raise ValueError("seed metrics mismatch")
        result["seed_sha256"] = old["assignment_sha256"]
        results.append(result)
        print(result["alpha"], result["swaps"], result["status"], json.dumps(result["final"]), flush=True)
    write_new(args.out, {"manifest_sha256": digest(args.manifest), "results": results})


if __name__ == "__main__":
    main()
