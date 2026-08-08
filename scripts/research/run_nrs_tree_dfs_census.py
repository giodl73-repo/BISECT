#!/usr/bin/env python3
"""Run the precommitted 2020 NRS complete-tree DFS census."""

from __future__ import annotations

import argparse
import csv
import json
import shutil
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import run_nrs_dfs_tie_census as tie


ROOT = Path(__file__).resolve().parents[2]
PROTOCOL_ID = "nrs-v0.3-complete-tree-dfs-census-v1"
SCHEMA_VERSION = "nrs-v0.3-complete-tree-dfs-census-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-complete-tree-dfs-census-manifest-v1"
GENERATED_AT = "2026-08-08T00:00:00Z"
PROTOCOL_PATH = Path("docs/specs/2026-08-08-nrs-v0.3-tree-dfs-census-protocol.md")
RUNNER_PATH = Path("scripts/research/run_nrs_tree_dfs_census.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_tree_dfs_census.py")
BASE_RUNNER_PATH = Path("scripts/research/run_nrs_dfs_tie_census.py")
ENGINE_PATH = Path("crates/bisect-cli/src/exact_cmd.rs")
OPS_PATH = Path("crates/bisect-ops/src/main.rs")
CLAIM_BOUNDARY = (
    "Governed 2020 complete-tree initial DFS partition and fallback activation "
    "diagnostics only; no seed-invariant label or plan, cross-census, national "
    "robustness, optimality, partisan, or legal-quality claim."
)
NODE_FIELDS = [
    "state",
    "path",
    "seats",
    "child_seats_left",
    "child_seats_right",
    "equal_child_seats",
    "unit_count",
    "minimum_deviation_candidates",
    "minimum_deviation_cut_candidates",
    "minimum_deviation_cut_partitions",
    "orientation_only_tie",
    "physical_cut_opportunity",
    "nrs_v0_2_fallback_activated",
    "nrs_v0_3_fallback_activated",
    "assignment_match",
    "objective_match",
]
STATE_FIELDS = [
    "state",
    "districts",
    "node_count",
    "status",
    "failure",
    "state_assignment_match",
    "node_assignment_match_count",
    "node_objective_match_count",
]


def method_bool(method: str, name: str) -> bool:
    value = tie.method_counter(method.replace("=true", "=1").replace("=false", "=0"), name)
    tie.require(value in (0, 1), f"invalid method boolean {name}")
    return bool(value)


def discovery_path(package: Path, path: str) -> Path:
    name = "root" if not path else f"node-{path}"
    return package / "nodes" / name / "certified-discovery.json"


def state_inputs(state_dir: Path, bisect: Path, ops: Path) -> dict:
    state = state_dir.name.upper()
    package = state_dir / "package"
    tree_path = package / "baseline-tree.json"
    tree = json.loads(tree_path.read_text(encoding="utf-8"))
    return {
        "state": state,
        "bisect": bisect,
        "ops": ops,
        "context": ROOT / f"data/2020/certified/{state.lower()}_blocks_2020.rctx",
        "seed": state_dir / "seed",
        "benchmark_package": package,
        "benchmark_tree": tree_path,
        "districts": tree["districts"],
    }


