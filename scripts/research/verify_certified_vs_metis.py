#!/usr/bin/env python3
"""Verify the committed path8 certified-versus-METIS comparison."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    root = Path("docs/examples/certified-comparison/path8-root")
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    if sha256(Path(manifest["generator_path"])) != manifest["generator_sha256"]:
        raise SystemExit("certified comparison generator hash mismatch")
    for relative, expected in manifest["files"].items():
        if sha256(root / relative) != expected:
            raise SystemExit(f"certified comparison hash mismatch: {relative}")
    report = json.loads((root / "comparison.json").read_text(encoding="utf-8"))
    rows = {row["method"]: row for row in report["rows"]}
    certified = rows["certified"]
    metis = rows["metis-seed-42"]
    control = rows["deliberate-suboptimal-control"]
    if not certified["connected"] or certified["objective"] != {
        "max_population_deviation_scaled": 0,
        "total_population_deviation_scaled": 0,
        "weighted_boundary_cut": 1,
    }:
        raise SystemExit("certified comparison optimum drift")
    if metis["objective"] != certified["objective"]:
        raise SystemExit("METIS no longer matches the path8 certified objective")
    if control["objective"]["max_population_deviation_scaled"] <= 0:
        raise SystemExit("suboptimal comparison control is no longer worse")
    print("Certified vs METIS comparison verification: PASS")


if __name__ == "__main__":
    main()
