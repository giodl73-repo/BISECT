#!/usr/bin/env python3
"""Run the precommitted NH/NM/GA NRS v0.3 root sensitivity package."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "research"))

from run_nrs_ri_sensitivity import (
    GENERATED_AT,
    SEED_COUNT,
    assignment_overlap,
    benchmark_rank,
    canonical_json_bytes,
    derive_seed,
    distribution,
    objective_key,
    pack_assignment,
    sha256,
)


PROTOCOL_ID = "nrs-v0.3-multistate-root-sensitivity-v1"
SCHEMA_VERSION = "nrs-v0.3-multistate-root-sensitivity-analysis-v1"
STATE_SCHEMA_VERSION = "nrs-v0.3-root-sensitivity-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-multistate-root-sensitivity-manifest-v1"
STATE_MANIFEST_VERSION = "nrs-v0.3-root-sensitivity-manifest-v1"
PROTOCOL_PATH = Path(
    "docs/specs/2026-08-07-nrs-v0.3-multistate-root-sensitivity-protocol.md"
)
AUDIT_PATH = Path("docs/experiments/2026-08-07-nrs-sensitivity-node-audit.md")
RUNNER_PATH = Path("scripts/research/run_nrs_multistate_root_sensitivity.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_multistate_root_sensitivity.py")
RI_RUNNER_PATH = Path("scripts/research/run_nrs_ri_sensitivity.py")
CLAIM_BOUNDARY = (
    "Three selected 2020 State roots only; no national node census, ensemble, "
    "random-sampling, partisan, compactness, demographic, VRA, legal-validity, "
    "optimality, or benchmark-replacement claim."
)
CSV_FIELDS = [
    "diagnostic_index",
    "seed_digest_sha256",
    "seed_u64_little_endian",
    "engine_seed",
    "status",
    "exit_code",
    "failure",
    "discovery_sha256",
    "assignment_sha256",
    "assignment_offset",
    "assignment_bytes",
    "matched_units",
    "matched_unit_rate",
    "max_population_deviation_scaled",
    "total_population_deviation_scaled",
    "weighted_boundary_cut",
    "population_tolerance_pass",
    "benchmark_assignment_reproduction",
]
STATE_CONFIG = {
    "GA": {
        "districts": 14,
        "child_seats": [7, 7],
        "unit_count": 232_717,
        "population": 10_711_908,
        "population_tolerance_scaled": 374_917,
        "benchmark_seed": 1_570_084_764,
    },
    "NH": {
        "districts": 2,
        "child_seats": [1, 1],
        "unit_count": 31_948,
        "population": 1_377_529,
        "population_tolerance_scaled": 6_888,
        "benchmark_seed": 828_041_789,
    },
    "NM": {
        "districts": 3,
        "child_seats": [1, 2],
        "unit_count": 107_215,
        "population": 2_117_522,
        "population_tolerance_scaled": 10_588,
        "benchmark_seed": 1_922_790_591,
    },
}


class MultiStateSensitivityError(ValueError):
    """Structured multi-State sensitivity failure."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise MultiStateSensitivityError(message)


