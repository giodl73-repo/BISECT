#!/usr/bin/env python3
"""Verify the immutable inputs and local executable custody for v2 Stage 0."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / "docs/experiments/nrs-v0.3-block-ensemble-expansion-v2"
READINESS_SCHEMA = "nrs-block-ensemble-expansion-v2-readiness-v1"
AUDIT_SCHEMA = "nrs-block-ensemble-input-audit-v1"
CAPACITY_SCHEMA = "nrs-block-ensemble-host-capacity-v1"
GIB = 1024**3

INPUTS = {
    "NH": {
        "slug": "nh",
        "districts": 2,
        "units": 31_948,
        "edges": 78_880,
        "population": 1_377_529,
        "max_deviation": 0.000007985312831889564,
    },
    "NM": {
        "slug": "nm",
        "districts": 3,
        "units": 107_215,
        "edges": 268_806,
        "population": 2_117_522,
        "max_deviation": 0.000010861752557887751,
    },
    "GA": {
        "slug": "ga",
        "districts": 14,
        "units": 232_717,
        "edges": 575_304,
        "population": 10_711_908,
        "max_deviation": 0.0029925574416808346,
    },
}


def fail(message: str) -> None:
    raise ValueError(f"block ensemble expansion v2 readiness failed: {message}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load(path: Path) -> dict:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        fail(f"{path.name} is not a JSON object")
    return value


def verify_audit(package: Path, state: str, expected: dict) -> None:
    path = package / f"input-audit-{expected['slug']}.json"
    audit = load(path)
    fixed = {
        "schema_version": AUDIT_SCHEMA,
        "status": "pass",
        "state": state,
        "year": 2020,
        "units": expected["units"],
        "districts": expected["districts"],
        "population_total": expected["population"],
        "max_population_deviation": expected["max_deviation"],
        "undirected_edges": expected["edges"],
        "claim_boundary": "Stage 0 candidate input audit only; no ensemble was executed.",
    }
    for key, value in fixed.items():
        if audit.get(key) != value:
            fail(f"{state} input audit {key} drift")
    rctx = ROOT / f"data/2020/certified/{expected['slug']}_blocks_2020.rctx"
    assignments = (
        ROOT
        / "runs/nrs-v0.3/neutral-analysis/national-2020/states"
        / expected["slug"]
        / "package/baseline_assignments.json"
    )
    if audit.get("rctx_sha256") != sha256(rctx):
        fail(f"{state} RCTX hash mismatch")
    if audit.get("assignments_sha256") != sha256(assignments):
        fail(f"{state} assignment hash mismatch")


def verify_capacity(snapshot: dict) -> None:
    expected = {
        "schema_version": CAPACITY_SCHEMA,
        "status": "pass",
        "retained_used_bytes": 0,
        "scratch_limit_bytes": 3 * GIB,
        "retained_limit_bytes": 3 * GIB,
        "safety_reserve_bytes": 2 * GIB,
        "retained_remaining_bytes": 3 * GIB,
        "required_free_bytes": 8 * GIB,
        "shortfall_bytes": 0,
        "process_launch_authorized": True,
    }
    for key, value in expected.items():
        if snapshot.get(key) != value:
            fail(f"capacity snapshot {key} drift")
    free = snapshot.get("free_bytes")
    if not isinstance(free, int) or isinstance(free, bool) or free < 8 * GIB:
        fail("capacity snapshot free bytes are invalid")
    if snapshot.get("claim_boundary") != (
        "Point-in-time readiness observation only; every process still requires a "
        "fresh host-capacity admission record."
    ):
        fail("capacity snapshot claim boundary drift")


def verify_empty_package(package: Path) -> None:
    ledger = load(package / "ledger.json")
    if ledger != {
        "schema_version": "nrs-block-ensemble-expansion-ledger-v2",
        "protocol_id": "nrs-v0.3-block-ensemble-expansion-v2",
        "status": "active",
        "completed": {
            "preflight": [],
            "preflight-replay": [],
            "primary": [],
            "replay": [],
        },
        "runner_wall_seconds": 0.0,
        "retained_bytes": 0,
        "failures": [],
    }:
        fail("ledger is not the pristine v2 state")
    forbidden = ("admission-", "resource-", "preflight-", "governed-", "replay-")
    unexpected = [
        path.name for path in package.iterdir() if path.name.startswith(forbidden)
    ]
    if unexpected:
        fail(f"process artifacts exist: {', '.join(sorted(unexpected))}")


def verify_readiness(package: Path = PACKAGE) -> dict:
    package = package.resolve()
    readiness = load(package / "readiness.json")
    if readiness.get("schema_version") != READINESS_SCHEMA:
        fail("readiness schema drift")
    if readiness.get("status") != "pass":
        fail("readiness status is not pass")
    if readiness.get("protocol_id") != "nrs-v0.3-block-ensemble-expansion-v2":
        fail("protocol identity drift")
    if readiness.get("implementation_base_commit") != (
        "0ac2b84da7785562ad0be389aa71395a8d9172f5"
    ):
        fail("implementation base commit drift")

    verify_empty_package(package)
    for state, expected in INPUTS.items():
        verify_audit(package, state, expected)

    bindings = readiness.get("sha256_bindings")
    if not isinstance(bindings, dict):
        fail("SHA-256 bindings are invalid")
    expected_paths = {
        "input-audit-nh.json": package / "input-audit-nh.json",
        "input-audit-nm.json": package / "input-audit-nm.json",
        "input-audit-ga.json": package / "input-audit-ga.json",
        "block_trace.exe": ROOT / "target/release/examples/block_trace.exe",
        "validate_block_input.exe": (
            ROOT / "target/release/examples/validate_block_input.exe"
        ),
        "block_trace.rs": ROOT / "crates/bisect-ensemble/examples/block_trace.rs",
        "validate_block_input.rs": (
            ROOT / "crates/bisect-ensemble/examples/validate_block_input.rs"
        ),
        "run_block_ensemble_expansion_v2.py": (
            ROOT / "scripts/research/run_block_ensemble_expansion_v2.py"
        ),
        "verify_block_ensemble_expansion_v2.py": (
            ROOT / "scripts/research/verify_block_ensemble_expansion_v2.py"
        ),
        "check_block_ensemble_host_capacity.py": (
            ROOT / "scripts/research/check_block_ensemble_host_capacity.py"
        ),
        "expansion-v2-protocol.md": (
            ROOT / "docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v2.md"
        ),
        "resource-audit-manifest.json": (
            ROOT
            / "docs/experiments/nrs-v0.3-block-ensemble-resource-audit/manifest.json"
        ),
    }
    if set(bindings) != set(expected_paths):
        fail("SHA-256 binding set drift")
    for name, path in expected_paths.items():
        if not path.is_file() or bindings[name] != sha256(path):
            fail(f"SHA-256 binding mismatch for {name}")

    verify_capacity(readiness.get("capacity_snapshot", {}))
    if readiness.get("claim_boundary") != (
        "Local Stage 0 readiness and custody only; this does not authorize a process, "
        "establish reproducible binaries on another host, or report an ensemble result."
    ):
        fail("readiness claim boundary drift")
    return readiness


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path, nargs="?", default=PACKAGE)
    args = parser.parse_args()
    try:
        readiness = verify_readiness(args.package)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
    print(
        "block ensemble expansion v2 readiness: PASS "
        f"(observed={readiness['observed_at_utc']}, process_artifacts=0)"
    )


if __name__ == "__main__":
    main()
