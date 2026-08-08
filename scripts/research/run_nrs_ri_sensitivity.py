#!/usr/bin/env python3
"""Run the precommitted Rhode Island NRS v0.3 sensitivity slice."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import statistics
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PROTOCOL_ID = "nrs-v0.3-ri-sensitivity-v1"
SCHEMA_VERSION = "nrs-v0.3-ri-sensitivity-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-ri-sensitivity-manifest-v1"
DOMAIN = b"NRS_SENSITIVITY_V1\x00"
SEED_MODULUS = 2_147_483_647
SEED_COUNT = 100
UNIT_COUNT = 25_649
POPULATION_TOLERANCE_SCALED = 5_487
GENERATED_AT = "2026-08-07T00:00:00Z"
PROTOCOL_PATH = Path("docs/specs/2026-08-07-nrs-v0.3-ri-sensitivity-protocol.md")
RUNNER_PATH = Path("scripts/research/run_nrs_ri_sensitivity.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_ri_sensitivity.py")
CLAIM_BOUNDARY = (
    "Rhode Island 2020 root sensitivity diagnostic only; no national robustness, "
    "ensemble convergence, random-sampling, partisan, compactness, demographic, "
    "VRA, legal-validity, optimality, or benchmark-replacement claim."
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
    "matched_blocks",
    "matched_block_rate",
    "max_population_deviation_scaled",
    "total_population_deviation_scaled",
    "weighted_boundary_cut",
    "population_tolerance_pass",
    "benchmark_assignment_reproduction",
]


class SensitivityError(ValueError):
    """Structured sensitivity input or output failure."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SensitivityError(message)


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


