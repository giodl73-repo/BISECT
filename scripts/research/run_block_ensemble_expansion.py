#!/usr/bin/env python3
"""Run the frozen NH/NM/GA block-ensemble schedule under hard resource caps."""

from __future__ import annotations

import argparse
import gzip
import json
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

import psutil


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from measure_block_ensemble_resources import memory_values, normalize_in_place, sha256

PROTOCOL = ROOT / "docs/specs/2026-08-10-nrs-v0.3-block-ensemble-expansion.md"
RUNNER = ROOT / "crates/bisect-ensemble/examples/block_trace.rs"
STATE_CONFIG = {
    "NH": {"districts": 2, "slug": "nh"},
    "NM": {"districts": 3, "slug": "nm"},
    "GA": {"districts": 14, "slug": "ga"},
}
ORDER = [f"{state}:{sampler}" for state in ("NH", "NM", "GA") for sampler in ("wilson", "kruskal")]
WALL_LIMIT_SECONDS = 21 * 3600
MEMORY_LIMIT_BYTES = 2415919104
RETAINED_LIMIT_BYTES = 3 * 1024**3
SCRATCH_LIMIT_BYTES = 3 * 1024**3


def new_ledger() -> dict:
    return {
        "schema_version": "nrs-block-ensemble-expansion-ledger-v1",
        "protocol_id": "nrs-v0.3-block-ensemble-expansion-v1",
        "status": "active",
        "completed": {"preflight": [], "preflight-replay": [], "primary": [], "replay": []},
        "runner_wall_seconds": 0.0,
        "retained_bytes": 0,
        "failures": [],
    }


def expected_next(ledger: dict, phase: str) -> str | None:
    completed = ledger["completed"]
    completed.setdefault("preflight-replay", [])
    if phase == "preflight":
        if completed["preflight-replay"] or completed["primary"] or completed["replay"]:
            return None
        index = len(completed["preflight"])
    elif phase == "preflight-replay":
        if completed["preflight"] != ORDER or completed["primary"] or completed["replay"]:
            return None
        index = len(completed["preflight-replay"])
    elif phase == "primary":
        if completed["preflight"] != ORDER or completed["preflight-replay"] != ORDER or completed["replay"]:
            return None
        index = len(completed["primary"])
    else:
        if completed["preflight"] != ORDER or completed["primary"] != ORDER:
            return None
        index = len(completed["replay"])
    return ORDER[index] if index < len(ORDER) else None


def validate_trace(trace: dict, state: str, sampler: str, phase: str) -> None:
    governed = phase in {"primary", "replay"}
    expected = {
        "schema_version": "nrs-block-ensemble-trace-v1",
        "status": "complete",
        "execution_class": "governed-stage2" if governed else "excluded-expansion-preflight",
        "state": state,
        "year": 2020,
        "districts": STATE_CONFIG[state]["districts"],
        "sampler": sampler,
        "chains": 4 if governed else 1,
        "steps_per_chain": 2000 if governed else 25,
        "population_tolerance": 0.005,
        "base_seed": 20260810,
        "snapshot_stride": 10,
    }
    for key, value in expected.items():
        if trace.get(key) != value:
            raise ValueError(f"trace {key} drift")
    steps = expected["steps_per_chain"]
    snapshots = list(range(10, steps + 1, 10))
    if len(trace["chain_traces"]) != expected["chains"]:
        raise ValueError("trace chain-count drift")
    for index, chain in enumerate(trace["chain_traces"]):
        if chain["chain_index"] != index:
            raise ValueError("trace chain-index drift")
        if [row["step"] for row in chain["metrics"]] != list(range(1, steps + 1)):
            raise ValueError("trace metric schedule drift")
        if [row["step"] for row in chain["snapshots"]] != snapshots:
            raise ValueError("trace snapshot schedule drift")
        if any(row["max_population_deviation"] > 0.005 for row in chain["metrics"]):
            raise ValueError("trace population tolerance failure")


