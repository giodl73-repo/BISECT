#!/usr/bin/env python3
"""Run the precommitted 2020 NRS initial DFS partition census."""

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
PROTOCOL_ID = "nrs-v0.3-initial-dfs-partition-census-v1"
SCHEMA_VERSION = "nrs-v0.3-initial-dfs-partition-census-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-initial-dfs-partition-census-manifest-v1"
PROTOCOL_PATH = Path(
    "docs/specs/2026-08-08-nrs-v0.3-dfs-partition-census-protocol.md"
)
RUNNER_PATH = Path("scripts/research/run_nrs_dfs_partition_census.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_dfs_partition_census.py")
BASE_RUNNER_PATH = Path("scripts/research/run_nrs_dfs_tie_census.py")
ENGINE_PATH = Path("crates/bisect-cli/src/exact_cmd.rs")
GENERATED_AT = "2026-08-08T00:00:00Z"
CLAIM_BOUNDARY = (
    "Unlabeled initial root-0 DFS tree-edge bipartitions for governed 2020 "
    "State roots only; no seed-invariant label, child-node, fallback, "
    "full-plan, national robustness, optimality, or legal-quality claim."
)
CSV_FIELDS = [
    "state",
    "districts",
    "child_seats_left",
    "child_seats_right",
    "equal_child_seats",
    "unit_count",
    "benchmark_seed",
    "status",
    "failure",
    "minimum_deviation_candidates",
    "minimum_deviation_cut_candidates",
    "minimum_deviation_cut_partitions",
    "orientation_only_tie",
    "physical_cut_opportunity",
    "assignment_match",
    "objective_match",
    "governed_assignment_sha256",
    "instrumented_assignment_sha256",
]