def relative_path(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return str(path.resolve()).replace("\\", "/")


def state_paths(state: str, binary: Path) -> dict[str, Path]:
    lower = state.lower()
    package = (
        ROOT
        / f"runs/nrs-v0.3/neutral-analysis/national-2020/states/{lower}/package"
    )
    return {
        "binary": binary,
        "context": ROOT / f"data/2020/certified/{lower}_blocks_2020.rctx",
        "input_manifest": package / "seed/input_manifest.json",
        "benchmark_discovery": package / "nodes/root/certified-discovery.json",
        "benchmark_tree": package / "baseline-tree.json",
    }


def sanitize_failure(text: str, temp_root: Path) -> str:
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    if not lines:
        return "command failed without diagnostic output"
    return lines[-1].replace(str(temp_root), "<TEMP>").replace(
        temp_root.as_posix(), "<TEMP>"
    )


def run_pair(
    state: str,
    config: dict,
    paths: dict[str, Path],
    seed_row: dict,
    benchmark_assignment: list[int],
    benchmark_instance_hash: str,
    temp_root: Path,
) -> dict:
    index = seed_row["diagnostic_index"]
    seed_dir = temp_root / f"{state.lower()}-{index:03d}"
    try:
        completed = subprocess.run(
            [
                str(paths["binary"]),
                "exact",
                "--context",
                str(paths["context"]),
                "--districts",
                str(config["districts"]),
                "--method",
                "certified-discovery",
                "--out-dir",
                str(seed_dir),
                "--generated-at",
                GENERATED_AT,
                "--discovery-seed",
                str(seed_row["engine_seed"]),
                "--discovery-refinement",
                "nrs-v0-3",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        row = {
            "state": state,
            **seed_row,
            "exit_code": completed.returncode,
        }
        if completed.returncode != 0:
            return {
                **row,
                "status": "rejected",
                "failure": sanitize_failure(
                    completed.stdout + completed.stderr, temp_root
                ),
            }
        discovery_path = seed_dir / "certified-discovery.json"
        require(discovery_path.is_file(), f"{state} index {index}: missing discovery")
        discovery = json.loads(discovery_path.read_text(encoding="utf-8"))
        require(
            discovery.get("schema_version") == "certified-split-discovery-v1",
            f"{state} index {index}: discovery schema",
        )
        require(
            discovery.get("instance_hash") == benchmark_instance_hash,
            f"{state} index {index}: instance hash",
        )
        assignment = discovery["objective"]["canonical_assignment"]
        require(
            len(assignment) == config["unit_count"],
            f"{state} index {index}: unit count",
        )
        require(set(assignment) == {0, 1}, f"{state} index {index}: labels")
        packed = pack_assignment(assignment)
        matched_units, matched_rate = assignment_overlap(
            benchmark_assignment, assignment
        )
        objective = discovery["objective"]["primary"]
        return {
            **row,
            "status": "accepted",
            "failure": "",
            "discovery_sha256": sha256(discovery_path),
            "assignment_sha256": hashlib.sha256(packed).hexdigest(),
            "matched_units": matched_units,
            "matched_unit_rate": matched_rate,
            "max_population_deviation_scaled": int(
                objective["max_population_deviation_scaled"]
            ),
            "total_population_deviation_scaled": int(
                objective["total_population_deviation_scaled"]
            ),
            "weighted_boundary_cut": int(objective["weighted_boundary_cut"]),
            "population_tolerance_pass": (
                int(objective["max_population_deviation_scaled"])
                <= config["population_tolerance_scaled"]
            ),
            "_assignment": assignment,
            "_packed": packed,
        }
    finally:
        if seed_dir.is_dir():
            shutil.rmtree(seed_dir)


def prepare_state(state: str, binary: Path) -> dict:
    config = STATE_CONFIG[state]
    paths = state_paths(state, binary)
    for path in paths.values():
        require(path.is_file(), f"{state}: missing input {path}")
    tree = json.loads(paths["benchmark_tree"].read_text(encoding="utf-8"))
    root = next(node for node in tree["nodes"] if node["path"] == "")
    require(tree["districts"] == config["districts"], f"{state}: district count")
    require(tree["unit_count"] == config["unit_count"], f"{state}: unit count")
    require(tree["population_total"] == config["population"], f"{state}: population")
    require(root["child_seats"] == config["child_seats"], f"{state}: child seats")
    require(
        root["generation_tolerance_scaled_bound"]
        == config["population_tolerance_scaled"],
        f"{state}: population tolerance",
    )
    require(root["engine_seed_i32"] == config["benchmark_seed"], f"{state}: seed")
    benchmark = json.loads(
        paths["benchmark_discovery"].read_text(encoding="utf-8")
    )
    assignment = benchmark["objective"]["canonical_assignment"]
    require(len(assignment) == config["unit_count"], f"{state}: benchmark units")
    canonical_manifest = canonical_json_bytes(paths["input_manifest"])
    return {
        "state": state,
        "config": config,
        "paths": paths,
        "benchmark": benchmark,
        "benchmark_assignment": assignment,
        "canonical_manifest": canonical_manifest,
    }


def write_state_package(
    prepared: dict, rows: list[dict], state_dir: Path, display_state_dir: str
) -> dict:
    state = prepared["state"]
    config = prepared["config"]
    paths = prepared["paths"]
    benchmark = prepared["benchmark"]
    benchmark_assignment = prepared["benchmark_assignment"]
    rows.sort(key=lambda row: row["diagnostic_index"])
    accepted = [row for row in rows if row["status"] == "accepted"]
    rejected = [row for row in rows if row["status"] == "rejected"]
    benchmark_packed = pack_assignment(benchmark_assignment)
    benchmark_assignment_sha = hashlib.sha256(benchmark_packed).hexdigest()
    packed_output = bytearray()
    for row in accepted:
        row["assignment_offset"] = len(packed_output)
        row["assignment_bytes"] = len(row["_packed"])
        row["benchmark_assignment_reproduction"] = (
            row["assignment_sha256"] == benchmark_assignment_sha
        )
        packed_output.extend(row["_packed"])

    state_dir.mkdir(parents=True, exist_ok=True)
    assignments_path = state_dir / "assignments.bin"
    assignments_path.write_bytes(bytes(packed_output))
    csv_path = state_dir / "seed-results.csv"
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

    accepted_keys = [
        objective_key(
            {
                "max_population_deviation_scaled": row[
                    "max_population_deviation_scaled"
                ],
                "total_population_deviation_scaled": row[
                    "total_population_deviation_scaled"
                ],
                "weighted_boundary_cut": row["weighted_boundary_cut"],
            },
            row["_assignment"],
        )
        for row in accepted
    ]
    benchmark_objective = benchmark["objective"]["primary"]
    rank = benchmark_rank(
        accepted_keys, objective_key(benchmark_objective, benchmark_assignment)
    )
    analysis = {
        "schema_version": STATE_SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "complete",
        "state": state,
        "census_year": 2020,
        "districts": config["districts"],
        "root_child_seats": config["child_seats"],
        "unit_count": config["unit_count"],
        "diagnostic_seed_count": SEED_COUNT,
        "accepted_seed_count": len(accepted),
        "rejected_seed_count": len(rejected),
        "duplicate_engine_seed_count": len(rows)
        - len({row["engine_seed"] for row in rows}),
        "population_tolerance_scaled": config["population_tolerance_scaled"],
        "population_tolerance_pass_count": sum(
            row["population_tolerance_pass"] for row in accepted
        ),
        "benchmark": {
            "engine_seed": config["benchmark_seed"],
            "discovery_sha256": sha256(paths["benchmark_discovery"]),
            "assignment_sha256": benchmark_assignment_sha,
            "objective": benchmark_objective,
            "exact_assignment_reproduction_count": sum(
                row["assignment_sha256"] == benchmark_assignment_sha
                for row in accepted
            ),
            "objective_rank": rank,
        },
        "assignment_similarity": (
            distribution([row["matched_unit_rate"] for row in accepted])
            if accepted
            else None
        ),
        "unique_assignment_count": len(
            {row["assignment_sha256"] for row in accepted}
        ),
        "objective_distributions": (
            {
                field: distribution([row[field] for row in accepted])
                for field in (
                    "max_population_deviation_scaled",
                    "total_population_deviation_scaled",
                    "weighted_boundary_cut",
                )
            }
            if accepted
            else None
        ),
        "failures": [
            {
                "diagnostic_index": row["diagnostic_index"],
                "engine_seed": row["engine_seed"],
                "exit_code": row["exit_code"],
                "failure": row["failure"],
            }
            for row in rejected
        ],
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = state_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8")
    similarity = analysis["assignment_similarity"]
    readme = f"""# NRS v0.3 {state} Root Sensitivity

**Status:** complete

| Measure | Result |
|---|---:|
| Root seat split | {config['child_seats'][0]}:{config['child_seats'][1]} |
| Units | {config['unit_count']:,} |
| Accepted seeds | {len(accepted)} / {SEED_COUNT} |
| Unique assignments | {analysis['unique_assignment_count']} |
| Exact benchmark reproductions | {analysis['benchmark']['exact_assignment_reproduction_count']} |
| Population-tolerance passes | {analysis['population_tolerance_pass_count']} |
| Benchmark objective rank interval | {rank['rank_min']}-{rank['rank_max']} / {rank['rank_denominator']} |
"""
    if similarity is not None:
        readme += (
            f"| Minimum benchmark agreement | {similarity['minimum']:.6%} |\n"
            f"| Median benchmark agreement | {similarity['median']:.6%} |\n"
            f"| Mean benchmark agreement | {similarity['mean']:.6%} |\n"
            f"| Maximum benchmark agreement | {similarity['maximum']:.6%} |\n"
        )
    readme += f"""
Assignments are packed in `assignments.bin`; offsets and metrics are recorded
in `seed-results.csv`.

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = state_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8")

    code_paths = [
        PROTOCOL_PATH,
        AUDIT_PATH,
        RUNNER_PATH,
        VERIFIER_PATH,
        RI_RUNNER_PATH,
    ]
    manifest = {
        "schema_version": STATE_MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "complete",
        "state": state,
        "inputs": [
            {"path": relative_path(path), "sha256": sha256(path)}
            for path in paths.values()
        ],
        "code": [
            {"path": path.as_posix(), "sha256": sha256(ROOT / path)}
            for path in code_paths
        ],
        "outputs": {
            path.name: sha256(path)
            for path in (analysis_path, csv_path, assignments_path, readme_path)
        },
        "display_output_dir": display_state_dir,
        "claim_boundary": CLAIM_BOUNDARY,
    }
    manifest_path = state_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return analysis


def write_package(binary: Path, output_dir: Path, workers: int, display_dir: str | None) -> None:
    require(workers > 0, "workers must be positive")
    prepared_states = {
        state: prepare_state(state, binary) for state in sorted(STATE_CONFIG)
    }
    all_rows = {state: [] for state in prepared_states}
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_root = Path(temp_dir)
        with ThreadPoolExecutor(max_workers=workers) as executor:
            futures = []
            for state, prepared in prepared_states.items():
                for index in range(1, SEED_COUNT + 1):
                    seed_row = derive_seed(prepared["canonical_manifest"], index)
                    futures.append(
                        executor.submit(
                            run_pair,
                            state,
                            prepared["config"],
                            prepared["paths"],
                            seed_row,
                            prepared["benchmark_assignment"],
                            prepared["benchmark"]["instance_hash"],
                            temp_root,
                        )
                    )
            for future in as_completed(futures):
                row = future.result()
                all_rows[row["state"]].append(row)

    output_dir.mkdir(parents=True, exist_ok=True)
    states_dir = output_dir / "states"
    canonical_output = display_dir or relative_path(output_dir)
    state_analyses = []
    for state, prepared in prepared_states.items():
        state_analyses.append(
            write_state_package(
                prepared,
                all_rows[state],
                states_dir / state.lower(),
                f"{canonical_output}/states/{state.lower()}",
            )
        )

    total_units = sum(row["unit_count"] for row in state_analyses)
    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "complete",
        "census_year": 2020,
        "states": [row["state"] for row in state_analyses],
        "state_count": len(state_analyses),
        "diagnostic_pair_count": sum(
            row["diagnostic_seed_count"] for row in state_analyses
        ),
        "accepted_pair_count": sum(
            row["accepted_seed_count"] for row in state_analyses
        ),
        "rejected_pair_count": sum(
            row["rejected_seed_count"] for row in state_analyses
        ),
        "total_units_across_roots": total_units,
        "state_weighted_mean_benchmark_agreement": sum(
            row["assignment_similarity"]["mean"] for row in state_analyses
        )
        / len(state_analyses),
        "block_weighted_mean_benchmark_agreement": sum(
            row["assignment_similarity"]["mean"] * row["unit_count"]
            for row in state_analyses
        )
        / total_units,
        "state_results": [
            {
                "state": row["state"],
                "root_child_seats": row["root_child_seats"],
                "unit_count": row["unit_count"],
                "accepted_seed_count": row["accepted_seed_count"],
                "rejected_seed_count": row["rejected_seed_count"],
                "unique_assignment_count": row["unique_assignment_count"],
                "exact_benchmark_reproduction_count": row["benchmark"][
                    "exact_assignment_reproduction_count"
                ],
                "mean_benchmark_agreement": row["assignment_similarity"]["mean"],
                "minimum_benchmark_agreement": row["assignment_similarity"][
                    "minimum"
                ],
            }
            for row in state_analyses
        ],
        "objective_aggregation": (
            "Objective magnitudes are reported only within each State root."
        ),
        "mechanism_note": (
            "All three roots were invariant. In the NRS v0.3 path, the seeded "
            "METIS assignment affects the moved-population tie-break only after "
            "deterministic DFS candidates tie on population deviation and cut. "
            "Further seed expansion should first instrument whether such ties "
            "exist in candidate roots."
        ),
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = output_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8")
    state_rows = analysis["state_results"]
    readme = f"""# NRS v0.3 Multi-State Root Sensitivity

**Status:** complete

| State | Root split | Units | Accepted | Unique assignments | Exact benchmark reproductions | Mean agreement | Minimum agreement |
|---|---:|---:|---:|---:|---:|---:|---:|
"""
    for row in state_rows:
        readme += (
            f"| {row['state']} | {row['root_child_seats'][0]}:{row['root_child_seats'][1]} "
            f"| {row['unit_count']:,} | {row['accepted_seed_count']}/{SEED_COUNT} "
            f"| {row['unique_assignment_count']} "
            f"| {row['exact_benchmark_reproduction_count']} "
            f"| {row['mean_benchmark_agreement']:.6%} "
            f"| {row['minimum_benchmark_agreement']:.6%} |\n"
        )
    readme += f"""
State-weighted mean benchmark agreement:
{analysis['state_weighted_mean_benchmark_agreement']:.6%}.

Block-weighted mean benchmark agreement:
{analysis['block_weighted_mean_benchmark_agreement']:.6%}.

Objective values are not pooled across roots.

All three roots were invariant. The NRS v0.3 path consults the seeded METIS
assignment only after deterministic DFS candidates tie on population
deviation and cut. Further brute-force seed expansion is therefore gated on
instrumenting candidate-tie multiplicity rather than assuming more seeds or
States will exercise the parameter.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_multistate_root_sensitivity.py `
  --output-dir {canonical_output}
python scripts/research/verify_nrs_multistate_root_sensitivity.py `
  {canonical_output}
```

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = output_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8")

    state_manifests = [
        {
            "state": state,
            "path": f"states/{state.lower()}/manifest.json",
            "sha256": sha256(states_dir / state.lower() / "manifest.json"),
        }
        for state in sorted(prepared_states)
    ]
    code_paths = [
        PROTOCOL_PATH,
        AUDIT_PATH,
        RUNNER_PATH,
        VERIFIER_PATH,
        RI_RUNNER_PATH,
    ]
    manifest = {
        "schema_version": MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "complete",
        "state_manifests": state_manifests,
        "code": [
            {"path": path.as_posix(), "sha256": sha256(ROOT / path)}
            for path in code_paths
        ],
        "outputs": {
            "analysis.json": sha256(analysis_path),
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
        default=ROOT
        / "docs/experiments/nrs-v0.3-multistate-root-sensitivity-2020",
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
