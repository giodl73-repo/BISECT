#!/usr/bin/env python3
"""Build hostile fixtures and the root manifest for single-district certificates."""

from __future__ import annotations

import hashlib
import json
from copy import deepcopy
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / "docs/examples/certified-single-district"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    positive = PACKAGE / "grid3x3"
    instance = json.loads(
        (positive / "single-district-instance.json").read_text(encoding="utf-8")
    )
    certificate = json.loads(
        (positive / "single-district-certificate.json").read_text(encoding="utf-8")
    )
    negative = PACKAGE / "negative"
    negative.mkdir(parents=True, exist_ok=True)

    omitted = deepcopy(certificate)
    omitted["assignment"].pop()
    (negative / "omitted-assignment.json").write_text(
        json.dumps(omitted, indent=2) + "\n", encoding="utf-8"
    )

    nonzero = deepcopy(certificate)
    nonzero["assignment"][4] = 1
    (negative / "nonzero-assignment.json").write_text(
        json.dumps(nonzero, indent=2) + "\n", encoding="utf-8"
    )

    disconnected = deepcopy(instance)
    disconnected["edges"] = [
        edge
        for edge in disconnected["edges"]
        if edge["left"] < 6 and edge["right"] < 6
    ]
    (negative / "disconnected-instance.json").write_text(
        json.dumps(disconnected, indent=2) + "\n", encoding="utf-8"
    )

    files = {}
    for path in sorted(PACKAGE.rglob("*")):
        if path.is_file() and path.name != "manifest.json":
            files[path.relative_to(PACKAGE).as_posix()] = sha256(path)
    manifest = {
        "schema_version": "certified-single-district-fixture-package-v1",
        "package_id": "certified-single-district-grid3x3",
        "status": "verified",
        "files": files,
        "verifier_path": "scripts/research/verify_certified_single_fixtures.py",
        "verification_commands": [
            "python scripts/research/verify_certified_single_fixtures.py",
            "cargo run -p bisect-ilp --example certified_single_district -- verify-package docs/examples/certified-single-district/grid3x3",
        ],
        "claim_boundary": "Synthetic wall-to-wall one-district certificate and hostile verifier corpus.",
    }
    (PACKAGE / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    print("Certified single-district fixtures: BUILT")


if __name__ == "__main__":
    main()
