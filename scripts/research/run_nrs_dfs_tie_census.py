#!/usr/bin/env python3
"""Run the precommitted 2020 NRS initial DFS tie census."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PROTOCOL_ID = "nrs-v0.3-initial-dfs-tie-census-v1"
SCHEMA_VERSION = "nrs-v0.3-initial-dfs-tie-census-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-initial-dfs-tie-census-manifest-v1"
PROTOCOL_PATH = Path("docs/specs/2026-08-07-nrs-v0.3-dfs-tie-census-protocol.md")
RUNNER_PATH = Path("scripts/research/run_nrs_dfs_tie_census.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_dfs_tie_census.py")
ENGINE_PATH = Path("crates/bisect-cli/src/exact_cmd.rs")
GENERATED_AT = "2026-08-07T00:00:00Z"
CLAIM_BOUNDARY = (
    "Initial root-0 DFS candidate multiplicity for governed 2020 State roots "
    "only; no alternate-root fallback, child-node, final seed-sensitivity, "
    "national robustness, optimality, or legal-quality claim."
)
CSV_FIELDS = [
    "state",
    "districts",
    "child_seats_left",
    "child_seats_right",
    "unit_count",
    "benchmark_seed",
    "status",
    "failure",
    "minimum_deviation_candidates",
    "minimum_deviation_cut_candidates",
    "seed_sensitive_tie_opportunity",
    "assignment_match",
    "objective_match",
    "governed_assignment_sha256",
    "instrumented_assignment_sha256",
    "governed_discovery_sha256",
    "instrumented_discovery_sha256",
]


class CensusError(ValueError):
    """Structured census input or replay failure."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise CensusError(message)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def relative_path(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return str(path.resolve()).replace("\\", "/")


def assignment_sha256(assignment: list[int]) -> str:
    return hashlib.sha256(bytes(assignment)).hexdigest()


def method_counter(method: str, name: str) -> int:
    match = re.search(rf"(?:^|; ){re.escape(name)}=(\d+)(?:;|$)", method)
    require(match is not None, f"missing method counter {name}")
    return int(match.group(1))


def state_inputs(state_dir: Path, binary: Path) -> dict:
    state = state_dir.name.upper()
    package = state_dir / "package"
    return {
        "state": state,
        "binary": binary,
        "context": ROOT / f"data/2020/certified/{state.lower()}_blocks_2020.rctx",
        "tree": package / "baseline-tree.json",
        "benchmark_discovery": package / "nodes/root/certified-discovery.json",
    }


def sanitize_failure(text: str, temp_root: Path) -> str:
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    if not lines:
        return "command failed without diagnostic output"
    return lines[-1].replace(str(temp_root), "<TEMP>").replace(
        temp_root.as_posix(), "<TEMP>"
    )


def run_state(inputs: dict, temp_root: Path) -> dict:
    state = inputs["state"]
    seed_dir = temp_root / state.lower()
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
                str(seed_dir),
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
        base = {
            "state": state,
            "districts": tree["districts"],
            "child_seats_left": root_node["child_seats"][0],
            "child_seats_right": root_node["child_seats"][1],
            "unit_count": tree["unit_count"],
            "benchmark_seed": root_node["engine_seed_i32"],
        }
        if completed.returncode != 0:
            return {
                **base,
                "status": "rejected",
                "failure": sanitize_failure(
                    completed.stdout + completed.stderr, temp_root
                ),
            }
        discovery_path = seed_dir / "certified-discovery.json"
        require(discovery_path.is_file(), f"{state}: missing discovery")
        instrumented = json.loads(discovery_path.read_text(encoding="utf-8"))
        minimum_deviation = method_counter(
            instrumented["method"], "initial-dfs-minimum-deviation-candidates"
        )
        minimum_cut = method_counter(
            instrumented["method"],
            "initial-dfs-minimum-deviation-cut-candidates",
        )
        require(minimum_deviation > 0, f"{state}: nonpositive deviation count")
        require(
            0 < minimum_cut <= minimum_deviation,
            f"{state}: invalid minimum-cut count",
        )
        governed_assignment = benchmark["objective"]["canonical_assignment"]
        instrumented_assignment = instrumented["objective"]["canonical_assignment"]
        assignment_match = instrumented_assignment == governed_assignment
        objective_match = (
            instrumented["objective"]["primary"] == benchmark["objective"]["primary"]
        )
        return {
            **base,
            "status": "accepted",
            "failure": "",
            "minimum_deviation_candidates": minimum_deviation,
            "minimum_deviation_cut_candidates": minimum_cut,
            "seed_sensitive_tie_opportunity": minimum_cut > 1,
            "assignment_match": assignment_match,
            "objective_match": objective_match,
            "governed_assignment_sha256": assignment_sha256(governed_assignment),
            "instrumented_assignment_sha256": assignment_sha256(
                instrumented_assignment
            ),
            "governed_discovery_sha256": sha256(inputs["benchmark_discovery"]),
            "instrumented_discovery_sha256": sha256(discovery_path),
        }
    except (CensusError, KeyError, TypeError, ValueError) as error:
        return {
            "state": state,
            "status": "rejected",
            "failure": str(error),
        }
    finally:
        if seed_dir.is_dir():
            shutil.rmtree(seed_dir)


