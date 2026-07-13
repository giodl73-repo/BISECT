#!/usr/bin/env python3
"""Verify committed proof-toolchain smoke custody."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    root = Path("docs/examples/proof-toolchain-smoke")
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    for relative, expected in manifest["files"].items():
        if sha256(root / relative) != expected:
            raise SystemExit(f"proof-toolchain smoke hash mismatch: {relative}")
    provenance = json.loads((root / "provenance.json").read_text(encoding="utf-8"))
    if provenance["status"] != "verified":
        raise SystemExit("proof-toolchain smoke status drift")
    if provenance["input"]["sha256"] != sha256(root / "population.opb"):
        raise SystemExit("proof-toolchain input custody mismatch")
    if provenance["proof"]["sha256"] != sha256(root / "population.pbp"):
        raise SystemExit("proof-toolchain proof custody mismatch")
    current_population = Path(
        "docs/examples/certified-proof-backend/path8-root/optimal/01-population.opb"
    )
    if sha256(current_population) != provenance["input"]["sha256"]:
        raise SystemExit("proof-toolchain population input no longer matches compiler output")
    compact = provenance["compact_connectivity_proof"]
    if compact["input_sha256"] != sha256(root / compact["input_path"]):
        raise SystemExit("compact connectivity input custody mismatch")
    if compact["proof_sha256"] != sha256(root / compact["proof_path"]):
        raise SystemExit("compact connectivity proof custody mismatch")
    current_compact = Path(
        "docs/examples/certified-proof-backend/path8-root/compact-optimal/02-boundary.opb"
    )
    if sha256(current_compact) != compact["input_sha256"]:
        raise SystemExit("compact proof input no longer matches compiler output")
    if compact["result"] != "Verification succeeded.":
        raise SystemExit("compact connectivity proof result drift")
    if provenance["roundingsat"]["result"] != "UNSATISFIABLE":
        raise SystemExit("RoundingSat smoke result drift")
    if provenance["veripb"]["result"] != "Verification succeeded.":
        raise SystemExit("VeriPB smoke result drift")
    print("Proof toolchain smoke verification: PASS")


if __name__ == "__main__":
    main()
