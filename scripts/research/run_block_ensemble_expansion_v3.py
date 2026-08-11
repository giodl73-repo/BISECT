#!/usr/bin/env python3
"""Run the frozen NH/NM/GA v3 schedule behind capacity admission."""

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
from types import SimpleNamespace

import psutil


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from launch_block_ensemble_admitted import launch_if_admitted
from measure_block_ensemble_resources import memory_values, normalize_in_place, sha256

PROTOCOL_ID = "nrs-v0.3-block-ensemble-expansion-v3"
LEDGER_SCHEMA = "nrs-block-ensemble-expansion-ledger-v3"
RESOURCE_SCHEMA = "nrs-block-ensemble-expansion-resource-v3"
TRACE_SCHEMA = "nrs-block-ensemble-trace-v1"
BASE_SEED = 20260812
PROTOCOL = ROOT / "docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v3.md"
RUNNER = ROOT / "crates/bisect-ensemble/examples/block_trace.rs"
WRAPPER = Path(__file__).resolve()
PACKAGE = ROOT / "docs/experiments/nrs-v0.3-block-ensemble-expansion-v3"
EXECUTABLE = ROOT / "target/release/examples/block_trace.exe"
READINESS = PACKAGE / "readiness.json"
STATE_CONFIG = {
    "NH": {"districts": 2, "slug": "nh"},
    "NM": {"districts": 3, "slug": "nm"},
    "GA": {"districts": 14, "slug": "ga"},
}
ORDER = [
    f"{state}:{sampler}"
    for state in ("NH", "NM", "GA")
    for sampler in ("wilson", "kruskal")
]
WALL_LIMIT_SECONDS = 21 * 3600
MEMORY_LIMIT_BYTES = 2_415_919_104
RETAINED_LIMIT_BYTES = 3 * 1024**3
SCRATCH_LIMIT_BYTES = 3 * 1024**3


def new_ledger() -> dict:
    return {
        "schema_version": LEDGER_SCHEMA,
        "protocol_id": PROTOCOL_ID,
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
    }


def validate_ledger_identity(ledger: dict) -> None:
    if ledger.get("schema_version") != LEDGER_SCHEMA:
        raise ValueError("v3 ledger schema mismatch")
    if ledger.get("protocol_id") != PROTOCOL_ID:
        raise ValueError("v3 ledger protocol mismatch")


def require_official_package(package: Path) -> Path:
    resolved = package.resolve()
    if resolved != PACKAGE.resolve():
        raise ValueError(f"v3 package path must be {PACKAGE.resolve()}")
    return resolved


def require_bound_executable(
    executable: Path, readiness_path: Path = READINESS
) -> Path:
    resolved = executable.resolve()
    if resolved != EXECUTABLE.resolve():
        raise ValueError(f"v3 executable path must be {EXECUTABLE.resolve()}")
    readiness = json.loads(readiness_path.read_text(encoding="utf-8"))
    expected = readiness.get("sha256_bindings", {}).get("block_trace.exe")
    if not isinstance(expected, str) or sha256(resolved) != expected:
        raise ValueError("v3 executable does not match the readiness binding")
    return resolved


def close_ledger_failure(ledger: dict, key: str, phase: str, reason: str) -> None:
    validate_ledger(ledger)
    ledger["status"] = "failed"
    ledger["failures"].append({"key": key, "phase": phase, "reason": reason})


def capacity_admitted_launch(
    package: Path,
    ledger_path: Path,
    admission_path: Path,
    run_command: list[str],
    monitored_run,
    launcher=launch_if_admitted,
) -> int:
    return launcher(
        package=package,
        ledger_path=ledger_path,
        admission_record=admission_path,
        command=run_command,
        cwd=ROOT,
        run=monitored_run,
    )


def require_prefix(values: list[str], phase: str) -> None:
    if values != ORDER[: len(values)]:
        raise ValueError(f"{phase} schedule is not a frozen-order prefix")


