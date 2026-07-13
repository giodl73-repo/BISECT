#!/usr/bin/env python3
"""Verify certified-recursive fixture hashes and declared corpus shape."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    root = Path("docs/examples/certified-recursive")
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    declared = {row["path"] for row in manifest["files"]}
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "manifest.json"
    }
    if declared != actual:
        raise SystemExit("certified recursive fixture inventory mismatch")
    for row in manifest["files"]:
        if sha256(root / row["path"]) != row["sha256"]:
            raise SystemExit(f"certified recursive fixture hash mismatch: {row['path']}")

    tree = json.loads(
        (
            root / "path8-k4/output/certified-bisection-tree.json"
        ).read_text(encoding="utf-8")
    )
    if tree["k"] != 4 or len(tree["nodes"]) != 3 or len(tree["leaves"]) != 4:
        raise SystemExit("certified recursive positive tree drift")
    expected_cases = {
        "false-root-optimum": "split-result-mismatch",
        "leaf-universe-tamper": "leaf-mismatch",
        "missing-leaf": "leaf-set-mismatch",
        "node-order-tamper": "node-schedule-mismatch",
        "tree-id-tamper": "tree-id-mismatch",
    }
    found = {
        path.name for path in (root / "negative-corpus").iterdir() if path.is_dir()
    }
    if found != set(expected_cases):
        raise SystemExit("certified recursive negative corpus drift")
    for case, expected_error in expected_cases.items():
        expected = json.loads(
            (root / "negative-corpus" / case / "expected.json").read_text(
                encoding="utf-8"
            )
        )
        if expected["expected_error"] != expected_error:
            raise SystemExit(f"certified recursive rejection drift: {case}")
    print("Certified recursive fixture verification: PASS")


if __name__ == "__main__":
    main()
