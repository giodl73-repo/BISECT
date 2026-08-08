#!/usr/bin/env python3
"""Verify and regenerate the Rhode Island NRS v0.3 sensitivity package."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ANALYSIS_VERSION = "nrs-v0.3-ri-sensitivity-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-ri-sensitivity-manifest-v1"
DOMAIN = b"NRS_SENSITIVITY_V1\x00"
SEED_MODULUS = 2_147_483_647
UNIT_COUNT = 25_649


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fail(message: str) -> None:
    raise SystemExit(f"verification failed: {message}")


def derive_seed(canonical_manifest: bytes, diagnostic_index: int) -> tuple[str, int, int]:
    digest = hashlib.sha256(
        DOMAIN + canonical_manifest + diagnostic_index.to_bytes(4, "big")
    ).digest()
    seed_u64 = int.from_bytes(digest[:8], "little")
    return digest.hex(), seed_u64, seed_u64 % SEED_MODULUS


def unpack_assignment(packed: bytes) -> list[int]:
    return [
        (packed[index // 8] >> (index % 8)) & 1 for index in range(UNIT_COUNT)
    ]


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

    for collection in ("inputs", "code"):
        for row in manifest[collection]:
            path = root / row["path"]
            if not path.is_file() or sha256(path) != row["sha256"]:
                fail(f"{collection[:-1]} hash mismatch for {row['path']}")
    for name, expected in manifest["outputs"].items():
        path = package / name
        if not path.is_file() or sha256(path) != expected:
            fail(f"output hash mismatch for {name}")

    reproduction = manifest["reproduction"]
    input_manifest = root / reproduction["input_manifest"]
    payload = json.loads(input_manifest.read_text(encoding="utf-8"))
    canonical_manifest = json.dumps(
        payload, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")
    benchmark = json.loads(
        (root / reproduction["benchmark_discovery"]).read_text(encoding="utf-8")
    )
    benchmark_assignment = benchmark["objective"]["canonical_assignment"]
    packed_file = (package / "assignments.bin").read_bytes()
    with (package / "seed-results.csv").open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if len(rows) != 100 or [int(row["diagnostic_index"]) for row in rows] != list(
        range(1, 101)
    ):
        fail("diagnostic index universe")

    accepted_count = 0
    tolerance_pass_count = 0
    reproduction_count = 0
    benchmark_packed = bytearray((UNIT_COUNT + 7) // 8)
    for index, label in enumerate(benchmark_assignment):
        if label:
            benchmark_packed[index // 8] |= 1 << (index % 8)
    benchmark_assignment_sha = hashlib.sha256(benchmark_packed).hexdigest()
    expected_offset = 0

    for row in rows:
        diagnostic_index = int(row["diagnostic_index"])
        digest, seed_u64, engine_seed = derive_seed(
            canonical_manifest, diagnostic_index
        )
        if (
            row["seed_digest_sha256"] != digest
            or int(row["seed_u64_little_endian"]) != seed_u64
            or int(row["engine_seed"]) != engine_seed
        ):
            fail(f"seed derivation for index {diagnostic_index}")
        if row["status"] == "rejected":
            if not row["failure"]:
                fail(f"missing failure for index {diagnostic_index}")
            continue
        if row["status"] != "accepted":
            fail(f"unknown status for index {diagnostic_index}")
        accepted_count += 1
        offset = int(row["assignment_offset"])
        byte_count = int(row["assignment_bytes"])
        if offset != expected_offset:
            fail(f"assignment offset for index {diagnostic_index}")
        packed = packed_file[offset : offset + byte_count]
        if byte_count != (UNIT_COUNT + 7) // 8:
            fail(f"packed length for index {diagnostic_index}")
        if hashlib.sha256(packed).hexdigest() != row["assignment_sha256"]:
            fail(f"assignment hash for index {diagnostic_index}")
        assignment = unpack_assignment(packed)
        direct = sum(
            left == right
            for left, right in zip(
                benchmark_assignment, assignment, strict=True
            )
        )
        matched = max(direct, UNIT_COUNT - direct)
        if (
            int(row["matched_blocks"]) != matched
            or abs(float(row["matched_block_rate"]) - matched / UNIT_COUNT) > 1e-15
        ):
            fail(f"assignment overlap for index {diagnostic_index}")
        tolerance_pass = (
            int(row["max_population_deviation_scaled"])
            <= analysis["population_tolerance_scaled"]
        )
        if (row["population_tolerance_pass"] == "true") != tolerance_pass:
            fail(f"population tolerance for index {diagnostic_index}")
        tolerance_pass_count += tolerance_pass
        is_reproduction = row["assignment_sha256"] == benchmark_assignment_sha
        if (row["benchmark_assignment_reproduction"] == "true") != is_reproduction:
            fail(f"benchmark reproduction for index {diagnostic_index}")
        reproduction_count += is_reproduction
        expected_offset += byte_count

    if expected_offset != len(packed_file):
        fail("packed assignment file length")

    if accepted_count != analysis["accepted_seed_count"]:
        fail("accepted count")
    if 100 - accepted_count != analysis["rejected_seed_count"]:
        fail("rejected count")
    if tolerance_pass_count != analysis["population_tolerance_pass_count"]:
        fail("population tolerance count")
    if (
        reproduction_count
        != analysis["benchmark"]["exact_assignment_reproduction_count"]
    ):
        fail("benchmark reproduction count")

    runner = root / "scripts/research/run_nrs_ri_sensitivity.py"
    with tempfile.TemporaryDirectory() as temp_dir:
        regenerated = Path(temp_dir) / "package"
        command = [
            sys.executable,
            str(runner),
            "--binary",
            str(root / reproduction["binary"]),
            "--context",
            str(root / reproduction["context"]),
            "--input-manifest",
            str(input_manifest),
            "--benchmark-discovery",
            str(root / reproduction["benchmark_discovery"]),
            "--benchmark-tree",
            str(root / reproduction["benchmark_tree"]),
            "--output-dir",
            str(regenerated),
            "--workers",
            str(reproduction["workers"]),
            "--display-output-dir",
            reproduction["display_output_dir"],
        ]
        completed = subprocess.run(command, cwd=root, check=False)
        if completed.returncode != 0:
            fail("regeneration command")
        for name in (
            "analysis.json",
            "seed-results.csv",
            "assignments.bin",
            "README.md",
            "manifest.json",
        ):
            if (regenerated / name).read_bytes() != (package / name).read_bytes():
                fail(f"regenerated {name} differs")

    print("NRS v0.3 Rhode Island sensitivity verification: PASS")


if __name__ == "__main__":
    main()
