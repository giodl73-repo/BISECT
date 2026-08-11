#!/usr/bin/env python3
"""Verify the excluded block-ensemble resource audit package."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from measure_block_ensemble_resources import calculate_budgets, normalize_in_place, normalized_sha256

RESOURCE_PROTOCOL = ROOT / "docs/specs/2026-08-10-nrs-v0.3-block-ensemble-resource-audit.md"
STAGE1_PROTOCOL = ROOT / "docs/specs/2026-08-10-nrs-v0.3-block-ensemble-gate.md"
RUNNER = ROOT / "docs/experiments/nrs-v0.3-block-ensemble-resource-audit/block_trace-stage1.rs"
STAGE1_PACKAGE = ROOT / "docs/experiments/nrs-v0.3-block-ensemble-gate"
STATE_INPUTS = {
    "RI": (
        ROOT / "data/2020/certified/ri_blocks_2020.rctx",
        ROOT / "runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/baseline_assignments.json",
    ),
    "NH": (
        ROOT / "data/2020/certified/nh_blocks_2020.rctx",
        ROOT / "runs/nrs-v0.3/neutral-analysis/national-2020/states/nh/package/baseline_assignments.json",
    ),
    "NM": (
        ROOT / "data/2020/certified/nm_blocks_2020.rctx",
        ROOT / "runs/nrs-v0.3/neutral-analysis/national-2020/states/nm/package/baseline_assignments.json",
    ),
    "GA": (
        ROOT / "data/2020/certified/ga_blocks_2020.rctx",
        ROOT / "runs/nrs-v0.3/neutral-analysis/national-2020/states/ga/package/baseline_assignments.json",
    ),
}


def fail(message: str) -> None:
    raise SystemExit(f"block ensemble resource verification failed: {message}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_measurement(record: dict, sampler: str) -> None:
    expected = {
        "schema_version": "nrs-block-ensemble-resource-measurement-v1",
        "status": "pass",
        "execution_class": "excluded-resource-replay",
        "protocol_id": "nrs-v0.3-block-ensemble-resource-audit-v1",
        "sampler": sampler,
        "normalized_trace_match": True,
        "scratch_disposition": "deleted after exact normalized comparison",
    }
    for key, value in expected.items():
        if record.get(key) != value:
            fail(f"{sampler} measurement {key} drift")
    for key in (
        "sample_count",
        "wall_seconds",
        "sampled_peak_rss_bytes",
        "peak_rss_bytes",
        "scratch_trace_size_bytes",
    ):
        if record.get(key, 0) <= 0:
            fail(f"{sampler} measurement {key} is not positive")
    if not 1 <= record.get("poll_interval_ms", 0) <= 50:
        fail(f"{sampler} polling interval drift")
    if record["maximum_population_deviation"] > 0.005:
        fail(f"{sampler} population tolerance failure")
    if not 0 <= record["acceptance_rate"] <= 1:
        fail(f"{sampler} acceptance rate is invalid")
    if record["normalized_committed_sha256"] != record["normalized_replay_sha256"]:
        fail(f"{sampler} normalized hashes differ")


def validate_input_audit(audit: dict, state: str) -> None:
    if audit.get("schema_version") != "nrs-block-ensemble-input-audit-v1":
        fail(f"{state} input-audit schema drift")
    if audit.get("status") != "pass" or audit.get("state") != state:
        fail(f"{state} input audit is not pass")
    if audit.get("year") != 2020 or audit.get("units", 0) <= 0:
        fail(f"{state} input-audit identity drift")
    if audit.get("max_population_deviation", 1) > 0.005:
        fail(f"{state} starting assignment exceeds tolerance")
    rctx, assignments = STATE_INPUTS[state]
    if sha256(rctx) != audit.get("rctx_sha256"):
        fail(f"{state} RCTX hash mismatch")
    if sha256(assignments) != audit.get("assignments_sha256"):
        fail(f"{state} assignment hash mismatch")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    package = args.package.resolve()
    manifest = load(package / "manifest.json")
    if manifest.get("schema_version") != "nrs-block-ensemble-resource-package-v1":
        fail("manifest schema drift")
    if manifest.get("status") != "complete":
        fail("manifest is not complete")
    required_artifacts = {
        "README.md",
        "commands.md",
        "deviations.md",
        "input-audit-nh.json",
        "input-audit-nm.json",
        "input-audit-ga.json",
        "resource-wilson.json",
        "resource-kruskal.json",
        "summary.json",
    }
    if set(manifest.get("artifacts", {})) != required_artifacts:
        fail("manifest artifact set drift")
    for name, expected_hash in manifest["artifacts"].items():
        path = package / name
        if not path.is_file() or sha256(path) != expected_hash:
            fail(f"artifact hash mismatch for {name}")
    for relative, expected_hash in manifest.get("sources", {}).items():
        path = ROOT / relative
        if not path.is_file() or sha256(path) != expected_hash:
            fail(f"source hash mismatch for {relative}")

    audits = {"RI": load(STAGE1_PACKAGE / "input-audit.json")}
    for state in ("NH", "NM", "GA"):
        audits[state] = load(package / f"input-audit-{state.lower()}.json")
    for state, audit in audits.items():
        validate_input_audit(audit, state)

    measurements = []
    for sampler in ("wilson", "kruskal"):
        record = load(package / f"resource-{sampler}.json")
        validate_measurement(record, sampler)
        if record["resource_protocol_sha256"] != sha256(RESOURCE_PROTOCOL):
            fail(f"{sampler} resource protocol hash mismatch")
        if record["stage1_protocol_sha256"] != sha256(STAGE1_PROTOCOL):
            fail(f"{sampler} Stage 1 protocol hash mismatch")
        if record["runner_source_sha256"] != sha256(RUNNER):
            fail(f"{sampler} runner source hash mismatch")
        committed_path = STAGE1_PACKAGE / f"governed-{sampler}.json"
        if record["committed_trace_sha256"] != sha256(committed_path):
            fail(f"{sampler} committed trace hash mismatch")
        committed = load(committed_path)
        normalize_in_place(committed)
        if record["normalized_committed_sha256"] != normalized_sha256(committed):
            fail(f"{sampler} normalized committed trace hash mismatch")
        measurements.append(record)

    summary = load(package / "summary.json")
    if summary.get("schema_version") != "nrs-block-ensemble-resource-summary-v1":
        fail("summary schema drift")
    units = {state: int(audit["units"]) for state, audit in audits.items()}
    recomputed = calculate_budgets(measurements, units)
    for key, value in recomputed.items():
        if summary.get(key) != value:
            fail(f"summary formula drift for {key}")
    if not summary.get("expansion_protocol_draft_eligible"):
        fail("summary does not authorize protocol drafting")
    print("block ensemble resource audit verification: PASS")


if __name__ == "__main__":
    main()
