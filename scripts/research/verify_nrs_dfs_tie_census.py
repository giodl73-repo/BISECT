#!/usr/bin/env python3
"""Verify and regenerate the NRS v0.3 initial DFS tie census."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


SCHEMA_VERSION = "nrs-v0.3-initial-dfs-tie-census-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-initial-dfs-tie-census-manifest-v1"


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
    root = Path.cwd().resolve()
    package = args.package.resolve()
    manifest = json.loads((package / "manifest.json").read_text(encoding="utf-8"))
    analysis = json.loads((package / "analysis.json").read_text(encoding="utf-8"))
    if manifest.get("schema_version") != MANIFEST_VERSION:
        fail("manifest schema")
    if analysis.get("schema_version") != SCHEMA_VERSION:
        fail("analysis schema")
    if manifest.get("status") != "pass" or analysis.get("status") != "pass":
        fail("package status")

    for collection in ("inputs", "code"):
        for row in manifest[collection]:
            path = root / row["path"]
            if not path.is_file() or sha256(path) != row["sha256"]:
                fail(f"{collection[:-1]} hash for {row['path']}")
    for name, expected in manifest["outputs"].items():
        path = package / name
        if not path.is_file() or sha256(path) != expected:
            fail(f"output hash for {name}")

    with (package / "state-results.csv").open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if len(rows) != 44 or rows != sorted(rows, key=lambda row: row["state"]):
        fail("State universe or order")
    opportunities = []
    deviation_counts = []
    cut_counts = []
    for row in rows:
        if row["status"] != "accepted":
            fail(f"State status for {row['state']}")
        deviation_count = int(row["minimum_deviation_candidates"])
        cut_count = int(row["minimum_deviation_cut_candidates"])
        if not (0 < cut_count <= deviation_count):
            fail(f"candidate counts for {row['state']}")
        if row["assignment_match"] != "true" or row["objective_match"] != "true":
            fail(f"behavior preservation for {row['state']}")
        opportunity = cut_count > 1
        if (row["seed_sensitive_tie_opportunity"] == "true") != opportunity:
            fail(f"tie opportunity for {row['state']}")
        if opportunity:
            opportunities.append(row["state"])
        deviation_counts.append(deviation_count)
        cut_counts.append(cut_count)
    if opportunities != analysis["seed_sensitive_tie_opportunity_states"]:
        fail("opportunity State list")
    if len(opportunities) != analysis["seed_sensitive_tie_opportunity_count"]:
        fail("opportunity count")
    expected_deviation_counts = {
        str(value): deviation_counts.count(value)
        for value in sorted(set(deviation_counts))
    }
    expected_cut_counts = {
        str(value): cut_counts.count(value) for value in sorted(set(cut_counts))
    }
    if (
        analysis["minimum_deviation_candidate_distribution"]["counts"]
        != expected_deviation_counts
    ):
        fail("deviation count distribution")
    if (
        analysis["minimum_deviation_cut_candidate_distribution"]["counts"]
        != expected_cut_counts
    ):
        fail("cut count distribution")

    reproduction = manifest["reproduction"]
    runner = root / "scripts/research/run_nrs_dfs_tie_census.py"
    with tempfile.TemporaryDirectory() as temp_dir:
        regenerated = Path(temp_dir) / "package"
        completed = subprocess.run(
            [
                sys.executable,
                str(runner),
                "--binary",
                str(root / reproduction["binary"]),
                "--output-dir",
                str(regenerated),
                "--workers",
                str(reproduction["workers"]),
                "--display-output-dir",
                reproduction["display_output_dir"],
            ],
            cwd=root,
            check=False,
        )
        if completed.returncode != 0:
            fail("regeneration command")
        for name in (
            "analysis.json",
            "state-results.csv",
            "README.md",
            "manifest.json",
        ):
            if (regenerated / name).read_bytes() != (package / name).read_bytes():
                fail(f"regenerated {name} differs")

    print("NRS v0.3 initial DFS tie census verification: PASS")


if __name__ == "__main__":
    main()
