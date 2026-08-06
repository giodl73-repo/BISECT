#!/usr/bin/env python3
"""Independently verify the NRS cross-census structural stability package."""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import math
import statistics
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_PACKAGE = (
    ROOT / "docs" / "experiments" / "nrs-cross-decade-2000-2020" / "comparison"
)


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def signature(node: dict[str, Any]) -> str:
    children = node["child_seats"]
    require(len(children) == 2, "node must have two child seat counts")
    return f"{node['path']}:{node['seats']}:{children[0]}:{children[1]}"


def state_map(snapshot: dict[str, Any]) -> dict[str, dict[str, Any]]:
    states = {state["state"]: state for state in snapshot["states"]}
    require(len(states) == 50, "snapshot must contain 50 distinct States")
    return states


def node_map(state: dict[str, Any]) -> dict[str, dict[str, Any]]:
    nodes = {signature(node): node for node in state["nodes"]}
    require(len(nodes) == len(state["nodes"]), "duplicate node signature")
    return nodes


def close(actual: Any, expected: Any, label: str) -> None:
    if actual is None or expected is None:
        require(actual is expected, f"{label}: null mismatch")
    elif isinstance(actual, float) or isinstance(expected, float):
        require(
            math.isclose(float(actual), float(expected), rel_tol=1e-12, abs_tol=1e-12),
            f"{label}: {actual!r} != {expected!r}",
        )
    else:
        require(actual == expected, f"{label}: {actual!r} != {expected!r}")


