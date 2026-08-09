#!/usr/bin/env python3
"""Verify and exactly regenerate a Tier 2 geometry bakeoff slice."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


MANIFEST_VERSION = "nrs-v0.3-bakeoff-geometry-slice-manifest-v1"
ANALYSIS_VERSION = "nrs-v0.3-bakeoff-geometry-slice-analysis-v1"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fail(message: str) -> None:
    raise SystemExit(f"verification failed: {message}")


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

    for collection in ("inputs", "code"):
        for row in manifest[collection]:
            path = root / row["path"]
            if not path.is_file():
                fail(f"missing {collection[:-1]} {row['path']}")
            if sha256(path) != row["sha256"]:
                fail(f"hash mismatch for {row['path']}")
    for name, expected in manifest["outputs"].items():
        path = package / name
        if not path.is_file() or sha256(path) != expected:
            fail(f"output hash mismatch for {name}")

    reproduction = manifest["reproduction"]
    analyzer = root / "scripts/research/analyze_nrs_bakeoff_geometry_slice.py"
    with tempfile.TemporaryDirectory() as temp_dir:
        regenerated = Path(temp_dir) / "package"
        command = [
            sys.executable,
            str(analyzer),
            "--state",
            reproduction["state"],
            "--state-fips",
            reproduction["state_fips"],
            "--year",
            str(reproduction["year"]),
            "--projection",
            reproduction["projection"],
            "--nrs-assignment",
            reproduction["nrs_assignment"],
            "--block-shapefile",
            reproduction["block_shapefile"],
            "--comparator-source",
            reproduction["comparator_source"],
            "--comparator-state-column",
            reproduction["comparator_state_column"],
            "--comparator-district-column",
            reproduction["comparator_district_column"],
            "--comparator-session-column",
            reproduction["comparator_session_column"],
            "--expected-session",
            reproduction["expected_session"],
            "--display-output-dir",
            reproduction["display_output_dir"],
            "--output-dir",
            str(regenerated),
        ]
        completed = subprocess.run(command, cwd=root, check=False)
        if completed.returncode != 0:
            fail("analysis regeneration command failed")
        for name in ("analysis.json", "README.md", "manifest.json"):
            if (regenerated / name).read_bytes() != (package / name).read_bytes():
                fail(f"regenerated {name} differs")

    print("NRS v0.3 Tier 2 geometry slice verification: PASS")


if __name__ == "__main__":
    main()
