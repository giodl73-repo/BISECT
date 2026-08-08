#!/usr/bin/env python3
"""Verify and regenerate the NRS v0.3 fallback candidate census."""

from __future__ import annotations

import argparse
import csv
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import run_nrs_dfs_tie_census as tie


SCHEMA_VERSION = "nrs-v0.3-fallback-candidate-census-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-fallback-candidate-census-manifest-v1"


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
            if not path.is_file() or tie.sha256(path) != row["sha256"]:
                fail(f"{collection[:-1]} hash for {row['path']}")
    for name, expected in manifest["outputs"].items():
        path = package / name
        if not path.is_file() or tie.sha256(path) != expected:
            fail(f"output hash for {name}")

    with (package / "state-results.csv").open(newline="", encoding="utf-8") as handle:
        states = list(csv.DictReader(handle))
    with (package / "stage-results.csv").open(newline="", encoding="utf-8") as handle:
        stages = list(csv.DictReader(handle))
    if len(states) != 6 or len(stages) != 8:
        fail("State or stage/node universe")
    if any(
        row["status"] != "accepted"
        or row["state_assignment_match"] != "true"
        or int(row["node_assignment_match_count"]) != int(row["node_count"])
        or int(row["node_objective_match_count"]) != int(row["node_count"])
        for row in states
    ):
        fail("State preservation")
    opportunities = []
    for row in stages:
        evaluated = int(row["evaluated_candidates"])
        deviation = int(row["minimum_deviation_candidates"])
        cut = int(row["minimum_deviation_cut_candidates"])
        partitions = int(row["minimum_deviation_cut_partitions"])
        if not (0 < partitions <= cut <= deviation <= evaluated):
            fail(f"diagnostics for {row['year']}/{row['state']}/{row['path']}")
        if row["assignment_match"] != "true" or row["objective_match"] != "true":
            fail(f"preservation for {row['year']}/{row['state']}/{row['path']}")
        opportunity = partitions > 1
        if (row["physical_partition_opportunity"] == "true") != opportunity:
            fail(f"opportunity for {row['year']}/{row['state']}/{row['path']}")
        if opportunity:
            opportunities.append(
                f"{row['year']}/{row['state']}/{row['path']}/{row['stage']}"
            )
    if opportunities != analysis["physical_partition_opportunities"]:
        fail("opportunity list")

    reproduction = manifest["reproduction"]
    runner = root / "scripts/research/run_nrs_fallback_candidate_census.py"
    with tempfile.TemporaryDirectory() as temp_dir:
        regenerated = Path(temp_dir) / "package"
        completed = subprocess.run(
            [
                sys.executable,
                str(runner),
                "--bisect",
                str(root / reproduction["bisect"]),
                "--ops",
                str(root / reproduction["ops"]),
                "--output-dir",
                str(regenerated),
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
            "stage-results.csv",
            "README.md",
            "manifest.json",
        ):
            if (regenerated / name).read_bytes() != (package / name).read_bytes():
                fail(f"regenerated {name} differs")
    print("NRS v0.3 fallback candidate census verification: PASS")


if __name__ == "__main__":
    main()
