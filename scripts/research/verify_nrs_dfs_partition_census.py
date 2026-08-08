#!/usr/bin/env python3
"""Verify and regenerate the NRS v0.3 initial DFS partition census."""

from __future__ import annotations

import argparse
import csv
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import run_nrs_dfs_tie_census as tie


SCHEMA_VERSION = "nrs-v0.3-initial-dfs-partition-census-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-initial-dfs-partition-census-manifest-v1"


def fail(message: str) -> None:
    raise SystemExit(f"verification failed: {message}")


def count_distribution(values: list[int]) -> dict[str, int]:
    return {str(value): values.count(value) for value in sorted(set(values))}


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
        rows = list(csv.DictReader(handle))
    if len(rows) != 44 or rows != sorted(rows, key=lambda row: row["state"]):
        fail("State universe or order")

    cut_counts = []
    partition_counts = []
    orientation_states = []
    physical_states = []
    for row in rows:
        if row["status"] != "accepted":
            fail(f"State status for {row['state']}")
        deviation_count = int(row["minimum_deviation_candidates"])
        cut_count = int(row["minimum_deviation_cut_candidates"])
        partition_count = int(row["minimum_deviation_cut_partitions"])
        if not (0 < partition_count <= cut_count <= deviation_count):
            fail(f"candidate or partition counts for {row['state']}")
        if row["assignment_match"] != "true" or row["objective_match"] != "true":
            fail(f"behavior preservation for {row['state']}")
        orientation_only = cut_count > partition_count
        physical = partition_count > 1
        if (row["orientation_only_tie"] == "true") != orientation_only:
            fail(f"orientation classification for {row['state']}")
        if (row["physical_cut_opportunity"] == "true") != physical:
            fail(f"physical-cut classification for {row['state']}")
        if orientation_only:
            orientation_states.append(row["state"])
        if physical:
            physical_states.append(row["state"])
        cut_counts.append(cut_count)
        partition_counts.append(partition_count)

    if orientation_states != analysis["orientation_only_tie_states"]:
        fail("orientation State list")
    if physical_states != analysis["physical_cut_opportunity_states"]:
        fail("physical-cut State list")
    if len(orientation_states) != analysis["orientation_only_tie_count"]:
        fail("orientation count")
    if len(physical_states) != analysis["physical_cut_opportunity_count"]:
        fail("physical-cut count")
    if (
        analysis["minimum_deviation_cut_candidate_distribution"]["counts"]
        != count_distribution(cut_counts)
    ):
        fail("oriented cut distribution")
    if (
        analysis["minimum_deviation_cut_partition_distribution"]["counts"]
        != count_distribution(partition_counts)
    ):
        fail("partition distribution")

    reproduction = manifest["reproduction"]
    runner = root / "scripts/research/run_nrs_dfs_partition_census.py"
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

    print("NRS v0.3 initial DFS partition census verification: PASS")


if __name__ == "__main__":
    main()
