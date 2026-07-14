#!/usr/bin/env python3
"""Build and verify a four-district operational recursive discovery tree."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
from collections import deque
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path("scripts/research/build_operational_recursive_tree.py")
SCREEN_TIMEOUT_SECONDS = 180


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_hash(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def connected(adjacency: list[list[dict]], assignment: list[int], label: int) -> bool:
    members = [unit for unit, value in enumerate(assignment) if value == label]
    allowed = set(members)
    seen = {members[0]}
    queue = deque([members[0]])
    while queue:
        unit = queue.popleft()
        for edge in adjacency[unit]:
            neighbor = edge["to"]
            if neighbor in allowed and neighbor not in seen:
                seen.add(neighbor)
                queue.append(neighbor)
    return seen == allowed


def ratio_arithmetic_floor(parent_population: int, seats: int, right_seats: int) -> int:
    remainder = (right_seats * parent_population) % seats
    return min(remainder, seats - remainder)


def discovery_seed(discovery: dict) -> int:
    for field in discovery["method"].split(";"):
        field = field.strip()
        if field.startswith("seed="):
            return int(field.removeprefix("seed="))
    raise ValueError("certified discovery method does not record its seed")


def prune_discovery_scratch(out_dir: Path) -> None:
    """Keep only the discovery needed for ranking and deterministic resume."""
    for path in out_dir.iterdir():
        if path.name == "certified-discovery.json":
            continue
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()


def subset_context(context: dict, selected: list[int], source_id: str) -> dict:
    selected_set = set(selected)
    remap = {old: new for new, old in enumerate(selected)}
    units = dict(context["units"])
    units["unit_ids"] = [context["units"]["unit_ids"][unit] for unit in selected]
    units["source_id"] = source_id
    units["unit_universe_hash"] = canonical_hash(units)
    adjacency = []
    for old in selected:
        adjacency.append(
            [
                {**edge, "to": remap[edge["to"]]}
                for edge in context["graph"]["adjacency"][old]
                if edge["to"] in selected_set
            ]
        )
    projection = {
        "units": units,
        "graph": {"edge_semantics": "undirected", "adjacency": adjacency},
        "populations": [context["populations"][unit] for unit in selected],
        "source_hashes": context["source_hashes"],
    }
    return {
        "rctx_version": "0.1",
        "context_hash": canonical_hash(projection),
        **projection,
    }


def run_discovery(
    bisect: Path,
    context_path: Path,
    districts: int,
    out_dir: Path,
    seed: int,
    refinement: str = "population",
    timeout_seconds: int | None = None,
) -> dict:
    subprocess.run(
        [
            str(bisect),
            "exact",
            "--context",
            str(context_path),
            "--districts",
            str(districts),
            "--method",
            "certified-discovery",
            "--out-dir",
            str(out_dir),
            "--generated-at",
            "2026-07-12T00:00:00Z",
            "--discovery-seed",
            str(seed),
            "--discovery-refinement",
            refinement,
        ],
        cwd=ROOT,
        check=True,
        timeout=timeout_seconds,
    )
    return json.loads(
        (out_dir / "certified-discovery.json").read_text(encoding="utf-8")
    )


def run_floor_discovery(
    bisect: Path,
    context_path: Path,
    districts: int,
    out_dir: Path,
    preferred_seed: int,
    population_floor: int,
    max_seed: int,
) -> tuple[dict, int, list[dict]]:
    completed_path = out_dir / "certified-discovery.json"
    if completed_path.is_file():
        completed = json.loads(completed_path.read_text(encoding="utf-8"))
        completed_deviation = completed["objective"]["primary"][
            "max_population_deviation_scaled"
        ]
        if completed_deviation == population_floor:
            report_path = out_dir / "seed-screening.json"
            report = (
                json.loads(report_path.read_text(encoding="utf-8"))
                if report_path.is_file()
                else [
                    {
                        "seed": discovery_seed(completed),
                        "status": "selected-node-reused",
                        "objective": completed["objective"]["primary"],
                    }
                ]
            )
            print(
                f"{out_dir.name}: reused completed node at arithmetic floor "
                f"{population_floor}",
                flush=True,
            )
            return completed, discovery_seed(completed), report
    seeds = [preferred_seed] + [
        seed for seed in range(1, max_seed + 1) if seed != preferred_seed
    ]
    screened = []
    screen_report = []
    for seed in seeds:
        screen_dir = out_dir.with_name(f"{out_dir.name}-screen-seed-{seed:02d}")
        timeout_path = screen_dir.with_suffix(".timeout.json")
        if timeout_path.is_file():
            timeout_report = json.loads(timeout_path.read_text(encoding="utf-8"))
            screen_report.append(
                {
                    "seed": seed,
                    "status": "timeout",
                    "timeout_seconds": timeout_report["timeout_seconds"],
                    "reused": True,
                }
            )
            print(
                f"{out_dir.name}: reused recorded screen timeout for seed {seed}",
                flush=True,
            )
            continue
        discovery_path = screen_dir / "certified-discovery.json"
        reused = discovery_path.is_file()
        if reused:
            discovery = json.loads(discovery_path.read_text(encoding="utf-8"))
            prune_discovery_scratch(screen_dir)
        else:
            if screen_dir.exists():
                shutil.rmtree(screen_dir)
            try:
                discovery = run_discovery(
                    bisect,
                    context_path,
                    districts,
                    screen_dir,
                    seed,
                    refinement="metis",
                    timeout_seconds=SCREEN_TIMEOUT_SECONDS,
                )
            except subprocess.TimeoutExpired:
                if screen_dir.exists():
                    shutil.rmtree(screen_dir)
                timeout_path.write_text(
                    json.dumps(
                        {
                            "status": "timeout",
                            "timeout_seconds": SCREEN_TIMEOUT_SECONDS,
                        },
                        indent=2,
                    )
                    + "\n",
                    encoding="utf-8",
                )
                screen_report.append(
                    {
                        "seed": seed,
                        "status": "timeout",
                        "timeout_seconds": SCREEN_TIMEOUT_SECONDS,
                    }
                )
                print(
                    f"{out_dir.name}: screen seed {seed} timed out after "
                    f"{SCREEN_TIMEOUT_SECONDS} seconds",
                    flush=True,
                )
                continue
            except subprocess.CalledProcessError as error:
                if screen_dir.exists():
                    shutil.rmtree(screen_dir)
                screen_report.append(
                    {
                        "seed": seed,
                        "status": "failed",
                        "exit_code": error.returncode,
                    }
                )
                print(
                    f"{out_dir.name}: screen seed {seed} failed with exit code "
                    f"{error.returncode}",
                    flush=True,
                )
                continue
            prune_discovery_scratch(screen_dir)
        if timeout_path.is_file():
            timeout_path.unlink()
        screened.append((discovery, seed, screen_dir))
        screen_report.append(
            {
                "seed": seed,
                "status": "completed",
                "reused": reused,
                "objective": discovery["objective"]["primary"],
            }
        )
        print(
            f"{out_dir.name}: screened seed {seed} at deviation "
            f"{discovery['objective']['primary']['max_population_deviation_scaled']}",
            flush=True,
        )
    screened.sort(
        key=lambda row: (
            row[0]["objective"]["primary"]["max_population_deviation_scaled"],
            row[0]["objective"]["primary"]["total_population_deviation_scaled"],
            row[0]["objective"]["primary"]["weighted_boundary_cut"],
            row[1],
        )
    )
    candidates = []
    selected = None
    for screened_discovery, seed, screen_dir in screened:
        if (
            screened_discovery["objective"]["primary"][
                "max_population_deviation_scaled"
            ]
            == population_floor
        ):
            selected = (screened_discovery, seed, screen_dir)
            break
        candidate_dir = out_dir.with_name(f"{out_dir.name}-seed-{seed:02d}")
        candidate_path = candidate_dir / "certified-discovery.json"
        refined_reused = candidate_path.is_file()
        if refined_reused:
            discovery = json.loads(candidate_path.read_text(encoding="utf-8"))
        else:
            if candidate_dir.exists():
                shutil.rmtree(candidate_dir)
            discovery = run_discovery(
                bisect,
                context_path,
                districts,
                candidate_dir,
                seed,
                refinement="population",
            )
        prune_discovery_scratch(candidate_dir)
        candidates.append((discovery, seed, candidate_dir))
        screen_row = next(row for row in screen_report if row["seed"] == seed)
        screen_row["refined_objective"] = discovery["objective"]["primary"]
        screen_row["refined_reused"] = refined_reused
        print(
            f"{out_dir.name}: refined seed {seed} to deviation "
            f"{discovery['objective']['primary']['max_population_deviation_scaled']}",
            flush=True,
        )
        if (
            discovery["objective"]["primary"]["max_population_deviation_scaled"]
            == population_floor
        ):
            selected = (discovery, seed, candidate_dir)
            break
    if selected is None:
        out_dir.mkdir(parents=True, exist_ok=True)
        (out_dir / "seed-screening.json").write_text(
            json.dumps(screen_report, indent=2) + "\n", encoding="utf-8"
        )
        if not candidates:
            (out_dir / "unresolved-floor.json").write_text(
                json.dumps(
                    {
                        "population_floor": population_floor,
                        "status": "no-completed-refinement",
                    },
                    indent=2,
                )
                + "\n",
                encoding="utf-8",
            )
            raise RuntimeError("no seed completed discovery screening")
        best = min(
            candidates,
            key=lambda row: (
                row[0]["objective"]["primary"]["max_population_deviation_scaled"],
                row[0]["objective"]["primary"]["total_population_deviation_scaled"],
                row[0]["objective"]["primary"]["weighted_boundary_cut"],
                row[1],
            ),
        )
        (out_dir / "unresolved-floor.json").write_text(
            json.dumps(
                {
                    "population_floor": population_floor,
                    "status": "unresolved-local-search-frontier",
                    "best_seed": best[1],
                    "best_objective": best[0]["objective"]["primary"],
                    "claim_boundary": (
                        "No screened deterministic seed reached the arithmetic "
                        "population floor; this is not a proof of infeasibility."
                    ),
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
        raise RuntimeError(
            f"no seed reached arithmetic population floor {population_floor}; "
            f"best seed {best[1]} reached "
            f"{best[0]['objective']['primary']['max_population_deviation_scaled']}"
        )
    discovery, seed, candidate_dir = selected
    if out_dir.exists():
        shutil.rmtree(out_dir)
    candidate_dir.rename(out_dir)
    (out_dir / "seed-screening.json").write_text(
        json.dumps(screen_report, indent=2) + "\n", encoding="utf-8"
    )
    for _, _, other_dir in candidates:
        if other_dir != candidate_dir and other_dir.exists():
            shutil.rmtree(other_dir)
    for _, _, screen_dir in screened:
        if screen_dir != candidate_dir and screen_dir.exists():
            shutil.rmtree(screen_dir)
    for name in (
        "audit-certificate.json",
        "certified-discovery-manifest.json",
        "discovery.rctx",
        "discovery.rplan",
    ):
        path = out_dir / name
        if path.is_file():
            path.unlink()
    return discovery, seed, screen_report


def build(
    bisect: Path,
    context_path: Path,
    out_dir: Path,
    districts: int,
    root_seed: int,
    child_seeds: tuple[int, int],
) -> None:
    context = json.loads(context_path.read_text(encoding="utf-8"))
    out_dir.mkdir(parents=True, exist_ok=True)
    final_assignment = [-1] * len(context["units"]["unit_ids"])
    nodes = []
    leaves = []

    def visit(
        node_context: dict,
        node_context_path: Path,
        global_units: list[int],
        seats: int,
        path: str,
        district_offset: int,
        seed: int,
    ) -> None:
        if seats == 1:
            for unit in global_units:
                final_assignment[unit] = district_offset
            leaves.append(
                {
                    "path": path,
                    "district": district_offset,
                    "unit_count": len(global_units),
                    "population": sum(
                        context["populations"][unit] for unit in global_units
                    ),
                }
            )
            return
        node_dir = out_dir / ("root" if path == "" else f"node-{path}")
        left_seats = seats // 2
        right_seats = seats - left_seats
        floor = ratio_arithmetic_floor(
            sum(node_context["populations"]), seats, right_seats
        )
        discovery, selected_seed, seed_screening = run_floor_discovery(
            bisect,
            node_context_path,
            seats,
            node_dir,
            seed,
            floor,
            16,
        )
        assignment = discovery["objective"]["canonical_assignment"]
        nodes.append(
            {
                "path": path,
                "seats": seats,
                "parent_population": sum(node_context["populations"]),
                "discovery_id": discovery["discovery_id"],
                "seed": selected_seed,
                "seed_screening": seed_screening,
                "objective": discovery["objective"]["primary"],
                "population_proof": {
                    "kind": "ratio-arithmetic-floor",
                    "lower_bound": floor,
                },
                "context_sha256": sha256(node_context_path),
            }
        )
        for label, child_seats, offset in (
            (0, left_seats, district_offset),
            (1, right_seats, district_offset + left_seats),
        ):
            local_units = [
                unit for unit, assigned in enumerate(assignment) if assigned == label
            ]
            child_global_units = [global_units[unit] for unit in local_units]
            child_context = subset_context(
                node_context,
                local_units,
                f"operational-tree-node-{path or 'root'}-{label}",
            )
            child_context_path = out_dir / f"context-{path}{label}.rctx"
            child_context_path.write_text(
                json.dumps(child_context, separators=(",", ":")), encoding="utf-8"
            )
            child_seed = (
                child_seeds[label] if path == "" else seed + label + 1
            )
            visit(
                child_context,
                child_context_path,
                child_global_units,
                child_seats,
                f"{path}{label}",
                offset,
                child_seed,
            )

    visit(
        context,
        context_path,
        list(range(len(final_assignment))),
        districts,
        "",
        0,
        root_seed,
    )
    if sorted(final_assignment)[:1] == [-1]:
        raise SystemExit("operational tree left units unassigned")
    if any(
        not connected(context["graph"]["adjacency"], final_assignment, district)
        for district in range(districts)
    ):
        raise SystemExit("operational tree has disconnected leaf")
    tree = {
        "schema_version": "certified-operational-recursive-tree-v1",
        "status": "operational-unproved-objectives",
        "context_sha256": sha256(context_path),
        "context_hash": context["context_hash"],
        "districts": districts,
        "nodes": nodes,
        "leaves": leaves,
        "assignment": final_assignment,
        "unit_count": len(final_assignment),
        "population_total": sum(context["populations"]),
        "claim_boundary": (
            "Complete connected wall-to-wall recursive assignment. Node objectives "
            "are discovery incumbents unless separately proved."
        ),
    }
    tree_path = out_dir / "operational-tree.json"
    tree_path.write_text(json.dumps(tree, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-operational-recursive-tree-package-v1",
        "package_id": f"operational-tree-{context['units'].get('state', 'unknown').lower()}-2020",
        "status": tree["status"],
        "files": [{"path": tree_path.name, "sha256": sha256(tree_path)}],
        "builder_path": SCRIPT.as_posix(),
        "builder_sha256": sha256(ROOT / SCRIPT),
        "claim_boundary": tree["claim_boundary"],
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    for path in list(out_dir.iterdir()):
        if path.name in {"manifest.json", "operational-tree.json"}:
            continue
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()
    print("Operational recursive tree: VERIFIED")


def verify(package: Path) -> None:
    manifest = json.loads((package / "manifest.json").read_text(encoding="utf-8"))
    if sha256(ROOT / manifest["builder_path"]) != manifest["builder_sha256"]:
        raise SystemExit("operational tree builder hash mismatch")
    tree_path = package / manifest["files"][0]["path"]
    if sha256(tree_path) != manifest["files"][0]["sha256"]:
        raise SystemExit("operational tree hash mismatch")
    tree = json.loads(tree_path.read_text(encoding="utf-8"))
    if (
        tree["districts"] < 2
        or tree["unit_count"] != len(tree["assignment"])
        or sorted(set(tree["assignment"])) != list(range(tree["districts"]))
        or sum(leaf["unit_count"] for leaf in tree["leaves"]) != tree["unit_count"]
        or sum(leaf["population"] for leaf in tree["leaves"]) != tree["population_total"]
        or any(
            node["objective"]["max_population_deviation_scaled"]
            != node["population_proof"]["lower_bound"]
            for node in tree["nodes"]
        )
    ):
        raise SystemExit("operational tree coverage drift")
    print("Operational recursive tree package verification: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    build_parser = subparsers.add_parser("build")
    build_parser.add_argument("--bisect", type=Path, required=True)
    build_parser.add_argument("--context", type=Path, required=True)
    build_parser.add_argument("--out-dir", type=Path, required=True)
    build_parser.add_argument("--districts", type=int, required=True)
    build_parser.add_argument("--root-seed", type=int, default=1)
    build_parser.add_argument("--child-seed-0", type=int, default=2)
    build_parser.add_argument("--child-seed-1", type=int, default=3)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("package", type=Path)
    args = parser.parse_args()
    if args.command == "build":
        build(
            args.bisect.resolve(),
            args.context.resolve(),
            args.out_dir.resolve(),
            args.districts,
            args.root_seed,
            (args.child_seed_0, args.child_seed_1),
        )
    else:
        verify(args.package.resolve())


if __name__ == "__main__":
    main()
