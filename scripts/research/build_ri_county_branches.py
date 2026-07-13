#!/usr/bin/env python3
"""Build an exhaustive five-branch Rhode Island county population split."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCHEMA = "ri-county-population-branches-v1"
OUTSIDE_COUNTIES = ("001", "003", "005", "009")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def county_units(instance: dict) -> dict[str, list[int]]:
    result: dict[str, list[int]] = {}
    for unit, geoid in enumerate(instance["unit_ids"]):
        result.setdefault(geoid[2:5], []).append(unit)
    return result


def branch_definitions(instance: dict) -> list[dict[str, object]]:
    units = county_units(instance)
    branches: list[dict[str, object]] = [
        {
            "branch_id": "outside-zero",
            "constraints": [
                {
                    "region_id": county,
                    "units": units[county],
                    "relation": "equal",
                    "population": 0,
                }
                for county in OUTSIDE_COUNTIES
            ],
            "meaning": "No right-child population outside Providence County.",
        }
    ]
    for index, county in enumerate(OUTSIDE_COUNTIES):
        constraints = [
            {
                "region_id": previous,
                "units": units[previous],
                "relation": "equal",
                "population": 0,
            }
            for previous in OUTSIDE_COUNTIES[:index]
        ]
        constraints.append(
            {
                "region_id": county,
                "units": units[county],
                "relation": "at-least",
                "population": 1,
            }
        )
        branches.append(
            {
                "branch_id": f"first-positive-{county}",
                "constraints": constraints,
                "meaning": (
                    f"{county} is the first ordered non-Providence county "
                    "with positive right-child population."
                ),
            }
        )
    return branches


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--instance", type=Path, required=True)
    parser.add_argument("--discovery", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--right-population", type=int, required=True)
    args = parser.parse_args()

    instance = json.loads(args.instance.read_text(encoding="utf-8"))
    out_dir = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    branches = branch_definitions(instance)
    central_fixed_labels = [
        0 if geoid[2:5] != "007" and population > 0 else None
        for geoid, population in zip(
            instance["unit_ids"], instance["populations"], strict=True
        )
    ]
    central_fixed_path = out_dir / "outside-zero-fixed-labels.json"
    central_fixed_path.write_text(
        json.dumps(central_fixed_labels, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    artifacts = []
    for branch in branches:
        branch_dir = out_dir / str(branch["branch_id"])
        constraints_path = branch_dir / "regional-constraints.json"
        branch_dir.mkdir(parents=True, exist_ok=True)
        constraints_path.write_text(
            json.dumps(branch["constraints"], indent=2) + "\n", encoding="utf-8"
        )
        subprocess.run(
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "bisect-ilp",
                "--example",
                "certified_regional_branch_model",
                "--",
                str(args.instance.resolve()),
                str(args.discovery.resolve()),
                str(args.right_population),
                str(constraints_path),
                str(branch_dir),
            ],
            cwd=ROOT,
            check=True,
        )
        artifacts.append(
            {
                "branch_id": branch["branch_id"],
                "meaning": branch["meaning"],
                "constraints_sha256": sha256(constraints_path),
                "opb_sha256": sha256(branch_dir / "boundary.opb"),
                "request_sha256": sha256(branch_dir / "request.json"),
            }
        )
    manifest = {
        "schema_version": SCHEMA,
        "instance_sha256": sha256(args.instance),
        "discovery_sha256": sha256(args.discovery),
        "right_population": args.right_population,
        "branch_count": len(artifacts),
        "branches": artifacts,
        "central_fixed_labels_sha256": sha256(central_fixed_path),
        "coverage": (
            "Branches are disjoint and exhaustive because outside-county "
            "populations are nonnegative: either all four are zero, or one "
            "county is the first ordered county with positive population."
        ),
        "claim_boundary": "Hash-bound proof inputs only; no branch has been classified.",
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    print(f"RI county branches: {len(artifacts)} exhaustive models")


if __name__ == "__main__":
    main()
