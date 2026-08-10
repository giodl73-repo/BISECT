#!/usr/bin/env python3
"""Verify and exactly regenerate a scheduled national structure bakeoff."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "research"))

from analyze_neutral_algorithm_family_bakeoff_national import (  # noqa: E402
    ANALYSIS_VERSION,
    EXPECTED_MODES,
    PROTOCOL_ID,
    STATE_BY_CODE,
    STRUCTURES,
    canonical_assignment,
)
from run_neutral_algorithm_family_bakeoff_national import (  # noqa: E402
    MANIFEST_VERSION,
    scheduled_states,
)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def fail(message: str) -> None:
    raise SystemExit(f"verification failed: {message}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    parser.add_argument(
        "--reuse-regeneration",
        type=Path,
        help=(
            "compare an already completed independent regeneration instead of "
            "launching another native matrix"
        ),
    )
    args = parser.parse_args()
    package = args.package.resolve()
    manifest = read_json(package / "manifest.json")
    phase = manifest.get("phase")
    if manifest.get("schema_version") != MANIFEST_VERSION:
        fail("unsupported manifest schema")
    if phase not in {"pilot", "full"}:
        fail("unsupported phase")
    if manifest.get("protocol_id") != PROTOCOL_ID:
        fail("protocol mismatch")
    expected_schedule = list(scheduled_states(phase))
    if manifest.get("schedule") != expected_schedule:
        fail("schedule mismatch")
    analysis = read_json(package / "analysis.json")
    if (
        analysis.get("schema_version") != ANALYSIS_VERSION
        or analysis.get("phase") != phase
        or analysis.get("status") != manifest.get("status")
    ):
        fail("analysis schema, phase, or status")

    binary = ROOT / manifest["binary"]["path"]
    if not binary.is_file() or sha256(binary) != manifest["binary"]["sha256"]:
        fail("binary hash mismatch")
    for row in manifest["code"]:
        path = ROOT / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            fail(f"code hash mismatch for {row['path']}")
    for row in manifest["native_artifacts"]:
        path = package / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            fail(f"native artifact hash mismatch for {row['path']}")
    for relative, expected in manifest["outputs"].items():
        path = package / relative
        if not path.is_file() or sha256(path) != expected:
            fail(f"output hash mismatch for {relative}")

    for state in expected_schedule:
        slug, expected_districts = STATE_BY_CODE[state]
        for structure in STRUCTURES:
            structure_dir = package / "states" / state.lower() / "structures" / structure
            run = read_json(structure_dir / "run.json")
            if (
                run.get("state") != state
                or run.get("structure") != structure
                or run.get("requested_seed") != 0
            ):
                fail(f"{state}/{structure} run provenance")
            command = run.get("command", [])
            if state not in command or structure not in command:
                fail(f"{state}/{structure} command provenance")
            if run.get("status") != "pass":
                continue
            native_root = structure_dir / "native" / "2020" / "states" / slug
            native = read_json(native_root / "manifest.json")
            audit = read_json(native_root / "audit-certificate.json")
            if native.get("partition_mode") != EXPECTED_MODES[structure]:
                fail(f"{state}/{structure} effective mode")
            if native.get("binary_sha256") != manifest["binary"]["sha256"]:
                fail(f"{state}/{structure} native binary hash")
            if native.get("audit_result") != audit.get("result"):
                fail(f"{state}/{structure} audit-result inconsistency")
            raw = read_json(native_root / "data" / "final_assignments.json")
            canonical = canonical_assignment(raw)
            if len(set(canonical.values())) != expected_districts:
                fail(f"{state}/{structure} district count")
            if read_json(structure_dir / "canonical_assignments.json") != canonical:
                fail(f"{state}/{structure} canonical assignment")

    target_temp = ROOT / "target" / "research-regeneration"
    target_temp.mkdir(parents=True, exist_ok=True)
    temp_dir = (
        None
        if args.reuse_regeneration
        else Path(tempfile.mkdtemp(prefix="national-bakeoff-", dir=target_temp))
    )
    regeneration_passed = False
    try:
        if args.reuse_regeneration:
            regenerated = args.reuse_regeneration.resolve()
            regenerated_manifest = read_json(regenerated / "manifest.json")
            if (
                regenerated_manifest.get("schema_version") != MANIFEST_VERSION
                or regenerated_manifest.get("protocol_id") != PROTOCOL_ID
                or regenerated_manifest.get("phase") != phase
                or regenerated_manifest.get("schedule") != expected_schedule
                or regenerated_manifest.get("binary") != manifest.get("binary")
                or regenerated_manifest.get("code") != manifest.get("code")
            ):
                fail("reused regeneration provenance differs")
        else:
            regenerated = temp_dir / "package"
            command = [
                sys.executable,
                str(ROOT / "scripts" / "research" /
                    "run_neutral_algorithm_family_bakeoff_national.py"),
                "--phase", phase,
                "--output-dir", str(regenerated),
                "--binary", str(binary),
            ]
            completed = subprocess.run(command, cwd=ROOT, check=False)
            expected_returncode = 0 if manifest["status"] == "pass" else 1
            if completed.returncode != expected_returncode:
                fail(
                    f"regeneration returned {completed.returncode}, "
                    f"expected {expected_returncode}"
                )
        for relative in manifest["outputs"]:
            regenerated_path = regenerated / relative
            published_path = package / relative
            if not regenerated_path.is_file():
                fail(f"regeneration omitted {relative}")
            if regenerated_path.read_bytes() != published_path.read_bytes():
                fail(f"regenerated {relative} differs")
        if read_json(regenerated / "analysis.json") != analysis:
            fail("regenerated analysis differs semantically")
        regeneration_passed = True
    finally:
        if regeneration_passed and temp_dir is not None:
            shutil.rmtree(temp_dir)
        elif not regeneration_passed and temp_dir is not None:
            print(
                f"failed regeneration retained at {temp_dir}",
                file=sys.stderr,
            )

    print(
        f"national algorithm-family {phase} evidence verification: PASS "
        f"(experiment status: {manifest['status'].upper()})"
    )


if __name__ == "__main__":
    main()