def validate_ledger(ledger: dict) -> None:
    validate_ledger_identity(ledger)
    if ledger.get("status") not in {"active", "complete", "failed"}:
        raise ValueError("v3 ledger status is invalid")
    completed = ledger.get("completed")
    if not isinstance(completed, dict) or set(completed) != {
        "preflight",
        "preflight-replay",
        "primary",
        "replay",
    }:
        raise ValueError("v3 ledger completion phases mismatch")
    for phase, values in completed.items():
        if not isinstance(values, list):
            raise ValueError(f"{phase} completion list is invalid")
        require_prefix(values, phase)
    if completed["preflight-replay"] and completed["preflight"] != ORDER:
        raise ValueError("preflight replay began before all preflights completed")
    if completed["primary"] and completed["preflight-replay"] != ORDER:
        raise ValueError("primary began before Stage 0 completed")
    if completed["replay"] and completed["primary"] != ORDER:
        raise ValueError("replay began before all primaries completed")
    if ledger["status"] == "complete" and completed["replay"] != ORDER:
        raise ValueError("v3 ledger completed before all replays")
    if ledger["status"] == "active" and completed["replay"] == ORDER:
        raise ValueError("v3 ledger remained active after all replays")
    failures = ledger.get("failures")
    if not isinstance(failures, list):
        raise ValueError("v3 ledger failures are invalid")
    if ledger["status"] == "failed" and not failures:
        raise ValueError("failed v3 ledger has no failure")
    if ledger["status"] != "failed" and failures:
        raise ValueError("non-failed v3 ledger contains failures")
    if not isinstance(ledger.get("runner_wall_seconds"), (int, float)):
        raise ValueError("v3 runner wall total is invalid")
    if not isinstance(ledger.get("retained_bytes"), int) or isinstance(
        ledger.get("retained_bytes"), bool
    ):
        raise ValueError("v3 retained-byte total is invalid")


def expected_next(ledger: dict, phase: str) -> str | None:
    validate_ledger(ledger)
    if ledger["status"] != "active":
        return None
    completed = ledger["completed"]
    prerequisites = {
        "preflight": not any(completed[p] for p in ("preflight-replay", "primary", "replay")),
        "preflight-replay": completed["preflight"] == ORDER
        and not completed["primary"]
        and not completed["replay"],
        "primary": completed["preflight-replay"] == ORDER and not completed["replay"],
        "replay": completed["primary"] == ORDER,
    }
    if phase not in prerequisites or not prerequisites[phase]:
        return None
    values = completed[phase]
    return ORDER[len(values)] if len(values) < len(ORDER) else None


def validate_trace(trace: dict, state: str, sampler: str, phase: str) -> None:
    governed = phase in {"primary", "replay"}
    expected = {
        "schema_version": TRACE_SCHEMA,
        "status": "complete",
        "execution_class": (
            "governed-stage2-v3" if governed else "excluded-expansion-v3-preflight"
        ),
        "state": state,
        "year": 2020,
        "districts": STATE_CONFIG[state]["districts"],
        "sampler": sampler,
        "chains": 4 if governed else 1,
        "steps_per_chain": 2000 if governed else 25,
        "population_tolerance": 0.005,
        "base_seed": BASE_SEED,
        "snapshot_stride": 10,
    }
    for key, value in expected.items():
        if trace.get(key) != value:
            raise ValueError(f"v3 trace {key} drift")
    steps = expected["steps_per_chain"]
    snapshots = list(range(10, steps + 1, 10))
    if len(trace.get("chain_traces", [])) != expected["chains"]:
        raise ValueError("v3 trace chain-count drift")
    for index, chain in enumerate(trace["chain_traces"]):
        if chain.get("chain_index") != index:
            raise ValueError("v3 trace chain-index drift")
        if [row.get("step") for row in chain.get("metrics", [])] != list(
            range(1, steps + 1)
        ):
            raise ValueError("v3 trace metric schedule drift")
        if [row.get("step") for row in chain.get("snapshots", [])] != snapshots:
            raise ValueError("v3 trace snapshot schedule drift")
        if any(
            row.get("max_population_deviation", 1.0) > 0.005
            for row in chain["metrics"]
        ):
            raise ValueError("v3 trace population tolerance failure")


