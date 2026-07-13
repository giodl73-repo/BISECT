#!/usr/bin/env python3
"""Run iterative certified connectivity-cut separation with RoundingSat."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from collections import deque
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCHEMA = "certified-cutset-search-v1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def adjacency_from_instance(instance: dict) -> list[list[int]]:
    adjacency = [[] for _ in instance["unit_ids"]]
    for edge in instance["edges"]:
        left, right = edge["left"], edge["right"]
        adjacency[left].append(right)
        adjacency[right].append(left)
    for neighbors in adjacency:
        neighbors.sort()
    return adjacency


def components(
    adjacency: list[list[int]], assignment: list[int], label: int
) -> list[list[int]]:
    members = [unit for unit, assigned in enumerate(assignment) if assigned == label]
    member_set = set(members)
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
            for neighbor in adjacency[unit]:
                if neighbor in member_set and neighbor not in seen:
                    seen.add(neighbor)
                    queue.append(neighbor)
        result.append(component)
    return result


def separate(
    adjacency: list[list[int]],
    assignment: list[int],
    fixed_labels: list[int | None] | None = None,
) -> list[dict[str, object]]:
    cuts: list[dict[str, object]] = []
    for label in (0, 1):
        anchor = (
            next(
                unit
                for unit, fixed_label in enumerate(fixed_labels)
                if fixed_label == label
            )
            if fixed_labels
            else None
        )
        label_components = components(adjacency, assignment, label)
        if len(label_components) <= 1:
            continue
        for component in label_components:
            if anchor is not None and anchor in component:
                continue
            component_set = set(component)
            outside = sorted(
                {
                    neighbor
                    for unit in component
                    for neighbor in adjacency[unit]
                    if neighbor not in component_set
                }
            )
            cuts.append(
                {
                    "district_id": label,
                    "component": component,
                    "outside_neighbors": outside,
                }
            )
    return cuts


def parse_assignment(output: str, unit_count: int) -> list[int]:
    values: dict[int, int] = {}
    for line in output.splitlines():
        if not line.startswith("v "):
            continue
        for token in line[2:].split():
            negative = token.startswith("-")
            name = token[1:] if negative else token
            if not name.startswith("x"):
                continue
            index = int(name[1:]) - 1
            if index < unit_count:
                values[index] = 0 if negative else 1
    if len(values) != unit_count:
        raise RuntimeError(
            f"solver assignment contains {len(values)} of {unit_count} unit variables"
        )
    return [values[index] for index in range(unit_count)]


def wsl_path(path: Path) -> str:
    resolved = path.resolve()
    drive = resolved.drive.rstrip(":").lower()
    if drive:
        relative = resolved.as_posix().split(":", 1)[1].lstrip("/")
        return f"/mnt/{drive}/{relative}"
    return resolved.as_posix()


def run_solver(
    solver: Path, model: Path, time_limit: int, use_lp: bool
) -> tuple[str, str]:
    lp_option = "" if use_lp else "--lp=0 "
    command = (
        f"'{wsl_path(solver)}' {lp_option}--time-limit={time_limit} "
        f"--print-sol=1 '{wsl_path(model)}'"
    )
    completed = subprocess.run(
        ["wsl", "bash", "-lc", command],
        check=False,
        capture_output=True,
        text=True,
    )
    output = completed.stdout + completed.stderr
    for status in ("SATISFIABLE", "UNSATISFIABLE", "TIMELIMIT"):
        if f"s {status}" in output:
            return status.lower(), output
    raise RuntimeError("RoundingSat produced no recognized status")


def compile_model(
    instance_path: Path,
    discovery_path: Path,
    exact_population: int,
    cuts_path: Path,
    fixed_labels_path: Path,
    round_dir: Path,
) -> None:
    subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "bisect-ilp",
            "--example",
            "certified_cutset_model",
            "--",
            str(instance_path),
            str(discovery_path),
            str(exact_population),
            str(cuts_path),
            str(fixed_labels_path),
            str(round_dir),
        ],
        cwd=ROOT,
        check=True,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--instance", type=Path, required=True)
    parser.add_argument("--discovery", type=Path, required=True)
    parser.add_argument("--right-population", type=int, required=True)
    parser.add_argument("--solver", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--max-rounds", type=int, default=100)
    parser.add_argument("--time-limit", type=int, default=300)
    parser.add_argument("--fixed-labels", type=Path)
    parser.add_argument("--lp", action="store_true")
    args = parser.parse_args()

    instance_path = args.instance.resolve()
    discovery_path = args.discovery.resolve()
    out_dir = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    instance = json.loads(instance_path.read_text(encoding="utf-8"))
    adjacency = adjacency_from_instance(instance)
    if args.fixed_labels:
        fixed_labels = json.loads(args.fixed_labels.read_text(encoding="utf-8"))
        if isinstance(fixed_labels, dict):
            fixed_labels = fixed_labels["consensus_labels"]
    else:
        fixed_labels = [None] * len(instance["unit_ids"])
    if len(fixed_labels) != len(instance["unit_ids"]):
        raise RuntimeError("fixed-label vector length differs from the instance")
    cuts: list[dict[str, object]] = []
    seen_cuts: set[str] = set()
    rounds: list[dict[str, object]] = []

    for round_index in range(args.max_rounds):
        round_dir = out_dir / f"round-{round_index:04d}"
        round_dir.mkdir(parents=True, exist_ok=True)
        cuts_path = round_dir / "input-cuts.json"
        cuts_path.write_text(json.dumps(cuts, indent=2) + "\n", encoding="utf-8")
        fixed_labels_path = round_dir / "fixed-labels.json"
        fixed_labels_path.write_text(
            json.dumps(fixed_labels, separators=(",", ":")) + "\n",
            encoding="utf-8",
        )
        compile_model(
            instance_path,
            discovery_path,
            args.right_population,
            cuts_path,
            fixed_labels_path,
            round_dir,
        )
        status, solver_output = run_solver(
            args.solver.resolve(),
            round_dir / "boundary.opb",
            args.time_limit,
            args.lp,
        )
        (round_dir / "roundingsat.log").write_text(solver_output, encoding="utf-8")
        row: dict[str, object] = {
            "round": round_index,
            "status": status,
            "input_cut_count": len(cuts),
            "model_sha256": sha256(round_dir / "boundary.opb"),
        }
        if status != "satisfiable":
            rounds.append(row)
            break
        active_units = [
            unit for unit, label in enumerate(fixed_labels) if label is None
        ]
        active_assignment = parse_assignment(solver_output, len(active_units))
        assignment = [
            int(label) if label is not None else 0 for label in fixed_labels
        ]
        for unit, label in zip(active_units, active_assignment, strict=True):
            assignment[unit] = label
        assignment_path = round_dir / "assignment.json"
        assignment_path.write_text(
            json.dumps(assignment, separators=(",", ":")) + "\n", encoding="utf-8"
        )
        separated = separate(adjacency, assignment, fixed_labels)
        new_cuts = []
        for cut in separated:
            identity = json.dumps(cut, separators=(",", ":"), sort_keys=True)
            if identity not in seen_cuts:
                seen_cuts.add(identity)
                new_cuts.append(cut)
        row["assignment_sha256"] = sha256(assignment_path)
        row["separated_cut_count"] = len(separated)
        row["new_cut_count"] = len(new_cuts)
        rounds.append(row)
        if not separated:
            row["connected_counterexample"] = True
            break
        if not new_cuts:
            raise RuntimeError("disconnected assignment produced no new connectivity cuts")
        cuts.extend(new_cuts)
    else:
        status = "round-limit"

    result = {
        "schema_version": SCHEMA,
        "instance_sha256": sha256(instance_path),
        "discovery_sha256": sha256(discovery_path),
        "exact_right_population": args.right_population,
        "status": status,
        "round_count": len(rounds),
        "final_cut_count": len(cuts),
        "rounds": rounds,
        "claim_boundary": (
            "Iterative SAT and connectivity-cut separation; UNSAT requires a "
            "separately generated and VeriPB-checked final proof."
        ),
    }
    (out_dir / "search.json").write_text(
        json.dumps(result, indent=2) + "\n", encoding="utf-8"
    )
    (out_dir / "cuts.json").write_text(
        json.dumps(cuts, indent=2) + "\n", encoding="utf-8"
    )
    print(
        f"Certified cutset search: {status}, {len(rounds)} rounds, {len(cuts)} cuts"
    )


if __name__ == "__main__":
    main()
