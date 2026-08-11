#!/usr/bin/env python3
"""Create the pristine v3 readiness package without launching a chain."""

from __future__ import annotations

import json
import platform
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from check_block_ensemble_host_capacity import capacity_report_for_paths
from run_block_ensemble_expansion_v3 import new_ledger
from verify_block_ensemble_v3_readiness import (
    INPUTS,
    PACKAGE,
    PROBE_SCHEMA,
    PROTOCOL_ID,
    READINESS_SCHEMA,
    binding_sha256,
    execute_probe,
    expected_probes,
    sha256,
)

EXECUTABLE = ROOT / "target/release/examples/block_trace.exe"
VALIDATOR = ROOT / "target/release/examples/validate_block_input.exe"
IMPLEMENTATION_BASE_COMMIT = "52dfb90df5c66b7e5969dec50924416bbb772ceb"


def write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def command_version(command: list[str]) -> str:
    completed = subprocess.run(command, capture_output=True, text=True, check=True)
    return completed.stdout.strip()


def prepare(package: Path = PACKAGE) -> dict:
    package = package.resolve()
    if package != PACKAGE.resolve():
        raise ValueError(f"v3 readiness package path must be {PACKAGE.resolve()}")
    if not EXECUTABLE.is_file() or not VALIDATOR.is_file():
        raise ValueError("release executables must be built before v3 readiness")
    package.mkdir(parents=True, exist_ok=True)
    allowed = {"README.md"}
    unexpected = [path.name for path in package.iterdir() if path.name not in allowed]
    if unexpected:
        raise ValueError(
            "v3 readiness package is not pristine: " + ", ".join(sorted(unexpected))
        )

    ledger_path = package / "ledger.json"
    write_json(ledger_path, new_ledger())
    for state, expected in INPUTS.items():
        slug = expected["slug"]
        rctx = ROOT / f"data/2020/certified/{slug}_blocks_2020.rctx"
        assignments = (
            ROOT
            / "runs/nrs-v0.3/neutral-analysis/national-2020/states"
            / slug
            / "package/baseline_assignments.json"
        )
        audit = {
            "schema_version": "nrs-block-ensemble-input-audit-v1",
            "status": "pass",
            "state": state,
            "year": 2020,
            "units": expected["units"],
            "districts": expected["districts"],
            "population_total": expected["population"],
            "max_population_deviation": expected["max_deviation"],
            "undirected_edges": expected["edges"],
            "rctx_sha256": sha256(rctx),
            "assignments_sha256": sha256(assignments),
            "claim_boundary": "Stage 0 candidate input audit only; no ensemble was executed.",
        }
        write_json(package / f"input-audit-{slug}.json", audit)

    probes = []
    for contract in expected_probes(EXECUTABLE):
        result = execute_probe(contract["argv"])
        if contract["kind"] == "positive" and result["returncode"] != 0:
            raise ValueError(f"compiled v3 contract rejected: {result['stderr']}")
        if contract["kind"] == "negative" and result["returncode"] == 0:
            raise ValueError("compiled runner accepted a forbidden v3 contract")
        probes.append({**contract, **result})
    probe_record = {
        "schema_version": PROBE_SCHEMA,
        "status": "pass",
        "protocol_id": PROTOCOL_ID,
        "observed_at_utc": datetime.now(timezone.utc).isoformat(),
        "runner_executable_sha256": sha256(EXECUTABLE),
        "probes": probes,
        "claim_boundary": (
            "Side-effect-free compiled argument validation only; no input was "
            "loaded, no trace was written, and no ensemble draw was executed."
        ),
    }
    probe_path = package / "compiled-contract-probes.json"
    write_json(probe_path, probe_record)

    capacity = capacity_report_for_paths(package, ledger_path)
    if capacity["status"] != "pass":
        raise ValueError("v3 readiness host capacity is insufficient")
    capacity.pop("package_path", None)
    capacity.pop("ledger_path", None)
    capacity.pop("volume_total_bytes", None)
    capacity.pop("volume_used_bytes", None)
    capacity["observed_at_utc"] = datetime.now(timezone.utc).isoformat()
    capacity["claim_boundary"] = (
        "Point-in-time readiness observation only; every process still requires a "
        "fresh host-capacity admission record."
    )

    paths = {
        "input-audit-nh.json": package / "input-audit-nh.json",
        "input-audit-nm.json": package / "input-audit-nm.json",
        "input-audit-ga.json": package / "input-audit-ga.json",
        "block_trace.exe": EXECUTABLE,
        "validate_block_input.exe": VALIDATOR,
        "block_trace.rs": ROOT / "crates/bisect-ensemble/examples/block_trace.rs",
        "validate_block_input.rs": ROOT / "crates/bisect-ensemble/examples/validate_block_input.rs",
        "run_block_ensemble_expansion_v3.py": ROOT / "scripts/research/run_block_ensemble_expansion_v3.py",
        "verify_block_ensemble_expansion_v3.py": ROOT / "scripts/research/verify_block_ensemble_expansion_v3.py",
        "verify_block_ensemble_v3_readiness.py": ROOT / "scripts/research/verify_block_ensemble_v3_readiness.py",
        "prepare_block_ensemble_v3_readiness.py": Path(__file__).resolve(),
        "check_block_ensemble_host_capacity.py": ROOT / "scripts/research/check_block_ensemble_host_capacity.py",
        "expansion-v3-protocol.md": ROOT / "docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v3.md",
        "resource-audit-manifest.json": ROOT / "docs/experiments/nrs-v0.3-block-ensemble-resource-audit/manifest.json",
        "compiled-contract-probes.json": probe_path,
    }
    readiness = {
        "schema_version": READINESS_SCHEMA,
        "status": "pass",
        "protocol_id": PROTOCOL_ID,
        "observed_at_utc": datetime.now(timezone.utc).isoformat(),
        "implementation_base_commit": IMPLEMENTATION_BASE_COMMIT,
        "build": {
            "profile": "release",
            "target": "x86_64-pc-windows-msvc",
            "rustc": command_version(["rustc", "--version"]),
            "cargo": command_version(["cargo", "--version"]),
            "platform": platform.platform(),
        },
        "sha256_bindings": {
            name: binding_sha256(path) for name, path in paths.items()
        },
        "capacity_snapshot": capacity,
        "claim_boundary": (
            "Local Stage 0 readiness and custody only; this does not authorize a "
            "process, establish reproducible binaries on another host, or report "
            "an ensemble result."
        ),
    }
    write_json(package / "readiness.json", readiness)
    return readiness


def main() -> None:
    try:
        readiness = prepare()
    except (OSError, ValueError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
    print(
        "block ensemble expansion v3 readiness prepared: PASS "
        f"(observed={readiness['observed_at_utc']}, probes=14)"
    )


if __name__ == "__main__":
    main()
