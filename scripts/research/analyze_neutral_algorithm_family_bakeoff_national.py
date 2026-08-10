#!/usr/bin/env python3
"""Build deterministic outputs for the scheduled national structure bakeoff."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
from pathlib import Path


PROTOCOL_ID = "neutral-algorithm-family-national-bakeoff-v1"
ANALYSIS_VERSION = "neutral-algorithm-family-national-bakeoff-analysis-v1"
EDGE_CUT_DECIMAL_PLACES = 5
STRUCTURES = (
    "standard-bisect",
    "ratio-optimal",
    "ratio-optimal-area",
    "prime-factor",
)
EXPECTED_MODES = {
    "standard-bisect": "edge-weighted",
    "ratio-optimal": "geosection",
    "ratio-optimal-area": "areasection",
    "prime-factor": "apportion-regions",
}
PILOT_STATES = ("RI", "NE", "CT", "KY", "SC", "WI", "AZ", "CA")
AT_LARGE_EXCLUSIONS = ("AK", "DE", "ND", "SD", "VT", "WY")
STATE_ROWS = (
    ("HI", "hawaii", 2), ("ID", "idaho", 2), ("ME", "maine", 2),
    ("MT", "montana", 2), ("NH", "new_hampshire", 2),
    ("RI", "rhode_island", 2), ("WV", "west_virginia", 2),
    ("NE", "nebraska", 3), ("NM", "new_mexico", 3),
    ("AR", "arkansas", 4), ("IA", "iowa", 4), ("KS", "kansas", 4),
    ("MS", "mississippi", 4), ("NV", "nevada", 4), ("UT", "utah", 4),
    ("CT", "connecticut", 5), ("OK", "oklahoma", 5),
    ("KY", "kentucky", 6), ("LA", "louisiana", 6), ("OR", "oregon", 6),
    ("AL", "alabama", 7), ("SC", "south_carolina", 7),
    ("CO", "colorado", 8), ("MD", "maryland", 8), ("MN", "minnesota", 8),
    ("MO", "missouri", 8), ("WI", "wisconsin", 8),
    ("AZ", "arizona", 9), ("IN", "indiana", 9),
    ("MA", "massachusetts", 9), ("TN", "tennessee", 9),
    ("WA", "washington", 10), ("VA", "virginia", 11),
    ("NJ", "new_jersey", 12), ("MI", "michigan", 13),
    ("GA", "georgia", 14), ("NC", "north_carolina", 14),
    ("OH", "ohio", 15), ("IL", "illinois", 17), ("PA", "pennsylvania", 17),
    ("NY", "new_york", 26), ("FL", "florida", 28),
    ("TX", "texas", 38), ("CA", "california", 52),
)
STATE_BY_CODE = {code: (slug, seats) for code, slug, seats in STATE_ROWS}
FULL_STATES = PILOT_STATES + tuple(
    code for code, _slug, _seats in STATE_ROWS if code not in PILOT_STATES
)
CLAIM_BOUNDARY = (
    "This scheduled package describes reproducible software and graph-partition "
    "mechanics for its frozen phase only. A pilot is not a probability sample "
    "and cannot rank algorithms or establish national prevalence, geometric, "
    "electoral, demographic, VRA, causal, or legal conclusions."
)


class BakeoffError(RuntimeError):
    pass


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def normalize_edge_cut(value: object) -> float:
    """Normalize native floating sums to the bakeoff's semantic precision."""
    return round(float(value), EDGE_CUT_DECIMAL_PLACES)


def canonical_assignment(raw: dict) -> dict[str, int]:
    if not raw:
        raise BakeoffError("empty assignment")
    numeric = {int(unit): int(district) for unit, district in raw.items()}
    members: dict[int, list[int]] = {}
    for unit, district in numeric.items():
        members.setdefault(district, []).append(unit)
    labels = {
        district: index + 1
        for index, district in enumerate(sorted(members, key=lambda d: min(members[d])))
    }
    return {str(unit): labels[numeric[unit]] for unit in sorted(numeric)}