def distribution(values: list[int]) -> dict:
    require(bool(values), "empty distribution")
    counts = {str(value): values.count(value) for value in sorted(set(values))}
    return {
        "minimum": min(values),
        "maximum": max(values),
        "counts": counts,
    }


def write_package(binary: Path, output_dir: Path, workers: int, display_dir: str | None) -> None:
    require(binary.is_file(), f"missing executable {binary}")
    require(workers > 0, "workers must be positive")
    states_root = ROOT / "runs/nrs-v0.3/neutral-analysis/national-2020/states"
    inputs = []
    for state_dir in sorted(states_root.iterdir()):
        tree_path = state_dir / "package/baseline-tree.json"
        tree = json.loads(tree_path.read_text(encoding="utf-8"))
        if tree["districts"] > 1:
            state = state_inputs(state_dir, binary)
            for path in (state["context"], state["tree"], state["benchmark_discovery"]):
                require(path.is_file(), f"missing State input {path}")
            inputs.append(state)
    require(len(inputs) == 44, "expected 44 multi-district State roots")

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
    opportunities = [
        row["state"]
        for row in accepted
        if row["seed_sensitive_tie_opportunity"]
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
        "minimum_deviation_candidate_distribution": distribution(
            [row["minimum_deviation_candidates"] for row in accepted]
        ),
        "minimum_deviation_cut_candidate_distribution": distribution(
            [row["minimum_deviation_cut_candidates"] for row in accepted]
        ),
        "seed_sensitive_tie_opportunity_count": len(opportunities),
        "seed_sensitive_tie_opportunity_states": opportunities,
        "failures": [
            {"state": row["state"], "failure": row["failure"]} for row in rejected
        ],
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = output_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8")

    canonical_output = display_dir or relative_path(output_dir)
    states_text = ", ".join(opportunities) if opportunities else "none"
    readme = f"""# NRS v0.3 Initial DFS Tie Census

**Status:** {status}

| Measure | Result |
|---|---:|
| Multi-district State roots | {len(rows)} |
| Accepted replays | {len(accepted)} |
| Assignment-preserving replays | {analysis['assignment_preservation_count']} |
| Objective-preserving replays | {analysis['objective_preservation_count']} |
| Roots with initial seed-sensitive tie opportunity | {len(opportunities)} |

Opportunity States: {states_text}

The full State ledger and both candidate counts are in `state-results.csv`.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_dfs_tie_census.py `
  --output-dir {canonical_output}
python scripts/research/verify_nrs_dfs_tie_census.py `
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
    code_paths = [PROTOCOL_PATH, ENGINE_PATH, RUNNER_PATH, VERIFIER_PATH]
    manifest = {
        "schema_version": MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "inputs": [
            {"path": relative_path(path), "sha256": sha256(path)}
            for path in input_paths
        ],
        "code": [
            {"path": path.as_posix(), "sha256": sha256(ROOT / path)}
            for path in code_paths
        ],
        "outputs": {
            "analysis.json": sha256(analysis_path),
            "state-results.csv": sha256(csv_path),
            "README.md": sha256(readme_path),
        },
        "reproduction": {
            "binary": relative_path(binary),
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
        default=ROOT / "docs/experiments/nrs-v0.3-initial-dfs-tie-census-2020",
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
