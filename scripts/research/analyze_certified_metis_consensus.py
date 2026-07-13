#!/usr/bin/env python3
"""Analyze stable cores and disagreement bands across certified METIS seeds."""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import deque
from pathlib import Path


SCHEMA = "certified-metis-consensus-v1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def consensus(assignments: list[list[int]]) -> tuple[list[int | None], list[int]]:
    if not assignments:
        raise ValueError("at least one assignment is required")
    unit_count = len(assignments[0])
    if any(len(assignment) != unit_count for assignment in assignments):
        raise ValueError("assignment lengths differ")
    labels: list[int | None] = []
    disagreement: list[int] = []
    for unit in range(unit_count):
        observed = {assignment[unit] for assignment in assignments}
        if len(observed) == 1:
            labels.append(observed.pop())
        else:
            labels.append(None)
            disagreement.append(unit)
    return labels, disagreement


def component_sizes(adjacency: list[list[int]], units: list[int]) -> list[int]:
    allowed = set(units)
    seen: set[int] = set()
    sizes: list[int] = []
    for start in units:
        if start in seen:
            continue
        queue = deque([start])
        seen.add(start)
        size = 0
        while queue:
            unit = queue.popleft()
            size += 1
            for neighbor in adjacency[unit]:
                if neighbor in allowed and neighbor not in seen:
                    seen.add(neighbor)
                    queue.append(neighbor)
        sizes.append(size)
    return sorted(sizes, reverse=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--instance", type=Path, required=True)
    parser.add_argument("--ensemble-dir", type=Path, action="append", required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--top", type=int)
    parser.add_argument("--seeds")
    parser.add_argument("--expand-hops", type=int, default=0)
    args = parser.parse_args()

    instance = json.loads(args.instance.read_text(encoding="utf-8"))
    rows: list[dict[str, object]] = []
    assignments: list[list[int]] = []
    for ensemble_dir in args.ensemble_dir:
        report = json.loads((ensemble_dir / "ensemble.json").read_text(encoding="utf-8"))
        for row in report["results"]:
            if row["status"] != "accepted":
                continue
            seed = int(row["seed"])
            discovery_path = ensemble_dir / f"seed-{seed:04d}" / "certified-discovery.json"
            discovery = json.loads(discovery_path.read_text(encoding="utf-8"))
            assignment = discovery["objective"]["canonical_assignment"]
            rows.append(
                {
                    "seed": seed,
                    "discovery_id": discovery["discovery_id"],
                    "discovery_sha256": sha256(discovery_path),
                    "objective": discovery["objective"]["primary"],
                    "assignment": assignment,
                }
            )
    rows.sort(
        key=lambda row: (
            row["objective"]["max_population_deviation_scaled"],
            row["objective"]["total_population_deviation_scaled"],
            row["objective"]["weighted_boundary_cut"],
            row["seed"],
        )
    )
    if args.seeds:
        selected_seeds = {int(value) for value in args.seeds.split(",")}
        rows = [row for row in rows if row["seed"] in selected_seeds]
        if {row["seed"] for row in rows} != selected_seeds:
            raise SystemExit("one or more requested seeds were not accepted")
    if args.top is not None:
        rows = rows[: args.top]
    assignments = [row.pop("assignment") for row in rows]
    labels, disagreement = consensus(assignments)
    graph = [[] for _ in instance["unit_ids"]]
    for edge in instance["edges"]:
        graph[edge["left"]].append(edge["right"])
        graph[edge["right"]].append(edge["left"])
    expanded = set(disagreement)
    frontier = set(disagreement)
    for _ in range(args.expand_hops):
        frontier = {
            neighbor
            for unit in frontier
            for neighbor in graph[unit]
            if neighbor not in expanded
        }
        expanded.update(frontier)
    if args.expand_hops:
        disagreement = sorted(expanded)
        for unit in disagreement:
            labels[unit] = None
    stable_units = [[unit for unit, label in enumerate(labels) if label == child] for child in (0, 1)]
    stable_population = [
        sum(instance["populations"][unit] for unit in units) for units in stable_units
    ]
    stable_component_sizes = [
        component_sizes(graph, units) for units in stable_units
    ]
    disagreement_population = sum(
        instance["populations"][unit] for unit in disagreement
    )
    disagreement_set = set(disagreement)
    incident_disagreement = set(disagreement)
    for edge in instance["edges"]:
        if edge["left"] in disagreement_set or edge["right"] in disagreement_set:
            incident_disagreement.add(edge["left"])
            incident_disagreement.add(edge["right"])
    report = {
        "schema_version": SCHEMA,
        "instance_sha256": sha256(args.instance),
        "accepted_assignment_count": len(assignments),
        "expand_hops": args.expand_hops,
        "stable_unit_counts": [len(units) for units in stable_units],
        "stable_populations": stable_population,
        "stable_component_counts": [len(sizes) for sizes in stable_component_sizes],
        "largest_stable_component_sizes": [
            sizes[0] if sizes else 0 for sizes in stable_component_sizes
        ],
        "disagreement_unit_count": len(disagreement),
        "disagreement_population": disagreement_population,
        "one_hop_band_unit_count": len(incident_disagreement),
        "consensus_labels": labels,
        "seeds": rows,
        "disagreement_units": disagreement,
        "claim_boundary": (
            "Heuristic consensus abstraction only. Stable labels are not proof-safe "
            "fixed assignments and cannot be contracted in an exact claim without "
            "complete branch coverage."
        ),
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(
        "Certified METIS consensus: "
        f"{len(disagreement)} disagreement units, "
        f"{len(incident_disagreement)} in the one-hop band"
    )


if __name__ == "__main__":
    main()
