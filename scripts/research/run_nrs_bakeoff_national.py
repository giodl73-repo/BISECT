#!/usr/bin/env python3
"""Run the governed 2020 NRS-versus-CD118 Tier 1 bakeoff nationally."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
sys.path.insert(0, str(ROOT / "scripts" / "research"))

from config.download_sources import STATE_FIPS
from state_config import STATE_CONFIG_2020
from analyze_nrs_bakeoff_slice import (
    BakeoffError,
    CLAIM_BOUNDARY,
    PROTOCOL_ID,
    sha256,
    write_package,
)


SCHEMA_VERSION = "nrs-v0.3-national-bakeoff-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-national-bakeoff-manifest-v1"
BATCH_PATH = Path("scripts/research/run_nrs_bakeoff_national.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_bakeoff_national.py")
PROTOCOL_PATH = Path("docs/specs/2026-08-07-nrs-v0.3-national-bakeoff-protocol.md")


def projection_for_state(state: str) -> str:
    if state == "AK":
        return "EPSG:3338"
    if state == "HI":
        return "EPSG:3759"
    return "EPSG:5070"


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def weighted_mean(rows: list[dict], field: str, weight_field: str) -> float:
    total_weight = sum(row[weight_field] for row in rows)
    return sum(row[field] * row[weight_field] for row in rows) / total_weight


def run_national(nrs_root: Path, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    states_dir = output_dir / "states"
    rows = []
    failures = []
    output_relative = output_dir.resolve().relative_to(ROOT.resolve()).as_posix()

    for state in sorted(STATE_CONFIG_2020):
        fips = STATE_FIPS[state]
        state_dir = states_dir / state.lower()
        nrs_path = nrs_root / "states" / state.lower() / "package" / "baseline_assignments.json"
        block_path = (
            ROOT
            / "data"
            / "2020"
            / "tiger"
            / "blocks"
            / f"tl_2020_{fips}_tabblock20"
            / f"tl_2020_{fips}_tabblock20.shp"
        )
        comparator_path = (
            ROOT / "data" / "enacted_districts" / f"tl_2020_{fips}_cd118.zip"
        )
        try:
            write_package(
                root=ROOT,
                package_dir=state_dir,
                state=state,
                year=2020,
                state_fips=fips,
                projection=projection_for_state(state),
                nrs_path=nrs_path,
                block_path=block_path,
                comparator_path=comparator_path,
                comparator_state_column="STATEFP20",
                comparator_district_column="CD118FP",
                comparator_session_column="CDSESSN",
                expected_session="118",
                display_output_dir=f"{output_relative}/states/{state.lower()}",
            )
            analysis = json.loads((state_dir / "analysis.json").read_text(encoding="utf-8"))
            rows.append(
                {
                    "state": state,
                    "districts": analysis["benchmark"]["districts"],
                    "source_blocks": analysis["atomic_universe"]["source_blocks"],
                    "excluded_water_only_blocks": analysis["atomic_universe"][
                        "excluded_water_only_blocks"
                    ],
                    "blocks": analysis["benchmark"]["blocks"],
                    "matched_blocks": analysis["comparison"]["matched_blocks"],
                    "moved_blocks": analysis["comparison"]["moved_blocks"],
                    "matched_block_rate": analysis["comparison"]["matched_block_rate"],
                    "nrs_county_splits": analysis["benchmark"]["county_splits"]["split_units"],
                    "comparator_county_splits": analysis["comparator"]["county_splits"]["split_units"],
                    "county_split_difference": analysis["comparison"][
                        "comparator_minus_benchmark"
                    ]["county_split_units"],
                    "nrs_tract_splits": analysis["benchmark"]["tract_splits"]["split_units"],
                    "comparator_tract_splits": analysis["comparator"]["tract_splits"][
                        "split_units"
                    ],
                    "tract_split_difference": analysis["comparison"][
                        "comparator_minus_benchmark"
                    ]["tract_split_units"],
                }
            )
        except (BakeoffError, FileNotFoundError, OSError) as error:
            failures.append({"state": state, "error": str(error)})

    rows.sort(key=lambda row: row["state"])
    failures.sort(key=lambda row: row["state"])
    status = "pass" if len(rows) == 50 and not failures else "partial"
    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "census_year": 2020,
        "comparator": "official-census-cd118-state-archives",
        "states_passed": len(rows),
        "states_failed": len(failures),
        "failures": failures,
        "national_totals": {
            "districts": sum(row["districts"] for row in rows),
            "source_blocks": sum(row["source_blocks"] for row in rows),
            "excluded_water_only_blocks": sum(
                row["excluded_water_only_blocks"] for row in rows
            ),
            "blocks": sum(row["blocks"] for row in rows),
            "matched_blocks": sum(row["matched_blocks"] for row in rows),
            "moved_blocks": sum(row["moved_blocks"] for row in rows),
            "matched_block_rate": sum(row["matched_blocks"] for row in rows)
            / sum(row["blocks"] for row in rows),
            "nrs_county_splits": sum(row["nrs_county_splits"] for row in rows),
            "comparator_county_splits": sum(
                row["comparator_county_splits"] for row in rows
            ),
            "nrs_tract_splits": sum(row["nrs_tract_splits"] for row in rows),
            "comparator_tract_splits": sum(
                row["comparator_tract_splits"] for row in rows
            ),
        },
        "state_weighted": {
            "mean_matched_block_rate": sum(row["matched_block_rate"] for row in rows)
            / len(rows),
            "mean_county_split_difference": sum(
                row["county_split_difference"] for row in rows
            )
            / len(rows),
            "mean_tract_split_difference": sum(
                row["tract_split_difference"] for row in rows
            )
            / len(rows),
        },
        "district_weighted": {
            "mean_matched_block_rate": weighted_mean(
                rows, "matched_block_rate", "districts"
            ),
            "mean_county_split_difference": weighted_mean(
                rows, "county_split_difference", "districts"
            ),
            "mean_tract_split_difference": weighted_mean(
                rows, "tract_split_difference", "districts"
            ),
        },
        "unavailable_metrics": {
            "population": "Tier 1 does not freeze comparator block-population allocation.",
            "geometric_compactness": "The national geometry/perimeter protocol is not frozen.",
            "partisan": "The evaluation schedule election and precinct-to-block inputs are not frozen.",
            "demographic_and_vra": "No frozen national within-unit allocation or legal analysis.",
            "sensitivity": "The 100 diagnostic seed package has not been run.",
            "ensemble": "No block-level preregistered national ensemble exists.",
        },
        "claim_boundary": CLAIM_BOUNDARY,
    }
    analysis_path = output_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8")

    csv_path = output_dir / "state-summary.csv"
    with csv_path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)

    totals = analysis["national_totals"]
    readme = f"""# NRS v0.3 National 2020 Tier 1 Bakeoff

