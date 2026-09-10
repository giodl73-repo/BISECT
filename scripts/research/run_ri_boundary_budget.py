#!/usr/bin/env python3
"""Frozen 500-move continuation experiment using the unchanged move engine."""
import argparse
import hashlib
import json
from pathlib import Path
from analyze_dfs_county_response import ROOT, digest
from run_multiroot_county_diagnostic import write_new
from run_ri_boundary_moves import improve, recover_seed, metrics
from run_ri_county_tree import edges_from

BASE = ROOT / "docs/experiments/ri-boundary-moves-2020"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "run"))
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    base = json.loads((BASE / "manifest.json").read_text())
    settings = dict(base["settings"], move_budget=500, checkpoints=[50, 100, 250, 500])
    sources = dict(base["sources"])
    sources[str(Path(__file__).absolute())] = digest(Path(__file__))
    if args.action == "prepare":
        write_new(args.manifest, {"schema": "ri-boundary-budget-v1", "settings": settings,
                  "seed_sha256": base["seed_sha256"], "input": base["input"], "sources": sources,
                  "baseline_manifest_sha256": digest(BASE / "manifest.json"),
                  "baseline_analysis_sha256": digest(BASE / "analysis.json")})
        return
    if args.out is None or args.out.exists():
        raise ValueError("requires unused output")
    manifest = json.loads(args.manifest.read_text())
    if (manifest["schema"] != "ri-boundary-budget-v1" or manifest["settings"] != settings
            or manifest["sources"] != sources or manifest["seed_sha256"] != base["seed_sha256"]
            or manifest["input"] != base["input"]
            or manifest["baseline_manifest_sha256"] != digest(BASE / "manifest.json")
            or manifest["baseline_analysis_sha256"] != digest(BASE / "analysis.json")):
        raise ValueError("manifest/baseline mismatch")
    for path, expected in sources.items():
        if digest(Path(path)) != expected:
            raise ValueError("source mismatch")
    path = Path(manifest["input"]["path"])
    if digest(path) != manifest["input"]["sha256"]:
        raise ValueError("input mismatch")
    context = json.loads(path.read_text())
    seed = recover_seed(context, manifest["seed_sha256"])
    baseline = {r["alpha"]: r for r in json.loads((BASE / "analysis.json").read_text())["results"]}
    edges, results = edges_from(context), []
    for alpha in settings["alphas"]:
        result = improve(context, seed, alpha, move_budget=settings["move_budget"],
                         check_budget=settings["connectivity_candidate_check_budget"], band=settings["band_ppm"])
        old = baseline[alpha]
        if result["trace"][:50] != old["trace"]:
            raise ValueError("50-move control prefix changed")
        labels, checkpoints = bytearray(seed), []
        for index, move in enumerate(result["trace"], 1):
            labels[move["unit_index"]] = move["new_label"]
            if index in settings["checkpoints"]:
                snapshot = metrics(context["units"]["unit_ids"], context["populations"], edges, labels, alpha)
                label_hash = hashlib.sha256(labels).hexdigest()
                if index == 50 and (snapshot != old["final"] or label_hash != old["assignment_sha256"]):
                    raise ValueError("50-move snapshot mismatch")
                checkpoints.append({"moves": index, "metrics": snapshot, "assignment_sha256": label_hash})
        result["checkpoints"] = checkpoints
        result["baseline_prefix_matches"] = True
        results.append(result)
        print(alpha, result["moves"], result["status"], json.dumps(result["final"]), flush=True)
    write_new(args.out, {"manifest_sha256": digest(args.manifest), "results": results})


if __name__ == "__main__":
    main()
