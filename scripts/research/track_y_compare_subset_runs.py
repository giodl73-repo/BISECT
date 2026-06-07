#!/usr/bin/env python3
"""Compare Track Y geographic and cohesion subset run artifacts.

This is a research helper for Pulse 05. It intentionally checks only artifact
structure and summary diagnostics; it does not make statewide, legal, or
fairness claims.
"""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare geographic and cohesion subset run artifacts."
    )
    parser.add_argument("--geographic-dir", type=Path)
    parser.add_argument("--cohesion-dir", type=Path)
    parser.add_argument("--write-json", type=Path)
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def read_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def load_assignments(run_dir: Path) -> dict[str, int]:
    path = run_dir / "data" / "final_assignments.json"
    if not path.exists():
        path = run_dir / "final_assignments.json"
    if not path.exists():
        raise FileNotFoundError(f"missing final_assignments.json under {run_dir}")
    raw = read_json(path)
    if not isinstance(raw, dict):
        raise ValueError(f"{path} must contain an object")
    return {str(key): int(value) for key, value in raw.items()}


def load_manifest(run_dir: Path) -> dict[str, object] | None:
    for path in [run_dir / "manifest.json", run_dir / "data" / "manifest.json"]:
        if path.exists():
            raw = read_json(path)
            if not isinstance(raw, dict):
                raise ValueError(f"{path} must contain an object")
            return raw
    return None


def load_cohesion_sidecar(run_dir: Path) -> dict[str, object] | None:
    for path in [run_dir / "data" / "cohesion.json", run_dir / "cohesion.json"]:
        if path.exists():
            raw = read_json(path)
            if not isinstance(raw, dict):
                raise ValueError(f"{path} must contain an object")
            return raw
    return None


def summarize_assignments(assignments: dict[str, int]) -> dict[str, int]:
    districts = sorted(set(assignments.values()))
    return {
        "tract_count": len(assignments),
        "district_count": len(districts),
        "district_min": min(districts) if districts else 0,
        "district_max": max(districts) if districts else 0,
    }


def compare_runs(geographic_dir: Path, cohesion_dir: Path) -> dict[str, object]:
    geo_assignments = load_assignments(geographic_dir)
    cohesion_assignments = load_assignments(cohesion_dir)
    geo_summary = summarize_assignments(geo_assignments)
    cohesion_summary = summarize_assignments(cohesion_assignments)
    cohesion_sidecar = load_cohesion_sidecar(cohesion_dir)
    cohesion_manifest = load_manifest(cohesion_dir)

    same_units = set(geo_assignments) == set(cohesion_assignments)
    changed_units = sorted(
        unit
        for unit in set(geo_assignments).intersection(cohesion_assignments)
        if geo_assignments[unit] != cohesion_assignments[unit]
    )

    manifest_sidecar_path = None
    if cohesion_manifest:
        value = cohesion_manifest.get("cohesion_sidecar_path")
        if isinstance(value, str):
            manifest_sidecar_path = value
    accepted, acceptance_reasons = evaluate_acceptance(
        same_units=same_units,
        geo_summary=geo_summary,
        cohesion_summary=cohesion_summary,
        cohesion_sidecar=cohesion_sidecar,
        manifest_sidecar_path=manifest_sidecar_path,
    )

    return {
        "schema": "bisect.track_y.subset_comparison.v1",
        "geographic_dir": str(geographic_dir),
        "cohesion_dir": str(cohesion_dir),
        "same_unit_set": same_units,
        "geographic": geo_summary,
        "cohesion": cohesion_summary,
        "assignment_changed_count": len(changed_units),
        "assignment_changed_sample": changed_units[:25],
        "accepted_for_pulse_05": accepted,
        "acceptance_reasons": acceptance_reasons,
        "next_step": "close_pulse_05_or_promote" if accepted else "repair_subset_runs",
        "cohesion_sidecar_present": cohesion_sidecar is not None,
        "cohesion_sidecar_schema": cohesion_sidecar.get("schema") if cohesion_sidecar else None,
        "cohesion_manifest_sidecar_path": manifest_sidecar_path,
        "cohesion_cut_edges": cohesion_sidecar.get("cut_edges") if cohesion_sidecar else None,
        "cohesion_cut_edges_low_cycle_share": (
            cohesion_sidecar.get("cut_edges_low_cycle_share") if cohesion_sidecar else None
        ),
        "cohesion_cut_edges_avg_bridge_likeness": (
            cohesion_sidecar.get("cut_edges_avg_bridge_likeness") if cohesion_sidecar else None
        ),
        "cohesion_mass_factor_min": cohesion_sidecar.get("mass_factor_min")
        if cohesion_sidecar
        else None,
        "cohesion_mass_factor_median": cohesion_sidecar.get("mass_factor_median")
        if cohesion_sidecar
        else None,
        "cohesion_mass_factor_max": cohesion_sidecar.get("mass_factor_max")
        if cohesion_sidecar
        else None,
        "cohesion_forbidden_fields_used": (
            cohesion_sidecar.get("forbidden_fields_used") if cohesion_sidecar else None
        ),
        "claim_boundary": "subset research comparison only; not statewide, legal, or fairness evidence",
    }


