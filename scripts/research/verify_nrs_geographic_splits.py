#!/usr/bin/env python3
"""Independently verify the compact NRS v0.3 geographic split package."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
from collections import defaultdict
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_PACKAGE = ROOT / "docs/experiments/nrs-v0.3-national-geographic-splits"
YEARS = (2000, 2010, 2020)
LEVELS = ("county", "tract")
STRUCTURAL_TREE_EXCEPTIONS = {(2010, "MD")}
BISECT_SHA256 = "2bcf6b13f17f237db6f755943ea1ccdac0d2e0267395c616892c6e46ce66e90e"
PROTOCOL_PATH = "docs/specs/2026-08-06-nrs-v0.3-national-geographic-split-audit-protocol.md"
ANALYZER_PATH = "scripts/research/analyze_nrs_geographic_splits.py"
VERIFIER_PATH = "scripts/research/verify_nrs_geographic_splits.py"
PACKAGE_FILES = {
    "README.md",
    "analysis.json",
    "geography-projection.csv",
    "state-metrics.csv",
}
CLAIM_BOUNDARY = (
    "Complete, hash-bound county and tract intersection counts for the governed "
    "NRS v0.3 assignments in 2000, 2010, and 2020; no compactness-superiority, "
    "municipality, community, demographic, partisan, VRA, legal-validity, "
    "cross-cycle-improvement, optimality, or adoption claim."
)
READINESS = [
    (
        "population-and-contiguity",
        "complete-in-national-baselines",
        "All assignments, districts, recursive children, and population tolerances were independently verified in the governed national packages.",
    ),
    (
        "county-and-tract-splits",
        "complete-in-this-package",
        "Every block GEOID deterministically projects to a county and tract prefix, and every geography-to-district set is published.",
    ),
    (
        "municipality-splits",
        "not-computed",
        "No uniform hash-bound block-to-municipality input is frozen for all three cycles.",
    ),
    (
        "geometric-compactness",
        "not-computed",
        "District dissolve, projection, water, and perimeter rules are not yet precommitted and versioned.",
    ),
    (
        "racial-and-language-opportunity",
        "not-computed",
        "Retained nationwide demographic tables are tract-level and no frozen within-tract allocation rule is available for split tracts.",
    ),
    (
        "partisan-diagnostics",
        "not-computed",
        "The retained national election allocation does not provide two presidential and two House elections with a published precinct-to-block crosswalk.",
    ),
    (
        "100-seed-sensitivity",
        "not-computed",
        "The diagnostic seed profile and 100 governed runs have not been executed.",
    ),
    (
        "ensemble-percentiles",
        "not-computed",
        "No four-chain national ensemble satisfying the frozen convergence and effective-sample-size rules has been produced.",
    ),
]


def fail(message: str) -> None:
    raise SystemExit(f"FAIL: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def empty_metric() -> dict[str, Any]:
    return {
        "total_geographies": 0,
        "split_geographies": 0,
        "unsplit_geographies": 0,
        "district_geography_pieces": 0,
        "excess_pieces": 0,
        "max_districts_in_geography": 0,
        "source_blocks": 0,
        "labels": set(),
    }


def aggregate(
    state_metrics: dict[tuple[int, str, str], dict[str, Any]],
    sources: dict[tuple[int, str], dict[str, Any]],
    year: int,
    multi_only: bool,
) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for level in LEVELS:
        rows = [
            row
            for (row_year, state, row_level), row in state_metrics.items()
            if row_year == year
            and row_level == level
            and (not multi_only or sources[(year, state)]["districts"] > 1)
        ]
        total = sum(row["total_geographies"] for row in rows)
        split = sum(row["split_geographies"] for row in rows)
        states = {
            state
            for (row_year, state, row_level) in state_metrics
            if row_year == year
            and row_level == level
            and (not multi_only or sources[(year, state)]["districts"] > 1)
        }
        result[level] = {
            "states": len(states),
            "districts": sum(sources[(year, state)]["districts"] for state in states),
            "total_geographies": total,
            "split_geographies": split,
            "unsplit_geographies": total - split,
            "split_fraction": split / total if total else 0.0,
            "district_geography_pieces": sum(
                row["district_geography_pieces"] for row in rows
            ),
            "excess_pieces": sum(row["excess_pieces"] for row in rows),
            "max_districts_in_geography": max(
                (row["max_districts_in_geography"] for row in rows), default=0
            ),
            "source_blocks": sum(row["source_blocks"] for row in rows),
        }
    return result


def compare_metric(actual: dict[str, Any], expected: dict[str, Any], label: str) -> None:
    require(set(actual) == set(expected), f"{label}: metric fields")
    for key, expected_value in expected.items():
        actual_value = actual[key]
        if isinstance(expected_value, float):
            require(
                math.isclose(float(actual_value), expected_value, rel_tol=0, abs_tol=5e-13),
                f"{label}: {key}",
            )
        else:
            require(actual_value == expected_value, f"{label}: {key}")


def verify(package: Path) -> None:
    manifest = load(package / "manifest.json")
    require(
        manifest["schema_version"] == "nrs-v0.3-national-geographic-split-package-v2",
        "manifest schema",
    )
    require(manifest["package_id"] == "nrs-v0.3-national-geographic-splits", "package id")
    require(manifest["claim_boundary"] == CLAIM_BOUNDARY, "manifest claim boundary")
    require(manifest["protocol_path"] == PROTOCOL_PATH, "manifest protocol path")
    require(manifest["analyzer_path"] == ANALYZER_PATH, "manifest analyzer path")
    require(manifest["verifier_path"] == VERIFIER_PATH, "manifest verifier path")
    require(set(manifest["files"]) == PACKAGE_FILES, "manifest package-file universe")
    for path_key, hash_key in (
        ("protocol_path", "protocol_sha256"),
        ("analyzer_path", "analyzer_sha256"),
        ("verifier_path", "verifier_sha256"),
    ):
        path = ROOT / manifest[path_key]
        require(path.is_file(), f"missing governed source: {path}")
        require(sha256(path) == manifest[hash_key], f"governed source hash: {path}")
    for relative, expected in manifest["files"].items():
        path = package / relative
        require(path.is_file(), f"missing package file: {relative}")
        require(sha256(path) == expected, f"package hash: {relative}")

    analysis = load(package / "analysis.json")
    require(
        analysis["schema_version"] == "nrs-v0.3-national-geographic-split-analysis-v2",
        "analysis schema",
    )
    require(analysis["protocol_id"] == "nrs-v0.3-national-geographic-split-audit-v2", "protocol id")
    require(analysis["protocol_path"] == manifest["protocol_path"], "protocol path")
    require(analysis["bisect_executable_sha256"] == BISECT_SHA256, "bisect hash")
    require(analysis["claim_boundary"] == CLAIM_BOUNDARY, "analysis claim boundary")
    cycles = {cycle["census_year"]: cycle for cycle in analysis["cycles"]}
    require(tuple(sorted(cycles)) == YEARS and len(cycles) == 3, "cycle universe")

    sources: dict[tuple[int, str], dict[str, Any]] = {}
    for year, cycle in cycles.items():
        summary_path = ROOT / cycle["committed_summary_path"]
        require(summary_path.is_file(), f"{year}: missing committed summary")
        require(sha256(summary_path) == cycle["committed_summary_sha256"], f"{year}: summary hash")
        snapshot_path = ROOT / cycle["committed_node_snapshot_path"]
        require(snapshot_path.is_file(), f"{year}: missing committed node snapshot")
        require(sha256(snapshot_path) == cycle["committed_node_snapshot_sha256"], f"{year}: node snapshot hash")
        snapshot = load(snapshot_path)
        summary = load(summary_path)
        committed = {row["state"]: row for row in summary["states"]}
        source_rows = {row["state"]: row for row in cycle["source_states"]}
        require(len(committed) == len(source_rows) == cycle["state_count"] == 50, f"{year}: States")
        require(set(committed) == set(source_rows), f"{year}: State source universe")
        require({row["state"] for row in snapshot["states"]} == set(source_rows), f"{year}: snapshot State universe")
        require(cycle["district_count"] == sum(row["districts"] for row in source_rows.values()) == 435, f"{year}: districts")
        require(cycle["unit_count"] == sum(row["unit_count"] for row in source_rows.values()), f"{year}: units")
        require(cycle["population_total"] == sum(row["population_total"] for row in source_rows.values()), f"{year}: population")
        for state, source in source_rows.items():
            expected = committed[state]
            require(source["state_fips"].isdigit() and len(source["state_fips"]) == 2, f"{year}/{state}: FIPS")
            require(source["districts"] == expected["districts"], f"{year}/{state}: districts")
            require(source["unit_count"] == expected["unit_count"], f"{year}/{state}: units")
            require(source["population_total"] == expected["population_total"], f"{year}/{state}: population")
            require(source["committed_tree_sha256"] == expected["baseline_tree_sha256"], f"{year}/{state}: tree binding")
            expected_validation = (
                "structural-snapshot-exception-2010-md"
                if (year, state) in STRUCTURAL_TREE_EXCEPTIONS
                else "exact-raw-sha256"
            )
            require(source["tree_validation"] == expected_validation, f"{year}/{state}: tree validation")
            require(source["source_package_verified"] is True, f"{year}/{state}: package verification")
            for key in ("assignment_sha256", "committed_tree_sha256"):
                require(len(source[key]) == 64 and all(character in "0123456789abcdef" for character in source[key]), f"{year}/{state}: {key}")
            sources[(year, state)] = source
    require(len(sources) == 150, "150 State sources")

    derived: defaultdict[tuple[int, str, str], dict[str, Any]] = defaultdict(empty_metric)
    geography_rows: dict[tuple[int, str, str, str], tuple[set[int], int]] = {}
    previous_key: tuple[int, str, str, str] | None = None
    with (package / "geography-projection.csv").open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        require(
            reader.fieldnames
            == [
                "census_year", "state", "level", "geoid", "district_count",
                "district_labels", "split", "source_block_count",
            ],
            "projection columns",
        )
        for line_number, row in enumerate(reader, 2):
            year = int(row["census_year"])
            state = row["state"]
            level = row["level"]
            geoid = row["geoid"]
            key = (year, state, level, geoid)
            require(previous_key is None or key > previous_key, f"projection order at line {line_number}")
            previous_key = key
            require((year, state) in sources and level in LEVELS, f"projection universe at line {line_number}")
            expected_length = 5 if level == "county" else 11
            require(len(geoid) == expected_length and geoid.isdigit(), f"projection GEOID at line {line_number}")
            require(geoid[:2] == sources[(year, state)]["state_fips"], f"projection State prefix at line {line_number}")
            labels = [int(value) for value in row["district_labels"].split(";")]
            require(labels == sorted(set(labels)) and labels, f"projection labels at line {line_number}")
            require(labels == [label for label in labels if 1 <= label <= sources[(year, state)]["districts"]], f"projection label range at line {line_number}")
            district_count = int(row["district_count"])
            split = row["split"] == "true"
            require(row["split"] in {"true", "false"}, f"projection split at line {line_number}")
            require(district_count == len(labels), f"projection district count at line {line_number}")
            require(split is (district_count > 1), f"projection split status at line {line_number}")
            blocks = int(row["source_block_count"])
            require(blocks > 0, f"projection block count at line {line_number}")
            geography_rows[key] = (set(labels), blocks)
            metric = derived[(year, state, level)]
            metric["total_geographies"] += 1
            metric["split_geographies"] += int(split)
            metric["unsplit_geographies"] += int(not split)
            metric["district_geography_pieces"] += district_count
            metric["excess_pieces"] += district_count - 1
            metric["max_districts_in_geography"] = max(
                metric["max_districts_in_geography"], district_count
            )
            metric["source_blocks"] += blocks
            metric["labels"].update(labels)
    require(len(derived) == 300, "300 State/level projections")
    for (year, state, level), metric in derived.items():
        source = sources[(year, state)]
        require(metric["source_blocks"] == source["unit_count"], f"{year}/{state}/{level}: block coverage")
        require(metric["labels"] == set(range(1, source["districts"] + 1)), f"{year}/{state}/{level}: district coverage")
    for (year, state), source in sources.items():
        county_rows = {
            geoid: value
            for (row_year, row_state, level, geoid), value in geography_rows.items()
            if row_year == year and row_state == state and level == "county"
        }
        tract_rows = {
            geoid: value
            for (row_year, row_state, level, geoid), value in geography_rows.items()
            if row_year == year and row_state == state and level == "tract"
        }
        tract_blocks: defaultdict[str, int] = defaultdict(int)
        tract_labels: defaultdict[str, set[int]] = defaultdict(set)
        for tract, (labels, blocks) in tract_rows.items():
            tract_blocks[tract[:5]] += blocks
            tract_labels[tract[:5]].update(labels)
        require(set(tract_blocks) == set(county_rows), f"{year}/{state}: county/tract universe")
        for county, (labels, blocks) in county_rows.items():
            require(tract_blocks[county] == blocks, f"{year}/{state}/{county}: county/tract block consistency")
            require(tract_labels[county] == labels, f"{year}/{state}/{county}: county/tract label consistency")

    metric_rows: dict[tuple[int, str, str], dict[str, Any]] = {}
    with (package / "state-metrics.csv").open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        expected_fields = [
            "census_year", "state", "state_districts", "level",
            "total_geographies", "split_geographies", "unsplit_geographies",
            "split_fraction", "district_geography_pieces", "excess_pieces",
            "max_districts_in_geography", "source_blocks",
        ]
        require(reader.fieldnames == expected_fields, "State metric columns")
        for row in reader:
            key = (int(row["census_year"]), row["state"], row["level"])
            require(key not in metric_rows and key in derived, f"duplicate or unknown State metric: {key}")
            source = sources[(key[0], key[1])]
            expected = dict(derived[key])
            expected.pop("labels")
            expected["split_fraction"] = (
                expected["split_geographies"] / expected["total_geographies"]
            )
            actual = {
                "total_geographies": int(row["total_geographies"]),
                "split_geographies": int(row["split_geographies"]),
                "unsplit_geographies": int(row["unsplit_geographies"]),
                "split_fraction": float(row["split_fraction"]),
                "district_geography_pieces": int(row["district_geography_pieces"]),
                "excess_pieces": int(row["excess_pieces"]),
                "max_districts_in_geography": int(row["max_districts_in_geography"]),
                "source_blocks": int(row["source_blocks"]),
            }
            require(int(row["state_districts"]) == source["districts"], f"{key}: State districts")
            compare_metric(actual, expected, f"{key}: State metric")
            actual["state_districts"] = source["districts"]
            metric_rows[key] = actual
    require(len(metric_rows) == 300, "300 State metric rows")

    for year, cycle in cycles.items():
        expected_all = aggregate(metric_rows, sources, year, False)
        expected_multi = aggregate(metric_rows, sources, year, True)
        for level in LEVELS:
            compare_metric(cycle["national"]["all_states"][level], expected_all[level], f"{year}/all/{level}")
            compare_metric(cycle["national"]["multi_district_states"][level], expected_multi[level], f"{year}/multi/{level}")

    expected_readiness = [
        {"metric_family": name, "status": status, "reason": reason}
        for name, status, reason in READINESS
    ]
    require(analysis["evaluation_readiness"] == expected_readiness, "evaluation-readiness matrix")

    readme = (package / "README.md").read_text(encoding="utf-8")
    expected_headlines = []
    for year in YEARS:
        all_states = cycles[year]["national"]["all_states"]
        expected_headlines.append(
            f"| {year} | {all_states['county']['split_geographies']:,} / "
            f"{all_states['county']['total_geographies']:,} | "
            f"{all_states['county']['excess_pieces']:,} | "
            f"{all_states['tract']['split_geographies']:,} / "
            f"{all_states['tract']['total_geographies']:,} | "
            f"{all_states['tract']['excess_pieces']:,} |"
        )
    require(all(readme.count(row) == 1 for row in expected_headlines), "README headline drift")
    require(readme.count(CLAIM_BOUNDARY) == 1, "README claim boundary")
    print(
        "NRS v0.3 national geographic split independent verification: PASS "
        f"({sum(row['total_geographies'] for row in metric_rows.values()):,} State/level geographies)"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", nargs="?", type=Path, default=DEFAULT_PACKAGE)
    args = parser.parse_args()
    try:
        verify(args.package.resolve())
    except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(f"FAIL: {error}") from error


if __name__ == "__main__":
    main()
