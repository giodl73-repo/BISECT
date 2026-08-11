#!/usr/bin/env python3
"""Verify v3 custody without accepting any closed predecessor completion."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from measure_block_ensemble_resources import sha256
from run_block_ensemble_expansion_v3 import (
    BASE_SEED,
    MEMORY_LIMIT_BYTES,
    ORDER,
    PACKAGE,
    PROTOCOL,
    PROTOCOL_ID,
    RESOURCE_SCHEMA,
    RETAINED_LIMIT_BYTES,
    RUNNER,
    SCRATCH_LIMIT_BYTES,
    WALL_LIMIT_SECONDS,
    WRAPPER,
    artifact_paths,
    load_trace,
    validate_ledger,
    validate_trace,
)

ADMISSION_SCHEMA = "nrs-block-ensemble-host-capacity-v1"
CLOSED_PROTOCOL_IDS = {
    "nrs-v0.3-block-ensemble-expansion-v1",
    "nrs-v0.3-block-ensemble-expansion-v2",
}


def fail(message: str) -> None:
    raise ValueError(f"block ensemble expansion v3 verification failed: {message}")


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def verify_admission(
    record: dict, package: Path, ledger_path: Path, require_pass: bool
) -> None:
    expected = {
        "schema_version": ADMISSION_SCHEMA,
        "scratch_limit_bytes": SCRATCH_LIMIT_BYTES,
        "retained_limit_bytes": RETAINED_LIMIT_BYTES,
        "safety_reserve_bytes": 2 * 1024**3,
        "package_path": str(package.resolve()),
        "ledger_path": str(ledger_path.resolve()),
    }
    for key, value in expected.items():
        if record.get(key) != value:
            fail(f"admission {key} drift")
    retained_used = record.get("retained_used_bytes")
    free_bytes = record.get("free_bytes")
    if not isinstance(retained_used, int) or not isinstance(free_bytes, int):
        fail("admission byte values are invalid")
    required = SCRATCH_LIMIT_BYTES + (RETAINED_LIMIT_BYTES - retained_used) + 2 * 1024**3
    shortfall = max(0, required - free_bytes)
    if record.get("required_free_bytes") != required:
        fail("admission required-byte formula drift")
    if record.get("shortfall_bytes") != shortfall:
        fail("admission shortfall formula drift")
    passed = shortfall == 0
    if record.get("status") != ("pass" if passed else "reject"):
        fail("admission status drift")
    if record.get("process_launch_authorized") is not passed:
        fail("admission launch authorization drift")
    if require_pass and not passed:
        fail("completed process used a rejected admission")


def verify_resource_record(
    record: dict,
    state: str,
    sampler: str,
    phase: str,
    admission_path: Path,
    executable_sha256: str,
) -> None:
    expected = {
        "schema_version": RESOURCE_SCHEMA,
        "status": "pass",
        "protocol_id": PROTOCOL_ID,
        "phase": phase,
        "state": state,
        "sampler": sampler,
        "returncode": 0,
        "failure": None,
        "runner_source_sha256": sha256(RUNNER),
        "wrapper_source_sha256": sha256(WRAPPER),
        "protocol_sha256": sha256(PROTOCOL),
        "runner_executable_sha256": executable_sha256,
        "admission_record": admission_path.name,
        "admission_record_sha256": sha256(admission_path),
    }
    for key, value in expected.items():
        if record.get(key) != value:
            fail(f"{phase} {state}:{sampler} {key} drift")
    if not 1 <= record.get("poll_interval_ms", 0) <= 50:
        fail(f"{phase} {state}:{sampler} polling drift")
    if not 0 < record.get("peak_rss_bytes", 0) <= MEMORY_LIMIT_BYTES:
        fail(f"{phase} {state}:{sampler} peak RSS invalid")


def reject_closed_protocol_claims(package: Path) -> None:
    for path in package.rglob("*.json"):
        value = load(path)
        if value.get("protocol_id") in CLOSED_PROTOCOL_IDS:
            fail(f"closed predecessor artifact present: {path.name}")


def verify_package(package: Path) -> dict:
    package = package.resolve()
    if not package.is_dir():
        fail(f"package does not exist: {package}")
    ledger_path = package / "ledger.json"
    ledger = load(ledger_path)
    try:
        validate_ledger(ledger)
    except ValueError as error:
        fail(str(error))
    reject_closed_protocol_claims(package)
    readiness_path = package / "readiness.json"
    executable_sha256 = None
    if readiness_path.is_file():
        executable_sha256 = load(readiness_path).get("sha256_bindings", {}).get(
            "block_trace.exe"
        )
    for admission_path in package.glob("admission-*.json"):
        verify_admission(load(admission_path), package, ledger_path, require_pass=False)

    retained_total = 0
    governed_wall_total = 0.0
    for phase in ("preflight", "preflight-replay", "primary", "replay"):
        for key in ledger["completed"][phase]:
            state, sampler = key.split(":")
            paths = artifact_paths(package, state, sampler, phase)
            record = load(paths["resource"])
            admission_name = record.get("admission_record")
            if not isinstance(admission_name, str) or Path(admission_name).name != admission_name:
                fail(f"{phase} {key} admission filename is invalid")
            admission_path = package / admission_name
            admission = load(admission_path)
            verify_admission(admission, package, ledger_path, require_pass=True)
            if not isinstance(executable_sha256, str):
                fail("completed process has no executable readiness binding")
            verify_resource_record(
                record,
                state,
                sampler,
                phase,
                admission_path,
                executable_sha256,
            )
            if phase == "preflight":
                trace = load_trace(paths["runner_trace"])
                validate_trace(trace, state, sampler, phase)
                if record.get("retained_trace_sha256") != sha256(paths["runner_trace"]):
                    fail(f"preflight {key} retained hash mismatch")
                retained_total += paths["runner_trace"].stat().st_size
            elif phase == "primary":
                trace = load_trace(paths["final_trace"])
                validate_trace(trace, state, sampler, phase)
                compressed = paths["final_trace"].read_bytes()
                raw = gzip.decompress(compressed)
                if record.get("raw_trace_sha256") != hashlib.sha256(raw).hexdigest():
                    fail(f"primary {key} raw hash mismatch")
                if record.get("retained_trace_sha256") != hashlib.sha256(
                    compressed
                ).hexdigest():
                    fail(f"primary {key} retained hash mismatch")
                if record.get("trace_disposition") != (
                    "raw deleted after deterministic gzip custody"
                ):
                    fail(f"primary {key} custody disposition drift")
                retained_total += len(compressed)
                governed_wall_total += record["wall_seconds"]
            else:
                if paths["runner_trace"].exists():
                    fail(f"{phase} {key} scratch trace was retained")
                if record.get("normalized_trace_match") is not True:
                    fail(f"{phase} {key} normalized replay mismatch")
                if record.get("trace_disposition") != (
                    "deleted after exact normalized comparison"
                ):
                    fail(f"{phase} {key} scratch disposition drift")
                if phase == "replay":
                    governed_wall_total += record["wall_seconds"]
    if ledger["retained_bytes"] != retained_total:
        fail("v3 retained-byte ledger mismatch")
    if abs(ledger["runner_wall_seconds"] - governed_wall_total) > 1e-6:
        fail("v3 runner-wall ledger mismatch")
    if ledger["retained_bytes"] > RETAINED_LIMIT_BYTES:
        fail("v3 retained-byte ceiling exceeded")
    if ledger["runner_wall_seconds"] > WALL_LIMIT_SECONDS:
        fail("v3 runner-wall ceiling exceeded")
    return ledger


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path, nargs="?", default=PACKAGE)
    args = parser.parse_args()
    try:
        ledger = verify_package(args.package)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
    print(
        "block ensemble expansion v3 verification: PASS "
        f"(status={ledger['status']}, "
        f"preflights={len(ledger['completed']['preflight'])}/6, "
        f"primaries={len(ledger['completed']['primary'])}/6, "
        f"replays={len(ledger['completed']['replay'])}/6, seed={BASE_SEED})"
    )


if __name__ == "__main__":
    main()
