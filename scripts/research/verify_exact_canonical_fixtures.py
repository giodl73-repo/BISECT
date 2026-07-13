#!/usr/bin/env python3
"""Verify committed Exact Canonical E0 fixture packages."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    root = Path("docs/examples/exact-canonical")
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    declared_paths = {row["path"] for row in manifest["files"]}
    actual_paths = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "manifest.json"
    }
    if declared_paths != actual_paths:
        raise SystemExit("exact fixture manifest inventory mismatch")
    for row in manifest["files"]:
        path = root / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            raise SystemExit(f"exact fixture hash mismatch: {row['path']}")

    optimal = json.loads(
        (
            root
            / "path4-optimal/output/exact-canonical-certificate.json"
        ).read_text(encoding="utf-8")
    )
    if optimal["result"]["result"] != "optimal":
        raise SystemExit("path4 fixture is not optimal")
    if optimal["result"]["assignment"] != [0, 0, 1, 1]:
        raise SystemExit("path4 canonical assignment drift")
    if optimal["result"]["objective"]["primary"] != {
        "max_population_deviation_scaled": 0,
        "total_population_deviation_scaled": 0,
        "weighted_boundary_cut": 1,
    }:
        raise SystemExit("path4 objective drift")
    optimal_proof = json.loads(
        (
            root / "path4-optimal/output/exact-canonical-proof.json"
        ).read_text(encoding="utf-8")
    )
    if optimal_proof["schema_version"] != "exact-canonical-proof-v1":
        raise SystemExit("path4 proof schema drift")
    if optimal["proof"]["transcript_id"] != optimal_proof["transcript_id"]:
        raise SystemExit("path4 certificate/proof binding mismatch")
    if optimal_proof["canonical_assignment"] != [0, 0, 1, 1]:
        raise SystemExit("path4 proof assignment drift")

    infeasible = json.loads(
        (
            root
            / "three-islands-infeasible/output/exact-canonical-certificate.json"
        ).read_text(encoding="utf-8")
    )
    if infeasible["result"]["result"] != "infeasible":
        raise SystemExit("three-islands fixture is not infeasible")
    if infeasible["proof"]["feasible_assignments"] != 0:
        raise SystemExit("infeasibility proof drift")
    infeasible_proof = json.loads(
        (
            root / "three-islands-infeasible/output/exact-canonical-proof.json"
        ).read_text(encoding="utf-8")
    )
    if infeasible["proof"]["transcript_id"] != infeasible_proof["transcript_id"]:
        raise SystemExit("infeasible certificate/proof binding mismatch")
    if infeasible_proof["feasible_count"] != 0:
        raise SystemExit("infeasible proof transcript drift")

    expected_cases = {
        "certificate-hash-tamper": "certificate-id-mismatch",
        "disconnected-assignment": "result-mismatch",
        "false-infeasibility": "result-mismatch",
        "false-optimum": "result-mismatch",
        "noncanonical-tie": "result-mismatch",
    }
    corpus = root / "negative-corpus"
    found_cases = {path.name for path in corpus.iterdir() if path.is_dir()}
    if found_cases != set(expected_cases):
        raise SystemExit("negative corpus case inventory drift")
    for case, expected_error in expected_cases.items():
        case_root = corpus / case
        expected = json.loads(
            (case_root / "expected.json").read_text(encoding="utf-8")
        )
        if expected != {
            "schema_version": "exact-canonical-negative-fixture-v1",
            "case": case,
            "expected_error": expected_error,
            "description": expected["description"],
        }:
            raise SystemExit(f"negative fixture declaration drift: {case}")
        for name in (
            "exact-canonical-instance.json",
            "exact-canonical-certificate.json",
            "exact-canonical-proof.json",
        ):
            if not (case_root / name).is_file():
                raise SystemExit(f"negative fixture artifact missing: {case}/{name}")
    independent_report = json.loads(
        (root / "independent-verifier-report.json").read_text(encoding="utf-8")
    )
    if independent_report["schema_version"] != (
        "exact-canonical-independent-verifier-report-v1"
    ):
        raise SystemExit("independent verifier report schema drift")
    if independent_report["verifier_id"] != (
        "python-exact-canonical-independent-v1"
    ):
        raise SystemExit("independent verifier identity drift")
    verifier_path = Path(independent_report["verifier_path"])
    if independent_report["verifier_sha256"] != sha256(verifier_path):
        raise SystemExit("independent verifier source hash mismatch")
    if len(independent_report["positive_cases"]) != 2 or {
        row["case"] for row in independent_report["negative_cases"]
    } != expected_cases.keys():
        raise SystemExit("independent verifier result inventory drift")
    print("Exact Canonical fixture verification: PASS")


if __name__ == "__main__":
    main()
