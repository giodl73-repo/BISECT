#!/usr/bin/env python3
"""Validate ceremony bakeoff cells and emit the procedural scoreboard."""

from __future__ import annotations

import argparse
import json
import subprocess
import time
from pathlib import Path


INPUT_SCHEMA = "certified-ceremony-bakeoff-input-v1"
OUTPUT_SCHEMA = "certified-ceremony-bakeoff-analysis-v1"
PROTOCOL_ID = "certified-ceremony-bakeoff-extension-v1"
PROCEDURE_CLASSES = {
    "replay-only",
    "audit-package",
    "certified-tree",
    "certified-ceremony",
}


class CeremonyBakeoffError(RuntimeError):
    pass


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def resolve_artifact(base: Path, value: str, field: str) -> Path:
    path = (base / value).resolve()
    if not path.is_file():
        raise CeremonyBakeoffError(f"{field} does not exist: {path}")
    return path


def unsupported_row(cell: dict) -> dict:
    districts = int(cell["districts"])
    if districts < 2:
        raise CeremonyBakeoffError("bakeoff cells require at least two districts")
    return {
        "state": cell["state"],
        "arm": cell["arm"],
        "procedure_class": cell["procedure_class"],
        "internal_cut_count": districts - 1,
        "certified_cut_count": 0,
        "proof_coverage": 0.0,
        "released_round_count": 0,
        "prefix_verification": False,
        "final_tree_binding": False,
        "external_witnesses_validated": False,
        "review_seconds": None,
        "status": "unsupported",
        "halt_reason": None,
        "transcript_bytes": None,
        "verification_seconds": None,
        "verification_exit_code": None,
        "tamper_suite": cell.get("tamper_suite", "not-run"),
    }


def ceremony_row(cell: dict, base: Path, bisect: Path) -> dict:
    transcript_path = resolve_artifact(base, cell["transcript"], "transcript")
    transcript = read_json(transcript_path)
    config = transcript.get("config", {})
    districts = int(cell["districts"])
    if config.get("district_count") != districts:
        raise CeremonyBakeoffError(
            f"{cell['state']}/{cell['arm']}: transcript district count mismatch"
        )
    status = transcript.get("status")
    command = [
        str(bisect),
        "ceremony",
        "verify",
        "--transcript",
        str(transcript_path),
    ]
    tree_value = cell.get("tree")
    if tree_value:
        tree_path = resolve_artifact(base, tree_value, "tree")
        command.extend(["--tree", str(tree_path)])
    started = time.perf_counter()
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    elapsed = time.perf_counter() - started
    verified = completed.returncode == 0
    if not verified:
        detail = completed.stderr.strip() or completed.stdout.strip()
        raise CeremonyBakeoffError(
            f"{cell['state']}/{cell['arm']}: ceremony verification failed: {detail}"
        )
    cuts = sum(len(round_record.get("cuts", [])) for round_record in transcript.get("rounds", []))
    required = districts - 1
    if cuts > required:
        raise CeremonyBakeoffError(f"{cell['state']}/{cell['arm']}: excess certified cuts")
    if status == "completed" and cuts != required:
        raise CeremonyBakeoffError(
            f"{cell['state']}/{cell['arm']}: completed transcript lacks k-1 cuts"
        )
    return {
        "state": cell["state"],
        "arm": cell["arm"],
        "procedure_class": cell["procedure_class"],
        "internal_cut_count": required,
        "certified_cut_count": cuts,
        "proof_coverage": cuts / required,
        "released_round_count": len(transcript.get("rounds", [])),
        "prefix_verification": verified,
        "final_tree_binding": status == "completed" and bool(tree_value) and verified,
        "external_witnesses_validated": bool(cell.get("external_witnesses_validated", False)),
        "review_seconds": config.get("minimum_review_seconds"),
        "status": status,
        "halt_reason": transcript.get("halt_reason"),
        "transcript_bytes": transcript_path.stat().st_size,
        "verification_seconds": round(elapsed, 6),
        "verification_exit_code": completed.returncode,
        "tamper_suite": cell.get("tamper_suite", "not-run"),
    }


def analyze(manifest_path: Path, bisect: Path) -> dict:
    manifest = read_json(manifest_path)
    if manifest.get("schema_version") != INPUT_SCHEMA:
        raise CeremonyBakeoffError("unsupported input schema")
    if manifest.get("protocol_id") != PROTOCOL_ID:
        raise CeremonyBakeoffError("protocol mismatch")
    if not bisect.is_file():
        raise CeremonyBakeoffError(f"bisect binary does not exist: {bisect}")
    cells = manifest.get("cells")
    if not isinstance(cells, list) or not cells:
        raise CeremonyBakeoffError("manifest must contain at least one cell")
    seen: set[tuple[str, str]] = set()
    rows = []
    for cell in cells:
        identity = (cell.get("state"), cell.get("arm"))
        if not all(isinstance(value, str) and value for value in identity):
            raise CeremonyBakeoffError("every cell requires nonempty state and arm")
        if identity in seen:
            raise CeremonyBakeoffError(f"duplicate cell: {identity}")
        seen.add(identity)
        procedure_class = cell.get("procedure_class")
        if procedure_class not in PROCEDURE_CLASSES:
            raise CeremonyBakeoffError(f"unsupported procedure class: {procedure_class}")
        if procedure_class == "certified-ceremony":
            if not cell.get("transcript"):
                raise CeremonyBakeoffError(f"{identity}: ceremony transcript required")
            rows.append(ceremony_row(cell, manifest_path.parent, bisect))
        else:
            if cell.get("transcript"):
                raise CeremonyBakeoffError(f"{identity}: transcript contradicts procedure class")
            rows.append(unsupported_row(cell))
    rows.sort(key=lambda row: (row["state"], row["arm"]))
    return {
        "schema_version": OUTPUT_SCHEMA,
        "protocol_id": PROTOCOL_ID,
        "parent_protocol_id": manifest.get("parent_protocol_id"),
        "claim_boundary": (
            "Procedural evidence is reported separately from map outcomes. "
            "This fixture does not establish fairness, legal validity, public "
            "legitimacy, external timestamp validity, or a generally best map."
        ),
        "cells": rows,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--bisect", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    result = analyze(args.manifest.resolve(), args.bisect.resolve())
    args.out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