def run_state(inputs: dict, temp_root: Path) -> tuple[dict, list[dict]]:
    state = inputs["state"]
    output = temp_root / state.lower()
    try:
        completed = subprocess.run(
            [
                str(inputs["ops"]),
                "build-nrs-state",
                "--bisect",
                str(inputs["bisect"]),
                "--context",
                str(inputs["context"]),
                "--districts",
                str(inputs["districts"]),
                "--seed-package",
                str(inputs["seed"]),
                "--out-dir",
                str(output),
                "--generated-at",
                GENERATED_AT,
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            return (
                {
                    "state": state,
                    "districts": inputs["districts"],
                    "status": "rejected",
                    "failure": tie.sanitize_failure(
                        completed.stdout + completed.stderr, temp_root
                    ),
                },
                [],
            )
        benchmark_tree = json.loads(
            inputs["benchmark_tree"].read_text(encoding="utf-8")
        )
        replay_tree = json.loads(
            (output / "baseline-tree.json").read_text(encoding="utf-8")
        )
        benchmark_nodes = {node["path"]: node for node in benchmark_tree["nodes"]}
        replay_nodes = {node["path"]: node for node in replay_tree["nodes"]}
        tie.require(
            benchmark_nodes.keys() == replay_nodes.keys(),
            f"{state}: node path universe changed",
        )
        node_rows = []
        for path in sorted(benchmark_nodes, key=lambda value: (len(value), value)):
            benchmark_node = benchmark_nodes[path]
            replay_node = replay_nodes[path]
            benchmark_discovery = json.loads(
                discovery_path(inputs["benchmark_package"], path).read_text(
                    encoding="utf-8"
                )
            )
            replay_discovery = json.loads(
                discovery_path(output, path).read_text(encoding="utf-8")
            )
            method = replay_discovery["method"]
            deviation_count = tie.method_counter(
                method, "initial-dfs-minimum-deviation-candidates"
            )
            cut_count = tie.method_counter(
                method, "initial-dfs-minimum-deviation-cut-candidates"
            )
            partition_count = tie.method_counter(
                method, "initial-dfs-minimum-deviation-cut-partitions"
            )
            tie.require(
                0 < partition_count <= cut_count <= deviation_count,
                f"{state}/{path or 'root'}: invalid counts",
            )
            child_left, child_right = replay_node["child_seats"]
            node_rows.append(
                {
                    "state": state,
                    "path": path or "root",
                    "seats": replay_node["seats"],
                    "child_seats_left": child_left,
                    "child_seats_right": child_right,
                    "equal_child_seats": child_left == child_right,
                    "unit_count": len(
                        replay_discovery["objective"]["canonical_assignment"]
                    ),
                    "minimum_deviation_candidates": deviation_count,
                    "minimum_deviation_cut_candidates": cut_count,
                    "minimum_deviation_cut_partitions": partition_count,
                    "orientation_only_tie": cut_count > partition_count,
                    "physical_cut_opportunity": partition_count > 1,
                    "nrs_v0_2_fallback_activated": method_bool(
                        method, "nrs-v0-2-fallback-activated"
                    ),
                    "nrs_v0_3_fallback_activated": method_bool(
                        method, "nrs-v0-3-fallback-activated"
                    ),
                    "assignment_match": (
                        replay_discovery["objective"]["canonical_assignment"]
                        == benchmark_discovery["objective"]["canonical_assignment"]
                    ),
                    "objective_match": (
                        replay_discovery["objective"]["primary"]
                        == benchmark_discovery["objective"]["primary"]
                        == replay_node["objective"]
                        == benchmark_node["objective"]
                    ),
                }
            )
        state_assignment_match = (
            replay_tree["assignment"] == benchmark_tree["assignment"]
        )
        return (
            {
                "state": state,
                "districts": inputs["districts"],
                "node_count": len(node_rows),
                "status": "accepted",
                "failure": "",
                "state_assignment_match": state_assignment_match,
                "node_assignment_match_count": sum(
                    row["assignment_match"] for row in node_rows
                ),
                "node_objective_match_count": sum(
                    row["objective_match"] for row in node_rows
                ),
            },
            node_rows,
        )
    except (tie.CensusError, KeyError, TypeError, ValueError) as error:
        return (
            {
                "state": state,
                "districts": inputs["districts"],
                "status": "rejected",
                "failure": str(error),
            },
            [],
        )
    finally:
        if output.is_dir():
            shutil.rmtree(output)


def write_csv(path: Path, fields: list[str], rows: list[dict]) -> None:
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        for row in rows:
            writer.writerow(
                {
                    field: (
                        str(row.get(field, "")).lower()
                        if isinstance(row.get(field), bool)
                        else row.get(field, "")
                    )
                    for field in fields
                }
            )


def write_package(
    bisect: Path,
    ops: Path,
    output_dir: Path,
    workers: int,
    display_dir: str | None,
) -> None:
    tie.require(bisect.is_file(), f"missing executable {bisect}")
    tie.require(ops.is_file(), f"missing operations executable {ops}")
    tie.require(workers > 0, "workers must be positive")
    states_root = ROOT / "runs/nrs-v0.3/neutral-analysis/national-2020/states"
    inputs = []
    for state_dir in sorted(states_root.iterdir()):
        tree = json.loads(
            (state_dir / "package/baseline-tree.json").read_text(encoding="utf-8")
        )
        if tree["districts"] > 1:
            state = state_inputs(state_dir, bisect, ops)
            tie.require(state["context"].is_file(), f"missing {state['context']}")
            tie.require(state["seed"].is_dir(), f"missing {state['seed']}")
            inputs.append(state)
    tie.require(len(inputs) == 44, "expected 44 multi-district States")

    state_rows = []
    node_rows = []
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_root = Path(temp_dir)
        with ThreadPoolExecutor(max_workers=workers) as executor:
            futures = [
                executor.submit(run_state, state, temp_root) for state in inputs
            ]
            for future in as_completed(futures):
                state_row, rows = future.result()
                state_rows.append(state_row)
                node_rows.extend(rows)
    state_rows.sort(key=lambda row: row["state"])
    node_rows.sort(key=lambda row: (row["state"], len(row["path"]), row["path"]))
    accepted = [row for row in state_rows if row["status"] == "accepted"]
    rejected = [row for row in state_rows if row["status"] == "rejected"]
    expected_nodes = sum(row["node_count"] for row in accepted)
    status = (
        "pass"
        if len(accepted) == 44
        and not rejected
        and expected_nodes == 385
        and len(node_rows) == 385
        and all(row["state_assignment_match"] for row in accepted)
        and all(row["assignment_match"] and row["objective_match"] for row in node_rows)
        else "partial"
    )

    output_dir.mkdir(parents=True, exist_ok=True)
    state_csv = output_dir / "state-results.csv"
    node_csv = output_dir / "node-results.csv"
    write_csv(state_csv, STATE_FIELDS, state_rows)
    write_csv(node_csv, NODE_FIELDS, node_rows)
    physical_nodes = [
        f"{row['state']}/{row['path']}"
        for row in node_rows
        if row["physical_cut_opportunity"]
    ]
    v02_nodes = [
        f"{row['state']}/{row['path']}"
        for row in node_rows
        if row["nrs_v0_2_fallback_activated"]
    ]
    v03_nodes = [
        f"{row['state']}/{row['path']}"
        for row in node_rows
        if row["nrs_v0_3_fallback_activated"]
    ]
    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "census_year": 2020,
        "state_count": len(state_rows),
        "accepted_state_count": len(accepted),
        "rejected_state_count": len(rejected),
        "split_node_count": len(node_rows),
        "assignment_preserving_state_count": sum(
            row["state_assignment_match"] for row in accepted
        ),
        "assignment_preserving_node_count": sum(
            row["assignment_match"] for row in node_rows
        ),
        "objective_preserving_node_count": sum(
            row["objective_match"] for row in node_rows
        ),
        "minimum_deviation_cut_partition_distribution": tie.distribution(
            [row["minimum_deviation_cut_partitions"] for row in node_rows]
        ),
        "orientation_only_tie_node_count": sum(
            row["orientation_only_tie"] for row in node_rows
        ),
        "physical_cut_opportunity_node_count": len(physical_nodes),
        "physical_cut_opportunity_nodes": physical_nodes,
        "nrs_v0_2_fallback_activation_count": len(v02_nodes),
        "nrs_v0_2_fallback_activation_nodes": v02_nodes,
        "nrs_v0_3_fallback_activation_count": len(v03_nodes),
        "nrs_v0_3_fallback_activation_nodes": v03_nodes,
        "failures": [
            {"state": row["state"], "failure": row["failure"]} for row in rejected
        ],
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = output_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8")

    canonical_output = display_dir or tie.relative_path(output_dir)
    readme = f"""# NRS v0.3 Complete-Tree DFS Census

**Status:** {status}

| Measure | Result |
|---|---:|
| Multi-district States | {len(state_rows)} |
| Governed split nodes | {len(node_rows)} |
| Assignment-preserving States | {analysis['assignment_preserving_state_count']} |
| Assignment-preserving nodes | {analysis['assignment_preserving_node_count']} |
| Objective-preserving nodes | {analysis['objective_preserving_node_count']} |
| Nodes with multiple physical initial cuts | {len(physical_nodes)} |
| Nodes activating v0.2 fallback | {len(v02_nodes)} |
| Nodes activating v0.3 fallback | {len(v03_nodes)} |

The complete State and node ledgers are in `state-results.csv` and
`node-results.csv`.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_tree_dfs_census.py `
  --output-dir {canonical_output}
python scripts/research/verify_nrs_tree_dfs_census.py `
  {canonical_output}
```

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = output_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8")

    input_paths = [bisect, ops]
    for row in inputs:
        input_paths.extend([row["context"], row["benchmark_tree"]])
        input_paths.extend(sorted(path for path in row["seed"].iterdir() if path.is_file()))
        input_paths.extend(
            discovery_path(row["benchmark_package"], node["path"])
            for node in json.loads(
                row["benchmark_tree"].read_text(encoding="utf-8")
            )["nodes"]
        )
    code_paths = [
        PROTOCOL_PATH,
        ENGINE_PATH,
        OPS_PATH,
        BASE_RUNNER_PATH,
        RUNNER_PATH,
        VERIFIER_PATH,
    ]
    manifest = {
        "schema_version": MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "inputs": [
            {"path": tie.relative_path(path), "sha256": tie.sha256(path)}
            for path in input_paths
        ],
        "code": [
            {"path": path.as_posix(), "sha256": tie.sha256(ROOT / path)}
            for path in code_paths
        ],
        "outputs": {
            "analysis.json": tie.sha256(analysis_path),
            "state-results.csv": tie.sha256(state_csv),
            "node-results.csv": tie.sha256(node_csv),
            "README.md": tie.sha256(readme_path),
        },
        "reproduction": {
            "bisect": tie.relative_path(bisect),
            "ops": tie.relative_path(ops),
            "workers": workers,
            "display_output_dir": canonical_output,
        },
        "claim_boundary": CLAIM_BOUNDARY,
    }
    (output_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--bisect", type=Path, default=ROOT / "target/release/bisect.exe"
    )
    parser.add_argument(
        "--ops", type=Path, default=ROOT / "target/release/bisect-ops.exe"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=ROOT / "docs/experiments/nrs-v0.3-complete-tree-dfs-census-2020",
    )
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--display-output-dir")
    args = parser.parse_args()
    write_package(
        args.bisect.resolve(),
        args.ops.resolve(),
        args.output_dir.resolve(),
        args.workers,
        args.display_output_dir,
    )


if __name__ == "__main__":
    main()
