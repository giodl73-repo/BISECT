#!/usr/bin/env python3
"""Run the frozen national neutral algorithm-family bakeoff schedule."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "research"))

from analyze_neutral_algorithm_family_bakeoff_national import (  # noqa: E402
    FULL_STATES,
    PILOT_STATES,
    PROTOCOL_ID,
    STATE_BY_CODE,
    STRUCTURES,
    write_derived,
)


MANIFEST_VERSION = "neutral-algorithm-family-national-bakeoff-manifest-v1"
PROTOCOL_PATH = Path(
    "docs/specs/2026-08-09-neutral-algorithm-family-national-bakeoff-protocol.md"
)
RUNNER_PATH = Path("scripts/research/run_neutral_algorithm_family_bakeoff_national.py")
ANALYZER_PATH = Path(
    "scripts/research/analyze_neutral_algorithm_family_bakeoff_national.py"
)
VERIFIER_PATH = Path(
    "scripts/research/verify_neutral_algorithm_family_bakeoff_national.py"
)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def scheduled_states(phase: str) -> tuple[str, ...]:
    return PILOT_STATES if phase == "pilot" else FULL_STATES


def command_template(state: str, structure: str) -> list[str]:
    return [
        "target/release/bisect.exe",
        "state",
        "--state", state,
        "--year", "2020",
        "--output-dir", f"{{PACKAGE}}/states/{state.lower()}/structures/{structure}/native",
        "--partition-mode", "edge-weighted",
        "--structure", structure,
        "--weights-override", "geographic",
        "--search", "single",
        "--seed", "0",
        "--manifest",
        "--force",
        "--time-partition",
    ]


def run_cell(package: Path, binary: Path, state: str, structure: str) -> None:
    structure_dir = package / "states" / state.lower() / "structures" / structure
    if structure_dir.exists():
        shutil.rmtree(structure_dir)
    structure_dir.mkdir(parents=True, exist_ok=True)
    template = command_template(state, structure)
    actual = [str(binary.resolve()) if index == 0 else token
              for index, token in enumerate(template)]
    actual_output = structure_dir / "native"
    actual[actual.index("--output-dir") + 1] = str(actual_output.resolve())
    completed = subprocess.run(
        actual,
        cwd=ROOT,
        check=False,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
    )
    (structure_dir / "stdout.txt").write_text(completed.stdout, encoding="utf-8")
    (structure_dir / "stderr.txt").write_text(completed.stderr, encoding="utf-8")
    state_slug, _seats = STATE_BY_CODE[state]
    native_root = actual_output / "2020" / "states" / state_slug
    required = [
        native_root / "manifest.json",
        native_root / "data" / "final_assignments.json",
        native_root / "audit-certificate.json",
        native_root / "plan.rplan",
        native_root / "context.rctx",
    ]
    missing = [str(path.relative_to(package)) for path in required if not path.is_file()]
    status = "pass" if completed.returncode == 0 and not missing else "fail"
    run = {
        "protocol_id": PROTOCOL_ID,
        "state": state,
        "structure": structure,
        "status": status,
        "requested_seed": 0,
        "command": template,
        "returncode": completed.returncode,
    }
    if missing:
        run["error"] = f"missing required native artifacts: {missing}"
    elif completed.returncode != 0:
        run["error"] = "BISECT invocation returned nonzero; see stdout.txt and stderr.txt"
    (structure_dir / "run.json").write_text(
        json.dumps(run, indent=2) + "\n", encoding="utf-8"
    )


def deterministic_output_paths(package: Path, phase: str) -> list[Path]:
    paths = [
        Path("analysis.json"), Path("state-summary.csv"), Path("cell-summary.csv"),
        Path("pairwise-overlap.csv"), Path("README.md"),
    ]
    for state in scheduled_states(phase):
        state_root = Path("states") / state.lower()
        paths.append(state_root / "analysis.json")
        for structure in STRUCTURES:
            candidate = state_root / "structures" / structure / "canonical_assignments.json"
            if (package / candidate).is_file():
                paths.append(candidate)
    return paths


def build_manifest(package: Path, binary: Path, phase: str, analysis: dict) -> None:
    code_paths = [PROTOCOL_PATH, RUNNER_PATH, ANALYZER_PATH, VERIFIER_PATH]
    deterministic = deterministic_output_paths(package, phase)
    deterministic_set = {path.as_posix() for path in deterministic}
    native_artifacts = []
    for path in sorted(package.rglob("*")):
        relative = path.relative_to(package).as_posix()
        if path.is_file() and relative not in deterministic_set and relative != "manifest.json":
            native_artifacts.append({"path": relative, "sha256": sha256(path)})
    manifest = {
        "schema_version": MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "phase": phase,
        "status": analysis["status"],
        "schedule": list(scheduled_states(phase)),
        "binary": {
            "path": binary.resolve().relative_to(ROOT.resolve()).as_posix(),
            "sha256": sha256(binary),
        },
        "code": [
            {"path": path.as_posix(), "sha256": sha256(ROOT / path)}
            for path in code_paths
        ],
        "native_artifacts": native_artifacts,
        "outputs": {path.as_posix(): sha256(package / path) for path in deterministic},
        "reproduction": {
            "command": [
                sys.executable, RUNNER_PATH.as_posix(),
                "--phase", phase,
                "--output-dir", "<TEMP_OUTPUT_DIR>",
                "--binary", "target/release/bisect.exe",
                "--force",
            ],
            "semantic_native_comparison": (
                "Timestamp-bearing native artifacts are hash-bound in the published "
                "package; regeneration compares deterministic derived outputs."
            ),
        },
    }
    (package / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


def run(package: Path, binary: Path, phase: str, force: bool, resume: bool = False) -> dict:
    package = package.resolve()
    if not binary.is_file():
        raise SystemExit(f"BISECT binary not found: {binary}")
    if package.exists():
        if force and resume:
            raise SystemExit("--force and --resume are mutually exclusive")
        if not force and not resume:
            raise SystemExit(f"output exists (use --force or --resume): {package}")
        if force:
            if package == ROOT or ROOT not in package.parents:
                raise SystemExit(f"refusing to replace unsafe output path: {package}")
            shutil.rmtree(package)
    package.mkdir(parents=True, exist_ok=True)
    for state in scheduled_states(phase):
        for structure in STRUCTURES:
            run_path = (
                package / "states" / state.lower() / "structures" / structure / "run.json"
            )
            if resume and run_path.is_file():
                print(f"preserving completed {state}/{structure}...", flush=True)
                continue
            print(f"running {state}/{structure}...", flush=True)
            run_cell(package, binary, state, structure)
    analysis = write_derived(package, phase)
    build_manifest(package, binary, phase, analysis)
    return analysis


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--phase", choices=("pilot", "full"), default="pilot")
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/bisect.exe")
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--resume", action="store_true")
    args = parser.parse_args()
    output = args.output_dir or (
        ROOT / f"docs/experiments/neutral-algorithm-family-bakeoff-{args.phase}-2020"
    )
    analysis = run(output, args.binary.resolve(), args.phase, args.force, args.resume)
    print(f"national algorithm-family {args.phase}: {analysis['status'].upper()}")
    if analysis["status"] != "pass":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