def canonical_json_bytes(path: Path) -> bytes:
    payload = json.loads(path.read_text(encoding="utf-8"))
    return json.dumps(
        payload, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def derive_seed(canonical_manifest: bytes, diagnostic_index: int) -> dict:
    require(1 <= diagnostic_index <= SEED_COUNT, "diagnostic index outside 1..100")
    digest = hashlib.sha256(
        DOMAIN + canonical_manifest + diagnostic_index.to_bytes(4, "big")
    ).digest()
    seed_u64 = int.from_bytes(digest[:8], "little")
    return {
        "diagnostic_index": diagnostic_index,
        "seed_digest_sha256": digest.hex(),
        "seed_u64_little_endian": seed_u64,
        "engine_seed": seed_u64 % SEED_MODULUS,
    }


def pack_assignment(assignment: list[int]) -> bytes:
    require(all(label in (0, 1) for label in assignment), "assignment is not binary")
    packed = bytearray((len(assignment) + 7) // 8)
    for index, label in enumerate(assignment):
        if label:
            packed[index // 8] |= 1 << (index % 8)
    return bytes(packed)


def unpack_assignment(packed: bytes, unit_count: int) -> list[int]:
    require(len(packed) == (unit_count + 7) // 8, "packed assignment length")
    return [
        (packed[index // 8] >> (index % 8)) & 1 for index in range(unit_count)
    ]


def assignment_overlap(
    benchmark: list[int], candidate: list[int]
) -> tuple[int, float]:
    require(len(benchmark) == len(candidate), "assignment lengths differ")
    direct = sum(left == right for left, right in zip(benchmark, candidate, strict=True))
    matched = max(direct, len(benchmark) - direct)
    return matched, matched / len(benchmark)


def sanitize_failure(text: str, temp_root: Path) -> str:
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    if not lines:
        return "command failed without diagnostic output"
    return lines[-1].replace(str(temp_root), "<TEMP>").replace(
        temp_root.as_posix(), "<TEMP>"
    )


def run_seed(
    binary: Path,
    context: Path,
    temp_root: Path,
    seed_row: dict,
    benchmark_assignment: list[int],
    benchmark_instance_hash: str,
) -> dict:
    diagnostic_index = seed_row["diagnostic_index"]
    seed_dir = temp_root / f"seed-{diagnostic_index:03d}"
    completed = subprocess.run(
        [
            str(binary),
            "exact",
            "--context",
            str(context),
            "--districts",
            "2",
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
    row = {**seed_row, "exit_code": completed.returncode}
    if completed.returncode != 0:
        return {
            **row,
            "status": "rejected",
            "failure": sanitize_failure(
                completed.stdout + completed.stderr, temp_root
            ),
        }

    discovery_path = seed_dir / "certified-discovery.json"
    require(discovery_path.is_file(), f"seed {diagnostic_index}: missing discovery")
    discovery = json.loads(discovery_path.read_text(encoding="utf-8"))
    require(
        discovery.get("schema_version") == "certified-split-discovery-v1",
        f"seed {diagnostic_index}: discovery schema",
    )
    require(
        discovery.get("instance_hash") == benchmark_instance_hash,
        f"seed {diagnostic_index}: instance hash",
    )
    assignment = discovery["objective"]["canonical_assignment"]
    require(
        len(assignment) == UNIT_COUNT,
        f"seed {diagnostic_index}: assignment unit count",
    )
    require(
        set(assignment) == {0, 1},
        f"seed {diagnostic_index}: assignment district labels",
    )
    packed = pack_assignment(assignment)
    matched_blocks, matched_rate = assignment_overlap(
        benchmark_assignment, assignment
    )
    objective = discovery["objective"]["primary"]
    return {
        **row,
        "status": "accepted",
        "failure": "",
        "discovery_sha256": sha256(discovery_path),
        "assignment_sha256": hashlib.sha256(packed).hexdigest(),
        "matched_blocks": matched_blocks,
        "matched_block_rate": matched_rate,
        "max_population_deviation_scaled": int(
            objective["max_population_deviation_scaled"]
        ),
        "total_population_deviation_scaled": int(
            objective["total_population_deviation_scaled"]
        ),
        "weighted_boundary_cut": int(objective["weighted_boundary_cut"]),
        "population_tolerance_pass": (
            int(objective["max_population_deviation_scaled"])
            <= POPULATION_TOLERANCE_SCALED
        ),
        "_assignment": assignment,
        "_packed": packed,
    }


def distribution(values: list[int | float]) -> dict:
    require(bool(values), "distribution is empty")
    return {
        "minimum": min(values),
        "median": statistics.median(values),
        "mean": statistics.fmean(values),
        "maximum": max(values),
    }


def objective_key(objective: dict, assignment: list[int]) -> tuple:
    return (
        int(objective["max_population_deviation_scaled"]),
        int(objective["total_population_deviation_scaled"]),
        int(objective["weighted_boundary_cut"]),
        tuple(assignment),
    )


def benchmark_rank(keys: list[tuple], benchmark_key: tuple) -> dict:
    better = sum(key < benchmark_key for key in keys)
    tied = sum(key == benchmark_key for key in keys)
    return {
        "diagnostic_seeds_better": better,
        "diagnostic_seeds_tied": tied,
        "rank_min": better + 1,
        "rank_max": better + tied + 1,
        "rank_denominator": len(keys) + 1,
    }


def write_package(
    binary: Path,
    context: Path,
    input_manifest: Path,
    benchmark_discovery: Path,
    benchmark_tree: Path,
    output_dir: Path,
    workers: int,
    display_output_dir: str | None = None,
) -> None:
    for path in (
        binary,
        context,
        input_manifest,
        benchmark_discovery,
        benchmark_tree,
    ):
        require(path.is_file(), f"missing input {path}")
    require(workers > 0, "workers must be positive")

    benchmark = json.loads(benchmark_discovery.read_text(encoding="utf-8"))
    benchmark_assignment = benchmark["objective"]["canonical_assignment"]
    require(len(benchmark_assignment) == UNIT_COUNT, "benchmark unit count")
    benchmark_objective = benchmark["objective"]["primary"]
    benchmark_instance_hash = benchmark["instance_hash"]
    tree = json.loads(benchmark_tree.read_text(encoding="utf-8"))
    require(tree["population_total"] == 1_097_379, "benchmark population")
    canonical_manifest = canonical_json_bytes(input_manifest)
    seed_rows = [derive_seed(canonical_manifest, index) for index in range(1, 101)]

    with tempfile.TemporaryDirectory() as temp_dir:
        temp_root = Path(temp_dir)
        rows = []
        with ThreadPoolExecutor(max_workers=workers) as executor:
            futures = [
                executor.submit(
                    run_seed,
                    binary,
                    context,
                    temp_root,
                    seed_row,
                    benchmark_assignment,
                    benchmark_instance_hash,
                )
                for seed_row in seed_rows
            ]
            for future in as_completed(futures):
                rows.append(future.result())
    rows.sort(key=lambda row: row["diagnostic_index"])

    accepted = [row for row in rows if row["status"] == "accepted"]
    rejected = [row for row in rows if row["status"] == "rejected"]
    benchmark_packed = pack_assignment(benchmark_assignment)
    benchmark_assignment_sha256 = hashlib.sha256(benchmark_packed).hexdigest()
    packed_output = bytearray()
    for row in accepted:
        row["assignment_offset"] = len(packed_output)
        row["assignment_bytes"] = len(row["_packed"])
        row["benchmark_assignment_reproduction"] = (
            row["assignment_sha256"] == benchmark_assignment_sha256
        )
        packed_output.extend(row["_packed"])

    output_dir.mkdir(parents=True, exist_ok=True)
    assignments_path = output_dir / "assignments.bin"
    assignments_path.write_bytes(bytes(packed_output))

    csv_path = output_dir / "seed-results.csv"
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
    benchmark_key = objective_key(benchmark_objective, benchmark_assignment)
    rank = benchmark_rank(accepted_keys, benchmark_key)
    duplicate_engine_seed_count = len(rows) - len(
        {row["engine_seed"] for row in rows}
    )

    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "complete",
        "state": "RI",
        "census_year": 2020,
        "diagnostic_seed_count": SEED_COUNT,
        "accepted_seed_count": len(accepted),
        "rejected_seed_count": len(rejected),
        "duplicate_engine_seed_count": duplicate_engine_seed_count,
        "unit_count": UNIT_COUNT,
        "population_tolerance_scaled": POPULATION_TOLERANCE_SCALED,
        "population_tolerance_pass_count": sum(
            row["population_tolerance_pass"] for row in accepted
        ),
        "benchmark": {
            "engine_seed": 1_983_447_153,
            "discovery_sha256": sha256(benchmark_discovery),
            "assignment_sha256": benchmark_assignment_sha256,
            "objective": benchmark_objective,
            "exact_assignment_reproduction_count": sum(
                row["assignment_sha256"] == benchmark_assignment_sha256
                for row in accepted
            ),
            "objective_rank": rank,
        },
        "assignment_similarity": (
            distribution([row["matched_block_rate"] for row in accepted])
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
        "unavailable_metrics": {
            "partisan": "No frozen election and precinct-to-block package is available.",
            "compactness": "This sensitivity slice evaluates assignments and the engine objective only.",
            "national_robustness": "Only the Rhode Island two-district root was run.",
            "ensemble_inference": "The diagnostic seeds are not a converged random sample.",
        },
        "mechanism_note": (
            "For the NRS v0.3 path, the seeded METIS assignment informs the "
            "moved-population tie-break among equal-deviation deterministic DFS "
            "cut candidates. Complete invariance at this root is therefore a "
            "mechanism-specific finding, not proof of general seed insensitivity."
        ),
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = output_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8")

    canonical_output_dir = display_output_dir or relative_path(output_dir)
    similarity = analysis["assignment_similarity"]
    readme = f"""# NRS v0.3 Rhode Island 100-Seed Sensitivity

**Status:** complete

| Measure | Result |
|---|---:|
| Diagnostic indices | {SEED_COUNT} |
| Accepted | {len(accepted)} |
| Rejected | {len(rejected)} |
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
The governed benchmark remains authoritative. Diagnostic assignments are
packed in `assignments.bin`; offsets and metrics are recorded in
`seed-results.csv`.

The NRS v0.3 path uses the seeded METIS assignment only in a moved-population
tie-break among equal-deviation deterministic DFS cut candidates. The complete
invariance observed here is specific to this Rhode Island root and mechanism;
it is not a national robustness claim.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_ri_sensitivity.py `
  --output-dir {canonical_output_dir}
python scripts/research/verify_nrs_ri_sensitivity.py `
  {canonical_output_dir}
```

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = output_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8")

    input_paths = [
        binary,
        context,
        input_manifest,
        benchmark_discovery,
        benchmark_tree,
    ]
    code_paths = [PROTOCOL_PATH, RUNNER_PATH, VERIFIER_PATH]
    manifest = {
        "schema_version": MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "complete",
        "inputs": [
            {"path": relative_path(path), "sha256": sha256(path)}
            for path in input_paths
        ],
        "code": [
            {"path": path.as_posix(), "sha256": sha256(ROOT / path)}
            for path in code_paths
        ],
        "outputs": {
            path.name: sha256(path)
            for path in (analysis_path, csv_path, assignments_path, readme_path)
        },
        "reproduction": {
            "binary": relative_path(binary),
            "context": relative_path(context),
            "input_manifest": relative_path(input_manifest),
            "benchmark_discovery": relative_path(benchmark_discovery),
            "benchmark_tree": relative_path(benchmark_tree),
            "workers": workers,
            "display_output_dir": canonical_output_dir,
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
        "--context",
        type=Path,
        default=ROOT / "data/2020/certified/ri_blocks_2020.rctx",
    )
    parser.add_argument(
        "--input-manifest",
        type=Path,
        default=ROOT
        / "runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/seed/input_manifest.json",
    )
    parser.add_argument(
        "--benchmark-discovery",
        type=Path,
        default=ROOT
        / "runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/nodes/root/certified-discovery.json",
    )
    parser.add_argument(
        "--benchmark-tree",
        type=Path,
        default=ROOT
        / "runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/baseline-tree.json",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=ROOT / "docs/experiments/nrs-v0.3-ri-sensitivity-2020",
    )
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--display-output-dir")
    args = parser.parse_args()
    write_package(
        binary=args.binary.resolve(),
        context=args.context.resolve(),
        input_manifest=args.input_manifest.resolve(),
        benchmark_discovery=args.benchmark_discovery.resolve(),
        benchmark_tree=args.benchmark_tree.resolve(),
        output_dir=args.output_dir.resolve(),
        workers=args.workers,
        display_output_dir=args.display_output_dir,
    )


if __name__ == "__main__":
    main()
