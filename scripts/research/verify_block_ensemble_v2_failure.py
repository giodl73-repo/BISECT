#!/usr/bin/env python3
"""Verify the retained terminal failure that closed expansion v2."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from measure_block_ensemble_resources import sha256
from run_block_ensemble_expansion_v2 import (
    MEMORY_LIMIT_BYTES,
    PACKAGE,
    PROTOCOL_ID,
    RESOURCE_SCHEMA,
)
from verify_block_ensemble_expansion_v2 import verify_admission, verify_package


FROZEN_RUNNER_SOURCE_SHA256 = (
    "4837c63091f6e2b91ca89e6dc29e3798dc8e427f20ad771a7e6b9c9e023a6853"
)
FROZEN_WRAPPER_SOURCE_SHA256 = (
    "9c42ade271a3d97da51f5ef6310f2d8729edafc6e4a72c0441fd111abb5e2e99"
)
FROZEN_PROTOCOL_SHA256 = (
    "3af99b81dda14d12fe14562e0ec70c5564e273123638b103040ae293f1af3316"
)


def fail(message: str) -> None:
    raise ValueError(f"block ensemble expansion v2 failure verification failed: {message}")


def load(path: Path) -> dict:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        fail(f"{path.name} is not a JSON object")
    return value


def verify_terminal_failure(package: Path = PACKAGE) -> dict:
    package = package.resolve()
    ledger = verify_package(package)
    expected_failure = {
        "key": "NH:wilson",
        "phase": "preflight",
        "reason": "runner returned 1",
    }
    if ledger.get("status") != "failed" or ledger.get("failures") != [
        expected_failure
    ]:
        fail("terminal ledger identity drift")
    if any(ledger["completed"].values()):
        fail("terminal ledger contains a completion")
    if ledger.get("retained_bytes") != 0 or ledger.get("runner_wall_seconds") != 0.0:
        fail("terminal ledger resource totals drift")

    readiness = load(package / "readiness.json")
    executable_sha256 = readiness.get("sha256_bindings", {}).get("block_trace.exe")
    admission_path = package / "admission-preflight-nh-wilson-attempt-01.json"
    resource_path = package / "resource-preflight-nh-wilson.json"
    admission = load(admission_path)
    resource = load(resource_path)
    verify_admission(admission, package, package / "ledger.json", require_pass=True)

    expected_resource = {
        "schema_version": RESOURCE_SCHEMA,
        "status": "fail",
        "protocol_id": PROTOCOL_ID,
        "phase": "preflight",
        "state": "NH",
        "sampler": "wilson",
        "returncode": 1,
        "failure": None,
        "runner_executable_sha256": executable_sha256,
        "runner_source_sha256": FROZEN_RUNNER_SOURCE_SHA256,
        "wrapper_source_sha256": FROZEN_WRAPPER_SOURCE_SHA256,
        "protocol_sha256": FROZEN_PROTOCOL_SHA256,
        "admission_record": admission_path.name,
        "admission_record_sha256": sha256(admission_path),
    }
    for key, value in expected_resource.items():
        if resource.get(key) != value:
            fail(f"terminal resource {key} drift")
    if not 1 <= resource.get("poll_interval_ms", 0) <= 50:
        fail("terminal resource polling drift")
    if not 0 < resource.get("wall_seconds", 0):
        fail("terminal resource wall time is invalid")
    if not 0 < resource.get("peak_rss_bytes", 0) <= MEMORY_LIMIT_BYTES:
        fail("terminal resource peak RSS is invalid")
    if resource.get("sample_count", 0) <= 0:
        fail("terminal resource sample count is invalid")
    if any(package.glob("preflight-*.json")):
        fail("failed process unexpectedly retained a trace")
    return resource


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path, nargs="?", default=PACKAGE)
    args = parser.parse_args()
    try:
        resource = verify_terminal_failure(args.package)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
    print(
        "block ensemble expansion v2 terminal failure: PASS "
        f"(returncode={resource['returncode']}, wall={resource['wall_seconds']:.4f}s)"
    )


if __name__ == "__main__":
    main()
