#!/usr/bin/env python3
"""Publish a committed frontier report for an operational recursive tree."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path("scripts/research/analyze_operational_tree_package.py")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def analyze(
    state: str,
    package: Path,
    rctx_report_path: Path,
    report_path: Path,
    manifest_path: Path,
) -> None:
    package_manifest = json.loads(
        (package / "manifest.json").read_text(encoding="utf-8")
    )
    tree_path = package / package_manifest["files"][0]["path"]
    if sha256(tree_path) != package_manifest["files"][0]["sha256"]:
        raise SystemExit("operational tree package hash mismatch")
    tree = json.loads(tree_path.read_text(encoding="utf-8"))
    rctx_report = json.loads(rctx_report_path.read_text(encoding="utf-8"))
    if (
        tree["unit_count"] != rctx_report["unit_count"]
        or len(tree["leaves"]) != tree["districts"]
        or sum(leaf["unit_count"] for leaf in tree["leaves"]) != tree["unit_count"]
        or sum(leaf["population"] for leaf in tree["leaves"])
        != tree["population_total"]
        or any(
            node["objective"]["max_population_deviation_scaled"]
            != node["population_proof"]["lower_bound"]
            for node in tree["nodes"]
        )
    ):
        raise SystemExit("operational tree coverage or population proof mismatch")
    report = {
        "schema_version": "certified-operational-tree-frontier-v1",
        "status": "operational-complete-population-proved",
        "state": state,
        "year": 2020,
        "districts": tree["districts"],
        "unit_count": tree["unit_count"],
        "population_total": tree["population_total"],
        "bridge_edge_count": rctx_report["bridge_edge_count"],
        "tree_sha256": sha256(tree_path),
        "package_manifest_sha256": sha256(package / "manifest.json"),
        "nodes": tree["nodes"],
        "leaves": tree["leaves"],
        "boundary_proof": "not-run",
        "canonical_proof": "blocked-by-boundary",
        "claim_boundary": (
            "Complete connected wall-to-wall recursive tree with arithmetic "
            "population optimality at every node; boundary and canonical "
            "optimality are unproved."
        ),
    }
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-operational-tree-frontier-package-v1",
        "package_id": f"{state.lower()}-operational-tree-2020",
        "status": report["status"],
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "analyzer_path": SCRIPT.as_posix(),
        "analyzer_sha256": sha256(ROOT / SCRIPT),
        "claim_boundary": report["claim_boundary"],
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"{state} operational tree frontier: VERIFIED")


def verify(manifest_path: Path) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if sha256(ROOT / manifest["analyzer_path"]) != manifest["analyzer_sha256"]:
        raise SystemExit("operational tree analyzer hash mismatch")
    report_path = manifest_path.parent / manifest["files"][0]["path"]
    if sha256(report_path) != manifest["files"][0]["sha256"]:
        raise SystemExit("operational tree report hash mismatch")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if (
        report["status"] != "operational-complete-population-proved"
        or len(report["leaves"]) != report["districts"]
        or sum(leaf["unit_count"] for leaf in report["leaves"])
        != report["unit_count"]
    ):
        raise SystemExit("operational tree report posture drift")
    print("Operational tree frontier report verification: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    analyze_parser = subparsers.add_parser("analyze")
    analyze_parser.add_argument("--state", required=True)
    analyze_parser.add_argument("--package", type=Path, required=True)
    analyze_parser.add_argument("--rctx-report", type=Path, required=True)
    analyze_parser.add_argument("--report", type=Path, required=True)
    analyze_parser.add_argument("--manifest", type=Path, required=True)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    if args.command == "analyze":
        analyze(
            args.state,
            ROOT / args.package,
            ROOT / args.rctx_report,
            ROOT / args.report,
            ROOT / args.manifest,
        )
    else:
        verify(ROOT / args.manifest)


if __name__ == "__main__":
    main()
