#!/usr/bin/env python3
"""Verify the completed excluded Stage 0 gate for expansion v3."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from run_block_ensemble_expansion_v3 import ORDER, PACKAGE, artifact_paths
from verify_block_ensemble_expansion_v3 import load, verify_package


def fail(message: str) -> None:
    raise ValueError(f"block ensemble expansion v3 Stage 0 failed: {message}")


def verify_stage0(package: Path = PACKAGE) -> dict:
    package = package.resolve()
    ledger = verify_package(package)
    completed = ledger["completed"]
    if ledger["status"] != "active":
        fail("ledger is not active after Stage 0")
    if completed["preflight"] != ORDER:
        fail("preflight schedule is incomplete")
    if completed["preflight-replay"] != ORDER:
        fail("preflight replay schedule is incomplete")
    if completed["primary"] or completed["replay"]:
        fail("governed execution began before Stage 0 retention")
    if ledger["failures"]:
        fail("ledger contains a failure")

    total_wall_seconds = 0.0
    peak_rss_bytes = 0
    retained_bytes = 0
    for phase in ("preflight", "preflight-replay"):
        for key in ORDER:
            state, sampler = key.split(":")
            paths = artifact_paths(package, state, sampler, phase)
            resource = load(paths["resource"])
            if resource.get("status") != "pass" or resource.get("returncode") != 0:
                fail(f"{phase} {key} resource did not pass")
            wall = resource.get("wall_seconds")
            peak = resource.get("peak_rss_bytes")
            if not isinstance(wall, (int, float)) or wall <= 0:
                fail(f"{phase} {key} wall time is invalid")
            if not isinstance(peak, int) or isinstance(peak, bool) or peak <= 0:
                fail(f"{phase} {key} peak RSS is invalid")
            total_wall_seconds += wall
            peak_rss_bytes = max(peak_rss_bytes, peak)
            if phase == "preflight":
                retained_bytes += paths["runner_trace"].stat().st_size
            else:
                if resource.get("normalized_trace_match") is not True:
                    fail(f"{phase} {key} is not an exact normalized match")
                if paths["runner_trace"].exists():
                    fail(f"{phase} {key} scratch trace remains")
    if retained_bytes != ledger["retained_bytes"]:
        fail("retained-byte total drift")
    if ledger["runner_wall_seconds"] != 0.0:
        fail("excluded Stage 0 time entered the governed wall ledger")
    return {
        "preflights": len(completed["preflight"]),
        "preflight_replays": len(completed["preflight-replay"]),
        "total_wall_seconds": total_wall_seconds,
        "peak_rss_bytes": peak_rss_bytes,
        "retained_bytes": retained_bytes,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path, nargs="?", default=PACKAGE)
    args = parser.parse_args()
    try:
        summary = verify_stage0(args.package)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
    print(
        "block ensemble expansion v3 Stage 0: PASS "
        f"(preflights={summary['preflights']}/6, "
        f"replays={summary['preflight_replays']}/6, "
        f"wall={summary['total_wall_seconds']:.4f}s, "
        f"peak_rss={summary['peak_rss_bytes']}, "
        f"retained={summary['retained_bytes']})"
    )


if __name__ == "__main__":
    main()
