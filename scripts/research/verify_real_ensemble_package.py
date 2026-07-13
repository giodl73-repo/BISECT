#!/usr/bin/env python3
"""Verify the G.1-G.3 real ensemble evidence package."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fail(message: str) -> None:
    raise SystemExit(f"verification failed: {message}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    root = args.package.resolve()
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    if manifest["schema_version"] != "g-ensemble-evidence-manifest v1":
        fail("unsupported manifest schema")
    if manifest["status"] != "active" or manifest.get("missing_evidence"):
        fail("real package must be active with no missing-evidence rows")

    for row in manifest["files"]:
        path = root / row["path"]
        if not path.is_file():
            fail(f"missing file {row['path']}")
        actual = sha256(path)
        if actual != row["sha256"]:
            fail(f"hash mismatch for {row['path']}: {actual}")

    states = ["ri", "ia", "nc"]
    for state in states:
        for implementation in ["rust", "gerrychain"]:
            trace = json.loads(
                (root / state / f"{implementation}-trace.json").read_text(encoding="utf-8")
            )
            if trace["chains"] != 4 or trace["steps_per_chain"] != 2000:
                fail(f"{state}/{implementation} trace has wrong chain shape")
            if abs(trace["population_tolerance"] - 0.005) > 1e-12:
                fail(f"{state}/{implementation} tolerance drift")
            if any(len(chain["metrics"]) != 2000 for chain in trace["chain_traces"]):
                fail(f"{state}/{implementation} missing metric rows")
        certificate = json.loads(
            (root / state / "audit-certificate.json").read_text(encoding="utf-8")
        )
        if certificate["result"] != "pass":
            fail(f"{state} RPLAN audit did not pass")
        required_checks = {"plan-shape", "population", "contiguity"}
        passed = {
            check["name"]
            for check in certificate["checks"]
            if check["status"] == "pass"
        }
        if not required_checks.issubset(passed):
            fail(f"{state} audit is missing required passes")

    analysis = json.loads((root / "analysis.json").read_text(encoding="utf-8"))
    if set(analysis["states"]) != {"RI", "IA", "NC"}:
        fail("analysis state set drift")
    if analysis["burn_in"] != 500:
        fail("analysis burn-in drift")

    software = json.loads((root / "software.json").read_text(encoding="utf-8"))
    repository = Path.cwd()
    for state, record in software["states"].items():
        for input_record in record.values():
            source_path = repository / input_record["path"]
            if source_path.is_file() and sha256(source_path) != input_record["sha256"]:
                fail(f"source input hash mismatch for {state}: {input_record['path']}")
    wisconsin_path = (
        repository
        / "runs/nrs_reference_v0_1/2020/wisconsin/final_assignments.json"
    )
    if wisconsin_path.is_file() and sha256(wisconsin_path) != software[
        "wisconsin_ineligible"
    ]["baseline_assignment_sha256"]:
        fail("Wisconsin ineligible baseline hash mismatch")

    script = Path("scripts/research/analyze_real_ensemble.py").resolve()
    with tempfile.TemporaryDirectory() as temp_dir:
        temp = Path(temp_dir)
        regenerated_json = temp / "analysis.json"
        regenerated_csv = temp / "summary.csv"
        command = [
            sys.executable,
            str(script),
            "--root",
            str(root),
            "--states",
            "RI",
            "IA",
            "NC",
            "--burn-in",
            "500",
            "--output",
            str(regenerated_json),
            "--summary-csv",
            str(regenerated_csv),
        ]
        completed = subprocess.run(command, check=False)
        if completed.returncode != 0:
            fail("analysis regeneration command failed")
        regenerated = json.loads(regenerated_json.read_text(encoding="utf-8"))
        if regenerated != analysis:
            fail("analysis.json does not match regenerated analysis")
        if regenerated_csv.read_text(encoding="utf-8") != (
            root / "summary.csv"
        ).read_text(encoding="utf-8"):
            fail("summary.csv does not match regenerated analysis")

    print("real ensemble package verification: PASS")


if __name__ == "__main__":
    main()
