#!/usr/bin/env python3
"""Analyze and verify the local Rhode Island compact proof-model package."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
LOCAL = Path("data/2020/certified/ri-root-models")
SCRIPT = Path("scripts/research/analyze_ri_model_package.py")
SCHEMA = "ri-compact-proof-model-frontier-v1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_hash(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def analyze(report_path: Path, manifest_path: Path) -> None:
    local_manifest_path = ROOT / LOCAL / "manifest.json"
    local = json.loads(local_manifest_path.read_text(encoding="utf-8"))
    if local["compiler_sha256"] != sha256(ROOT / local["compiler_path"]):
        raise SystemExit("RI model compiler hash mismatch")
    for artifact in local["artifacts"].values():
        opb_path = ROOT / LOCAL / artifact["path"]
        request_path = ROOT / LOCAL / artifact["request_path"]
        if sha256(opb_path) != artifact["sha256"]:
            raise SystemExit(f"RI model hash mismatch: {artifact['path']}")
        if sha256(request_path) != artifact["request_sha256"]:
            raise SystemExit(f"RI request hash mismatch: {artifact['request_path']}")
        request = json.loads(request_path.read_text(encoding="utf-8"))
        if request["opb_sha256"] != f"sha256:{artifact['sha256']}":
            raise SystemExit(f"RI request OPB identity mismatch: {artifact['path']}")
        if (
            request["variable_count"] != artifact["variable_count"]
            or request["constraint_count"] != artifact["constraint_count"]
        ):
            raise SystemExit(f"RI request model-size mismatch: {artifact['path']}")
        projection = {
            key: request[key]
            for key in (
                "schema_version",
                "instance_hash",
                "discovery_id",
                "stage",
                "connectivity_encoding",
                "exact_right_population",
                "status",
                "opb_sha256",
                "variable_count",
                "constraint_count",
                "proof_format",
                "proof_status",
                "solver_command_template",
                "claim",
            )
        }
        if request["request_id"] != canonical_hash(projection):
            raise SystemExit(f"RI request ID mismatch: {artifact['request_path']}")
    proof_frontier_path = ROOT / "docs/examples/ri-proof-frontier/provenance.json"
    proof_frontier = (
        json.loads(proof_frontier_path.read_text(encoding="utf-8"))
        if proof_frontier_path.is_file()
        else None
    )
    report = {
        "schema_version": SCHEMA,
        "status": "partial-proof",
        "state": "rhode_island",
        "year": 2020,
        "instance_hash": local["instance_hash"],
        "discovery_id": local["discovery_id"],
        "local_package": {
            "path": LOCAL.as_posix(),
            "committed": False,
            "manifest_sha256": sha256(local_manifest_path),
            "total_model_bytes": sum(
                row["bytes"] for row in local["artifacts"].values()
            ),
        },
        "artifacts": local["artifacts"],
        "population_stage": (
            proof_frontier["population_stage"]
            if proof_frontier
            else {"status": "not-run"}
        ),
        "boundary_stage": (
            proof_frontier["boundary_stage"]
            if proof_frontier
            else {"status": "not-run"}
        ),
        "strengthened_boundary_branches": (
            proof_frontier.get("current_boundary_branches", [])
            if proof_frontier
            else []
        ),
        "claim_boundary": (
            "Hash-bound compact RI proof inputs and a timed solver probe; "
            "not a proof result or certified split."
        ),
    }
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "ri-compact-proof-model-frontier-package-v1",
        "package_id": "ri-2020-root-proof-model-frontier",
        "status": report["status"],
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "analyzer_path": SCRIPT.as_posix(),
        "analyzer_sha256": sha256(ROOT / SCRIPT),
        "verification_commands": [
            (
                "python scripts/research/analyze_ri_model_package.py verify "
                f"{manifest_path.relative_to(ROOT).as_posix()}"
            ),
            (
                "python scripts/research/analyze_ri_model_package.py verify "
                f"{manifest_path.relative_to(ROOT).as_posix()} --check-local"
            ),
        ],
        "claim_boundary": report["claim_boundary"],
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print("RI compact proof model analysis: PROOF REQUIRED")


def verify(manifest_path: Path, check_local: bool) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["analyzer_sha256"] != sha256(ROOT / manifest["analyzer_path"]):
        raise SystemExit("RI model analyzer hash mismatch")
    report_path = manifest_path.parent / manifest["files"][0]["path"]
    if manifest["files"][0]["sha256"] != sha256(report_path):
        raise SystemExit("RI model report hash mismatch")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if report["schema_version"] != SCHEMA:
        raise SystemExit("RI model report schema drift")
    if report["population_stage"]["status"] not in {
        "verified-unsat",
        "sat-counterexample",
        "timelimit",
        "not-run",
    }:
        raise SystemExit("RI population stage status drift")
    if len(report["artifacts"]) < 5:
        raise SystemExit("RI proof model artifact inventory drift")
    if check_local:
        local = json.loads((ROOT / LOCAL / "manifest.json").read_text(encoding="utf-8"))
        if local["instance_hash"] != report["instance_hash"]:
            raise SystemExit("RI local model instance drift")
        for artifact in local["artifacts"].values():
            if sha256(ROOT / LOCAL / artifact["path"]) != artifact["sha256"]:
                raise SystemExit(f"RI local model hash mismatch: {artifact['path']}")
    if check_local:
        print("RI compact proof model verification: PASS")
    else:
        print("RI compact proof model metadata verification: PASS (models not checked)")


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    analyze_parser = subparsers.add_parser("analyze")
    analyze_parser.add_argument("--report", type=Path, required=True)
    analyze_parser.add_argument("--manifest", type=Path, required=True)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("manifest", type=Path)
    verify_parser.add_argument("--check-local", action="store_true")
    args = parser.parse_args()
    if args.command == "analyze":
        analyze(ROOT / args.report, ROOT / args.manifest)
    else:
        verify(ROOT / args.manifest, args.check_local)


if __name__ == "__main__":
    main()
