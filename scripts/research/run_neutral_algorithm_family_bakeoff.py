#!/usr/bin/env python3
"""Run the preregistered Wisconsin neutral algorithm-family proof slice."""

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

from analyze_neutral_algorithm_family_bakeoff import (  # noqa: E402
    PROTOCOL_ID,
    STRUCTURES,
    write_derived,
)


MANIFEST_VERSION = "neutral-algorithm-family-bakeoff-manifest-v1"
PROTOCOL_PATH = Path("docs/specs/2026-08-09-neutral-algorithm-family-bakeoff-protocol.md")
RUNNER_PATH = Path("scripts/research/run_neutral_algorithm_family_bakeoff.py")
ANALYZER_PATH = Path("scripts/research/analyze_neutral_algorithm_family_bakeoff.py")
VERIFIER_PATH = Path("scripts/research/verify_neutral_algorithm_family_bakeoff.py")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def command_template(structure: str) -> list[str]:
    return [
        "target/release/bisect.exe",
        "state",
        "--state", "WI",
        "--year", "2020",
        "--output-dir", f"{{PACKAGE}}/structures/{structure}/native",
        "--partition-mode", "edge-weighted",
        "--structure", structure,
        "--weights-override", "geographic",
        "--search", "single",
        "--seed", "0",
        "--manifest",
        "--force",
        "--time-partition",
    ]


def run_structure(package: Path, binary: Path, structure: str) -> None:
    structure_dir = package / "structures" / structure
    structure_dir.mkdir(parents=True, exist_ok=True)
    template = command_template(structure)
    actual = [str(binary.resolve()) if index == 0 else token for index, token in enumerate(template)]
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
    native_root = actual_output / "2020" / "states" / "wisconsin"
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


def build_manifest(package: Path, binary: Path, analysis: dict) -> None:
    code_paths = [PROTOCOL_PATH, RUNNER_PATH, ANALYZER_PATH, VERIFIER_PATH]
    deterministic_outputs = [
        Path("analysis.json"),
        Path("structure-summary.csv"),
        Path("pairwise-overlap.csv"),
        Path("README.md"),
    ] + [
        Path("structures") / structure / "canonical_assignments.json"
        for structure in STRUCTURES
        if (package / "structures" / structure / "canonical_assignments.json").is_file()
    ]
    native_artifacts = []
    for structure in STRUCTURES:
        structure_dir = package / "structures" / structure
        for path in sorted(structure_dir.rglob("*")):
            if path.is_file() and path.name != "canonical_assignments.json":
                native_artifacts.append(
                    {"path": path.relative_to(package).as_posix(), "sha256": sha256(path)}
                )
    manifest = {
        "schema_version": MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": analysis["status"],
        "binary": {
            "path": binary.resolve().relative_to(ROOT.resolve()).as_posix(),
            "sha256": sha256(binary),
        },
        "code": [
            {"path": path.as_posix(), "sha256": sha256(ROOT / path)} for path in code_paths
        ],
        "native_artifacts": native_artifacts,
        "outputs": {
            path.as_posix(): sha256(package / path) for path in deterministic_outputs
        },
        "reproduction": {
            "command": [
                sys.executable,
                RUNNER_PATH.as_posix(),
                "--output-dir", "<TEMP_OUTPUT_DIR>",
                "--binary", "target/release/bisect.exe",
                "--force",
            ],
            "semantic_native_comparison": (
                "Timestamp-bearing native packages are verified by bound hashes; "
                "regeneration compares canonical assignments and deterministic derived outputs."
            ),
        },
    }
    (package / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


def run(package: Path, binary: Path, force: bool) -> dict:
    package = package.resolve()
    if not binary.is_file():
        raise SystemExit(f"BISECT binary not found: {binary}")
    if package.exists():
        if not force:
            raise SystemExit(f"output exists (use --force): {package}")
        if package == ROOT or ROOT not in package.parents:
            raise SystemExit(f"refusing to replace unsafe output path: {package}")
        shutil.rmtree(package)
    package.mkdir(parents=True)
    for structure in STRUCTURES:
        print(f"running {structure}...", flush=True)
        run_structure(package, binary, structure)
    analysis = write_derived(package)
    build_manifest(package, binary, analysis)
    return analysis


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=ROOT / "docs/experiments/neutral-algorithm-family-bakeoff-wi-2020",
    )
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/bisect.exe")
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()
    analysis = run(args.output_dir, args.binary.resolve(), args.force)
    print(f"neutral algorithm-family bakeoff: {analysis['status'].upper()}")
    if analysis["status"] != "pass":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
