#!/usr/bin/env python3
"""Run the governed 2020 Tier 2 geometry bakeoff sequentially."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
sys.path.insert(0, str(ROOT / "scripts" / "research"))

from config.download_sources import STATE_FIPS
from config_2020 import STATE_CONFIG_2020
from analyze_nrs_bakeoff_geometry_slice import BakeoffError, write_package


SCHEMA_VERSION = "nrs-v0.3-national-bakeoff-geometry-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-national-bakeoff-geometry-manifest-v1"
PROTOCOL_ID = "nrs-v0.3-national-tier2-geometry-v1"
PROTOCOL_PATH = Path("docs/specs/2026-08-08-nrs-v0.3-national-tier2-geometry-protocol.md")
RUNNER_PATH = Path("scripts/research/run_nrs_bakeoff_geometry_national.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_bakeoff_geometry_national.py")
ANALYZER_PATH = Path("scripts/research/analyze_nrs_bakeoff_geometry_slice.py")
CLAIM_BOUNDARY = (
    "Descriptive national compactness summaries for 2020 NRS v0.3 and official "
    "CD118 assignments projected to identical retained Census-block geometry; "
    "no original-linework, compactness-superiority, fairness, intent, VRA, legal, "
    "community, robustness, optimality, causal, or adoption claim."
)
METRICS = ("polsby_popper", "reock", "convex_hull_ratio", "schwartzberg")


def projection_for_state(state: str) -> str:
    if state == "AK":
        return "EPSG:3338"
    if state == "HI":
        return "EPSG:3759"
    return "EPSG:5070"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_csv(path: Path, fieldnames: list[str], rows: list[dict]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def metric_summary(state_rows: list[dict], district_rows: list[dict]) -> dict:
    result = {}
    for metric in METRICS:
        nrs_state = sum(row[f"nrs_{metric}"] for row in state_rows) / len(state_rows)
        comparator_state = (
            sum(row[f"comparator_{metric}"] for row in state_rows) / len(state_rows)
        )
        nrs_district_values = [
            row[metric] for row in district_rows if row["plan_family"] == "NRS-v0.3"
        ]
        comparator_district_values = [
            row[metric]
            for row in district_rows
            if row["plan_family"] == "enacted-congressional-session-118"
        ]
        nrs_district = sum(nrs_district_values) / len(nrs_district_values)
        comparator_district = (
            sum(comparator_district_values) / len(comparator_district_values)
        )
        result[metric] = {
            "state_weighted": {
                "nrs": nrs_state,
                "comparator": comparator_state,
                "comparator_minus_nrs": comparator_state - nrs_state,
            },
            "district_weighted": {
                "nrs": nrs_district,
                "comparator": comparator_district,
                "comparator_minus_nrs": comparator_district - nrs_district,
            },
        }
    return result


def run_national(nrs_root: Path, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    states_dir = output_dir / "states"
    state_rows: list[dict] = []
    district_rows: list[dict] = []
    failures: list[dict] = []
    started = time.perf_counter()

    for state in sorted(STATE_CONFIG_2020):
        state_started = time.perf_counter()
        fips = STATE_FIPS[state]
        state_dir = states_dir / state.lower()
        try:
            write_package(
                root=ROOT,
                package_dir=state_dir,
                state=state,
                year=2020,
                state_fips=fips,
                projection=projection_for_state(state),
                nrs_path=nrs_root
                / "states"
                / state.lower()
                / "package"
                / "baseline_assignments.json",
                block_path=ROOT
                / "data"
                / "2020"
                / "tiger"
                / "blocks"
                / f"tl_2020_{fips}_tabblock20"
                / f"tl_2020_{fips}_tabblock20.shp",
                comparator_path=ROOT
                / "data"
                / "enacted_districts"
                / f"tl_2020_{fips}_cd118.zip",
                comparator_state_column="STATEFP20",
                comparator_district_column="CD118FP",
                comparator_session_column="CDSESSN",
                expected_session="118",
                display_output_dir=(
                    f"{output_dir.resolve().relative_to(ROOT.resolve()).as_posix()}"
                    f"/states/{state.lower()}"
                ),
            )
            analysis = json.loads(
                (state_dir / "analysis.json").read_text(encoding="utf-8")
            )
            state_row = {
                "state": state,
                "projection": projection_for_state(state),
                "retained_blocks": analysis["geometry_contract"]["retained_blocks"],
                "districts": analysis["benchmark"]["districts"],
            }
            for metric in METRICS:
                state_row[f"nrs_{metric}"] = analysis["benchmark"]["unweighted_mean"][
                    metric
                ]
                state_row[f"comparator_{metric}"] = analysis["comparator"][
                    "unweighted_mean"
                ][metric]
            state_rows.append(state_row)
            for key in ("benchmark", "comparator"):
                for row in analysis[key]["district_metrics"]:
                    district_rows.append(
                        {
                            "state": state,
                            "plan_family": analysis[key]["plan_family"],
                            **row,
                        }
                    )
            print(f"{state}: PASS ({time.perf_counter() - state_started:.1f}s)")
        except (BakeoffError, FileNotFoundError, OSError, ValueError) as error:
            failures.append({"state": state, "error": str(error)})
            print(f"{state}: FAIL {error}")

    state_rows.sort(key=lambda row: row["state"])
    district_rows.sort(
        key=lambda row: (row["state"], row["plan_family"], row["district"])
    )
    failures.sort(key=lambda row: row["state"])
    nrs_districts = sum(
        1 for row in district_rows if row["plan_family"] == "NRS-v0.3"
    )
    comparator_districts = sum(
        1
        for row in district_rows
        if row["plan_family"] == "enacted-congressional-session-118"
    )
    status = (
        "pass"
        if len(state_rows) == 50
        and not failures
        and nrs_districts == comparator_districts == 435
        else "partial"
    )

    district_fields = [
        "state",
        "plan_family",
        "district",
        "block_count",
        "area_m2",
        "perimeter_m",
        "minimum_bounding_radius_m",
        "polsby_popper",
        "reock",
        "convex_hull_ratio",
        "schwartzberg",
        "component_count",
        "interior_ring_count",
    ]
    state_fields = ["state", "projection", "retained_blocks", "districts"]
    for metric in METRICS:
        state_fields.extend((f"nrs_{metric}", f"comparator_{metric}"))
    district_path = output_dir / "district-results.csv"
    state_path = output_dir / "state-results.csv"
    write_csv(district_path, district_fields, district_rows)
    write_csv(state_path, state_fields, state_rows)

    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "census_year": 2020,
        "states_passed": len(state_rows),
        "states_failed": len(failures),
        "failures": failures,
        "districts": {
            "nrs": nrs_districts,
            "comparator": comparator_districts,
        },
        "retained_blocks": sum(row["retained_blocks"] for row in state_rows),
        "metrics": metric_summary(state_rows, district_rows) if state_rows else {},
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = output_dir / "analysis.json"
    analysis_path.write_text(
        json.dumps(analysis, indent=2) + "\n", encoding="utf-8", newline="\n"
    )
    metric_rows = "\n".join(
        (
            f"| {metric.replace('_', ' ').title()} | "
            f"{analysis['metrics'][metric]['district_weighted']['nrs']:.9f} | "
            f"{analysis['metrics'][metric]['district_weighted']['comparator']:.9f} | "
            f"{analysis['metrics'][metric]['district_weighted']['comparator_minus_nrs']:+.9f} |"
        )
        for metric in METRICS
    )
    readme = f"""# NRS v0.3 National 2020 Tier 2 Geometry Bakeoff

