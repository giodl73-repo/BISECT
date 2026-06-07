#!/usr/bin/env python3
"""Refresh Track Y Pulse 05 diagnostic artifacts.

This orchestrates the non-destructive helper chain:

1. CA adjacency preflight.
2. Subset run-plan generation.
3. Pulse 05 status summary.
4. Pulse 05 closure gate.

It does not build a subset fixture, run BISECT, or make any statewide claim.
"""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path

from track_y_ca_subset_preflight import build_preflight_result
from track_y_gate_pulse05 import gate
from track_y_plan_subset_runs import plan_commands
from track_y_pulse05_status import summarize


DEFAULT_BASE = Path(
    "research/tracks/Y-bio-symmetry-spatial-factorization/"
    "Y.1+cohesion-weighted-bisection"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Refresh Track Y Pulse 05 artifacts.")
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--base-dir", type=Path, default=DEFAULT_BASE)
    parser.add_argument("--state", default="CA")
    parser.add_argument("--year", type=int, default=2020)
    parser.add_argument("--districts", type=int, default=2)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--structure", default="prime-factor")
    parser.add_argument("--output-root", default="outputs/track-y-ca-subset")
    parser.add_argument("--adjacency-override", type=Path)
    parser.add_argument(
        "--allow-blocked",
        action="store_true",
        help="Return success after refreshing blocked diagnostic artifacts.",
    )
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def refresh(
    repo_root: Path,
    base_dir: Path,
    state: str,
    year: int,
    districts: int,
    seed: int,
    structure: str,
    output_root: str,
    adjacency_override: Path | None,
) -> dict[str, object]:
    repo_root = repo_root.resolve()
    preflight_path = base_dir / "ca-subset-preflight.json"
    run_plan_path = base_dir / "ca-subset-run-plan.json"
    status_path = base_dir / "ca-subset-status.json"
    gate_path = base_dir / "ca-subset-gate.json"

    preflight = build_preflight_result(
        repo_root,
        year,
        state,
        preflight_path,
        adjacency_override,
    )
    write_json(preflight_path, preflight)

    plan = plan_commands(
        base_dir,
        state,
        str(year),
        districts,
        seed,
        structure,
        output_root,
    )
    write_json(run_plan_path, plan)

    status = summarize(base_dir)
    write_json(status_path, status)
    gate_result = gate(base_dir)
    write_json(gate_path, gate_result)

    return {
        "schema": "bisect.track_y.pulse05_refresh.v1",
        "accepted_for_pulse_05": gate_result["accepted_for_pulse_05"],
        "stage": status["stage"],
        "next_step": gate_result["next_step"],
        "artifacts": {
            "preflight": str(preflight_path),
            "run_plan": str(run_plan_path),
            "status": str(status_path),
            "gate": str(gate_path),
        },
        "blocking_reasons": gate_result["blocking_reasons"],
        "claim_boundary": "diagnostic refresh only; not statewide, legal, or fairness evidence",
    }


def self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        repo_root = Path(tmp) / "repo"
        base_dir = Path(tmp) / "base"
        (repo_root / "data").mkdir(parents=True)
        (repo_root / "data" / "manifest.json").write_text(
            json.dumps(
                {
                    "version": "1",
                    "github_repo": "example/repo",
                    "releases": {"data_inputs": "data-inputs-test"},
                    "local_data_dir": "data",
                    "local_outputs_dir": "outputs",
                }
            ),
            encoding="utf-8",
        )

        refreshed = refresh(
            repo_root,
            base_dir,
            "CA",
            2020,
            2,
            42,
            "prime-factor",
            "outputs/test",
            None,
        )
        assert refreshed["stage"] == "run_plan"
        assert refreshed["next_step"] == "fetch_or_supply_adjacency_artifact"
        assert refreshed["accepted_for_pulse_05"] is False
        assert (base_dir / "ca-subset-preflight.json").exists()
        assert (base_dir / "ca-subset-run-plan.json").exists()
        assert (base_dir / "ca-subset-status.json").exists()
        assert (base_dir / "ca-subset-gate.json").exists()


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        print("[OK] track_y_refresh_pulse05 self-test passed")
        return 0

    refreshed = refresh(
        args.repo_root,
        args.base_dir,
        args.state,
        args.year,
        args.districts,
        args.seed,
        args.structure,
        args.output_root,
        args.adjacency_override,
    )
    print(json.dumps(refreshed, indent=2, sort_keys=True))
    return 0 if refreshed["accepted_for_pulse_05"] or args.allow_blocked else 2


if __name__ == "__main__":
    raise SystemExit(main())
