#!/usr/bin/env python3
"""Build the exact pure-tract branch and its fixed-label complement contract."""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path


SCHEMA = "ri-tract-purity-branch-v1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_fixed_labels(instance: dict, assignment: list[int]) -> tuple[list[int | None], dict]:
    tract_labels: dict[str, set[int]] = defaultdict(set)
    for unit, (geoid, population) in enumerate(
        zip(instance["unit_ids"], instance["populations"], strict=True)
    ):
        if geoid[2:5] == "007" and population > 0:
            tract_labels[geoid[:11]].add(assignment[unit])
    pure_tracts = {
        tract: next(iter(labels))
        for tract, labels in tract_labels.items()
        if len(labels) == 1
    }
    fixed: list[int | None] = [None] * len(assignment)
    for unit, (geoid, population) in enumerate(
        zip(instance["unit_ids"], instance["populations"], strict=True)
    ):
        if population <= 0:
            continue
        if geoid[2:5] != "007":
            fixed[unit] = 0
        elif geoid[:11] in pure_tracts:
            fixed[unit] = pure_tracts[geoid[:11]]
    report = {
        "pure_tract_count": len(pure_tracts),
        "split_tract_count": len(tract_labels) - len(pure_tracts),
        "fixed_unit_count": sum(label is not None for label in fixed),
        "active_unit_count": sum(label is None for label in fixed),
        "pure_tract_labels": dict(sorted(pure_tracts.items())),
    }
    return fixed, report


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--instance", type=Path, required=True)
    parser.add_argument("--discovery", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    args = parser.parse_args()
    instance = json.loads(args.instance.read_text(encoding="utf-8"))
    discovery = json.loads(args.discovery.read_text(encoding="utf-8"))
    fixed, report = build_fixed_labels(
        instance, discovery["objective"]["canonical_assignment"]
    )
    args.out_dir.mkdir(parents=True, exist_ok=True)
    labels_path = args.out_dir / "fixed-labels.json"
    labels_path.write_text(
        json.dumps(fixed, separators=(",", ":")) + "\n", encoding="utf-8"
    )
    package = {
        "schema_version": SCHEMA,
        "instance_sha256": sha256(args.instance),
        "discovery_sha256": sha256(args.discovery),
        **report,
        "fixed_labels_sha256": sha256(labels_path),
        "coverage": (
            "The central branch preserves all positive-population labels outside "
            "Providence County and in incumbent-pure Providence tracts. Its exact "
            "complement requires at least one such label to change."
        ),
        "claim_boundary": "Branch contract only; no optimality claim.",
    }
    (args.out_dir / "manifest.json").write_text(
        json.dumps(package, indent=2) + "\n", encoding="utf-8"
    )
    print(
        f"RI tract purity branch: {report['fixed_unit_count']} fixed, "
        f"{report['active_unit_count']} active"
    )


if __name__ == "__main__":
    main()
