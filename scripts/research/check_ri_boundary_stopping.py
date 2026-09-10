#!/usr/bin/env python3
"""Check claimed single-vertex stopping states using the frozen search engine."""
import argparse
import json
from pathlib import Path
from analyze_dfs_county_response import digest
from run_multiroot_county_diagnostic import write_new
from run_ri_boundary_moves import improve


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--analysis", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    analysis = json.loads(args.analysis.read_text())
    if analysis["manifest_sha256"] != digest(args.manifest):
        raise ValueError("manifest mismatch")
    for path, expected in manifest["sources"].items():
        if digest(Path(path)) != expected:
            raise ValueError("source mismatch")
    path = Path(manifest["input"]["path"])
    if digest(path) != manifest["input"]["sha256"]:
        raise ValueError("context mismatch")
    context = json.loads(path.read_text())
    checks = []
    for result in analysis["results"]:
        if result["moves"] > manifest["settings"]["move_budget"] or result["connectivity_checks"] > manifest["settings"]["connectivity_candidate_check_budget"]:
            raise ValueError("budget violation")
        if result["status"] != "no-improving-feasible-single-vertex-move":
            checks.append({"alpha": result["alpha"], "status": "not-claimed"})
            continue
        recheck = improve(context, result["assignment"], result["alpha"], move_budget=1,
                          check_budget=manifest["settings"]["connectivity_candidate_check_budget"],
                          band=manifest["settings"]["band_ppm"])
        if recheck["moves"] != 0 or recheck["status"] != result["status"] or recheck["final"] != result["final"]:
            raise ValueError("stopping condition not reproduced")
        checks.append({"alpha": result["alpha"], "status": "reproduced",
                       "connectivity_checks": recheck["connectivity_checks"]})
    write_new(args.out, {"analysis_sha256": digest(args.analysis), "checker_sha256": digest(Path(__file__)),
                        "checks": checks, "scope": "Same-engine local stopping check; not independent or global optimality."})
    print(json.dumps(checks))


if __name__ == "__main__":
    main()
