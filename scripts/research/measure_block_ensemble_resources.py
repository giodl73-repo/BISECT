#!/usr/bin/env python3
"""Measure excluded resource replays for the governed RI block ensemble."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

import psutil


ROOT = Path(__file__).resolve().parents[2]
RESOURCE_PROTOCOL = ROOT / "docs/specs/2026-08-10-nrs-v0.3-block-ensemble-resource-audit.md"
STAGE1_PROTOCOL = ROOT / "docs/specs/2026-08-10-nrs-v0.3-block-ensemble-gate.md"
RUNNER = ROOT / "crates/bisect-ensemble/examples/block_trace.rs"
RCTX = ROOT / "data/2020/certified/ri_blocks_2020.rctx"
ASSIGNMENTS = ROOT / (
    "runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/"
    "baseline_assignments.json"
)
PACKAGE = ROOT / "docs/experiments/nrs-v0.3-block-ensemble-gate"
MIB = 1024**2
GIB = 1024**3


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalize_in_place(trace: dict) -> None:
    for chain in trace["chain_traces"]:
        for metric in chain["metrics"]:
            metric["runtime_ms"] = 0.0


def normalized_sha256(trace: dict) -> str:
    payload = json.dumps(trace, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def memory_values(process: psutil.Process) -> tuple[int, int | None]:
    processes = [process]
    try:
        processes.extend(process.children(recursive=True))
    except (psutil.NoSuchProcess, psutil.AccessDenied):
        pass
    sampled = 0
    reported_peaks: list[int] = []
    for item in processes:
        try:
            info = item.memory_info()
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
        sampled += int(info.rss)
        peak = getattr(info, "peak_wset", None)
        if peak is not None:
            reported_peaks.append(int(peak))
    return sampled, sum(reported_peaks) if reported_peaks else None


def runner_command(executable: Path, sampler: str, scratch_trace: Path) -> list[str]:
    return [
        str(executable),
        "--rctx", str(RCTX),
        "--assignments", str(ASSIGNMENTS),
        "--state", "RI",
        "--year", "2020",
        "--districts", "2",
        "--tolerance", "0.005",
        "--sampler", sampler,
        "--steps", "2000",
        "--chains", "4",
        "--base-seed", "20260810",
        "--snapshot-stride", "10",
        "--execution-class", "governed-stage1",
        "--output", str(scratch_trace),
    ]


def measure(executable: Path, sampler: str, scratch_trace: Path, poll_ms: int) -> dict:
    if sampler not in {"wilson", "kruskal"}:
        raise ValueError("sampler must be wilson or kruskal")
    if poll_ms <= 0 or poll_ms > 50:
        raise ValueError("poll interval must be between 1 and 50 ms")
    committed_path = PACKAGE / f"governed-{sampler}.json"
    command = runner_command(executable, sampler, scratch_trace)
    scratch_trace.parent.mkdir(parents=True, exist_ok=True)
    if scratch_trace.exists():
        raise FileExistsError(f"scratch trace already exists: {scratch_trace}")

    started_at = datetime.now(timezone.utc)
    started = time.perf_counter()
    child = subprocess.Popen(command, cwd=ROOT)
    process = psutil.Process(child.pid)
    sampled_peak = 0
    os_peak: int | None = None
    sample_count = 0
    while True:
        sampled, reported = memory_values(process)
        sampled_peak = max(sampled_peak, sampled)
        if reported is not None:
            os_peak = max(os_peak or 0, reported)
        sample_count += 1
        returncode = child.poll()
        if returncode is not None:
            break
        time.sleep(poll_ms / 1000)
    wall_seconds = time.perf_counter() - started
    finished_at = datetime.now(timezone.utc)
    if returncode != 0:
        raise RuntimeError(
            f"{sampler} resource replay returned {returncode}; scratch retained at {scratch_trace}"
        )
    if not scratch_trace.is_file():
        raise RuntimeError(f"{sampler} resource replay did not produce a trace")

    raw_size = scratch_trace.stat().st_size
    raw_hash = sha256(scratch_trace)
    replay = json.loads(scratch_trace.read_text(encoding="utf-8"))
    committed = json.loads(committed_path.read_text(encoding="utf-8"))
    for name, trace in (("replay", replay), ("committed", committed)):
        if trace.get("execution_class") != "governed-stage1":
            raise ValueError(f"{sampler} {name} execution class drift")
        if trace.get("sampler") != sampler or trace.get("chains") != 4:
            raise ValueError(f"{sampler} {name} frozen shape drift")
        if trace.get("steps_per_chain") != 2000:
            raise ValueError(f"{sampler} {name} step-count drift")
    rows = [row for chain in replay["chain_traces"] for row in chain["metrics"]]
    acceptance_rate = sum(bool(row["accepted"]) for row in rows) / len(rows)
    maximum_deviation = max(row["max_population_deviation"] for row in rows)
    mean_runtime_ms = sum(row["runtime_ms"] for row in rows) / len(rows)
    normalize_in_place(replay)
    normalize_in_place(committed)
    replay_normalized_hash = normalized_sha256(replay)
    committed_normalized_hash = normalized_sha256(committed)
    normalized_match = replay == committed
    if not normalized_match:
        raise ValueError(f"{sampler} normalized resource replay mismatch; scratch retained")

    record = {
        "schema_version": "nrs-block-ensemble-resource-measurement-v1",
        "status": "pass",
        "execution_class": "excluded-resource-replay",
        "protocol_id": "nrs-v0.3-block-ensemble-resource-audit-v1",
        "sampler": sampler,
        "poll_interval_ms": poll_ms,
        "sample_count": sample_count,
        "started_at_utc": started_at.isoformat(),
        "finished_at_utc": finished_at.isoformat(),
        "wall_seconds": wall_seconds,
        "sampled_peak_rss_bytes": sampled_peak,
        "os_reported_peak_rss_bytes": os_peak,
        "peak_rss_bytes": max(sampled_peak, os_peak or 0),
        "platform": platform.platform(),
        "python_version": platform.python_version(),
        "psutil_version": psutil.__version__,
        "runner_executable_sha256": sha256(executable),
        "runner_source_sha256": sha256(RUNNER),
        "resource_protocol_sha256": sha256(RESOURCE_PROTOCOL),
        "stage1_protocol_sha256": sha256(STAGE1_PROTOCOL),
        "rctx_sha256": sha256(RCTX),
        "assignments_sha256": sha256(ASSIGNMENTS),
        "committed_trace_sha256": sha256(committed_path),
        "scratch_trace_sha256": raw_hash,
        "scratch_trace_size_bytes": raw_size,
        "normalized_committed_sha256": committed_normalized_hash,
        "normalized_replay_sha256": replay_normalized_hash,
        "normalized_trace_match": normalized_match,
        "acceptance_rate": acceptance_rate,
        "maximum_population_deviation": maximum_deviation,
        "mean_step_runtime_ms": mean_runtime_ms,
        "command": ["<runner>" if value == str(executable) else "<scratch_trace>" if value == str(scratch_trace) else value for value in command],
        "scratch_disposition": "deleted after exact normalized comparison",
        "claim_boundary": "Excluded author-machine resource replay only; not an additional statistical sample.",
    }
    scratch_trace.unlink()
    return record


def calculate_budgets(measurements: list[dict], units: dict[str, int]) -> dict:
    if set(units) != {"RI", "NH", "NM", "GA"} or any(value <= 0 for value in units.values()):
        raise ValueError("unit counts must contain positive RI, NH, NM, and GA values")
    if {record.get("sampler") for record in measurements} != {"wilson", "kruskal"}:
        raise ValueError("one Wilson and one Kruskal measurement are required")
    for record in measurements:
        if record.get("status") != "pass" or not record.get("normalized_trace_match"):
            raise ValueError("resource measurement did not pass exact replay")
    ratios = {state: units[state] / units["RI"] for state in ("NH", "NM", "GA")}
    ratio_sum = sum(ratios.values())
    ratio_max = max(ratios.values())
    ri_wall = sum(float(record["wall_seconds"]) for record in measurements)
    ri_peak = max(int(record["peak_rss_bytes"]) for record in measurements)
    ri_storage = sum(int(record["scratch_trace_size_bytes"]) for record in measurements)
    projected_compute_seconds = ratio_sum * ri_wall
    compute_budget_hours = math.ceil(2 * projected_compute_seconds / 3600)
    projected_peak_bytes = ratio_max * ri_peak
    memory_budget_bytes = math.ceil(1.5 * projected_peak_bytes / (256 * MIB)) * 256 * MIB
    projected_storage_bytes = ratio_sum * ri_storage
    retained_storage_budget_gib = math.ceil(2 * projected_storage_bytes / GIB)
    eligible = (
        compute_budget_hours <= 48
        and memory_budget_bytes <= 4 * GIB
        and retained_storage_budget_gib <= 8
    )
    return {
        "unit_counts": units,
        "unit_ratios_to_ri": ratios,
        "ri_measured_wall_seconds": ri_wall,
        "ri_measured_peak_rss_bytes": ri_peak,
        "ri_measured_trace_bytes": ri_storage,
        "projected_compute_seconds": projected_compute_seconds,
        "authorized_compute_budget_hours": compute_budget_hours,
        "projected_peak_rss_bytes": projected_peak_bytes,
        "authorized_memory_budget_bytes": memory_budget_bytes,
        "projected_retained_storage_bytes": projected_storage_bytes,
        "authorized_retained_storage_gib": retained_storage_budget_gib,
        "authorized_scratch_storage_gib": retained_storage_budget_gib,
        "hard_ceilings": {
            "compute_hours": 48,
            "memory_bytes": 4 * GIB,
            "retained_storage_gib": 8,
        },
        "expansion_protocol_draft_eligible": eligible,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command_name", required=True)
    measure_parser = subparsers.add_parser("measure")
    measure_parser.add_argument("--sampler", choices=("wilson", "kruskal"), required=True)
    measure_parser.add_argument("--output", type=Path, required=True)
    measure_parser.add_argument("--scratch-trace", type=Path, required=True)
    measure_parser.add_argument("--poll-ms", type=int, default=50)
    measure_parser.add_argument(
        "--executable",
        type=Path,
        default=ROOT / "target/release/examples/block_trace.exe",
    )
    summary_parser = subparsers.add_parser("summarize")
    summary_parser.add_argument("--wilson", type=Path, required=True)
    summary_parser.add_argument("--kruskal", type=Path, required=True)
    summary_parser.add_argument("--input-audit", type=Path, action="append", required=True)
    summary_parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    if args.command_name == "measure":
        record = measure(
            args.executable.resolve(),
            args.sampler,
            args.scratch_trace.resolve(),
            args.poll_ms,
        )
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        print(f"{args.sampler} resource replay: PASS")
        return

    measurements = [
        json.loads(args.wilson.read_text(encoding="utf-8")),
        json.loads(args.kruskal.read_text(encoding="utf-8")),
    ]
    audits = [json.loads(path.read_text(encoding="utf-8")) for path in args.input_audit]
    units = {audit["state"]: int(audit["units"]) for audit in audits}
    summary = {
        "schema_version": "nrs-block-ensemble-resource-summary-v1",
        "status": "pass",
        "protocol_id": "nrs-v0.3-block-ensemble-resource-audit-v1",
        **calculate_budgets(measurements, units),
        "claim_boundary": "Author-machine planning only; drafting eligibility is not execution authorization.",
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print("block ensemble resource budget summary: PASS")


if __name__ == "__main__":
    main()
