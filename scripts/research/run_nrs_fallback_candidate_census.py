#!/usr/bin/env python3
"""Run the precommitted NRS fallback candidate census."""

from __future__ import annotations

import argparse
import csv
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

import run_nrs_dfs_tie_census as tie
import run_nrs_tree_dfs_census as tree


ROOT = Path(__file__).resolve().parents[2]
PROTOCOL_ID = "nrs-v0.3-fallback-candidate-census-v1"
SCHEMA_VERSION = "nrs-v0.3-fallback-candidate-census-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-fallback-candidate-census-manifest-v1"
GENERATED_AT = "2026-08-08T00:00:00Z"
PROTOCOL_PATH = Path(
    "docs/specs/2026-08-08-nrs-v0.3-fallback-candidate-census-protocol.md"
)
RUNNER_PATH = Path("scripts/research/run_nrs_fallback_candidate_census.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_fallback_candidate_census.py")
BASE_RUNNER_PATH = Path("scripts/research/run_nrs_dfs_tie_census.py")
TREE_RUNNER_PATH = Path("scripts/research/run_nrs_tree_dfs_census.py")
ENGINE_PATH = Path("crates/bisect-cli/src/exact_cmd.rs")
OPS_PATH = Path("crates/bisect-ops/src/main.rs")
CLAIM_BOUNDARY = (
    "Candidates evaluated by the current v0.2 and v0.3 fallback algorithms at "
    "eight activated governed stage/node pairs only; no seed-invariant "
    "candidate generation, label, plan, robustness, optimality, partisan, or "
    "legal-quality claim."
)
TARGETS = {
    (2000, "AZ"): {"10": ("v0.2",)},
    (2000, "CA"): {"11010": ("v0.2",)},
    (2000, "GA"): {"100": ("v0.2",)},
    (2000, "HI"): {"": ("v0.2", "v0.3")},
    (2000, "TX"): {"100": ("v0.2",), "0100": ("v0.2",)},
    (2010, "CA"): {"00110": ("v0.2",)},
}
STAGE_FIELDS = [
    "year",
    "state",
    "path",
    "stage",
    "evaluated_candidates",
    "minimum_deviation_candidates",
    "minimum_deviation_cut_candidates",
    "minimum_deviation_cut_partitions",
    "physical_partition_opportunity",
    "assignment_match",
    "objective_match",
]
STATE_FIELDS = [
    "year",
    "state",
    "districts",
    "node_count",
    "status",
    "failure",
    "state_assignment_match",
    "node_assignment_match_count",
    "node_objective_match_count",
]


def stage_metrics(method: str, stage: str) -> dict:
    prefix = f"nrs-{stage.replace('.', '-')}-fallback"
    evaluated = tie.method_counter(method, f"{prefix}-evaluated-candidates")
    deviation = tie.method_counter(
        method, f"{prefix}-minimum-deviation-candidates"
    )
    cut = tie.method_counter(
        method, f"{prefix}-minimum-deviation-cut-candidates"
    )
    partitions = tie.method_counter(
        method, f"{prefix}-minimum-deviation-cut-partitions"
    )
    tie.require(
        0 < partitions <= cut <= deviation <= evaluated,
        f"invalid {stage} fallback diagnostics",
    )
    return {
        "evaluated_candidates": evaluated,
        "minimum_deviation_candidates": deviation,
        "minimum_deviation_cut_candidates": cut,
        "minimum_deviation_cut_partitions": partitions,
        "physical_partition_opportunity": partitions > 1,
    }


def state_inputs(year: int, state: str, bisect: Path, ops: Path) -> dict:
    state_dir = (
        ROOT
        / f"runs/nrs-v0.3/neutral-analysis/national-{year}/states/{state.lower()}"
    )
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
        "targets": TARGETS[(year, state)],
    }


