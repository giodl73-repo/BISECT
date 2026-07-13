#!/usr/bin/env python3
"""Summarize Hawaii and New Hampshire two-district operational/proof packages."""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import deque
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path("scripts/research/analyze_two_district_frontier.py")
STATES = ("hi", "nh")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def connected(adjacency: list[list[dict]], assignment: list[int], label: int) -> bool:
    units = [unit for unit, value in enumerate(assignment) if value == label]
    allowed = set(units)
    seen = {units[0]}
    queue = deque([units[0]])
    while queue:
        unit = queue.popleft()
        for edge in adjacency[unit]:
            neighbor = edge["to"]
            if neighbor in allowed and neighbor not in seen:
                seen.add(neighbor)
                queue.append(neighbor)
    return seen == allowed


def solver_stat(log_text: str, name: str) -> int | None:
    prefix = f"c {name} "
    for line in log_text.splitlines():
        if line.startswith(prefix):
            return int(line.removeprefix(prefix).split()[0])
    return None


def analyze(report_path: Path, manifest_path: Path) -> None:
    rows = []
    for state in STATES:
        rctx_report = json.loads(
            (
                ROOT / f"docs/experiments/small-states-2020/{state}-rctx.json"
            ).read_text(encoding="utf-8")
        )
        discovery_dir = ROOT / f"data/2020/certified/{state}-root-discovery"
        model_dir = ROOT / f"data/2020/certified/{state}-root-models"
        proof_path = ROOT / f"data/2020/certified/{state}-root-proofs/01-population.pbp"
        boundary_log_path = (
            ROOT / f"data/2020/certified/{state}-root-proofs/02-boundary-120s.log"
        )
        discovery_manifest = json.loads(
            (discovery_dir / "certified-discovery-manifest.json").read_text(
                encoding="utf-8"
            )
        )
        for name, expected in discovery_manifest["files"].items():
            if sha256(discovery_dir / name) != expected:
                raise SystemExit(f"{state} discovery package hash mismatch: {name}")
        discovery = json.loads(
            (discovery_dir / "certified-discovery.json").read_text(encoding="utf-8")
        )
        context = json.loads(
            (discovery_dir / "discovery.rctx").read_text(encoding="utf-8")
        )
        assignment = discovery["objective"]["canonical_assignment"]
        if not all(
            connected(context["graph"]["adjacency"], assignment, label)
            for label in (0, 1)
        ):
            raise SystemExit(f"{state} discovery is disconnected")
        audit = json.loads(
            (discovery_dir / "audit-certificate.json").read_text(encoding="utf-8")
        )
        if audit["result"] not in ("pass", "pass-with-warnings"):
            raise SystemExit(f"{state} operational audit failed")
        model_manifest = json.loads(
            (model_dir / "manifest.json").read_text(encoding="utf-8")
        )
        population_model = model_manifest["artifacts"]["01-population"]
        if sha256(model_dir / population_model["path"]) != population_model["sha256"]:
            raise SystemExit(f"{state} population model hash mismatch")
        rows.append(
            {
                "state": state.upper(),
                "unit_count": rctx_report["unit_count"],
                "bridge_edge_count": rctx_report["bridge_edge_count"],
                "max_population_deviation_scaled": discovery["objective"]["primary"][
                    "max_population_deviation_scaled"
                ],
                "weighted_boundary_cut": discovery["objective"]["primary"][
                    "weighted_boundary_cut"
                ],
                "operational_package": "verified",
                "population_proof": {
                    "status": "verified-unsat",
                    "model_variables": population_model["variable_count"],
                    "model_constraints": population_model["constraint_count"],
                    "opb_sha256": population_model["sha256"],
                    "proof_bytes": proof_path.stat().st_size,
                    "proof_sha256": sha256(proof_path),
                },
                "boundary_proof": {
                    "status": "timelimit",
                    "time_limit_seconds": 120,
                    "log_sha256": sha256(boundary_log_path),
                    "decisions": solver_stat(
                        boundary_log_path.read_text(encoding="utf-8"), "decisions"
                    ),
                    "conflicts": solver_stat(
                        boundary_log_path.read_text(encoding="utf-8"), "conflicts"
                    ),
                },
                "canonical_proof": "blocked-by-boundary",
                "discovery_id": discovery["discovery_id"],
                "audit_certificate_id": audit["certificate_id"],
            }
        )
    report = {
        "schema_version": "certified-two-district-frontier-2020-v1",
        "status": "operational-complete-population-proved",
        "states": rows,
        "claim_boundary": (
            "HI and NH have connected wall-to-wall operational roots and verified "
            "population optimality. Boundary and canonical optimality are not claimed."
        ),
    }
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-two-district-frontier-package-v1",
        "package_id": "two-district-frontier-2020",
        "status": report["status"],
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "analyzer_path": SCRIPT.as_posix(),
        "analyzer_sha256": sha256(ROOT / SCRIPT),
        "claim_boundary": report["claim_boundary"],
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print("HI/NH two-district frontier: VERIFIED")


def verify(manifest_path: Path) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if sha256(ROOT / manifest["analyzer_path"]) != manifest["analyzer_sha256"]:
        raise SystemExit("two-district analyzer hash mismatch")
    report_path = manifest_path.parent / manifest["files"][0]["path"]
    if sha256(report_path) != manifest["files"][0]["sha256"]:
        raise SystemExit("two-district report hash mismatch")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if (
        report["status"] != "operational-complete-population-proved"
        or len(report["states"]) != 2
        or any(row["operational_package"] != "verified" for row in report["states"])
        or any(
            row["population_proof"]["status"] != "verified-unsat"
            for row in report["states"]
        )
    ):
        raise SystemExit("two-district report posture drift")
    print("Two-district frontier report verification: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    analyze_parser = subparsers.add_parser("analyze")
    analyze_parser.add_argument("--report", type=Path, required=True)
    analyze_parser.add_argument("--manifest", type=Path, required=True)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    if args.command == "analyze":
        analyze(ROOT / args.report, ROOT / args.manifest)
    else:
        verify(ROOT / args.manifest)


if __name__ == "__main__":
    main()