def command(executable: Path, state: str, sampler: str, phase: str, output: Path) -> list[str]:
    config = STATE_CONFIG[state]
    governed = phase in {"primary", "replay"}
    slug = config["slug"]
    return [
        str(executable),
        "--rctx", str(ROOT / f"data/2020/certified/{slug}_blocks_2020.rctx"),
        "--assignments", str(ROOT / f"runs/nrs-v0.3/neutral-analysis/national-2020/states/{slug}/package/baseline_assignments.json"),
        "--state", state,
        "--year", "2020",
        "--districts", str(config["districts"]),
        "--tolerance", "0.005",
        "--sampler", sampler,
        "--steps", "2000" if governed else "25",
        "--chains", "4" if governed else "1",
        "--base-seed", "20260810",
        "--snapshot-stride", "10",
        "--execution-class", "governed-stage2" if governed else "excluded-expansion-preflight",
        "--output", str(output),
    ]


def write_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def load_trace(path: Path) -> dict:
    if path.suffix == ".gz":
        with gzip.open(path, "rt", encoding="utf-8") as handle:
            return json.load(handle)
    return json.loads(path.read_text(encoding="utf-8"))


def compress_trace(raw: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    with raw.open("rb") as source, destination.open("wb") as target:
        with gzip.GzipFile(filename="", mode="wb", fileobj=target, mtime=0) as compressed:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                compressed.write(chunk)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--state", choices=tuple(STATE_CONFIG), required=True)
    parser.add_argument("--sampler", choices=("wilson", "kruskal"), required=True)
    parser.add_argument(
        "--phase",
        choices=("preflight", "preflight-replay", "primary", "replay"),
        required=True,
    )
    parser.add_argument("--trace", type=Path, required=True)
    parser.add_argument("--resource-record", type=Path, required=True)
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--committed-trace", type=Path)
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument(
        "--executable",
        type=Path,
        default=ROOT / "target/release/examples/block_trace.exe",
    )
    args = parser.parse_args()
    if not 1 <= args.poll_ms <= 50:
        raise SystemExit("poll interval must be between 1 and 50 ms")
    if args.phase in {"preflight-replay", "replay"} and args.committed_trace is None:
        raise SystemExit("replay phases require --committed-trace")
    final_trace_path = args.trace.resolve()
    if args.phase == "primary" and not final_trace_path.name.endswith(".json.gz"):
        raise SystemExit("primary trace path must end in .json.gz")
    runner_trace_path = (
        final_trace_path.with_suffix("") if args.phase == "primary" else final_trace_path
    )
    if final_trace_path.exists() or runner_trace_path.exists() or args.resource_record.exists():
        raise SystemExit("trace and resource-record paths must not already exist")
    ledger = json.loads(args.ledger.read_text()) if args.ledger.exists() else new_ledger()
    key = f"{args.state}:{args.sampler}"
    if expected_next(ledger, args.phase) != key:
        raise SystemExit(f"schedule violation: {key} is not the next {args.phase} run")

    executable = args.executable.resolve()
    run_command = command(executable, args.state, args.sampler, args.phase, runner_trace_path)
    runner_trace_path.parent.mkdir(parents=True, exist_ok=True)
    started_at = datetime.now(timezone.utc)
    started = time.perf_counter()
    child = subprocess.Popen(run_command, cwd=ROOT)
    process = psutil.Process(child.pid)
    sampled_peak = 0
    os_peak: int | None = None
    samples = 0
    failure: str | None = None
    while True:
        sampled, reported = memory_values(process)
        sampled_peak = max(sampled_peak, sampled)
        if reported is not None:
            os_peak = max(os_peak or 0, reported)
        samples += 1
        elapsed = time.perf_counter() - started
        if max(sampled_peak, os_peak or 0) > MEMORY_LIMIT_BYTES:
            failure = "per-process memory ceiling exceeded"
        if args.phase in {"primary", "replay"} and ledger["runner_wall_seconds"] + elapsed > WALL_LIMIT_SECONDS:
            failure = "cumulative runner wall ceiling exceeded"
        if failure and child.poll() is None:
            child.terminate()
        returncode = child.poll()
        if returncode is not None:
            break
        time.sleep(args.poll_ms / 1000)
    wall_seconds = time.perf_counter() - started
    peak = max(sampled_peak, os_peak or 0)
    record = {
        "schema_version": "nrs-block-ensemble-expansion-resource-v1",
        "status": "fail" if failure or returncode != 0 else "pass",
        "protocol_id": "nrs-v0.3-block-ensemble-expansion-v1",
        "phase": args.phase,
        "state": args.state,
        "sampler": args.sampler,
        "poll_interval_ms": args.poll_ms,
        "sample_count": samples,
        "started_at_utc": started_at.isoformat(),
        "finished_at_utc": datetime.now(timezone.utc).isoformat(),
        "wall_seconds": wall_seconds,
        "sampled_peak_rss_bytes": sampled_peak,
        "os_reported_peak_rss_bytes": os_peak,
        "peak_rss_bytes": peak,
        "returncode": returncode,
        "failure": failure,
        "platform": platform.platform(),
        "runner_executable_sha256": sha256(executable),
        "runner_source_sha256": sha256(RUNNER),
        "protocol_sha256": sha256(PROTOCOL),
        "trace_disposition": "retained",
        "claim_boundary": "Resource enforcement only; preflight and replay samples are excluded from statistical analysis.",
    }
    if runner_trace_path.is_file():
        record["raw_trace_size_bytes"] = runner_trace_path.stat().st_size
        record["raw_trace_sha256"] = sha256(runner_trace_path)
    if failure or returncode != 0:
        ledger["status"] = "failed"
        ledger["failures"].append({"key": key, "phase": args.phase, "reason": failure or f"runner returned {returncode}"})
        write_json(args.resource_record, record)
        write_json(args.ledger, ledger)
        raise SystemExit(f"expansion run failed: {ledger['failures'][-1]['reason']}")

    trace = load_trace(runner_trace_path)
    validate_trace(trace, args.state, args.sampler, args.phase)
    if args.phase in {"preflight-replay", "replay"}:
        committed = load_trace(args.committed_trace)
        validate_trace(
            committed,
            args.state,
            args.sampler,
            "primary" if args.phase == "replay" else "preflight",
        )
        normalize_in_place(trace)
        normalize_in_place(committed)
        if trace != committed:
            record["status"] = "fail"
            record["failure"] = "normalized replay mismatch"
            ledger["status"] = "failed"
            ledger["failures"].append({"key": key, "phase": args.phase, "reason": record["failure"]})
            write_json(args.resource_record, record)
            write_json(args.ledger, ledger)
            raise SystemExit("normalized replay mismatch; scratch retained")
        if record["raw_trace_size_bytes"] > SCRATCH_LIMIT_BYTES:
            raise SystemExit("scratch storage ceiling exceeded")
        runner_trace_path.unlink()
        record["trace_disposition"] = "deleted after exact normalized comparison"
        record["normalized_trace_match"] = True
    else:
        if args.phase == "primary":
            compress_trace(runner_trace_path, final_trace_path)
            runner_trace_path.unlink()
            record["trace_disposition"] = "raw deleted after deterministic gzip custody"
            record["retained_trace_size_bytes"] = final_trace_path.stat().st_size
            record["retained_trace_sha256"] = sha256(final_trace_path)
            retained_size = record["retained_trace_size_bytes"]
        else:
            record["retained_trace_size_bytes"] = record["raw_trace_size_bytes"]
            record["retained_trace_sha256"] = record["raw_trace_sha256"]
            retained_size = record["retained_trace_size_bytes"]
        retained_after = ledger["retained_bytes"] + retained_size
        if retained_after > RETAINED_LIMIT_BYTES:
            raise SystemExit("retained storage ceiling exceeded")
        ledger["retained_bytes"] = retained_after
    if args.phase in {"primary", "replay"}:
        ledger["runner_wall_seconds"] += wall_seconds
    ledger["completed"][args.phase].append(key)
    if ledger["completed"]["replay"] == ORDER:
        ledger["status"] = "complete"
    write_json(args.resource_record, record)
    write_json(args.ledger, ledger)
    print(f"{args.phase} {key}: PASS")


if __name__ == "__main__":
    main()
