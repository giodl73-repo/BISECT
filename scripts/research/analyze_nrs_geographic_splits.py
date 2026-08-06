#!/usr/bin/env python3
"""Build the governed NRS v0.3 county/tract split-audit package."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
from collections import defaultdict
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
PROTOCOL_PATH = Path(
    "docs/specs/2026-08-06-nrs-v0.3-national-geographic-split-audit-protocol.md"
)
ANALYZER_PATH = Path("scripts/research/analyze_nrs_geographic_splits.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_geographic_splits.py")
BISECT_SHA256 = "2bcf6b13f17f237db6f755943ea1ccdac0d2e0267395c616892c6e46ce66e90e"
YEARS = (2000, 2010, 2020)
LEVELS = ("county", "tract")
STRUCTURAL_TREE_EXCEPTIONS = {(2010, "MD")}
CLAIM_BOUNDARY = (
    "Complete, hash-bound county and tract intersection counts for the governed "
    "NRS v0.3 assignments in 2000, 2010, and 2020; no compactness-superiority, "
    "municipality, community, demographic, partisan, VRA, legal-validity, "
    "cross-cycle-improvement, optimality, or adoption claim."
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def tree_snapshot_projection(tree: dict[str, Any]) -> dict[str, Any]:
    nodes = []
    for node in tree["nodes"]:
        bound = node["generation_tolerance_scaled_bound"]
        deviation = node["objective"]["max_population_deviation_scaled"]
        population = node["parent_population"]
        boundary = node["objective"]["weighted_boundary_cut"]
        nodes.append(
            {
                "child_seats": node["child_seats"],
                "depth": len(node["path"]),
                "generation_tolerance_scaled_bound": bound,
                "max_population_deviation_scaled": deviation,
                "parent_population": population,
                "path": node["path"],
                "population_floor_attained": node["population_floor"]["attained"],
                "population_floor_lower_bound": node["population_floor"]["lower_bound"],
                "seats": node["seats"],
                "tolerance_usage": deviation / bound if bound else 0.0,
                "weighted_boundary_cut": boundary,
                "weighted_boundary_cut_per_parent_person": boundary / population,
            }
        )
    return {
        "districts": tree["districts"],
        "leaf_paths": [leaf["path"] for leaf in tree["leaves"]],
        "nodes": nodes,
        "population_total": tree["population_total"],
        "state": tree["state"],
        "unit_count": tree["unit_count"],
    }


def parse_cycles(values: list[str]) -> dict[int, Path]:
    cycles: dict[int, Path] = {}
    for value in values:
        year_text, separator, directory = value.partition("=")
        require(separator == "=" and year_text.isdigit() and directory, f"invalid --cycle: {value}")
        year = int(year_text)
        require(year in YEARS and year not in cycles, f"unexpected or duplicate cycle: {year}")
        cycles[year] = Path(directory).resolve()
    require(tuple(sorted(cycles)) == YEARS, "exactly one --cycle is required for 2000, 2010, and 2020")
    return cycles


def metric_row(
    year: int,
    state: str,
    districts: int,
    level: str,
    geographies: dict[str, set[int]],
    block_counts: dict[str, int],
) -> dict[str, Any]:
    counts = [len(labels) for labels in geographies.values()]
    total = len(counts)
    split = sum(count > 1 for count in counts)
    source_blocks = sum(block_counts.values())
    return {
        "census_year": year,
        "state": state,
        "state_districts": districts,
        "level": level,
        "total_geographies": total,
        "split_geographies": split,
        "unsplit_geographies": total - split,
        "split_fraction": split / total if total else 0.0,
        "district_geography_pieces": sum(counts),
        "excess_pieces": sum(count - 1 for count in counts),
        "max_districts_in_geography": max(counts, default=0),
        "source_blocks": source_blocks,
    }


def aggregate_metrics(rows: list[dict[str, Any]], multi_only: bool) -> dict[str, dict[str, Any]]:
    aggregate: dict[str, dict[str, Any]] = {}
    for level in LEVELS:
        selected = [
            row
            for row in rows
            if row["level"] == level and (not multi_only or row["state_districts"] > 1)
        ]
        total = sum(row["total_geographies"] for row in selected)
        split = sum(row["split_geographies"] for row in selected)
        aggregate[level] = {
            "states": len(selected),
            "districts": sum(row["state_districts"] for row in selected),
            "total_geographies": total,
            "split_geographies": split,
            "unsplit_geographies": total - split,
            "split_fraction": split / total if total else 0.0,
            "district_geography_pieces": sum(
                row["district_geography_pieces"] for row in selected
            ),
            "excess_pieces": sum(row["excess_pieces"] for row in selected),
            "max_districts_in_geography": max(
                (row["max_districts_in_geography"] for row in selected), default=0
            ),
            "source_blocks": sum(row["source_blocks"] for row in selected),
        }
    return aggregate


def readiness_matrix() -> list[dict[str, str]]:
    return [
        {
            "metric_family": "population-and-contiguity",
            "status": "complete-in-national-baselines",
            "reason": "All assignments, districts, recursive children, and population tolerances were independently verified in the governed national packages.",
        },
        {
            "metric_family": "county-and-tract-splits",
            "status": "complete-in-this-package",
            "reason": "Every block GEOID deterministically projects to a county and tract prefix, and every geography-to-district set is published.",
        },
        {
            "metric_family": "municipality-splits",
            "status": "not-computed",
            "reason": "No uniform hash-bound block-to-municipality input is frozen for all three cycles.",
        },
        {
            "metric_family": "geometric-compactness",
            "status": "not-computed",
            "reason": "District dissolve, projection, water, and perimeter rules are not yet precommitted and versioned.",
        },
        {
            "metric_family": "racial-and-language-opportunity",
            "status": "not-computed",
            "reason": "Retained nationwide demographic tables are tract-level and no frozen within-tract allocation rule is available for split tracts.",
        },
        {
            "metric_family": "partisan-diagnostics",
            "status": "not-computed",
            "reason": "The retained national election allocation does not provide two presidential and two House elections with a published precinct-to-block crosswalk.",
        },
        {
            "metric_family": "100-seed-sensitivity",
            "status": "not-computed",
            "reason": "The diagnostic seed profile and 100 governed runs have not been executed.",
        },
        {
            "metric_family": "ensemble-percentiles",
            "status": "not-computed",
            "reason": "No four-chain national ensemble satisfying the frozen convergence and effective-sample-size rules has been produced.",
        },
    ]


def render_readme(analysis: dict[str, Any]) -> str:
    rows = [
        "| Cycle | Counties split | County excess pieces | Tracts split | Tract excess pieces |",
        "|---:|---:|---:|---:|---:|",
    ]
    for cycle in analysis["cycles"]:
        all_states = cycle["national"]["all_states"]
        rows.append(
            f"| {cycle['census_year']} | {all_states['county']['split_geographies']:,} / "
            f"{all_states['county']['total_geographies']:,} | {all_states['county']['excess_pieces']:,} | "
            f"{all_states['tract']['split_geographies']:,} / {all_states['tract']['total_geographies']:,} | "
            f"{all_states['tract']['excess_pieces']:,} |"
        )
    readiness = [
        "| Metric family | Status |",
        "|---|---|",
        *[
            f"| {item['metric_family']} | `{item['status']}` |"
            for item in analysis["evaluation_readiness"]
        ],
    ]
    return (
        "# NRS v0.3 National Geographic Split Audit\n\n"
        "**Status:** governed analyzer output with independent aggregate verification\n\n"
        + "\n".join(rows)
        + "\n\nRaw counts are descriptive within each Census geography vintage. They are not "
        "cross-cycle improvement measures because county and tract definitions and "
        "district allocations change. Every county and tract district set is in "
        "`geography-projection.csv`; every State source assignment hash and committed "
        "recursive-structure snapshot is bound in `analysis.json`. Raw tree transport "
        "hashes match in 149 of 150 State-cycle packages; Maryland 2010 uses the "
        "protocol's metadata-only diagnostic exception.\n\n## Evaluation readiness\n\n"
        + "\n".join(readiness)
        + "\n\nUnavailable metrics were not replaced with zeros or post-outcome proxies. See "
        "`analysis.json` for the exact reason attached to each status.\n\n"
        "## Rebuild and verify\n\n```powershell\n"
        "python scripts/research/analyze_nrs_geographic_splits.py `\n"
        "  --cycle 2000=<run-dir-2000> --cycle 2010=<run-dir-2010> `\n"
        "  --cycle 2020=<run-dir-2020> --out-dir <new-output-directory>\n"
        "python scripts/research/verify_nrs_geographic_splits.py <new-output-directory>\n"
        "```\n\n## Claim boundary\n\n"
        + analysis["claim_boundary"]
        + "\n"
    )


def build(cycles: dict[int, Path], out_dir: Path) -> None:
    require(not out_dir.exists(), f"output directory already exists: {out_dir}")
    require(sha256(ROOT / "target/release/bisect.exe") == BISECT_SHA256, "bisect executable hash drift")
    for path in (ROOT / PROTOCOL_PATH, ROOT / ANALYZER_PATH, ROOT / VERIFIER_PATH):
        require(path.is_file(), f"missing governed source: {path}")
    out_dir.mkdir(parents=True)
    projection_path = out_dir / "geography-projection.csv"
    metrics_path = out_dir / "state-metrics.csv"
    analysis_cycles: list[dict[str, Any]] = []
    all_metric_rows: list[dict[str, Any]] = []
    with projection_path.open("w", encoding="utf-8", newline="") as projection_file:
        projection_writer = csv.DictWriter(
            projection_file,
            fieldnames=[
                "census_year", "state", "level", "geoid", "district_count",
                "district_labels", "split", "source_block_count",
            ],
            lineterminator="\n",
        )
        projection_writer.writeheader()
        for year, run_dir in sorted(cycles.items()):
            summary_path = ROOT / f"docs/experiments/nrs-v0.3-national-{year}/national-summary.json"
            summary = load(summary_path)
            expected_states = {row["state"]: row for row in summary["states"]}
            require(len(expected_states) == 50, f"{year}: committed State universe")
            snapshot_path = ROOT / f"docs/experiments/nrs-cross-decade-2000-2020/node-snapshot-{year}.json"
            snapshot = load(snapshot_path)
            expected_snapshots = {row["state"]: row for row in snapshot["states"]}
            require(set(expected_snapshots) == set(expected_states), f"{year}: structural snapshot State universe")
            inventory = load(ROOT / f"docs/experiments/nationwide-{year}/inventory.json")
            expected_fips = {row["state"]: row["fips"] for row in inventory["states"]}
            require(set(expected_fips) == set(expected_states), f"{year}: inventory State universe")
            state_sources: list[dict[str, Any]] = []
            cycle_metrics: list[dict[str, Any]] = []
            for state, expected in sorted(expected_states.items()):
                package = run_dir / "states" / state.lower() / "package"
                assignment_path = package / "baseline_assignments.json"
                tree_path = package / "baseline-tree.json"
                package_manifest_path = package / "baseline_manifest.json"
                for path in (assignment_path, tree_path, package_manifest_path):
                    require(path.is_file(), f"{year}/{state}: missing {path.name}")
                tree_hash = sha256(tree_path)
                package_manifest = load(package_manifest_path)
                artifacts = {row["path"]: row["sha256"] for row in package_manifest["artifacts"]}
                require(artifacts["baseline-tree.json"] == tree_hash, f"{year}/{state}: package tree hash")
                tree = load(tree_path)
                projected = tree_snapshot_projection(tree)
                expected_snapshot = dict(expected_snapshots[state])
                expected_snapshot.pop("baseline_tree_sha256")
                require(projected == expected_snapshot, f"{year}/{state}: committed structural snapshot")
                exception = (year, state) in STRUCTURAL_TREE_EXCEPTIONS
                if not exception:
                    require(tree_hash == expected["baseline_tree_sha256"], f"{year}/{state}: governed tree hash mismatch")
                assignment_hash = sha256(assignment_path)
                require(artifacts["baseline_assignments.json"] == assignment_hash, f"{year}/{state}: package assignment hash")
                assignment_package = load(assignment_path)
                require(assignment_package["schema_version"] == "nrs-baseline-assignments-v0.1-v1", f"{year}/{state}: assignment schema")
                require(assignment_package["canonical_order"] == "sorted-geoid", f"{year}/{state}: assignment order")
                require(assignment_package["label_base"] == 1, f"{year}/{state}: label base")
                assignments = assignment_package["assignments"]
                require(len(assignments) == expected["unit_count"], f"{year}/{state}: unit count")
                require(list(assignments) == sorted(assignments), f"{year}/{state}: noncanonical assignment order")
                labels = set(assignments.values())
                require(labels == set(range(1, expected["districts"] + 1)), f"{year}/{state}: district labels")
                geographies = {level: defaultdict(set) for level in LEVELS}
                block_counts = {level: defaultdict(int) for level in LEVELS}
                for geoid, district in assignments.items():
                    require(len(geoid) == 15 and geoid.isdigit(), f"{year}/{state}: invalid block GEOID")
                    require(geoid[:2] == expected_fips[state], f"{year}/{state}: State prefix")
                    for level, prefix in (("county", 5), ("tract", 11)):
                        geography = geoid[:prefix]
                        geographies[level][geography].add(district)
                        block_counts[level][geography] += 1
                for level in LEVELS:
                    for geography, district_set in sorted(geographies[level].items()):
                        projection_writer.writerow(
                            {
                                "census_year": year,
                                "state": state,
                                "level": level,
                                "geoid": geography,
                                "district_count": len(district_set),
                                "district_labels": ";".join(map(str, sorted(district_set))),
                                "split": str(len(district_set) > 1).lower(),
                                "source_block_count": block_counts[level][geography],
                            }
                        )
                    row = metric_row(
                        year, state, expected["districts"], level,
                        geographies[level], block_counts[level],
                    )
                    cycle_metrics.append(row)
                    all_metric_rows.append(row)
                state_sources.append(
                    {
                        "state": state,
                        "state_fips": expected_fips[state],
                        "districts": expected["districts"],
                        "unit_count": expected["unit_count"],
                        "population_total": expected["population_total"],
                        "assignment_sha256": assignment_hash,
                        "committed_tree_sha256": expected["baseline_tree_sha256"],
                        "tree_validation": (
                            "structural-snapshot-exception-2010-md"
                            if exception else "exact-raw-sha256"
                        ),
                        "source_package_verified": True,
                    }
                )
            require(sum(row["districts"] for row in state_sources) == 435, f"{year}: district total")
            analysis_cycles.append(
                {
                    "census_year": year,
                    "committed_summary_path": str(summary_path.relative_to(ROOT)).replace("\\", "/"),
                    "committed_summary_sha256": sha256(summary_path),
                    "committed_node_snapshot_path": str(snapshot_path.relative_to(ROOT)).replace("\\", "/"),
                    "committed_node_snapshot_sha256": sha256(snapshot_path),
                    "state_count": 50,
                    "district_count": 435,
                    "unit_count": sum(row["unit_count"] for row in state_sources),
                    "population_total": sum(row["population_total"] for row in state_sources),
                    "source_states": state_sources,
                    "national": {
                        "all_states": aggregate_metrics(cycle_metrics, False),
                        "multi_district_states": aggregate_metrics(cycle_metrics, True),
                    },
                }
            )
    with metrics_path.open("w", encoding="utf-8", newline="") as metrics_file:
        fields = [
            "census_year", "state", "state_districts", "level",
            "total_geographies", "split_geographies", "unsplit_geographies",
            "split_fraction", "district_geography_pieces", "excess_pieces",
            "max_districts_in_geography", "source_blocks",
        ]
        writer = csv.DictWriter(metrics_file, fieldnames=fields, lineterminator="\n")
        writer.writeheader()
        for row in all_metric_rows:
            stored = dict(row)
            stored["split_fraction"] = f"{row['split_fraction']:.12f}"
            writer.writerow(stored)
    analysis = {
        "schema_version": "nrs-v0.3-national-geographic-split-analysis-v2",
        "protocol_id": "nrs-v0.3-national-geographic-split-audit-v2",
        "protocol_path": str(PROTOCOL_PATH).replace("\\", "/"),
        "bisect_executable_sha256": BISECT_SHA256,
        "cycles": analysis_cycles,
        "evaluation_readiness": readiness_matrix(),
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = out_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8")
    readme_path = out_dir / "README.md"
    readme_path.write_text(render_readme(analysis), encoding="utf-8")
    manifest = {
        "schema_version": "nrs-v0.3-national-geographic-split-package-v2",
        "package_id": "nrs-v0.3-national-geographic-splits",
        "protocol_path": str(PROTOCOL_PATH).replace("\\", "/"),
        "protocol_sha256": sha256(ROOT / PROTOCOL_PATH),
        "analyzer_path": str(ANALYZER_PATH).replace("\\", "/"),
        "analyzer_sha256": sha256(ROOT / ANALYZER_PATH),
        "verifier_path": str(VERIFIER_PATH).replace("\\", "/"),
        "verifier_sha256": sha256(ROOT / VERIFIER_PATH),
        "files": {
            path.name: sha256(path)
            for path in (readme_path, analysis_path, projection_path, metrics_path)
        },
        "claim_boundary": CLAIM_BOUNDARY,
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print("NRS v0.3 national geographic split analysis: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cycle", action="append", required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    args = parser.parse_args()
    try:
        build(parse_cycles(args.cycle), args.out_dir.resolve())
    except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(f"FAIL: {error}") from error


if __name__ == "__main__":
    main()