def compare_pair(
    left: dict[str, Any], right: dict[str, Any]
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    left_year = left["census_year"]
    right_year = right["census_year"]
    left_states = state_map(left)
    right_states = state_map(right)
    require(left_states.keys() == right_states.keys(), "snapshot State universes differ")

    exact_states = 0
    same_seat_states = 0
    matched_nodes = 0
    union_nodes = 0
    cut_changes: list[float] = []
    tolerance_changes: list[float] = []
    state_rows: list[dict[str, Any]] = []
    for state in sorted(left_states):
        left_state = left_states[state]
        right_state = right_states[state]
        left_nodes = node_map(left_state)
        right_nodes = node_map(right_state)
        left_signatures = set(left_nodes)
        right_signatures = set(right_nodes)
        common = left_signatures & right_signatures
        union = left_signatures | right_signatures
        same_seats = left_state["districts"] == right_state["districts"]
        exact = (
            left_signatures == right_signatures
            and left_state["leaf_paths"] == right_state["leaf_paths"]
        )
        same_seat_states += int(same_seats)
        exact_states += int(exact)
        matched_nodes += len(common)
        union_nodes += len(union)

        state_cut_changes: list[float] = []
        state_tolerance_changes: list[float] = []
        for node_id in common:
            left_node = left_nodes[node_id]
            right_node = right_nodes[node_id]
            left_cut = float(left_node["weighted_boundary_cut_per_parent_person"])
            right_cut = float(right_node["weighted_boundary_cut_per_parent_person"])
            if left_cut > 0.0:
                state_cut_changes.append(abs((right_cut - left_cut) / left_cut))
            state_tolerance_changes.append(
                abs(float(right_node["tolerance_usage"]) - float(left_node["tolerance_usage"]))
            )
        cut_changes.extend(state_cut_changes)
        tolerance_changes.extend(state_tolerance_changes)
        state_rows.append(
            {
                "left_year": left_year,
                "right_year": right_year,
                "state": state,
                "left_districts": left_state["districts"],
                "right_districts": right_state["districts"],
                "same_seat_count": same_seats,
                "exact_tree_topology": exact,
                "matched_node_signatures": len(common),
                "union_node_signatures": len(union),
                "tree_topology_jaccard": len(common) / len(union) if union else 1.0,
                "median_absolute_relative_normalized_cut_change": (
                    statistics.median(state_cut_changes) if state_cut_changes else None
                ),
                "median_absolute_tolerance_usage_change": (
                    statistics.median(state_tolerance_changes)
                    if state_tolerance_changes
                    else None
                ),
            }
        )

    return (
        {
            "left_year": left_year,
            "right_year": right_year,
            "same_seat_count_states": same_seat_states,
            "exact_tree_topology_states": exact_states,
            "matched_node_signatures": matched_nodes,
            "union_node_signatures": union_nodes,
            "tree_topology_jaccard": matched_nodes / union_nodes,
            "median_absolute_relative_normalized_cut_change": statistics.median(
                cut_changes
            ),
            "median_absolute_tolerance_usage_change": statistics.median(
                tolerance_changes
            ),
        },
        state_rows,
    )


def check_row(actual: dict[str, Any], expected: dict[str, Any], label: str) -> None:
    require(actual.keys() == expected.keys(), f"{label}: field set mismatch")
    for key, value in expected.items():
        close(actual[key], value, f"{label}.{key}")


def verify(package: Path) -> None:
    manifest_path = package / "manifest.json"
    matrix_path = package / "stability-matrix.json"
    manifest = load(manifest_path)
    matrix = load(matrix_path)
    require(
        manifest["schema_version"] == "nrs-cross-census-stability-package-v1",
        "unknown comparison manifest schema",
    )
    require(
        manifest["status"] == "verified-structural-and-objective-comparison",
        "comparison manifest is not verified",
    )
    expected_matrix_hash = next(
        row["sha256"] for row in manifest["files"] if row["path"] == matrix_path.name
    )
    require(sha256(matrix_path) == expected_matrix_hash, "matrix transport hash mismatch")
    require(
        matrix["schema_version"] == "nrs-cross-census-stability-v1",
        "unknown matrix schema",
    )

    snapshots: list[dict[str, Any]] = []
    for artifact in manifest["snapshots"]:
        path = Path(artifact["path"])
        if not path.is_absolute():
            path = ROOT / path
        require(path.is_file(), f"missing snapshot: {path}")
        require(sha256(path) == artifact["sha256"], f"snapshot hash mismatch: {path}")
        snapshot = load(path)
        require(snapshot["schema_version"] == "nrs-node-snapshot-v1", "unknown snapshot")
        require(snapshot["census_year"] == artifact["census_year"], "snapshot year drift")
        require(snapshot["state_count"] == 50, "snapshot State count drift")
        require(snapshot["district_count"] == 435, "snapshot district count drift")
        require(snapshot["recursive_node_count"] == 385, "snapshot node count drift")
        states = state_map(snapshot)
        require(sum(state["districts"] for state in states.values()) == 435, "district sum drift")
        require(sum(len(state["nodes"]) for state in states.values()) == 385, "node sum drift")
        snapshots.append(snapshot)
    snapshots.sort(key=lambda snapshot: snapshot["census_year"])
    require(
        [snapshot["census_year"] for snapshot in snapshots] == matrix["census_years"],
        "matrix Census years do not match snapshots",
    )

    expected_pairs: list[dict[str, Any]] = []
    expected_states: list[dict[str, Any]] = []
    for left, right in itertools.combinations(snapshots, 2):
        pair, states = compare_pair(left, right)
        expected_pairs.append(pair)
        expected_states.extend(states)
    require(len(matrix["pairwise"]) == len(expected_pairs), "pairwise row count drift")
    for actual, expected in zip(matrix["pairwise"], expected_pairs, strict=True):
        check_row(actual, expected, f"pair-{expected['left_year']}-{expected['right_year']}")

    actual_states = {
        (row["left_year"], row["right_year"], row["state"]): row
        for row in matrix["state_matrix"]
    }
    require(len(actual_states) == len(expected_states), "State matrix row count drift")
    for expected in expected_states:
        key = (expected["left_year"], expected["right_year"], expected["state"])
        check_row(actual_states[key], expected, f"state-{key[0]}-{key[1]}-{key[2]}")

    maps = [state_map(snapshot) for snapshot in snapshots]
    exact_all = 0
    common_all = 0
    for state in sorted(maps[0]):
        sets = [set(node_map(states[state])) for states in maps]
        exact_all += int(all(nodes == sets[0] for nodes in sets[1:]))
        common_all += len(set.intersection(*sets))
    require(matrix["all_cycle_exact_topology_states"] == exact_all, "all-cycle State drift")
    require(matrix["all_cycle_common_node_signatures"] == common_all, "all-cycle node drift")
    require(matrix["assignment_overlap"]["status"] == "not-computed", "overlap overclaim")

    print(
        "NRS cross-census independent verification: PASS "
        f"({len(snapshots)} snapshots, {common_all} all-cycle common nodes)"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", nargs="?", type=Path, default=DEFAULT_PACKAGE)
    args = parser.parse_args()
    package = args.package.resolve()
    try:
        verify(package)
    except (KeyError, StopIteration, TypeError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(f"FAIL: {error}") from error


if __name__ == "__main__":
    main()
