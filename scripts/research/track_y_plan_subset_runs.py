#!/usr/bin/env python3
"""Plan the Track Y geographic/cohesion subset mode-pair commands."""

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
    parser = argparse.ArgumentParser(description="Plan Track Y subset run commands.")
    parser.add_argument("--base-dir", type=Path, default=DEFAULT_BASE)
    parser.add_argument("--state", default="CA")
    parser.add_argument("--year", default="2020")
    parser.add_argument("--districts", type=int, default=2)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--structure", default="prime-factor")
    parser.add_argument("--output-root", default="outputs/track-y-ca-subset")
    parser.add_argument("--write-json", type=Path)
    parser.add_argument(
        "--allow-rejected",
        action="store_true",
        help="Return success after writing a rejected diagnostic plan.",
    )
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def read_json(path: Path) -> dict[str, object]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise ValueError(f"{path} must contain an object")
    return raw


def read_optional_json(path: Path) -> dict[str, object] | None:
    if not path.exists():
        return None
    return read_json(path)


def adjacency_path_from_preflight(preflight: dict[str, object]) -> str | None:
    adjacency = preflight.get("adjacency")
    if isinstance(adjacency, dict):
        path = adjacency.get("path")
        if isinstance(path, str):
            return path
    return None


def rejected_next_step(
    preflight: dict[str, object] | None,
    fixture: dict[str, object] | None,
) -> str:
    if preflight is None:
        return "run_preflight"
    if preflight.get("ready") is not True:
        return str(preflight.get("next_step", "repair_or_replace_adjacency_artifact"))
    if fixture is None:
        return "build_subset_fixture"
    return str(fixture.get("next_step", "adjust_subset_selection"))


def plan_commands(
    base_dir: Path,
    state: str,
    year: str,
    districts: int,
    seed: int,
    structure: str,
    output_root: str,
) -> dict[str, object]:
    preflight_path = base_dir / "ca-subset-preflight.json"
    fixture_path = base_dir / "ca-subset-fixture.json"
    preflight = read_optional_json(preflight_path)
    fixture = read_optional_json(fixture_path)

    reasons: list[str] = []
    if preflight is None:
        reasons.append("preflight is missing")
    elif preflight.get("ready") is not True:
        reasons.append("preflight is not ready")
    if fixture is None:
        reasons.append("fixture is missing")
    elif fixture.get("accepted_for_pulse_05") is not True:
        reasons.append("fixture is not accepted for Pulse 05")
    adjacency_path = adjacency_path_from_preflight(preflight) if preflight is not None else None
    if adjacency_path is None:
        reasons.append("preflight has no adjacency path")

    accepted = len(reasons) == 0
    commands: dict[str, str] = {}
    if accepted:
        common = (
            f"cargo run -p bisect-cli -- state --state {state.upper()} --year {year} "
            f"--districts {districts} --structure {structure} --seed {seed} "
            f"--adjacency \"{adjacency_path}\""
        )
        commands = {
            "geographic": (
                f"{common} --weights-override geographic "
                f"--output-dir \"{output_root}/geographic\""
            ),
            "cohesion": (
                f"{common} --weights-override cohesion "
                f"--output-dir \"{output_root}/cohesion\""
            ),
        }

    return {
        "schema": "bisect.track_y.subset_run_plan.v1",
        "base_dir": str(base_dir),
        "accepted_for_pulse_05": accepted,
        "acceptance_reasons": reasons,
        "commands": commands,
        "next_step": "run_mode_pair" if accepted else rejected_next_step(preflight, fixture),
        "comparison_command": (
            "python scripts/research/track_y_compare_subset_runs.py "
            f"--geographic-dir \"{output_root}/geographic\" "
            f"--cohesion-dir \"{output_root}/cohesion\" "
            f"--write-json \"{base_dir / 'ca-subset-comparison.json'}\""
        ),
        "claim_boundary": "command plan only; not a statewide, legal, or fairness claim",
    }


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        base = Path(tmp)
        missing = plan_commands(base, "CA", "2020", 2, 42, "prime-factor", "outputs/test")
        assert missing["accepted_for_pulse_05"] is False
        assert "preflight is missing" in missing["acceptance_reasons"]
        assert "fixture is missing" in missing["acceptance_reasons"]
        assert missing["next_step"] == "run_preflight"

        write_json(
            base / "ca-subset-preflight.json",
            {
                "ready": False,
                "next_step": "fetch_or_supply_adjacency_artifact",
                "error": "No California adjacency artifact found.",
            },
        )
        blocked = plan_commands(base, "CA", "2020", 2, 42, "prime-factor", "outputs/test")
        assert blocked["accepted_for_pulse_05"] is False
        assert blocked["next_step"] == "fetch_or_supply_adjacency_artifact"

        write_json(
            base / "ca-subset-preflight.json",
            {
                "ready": True,
                "adjacency": {"path": "C:/tmp/ca.adj.bin", "format": "radj"},
            },
        )
        write_json(
            base / "ca-subset-fixture.json",
            {"accepted_for_pulse_05": True, "acceptance_reasons": []},
        )
        planned = plan_commands(base, "CA", "2020", 2, 42, "prime-factor", "outputs/test")
        assert planned["accepted_for_pulse_05"] is True
        assert planned["next_step"] == "run_mode_pair"
        assert "--weights-override geographic" in planned["commands"]["geographic"]
        assert "--weights-override cohesion" in planned["commands"]["cohesion"]
        assert "--adjacency \"C:/tmp/ca.adj.bin\"" in planned["commands"]["cohesion"]

        write_json(
            base / "ca-subset-fixture.json",
            {"accepted_for_pulse_05": False, "acceptance_reasons": ["missing bridge evidence"]},
        )
        rejected = plan_commands(base, "CA", "2020", 2, 42, "prime-factor", "outputs/test")
        assert rejected["accepted_for_pulse_05"] is False
        assert "fixture is not accepted for Pulse 05" in rejected["acceptance_reasons"]
        assert rejected["next_step"] == "adjust_subset_selection"
        assert rejected["commands"] == {}


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        print("[OK] track_y_plan_subset_runs self-test passed")
        return 0
    plan = plan_commands(
        args.base_dir,
        args.state,
        args.year,
        args.districts,
        args.seed,
        args.structure,
        args.output_root,
    )
    text = json.dumps(plan, indent=2, sort_keys=True)
    print(text)
    if args.write_json:
        write_json(args.write_json, plan)
    return 0 if plan["accepted_for_pulse_05"] or args.allow_rejected else 2


if __name__ == "__main__":
    raise SystemExit(main())