def maximum_weight_assignment(weights: list[list[int]]) -> int:
    """Return an exact maximum-weight square assignment via Hungarian method."""
    size = len(weights)
    if size == 0 or any(len(row) != size for row in weights):
        raise BakeoffError("overlap matrix must be nonempty and square")
    maximum = max(max(row) for row in weights)
    costs = [[maximum - value for value in row] for row in weights]
    u = [0] * (size + 1)
    v = [0] * (size + 1)
    p = [0] * (size + 1)
    way = [0] * (size + 1)
    for row in range(1, size + 1):
        p[0] = row
        column0 = 0
        minimum = [10**30] * (size + 1)
        used = [False] * (size + 1)
        while True:
            used[column0] = True
            row0 = p[column0]
            delta = 10**30
            column1 = 0
            for column in range(1, size + 1):
                if used[column]:
                    continue
                current = costs[row0 - 1][column - 1] - u[row0] - v[column]
                if current < minimum[column]:
                    minimum[column] = current
                    way[column] = column0
                if minimum[column] < delta:
                    delta = minimum[column]
                    column1 = column
            for column in range(size + 1):
                if used[column]:
                    u[p[column]] += delta
                    v[column] -= delta
                else:
                    minimum[column] -= delta
            column0 = column1
            if p[column0] == 0:
                break
        while True:
            column1 = way[column0]
            p[column0] = p[column1]
            column0 = column1
            if column0 == 0:
                break
    return sum(weights[p[column] - 1][column - 1] for column in range(1, size + 1))


def optimal_overlap(left: dict[str, int], right: dict[str, int]) -> dict:
    if set(left) != set(right):
        raise BakeoffError("assignment universes differ")
    left_labels = sorted(set(left.values()))
    right_labels = sorted(set(right.values()))
    if len(left_labels) != len(right_labels):
        raise BakeoffError("district counts differ")
    left_index = {label: index for index, label in enumerate(left_labels)}
    right_index = {label: index for index, label in enumerate(right_labels)}
    weights = [[0] * len(left_labels) for _ in left_labels]
    for unit in left:
        weights[left_index[left[unit]]][right_index[right[unit]]] += 1
    matched = maximum_weight_assignment(weights)
    return {
        "matched_units": matched,
        "different_units": len(left) - matched,
        "matched_unit_rate": matched / len(left),
    }


