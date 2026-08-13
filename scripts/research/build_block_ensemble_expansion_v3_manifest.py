#!/usr/bin/env python3
"""Build the final v3 custody manifest after registered analysis closes the gate."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/research"))
from measure_block_ensemble_resources import sha256
from run_block_ensemble_expansion_v3 import PROTOCOL, PROTOCOL_ID, RUNNER, WRAPPER
from verify_block_ensemble_expansion_v3 import ANALYZER, MANIFEST_SCHEMA
from verify_block_ensemble_v3_readiness import binding_sha256


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    package = args.package.resolve()
    analysis = json.loads((package / "analysis.json").read_text(encoding="utf-8"))
    if analysis.get("gate_passed") is not False:
        raise SystemExit("refusing to close manifest without the frozen negative decision")
    verifier = ROOT / "scripts/research/verify_block_ensemble_expansion_v3.py"
    artifacts = {
        path.name: sha256(path)
        for path in sorted(package.iterdir(), key=lambda item: item.name)
        if path.is_file() and path.name != "manifest.json"
    }
    sources = {
        path.relative_to(ROOT).as_posix(): binding_sha256(path)
    for path in (PROTOCOL, RUNNER, WRAPPER, ANALYZER, Path(__file__).resolve(), verifier)
    }
    manifest = {
        "schema_version": MANIFEST_SCHEMA,
        "protocol_id": PROTOCOL_ID,
        "status": "closed-nonconverged",
        "artifacts": artifacts,
        "sources": sources,
        "decision": {
            "gate_passed": False,
            "reason": "At least one registered scalar diagnostic failed in GA for each kernel.",
            "retry_or_extension_authorized": False,
        },
        "claim_boundary": analysis["claim_boundary"],
    }
    (package / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n"
    )


if __name__ == "__main__":
    main()
