#!/usr/bin/env python3
"""Build deterministic derived outputs for the neutral algorithm-family bakeoff."""

from __future__ import annotations

import argparse
import csv
import json
from functools import lru_cache
from pathlib import Path


PROTOCOL_ID = "neutral-algorithm-family-bakeoff-v1"
ANALYSIS_VERSION = "neutral-algorithm-family-bakeoff-analysis-v1"
STRUCTURES = (
    "standard-bisect",
    "ratio-optimal",
    "ratio-optimal-area",
    "prime-factor",
)
EXPECTED_MODES = {
    # The native manifest retains the preset label for the standard override.
    # The governed command is therefore required to disambiguate this row.
    "standard-bisect": "edge-weighted",
    "ratio-optimal": "geosection",
    "ratio-optimal-area": "areasection",
    "prime-factor": "apportion-regions",
}
CLAIM_BOUNDARY = (
    "This Wisconsin tract-level proof slice describes reproducible software and "
    "graph-partition mechanics only. It does not identify a generally best "
    "algorithm, isolate a causal structure effect, or establish national, "
    "geometric-compactness, electoral, VRA, or legal conclusions."
)


class BakeoffError(RuntimeError):
    pass


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def canonical_assignment(raw: dict) -> dict[str, int]:
    """Sort unit keys and relabel districts by their smallest member unit."""
    if not raw:
        raise BakeoffError("empty assignment")
    numeric = {int(unit): int(district) for unit, district in raw.items()}
    by_district: dict[int, list[int]] = {}
    for unit, district in numeric.items():
        by_district.setdefault(district, []).append(unit)
    ordered = sorted(by_district, key=lambda district: min(by_district[district]))
    labels = {district: index + 1 for index, district in enumerate(ordered)}
    return {str(unit): labels[numeric[unit]] for unit in sorted(numeric)}


def optimal_overlap(left: dict[str, int], right: dict[str, int]) -> dict:
    """Maximum-overlap label matching using a deterministic subset DP."""
    if set(left) != set(right):
        raise BakeoffError("assignment universes differ")
    left_labels = sorted(set(left.values()))
    right_labels = sorted(set(right.values()))
    if len(left_labels) != len(right_labels):
        raise BakeoffError("district counts differ")
    size = len(left_labels)
    left_index = {label: index for index, label in enumerate(left_labels)}
    right_index = {label: index for index, label in enumerate(right_labels)}
    weights = [[0] * size for _ in range(size)]
    for unit in left:
        weights[left_index[left[unit]]][right_index[right[unit]]] += 1

    @lru_cache(maxsize=None)
    def best(row: int, used: int) -> int:
        if row == size:
            return 0
        return max(
            weights[row][column] + best(row + 1, used | (1 << column))
            for column in range(size)
            if not used & (1 << column)
        )

    matched = best(0, 0)
    total = len(left)
    return {
        "matched_units": matched,
        "different_units": total - matched,
        "matched_unit_rate": matched / total,
    }


def structure_row(structure_dir: Path, structure: str) -> tuple[dict, dict[str, int]]:
    run = read_json(structure_dir / "run.json")
    if run.get("status") != "pass":
        return (
            {
                "structure": structure,
                "status": "fail",
                "error": run.get("error", "runner failed without an error message"),
            },
            {},
        )
    native_root = structure_dir / "native" / "2020" / "states" / "wisconsin"
    manifest = read_json(native_root / "manifest.json")
    audit = read_json(native_root / "audit-certificate.json")
    assignment = canonical_assignment(read_json(native_root / "data" / "final_assignments.json"))
    (structure_dir / "canonical_assignments.json").write_text(
        json.dumps(assignment, indent=2) + "\n", encoding="utf-8"
    )
    districts = len(set(assignment.values()))
    expected_mode = EXPECTED_MODES[structure]
    if manifest.get("partition_mode") != expected_mode:
        raise BakeoffError(
            f"{structure}: expected native mode {expected_mode}, "
            f"got {manifest.get('partition_mode')}"
        )
    failed_checks = [
        {"name": check["name"], "summary": check["summary"]}
        for check in audit.get("checks", [])
        if check.get("status") == "fail"
    ]
    population_checks = [
        check for check in audit.get("checks", []) if check.get("name") == "population"
    ]
    audit_population_pass = (
        len(population_checks) == 1 and population_checks[0].get("status") == "pass"
    )
    audit_result_consistent = manifest.get("audit_result") == audit.get("result")
    population_flag_consistent = (
        manifest.get("population_balance_valid") is audit_population_pass
    )
    valid = (
        audit.get("result") == "pass"
        and audit_result_consistent
        and population_flag_consistent
    )
    return (
        {
            "structure": structure,
            "status": "pass" if valid else "fail",
            "execution_status": "pass",
            "requested_seed": run["requested_seed"],
            "final_seed": manifest.get("seed"),
            "seed_retried": manifest.get("seed") != run["requested_seed"],
            "partition_mode": manifest["partition_mode"],
            "audit_result": manifest.get("audit_result"),
            "audit_certificate_result": audit.get("result"),
            "failed_audit_checks": failed_checks,
            "manifest_population_balance_valid": manifest.get("population_balance_valid"),
            "audit_population_pass": audit_population_pass,
            "native_manifest_consistent": audit_result_consistent and population_flag_consistent,
            "units": len(assignment),
            "districts": districts,
            # Native parallel reduction can vary below 1e-9 with an identical
            # assignment. Six decimals preserves the reported weighted metric
            # while making the governed derivative byte-stable.
            "edge_cut": round(float(manifest["edge_cut"]), 6),
            "adjacency_sha256": manifest.get("adjacency_sha256"),
            "binary_sha256": manifest.get("binary_sha256"),
            "population_source": manifest.get("population_source"),
            "balance_tolerance_pct": manifest.get("balance_tolerance_pct"),
            "ufactor": manifest.get("ufactor"),
            "niter": manifest.get("niter"),
            "alpha_county": manifest.get("alpha_county"),
            "directional_lambda": manifest.get("directional_lambda"),
        },
        assignment,
    )