def command(executable: Path, state: str, sampler: str, phase: str, output: Path) -> list[str]:
    config = STATE_CONFIG[state]
    governed = phase in {"primary", "replay"}
    slug = config["slug"]
    return [
        str(executable),
        "--rctx", str(ROOT / f"data/2020/certified/{slug}_blocks_2020.rctx"),
        "--assignments",
        str(
            ROOT
            / "runs/nrs-v0.3/neutral-analysis/national-2020/states"
            / slug
            / "package/baseline_assignments.json"
        ),
        "--state", state,
        "--year", "2020",
        "--districts", str(config["districts"]),
        "--tolerance", "0.005",
        "--sampler", sampler,
        "--steps", "2000" if governed else "25",
        "--chains", "4" if governed else "1",
        "--base-seed", str(BASE_SEED),
        "--snapshot-stride", "10",
        "--execution-class",
        "governed-stage2-v3" if governed else "excluded-expansion-v3-preflight",
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
    with raw.open("rb") as source, destination.open("xb") as target:
        with gzip.GzipFile(filename="", mode="wb", fileobj=target, mtime=0) as compressed:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                compressed.write(chunk)


def artifact_paths(
    package: Path, state: str, sampler: str, phase: str
) -> dict[str, Path | None]:
    tag = f"{state.lower()}-{sampler}"
    governed = phase in {"primary", "replay"}
    if phase == "primary":
        final_trace = package / f"governed-{tag}.json.gz"
        runner_trace = package / f"governed-{tag}.json"
    elif phase == "replay":
        final_trace = package / f"replay-{tag}.json"
        runner_trace = final_trace
    else:
        final_trace = package / f"{phase}-{tag}.json"
        runner_trace = final_trace
    committed = (
        package / f"governed-{tag}.json.gz"
        if phase == "replay"
        else package / f"preflight-{tag}.json"
        if phase == "preflight-replay"
        else None
    )
    return {
        "final_trace": final_trace,
        "runner_trace": runner_trace,
        "committed_trace": committed,
        "resource": package / f"resource-{phase}-{tag}.json",
    }


def next_admission_path(package: Path, state: str, sampler: str, phase: str) -> Path:
    tag = f"{state.lower()}-{sampler}"
    for attempt in range(1, 10_000):
        candidate = package / f"admission-{phase}-{tag}-attempt-{attempt:02d}.json"
        if not candidate.exists():
            return candidate
    raise ValueError(f"v3 admission attempt namespace exhausted for {phase} {tag}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--state", choices=tuple(STATE_CONFIG), required=True)
    parser.add_argument("--sampler", choices=("wilson", "kruskal"), required=True)
    parser.add_argument(
        "--phase",
        choices=("preflight", "preflight-replay", "primary", "replay"),
        required=True,
    )
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument(
        "--executable",
        type=Path,
        default=EXECUTABLE,
    )
    args = parser.parse_args()
    try:
        package = require_official_package(args.package)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    if not 1 <= args.poll_ms <= 50:
        raise SystemExit("poll interval must be between 1 and 50 ms")
    package.mkdir(parents=True, exist_ok=True)
    ledger_path = package / "ledger.json"
    if ledger_path.exists():
        ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    else:
        ledger = new_ledger()
        write_json(ledger_path, ledger)
    validate_ledger(ledger)
    key = f"{args.state}:{args.sampler}"
    if expected_next(ledger, args.phase) != key:
        raise SystemExit(f"v3 schedule violation: {key} is not the next {args.phase} run")
    paths = artifact_paths(package, args.state, args.sampler, args.phase)
    for name in ("final_trace", "runner_trace", "resource"):
        if paths[name].exists():
            raise SystemExit(f"v3 {name} path already exists: {paths[name]}")
    if paths["committed_trace"] is not None and not paths["committed_trace"].is_file():
        raise SystemExit("v3 replay source trace does not exist")

    try:
        executable = require_bound_executable(args.executable)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
    admission_path = next_admission_path(
        package, args.state, args.sampler, args.phase
    )
    run_command = command(
        executable, args.state, args.sampler, args.phase, paths["runner_trace"]
    )
    measurement: dict = {}

    def monitored_run(command_args, cwd, check):
        started_at = datetime.now(timezone.utc)
        started = time.perf_counter()
        child = subprocess.Popen(command_args, cwd=cwd)
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
            if (
                args.phase in {"primary", "replay"}
                and ledger["runner_wall_seconds"] + elapsed > WALL_LIMIT_SECONDS
            ):
                failure = "cumulative runner wall ceiling exceeded"
            if failure and child.poll() is None:
                child.terminate()
            returncode = child.poll()
            if returncode is not None:
                break
            time.sleep(args.poll_ms / 1000)
        measurement.update(
            {
                "started_at_utc": started_at.isoformat(),
                "finished_at_utc": datetime.now(timezone.utc).isoformat(),
                "wall_seconds": time.perf_counter() - started,
                "sample_count": samples,
                "sampled_peak_rss_bytes": sampled_peak,
                "os_reported_peak_rss_bytes": os_peak,
                "peak_rss_bytes": max(sampled_peak, os_peak or 0),
                "failure": failure,
            }
        )
        return SimpleNamespace(returncode=returncode)

    returncode = capacity_admitted_launch(
        package=package,
        ledger_path=ledger_path,
        admission_path=admission_path,
        run_command=run_command,
        monitored_run=monitored_run,
    )
    admission = json.loads(admission_path.read_text(encoding="utf-8"))
    if not admission["process_launch_authorized"]:
        raise SystemExit("v3 host-capacity admission rejected; no runner process created")

    record = {
        "schema_version": RESOURCE_SCHEMA,
        "status": "fail" if measurement.get("failure") or returncode != 0 else "pass",
        "protocol_id": PROTOCOL_ID,
        "phase": args.phase,
        "state": args.state,
        "sampler": args.sampler,
        "poll_interval_ms": args.poll_ms,
        **measurement,
        "returncode": returncode,
        "platform": platform.platform(),
        "runner_executable_sha256": sha256(executable),
        "runner_source_sha256": sha256(RUNNER),
        "wrapper_source_sha256": sha256(WRAPPER),
        "protocol_sha256": sha256(PROTOCOL),
        "admission_record": admission_path.name,
        "admission_record_sha256": sha256(admission_path),
        "trace_disposition": "retained",
        "claim_boundary": (
            "Resource enforcement only; preflight and replay samples are excluded "
            "from statistical analysis."
        ),
    }
    if paths["runner_trace"].is_file():
        record["raw_trace_size_bytes"] = paths["runner_trace"].stat().st_size
        record["raw_trace_sha256"] = sha256(paths["runner_trace"])
    if record["status"] != "pass":
        reason = measurement.get("failure") or f"runner returned {returncode}"
        close_ledger_failure(ledger, key, args.phase, reason)
        write_json(paths["resource"], record)
        write_json(ledger_path, ledger)
        raise SystemExit(f"v3 expansion run failed: {reason}")

    try:
        trace = load_trace(paths["runner_trace"])
        validate_trace(trace, args.state, args.sampler, args.phase)
        if args.phase in {"preflight-replay", "replay"}:
            committed = load_trace(paths["committed_trace"])
            validate_trace(
                committed,
                args.state,
                args.sampler,
                "primary" if args.phase == "replay" else "preflight",
            )
            normalize_in_place(trace)
            normalize_in_place(committed)
            if trace != committed:
                raise ValueError("normalized replay mismatch")
            if record["raw_trace_size_bytes"] > SCRATCH_LIMIT_BYTES:
                raise ValueError("scratch storage ceiling exceeded")
            paths["runner_trace"].unlink()
            record["trace_disposition"] = "deleted after exact normalized comparison"
            record["normalized_trace_match"] = True
        else:
            if args.phase == "primary":
                compress_trace(paths["runner_trace"], paths["final_trace"])
                paths["runner_trace"].unlink()
                record["trace_disposition"] = "raw deleted after deterministic gzip custody"
                retained_path = paths["final_trace"]
            else:
                retained_path = paths["runner_trace"]
            retained_size = retained_path.stat().st_size
            record["retained_trace_size_bytes"] = retained_size
            record["retained_trace_sha256"] = sha256(retained_path)
            if ledger["retained_bytes"] + retained_size > RETAINED_LIMIT_BYTES:
                raise ValueError("retained storage ceiling exceeded")
            ledger["retained_bytes"] += retained_size
    except (OSError, ValueError, json.JSONDecodeError) as error:
        record["status"] = "fail"
        record["failure"] = str(error)
        close_ledger_failure(ledger, key, args.phase, str(error))
        write_json(paths["resource"], record)
        write_json(ledger_path, ledger)
        raise SystemExit(f"v3 expansion validation failed: {error}") from error

    if args.phase in {"primary", "replay"}:
        ledger["runner_wall_seconds"] += record["wall_seconds"]
    ledger["completed"][args.phase].append(key)
    if ledger["completed"]["replay"] == ORDER:
        ledger["status"] = "complete"
    write_json(paths["resource"], record)
    write_json(ledger_path, ledger)
    print(f"v3 {args.phase} {key}: PASS")


if __name__ == "__main__":
    main()