def run_state(inputs: dict, temp_root: Path) -> tuple[dict, list[dict]]:
    year = inputs["year"]
    state = inputs["state"]
    output = temp_root / str(year) / state.lower()
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
                    "year": year,
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
            f"{year}/{state}: node path universe changed",
        )
        node_assignment_matches = 0
        node_objective_matches = 0
        stage_rows = []
        for path in sorted(benchmark_nodes, key=lambda value: (len(value), value)):
            benchmark_discovery = json.loads(
                tree.discovery_path(inputs["benchmark_package"], path).read_text(
                    encoding="utf-8"
                )
            )
            replay_discovery = json.loads(
                tree.discovery_path(output, path).read_text(encoding="utf-8")
            )
            assignment_match = (
                replay_discovery["objective"]["canonical_assignment"]
                == benchmark_discovery["objective"]["canonical_assignment"]
            )
            objective_match = (
                replay_discovery["objective"]["primary"]
                == benchmark_discovery["objective"]["primary"]
                == replay_nodes[path]["objective"]
                == benchmark_nodes[path]["objective"]
            )
            node_assignment_matches += assignment_match
            node_objective_matches += objective_match
            for stage in inputs["targets"].get(path, ()):
                method = replay_discovery["method"]
                method_stage = stage.replace(".", "-")
                tie.require(
                    tree.method_bool(
                        method, f"nrs-{method_stage}-fallback-activated"
                    ),
                    f"{year}/{state}/{path or 'root'}: {stage} did not activate",
                )
                stage_rows.append(
                    {
                        "year": year,
                        "state": state,
                        "path": path or "root",
                        "stage": stage,
                        **stage_metrics(method, stage),
                        "assignment_match": assignment_match,
                        "objective_match": objective_match,
                    }
                )
        return (
            {
                "year": year,
                "state": state,
                "districts": inputs["districts"],
                "node_count": len(replay_nodes),
                "status": "accepted",
                "failure": "",
                "state_assignment_match": (
                    replay_tree["assignment"] == benchmark_tree["assignment"]
                ),
                "node_assignment_match_count": node_assignment_matches,
                "node_objective_match_count": node_objective_matches,
            },
            stage_rows,
        )
    except (tie.CensusError, KeyError, TypeError, ValueError) as error:
        return (
            {
                "year": year,
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
    bisect: Path, ops: Path, output_dir: Path, display_dir: str | None
) -> None:
    tie.require(bisect.is_file(), f"missing executable {bisect}")
    tie.require(ops.is_file(), f"missing operations executable {ops}")
    inputs = [
        state_inputs(year, state, bisect, ops)
        for year, state in sorted(TARGETS)
    ]
    for row in inputs:
        tie.require(row["context"].is_file(), f"missing {row['context']}")
        tie.require(row["seed"].is_dir(), f"missing {row['seed']}")

    state_rows = []
    stage_rows = []
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_root = Path(temp_dir)
        for year in (2000, 2010):
            (temp_root / str(year)).mkdir()
        for inputs_row in inputs:
            state_row, rows = run_state(inputs_row, temp_root)
            state_rows.append(state_row)
            stage_rows.extend(rows)
    state_rows.sort(key=lambda row: (row["year"], row["state"]))
    stage_rows.sort(
        key=lambda row: (
            row["year"],
            row["state"],
            len(row["path"]),
            row["path"],
            row["stage"],
        )
    )
    accepted = [row for row in state_rows if row["status"] == "accepted"]
    rejected = [row for row in state_rows if row["status"] == "rejected"]
    status = (
        "pass"
        if len(accepted) == 6
        and not rejected
        and len(stage_rows) == 8
        and all(row["state_assignment_match"] for row in accepted)
        and all(row["assignment_match"] and row["objective_match"] for row in stage_rows)
        else "partial"
    )

    output_dir.mkdir(parents=True, exist_ok=True)
    state_csv = output_dir / "state-results.csv"
    stage_csv = output_dir / "stage-results.csv"
    write_csv(state_csv, STATE_FIELDS, state_rows)
    write_csv(stage_csv, STAGE_FIELDS, stage_rows)
    opportunities = [
        f"{row['year']}/{row['state']}/{row['path']}/{row['stage']}"
        for row in stage_rows
        if row["physical_partition_opportunity"]
    ]
    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "state_package_count": len(state_rows),
        "activated_stage_node_count": len(stage_rows),
        "assignment_preserving_state_count": sum(
            row["state_assignment_match"] for row in accepted
        ),
        "assignment_preserving_stage_node_count": sum(
            row["assignment_match"] for row in stage_rows
        ),
        "objective_preserving_stage_node_count": sum(
            row["objective_match"] for row in stage_rows
        ),
        "physical_partition_opportunity_count": len(opportunities),
        "physical_partition_opportunities": opportunities,
        "stage_results": [
            {
                key: row[key]
                for key in (
                    "year",
                    "state",
                    "path",
                    "stage",
                    "evaluated_candidates",
                    "minimum_deviation_candidates",
                    "minimum_deviation_cut_candidates",
                    "minimum_deviation_cut_partitions",
                )
            }
            for row in stage_rows
        ],
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
    readme = f"""# NRS v0.3 Fallback Candidate Census

**Status:** {status}

| Measure | Result |
|---|---:|
| Replayed State packages | {len(state_rows)} |
| Activated stage/node pairs | {len(stage_rows)} |
| Assignment-preserving States | {analysis['assignment_preserving_state_count']} |
| Assignment-preserving stage/nodes | {analysis['assignment_preserving_stage_node_count']} |
| Objective-preserving stage/nodes | {analysis['objective_preserving_stage_node_count']} |
| Stage/nodes with multiple tied physical partitions | {len(opportunities)} |

The exact stage and State ledgers are in `stage-results.csv` and
`state-results.csv`.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_fallback_candidate_census.py `
  --output-dir {canonical_output}
python scripts/research/verify_nrs_fallback_candidate_census.py `
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
            "stage-results.csv": tie.sha256(stage_csv),
            "README.md": tie.sha256(readme_path),
        },
        "reproduction": {
            "bisect": tie.relative_path(bisect),
            "ops": tie.relative_path(ops),
            "display_output_dir": canonical_output,
            "workers": 1,
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
        default=ROOT / "docs/experiments/nrs-v0.3-fallback-candidate-census",
    )
    parser.add_argument("--display-output-dir")
    args = parser.parse_args()
    write_package(
        args.bisect.resolve(),
        args.ops.resolve(),
        args.output_dir.resolve(),
        args.display_output_dir,
    )


if __name__ == "__main__":
    main()
