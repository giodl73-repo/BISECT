#!/usr/bin/env python3
"""Resumable batch runner for nationwide operational recursive trees."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
INVENTORY = ROOT / "docs/experiments/nationwide-2020/inventory.json"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bisect", type=Path, required=True)
    parser.add_argument("--limit", type=int)
    parser.add_argument("--retry-failed", action="store_true")
    args = parser.parse_args()
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    ledger_path = ROOT / "docs/experiments/nationwide-2020/tree-build-ledger.json"
    try:
        prior = (
            json.loads(ledger_path.read_text(encoding="utf-8"))["results"]
            if ledger_path.is_file()
            else []
        )
    except (json.JSONDecodeError, KeyError):
        prior = []
    prior_by_state = {row["state"]: row for row in prior}
    package_root = ROOT / "data/2020/certified/operational-trees"
    if package_root.is_dir():
        for package in package_root.iterdir():
            if package.is_dir() and (package / "manifest.json").is_file():
                state = package.name.upper()
                source = next(
                    (row for row in inventory["states"] if row["state"] == state),
                    None,
                )
                if source is not None:
                    prior_by_state[state] = {
                        "state": state,
                        "districts": source["districts"],
                        "block_count": source["block_count"],
                        "status": "built",
                        "exit_code": 0,
                        "command": ["recovered-from-package"],
                        "output": "Recovered from verified package manifest.",
                    }
    prior = list(prior_by_state.values())
    failed_states = {
        row["state"] for row in prior if row["status"] == "failed"
    }
    states = [
        row
        for row in inventory["states"]
        if row["districts"] > 1
        and (args.retry_failed or row["state"] not in failed_states)
        and not (
            ROOT
            / f"data/2020/certified/operational-trees/{row['state'].lower()}/manifest.json"
        ).is_file()
    ]
    states.sort(key=lambda row: (row["block_count"], row["state"]))
    if args.limit is not None:
        states = states[: args.limit]
    results = []
    for row in states:
        code = row["state"]
        lower = code.lower()
        command = [
            "python",
            "scripts/research/build_operational_recursive_tree.py",
            "build",
            "--bisect",
            str(args.bisect.resolve()),
            "--context",
            str(
                ROOT / f"data/2020/certified/{lower}_blocks_2020.rctx"
            ),
            "--out-dir",
            str(
                ROOT / f"data/2020/certified/operational-trees/{lower}"
            ),
            "--districts",
            str(row["districts"]),
            "--root-seed",
            "1",
            "--child-seed-0",
            "2",
            "--child-seed-1",
            "3",
        ]
        completed = subprocess.run(
            command, cwd=ROOT, capture_output=True, text=True, check=False
        )
        result = {
            "state": code,
            "districts": row["districts"],
            "block_count": row["block_count"],
            "status": "built" if completed.returncode == 0 else "failed",
            "exit_code": completed.returncode,
            "command": command,
            "output": (completed.stdout + completed.stderr)[-4000:],
        }
        results.append(result)
        print(f"{code}: {result['status']}")
    merged = {row["state"]: row for row in prior + results}
    ledger = {
        "schema_version": "certified-national-tree-build-ledger-v1",
        "results": [merged[state] for state in sorted(merged)],
        "built_count": sum(row["status"] == "built" for row in merged.values()),
        "failed_count": sum(row["status"] == "failed" for row in merged.values()),
        "claim_boundary": "Resumable operational tree build ledger; national coverage verification is separate.",
    }
    ledger_path.write_text(json.dumps(ledger, indent=2) + "\n", encoding="utf-8")
    print(
        f"National tree batch: {ledger['built_count']} built, "
        f"{ledger['failed_count']} failed"
    )


if __name__ == "__main__":
    main()
