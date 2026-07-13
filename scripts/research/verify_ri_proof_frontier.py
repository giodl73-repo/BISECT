#!/usr/bin/env python3
"""Verify committed Rhode Island partial proof custody."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check-local", action="store_true")
    args = parser.parse_args()
    package = ROOT / "docs/examples/ri-proof-frontier"
    manifest = json.loads((package / "manifest.json").read_text(encoding="utf-8"))
    for relative, expected in manifest["files"].items():
        if sha256(package / relative) != expected:
            raise SystemExit(f"RI proof frontier hash mismatch: {relative}")
    provenance = json.loads((package / "provenance.json").read_text(encoding="utf-8"))
    population = provenance["population_stage"]
    if population["status"] != "verified-unsat":
        raise SystemExit("RI population proof posture drift")
    if population["proof_sha256"] != sha256(package / population["proof_path"]):
        raise SystemExit("RI population proof hash mismatch")
    if provenance["boundary_stage"]["status"] != "not-run":
        raise SystemExit("RI boundary proof posture drift")
    if provenance["prior_boundary_attempt"]["status"] != "timelimit":
        raise SystemExit("RI prior boundary attempt posture drift")
    if provenance["compiler"]["sha256"] != sha256(ROOT / provenance["compiler"]["path"]):
        raise SystemExit("RI proof compiler identity mismatch")
    branch = json.loads((package / "elite2-branch.json").read_text(encoding="utf-8"))
    if branch["status"] != "verified-unsat-branch-only":
        raise SystemExit("RI reduced branch proof posture drift")
    if branch["toolchain"]["result"] != "VERIFIED UNSATISFIABLE":
        raise SystemExit("RI reduced branch verifier result drift")
    if args.check_local:
        input_path = ROOT / population["input_path"]
        if not input_path.is_file() or sha256(input_path) != population["input_sha256"]:
            raise SystemExit("RI population OPB custody mismatch")
        branch_input = ROOT / branch["input"]["path"]
        if (
            not branch_input.is_file()
            or sha256(branch_input) != branch["input"]["sha256"]
        ):
            raise SystemExit("RI reduced branch OPB custody mismatch")
        if branch["proof"].get("local_available", True):
            branch_proof = ROOT / branch["proof"]["path"]
            if (
                not branch_proof.is_file()
                or sha256(branch_proof) != branch["proof"]["compressed_sha256"]
            ):
                raise SystemExit("RI reduced branch proof custody mismatch")
    print("RI proof frontier verification: PASS")


if __name__ == "__main__":
    main()