**Status:** {status}

| District-weighted mean | NRS v0.3 | Official CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
{metric_rows}

The package covers {len(state_rows)} States, {nrs_districts} NRS districts,
{comparator_districts} comparator districts, and
{analysis['retained_blocks']:,} retained land-containing Census blocks.
State-weighted results, all district rows, all State rows, and failures are in
the machine-readable package.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_bakeoff_geometry_national.py `
  --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020
python scripts/research/verify_nrs_bakeoff_geometry_national.py `
  docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020
```

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = output_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8", newline="\n")

    state_package_files = []
    for path in sorted(states_dir.rglob("*")):
        if path.is_file():
            state_package_files.append(
                {
                    "path": path.relative_to(output_dir).as_posix(),
                    "sha256": sha256(path),
                }
            )
    manifest = {
        "schema_version": MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "code": [
            {"path": path.as_posix(), "sha256": sha256(ROOT / path)}
            for path in (PROTOCOL_PATH, RUNNER_PATH, VERIFIER_PATH, ANALYZER_PATH)
        ],
        "state_package_files": state_package_files,
        "outputs": {
            name: sha256(output_dir / name)
            for name in (
                "analysis.json",
                "district-results.csv",
                "state-results.csv",
                "README.md",
            )
        },
        "reproduction": {
            "nrs_root": nrs_root.resolve().relative_to(ROOT.resolve()).as_posix(),
            "output_dir": output_dir.resolve().relative_to(ROOT.resolve()).as_posix(),
            "workers": 1,
        },
        "claim_boundary": CLAIM_BOUNDARY,
    }
    (output_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n"
    )
    print(f"National geometry status: {status}")
    print(f"Total elapsed: {time.perf_counter() - started:.1f}s")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--nrs-root",
        type=Path,
        default=Path("runs/nrs-v0.3/neutral-analysis/national-2020"),
    )
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    run_national(args.nrs_root, args.output_dir)


if __name__ == "__main__":
    main()
