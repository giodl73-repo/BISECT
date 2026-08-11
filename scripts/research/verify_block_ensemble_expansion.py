#!/usr/bin/env python3
"""Verify Stage 0 evidence for the frozen NH/NM/GA block expansion."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from measure_block_ensemble_resources import sha256
from run_block_ensemble_expansion import ORDER, PROTOCOL, RUNNER, load_trace, validate_trace

RESOURCE_PACKAGE = ROOT / "docs/experiments/nrs-v0.3-block-ensemble-resource-audit"
RESOURCE_VERIFIER = ROOT / "scripts/research/verify_block_ensemble_resources.py"


def fail(message: str) -> None:
    raise SystemExit(f"block ensemble expansion verification failed: {message}")


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def verify_resource_record(record: dict, state: str, sampler: str, phase: str) -> None:
    expected = {
        "schema_version": "nrs-block-ensemble-expansion-resource-v1",
        "status": "pass",
        "protocol_id": "nrs-v0.3-block-ensemble-expansion-v1",
        "phase": phase,
        "state": state,
        "sampler": sampler,
        "returncode": 0,
        "failure": None,
    }
    for key, value in expected.items():
        if record.get(key) != value:
            fail(f"{phase} {state}:{sampler} {key} drift")
    if not 1 <= record.get("poll_interval_ms", 0) <= 50:
        fail(f"{phase} {state}:{sampler} polling drift")
    if record.get("peak_rss_bytes", 0) <= 0 or record["peak_rss_bytes"] > 2415919104:
        fail(f"{phase} {state}:{sampler} peak RSS invalid")
    if record.get("runner_source_sha256") != sha256(RUNNER):
        fail(f"{phase} {state}:{sampler} runner source mismatch")
    if record.get("protocol_sha256") != sha256(PROTOCOL):
        fail(f"{phase} {state}:{sampler} protocol mismatch")


def require_prefix(values: list[str], schedule: list[str], phase: str) -> None:
    if values != schedule[: len(values)]:
        fail(f"{phase} schedule is not a frozen-order prefix")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    package = args.package.resolve()
    completed = subprocess.run([sys.executable, str(RESOURCE_VERIFIER), str(RESOURCE_PACKAGE)], cwd=ROOT)
    if completed.returncode != 0:
        fail("prerequisite resource package did not verify")

    failure_dir = package / "preflight-determinism-failure"
    failed_ledger = load(failure_dir / "ledger.json")
    if failed_ledger.get("status") != "failed":
        fail("retained determinism ledger is not failed")
    expected_failure = {
        "key": "NM:wilson",
        "phase": "preflight-replay",
        "reason": "normalized replay mismatch",
    }
    if expected_failure not in failed_ledger.get("failures", []):
        fail("retained NM determinism failure is missing")
    if not (failure_dir / "preflight-replay-nm-wilson-mismatch.json").is_file():
        fail("retained NM mismatch trace is missing")
    if not (package / "preflight-pre-compression/ledger.json").is_file():
        fail("pre-compression engineering record is missing")

    ledger = load(package / "ledger.json")
    if ledger.get("status") not in {"active", "complete"} or ledger.get("failures") != []:
        fail("canonical ledger is not clean")
    if ledger["completed"].get("preflight") != ORDER:
        fail("canonical preflight schedule incomplete")
    if ledger["completed"].get("preflight-replay") != ORDER:
        fail("canonical preflight replay schedule incomplete")
    primaries = ledger["completed"].get("primary", [])
    replays = ledger["completed"].get("replay", [])
    require_prefix(primaries, ORDER, "primary")
    require_prefix(replays, ORDER, "replay")
    if replays and primaries != ORDER:
        fail("governed replay began before all primaries completed")
    if ledger.get("status") == "complete" and replays != ORDER:
        fail("ledger completed before the replay schedule")
    if ledger.get("status") == "active" and replays == ORDER:
        fail("ledger remained active after the replay schedule")
    retained_total = 0
    governed_wall_total = 0.0
    for state in ("NH", "NM", "GA"):
        for sampler in ("wilson", "kruskal"):
            tag = f"{state.lower()}-{sampler}"
            trace_path = package / f"preflight-{tag}.json"
            trace = load_trace(trace_path)
            validate_trace(trace, state, sampler, "preflight")
            record = load(package / f"resource-preflight-{tag}.json")
            verify_resource_record(record, state, sampler, "preflight")
            if record.get("retained_trace_sha256") != sha256(trace_path):
                fail(f"preflight {state}:{sampler} trace hash mismatch")
            if record.get("retained_trace_size_bytes") != trace_path.stat().st_size:
                fail(f"preflight {state}:{sampler} trace size mismatch")
            retained_total += trace_path.stat().st_size
            replay = load(package / f"resource-preflight-replay-{tag}.json")
            verify_resource_record(replay, state, sampler, "preflight-replay")
            if replay.get("normalized_trace_match") is not True:
                fail(f"preflight replay {state}:{sampler} did not match")
            if replay.get("trace_disposition") != "deleted after exact normalized comparison":
                fail(f"preflight replay {state}:{sampler} scratch disposition drift")
    for key in primaries:
        state, sampler = key.split(":")
        tag = f"{state.lower()}-{sampler}"
        trace_path = package / f"governed-{tag}.json.gz"
        trace = load_trace(trace_path)
        validate_trace(trace, state, sampler, "primary")
        record = load(package / f"resource-primary-{tag}.json")
        verify_resource_record(record, state, sampler, "primary")
        compressed = trace_path.read_bytes()
        raw = gzip.decompress(compressed)
        if record.get("raw_trace_sha256") != hashlib.sha256(raw).hexdigest():
            fail(f"primary {key} raw trace hash mismatch")
        if record.get("raw_trace_size_bytes") != len(raw):
            fail(f"primary {key} raw trace size mismatch")
        if record.get("retained_trace_sha256") != hashlib.sha256(compressed).hexdigest():
            fail(f"primary {key} retained trace hash mismatch")
        if record.get("retained_trace_size_bytes") != len(compressed):
            fail(f"primary {key} retained trace size mismatch")
        if record.get("trace_disposition") != "raw deleted after deterministic gzip custody":
            fail(f"primary {key} custody disposition drift")
        retained_total += len(compressed)
        governed_wall_total += record["wall_seconds"]
    for key in replays:
        state, sampler = key.split(":")
        tag = f"{state.lower()}-{sampler}"
        replay = load(package / f"resource-replay-{tag}.json")
        verify_resource_record(replay, state, sampler, "replay")
        if replay.get("normalized_trace_match") is not True:
            fail(f"governed replay {key} did not match")
        if replay.get("trace_disposition") != "deleted after exact normalized comparison":
            fail(f"governed replay {key} scratch disposition drift")
        governed_wall_total += replay["wall_seconds"]
    if ledger.get("retained_bytes") != retained_total:
        fail("canonical retained-byte ledger mismatch")
    if abs(ledger.get("runner_wall_seconds", -1) - governed_wall_total) > 1e-6:
        fail("canonical runner-wall ledger mismatch")
    print(
        "block ensemble expansion verification: PASS "
        f"(primaries={len(primaries)}/6, replays={len(replays)}/6)"
    )


if __name__ == "__main__":
    main()