def structure_row(
    structure_dir: Path, state_code: str, state_slug: str, expected_districts: int,
) -> tuple[dict, dict[str, int]]:
    run = read_json(structure_dir / "run.json")
    structure = run["structure"]
    if run.get("status") != "pass":
        return ({
            "state": state_code,
            "structure": structure,
            "status": "fail",
            "error": run.get("error", "runner failed without an error message"),
        }, {})
    native_root = structure_dir / "native" / "2020" / "states" / state_slug
    manifest = read_json(native_root / "manifest.json")
    audit = read_json(native_root / "audit-certificate.json")
    assignment = canonical_assignment(
        read_json(native_root / "data" / "final_assignments.json")
    )
    canonical_payload = json.dumps(assignment, indent=2).encode() + b"\n"
    (structure_dir / "canonical_assignments.json").write_bytes(canonical_payload)
    districts = len(set(assignment.values()))
    expected_mode = EXPECTED_MODES[structure]
    if manifest.get("partition_mode") != expected_mode:
        raise BakeoffError(
            f"{state_code}/{structure}: expected {expected_mode}, "
            f"got {manifest.get('partition_mode')}"
        )
    failed_checks = [
        {"name": check["name"], "summary": check["summary"]}
        for check in audit.get("checks", []) if check.get("status") == "fail"
    ]
    population_checks = [
        check for check in audit.get("checks", []) if check.get("name") == "population"
    ]
    population_pass = (
        len(population_checks) == 1 and population_checks[0].get("status") == "pass"
    )
    consistent = (
        manifest.get("audit_result") == audit.get("result")
        and manifest.get("population_balance_valid") is population_pass
    )
    valid = (
        audit.get("result") == "pass"
        and population_pass
        and consistent
        and districts == expected_districts
    )
    return ({
        "state": state_code,
        "structure": structure,
        "status": "pass" if valid else "fail",
        "execution_status": "pass",
        "requested_seed": run["requested_seed"],
        "final_seed": manifest.get("seed"),
        "seed_retried": manifest.get("seed") != run["requested_seed"],
        "partition_mode": manifest.get("partition_mode"),
        "audit_result": manifest.get("audit_result"),
        "audit_certificate_result": audit.get("result"),
        "failed_audit_checks": failed_checks,
        "manifest_population_balance_valid": manifest.get("population_balance_valid"),
        "audit_population_pass": population_pass,
        "native_manifest_consistent": consistent,
        "units": len(assignment),
        "districts": districts,
        "expected_districts": expected_districts,
        # Native manifests sum floating edge weights. The last machine-order bit
        # is not a research result, so derived evidence binds five decimals.
        "edge_cut": normalize_edge_cut(manifest["edge_cut"]),
        "canonical_assignment_sha256": sha256_bytes(canonical_payload),
        "adjacency_sha256": manifest.get("adjacency_sha256"),
        "binary_sha256": manifest.get("binary_sha256"),
        "population_source": manifest.get("population_source"),
        "balance_tolerance_pct": manifest.get("balance_tolerance_pct"),
        "ufactor": manifest.get("ufactor"),
        "niter": manifest.get("niter"),
        "alpha_county": manifest.get("alpha_county"),
        "directional_lambda": manifest.get("directional_lambda"),
    }, assignment)


def common_invariants(rows: list[dict]) -> dict:
    passing = [row for row in rows if row.get("execution_status") == "pass"]
    fields = (
        "adjacency_sha256", "binary_sha256", "population_source",
        "balance_tolerance_pct", "units", "districts", "expected_districts",
        "ufactor", "niter", "alpha_county", "directional_lambda",
    )
    values = {
        field: sorted({str(row.get(field)) for row in passing}) for field in fields
    }
    return {
        "pass": bool(passing) and all(len(value) == 1 for value in values.values()),
        "values": values,
    }


def analyze_state(package: Path, code: str) -> dict:
    slug, seats = STATE_BY_CODE[code]
    state_dir = package / "states" / code.lower()
    rows = []
    assignments = {}
    for structure in STRUCTURES:
        row, assignment = structure_row(
            state_dir / "structures" / structure, code, slug, seats
        )
        rows.append(row)
        if assignment:
            assignments[structure] = assignment
    pairs = []
    names = list(assignments)
    for left_index, left in enumerate(names):
        for right in names[left_index + 1:]:
            pairs.append({
                "state": code,
                "left": left,
                "right": right,
                **optimal_overlap(assignments[left], assignments[right]),
            })
    standard_cut = next(
        (row["edge_cut"] for row in rows if row["structure"] == "standard-bisect"
         and row.get("execution_status") == "pass"),
        None,
    )
    for row in rows:
        row["edge_cut_ratio_to_standard"] = (
            round(row["edge_cut"] / standard_cut, 12)
            if standard_cut and row.get("execution_status") == "pass" else None
        )
    invariants = common_invariants(rows)
    passed = (
        len(assignments) == len(STRUCTURES)
        and invariants["pass"]
        and all(row["status"] == "pass" for row in rows)
    )
    result = {
        "state": code,
        "state_slug": slug,
        "expected_districts": seats,
        "status": "pass" if passed else "fail",
        "structures": rows,
        "common_input_invariants": invariants,
        "pairwise_assignment_overlap": pairs,
    }
    (state_dir / "analysis.json").write_text(
        json.dumps(result, indent=2) + "\n", encoding="utf-8"
    )
    return result


