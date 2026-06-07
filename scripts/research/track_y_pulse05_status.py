#!/usr/bin/env python3
"""Summarize Track Y Pulse 05 helper state.

Reads the optional preflight, fixture, run-plan, and comparison JSON artifacts
and reports the current stage plus the next machine-readable action.
"""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path


DEFAULT_BASE = Path(
    "research/tracks/Y-bio-symmetry-spatial-factorization/"
    "Y.1+cohesion-weighted-bisection"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Summarize Track Y Pulse 05 status.")
    parser.add_argument("--base-dir", type=Path, default=DEFAULT_BASE)
    parser.add_argument("--write-json", type=Path)
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def read_optional_json(path: Path) -> dict[str, object] | None:
    if not path.exists():
        return None
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise ValueError(f"{path} must contain an object")
    return raw


def summarize(base_dir: Path) -> dict[str, object]:
    preflight_path = base_dir / "ca-subset-preflight.json"
    fixture_path = base_dir / "ca-subset-fixture.json"
    run_plan_path = base_dir / "ca-subset-run-plan.json"
    comparison_path = base_dir / "ca-subset-comparison.json"

    preflight = read_optional_json(preflight_path)
    fixture = read_optional_json(fixture_path)
    run_plan = read_optional_json(run_plan_path)
    comparison = read_optional_json(comparison_path)

    stage = "preflight_missing"
    next_step = "run_preflight"
    accepted = False
    reasons: list[str] = []

    if preflight is not None:
        stage = "preflight"
        next_step = str(preflight.get("next_step", "unknown"))
        if preflight.get("ready") is not True:
            reasons.append(str(preflight.get("error", "preflight not ready")))

    if fixture is not None:
        stage = "fixture"
        next_step = str(fixture.get("next_step", "unknown"))
        if fixture.get("accepted_for_pulse_05") is not True:
            raw_reasons = fixture.get("acceptance_reasons", [])
            if isinstance(raw_reasons, list):
                reasons.extend(str(reason) for reason in raw_reasons)

    if run_plan is not None:
        stage = "run_plan"
        next_step = str(run_plan.get("next_step", "unknown"))
        if run_plan.get("accepted_for_pulse_05") is not True:
            raw_reasons = run_plan.get("acceptance_reasons", [])
            if isinstance(raw_reasons, list):
                reasons.extend(str(reason) for reason in raw_reasons)

    if comparison is not None:
        stage = "comparison"
        next_step = str(comparison.get("next_step", "unknown"))
        accepted = comparison.get("accepted_for_pulse_05") is True
        raw_reasons = comparison.get("acceptance_reasons", [])
        if isinstance(raw_reasons, list):
            reasons.extend(str(reason) for reason in raw_reasons)

    return {
        "schema": "bisect.track_y.pulse05_status.v1",
        "base_dir": str(base_dir),
        "stage": stage,
        "next_step": next_step,
        "accepted_for_pulse_05": accepted,
        "blocking_reasons": reasons,
        "artifacts": {
            "preflight": str(preflight_path) if preflight is not None else None,
            "fixture": str(fixture_path) if fixture is not None else None,
            "run_plan": str(run_plan_path) if run_plan is not None else None,
            "comparison": str(comparison_path) if comparison is not None else None,
        },
        "claim_boundary": "workflow status only; not statewide, legal, or fairness evidence",
    }


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        base = Path(tmp)
        status = summarize(base)
        assert status["stage"] == "preflight_missing"
        assert status["next_step"] == "run_preflight"

        write_json(
            base / "ca-subset-preflight.json",
            {
                "ready": False,
                "next_step": "fetch_or_supply_adjacency_artifact",
                "error": "No California adjacency artifact found.",
            },
        )
        status = summarize(base)
        assert status["stage"] == "preflight"
        assert status["next_step"] == "fetch_or_supply_adjacency_artifact"
        assert status["accepted_for_pulse_05"] is False

        write_json(
            base / "ca-subset-fixture.json",
            {
                "accepted_for_pulse_05": True,
                "acceptance_reasons": [],
                "next_step": "run_mode_pair",
            },
        )
        status = summarize(base)
        assert status["stage"] == "fixture"
        assert status["next_step"] == "run_mode_pair"

        write_json(
            base / "ca-subset-run-plan.json",
            {
                "accepted_for_pulse_05": True,
                "acceptance_reasons": [],
                "next_step": "run_mode_pair",
            },
        )
        status = summarize(base)
        assert status["stage"] == "run_plan"
        assert status["next_step"] == "run_mode_pair"

        write_json(
            base / "ca-subset-comparison.json",
            {
                "accepted_for_pulse_05": True,
                "acceptance_reasons": [],
                "next_step": "close_pulse_05_or_promote",
            },
        )
        status = summarize(base)
        assert status["stage"] == "comparison"
        assert status["accepted_for_pulse_05"] is True
        assert status["next_step"] == "close_pulse_05_or_promote"


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        print("[OK] track_y_pulse05_status self-test passed")
        return 0
    status = summarize(args.base_dir)
    text = json.dumps(status, indent=2, sort_keys=True)
    print(text)
    if args.write_json:
        write_json(args.write_json, status)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
