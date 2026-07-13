#!/usr/bin/env python3
"""Optimize a heuristic fixed-core band with SciPy HiGHS and connectivity cuts."""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import deque
from pathlib import Path

import numpy as np
from scipy.optimize import Bounds, LinearConstraint, milp
from scipy.sparse import coo_matrix


SCHEMA = "certified-reduced-band-milp-v1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def adjacency(instance: dict) -> list[list[int]]:
    result = [[] for _ in instance["unit_ids"]]
    for edge in instance["edges"]:
        result[edge["left"]].append(edge["right"])
        result[edge["right"]].append(edge["left"])
    return result


def components(
    graph: list[list[int]], assignment: list[int], label: int
) -> list[list[int]]:
    members = [unit for unit, value in enumerate(assignment) if value == label]
    allowed = set(members)
    seen: set[int] = set()
    result: list[list[int]] = []
    for start in members:
        if start in seen:
            continue
        queue = deque([start])
        seen.add(start)
        component: list[int] = []
        while queue:
            unit = queue.popleft()
            component.append(unit)
            for neighbor in graph[unit]:
                if neighbor in allowed and neighbor not in seen:
                    seen.add(neighbor)
                    queue.append(neighbor)
        result.append(component)
    return result


def separate(
    graph: list[list[int]],
    assignment: list[int],
    fixed_labels: list[int | None],
) -> list[tuple[int, list[int], list[int]]]:
    cuts = []
    for label in (0, 1):
        anchor = next(
            (
                unit
                for unit, fixed_label in enumerate(fixed_labels)
                if fixed_label == label
            ),
            next(
                unit
                for unit, assigned_label in enumerate(assignment)
                if assigned_label == label
            ),
        )
        label_components = components(graph, assignment, label)
        if len(label_components) <= 1:
            continue
        for component in label_components:
            if anchor in component:
                continue
            component_set = set(component)
            boundary = sorted(
                {
                    neighbor
                    for unit in component
                    for neighbor in graph[unit]
                    if neighbor not in component_set
                    and fixed_labels[neighbor] is None
                }
            )
            cuts.append((label, component, boundary))
    return cuts