def evaluate_acceptance(
    same_units: bool,
    geo_summary: dict[str, int],
    cohesion_summary: dict[str, int],
    cohesion_sidecar: dict[str, object] | None,
    manifest_sidecar_path: str | None,
) -> tuple[bool, list[str]]:
    reasons: list[str] = []
    if not same_units:
        reasons.append("geographic and cohesion runs cover different unit sets")
    if geo_summary["tract_count"] == 0:
        reasons.append("geographic run has no assignments")
    if cohesion_summary["tract_count"] == 0:
        reasons.append("cohesion run has no assignments")
    if geo_summary["district_count"] != cohesion_summary["district_count"]:
        reasons.append("district counts differ between runs")
    if cohesion_sidecar is None:
        reasons.append("cohesion sidecar missing")
    else:
        if cohesion_sidecar.get("schema") != "bisect.cohesion.v1":
            reasons.append("cohesion sidecar schema is not bisect.cohesion.v1")
        forbidden = cohesion_sidecar.get("forbidden_fields_used")
        if forbidden not in ([], None):
            reasons.append("cohesion sidecar reports forbidden fields")
    if manifest_sidecar_path is None:
        reasons.append("cohesion manifest does not declare cohesion_sidecar_path")
    return len(reasons) == 0, reasons


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_fixture_run(
    run_dir: Path,
    assignments: dict[str, int],
    cohesion: bool,
) -> None:
    data_dir = run_dir / "data"
    data_dir.mkdir(parents=True)
    write_json(data_dir / "final_assignments.json", assignments)
    if cohesion:
        write_json(
            run_dir / "manifest.json",
            {"cohesion_sidecar_path": "data/cohesion.json"},
        )
        write_json(
            data_dir / "cohesion.json",
            {
                "schema": "bisect.cohesion.v1",
                "cut_edges": 3,
                "cut_edges_low_cycle_share": 0.667,
                "cut_edges_avg_bridge_likeness": 0.75,
                "mass_factor_min": 0.5,
                "mass_factor_median": 1.0,
                "mass_factor_max": 2.0,
                "forbidden_fields_used": [],
            },
        )


def self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        geo_dir = root / "geographic"
        cohesion_dir = root / "cohesion"
        write_fixture_run(geo_dir, {"0": 1, "1": 1, "2": 2}, cohesion=False)
        write_fixture_run(cohesion_dir, {"0": 1, "1": 2, "2": 2}, cohesion=True)
        comparison = compare_runs(geo_dir, cohesion_dir)
        assert comparison["same_unit_set"] is True
        assert comparison["geographic"]["tract_count"] == 3
        assert comparison["cohesion"]["district_count"] == 2
        assert comparison["assignment_changed_count"] == 1
        assert comparison["accepted_for_pulse_05"] is True
        assert comparison["acceptance_reasons"] == []
        assert comparison["next_step"] == "close_pulse_05_or_promote"
        assert comparison["cohesion_sidecar_present"] is True
        assert comparison["cohesion_sidecar_schema"] == "bisect.cohesion.v1"
        assert comparison["cohesion_forbidden_fields_used"] == []

        missing_sidecar_dir = root / "missing-sidecar"
        write_fixture_run(missing_sidecar_dir, {"0": 1, "1": 2, "2": 2}, cohesion=False)
        rejected = compare_runs(geo_dir, missing_sidecar_dir)
        assert rejected["accepted_for_pulse_05"] is False
        assert "cohesion sidecar missing" in rejected["acceptance_reasons"]
        assert rejected["next_step"] == "repair_subset_runs"


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        print("[OK] track_y_compare_subset_runs self-test passed")
        return 0
    if args.geographic_dir is None:
        raise SystemExit("--geographic-dir is required unless --self-test is set")
    if args.cohesion_dir is None:
        raise SystemExit("--cohesion-dir is required unless --self-test is set")
    if args.write_json is None:
        raise SystemExit("--write-json is required unless --self-test is set")

    comparison = compare_runs(args.geographic_dir, args.cohesion_dir)
    write_json(args.write_json, comparison)
    print(json.dumps(comparison, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
