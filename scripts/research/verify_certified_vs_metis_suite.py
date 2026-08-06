#!/usr/bin/env python3
"""Independently verify the precommitted certified-versus-METIS suite."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from collections import defaultdict, deque
from pathlib import Path
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / "docs/experiments/certified-vs-metis-multi-instance"
SEEDS = [1, 7, 42, 2020, 314159]
CLAIM_BOUNDARY = (
    "Eight precommitted bounded synthetic instances and five fixed seeds only; "
    "no State-scale, national, map-quality, fairness, VRA, legal-validity, or "
    "adoption claim."
)


def fail(message: str) -> None:
    raise SystemExit(f"FAIL: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_hash(value: Any) -> str:
    encoded = json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def edge(left: int, right: int, weight: int = 1) -> dict[str, int]:
    return {"left": min(left, right), "right": max(left, right), "weight": weight}


def path_edges(units: int) -> list[dict[str, int]]:
    return [edge(left, left + 1) for left in range(units - 1)]


def cycle_edges(
    units: int, weight: Callable[[int, int], int] = lambda _left, _right: 1
) -> list[dict[str, int]]:
    pairs = [(left, left + 1) for left in range(units - 1)] + [(0, units - 1)]
    return [edge(left, right, weight(left, right)) for left, right in sorted(pairs)]


def ladder_edges(columns: int) -> list[dict[str, int]]:
    pairs = []
    for row in range(2):
        pairs.extend(
            (row * columns + column, row * columns + column + 1)
            for column in range(columns - 1)
        )
    pairs.extend((column, columns + column) for column in range(columns))
    return [edge(left, right) for left, right in sorted(pairs)]


def grid_edges(
    rows: int,
    columns: int,
    weight: Callable[[int, int], int] = lambda _left, _right: 1,
) -> list[dict[str, int]]:
    pairs = []
    for row in range(rows):
        for column in range(columns):
            unit = row * columns + column
            if column + 1 < columns:
                pairs.append((unit, unit + 1))
            if row + 1 < rows:
                pairs.append((unit, unit + columns))
    return [edge(left, right, weight(left, right)) for left, right in sorted(pairs)]


def barbell_edges() -> list[dict[str, int]]:
    pairs = []
    for start in (0, 6):
        pairs.extend(
            (left, right)
            for left in range(start, start + 6)
            for right in range(left + 1, start + 6)
        )
    return sorted(
        [edge(left, right) for left, right in pairs] + [edge(5, 6, 3)],
        key=lambda item: (item["left"], item["right"]),
    )


def fixture_definitions() -> dict[str, tuple[list[int], list[dict[str, int]], int]]:
    weighted_grid_edges = {(1, 2), (5, 6), (9, 10)}
    return {
        "path8-equal": ([100] * 8, path_edges(8), 4),
        "cycle10-varied": (
            [80, 120, 90, 110, 95, 105, 85, 115, 100, 100],
            cycle_edges(10),
            4,
        ),
        "ladder2x6-varied": (
            [90, 110, 95, 105, 85, 115, 120, 80, 100, 100, 108, 92],
            ladder_edges(6),
            4,
        ),
        "grid3x4-weighted": (
            [100] * 12,
            grid_edges(
                3,
                4,
                lambda left, right: 5 if (left, right) in weighted_grid_edges else 1,
            ),
            4,
        ),
        "barbell12-bridge": (
            [95, 105, 100, 100, 90, 110, 110, 90, 100, 100, 105, 95],
            barbell_edges(),
            4,
        ),
        "tree13-unequal": (
            [70, 130, 90, 110, 80, 120, 100, 95, 105, 85, 115, 75, 125],
            [edge((child - 1) // 2, child) for child in range(1, 13)],
            5,
        ),
        "grid4x4-unequal": (
            [82, 118, 91, 109, 97, 103, 86, 114, 121, 79, 106, 94, 88, 112, 99, 101],
            grid_edges(4, 4),
            5,
        ),
        "cycle20-equal": (
            [90 if unit % 2 == 0 else 110 for unit in range(20)],
            cycle_edges(
                20,
                lambda left, right: 2 if left % 5 == 0 or right % 5 == 0 else 1,
            ),
            6,
        ),
    }


def expected_instance(
    populations: list[int], edges: list[dict[str, int]], k_parent: int
) -> dict[str, Any]:
    unit_ids = [f"u{unit:02}" for unit in range(len(populations))]
    k_left = k_parent // 2
    k_right = k_parent - k_left
    return {
        "schema_version": "certified-recursive-bisection-split-instance-v1",
        "model_id": "certified-standard-bisect-split-v1",
        "node_path": "",
        "parent_certificate_id": None,
        "unit_universe_hash": canonical_hash({"unit_ids": unit_ids}),
        "unit_ids": unit_ids,
        "populations": populations,
        "edges": edges,
        "k_parent": k_parent,
        "k_left": k_left,
        "k_right": k_right,
        "orientation_rule": (
            "equal-seats-unit-zero-left"
            if k_left == k_right
            else "seat-ordered-floor-left-ceil-right"
        ),
    }


def objective(instance: dict[str, Any], assignment: list[int]) -> dict[str, int]:
    populations = instance["populations"]
    child_populations = [0, 0]
    for unit, label in enumerate(assignment):
        child_populations[label] += populations[unit]
    parent_population = sum(child_populations)
    deviations = [
        abs(instance["k_parent"] * child_populations[0] - instance["k_left"] * parent_population),
        abs(instance["k_parent"] * child_populations[1] - instance["k_right"] * parent_population),
    ]
    boundary = sum(
        item["weight"]
        for item in instance["edges"]
        if assignment[item["left"]] != assignment[item["right"]]
    )
    return {
        "max_population_deviation_scaled": max(deviations),
        "total_population_deviation_scaled": sum(deviations),
        "weighted_boundary_cut": boundary,
    }


def connected(instance: dict[str, Any], assignment: list[int]) -> bool:
    adjacency = [[] for _ in assignment]
    for item in instance["edges"]:
        adjacency[item["left"]].append(item["right"])
        adjacency[item["right"]].append(item["left"])
    for label in (0, 1):
        allowed = {unit for unit, value in enumerate(assignment) if value == label}
        if not allowed:
            return False
        visited = {next(iter(allowed))}
        queue = deque(visited)
        while queue:
            unit = queue.popleft()
            for neighbor in adjacency[unit]:
                if neighbor in allowed and neighbor not in visited:
                    visited.add(neighbor)
                    queue.append(neighbor)
        if visited != allowed:
            return False
    return True


def sufficient(instance: dict[str, Any], assignment: list[int]) -> bool:
    left = assignment.count(0)
    return left >= instance["k_left"] and len(assignment) - left >= instance["k_right"]


def check_assignment_shape(instance: dict[str, Any], assignment: Any, label: str) -> None:
    require(isinstance(assignment, list), f"{label}: assignment is not a list")
    require(len(assignment) == len(instance["unit_ids"]), f"{label}: assignment length")
    require(set(assignment) == {0, 1}, f"{label}: assignment labels")
    if instance["k_left"] == instance["k_right"]:
        require(assignment[0] == 0, f"{label}: equal-seat orientation")


def empty_summary() -> dict[str, int]:
    return {
        "total_rows": 0,
        "ok": 0,
        "errors": 0,
        "disconnected": 0,
        "insufficient_child_units": 0,
        "exact_assignment_agreements": 0,
        "exact_primary_objective_agreements": 0,
        "exact_population_objective_agreements": 0,
    }


def update_summary(summary: dict[str, int], row: dict[str, Any]) -> None:
    summary["total_rows"] += 1
    status_keys = {
        "ok": "ok",
        "error": "errors",
        "disconnected": "disconnected",
        "insufficient-child-units": "insufficient_child_units",
    }
    require(row["status"] in status_keys, f"unknown row status: {row['status']}")
    summary[status_keys[row["status"]]] += 1
    summary["exact_assignment_agreements"] += row["matches_exact_assignment"] is True
    summary["exact_primary_objective_agreements"] += row["matches_exact_primary_objective"] is True
    summary["exact_population_objective_agreements"] += row["matches_exact_population_objective"] is True


def verify(package: Path = PACKAGE) -> None:
    manifest = json.loads((package / "manifest.json").read_text(encoding="utf-8"))
    require(
        manifest["schema_version"] == "certified-vs-metis-multi-instance-package-v1",
        "manifest schema",
    )
    for path_key, hash_key in (
        ("protocol_path", "protocol_sha256"),
        ("generator_path", "generator_sha256"),
        ("verifier_path", "verifier_sha256"),
    ):
        path = ROOT / manifest[path_key]
        require(path.is_file(), f"missing bound source: {path}")
        require(sha256(path) == manifest[hash_key], f"bound source hash: {path}")
    for relative, expected in manifest["files"].items():
        require(sha256(package / relative) == expected, f"package hash: {relative}")

    report = json.loads((package / "comparison-suite.json").read_text(encoding="utf-8"))
    require(report["schema_version"] == "certified-vs-metis-multi-instance-report-v1", "report schema")
    require(report["protocol_id"] == "certified-vs-metis-multi-instance-v1", "protocol id")
    require(report["protocol_path"] == manifest["protocol_path"], "protocol path")
    require(report["metis_seeds"] == SEEDS, "METIS seed list")
    require(report["metis_ufactor"] == 1.1, "METIS ufactor")
    require(report["metis_niter"] == 10, "METIS niter")
    require(report["claim_boundary"] == CLAIM_BOUNDARY, "claim boundary drift")
    environment = report["execution_environment"]
    require(environment["operating_system"], "operating system not recorded")
    require(environment["architecture"], "architecture not recorded")
    require(environment["build_profile"] in {"debug", "release"}, "build profile")
    require(environment["rustc_version"], "rustc version not recorded")
    require(environment["machine_name"], "machine name not recorded")
    require(environment["processor"], "processor not recorded")

    definitions = fixture_definitions()
    instances = {row["fixture_id"]: row for row in report["instances"]}
    require(len(instances) == len(report["instances"]) == 8, "instance universe")
    require(set(instances) == set(definitions), "precommitted fixtures")
    exact_by_fixture: dict[str, dict[str, Any]] = {}
    for fixture_id, (populations, edges, k_parent) in definitions.items():
        row = instances[fixture_id]
        expected = expected_instance(populations, edges, k_parent)
        require(row["instance"] == expected, f"{fixture_id}: reconstructed instance")
        require(row["instance_hash"] == canonical_hash(expected), f"{fixture_id}: instance hash")
        require(row["unit_count"] == len(populations), f"{fixture_id}: unit count")
        require(row["edge_count"] == len(edges), f"{fixture_id}: edge count")
        exact = row["exact"]
        assignment = exact["canonical_assignment"]
        check_assignment_shape(expected, assignment, f"{fixture_id}: exact")
        require(sufficient(expected, assignment), f"{fixture_id}: exact child units")
        require(connected(expected, assignment), f"{fixture_id}: exact connectivity")
        require(exact["objective"] == objective(expected, assignment), f"{fixture_id}: exact objective")
        candidate_count = (
            (1 << (len(populations) - 1)) - 1
            if expected["k_left"] == expected["k_right"]
            else (1 << len(populations)) - 2
        )
        require(exact["candidate_count"] == candidate_count, f"{fixture_id}: candidate count")
        require(0 < exact["feasible_count"] <= candidate_count, f"{fixture_id}: feasible count")
        require(exact["primary_objective_ties"] > 0, f"{fixture_id}: tie count")
        require(exact["bounded_verifier_passed"] is True, f"{fixture_id}: bounded verifier")
        require(exact["proof_id"].startswith("sha256:") and len(exact["proof_id"]) == 71, f"{fixture_id}: proof id")
        require(exact["search_commitment"].startswith("sha256:") and len(exact["search_commitment"]) == 71, f"{fixture_id}: search commitment")
        require(math.isfinite(exact["elapsed_milliseconds"]) and exact["elapsed_milliseconds"] >= 0, f"{fixture_id}: exact timing")
        exact_by_fixture[fixture_id] = exact

    rows = report["metis_rows"]
    expected_keys = {(fixture, seed) for fixture in definitions for seed in SEEDS}
    actual_keys = {(row["fixture_id"], row["seed"]) for row in rows}
    require(len(rows) == len(actual_keys) == 40 and actual_keys == expected_keys, "40 precommitted rows")
    overall = empty_summary()
    by_instance: defaultdict[str, dict[str, int]] = defaultdict(empty_summary)
    by_seed: defaultdict[str, dict[str, int]] = defaultdict(empty_summary)
    for row in rows:
        fixture_id = row["fixture_id"]
        instance = instances[fixture_id]["instance"]
        exact = exact_by_fixture[fixture_id]
        require(math.isfinite(row["elapsed_milliseconds"]) and row["elapsed_milliseconds"] >= 0, f"{fixture_id}/{row['seed']}: timing")
        if row["status"] == "error":
            require(isinstance(row["error"], str) and row["error"], f"{fixture_id}/{row['seed']}: error text")
            for field in (
                "assignment", "objective", "connected", "child_units_sufficient",
                "matches_exact_assignment", "matches_exact_primary_objective",
                "matches_exact_population_objective", "weighted_boundary_difference",
            ):
                require(row[field] is None, f"{fixture_id}/{row['seed']}: error field {field}")
        else:
            assignment = row["assignment"]
            check_assignment_shape(instance, assignment, f"{fixture_id}/{row['seed']}")
            has_units = sufficient(instance, assignment)
            require(row["child_units_sufficient"] is has_units, f"{fixture_id}/{row['seed']}: sufficiency")
            if not has_units:
                require(row["status"] == "insufficient-child-units", f"{fixture_id}/{row['seed']}: insufficient status")
                for field in (
                    "objective", "connected", "matches_exact_assignment",
                    "matches_exact_primary_objective", "matches_exact_population_objective",
                    "weighted_boundary_difference",
                ):
                    require(row[field] is None, f"{fixture_id}/{row['seed']}: insufficient field {field}")
            else:
                observed_objective = objective(instance, assignment)
                observed_connected = connected(instance, assignment)
                require(row["objective"] == observed_objective, f"{fixture_id}/{row['seed']}: objective")
                require(row["connected"] is observed_connected, f"{fixture_id}/{row['seed']}: connectivity")
                require(row["status"] == ("ok" if observed_connected else "disconnected"), f"{fixture_id}/{row['seed']}: status")
                assignment_match = assignment == exact["canonical_assignment"]
                primary_match = observed_objective == exact["objective"]
                population_match = all(
                    observed_objective[key] == exact["objective"][key]
                    for key in ("max_population_deviation_scaled", "total_population_deviation_scaled")
                )
                require(row["matches_exact_assignment"] is assignment_match, f"{fixture_id}/{row['seed']}: assignment agreement")
                require(row["matches_exact_primary_objective"] is primary_match, f"{fixture_id}/{row['seed']}: objective agreement")
                require(row["matches_exact_population_objective"] is population_match, f"{fixture_id}/{row['seed']}: population agreement")
                require(
                    row["weighted_boundary_difference"]
                    == observed_objective["weighted_boundary_cut"] - exact["objective"]["weighted_boundary_cut"],
                    f"{fixture_id}/{row['seed']}: boundary difference",
                )
        update_summary(overall, row)
        update_summary(by_instance[fixture_id], row)
        update_summary(by_seed[str(row["seed"])], row)

    expected_aggregate = {
        "total_precommitted_rows": 40,
        "seed_invariant_within_fixture": all(
            len(
                {
                    (
                        row["status"],
                        json.dumps(row["assignment"], sort_keys=True),
                        json.dumps(row["objective"], sort_keys=True),
                    )
                    for row in rows
                    if row["fixture_id"] == fixture_id
                }
            )
            == 1
            for fixture_id in definitions
        ),
        "summary": overall,
        "by_instance": dict(sorted(by_instance.items())),
        "by_seed": dict(sorted(by_seed.items())),
    }
    require(report["aggregate"] == expected_aggregate, "aggregate recomputation")
    expected_conclusion = (
        "Across all 40 precommitted rows, METIS matched the exact canonical assignment in "
        f"{overall['exact_assignment_agreements']}, the complete exact primary objective in "
        f"{overall['exact_primary_objective_agreements']}, and the exact population objective in "
        f"{overall['exact_population_objective_agreements']}; {overall['errors']} rows errored, "
        f"{overall['disconnected']} were disconnected, and {overall['insufficient_child_units']} "
        "had insufficient child units."
    )
    require(report["conclusion"] == expected_conclusion, "conclusion recomputation")
    print(
        "Certified vs METIS multi-instance independent verification: PASS "
        f"(8 instances, 40 rows, {overall['exact_primary_objective_agreements']} objective agreements)"
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("package", nargs="?", type=Path, default=PACKAGE)
    verify(parser.parse_args().package.resolve())
