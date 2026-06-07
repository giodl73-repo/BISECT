#!/usr/bin/env python3
"""Gate Track Y Pulse 05 closure on accepted subset comparison evidence."""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path

from track_y_pulse05_status import DEFAULT_BASE, summarize, write_json


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Gate Track Y Pulse 05 acceptance.")
    parser.add_argument("--base-dir", type=Path, default=DEFAULT_BASE)
    parser.add_argument("--write-json", type=Path)
    parser.add_argument("--allow-blocked", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def gate(base_dir: Path) -> dict[str, object]:
    status = summarize(base_dir)
    accepted = status["accepted_for_pulse_05"] is True
    return {
        "schema": "bisect.track_y.pulse05_gate.v1",
        "accepted_for_pulse_05": accepted,
        "stage": status["stage"],
        "next_step": (
            "close_pulse_05_or_promote"
            if accepted
            else status["next_step"]
        ),
        "blocking_reasons": status["blocking_reasons"],
        "artifacts": status["artifacts"],
        "claim_boundary": "gate result only; not statewide, legal, or fairness evidence",
    }


def self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        base = Path(tmp)
        blocked = gate(base)
        assert blocked["accepted_for_pulse_05"] is False
        assert blocked["stage"] == "preflight_missing"
        assert blocked["next_step"] == "run_preflight"

        write_json(
            base / "ca-subset-comparison.json",
            {
                "accepted_for_pulse_05": True,
                "acceptance_reasons": [],
                "next_step": "close_pulse_05_or_promote",
            },
        )
        accepted = gate(base)
        assert accepted["accepted_for_pulse_05"] is True
        assert accepted["stage"] == "comparison"
        assert accepted["next_step"] == "close_pulse_05_or_promote"


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        print("[OK] track_y_gate_pulse05 self-test passed")
        return 0
    result = gate(args.base_dir)
    print(json.dumps(result, indent=2, sort_keys=True))
    if args.write_json:
        write_json(args.write_json, result)
    return 0 if result["accepted_for_pulse_05"] or args.allow_blocked else 2


if __name__ == "__main__":
    raise SystemExit(main())
