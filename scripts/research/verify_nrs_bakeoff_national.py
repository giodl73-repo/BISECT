#!/usr/bin/env python3
"""Verify the governed national NRS v0.3 Tier 1 bakeoff package."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import subprocess
import sys
from pathlib import Path


MANIFEST_VERSION = "nrs-v0.3-national-bakeoff-manifest-v1"
ANALYSIS_VERSION = "nrs-v0.3-national-bakeoff-analysis-v1"


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
        fail("unsupported manifest schema")
    if analysis.get("schema_version") != ANALYSIS_VERSION:
        fail("unsupported analysis schema")
    if manifest.get("status") != "pass" or analysis.get("status") != "pass":
        fail("national package is not complete")

    for row in manifest["code"]:
        path = root / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            fail(f"code hash mismatch for {row['path']}")
    for name, expected in manifest["outputs"].items():
        path = package / name
        if not path.is_file() or sha256(path) != expected:
            fail(f"output hash mismatch for {name}")
    for row in manifest["state_manifests"]:
        path = package / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            fail(f"state manifest hash mismatch for {row['state']}")
        completed = subprocess.run(
            [
                sys.executable,
                str(root / "scripts/research/verify_nrs_bakeoff_slice.py"),
                str(path.parent),
            ],
            cwd=root,
            check=False,
        )
        if completed.returncode != 0:
            fail(f"State regeneration failed for {row['state']}")

    with (package / "state-summary.csv").open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if len(rows) != 50 or {row["state"] for row in rows} != {
        row["state"] for row in manifest["state_manifests"]
    }:
        fail("State summary universe")

    integer_state_fields = {
        "districts": ("benchmark", "districts"),
        "source_blocks": ("atomic_universe", "source_blocks"),
        "excluded_water_only_blocks": (
            "atomic_universe",
            "excluded_water_only_blocks",
        ),
        "blocks": ("benchmark", "blocks"),
        "matched_blocks": ("comparison", "matched_blocks"),
        "moved_blocks": ("comparison", "moved_blocks"),
    }
    for row in rows:
        state_analysis = json.loads(
            (package / "states" / row["state"].lower() / "analysis.json").read_text(
                encoding="utf-8"
            )
        )
        for csv_field, keys in integer_state_fields.items():
            expected = state_analysis[keys[0]][keys[1]]
            if int(row[csv_field]) != expected:
                fail(f"State summary mismatch for {row['state']} {csv_field}")
        split_fields = {
            "nrs_county_splits": state_analysis["benchmark"]["county_splits"][
                "split_units"
            ],
            "comparator_county_splits": state_analysis["comparator"][
                "county_splits"
            ]["split_units"],
            "county_split_difference": state_analysis["comparison"][
                "comparator_minus_benchmark"
            ]["county_split_units"],
            "nrs_tract_splits": state_analysis["benchmark"]["tract_splits"][
                "split_units"
            ],
            "comparator_tract_splits": state_analysis["comparator"][
                "tract_splits"
            ]["split_units"],
            "tract_split_difference": state_analysis["comparison"][
                "comparator_minus_benchmark"
            ]["tract_split_units"],
        }
        for csv_field, expected in split_fields.items():
            if int(row[csv_field]) != expected:
                fail(f"State summary mismatch for {row['state']} {csv_field}")
        if abs(
            float(row["matched_block_rate"])
            - state_analysis["comparison"]["matched_block_rate"]
        ) > 1e-15:
            fail(f"State summary mismatch for {row['state']} matched_block_rate")

    integer_fields = [
        "districts",
        "source_blocks",
        "excluded_water_only_blocks",
        "blocks",
        "matched_blocks",
        "moved_blocks",
        "nrs_county_splits",
        "comparator_county_splits",
        "nrs_tract_splits",
        "comparator_tract_splits",
    ]
    totals = analysis["national_totals"]
    expected_fields = {
        "districts": "districts",
        "source_blocks": "source_blocks",
        "excluded_water_only_blocks": "excluded_water_only_blocks",
        "blocks": "blocks",
        "matched_blocks": "matched_blocks",
        "moved_blocks": "moved_blocks",
        "nrs_county_splits": "nrs_county_splits",
        "comparator_county_splits": "comparator_county_splits",
        "nrs_tract_splits": "nrs_tract_splits",
        "comparator_tract_splits": "comparator_tract_splits",
    }
    for output_field, csv_field in expected_fields.items():
        actual = sum(int(row[csv_field]) for row in rows)
        if actual != totals[output_field]:
            fail(f"national total mismatch for {output_field}")
    if totals["districts"] != 435 or totals["source_blocks"] != 8_126_956:
        fail("expected national universe")
    if (
        totals["source_blocks"] - totals["excluded_water_only_blocks"]
        != totals["blocks"]
    ):
        fail("land-containing block universe")
    if totals["matched_blocks"] + totals["moved_blocks"] != totals["blocks"]:
        fail("block overlap accounting")
    if abs(
        totals["matched_block_rate"]
        - totals["matched_blocks"] / totals["blocks"]
    ) > 1e-15:
        fail("matched block rate")
    if any(int(row[field]) < 0 for row in rows for field in integer_fields):
        fail("negative count in State summary")

    print("NRS v0.3 national bakeoff verification: PASS")


if __name__ == "__main__":
    main()
