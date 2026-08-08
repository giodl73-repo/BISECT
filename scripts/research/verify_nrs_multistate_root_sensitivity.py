#!/usr/bin/env python3
"""Verify and regenerate the NH/NM/GA NRS root sensitivity package."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ANALYSIS_VERSION = "nrs-v0.3-multistate-root-sensitivity-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-multistate-root-sensitivity-manifest-v1"
STATE_ANALYSIS_VERSION = "nrs-v0.3-root-sensitivity-analysis-v1"
STATE_MANIFEST_VERSION = "nrs-v0.3-root-sensitivity-manifest-v1"
DOMAIN = b"NRS_SENSITIVITY_V1\x00"
SEED_MODULUS = 2_147_483_647


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fail(message: str) -> None:
    raise SystemExit(f"verification failed: {message}")


def derive_seed(canonical_manifest: bytes, index: int) -> tuple[str, int, int]:
    digest = hashlib.sha256(
        DOMAIN + canonical_manifest + index.to_bytes(4, "big")
    ).digest()
    seed_u64 = int.from_bytes(digest[:8], "little")
    return digest.hex(), seed_u64, seed_u64 % SEED_MODULUS


def verify_state(root: Path, package: Path, state: str) -> dict:
    manifest = json.loads((package / "manifest.json").read_text(encoding="utf-8"))
    analysis = json.loads((package / "analysis.json").read_text(encoding="utf-8"))
    if manifest.get("schema_version") != STATE_MANIFEST_VERSION:
        fail(f"{state}: manifest schema")
    if analysis.get("schema_version") != STATE_ANALYSIS_VERSION:
        fail(f"{state}: analysis schema")
    for collection in ("inputs", "code"):
        for row in manifest[collection]:
            path = root / row["path"]
            if not path.is_file() or sha256(path) != row["sha256"]:
                fail(f"{state}: {collection[:-1]} hash for {row['path']}")
    for name, expected in manifest["outputs"].items():
        path = package / name
        if not path.is_file() or sha256(path) != expected:
            fail(f"{state}: output hash for {name}")

    input_manifest_row = next(
        row for row in manifest["inputs"] if row["path"].endswith("input_manifest.json")
    )
    input_payload = json.loads(
        (root / input_manifest_row["path"]).read_text(encoding="utf-8")
    )
    canonical_manifest = json.dumps(
        input_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")
    benchmark_row = next(
        row
        for row in manifest["inputs"]
        if row["path"].endswith("certified-discovery.json")
    )
    benchmark = json.loads(
        (root / benchmark_row["path"]).read_text(encoding="utf-8")
    )
    benchmark_assignment = benchmark["objective"]["canonical_assignment"]
    unit_count = analysis["unit_count"]
    packed_file = (package / "assignments.bin").read_bytes()
    with (package / "seed-results.csv").open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if len(rows) != 100:
        fail(f"{state}: seed row count")
    expected_offset = 0
    accepted = 0
    matched_rates = []
    benchmark_packed = bytearray((unit_count + 7) // 8)
    for index, label in enumerate(benchmark_assignment):
        if label:
            benchmark_packed[index // 8] |= 1 << (index % 8)
    benchmark_sha = hashlib.sha256(benchmark_packed).hexdigest()
    reproductions = 0
    tolerance_passes = 0
    for expected_index, row in enumerate(rows, start=1):
        if int(row["diagnostic_index"]) != expected_index:
            fail(f"{state}: diagnostic index")
        digest, seed_u64, engine_seed = derive_seed(
            canonical_manifest, expected_index
        )
        if (
            row["seed_digest_sha256"] != digest
            or int(row["seed_u64_little_endian"]) != seed_u64
            or int(row["engine_seed"]) != engine_seed
        ):
            fail(f"{state}: seed derivation {expected_index}")
        if row["status"] == "rejected":
            if not row["failure"]:
                fail(f"{state}: missing failure {expected_index}")
            continue
        if row["status"] != "accepted":
            fail(f"{state}: status {expected_index}")
        accepted += 1
        offset = int(row["assignment_offset"])
        byte_count = int(row["assignment_bytes"])
        if offset != expected_offset or byte_count != (unit_count + 7) // 8:
            fail(f"{state}: packed offset {expected_index}")
        packed = packed_file[offset : offset + byte_count]
        if hashlib.sha256(packed).hexdigest() != row["assignment_sha256"]:
            fail(f"{state}: assignment hash {expected_index}")
        assignment = [
            (packed[index // 8] >> (index % 8)) & 1
            for index in range(unit_count)
        ]
        direct = sum(
            left == right
            for left, right in zip(
                benchmark_assignment, assignment, strict=True
            )
        )
        matched = max(direct, unit_count - direct)
        rate = matched / unit_count
        if (
            int(row["matched_units"]) != matched
            or abs(float(row["matched_unit_rate"]) - rate) > 1e-15
        ):
            fail(f"{state}: overlap {expected_index}")
        matched_rates.append(rate)
        tolerance_pass = (
            int(row["max_population_deviation_scaled"])
            <= analysis["population_tolerance_scaled"]
        )
        if (row["population_tolerance_pass"] == "true") != tolerance_pass:
            fail(f"{state}: tolerance {expected_index}")
        tolerance_passes += tolerance_pass
        reproduction = row["assignment_sha256"] == benchmark_sha
        if (row["benchmark_assignment_reproduction"] == "true") != reproduction:
            fail(f"{state}: reproduction {expected_index}")
        reproductions += reproduction
        expected_offset += byte_count
    if expected_offset != len(packed_file):
        fail(f"{state}: packed file length")
    if accepted != analysis["accepted_seed_count"]:
        fail(f"{state}: accepted count")
    if tolerance_passes != analysis["population_tolerance_pass_count"]:
        fail(f"{state}: tolerance count")
    if reproductions != analysis["benchmark"]["exact_assignment_reproduction_count"]:
        fail(f"{state}: reproduction count")
    if matched_rates and abs(
        sum(matched_rates) / len(matched_rates)
        - analysis["assignment_similarity"]["mean"]
    ) > 1e-15:
        fail(f"{state}: mean agreement")
    return analysis


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    root = Path.cwd().resolve()
    package = args.package.resolve()
    manifest = json.loads((package / "manifest.json").read_text(encoding="utf-8"))
    analysis = json.loads((package / "analysis.json").read_text(encoding="utf-8"))
    if manifest.get("schema_version") != MANIFEST_VERSION:
        fail("manifest schema")
    if analysis.get("schema_version") != ANALYSIS_VERSION:
        fail("analysis schema")
    if manifest.get("status") != "complete" or analysis.get("status") != "complete":
        fail("package status")
    for row in manifest["code"]:
        path = root / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            fail(f"code hash for {row['path']}")
    for name, expected in manifest["outputs"].items():
        path = package / name
        if not path.is_file() or sha256(path) != expected:
            fail(f"output hash for {name}")

    state_analyses = []
    for row in manifest["state_manifests"]:
        state_manifest = package / row["path"]
        if not state_manifest.is_file() or sha256(state_manifest) != row["sha256"]:
            fail(f"State manifest hash for {row['state']}")
        state_analyses.append(
            verify_state(root, state_manifest.parent, row["state"])
        )
    if [row["state"] for row in state_analyses] != analysis["states"]:
        fail("State order")
    total_units = sum(row["unit_count"] for row in state_analyses)
    state_weighted = sum(
        row["assignment_similarity"]["mean"] for row in state_analyses
    ) / len(state_analyses)
    block_weighted = sum(
        row["assignment_similarity"]["mean"] * row["unit_count"]
        for row in state_analyses
    ) / total_units
    if abs(
        state_weighted - analysis["state_weighted_mean_benchmark_agreement"]
    ) > 1e-15:
        fail("State-weighted agreement")
    if abs(
        block_weighted - analysis["block_weighted_mean_benchmark_agreement"]
    ) > 1e-15:
        fail("block-weighted agreement")

    reproduction = manifest["reproduction"]
    runner = root / "scripts/research/run_nrs_multistate_root_sensitivity.py"
    with tempfile.TemporaryDirectory() as temp_dir:
        regenerated = Path(temp_dir) / "package"
        completed = subprocess.run(
            [
                sys.executable,
                str(runner),
                "--binary",
                str(root / reproduction["binary"]),
                "--output-dir",
                str(regenerated),
                "--workers",
                str(reproduction["workers"]),
                "--display-output-dir",
                reproduction["display_output_dir"],
            ],
            cwd=root,
            check=False,
        )
        if completed.returncode != 0:
            fail("regeneration command")
        expected_files = [
            "analysis.json",
            "README.md",
            "manifest.json",
            *[
                f"states/{state.lower()}/{name}"
                for state in analysis["states"]
                for name in (
                    "analysis.json",
                    "seed-results.csv",
                    "assignments.bin",
                    "README.md",
                    "manifest.json",
                )
            ],
        ]
        for name in expected_files:
            if (regenerated / name).read_bytes() != (package / name).read_bytes():
                fail(f"regenerated {name} differs")

    print("NRS v0.3 multi-State root sensitivity verification: PASS")


if __name__ == "__main__":
    main()
