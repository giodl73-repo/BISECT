#!/usr/bin/env python3
"""Launch a future block-ensemble process only after host-capacity admission."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from collections.abc import Callable, Sequence
from pathlib import Path
from typing import Any

from check_block_ensemble_host_capacity import (
    DEFAULT_RETAINED_LIMIT_BYTES,
    DEFAULT_SAFETY_RESERVE_BYTES,
    DEFAULT_SCRATCH_LIMIT_BYTES,
    GIB,
    capacity_report_for_paths,
)


def launch_if_admitted(
    package: Path,
    ledger_path: Path,
    admission_record: Path,
    command: Sequence[str],
    cwd: Path,
    scratch_limit_bytes: int = DEFAULT_SCRATCH_LIMIT_BYTES,
    retained_limit_bytes: int = DEFAULT_RETAINED_LIMIT_BYTES,
    safety_reserve_bytes: int = DEFAULT_SAFETY_RESERVE_BYTES,
    disk_usage: Callable[[Path], Any] = shutil.disk_usage,
    run: Callable[..., subprocess.CompletedProcess] = subprocess.run,
) -> int:
    """Write a new admission record, then launch only when it authorizes launch."""
    if not command:
        raise ValueError("launch command must not be empty")
    if admission_record.exists():
        raise FileExistsError(f"admission record already exists: {admission_record}")
    report = capacity_report_for_paths(
        package=package,
        ledger_path=ledger_path,
        scratch_limit_bytes=scratch_limit_bytes,
        retained_limit_bytes=retained_limit_bytes,
        safety_reserve_bytes=safety_reserve_bytes,
        disk_usage=disk_usage,
    )
    admission_record.parent.mkdir(parents=True, exist_ok=True)
    with admission_record.open("x", encoding="utf-8", newline="\n") as handle:
        json.dump(report, handle, indent=2)
        handle.write("\n")
    if not report["process_launch_authorized"]:
        return 1
    completed = run(list(command), cwd=cwd.resolve(), check=False)
    return completed.returncode


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--admission-record", type=Path, required=True)
    parser.add_argument("--cwd", type=Path, default=Path.cwd())
    parser.add_argument("--scratch-limit-gib", type=int, default=3)
    parser.add_argument("--retained-limit-gib", type=int, default=3)
    parser.add_argument("--safety-reserve-gib", type=int, default=2)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    try:
        returncode = launch_if_admitted(
            package=args.package,
            ledger_path=args.ledger,
            admission_record=args.admission_record,
            command=command,
            cwd=args.cwd,
            scratch_limit_bytes=args.scratch_limit_gib * GIB,
            retained_limit_bytes=args.retained_limit_gib * GIB,
            safety_reserve_bytes=args.safety_reserve_gib * GIB,
        )
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
    raise SystemExit(returncode)


if __name__ == "__main__":
    main()
