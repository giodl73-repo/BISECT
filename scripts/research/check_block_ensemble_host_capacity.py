#!/usr/bin/env python3
"""Reject future governed ensemble launches without reserved host capacity."""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path


GIB = 1024**3
DEFAULT_SCRATCH_LIMIT_BYTES = 3 * GIB
DEFAULT_RETAINED_LIMIT_BYTES = 3 * GIB
DEFAULT_SAFETY_RESERVE_BYTES = 2 * GIB


def capacity_report(
    free_bytes: int,
    retained_used_bytes: int,
    scratch_limit_bytes: int = DEFAULT_SCRATCH_LIMIT_BYTES,
    retained_limit_bytes: int = DEFAULT_RETAINED_LIMIT_BYTES,
    safety_reserve_bytes: int = DEFAULT_SAFETY_RESERVE_BYTES,
) -> dict:
    values = {
        "free_bytes": free_bytes,
        "retained_used_bytes": retained_used_bytes,
        "scratch_limit_bytes": scratch_limit_bytes,
        "retained_limit_bytes": retained_limit_bytes,
        "safety_reserve_bytes": safety_reserve_bytes,
    }
    if any(value < 0 for value in values.values()):
        raise ValueError("capacity values must be non-negative")
    if retained_used_bytes > retained_limit_bytes:
        raise ValueError("retained evidence already exceeds its ceiling")
    retained_remaining = retained_limit_bytes - retained_used_bytes
    required_free = scratch_limit_bytes + retained_remaining + safety_reserve_bytes
    shortfall = max(0, required_free - free_bytes)
    return {
        "schema_version": "nrs-block-ensemble-host-capacity-v1",
        "status": "pass" if shortfall == 0 else "reject",
        **values,
        "retained_remaining_bytes": retained_remaining,
        "required_free_bytes": required_free,
        "shortfall_bytes": shortfall,
        "process_launch_authorized": shortfall == 0,
        "claim_boundary": (
            "Host-capacity admission only; this is not a governed chain, "
            "resource measurement, or statistical result."
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--scratch-limit-gib", type=int, default=3)
    parser.add_argument("--retained-limit-gib", type=int, default=3)
    parser.add_argument("--safety-reserve-gib", type=int, default=2)
    args = parser.parse_args()

    package = args.package.resolve()
    if not package.is_dir():
        raise SystemExit(f"package directory does not exist: {package}")
    ledger = json.loads(args.ledger.read_text(encoding="utf-8"))
    retained_used = ledger.get("retained_bytes")
    if not isinstance(retained_used, int):
        raise SystemExit("ledger retained_bytes must be an integer")
    usage = shutil.disk_usage(package)
    report = capacity_report(
        free_bytes=usage.free,
        retained_used_bytes=retained_used,
        scratch_limit_bytes=args.scratch_limit_gib * GIB,
        retained_limit_bytes=args.retained_limit_gib * GIB,
        safety_reserve_bytes=args.safety_reserve_gib * GIB,
    )
    report["package_path"] = str(package)
    report["ledger_path"] = str(args.ledger.resolve())
    report["volume_total_bytes"] = usage.total
    report["volume_used_bytes"] = usage.used
    payload = json.dumps(report, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload, encoding="utf-8")
    print(payload, end="")
    if report["status"] != "pass":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