def run_state(inputs: dict, temp_root: Path) -> dict:
    state = inputs["state"]
    replay_dir = temp_root / state.lower()
    try:
        tree = json.loads(inputs["tree"].read_text(encoding="utf-8"))
        root_node = next(node for node in tree["nodes"] if node["path"] == "")
        benchmark = json.loads(
            inputs["benchmark_discovery"].read_text(encoding="utf-8")
        )
        completed = subprocess.run(
            [
                str(inputs["binary"]),
                "exact",
                "--context",
                str(inputs["context"]),
                "--districts",
                str(tree["districts"]),
                "--method",
                "certified-discovery",
                "--out-dir",
                str(replay_dir),
                "--generated-at",
                GENERATED_AT,
                "--discovery-seed",
                str(root_node["engine_seed_i32"]),
                "--discovery-refinement",
                "nrs-v0-3",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        left_seats, right_seats = root_node["child_seats"]
        base = {
            "state": state,
            "districts": tree["districts"],
            "child_seats_left": left_seats,
            "child_seats_right": right_seats,
            "equal_child_seats": left_seats == right_seats,
            "unit_count": tree["unit_count"],
            "benchmark_seed": root_node["engine_seed_i32"],
        }
        if completed.returncode != 0:
            return {
                **base,
                "status": "rejected",
                "failure": tie.sanitize_failure(
                    completed.stdout + completed.stderr, temp_root
                ),
            }
        discovery_path = replay_dir / "certified-discovery.json"
        tie.require(discovery_path.is_file(), f"{state}: missing discovery")
        instrumented = json.loads(discovery_path.read_text(encoding="utf-8"))
        method = instrumented["method"]
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
            f"{state}: invalid candidate or partition counts",
        )
        governed_assignment = benchmark["objective"]["canonical_assignment"]
        instrumented_assignment = instrumented["objective"]["canonical_assignment"]
        return {
            **base,
            "status": "accepted",
            "failure": "",
            "minimum_deviation_candidates": deviation_count,
            "minimum_deviation_cut_candidates": cut_count,
            "minimum_deviation_cut_partitions": partition_count,
            "orientation_only_tie": cut_count > partition_count,
            "physical_cut_opportunity": partition_count > 1,
            "assignment_match": instrumented_assignment == governed_assignment,
            "objective_match": (
                instrumented["objective"]["primary"]
                == benchmark["objective"]["primary"]
            ),
            "governed_assignment_sha256": tie.assignment_sha256(
                governed_assignment
            ),
            "instrumented_assignment_sha256": tie.assignment_sha256(
                instrumented_assignment
            ),
        }
    except (tie.CensusError, KeyError, StopIteration, TypeError, ValueError) as error:
        return {"state": state, "status": "rejected", "failure": str(error)}
    finally:
        if replay_dir.is_dir():
            shutil.rmtree(replay_dir)


def write_package(
    binary: Path, output_dir: Path, workers: int, display_dir: str | None
) -> None:
    tie.require(binary.is_file(), f"missing executable {binary}")
    tie.require(workers > 0, "workers must be positive")
    states_root = ROOT / "runs/nrs-v0.3/neutral-analysis/national-2020/states"
    inputs = []
    for state_dir in sorted(states_root.iterdir()):
        tree_path = state_dir / "package/baseline-tree.json"
        tree = json.loads(tree_path.read_text(encoding="utf-8"))
        if tree["districts"] > 1:
            state = tie.state_inputs(state_dir, binary)
            for path in (
                state["context"],
                state["tree"],
                state["benchmark_discovery"],
            ):
                tie.require(path.is_file(), f"missing State input {path}")
            inputs.append(state)
    tie.require(len(inputs) == 44, "expected 44 multi-district State roots")

    rows = []
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_root = Path(temp_dir)
        with ThreadPoolExecutor(max_workers=workers) as executor:
            futures = [
                executor.submit(run_state, state_input, temp_root)
                for state_input in inputs
            ]
            for future in as_completed(futures):
                rows.append(future.result())
    rows.sort(key=lambda row: row["state"])
    accepted = [row for row in rows if row["status"] == "accepted"]
    rejected = [row for row in rows if row["status"] == "rejected"]
    status = (
        "pass"
        if len(accepted) == 44
        and not rejected
        and all(row["assignment_match"] and row["objective_match"] for row in accepted)
        else "partial"
    )
    orientation_states = [
        row["state"] for row in accepted if row["orientation_only_tie"]
    ]
    physical_cut_states = [
        row["state"] for row in accepted if row["physical_cut_opportunity"]
    ]

    output_dir.mkdir(parents=True, exist_ok=True)
    csv_path = output_dir / "state-results.csv"
    with csv_path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=CSV_FIELDS)
        writer.writeheader()
        for row in rows:
            writer.writerow(
                {
                    field: (
                        str(row.get(field, "")).lower()
                        if isinstance(row.get(field), bool)
                        else row.get(field, "")
                    )
                    for field in CSV_FIELDS
                }
            )

    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "census_year": 2020,
        "state_root_count": len(rows),
        "accepted_state_count": len(accepted),
        "rejected_state_count": len(rejected),
        "assignment_preservation_count": sum(
            row["assignment_match"] for row in accepted
        ),
        "objective_preservation_count": sum(
            row["objective_match"] for row in accepted
        ),
        "minimum_deviation_cut_candidate_distribution": tie.distribution(
            [row["minimum_deviation_cut_candidates"] for row in accepted]
        ),
        "minimum_deviation_cut_partition_distribution": tie.distribution(
            [row["minimum_deviation_cut_partitions"] for row in accepted]
        ),
        "orientation_only_tie_count": len(orientation_states),
        "orientation_only_tie_states": orientation_states,
        "physical_cut_opportunity_count": len(physical_cut_states),
        "physical_cut_opportunity_states": physical_cut_states,
        "failures": [
            {"state": row["state"], "failure": row["failure"]} for row in rejected
        ],
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = output_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8")

    canonical_output = display_dir or tie.relative_path(output_dir)
    physical_text = ", ".join(physical_cut_states) if physical_cut_states else "none"
    readme = f"""# NRS v0.3 Initial DFS Partition Census

**Status:** {status}

| Measure | Result |
|---|---:|
| Multi-district State roots | {len(rows)} |
| Accepted replays | {len(accepted)} |
| Assignment-preserving replays | {analysis['assignment_preservation_count']} |
| Objective-preserving replays | {analysis['objective_preservation_count']} |
| Roots with orientation-only ties | {len(orientation_states)} |
| Roots with multiple physical cuts | {len(physical_cut_states)} |

Physical-cut opportunity States: {physical_text}

The complete oriented and unlabeled count ledger is in `state-results.csv`.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_dfs_partition_census.py `
  --output-dir {canonical_output}
python scripts/research/verify_nrs_dfs_partition_census.py `
  {canonical_output}
```

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = output_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8")

    input_paths = [binary]
    for row in inputs:
        input_paths.extend(
            [row["context"], row["tree"], row["benchmark_discovery"]]
        )
    code_paths = [
        PROTOCOL_PATH,
        ENGINE_PATH,
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
            "state-results.csv": tie.sha256(csv_path),
            "README.md": tie.sha256(readme_path),
        },
        "reproduction": {
            "binary": tie.relative_path(binary),
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
        "--binary", type=Path, default=ROOT / "target/release/bisect.exe"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=ROOT
        / "docs/experiments/nrs-v0.3-initial-dfs-partition-census-2020",
    )
    parser.add_argument("--workers", type=int, default=6)
    parser.add_argument("--display-output-dir")
    args = parser.parse_args()
    write_package(
        args.binary.resolve(),
        args.output_dir.resolve(),
        args.workers,
        args.display_output_dir,
    )


if __name__ == "__main__":
    main()
