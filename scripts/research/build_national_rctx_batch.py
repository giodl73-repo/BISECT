#!/usr/bin/env python3
"""Resumable size-ordered batch builder for nationwide 2020 block RCTX files."""

from __future__ import annotations

import argparse
import json
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
INVENTORY = ROOT / "docs/experiments/nationwide-2020/inventory.json"


def build_state(row: dict) -> dict:
    code = row["state"]
    lower = code.lower()
    report = Path(f"docs/experiments/nationwide-2020/rctx/{lower}.json")
    manifest = Path(f"docs/experiments/nationwide-2020/rctx/{lower}-manifest.json")
    command = [
        "python",
        "scripts/research/build_state_block_rctx.py",
        "--state-code",
        code,
        "--state-fips",
        row["fips"],
        "--state-name",
        row["name"].lower().replace(" ", "_"),
        "--rctx",
        f"data/2020/certified/{lower}_blocks_2020.rctx",
        "--report",
        str(report),
        "--manifest",
        str(manifest),
    ]
    completed = subprocess.run(
        command, cwd=ROOT, capture_output=True, text=True, check=False
    )
    return {
        "state": code,
        "block_count": row["block_count"],
        "status": "built" if completed.returncode == 0 else "failed",
        "exit_code": completed.returncode,
        "command": command,
        "output": (completed.stdout + completed.stderr)[-4000:],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workers", type=int, default=2)
    parser.add_argument("--limit", type=int)
    args = parser.parse_args()
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    rows_by_state = {row["state"]: row for row in inventory["states"]}
    pending = [
        rows_by_state[state]
        for state in inventory["batch_order"]
        if not (ROOT / f"data/2020/certified/{state.lower()}_blocks_2020.rctx").is_file()
    ]
    if args.limit is not None:
        pending = pending[: args.limit]
    ledger_path = ROOT / "docs/experiments/nationwide-2020/rctx-build-ledger.json"
    prior = (
        json.loads(ledger_path.read_text(encoding="utf-8"))["results"]
        if ledger_path.is_file()
        else []
    )
    results = []
    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {executor.submit(build_state, row): row["state"] for row in pending}
        for future in as_completed(futures):
            result = future.result()
            results.append(result)
            print(f"{result['state']}: {result['status']}")
    merged = {
        result["state"]: result for result in prior + results
    }
    ledger = {
        "schema_version": "certified-national-rctx-build-ledger-v1",
        "results": [merged[state] for state in sorted(merged)],
        "built_count": sum(row["status"] == "built" for row in merged.values()),
        "failed_count": sum(row["status"] == "failed" for row in merged.values()),
        "remaining_count": max(0, len(inventory["batch_order"]) - len(merged)),
        "claim_boundary": "Resumable engineering ledger; aggregate verification occurs after all State contexts exist.",
    }
    ledger_path.write_text(json.dumps(ledger, indent=2) + "\n", encoding="utf-8")
    print(
        f"National RCTX batch: {ledger['built_count']} built, "
        f"{ledger['failed_count']} failed, {ledger['remaining_count']} remaining"
    )


if __name__ == "__main__":
    main()
