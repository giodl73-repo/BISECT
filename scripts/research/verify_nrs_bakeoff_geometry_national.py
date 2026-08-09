#!/usr/bin/env python3
"""Verify and exactly regenerate the national Tier 2 geometry package."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


MANIFEST_VERSION = "nrs-v0.3-national-bakeoff-geometry-manifest-v1"
ANALYSIS_VERSION = "nrs-v0.3-national-bakeoff-geometry-analysis-v1"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fail(message: str) -> None:
    raise SystemExit(f"verification failed: {message}")


def package_files(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    root = Path.cwd().resolve()
    package = args.package.resolve()
    manifest = json.loads((package / "manifest.json").read_text(encoding="utf-8"))
    analysis = json.loads((package / "analysis.json").read_text(encoding="utf-8"))
    if manifest.get("schema_version") != MANIFEST_VERSION or manifest.get("status") != "pass":
        fail("manifest schema or status")
    if analysis.get("schema_version") != ANALYSIS_VERSION or analysis.get("status") != "pass":
        fail("analysis schema or status")
    if manifest["reproduction"].get("workers") != 1:
        fail("reproduction worker count is not one")

    for row in manifest["code"]:
        path = root / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            fail(f"code hash mismatch for {row['path']}")
    for row in manifest["state_package_files"]:
        path = package / row["path"]
        if not path.is_file() or sha256(path) != row["sha256"]:
            fail(f"state package hash mismatch for {row['path']}")
    for name, expected in manifest["outputs"].items():
        path = package / name
        if not path.is_file() or sha256(path) != expected:
            fail(f"output hash mismatch for {name}")

    runner = root / "scripts/research/run_nrs_bakeoff_geometry_national.py"
    with tempfile.TemporaryDirectory() as temp_dir:
        regenerated = Path(temp_dir) / "package"
        completed = subprocess.run(
            [
                sys.executable,
                str(runner),
                "--nrs-root",
                manifest["reproduction"]["nrs_root"],
                "--output-dir",
                str(regenerated),
                "--display-output-dir",
                manifest["reproduction"]["output_dir"],
            ],
            cwd=root,
            check=False,
        )
        if completed.returncode != 0:
            fail("national regeneration command failed")
        expected_files = package_files(package)
        actual_files = package_files(regenerated)
        if actual_files.keys() != expected_files.keys():
            fail("regenerated package file set differs")
        for name, expected in expected_files.items():
            if actual_files[name] != expected:
                fail(f"regenerated {name} differs")

    print("NRS v0.3 national Tier 2 geometry verification: PASS")


if __name__ == "__main__":
    main()