**Status:** {status}

| Measure | NRS v0.3 | Official CD118 comparator |
|---|---:|---:|
| States passed | {len(rows)} | {len(rows)} |
| Districts | {totals['districts']:,} | {totals['districts']:,} |
| Source Census blocks | {totals['source_blocks']:,} | {totals['source_blocks']:,} |
| Excluded water-only blocks | {totals['excluded_water_only_blocks']:,} | {totals['excluded_water_only_blocks']:,} |
| Analyzed land-containing blocks | {totals['blocks']:,} | {totals['blocks']:,} |
| County split units | {totals['nrs_county_splits']:,} | {totals['comparator_county_splits']:,} |
| Tract split units | {totals['nrs_tract_splits']:,} | {totals['comparator_tract_splits']:,} |

After State-level maximum-overlap district-label matching,
{totals['matched_blocks']:,} blocks match and {totals['moved_blocks']:,}
differ ({totals['matched_block_rate']:.6%} block-weighted agreement).

State-weighted and district-weighted estimands are reported separately in
`analysis.json`; per-State results and every failure are in
`state-summary.csv`.

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = output_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8")

    state_manifests = [
        {
            "state": row["state"],
            "path": f"states/{row['state'].lower()}/manifest.json",
            "sha256": file_sha256(states_dir / row["state"].lower() / "manifest.json"),
        }
        for row in rows
    ]
    code_paths = [
        PROTOCOL_PATH,
        Path("scripts/research/analyze_nrs_bakeoff_slice.py"),
        Path("scripts/research/verify_nrs_bakeoff_slice.py"),
        BATCH_PATH,
        VERIFIER_PATH,
    ]
    manifest = {
        "schema_version": MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": status,
        "census_year": 2020,
        "state_manifests": state_manifests,
        "code": [
            {"path": path.as_posix(), "sha256": sha256(ROOT / path)}
            for path in code_paths
        ],
        "outputs": {
            "analysis.json": file_sha256(analysis_path),
            "state-summary.csv": file_sha256(csv_path),
            "README.md": file_sha256(readme_path),
        },
        "claim_boundary": CLAIM_BOUNDARY,
    }
    (output_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--nrs-root",
        type=Path,
        default=ROOT / "runs/nrs-v0.3/neutral-analysis/national-2020",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=ROOT / "docs/experiments/nrs-v0.3-national-bakeoff-2020",
    )
    args = parser.parse_args()
    run_national(args.nrs_root.resolve(), args.output_dir.resolve())


if __name__ == "__main__":
    main()
