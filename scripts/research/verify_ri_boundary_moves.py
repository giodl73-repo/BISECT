#!/usr/bin/env python3
"""Replay published moves and verify feasibility/costs without running search."""
import argparse
import hashlib
import json
from fractions import Fraction
from pathlib import Path
from run_ri_boundary_moves import connected, metrics, recover_seed
from run_ri_county_tree import edges_from
from run_ri_population_band import eligible
from run_multiroot_county_diagnostic import write_new
from analyze_dfs_county_response import digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--analysis", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    manifest, analysis = json.loads(args.manifest.read_text()), json.loads(args.analysis.read_text())
    if analysis["manifest_sha256"] != digest(args.manifest):
        raise ValueError("manifest mismatch")
    for path, expected in manifest["sources"].items():
        if digest(Path(path)) != expected:
            raise ValueError("source mismatch")
    path = Path(manifest["input"]["path"])
    if digest(path) != manifest["input"]["sha256"]:
        raise ValueError("context mismatch")
    context = json.loads(path.read_text())
    ids, pops = context["units"]["unit_ids"], context["populations"]
    edges = edges_from(context)
    adjacency = [[] for _ in ids]
    for (u, v), weight in edges.items():
        adjacency[u].append((v, weight))
        adjacency[v].append((u, weight))
    seed = recover_seed(context, manifest["seed_sha256"])
    if [r["alpha"] for r in analysis["results"]] != manifest["settings"]["alphas"]:
        raise ValueError("alpha coverage mismatch")
    count = 0
    for result in analysis["results"]:
        labels = bytearray(seed)
        previous = metrics(ids, pops, edges, labels, result["alpha"])
        if previous != result["initial"] or len(result["trace"]) != result["moves"]:
            raise ValueError("initial/trace mismatch")
        for move in result["trace"]:
            u = move["unit_index"]
            if ids[u] != move["geoid"] or move["new_label"] != 1 - labels[u]:
                raise ValueError("invalid move")
            labels[u] = move["new_label"]
            current = metrics(ids, pops, edges, labels, result["alpha"])
            delta = Fraction(current["weighted_cut"]) - Fraction(previous["weighted_cut"])
            if delta >= 0 or delta != Fraction(move["scaled_cost_delta"], Fraction(result["alpha"]).denominator):
                raise ValueError("cost descent mismatch")
            if not eligible(current["deviation_numerator"], sum(pops), manifest["settings"]["band_ppm"]):
                raise ValueError("population violation")
            if not all(connected(adjacency, labels, label) for label in (0, 1)):
                raise ValueError("connectivity violation")
            previous = current
            count += 1
        if list(labels) != result["assignment"] or hashlib.sha256(labels).hexdigest() != result["assignment_sha256"] or previous != result["final"]:
            raise ValueError("final assignment/metrics mismatch")
    write_new(args.out, {"status": "passed", "moves_replayed": count,
                        "analysis_sha256": digest(args.analysis), "manifest_sha256": digest(args.manifest),
                        "verifier_sha256": digest(Path(__file__)),
                        "scope": "Trace replay, not search optimality; shares metric/connectivity helpers, not independent implementation."})
    print(f"Verified {count} published moves")


if __name__ == "__main__":
    main()
