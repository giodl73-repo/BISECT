#!/usr/bin/env python3
"""Verify Stage 0 and governed Stage 1 block-ensemble gate evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PROTOCOL = ROOT / "docs/specs/2026-08-10-nrs-v0.3-block-ensemble-gate.md"
ADAPTER = ROOT / "crates/bisect-ensemble/src/block_input.rs"
VALIDATOR = ROOT / "crates/bisect-ensemble/examples/validate_block_input.rs"
RUNNER = ROOT / "crates/bisect-ensemble/examples/block_trace.rs"
ANALYZER = ROOT / "scripts/research/analyze_block_ensemble.py"
RCTX = ROOT / "data/2020/certified/ri_blocks_2020.rctx"
ASSIGNMENTS = ROOT / (
    "runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/"
    "baseline_assignments.json"
)


def fail(message: str) -> None:
    raise SystemExit(f"block ensemble gate verification failed: {message}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def normalized(trace: dict) -> dict:
    result = json.loads(json.dumps(trace))
    for chain in result["chain_traces"]:
        for metric in chain["metrics"]:
            metric["runtime_ms"] = 0.0
    return result


def verify_trace(trace: dict, sampler: str, governed: bool = False) -> None:
    if trace.get("schema_version") != "nrs-block-ensemble-trace-v1":
        fail(f"{sampler} trace schema drift")
    expected = {
        "status": "complete",
        "execution_class": "governed-stage1" if governed else "excluded-engineering-preflight",
        "state": "RI",
        "year": 2020,
        "units": 25649,
        "districts": 2,
        "sampler": sampler,
        "chains": 4 if governed else 1,
        "steps_per_chain": 2000 if governed else 25,
        "population_tolerance": 0.005,
        "base_seed": 20260810,
        "snapshot_stride": 10,
    }
    for key, value in expected.items():
        if trace.get(key) != value:
            fail(f"{sampler} trace {key} drift")
    chain_count = 4 if governed else 1
    steps = 2000 if governed else 25
    snapshots = list(range(10, steps + 1, 10))
    if len(trace["chain_traces"]) != chain_count:
        fail(f"{sampler} trace chain-count drift")
    if [chain["chain_index"] for chain in trace["chain_traces"]] != list(range(chain_count)):
        fail(f"{sampler} trace chain-index drift")
    if len({chain["seed"] for chain in trace["chain_traces"]}) != chain_count:
        fail(f"{sampler} trace chain-seed drift")
    for chain in trace["chain_traces"]:
        metrics = chain["metrics"]
        if [row["step"] for row in metrics] != list(range(1, steps + 1)):
            fail(f"{sampler} metric step sequence drift")
        if [row["step"] for row in chain["snapshots"]] != snapshots:
            fail(f"{sampler} snapshot schedule drift")
        if any(len(row["assignment"]) != 25649 for row in chain["snapshots"]):
            fail(f"{sampler} snapshot universe drift")
        if any(row["max_population_deviation"] > 0.005 for row in metrics):
            fail(f"{sampler} population tolerance failure")
        if any(row["runtime_ms"] < 0 for row in metrics):
            fail(f"{sampler} invalid runtime")


def command(executable: Path, sampler: str, output: Path, governed: bool = False) -> list[str]:
    return [
        str(executable),
        "--rctx", str(RCTX),
        "--assignments", str(ASSIGNMENTS),
        "--state", "RI",
        "--year", "2020",
        "--districts", "2",
        "--tolerance", "0.005",
        "--sampler", sampler,
        "--steps", "2000" if governed else "25",
        "--chains", "4" if governed else "1",
        "--base-seed", "20260810",
        "--snapshot-stride", "10",
        "--execution-class", "governed-stage1" if governed else "excluded-engineering-preflight",
        "--output", str(output),
    ]


def verify_manifest(package: Path) -> dict:
    manifest = load(package / "manifest.json")
    if manifest.get("schema_version") != "nrs-block-ensemble-package-v1":
        fail("Stage 1 manifest schema drift")
    if manifest.get("status") != "complete":
        fail("Stage 1 manifest is not complete")
    required = {
        "governed-wilson.json",
        "governed-kruskal.json",
        "analysis.json",
        "summary.csv",
        "README.md",
        "commands.md",
        "deviations.md",
    }
    entries = manifest.get("artifacts", {})
    if set(entries) != required:
        fail("Stage 1 manifest artifact set drift")
    for name, expected in entries.items():
        path = package / name
        if not path.is_file() or sha256(path) != expected:
            fail(f"Stage 1 hash mismatch for {name}")
    source_entries = {
        PROTOCOL.relative_to(ROOT).as_posix(): PROTOCOL,
        ADAPTER.relative_to(ROOT).as_posix(): ADAPTER,
        VALIDATOR.relative_to(ROOT).as_posix(): VALIDATOR,
        RUNNER.relative_to(ROOT).as_posix(): RUNNER,
        ANALYZER.relative_to(ROOT).as_posix(): ANALYZER,
        Path(__file__).resolve().relative_to(ROOT).as_posix(): Path(__file__).resolve(),
        RCTX.relative_to(ROOT).as_posix(): RCTX,
        ASSIGNMENTS.relative_to(ROOT).as_posix(): ASSIGNMENTS,
    }
    if set(manifest.get("sources", {})) != set(source_entries):
        fail("Stage 1 manifest source set drift")
    for name, path in source_entries.items():
        if sha256(path) != manifest["sources"][name]:
            fail(f"Stage 1 source hash mismatch for {name}")
    return manifest


def verify_analysis(package: Path, temp: Path) -> None:
    recomputed_json = temp / "analysis.json"
    recomputed_csv = temp / "summary.csv"
    completed = subprocess.run(
        [
            sys.executable,
            str(ANALYZER),
            "--wilson", str(package / "governed-wilson.json"),
            "--kruskal", str(package / "governed-kruskal.json"),
            "--burn-in", "500",
            "--output", str(recomputed_json),
            "--summary-csv", str(recomputed_csv),
        ],
        cwd=ROOT,
    )
    if completed.returncode != 0:
        fail(f"analysis replay returned {completed.returncode}")
    if recomputed_json.read_bytes() != (package / "analysis.json").read_bytes():
        fail("recomputed analysis mismatch")
    if recomputed_csv.read_bytes() != (package / "summary.csv").read_bytes():
        fail("recomputed summary mismatch")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    parser.add_argument(
        "--executable",
        type=Path,
        default=ROOT / "target/release/examples/block_trace.exe",
    )
    parser.add_argument(
        "--stage1",
        action="store_true",
        help="verify governed artifacts and perform the full sequential deterministic replay",
    )
    args = parser.parse_args()
    package = args.package.resolve()
    executable = args.executable.resolve()
    audit = load(package / "input-audit.json")
    summary = load(package / "preflight-summary.json")
    if audit.get("status") != "pass" or summary.get("status") != "pass":
        fail("Stage 0 evidence is not marked pass")
    hash_checks = {
        PROTOCOL: audit["protocol_sha256"],
        ADAPTER: audit["adapter_sha256"],
        VALIDATOR: audit["validator_sha256"],
        RCTX: audit["rctx_sha256"],
        ASSIGNMENTS: audit["assignments_sha256"],
        RUNNER: summary["runner_source_sha256"],
    }
    for path, expected in hash_checks.items():
        if not path.is_file() or sha256(path) != expected:
            fail(f"hash mismatch for {path.relative_to(ROOT)}")
    if not executable.is_file():
        fail(f"missing runner executable {executable}")
    if sha256(executable) != summary["runner_executable_sha256"]:
        fail("runner executable hash mismatch")

    committed = {}
    for sampler in ("wilson", "kruskal"):
        path = package / f"preflight-{sampler}.json"
        expected_hash = summary["kernels"][sampler]["trace_sha256"]
        if sha256(path) != expected_hash:
            fail(f"hash mismatch for {path.name}")
        trace = load(path)
        verify_trace(trace, sampler)
        committed[sampler] = trace

    with tempfile.TemporaryDirectory(prefix="block-ensemble-preflight-") as temp_dir:
        temp = Path(temp_dir)
        for sampler in ("wilson", "kruskal"):
            replay = temp / f"{sampler}.json"
            completed = subprocess.run(command(executable, sampler, replay), cwd=ROOT)
            if completed.returncode != 0:
                fail(f"{sampler} replay returned {completed.returncode}")
            replay_trace = load(replay)
            verify_trace(replay_trace, sampler)
            if normalized(replay_trace) != normalized(committed[sampler]):
                fail(f"{sampler} normalized replay mismatch")

    print("block ensemble Stage 0 verification: PASS")

    if not args.stage1:
        return

    verify_manifest(package)
    governed = {}
    for sampler in ("wilson", "kruskal"):
        trace = load(package / f"governed-{sampler}.json")
        verify_trace(trace, sampler, governed=True)
        governed[sampler] = trace

    with tempfile.TemporaryDirectory(prefix="block-ensemble-stage1-") as temp_dir:
        temp = Path(temp_dir)
        verify_analysis(package, temp)
        for sampler in ("wilson", "kruskal"):
            print(f"block ensemble Stage 1 replay: starting {sampler}", flush=True)
            replay = temp / f"governed-{sampler}.json"
            completed = subprocess.run(
                command(executable, sampler, replay, governed=True), cwd=ROOT
            )
            if completed.returncode != 0:
                fail(f"governed {sampler} replay returned {completed.returncode}")
            replay_trace = load(replay)
            verify_trace(replay_trace, sampler, governed=True)
            if normalized(replay_trace) != normalized(governed[sampler]):
                fail(f"governed {sampler} normalized replay mismatch")
            print(f"block ensemble Stage 1 replay: {sampler} exact normalized PASS", flush=True)

    print("block ensemble governed Stage 1 verification: PASS")


if __name__ == "__main__":
    main()
