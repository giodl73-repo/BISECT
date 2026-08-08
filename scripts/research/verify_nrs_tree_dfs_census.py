#!/usr/bin/env python3
"""Verify and regenerate the NRS v0.3 complete-tree DFS census."""

from __future__ import annotations

import argparse
import csv
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import run_nrs_dfs_tie_census as tie


SCHEMA_VERSION = "nrs-v0.3-complete-tree-dfs-census-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-complete-tree-dfs-census-manifest-v1"


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
    with (package / "node-results.csv").open(newline="", encoding="utf-8") as handle:
        nodes = list(csv.DictReader(handle))
    if len(states) != 44 or len(nodes) != 385:
        fail("State or node universe")
    if states != sorted(states, key=lambda row: row["state"]):
        fail("State order")
    if nodes != sorted(
        nodes, key=lambda row: (row["state"], len(row["path"]), row["path"])
    ):
        fail("node order")
    if any(
        row["status"] != "accepted"
        or row["state_assignment_match"] != "true"
        or int(row["node_assignment_match_count"]) != int(row["node_count"])
        or int(row["node_objective_match_count"]) != int(row["node_count"])
        for row in states
    ):
        fail("State preservation")

    physical = []
    v02 = []
    v03 = []
    orientation_count = 0
    partition_counts = []
    for row in nodes:
        deviation = int(row["minimum_deviation_candidates"])
        cut = int(row["minimum_deviation_cut_candidates"])
        partitions = int(row["minimum_deviation_cut_partitions"])
        if not (0 < partitions <= cut <= deviation):
            fail(f"candidate counts for {row['state']}/{row['path']}")
        if row["assignment_match"] != "true" or row["objective_match"] != "true":
            fail(f"node preservation for {row['state']}/{row['path']}")
        orientation = cut > partitions
        multiple_physical = partitions > 1
        if (row["orientation_only_tie"] == "true") != orientation:
            fail(f"orientation class for {row['state']}/{row['path']}")
        if (row["physical_cut_opportunity"] == "true") != multiple_physical:
            fail(f"physical class for {row['state']}/{row['path']}")
        node_id = f"{row['state']}/{row['path']}"
        if orientation:
            orientation_count += 1
        if multiple_physical:
            physical.append(node_id)
        if row["nrs_v0_2_fallback_activated"] == "true":
            v02.append(node_id)
        if row["nrs_v0_3_fallback_activated"] == "true":
            v03.append(node_id)
        partition_counts.append(partitions)
    if physical != analysis["physical_cut_opportunity_nodes"]:
        fail("physical node list")
    if v02 != analysis["nrs_v0_2_fallback_activation_nodes"]:
        fail("v0.2 fallback node list")
    if v03 != analysis["nrs_v0_3_fallback_activation_nodes"]:
        fail("v0.3 fallback node list")
    if orientation_count != analysis["orientation_only_tie_node_count"]:
        fail("orientation count")
    if (
        tie.distribution(partition_counts)
        != analysis["minimum_deviation_cut_partition_distribution"]
    ):
        fail("partition distribution")

    reproduction = manifest["reproduction"]
    runner = root / "scripts/research/run_nrs_tree_dfs_census.py"
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
            "node-results.csv",
            "README.md",
            "manifest.json",
        ):
            if (regenerated / name).read_bytes() != (package / name).read_bytes():
                fail(f"regenerated {name} differs")
    print("NRS v0.3 complete-tree DFS census verification: PASS")


if __name__ == "__main__":
    main()
