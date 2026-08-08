#!/usr/bin/env python3
"""Run the precommitted 2000/2010 NRS complete-tree DFS census."""

from __future__ import annotations

import argparse
import csv
import json
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import run_nrs_dfs_tie_census as tie
import run_nrs_tree_dfs_census as tree


ROOT = Path(__file__).resolve().parents[2]
YEARS = (2000, 2010)
PROTOCOL_ID = "nrs-v0.3-cross-census-complete-tree-dfs-v1"
SCHEMA_VERSION = "nrs-v0.3-cross-census-complete-tree-dfs-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-cross-census-complete-tree-dfs-manifest-v1"
PROTOCOL_PATH = Path(
    "docs/specs/2026-08-08-nrs-v0.3-cross-census-dfs-protocol.md"
)
RUNNER_PATH = Path("scripts/research/run_nrs_cross_census_dfs.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_cross_census_dfs.py")
TREE_RUNNER_PATH = Path("scripts/research/run_nrs_tree_dfs_census.py")
TREE_VERIFIER_PATH = Path("scripts/research/verify_nrs_tree_dfs_census.py")
BASE_RUNNER_PATH = Path("scripts/research/run_nrs_dfs_tie_census.py")
ENGINE_PATH = Path("crates/bisect-cli/src/exact_cmd.rs")
OPS_PATH = Path("crates/bisect-ops/src/main.rs")
CLAIM_BOUNDARY = (
    "Governed 2000/2010 complete-tree initial DFS partition and fallback "
    "activation diagnostics only; no seed-invariant label or plan, national "
    "robustness, optimality, partisan, legal-quality, or assignment-overlap claim."
)
STATE_FIELDS = ["year", *tree.STATE_FIELDS]
NODE_FIELDS = ["year", *tree.NODE_FIELDS]


def census_state_inputs(
    year: int, state_dir: Path, bisect: Path, ops: Path
) -> dict:
    state = state_dir.name.upper()
    package = state_dir / "package"
    benchmark_tree = package / "baseline-tree.json"
    districts = json.loads(benchmark_tree.read_text(encoding="utf-8"))["districts"]
    return {
        "year": year,
        "state": state,
        "bisect": bisect,
        "ops": ops,
        "context": ROOT
        / f"data/{year}/certified/{state.lower()}_blocks_{year}.rctx",
        "seed": state_dir / "seed",
        "benchmark_package": package,
        "benchmark_tree": benchmark_tree,
        "districts": districts,
    }


def run_census_state(inputs: dict, temp_root: Path) -> tuple[dict, list[dict]]:
    year = inputs["year"]
    state_row, node_rows = tree.run_state(inputs, temp_root / str(year))
    return (
        {"year": year, **state_row},
        [{"year": year, **row} for row in node_rows],
    )


def write_csv(path: Path, fields: list[str], rows: list[dict]) -> None:
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields, lineterminator="\n")
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
    inputs = []
    for year in YEARS:
        states_root = (
            ROOT / f"runs/nrs-v0.3/neutral-analysis/national-{year}/states"
        )
        year_count = 0
        for state_dir in sorted(states_root.iterdir()):
            state = census_state_inputs(year, state_dir, bisect, ops)
            if state["districts"] > 1:
                tie.require(state["context"].is_file(), f"missing {state['context']}")
                tie.require(state["seed"].is_dir(), f"missing {state['seed']}")
                inputs.append(state)
                year_count += 1
        tie.require(year_count == 43, f"expected 43 multi-district States in {year}")

    state_rows = []
    node_rows = []
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_root = Path(temp_dir)
        for year in YEARS:
            (temp_root / str(year)).mkdir()
        with ThreadPoolExecutor(max_workers=workers) as executor:
            futures = [
                executor.submit(run_census_state, state, temp_root)
                for state in inputs
            ]
            for future in as_completed(futures):
                state_row, rows = future.result()
                state_rows.append(state_row)
                node_rows.extend(rows)
    state_rows.sort(key=lambda row: (row["year"], row["state"]))
    node_rows.sort(
        key=lambda row: (
            row["year"],
            row["state"],
            len(row["path"]),
            row["path"],
        )
    )
    accepted = [row for row in state_rows if row["status"] == "accepted"]
    rejected = [row for row in state_rows if row["status"] == "rejected"]
    status = (
        "pass"
        if len(accepted) == 86
        and not rejected
        and len(node_rows) == 770
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
        f"{row['year']}/{row['state']}/{row['path']}"
        for row in node_rows
        if row["physical_cut_opportunity"]
    ]
    v02_nodes = [
        f"{row['year']}/{row['state']}/{row['path']}"
        for row in node_rows
        if row["nrs_v0_2_fallback_activated"]
    ]
    v03_nodes = [
        f"{row['year']}/{row['state']}/{row['path']}"
        for row in node_rows
        if row["nrs_v0_3_fallback_activated"]
    ]
    year_summaries = {}
    for year in YEARS:
        year_states = [row for row in state_rows if row["year"] == year]
        year_nodes = [row for row in node_rows if row["year"] == year]
        year_summaries[str(year)] = {
            "state_count": len(year_states),
            "split_node_count": len(year_nodes),
            "orientation_only_tie_node_count": sum(
                row["orientation_only_tie"] for row in year_nodes
            ),
            "physical_cut_opportunity_node_count": sum(
                row["physical_cut_opportunity"] for row in year_nodes
            ),
            "nrs_v0_2_fallback_activation_count": sum(
                row["nrs_v0_2_fallback_activated"] for row in year_nodes
            ),
            "nrs_v0_3_fallback_activation_count": sum(
                row["nrs_v0_3_fallback_activated"] for row in year_nodes
            ),
        }
    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "census_years": list(YEARS),
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
        "year_summaries": year_summaries,
        "failures": [
            {
                "year": row["year"],
                "state": row["state"],
                "failure": row["failure"],
            }
            for row in rejected
        ],
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = output_dir / "analysis.json"
    analysis_path.write_text(
        json.dumps(analysis, indent=2) + "\n", encoding="utf-8", newline="\n"
    )

    canonical_output = display_dir or tie.relative_path(output_dir)
    readme = f"""# NRS v0.3 Cross-Census Complete-Tree DFS Census

**Status:** {status}

| Measure | Result |
|---|---:|
| Census years | 2000, 2010 |
| Multi-district State packages | {len(state_rows)} |
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
python scripts/research/run_nrs_cross_census_dfs.py `
  --output-dir {canonical_output}
python scripts/research/verify_nrs_cross_census_dfs.py `
  {canonical_output}
```

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = output_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8", newline="\n")

    input_paths = [bisect, ops]
    for row in inputs:
        input_paths.extend([row["context"], row["benchmark_tree"]])
        input_paths.extend(sorted(path for path in row["seed"].iterdir() if path.is_file()))
        benchmark_tree = json.loads(
            row["benchmark_tree"].read_text(encoding="utf-8")
        )
        input_paths.extend(
            tree.discovery_path(row["benchmark_package"], node["path"])
            for node in benchmark_tree["nodes"]
        )
    code_paths = [
        PROTOCOL_PATH,
        ENGINE_PATH,
        OPS_PATH,
        BASE_RUNNER_PATH,
        TREE_RUNNER_PATH,
        TREE_VERIFIER_PATH,
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
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n"
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
        default=ROOT
        / "docs/experiments/nrs-v0.3-cross-census-complete-tree-dfs",
    )
    parser.add_argument("--workers", type=int, default=1)
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