def write_csv(path: Path, fieldnames: list[str], rows: list[dict]) -> None:
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)


def write_derived(package: Path, phase: str) -> dict:
    codes = PILOT_STATES if phase == "pilot" else FULL_STATES
    states = [analyze_state(package, code) for code in codes]
    state_rows = [{
        "state": state["state"],
        "status": state["status"],
        "expected_districts": state["expected_districts"],
        "units": next(
            (row.get("units") for row in state["structures"]
             if row.get("execution_status") == "pass"), None
        ),
        "structures_passed": sum(row["status"] == "pass" for row in state["structures"]),
    } for state in states]
    cell_rows = [row for state in states for row in state["structures"]]
    pair_rows = [row for state in states for row in state["pairwise_assignment_overlap"]]
    passed_states = sum(row["status"] == "pass" for row in state_rows)
    passed_cells = sum(row["status"] == "pass" for row in cell_rows)
    passed = passed_states == len(codes) and passed_cells == len(codes) * len(STRUCTURES)
    analysis = {
        "schema_version": ANALYSIS_VERSION,
        "protocol_id": PROTOCOL_ID,
        "phase": phase,
        "status": "pass" if passed else "fail",
        "scope": {
            "year": 2020,
            "chamber": "congressional",
            "scheduled_states": list(codes),
            "scheduled_state_count": len(codes),
            "scheduled_cell_count": len(codes) * len(STRUCTURES),
            "national_target_state_count": len(FULL_STATES),
            "at_large_design_exclusions": list(AT_LARGE_EXCLUSIONS),
        },
        "frozen_controls": {
            "partition_preset": "edge-weighted",
            "weights_override": "geographic",
            "outer_search": "single",
            "requested_initial_seed": 0,
        },
        "result_counts": {
            "states_passed": passed_states,
            "states_scheduled": len(codes),
            "cells_passed": passed_cells,
            "cells_scheduled": len(codes) * len(STRUCTURES),
        },
        "states": states,
        "claim_boundary": CLAIM_BOUNDARY,
    }
    (package / "analysis.json").write_text(
        json.dumps(analysis, indent=2) + "\n", encoding="utf-8"
    )
    write_csv(
        package / "state-summary.csv",
        ["state", "status", "expected_districts", "units", "structures_passed"],
        state_rows,
    )
    write_csv(
        package / "cell-summary.csv",
        ["state", "structure", "status", "requested_seed", "final_seed",
         "seed_retried", "partition_mode", "audit_result", "audit_population_pass",
         "manifest_population_balance_valid", "native_manifest_consistent", "units",
         "districts", "edge_cut", "edge_cut_ratio_to_standard",
         "canonical_assignment_sha256"],
        cell_rows,
    )
    write_csv(
        package / "pairwise-overlap.csv",
        ["state", "left", "right", "matched_units", "different_units",
         "matched_unit_rate"],
        pair_rows,
    )
    readme = f"""# Neutral Algorithm-Family Bakeoff: {phase.title()} Phase

**Status:** {analysis['status']}

**States:** {passed_states}/{len(codes)} passed

**State-structure cells:** {passed_cells}/{len(codes) * len(STRUCTURES)} passed

This package executes the frozen `{phase}` schedule under
`neutral-algorithm-family-national-bakeoff-v1`. State-level validity and
within-State comparisons are in the CSV and JSON outputs. The six at-large
States are design exclusions, not passes.

## Claim boundary

{CLAIM_BOUNDARY}
"""
    (package / "README.md").write_text(readme, encoding="utf-8")
    return analysis


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    parser.add_argument("--phase", choices=("pilot", "full"), default="pilot")
    args = parser.parse_args()
    analysis = write_derived(args.package.resolve(), args.phase)
    print(f"national algorithm-family {args.phase} analysis: {analysis['status'].upper()}")


if __name__ == "__main__":
    main()