def common_invariants(rows: list[dict]) -> dict:
    passing = [row for row in rows if row.get("execution_status") == "pass"]
    fields = (
        "adjacency_sha256",
        "binary_sha256",
        "population_source",
        "balance_tolerance_pct",
        "units",
        "districts",
        "ufactor",
        "niter",
        "alpha_county",
        "directional_lambda",
    )
    values = {field: sorted({str(row[field]) for row in passing}) for field in fields}
    return {
        "pass": bool(passing) and all(len(value) == 1 for value in values.values()),
        "values": values,
    }


def write_derived(package: Path) -> dict:
    rows: list[dict] = []
    assignments: dict[str, dict[str, int]] = {}
    for structure in STRUCTURES:
        row, assignment = structure_row(package / "structures" / structure, structure)
        rows.append(row)
        if assignment:
            assignments[structure] = assignment

    pairs = []
    names = list(assignments)
    for left_index, left in enumerate(names):
        for right in names[left_index + 1 :]:
            pairs.append(
                {"left": left, "right": right, **optimal_overlap(assignments[left], assignments[right])}
            )

    invariants = common_invariants(rows)
    all_pass = (
        len(assignments) == len(STRUCTURES)
        and invariants["pass"]
        and all(
            row["status"] == "pass"
            for row in rows
        )
    )
    analysis = {
        "schema_version": ANALYSIS_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "pass" if all_pass else "fail",
        "scope": {"state": "WI", "year": 2020, "chamber": "congressional", "districts": 8},
        "frozen_controls": {
            "partition_preset": "edge-weighted",
            "weights_override": "geographic",
            "outer_search": "single",
            "requested_initial_seed": 0,
        },
        "structures": rows,
        "common_input_invariants": invariants,
        "pairwise_assignment_overlap": pairs,
        "claim_boundary": CLAIM_BOUNDARY,
    }
    (package / "analysis.json").write_text(
        json.dumps(analysis, indent=2) + "\n", encoding="utf-8"
    )

    completed_rows = [row for row in rows if row.get("execution_status") == "pass"]
    with (package / "structure-summary.csv").open("w", newline="", encoding="utf-8") as handle:
        fieldnames = [
            "structure", "status", "requested_seed", "final_seed", "seed_retried",
            "partition_mode", "audit_result", "audit_population_pass",
            "manifest_population_balance_valid", "native_manifest_consistent", "units",
            "districts", "edge_cut",
        ]
        writer = csv.DictWriter(handle, fieldnames=fieldnames, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)
    with (package / "pairwise-overlap.csv").open("w", newline="", encoding="utf-8") as handle:
        fieldnames = ["left", "right", "matched_units", "different_units", "matched_unit_rate"]
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(pairs)

    table_rows = "\n".join(
        f"| {row['structure']} | {row['partition_mode']} | {row['final_seed']} | "
        f"{row['edge_cut']:.6f} | {row['audit_result']} |" for row in completed_rows
    )
    readme = f"""# Neutral Algorithm-Family Bakeoff: Wisconsin 2020

**Status:** {analysis['status']}

This preregistered proof slice holds the tract input and neutral controls fixed
while running four implemented BISECT structure families.

| Structure | Effective native mode | Final seed | Weighted edge cut | Audit |
|---|---|---:|---:|---|
{table_rows}

Requested seed was `0`; `analysis.json` reports every automatic balance retry.
Pairwise assignment agreement after maximum-overlap district-label matching is
in `pairwise-overlap.csv`.

## Claim boundary

{CLAIM_BOUNDARY}
"""
    (package / "README.md").write_text(readme, encoding="utf-8")
    return analysis


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    analysis = write_derived(args.package.resolve())
    print(f"neutral algorithm-family analysis: {analysis['status'].upper()}")


if __name__ == "__main__":
    main()