def solve_round(
    instance: dict,
    fixed_labels: list[int | None],
    exact_right_population: int,
    cuts: list[tuple[int, list[int], list[int]]],
    time_limit: float,
    cut_threshold: int | None,
):
    active_units = [unit for unit, label in enumerate(fixed_labels) if label is None]
    assignment_index = {unit: index for index, unit in enumerate(active_units)}
    active_edges = [
        edge_index
        for edge_index, edge in enumerate(instance["edges"])
        if fixed_labels[edge["left"]] is None
        or fixed_labels[edge["right"]] is None
    ]
    edge_offset = len(active_units)
    edge_variable = {
        edge: edge_offset + index for index, edge in enumerate(active_edges)
    }
    variable_count = len(active_units) + len(active_edges)
    objective = np.zeros(variable_count)
    fixed_cut = 0
    rows: list[int] = []
    columns: list[int] = []
    values: list[float] = []
    lower: list[float] = []
    upper: list[float] = []

    def add_constraint(terms: list[tuple[int, float]], lb: float, ub: float) -> None:
        row = len(lower)
        for column, value in terms:
            rows.append(row)
            columns.append(column)
            values.append(value)
        lower.append(lb)
        upper.append(ub)

    for edge_index, edge in enumerate(instance["edges"]):
        left, right = edge["left"], edge["right"]
        left_fixed, right_fixed = fixed_labels[left], fixed_labels[right]
        if left_fixed is not None and right_fixed is not None:
            if left_fixed != right_fixed:
                fixed_cut += edge["weight"]
            continue
        y = edge_variable[edge_index]
        objective[y] = edge["weight"]
        if left_fixed is None and right_fixed is None:
            x_left, x_right = assignment_index[left], assignment_index[right]
            add_constraint([(y, 1), (x_left, -1), (x_right, 1)], 0, np.inf)
            add_constraint([(y, 1), (x_left, 1), (x_right, -1)], 0, np.inf)
            add_constraint([(y, -1), (x_left, 1), (x_right, 1)], 0, np.inf)
            add_constraint([(y, -1), (x_left, -1), (x_right, -1)], -2, np.inf)
        else:
            active = left if left_fixed is None else right
            label = right_fixed if left_fixed is None else left_fixed
            x_active = assignment_index[active]
            if label == 0:
                add_constraint([(y, 1), (x_active, -1)], 0, 0)
            else:
                add_constraint([(y, 1), (x_active, 1)], 1, 1)

    fixed_right_population = sum(
        population
        for population, label in zip(
            instance["populations"], fixed_labels, strict=True
        )
        if label == 1
    )
    add_constraint(
        [
            (assignment_index[unit], instance["populations"][unit])
            for unit in active_units
        ],
        exact_right_population - fixed_right_population,
        exact_right_population - fixed_right_population,
    )

    for label, component, boundary in cuts:
        contains_fixed_core = any(
            fixed_labels[unit] == label for unit in component
        )
        representative = next(
            (
                unit
                for unit in component
                if fixed_labels[unit] is None
            ),
            None,
        )
        if label == 1 and contains_fixed_core:
            terms = [(assignment_index[unit], 1) for unit in boundary]
            add_constraint(terms, 1, np.inf)
        elif label == 1:
            terms = [(assignment_index[unit], 1) for unit in boundary]
            assert representative is not None
            terms.append((assignment_index[representative], -1))
            add_constraint(terms, 0, np.inf)
        elif contains_fixed_core:
            terms = [(assignment_index[unit], -1) for unit in boundary]
            add_constraint(terms, 1 - len(boundary), np.inf)
        else:
            terms = [(assignment_index[unit], -1) for unit in boundary]
            assert representative is not None
            terms.append((assignment_index[representative], 1))
            add_constraint(terms, 1 - len(boundary), np.inf)

    if cut_threshold is not None:
        add_constraint(
            [
                (edge_variable[edge], instance["edges"][edge]["weight"])
                for edge in active_edges
            ],
            -np.inf,
            cut_threshold - fixed_cut,
        )
        objective[:] = 0

    matrix = coo_matrix(
        (values, (rows, columns)), shape=(len(lower), variable_count)
    ).tocsr()
    result = milp(
        c=objective,
        integrality=np.ones(variable_count),
        bounds=Bounds(np.zeros(variable_count), np.ones(variable_count)),
        constraints=LinearConstraint(matrix, np.array(lower), np.array(upper)),
        options={"time_limit": time_limit, "mip_rel_gap": 0.0},
    )
    return result, active_units, fixed_cut


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--instance", type=Path, required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--consensus", type=Path)
    mode.add_argument("--unrestricted", action="store_true")
    parser.add_argument("--right-population", type=int, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--time-limit", type=float, default=300)
    parser.add_argument("--max-rounds", type=int, default=50)
    parser.add_argument("--cut-threshold", type=int)
    args = parser.parse_args()

    instance = json.loads(args.instance.read_text(encoding="utf-8"))
    if args.unrestricted:
        fixed_labels = [None] * len(instance["unit_ids"])
    else:
        consensus_report = json.loads(args.consensus.read_text(encoding="utf-8"))
        fixed_labels = consensus_report["consensus_labels"]
    graph = adjacency(instance)
    cuts: list[tuple[int, list[int], list[int]]] = []
    seen_cuts: set[str] = set()
    rounds: list[dict[str, object]] = []
    final_assignment = None
    final_cut = None

    for round_index in range(args.max_rounds):
        result, active_units, fixed_cut = solve_round(
            instance,
            fixed_labels,
            args.right_population,
            cuts,
            args.time_limit,
            args.cut_threshold,
        )
        row: dict[str, object] = {
            "round": round_index,
            "status": int(result.status),
            "message": result.message,
            "cut_count": len(cuts),
            "mip_gap": getattr(result, "mip_gap", None),
        }
        if result.x is None:
            rounds.append(row)
            break
        assignment = [
            int(label) if label is not None else 0 for label in fixed_labels
        ]
        for index, unit in enumerate(active_units):
            assignment[unit] = int(result.x[index] >= 0.5)
        separated = separate(graph, assignment, fixed_labels)
        row["objective"] = sum(
            edge["weight"]
            for edge in instance["edges"]
            if assignment[edge["left"]] != assignment[edge["right"]]
        )
        row["separated_cut_count"] = len(separated)
        rounds.append(row)
        if not separated:
            final_assignment = assignment
            final_cut = row["objective"]
            break
        new_count = 0
        for cut in separated:
            identity = json.dumps(cut, separators=(",", ":"))
            if identity not in seen_cuts:
                seen_cuts.add(identity)
                cuts.append(cut)
                new_count += 1
        row["new_cut_count"] = new_count
        if new_count == 0:
            raise RuntimeError("disconnected MILP assignment produced no new cuts")

    output = {
        "schema_version": SCHEMA,
        "instance_sha256": sha256(args.instance),
        "consensus_sha256": sha256(args.consensus) if args.consensus else None,
        "unrestricted": args.unrestricted,
        "right_population": args.right_population,
        "rounds": rounds,
        "final_connected_assignment": final_assignment,
        "final_weighted_cut": final_cut,
        "connectivity_cuts": [
            {
                "district_id": label,
                "component": component,
                "outside_neighbors": boundary,
            }
            for label, component, boundary in cuts
        ],
        "claim_boundary": (
            "Heuristic fixed-core MILP search only; stable cores are not globally "
            "proof-safe and no optimality claim follows."
        ),
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")
    print(
        f"Reduced-band MILP: {len(rounds)} rounds, "
        f"connected cut {final_cut}"
    )


if __name__ == "__main__":
    main()
