#!/usr/bin/env python3
"""Regenerate the ceremony scoreboard and exercise one transcript attack."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "research"))

from analyze_certified_ceremony_bakeoff import analyze  # noqa: E402


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def fail(message: str) -> None:
    raise SystemExit(f"verification failed: {message}")


def without_timing(value: dict) -> dict:
    normalized = json.loads(json.dumps(value))
    for cell in normalized["cells"]:
        cell["verification_seconds"] = None
    return normalized


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    parser.add_argument("--bisect", type=Path, required=True)
    args = parser.parse_args()

    package = args.package.resolve()
    binary = args.bisect.resolve()
    manifest_path = package / "input.json"
    published = read_json(package / "analysis.json")
    regenerated = analyze(manifest_path, binary)
    if without_timing(regenerated) != without_timing(published):
        fail("regenerated procedural scoreboard differs")

    manifest = read_json(manifest_path)
    ceremony_cells = [
        cell for cell in manifest["cells"]
        if cell["procedure_class"] == "certified-ceremony"
    ]
    if not ceremony_cells:
        fail("no certified-ceremony cell")
    cell = ceremony_cells[0]
    transcript_path = (package / cell["transcript"]).resolve()
    tree_path = (package / cell["tree"]).resolve()
    tampered = read_json(transcript_path)
    tampered["rounds"][0]["published_unix_seconds"] += 1

    target_temp = ROOT / "target" / "research-regeneration"
    target_temp.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=target_temp) as temp_dir:
        attack_path = Path(temp_dir) / "timestamp-tamper.json"
        attack_path.write_text(json.dumps(tampered, indent=2) + "\n", encoding="utf-8")
        completed = subprocess.run(
            [
                str(binary), "ceremony", "verify",
                "--transcript", str(attack_path),
                "--tree", str(tree_path),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode == 0:
            fail("timestamp tamper was accepted")

    print(
        "certified ceremony bakeoff verification: PASS "
        "(scoreboard regenerated; timestamp tamper rejected)"
    )


if __name__ == "__main__":
    main()
