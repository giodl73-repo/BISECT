#!/usr/bin/env python3
"""Verify and summarize the six 2020 one-district State packages."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from verify_certified_single_fixtures import verify


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path("scripts/research/analyze_single_district_states.py")
STATES = ("ak", "de", "nd", "sd", "vt", "wy")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def analyze(report_path: Path, manifest_path: Path) -> None:
    rows = []
    for state in STATES:
        rctx_report_path = (
            ROOT / f"docs/experiments/small-states-2020/{state}-rctx.json"
        )
        package = ROOT / f"data/2020/certified/single-district/{state}"
        package_manifest_path = package / "manifest.json"
        instance_path = package / "single-district-instance.json"
        certificate_path = package / "single-district-certificate.json"
        rctx_report = json.loads(rctx_report_path.read_text(encoding="utf-8"))
        package_manifest = json.loads(
            package_manifest_path.read_text(encoding="utf-8")
        )
        for name, expected in package_manifest["files"].items():
            if sha256(package / name) != expected:
                raise SystemExit(f"{state} package hash mismatch: {name}")
        instance = json.loads(instance_path.read_text(encoding="utf-8"))
        certificate = json.loads(certificate_path.read_text(encoding="utf-8"))
        verify(instance, certificate)
        rows.append(
            {
                "state": state.upper(),
                "unit_count": certificate["unit_count"],
                "population_total": certificate["population_total"],
                "land_edge_count": rctx_report["land_edge_count"],
                "bridge_edge_count": rctx_report["bridge_edge_count"],
                "final_component_count": rctx_report["final_component_count"],
                "weighted_boundary_cut": certificate["weighted_boundary_cut"],
                "certificate_id": certificate["certificate_id"],
                "rctx_sha256": rctx_report["rctx_sha256"],
                "package_manifest_sha256": sha256(package_manifest_path),
                "status": "verified",
            }
        )
    report = {
        "schema_version": "certified-single-district-states-2020-v1",
        "status": "verified",
        "state_count": len(rows),
        "states": rows,
        "total_units": sum(row["unit_count"] for row in rows),
        "total_population": sum(row["population_total"] for row in rows),
        "claim_boundary": (
            "Six complete connected wall-to-wall 2020 one-district packages; "
            "source data remain under local hash-bound custody."
        ),
    }
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-single-district-states-package-v1",
        "package_id": "one-district-states-2020",
        "status": "verified",
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "analyzer_path": SCRIPT.as_posix(),
        "analyzer_sha256": sha256(ROOT / SCRIPT),
        "verification_commands": [
            (
                "python scripts/research/analyze_single_district_states.py verify "
                f"{manifest_path.relative_to(ROOT).as_posix()}"
            )
        ],
        "claim_boundary": report["claim_boundary"],
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print("Six one-district States: VERIFIED")


def verify_report(manifest_path: Path) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if sha256(ROOT / manifest["analyzer_path"]) != manifest["analyzer_sha256"]:
        raise SystemExit("single-district analyzer hash mismatch")
    report_path = manifest_path.parent / manifest["files"][0]["path"]
    if sha256(report_path) != manifest["files"][0]["sha256"]:
        raise SystemExit("single-district report hash mismatch")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if (
        report["status"] != "verified"
        or report["state_count"] != 6
        or any(row["status"] != "verified" for row in report["states"])
        or any(row["final_component_count"] != 1 for row in report["states"])
        or any(row["weighted_boundary_cut"] != 0 for row in report["states"])
    ):
        raise SystemExit("single-district report posture drift")
    print("Six one-district State report verification: PASS")


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
        verify_report(ROOT / args.manifest)


if __name__ == "__main__":
    main()
