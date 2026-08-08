#!/usr/bin/env python3
"""Build a governed Tier 1 NRS-versus-comparator evidence slice."""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path

import geopandas as gpd
import numpy as np
import pandas as pd
from scipy.optimize import linear_sum_assignment


SCHEMA_VERSION = "nrs-v0.3-bakeoff-slice-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-bakeoff-slice-manifest-v1"
PROTOCOL_ID = "nrs-v0.3-national-bakeoff-v1"
PROTOCOL_PATH = Path("docs/specs/2026-08-07-nrs-v0.3-national-bakeoff-protocol.md")
ANALYZER_PATH = Path("scripts/research/analyze_nrs_bakeoff_slice.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_bakeoff_slice.py")
CLAIM_BOUNDARY = (
    "Descriptive same-vintage atomic-block assignment overlap and county/tract "
    "split counts only; no compactness, population, partisan, demographic, VRA, "
    "legal-validity, optimality, superiority, or adoption claim."
)


class BakeoffError(ValueError):
    """Structured input or conformance failure."""


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def relative_path(path: Path, root: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return str(path.resolve()).replace("\\", "/")


def require(condition: bool, code: str, message: str) -> None:
    if not condition:
        raise BakeoffError(f"[{code}] {message}")


def validate_comparator_session(values: list[str], expected_session: str) -> str:
    sessions = sorted(set(str(value) for value in values))
    require(
        sessions == [expected_session],
        "INPUT",
        f"comparator-session-mismatch expected={expected_session} actual={sessions}",
    )
    return expected_session


def select_land_blocks(blocks: pd.DataFrame) -> tuple[pd.DataFrame, int]:
    require("ALAND20" in blocks.columns, "INPUT", "block schema is missing ALAND20")
    land_area = pd.to_numeric(blocks["ALAND20"], errors="coerce")
    require(land_area.notna().all(), "INPUT", "block ALAND20 contains invalid values")
    land_mask = land_area > 0
    return blocks.loc[land_mask].copy(), int((~land_mask).sum())


def select_numbered_districts(
    comparator: pd.DataFrame, district_column: str
) -> tuple[pd.DataFrame, list[str]]:
    labels = comparator[district_column].astype(str)
    numbered_mask = labels.str.fullmatch(r"\d+")
    excluded_labels = sorted(set(labels.loc[~numbered_mask].tolist()))
    return comparator.loc[numbered_mask].copy(), excluded_labels


def summarize_splits(assignments: dict[str, int], prefix_length: int) -> dict:
    memberships: dict[str, set[int]] = defaultdict(set)
    for geoid, district in assignments.items():
        memberships[geoid[:prefix_length]].add(district)
    split_units = sum(len(districts) > 1 for districts in memberships.values())
    excess_pieces = sum(max(0, len(districts) - 1) for districts in memberships.values())
    return {
        "units": len(memberships),
        "split_units": split_units,
        "excess_pieces": excess_pieces,
    }


def optimal_overlap(
    benchmark: dict[str, int], comparator: dict[str, int]
) -> dict:
    require(
        set(benchmark) == set(comparator),
        "INPUT",
        "benchmark and comparator atomic-unit universes differ",
    )
    benchmark_labels = sorted(set(benchmark.values()))
    comparator_labels = sorted(set(comparator.values()))
    require(
        len(benchmark_labels) == len(comparator_labels),
        "INPUT",
        "district-count-mismatch",
    )
    matrix = np.zeros((len(benchmark_labels), len(comparator_labels)), dtype=np.int64)
    benchmark_index = {label: index for index, label in enumerate(benchmark_labels)}
    comparator_index = {label: index for index, label in enumerate(comparator_labels)}
    for geoid, benchmark_label in benchmark.items():
        matrix[
            benchmark_index[benchmark_label],
            comparator_index[comparator[geoid]],
        ] += 1
    rows, columns = linear_sum_assignment(-matrix)
    matches = [
        {
            "benchmark_district": benchmark_labels[row],
            "comparator_district": comparator_labels[column],
            "matched_blocks": int(matrix[row, column]),
        }
        for row, column in zip(rows, columns, strict=True)
    ]
    matches.sort(key=lambda row: row["benchmark_district"])
    matched_blocks = sum(row["matched_blocks"] for row in matches)
    total_blocks = len(benchmark)
    return {
        "label_matching": matches,
        "matched_blocks": matched_blocks,
        "moved_blocks": total_blocks - matched_blocks,
        "matched_block_rate": matched_blocks / total_blocks,
    }


def load_nrs_assignments(path: Path) -> dict[str, int]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    require(
        payload.get("schema_version") is not None
        and isinstance(payload.get("assignments"), dict),
        "INPUT",
        "unsupported NRS assignment payload",
    )
    try:
        assignments = {
            str(geoid): int(district)
            for geoid, district in payload["assignments"].items()
        }
    except (TypeError, ValueError) as error:
        raise BakeoffError("[INPUT] invalid NRS district label") from error
    require(assignments, "INPUT", "NRS assignment is empty")
    require(
        all(len(geoid) == 15 and district > 0 for geoid, district in assignments.items()),
        "INPUT",
        "invalid NRS block GEOID or district label",
    )
    return assignments


def project_comparator_assignments(
    block_path: Path,
    comparator_path: Path,
    state_fips: str,
    projection: str,
    comparator_state_column: str,
    comparator_district_column: str,
    comparator_session_column: str,
    expected_session: str,
) -> tuple[dict[str, int], dict]:
    blocks = gpd.read_file(block_path)
    require(
        {"GEOID20", "ALAND20", "geometry"}.issubset(blocks.columns),
        "INPUT",
        "block schema is missing required columns",
    )
    blocks = blocks[["GEOID20", "ALAND20", "geometry"]]
    source_block_count = len(blocks)
    require(
        blocks["GEOID20"].astype(str).nunique() == source_block_count,
        "INPUT",
        "block GEOIDs are not unique",
    )
    blocks, excluded_water_only_blocks = select_land_blocks(blocks)
    require(not blocks.empty, "INPUT", "no land-containing blocks remain")
    if comparator_path.suffix.lower() == ".parquet":
        comparator = gpd.read_parquet(comparator_path)
    else:
        source = (
            f"zip://{comparator_path.resolve().as_posix()}"
            if comparator_path.suffix.lower() == ".zip"
            else comparator_path
        )
        comparator = gpd.read_file(source)
    require(
        {
            comparator_state_column,
            comparator_district_column,
            comparator_session_column,
            "geometry",
        }.issubset(comparator.columns),
        "INPUT",
        "comparator polygon schema is missing required columns",
    )
    comparator = comparator.loc[
        comparator[comparator_state_column].astype(str) == state_fips
    ].copy()
    require(
        not comparator.empty,
        "INPUT",
        f"no comparator polygons for {comparator_state_column}={state_fips}",
    )
    validate_comparator_session(
        comparator[comparator_session_column].astype(str).tolist(),
        expected_session,
    )
    comparator, excluded_district_labels = select_numbered_districts(
        comparator, comparator_district_column
    )
    require(not comparator.empty, "INPUT", "no numbered comparator districts remain")
    comparator["district"] = comparator[comparator_district_column].astype(int)

    block_points = blocks.to_crs(projection)
    block_points["geometry"] = block_points.geometry.representative_point()
    comparator = comparator[["district", "geometry"]].to_crs(projection)
    joined = gpd.sjoin(block_points, comparator, how="left", predicate="within")
    counts = joined.groupby("GEOID20", sort=False)["district"].count()
    unmatched = counts[counts == 0].index.tolist()
    multiply_matched = counts[counts > 1].index.tolist()
    require(not unmatched, "PROJECTION", f"{len(unmatched)} blocks matched no comparator polygon")
    require(
        not multiply_matched,
        "PROJECTION",
        f"{len(multiply_matched)} blocks matched multiple comparator polygons",
    )
    assignments = {
        str(row.GEOID20): int(row.district)
        for row in joined[["GEOID20", "district"]].itertuples(index=False)
    }
    return assignments, {
        "method": "block-representative-point-within-exactly-one-polygon",
        "projection": projection,
        "comparator_session": expected_session,
        "source_blocks": source_block_count,
        "excluded_water_only_blocks": excluded_water_only_blocks,
        "excluded_non_district_labels": excluded_district_labels,
        "unmatched_blocks": 0,
        "multiply_matched_blocks": 0,
    }


def write_package(
    root: Path,
    package_dir: Path,
    state: str,
    year: int,
    state_fips: str,
    projection: str,
    nrs_path: Path,
    block_path: Path,
    comparator_path: Path,
    comparator_state_column: str,
    comparator_district_column: str,
    comparator_session_column: str,
    expected_session: str,
    display_output_dir: str | None = None,
) -> None:
    benchmark_source = load_nrs_assignments(nrs_path)
    comparator, projection_diagnostics = project_comparator_assignments(
        block_path,
        comparator_path,
        state_fips,
        projection,
        comparator_state_column,
        comparator_district_column,
        comparator_session_column,
        expected_session,
    )
    require(
        len(benchmark_source) == projection_diagnostics["source_blocks"],
        "INPUT",
        "NRS and source block counts differ",
    )
    require(
        set(comparator).issubset(benchmark_source),
        "INPUT",
        "land-containing comparator block universe is not covered by NRS",
    )
    benchmark = {geoid: benchmark_source[geoid] for geoid in comparator}
    overlap = optimal_overlap(benchmark, comparator)
    benchmark_counties = summarize_splits(benchmark, 5)
    benchmark_tracts = summarize_splits(benchmark, 11)
    comparator_counties = summarize_splits(comparator, 5)
    comparator_tracts = summarize_splits(comparator, 11)

    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "pass",
        "state": state.upper(),
        "state_fips": state_fips,
        "census_year": year,
        "atomic_unit": f"{year}-census-tabulation-block",
        "atomic_universe": {
            "source_blocks": projection_diagnostics["source_blocks"],
            "excluded_water_only_blocks": projection_diagnostics[
                "excluded_water_only_blocks"
            ],
            "analyzed_land_containing_blocks": len(benchmark),
        },
        "benchmark": {
            "plan_family": "NRS-v0.3",
            "blocks": len(benchmark),
            "districts": len(set(benchmark.values())),
            "county_splits": benchmark_counties,
            "tract_splits": benchmark_tracts,
        },
        "comparator": {
            "plan_family": f"enacted-congressional-session-{expected_session}",
            "blocks": len(comparator),
            "districts": len(set(comparator.values())),
            "county_splits": comparator_counties,
            "tract_splits": comparator_tracts,
            "projection_diagnostics": projection_diagnostics,
        },
        "comparison": {
            **overlap,
            "comparator_minus_benchmark": {
                "county_split_units": comparator_counties["split_units"]
                - benchmark_counties["split_units"],
                "county_excess_pieces": comparator_counties["excess_pieces"]
                - benchmark_counties["excess_pieces"],
                "tract_split_units": comparator_tracts["split_units"]
                - benchmark_tracts["split_units"],
                "tract_excess_pieces": comparator_tracts["excess_pieces"]
                - benchmark_tracts["excess_pieces"],
            },
        },
        "unavailable_metrics": {
            "population": "No comparator block-population allocation is frozen in Tier 1.",
            "geometric_compactness": "The national geometry/perimeter protocol is not frozen.",
            "partisan": "The evaluation schedule election and precinct-to-block inputs are not frozen.",
            "demographic_and_vra": "No frozen within-unit demographic allocation or legal analysis.",
            "sensitivity": "The 100 diagnostic seed package has not been run.",
            "ensemble": "No block-level preregistered ensemble exists for this slice.",
        },
        "claim_boundary": CLAIM_BOUNDARY,
    }

    package_dir.mkdir(parents=True, exist_ok=True)
    analysis_path = package_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8")
    canonical_output_dir = display_output_dir or (
        f"docs/experiments/nrs-v0.3-bakeoff-{state.lower()}-{year}"
    )
    readme = f"""# NRS v0.3 {state.upper()} {year} Tier 1 Bakeoff

**Status:** governed accepted evidence slice

| Measure | NRS v0.3 | Enacted CD{expected_session} comparator | Comparator minus NRS |
|---|---:|---:|---:|
| Blocks | {len(benchmark):,} | {len(comparator):,} | 0 |
| County split units | {benchmark_counties['split_units']} | {comparator_counties['split_units']} | {comparator_counties['split_units'] - benchmark_counties['split_units']:+d} |
| Tract split units | {benchmark_tracts['split_units']} | {comparator_tracts['split_units']} | {comparator_tracts['split_units'] - benchmark_tracts['split_units']:+d} |

After maximum-overlap district-label matching, {overlap['matched_blocks']:,} of
{len(benchmark):,} blocks match ({overlap['matched_block_rate']:.6%}); the
remaining {overlap['moved_blocks']:,} blocks are assigned to different
districts.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_slice.py --state RI --state-fips 44 `
  --year 2020 --projection EPSG:32130 `
  --nrs-assignment {relative_path(nrs_path, root)} `
  --block-shapefile {relative_path(block_path, root)} `
  --comparator-source {relative_path(comparator_path, root)} `
  --comparator-state-column {comparator_state_column} `
  --comparator-district-column {comparator_district_column} `
  --comparator-session-column {comparator_session_column} `
  --expected-session {expected_session} `
  --output-dir {canonical_output_dir}
python scripts/research/verify_nrs_bakeoff_slice.py {canonical_output_dir}
```

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = package_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8")

    sidecars = [
        block_path,
        block_path.with_suffix(".shx"),
        block_path.with_suffix(".dbf"),
        block_path.with_suffix(".prj"),
        block_path.with_suffix(".cpg"),
    ]
    for sidecar in sidecars:
        require(sidecar.is_file(), "INPUT", f"missing shapefile sidecar {sidecar}")
    input_paths = [nrs_path, *sidecars, comparator_path]
    code_paths = [PROTOCOL_PATH, ANALYZER_PATH, VERIFIER_PATH]
    manifest = {
        "schema_version": MANIFEST_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "pass",
        "state": state.upper(),
        "census_year": year,
        "inputs": [
            {"path": relative_path(path, root), "sha256": sha256(path)}
            for path in input_paths
        ],
        "code": [
            {"path": relative_path(path, root), "sha256": sha256(path)}
            for path in code_paths
        ],
        "outputs": {
            "analysis.json": sha256(analysis_path),
            "README.md": sha256(readme_path),
        },
        "reproduction": {
            "state_fips": state_fips,
            "projection": projection,
            "comparator_state_column": comparator_state_column,
            "comparator_district_column": comparator_district_column,
            "comparator_session_column": comparator_session_column,
            "expected_session": expected_session,
            "display_output_dir": canonical_output_dir,
            "nrs_assignment": relative_path(nrs_path, root),
            "block_shapefile": relative_path(block_path, root),
            "comparator_source": relative_path(comparator_path, root),
        },
        "claim_boundary": CLAIM_BOUNDARY,
    }
    (package_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--state", required=True)
    parser.add_argument("--state-fips", required=True)
    parser.add_argument("--year", type=int, required=True)
    parser.add_argument("--projection", required=True)
    parser.add_argument("--nrs-assignment", type=Path, required=True)
    parser.add_argument("--block-shapefile", type=Path, required=True)
    parser.add_argument("--comparator-source", type=Path, required=True)
    parser.add_argument("--comparator-state-column", required=True)
    parser.add_argument("--comparator-district-column", required=True)
    parser.add_argument("--comparator-session-column", required=True)
    parser.add_argument("--expected-session", required=True)
    parser.add_argument("--display-output-dir")
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    root = Path.cwd()
    write_package(
        root=root,
        package_dir=args.output_dir,
        state=args.state,
        year=args.year,
        state_fips=args.state_fips,
        projection=args.projection,
        nrs_path=args.nrs_assignment,
        block_path=args.block_shapefile,
        comparator_path=args.comparator_source,
        comparator_state_column=args.comparator_state_column,
        comparator_district_column=args.comparator_district_column,
        comparator_session_column=args.comparator_session_column,
        expected_session=args.expected_session,
        display_output_dir=args.display_output_dir,
    )


if __name__ == "__main__":
    main()
