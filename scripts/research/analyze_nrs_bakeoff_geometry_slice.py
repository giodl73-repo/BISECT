#!/usr/bin/env python3
"""Build a governed Tier 2 geometry bakeoff evidence slice."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

import geopandas as gpd
import numpy as np
import pandas as pd
import shapely


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "research"))

from analyze_nrs_bakeoff_slice import (
    BakeoffError,
    load_nrs_assignments,
    project_comparator_assignments,
    relative_path,
    require,
    select_land_blocks,
)


SCHEMA_VERSION = "nrs-v0.3-bakeoff-geometry-slice-analysis-v1"
MANIFEST_VERSION = "nrs-v0.3-bakeoff-geometry-slice-manifest-v1"
PROTOCOL_ID = "nrs-v0.3-tier2-geometry-v1"
PROTOCOL_PATH = Path("docs/specs/2026-08-08-nrs-v0.3-tier2-geometry-protocol.md")
ANALYZER_PATH = Path("scripts/research/analyze_nrs_bakeoff_geometry_slice.py")
VERIFIER_PATH = Path("scripts/research/verify_nrs_bakeoff_geometry_slice.py")
TIER1_ANALYZER_PATH = Path("scripts/research/analyze_nrs_bakeoff_slice.py")
CLAIM_BOUNDARY = (
    "Descriptive compactness measurements of Rhode Island block-projected NRS "
    "v0.3 and enacted CD118 assignments under one frozen geometry contract; no "
    "compactness superiority, fairness, intent, VRA, legal-validity, community, "
    "robustness, optimality, or adoption claim."
)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def finite_positive(value: float, name: str) -> float:
    require(math.isfinite(value) and value > 0.0, "GEOMETRY", f"invalid {name}: {value}")
    return value


def unit_interval(value: float, name: str) -> float:
    require(math.isfinite(value), "GEOMETRY", f"non-finite {name}: {value}")
    require(-1e-12 <= value <= 1.0 + 1e-12, "GEOMETRY", f"{name} out of range: {value}")
    return min(1.0, max(0.0, value))


def measure_geometry(geometry) -> dict:
    require(geometry is not None and not shapely.is_empty(geometry), "GEOMETRY", "empty district geometry")
    require(bool(shapely.is_valid(geometry)), "GEOMETRY", "invalid district geometry")
    area = finite_positive(float(shapely.area(geometry)), "area")
    perimeter = finite_positive(float(shapely.length(geometry)), "perimeter")
    radius = finite_positive(
        float(shapely.minimum_bounding_radius(geometry)),
        "minimum bounding radius",
    )
    hull_area = finite_positive(float(shapely.area(shapely.convex_hull(geometry))), "convex hull area")
    polsby_popper = unit_interval(4.0 * math.pi * area / (perimeter * perimeter), "Polsby-Popper")
    reock = unit_interval(area / (math.pi * radius * radius), "Reock")
    convex_hull_ratio = unit_interval(area / hull_area, "convex hull ratio")
    schwartzberg = perimeter / (2.0 * math.sqrt(math.pi * area))
    require(
        math.isfinite(schwartzberg) and schwartzberg >= 1.0 - 1e-12,
        "GEOMETRY",
        f"Schwartzberg out of range: {schwartzberg}",
    )
    identity = 1.0 / math.sqrt(polsby_popper)
    require(
        abs(schwartzberg - identity) <= 1e-12,
        "GEOMETRY",
        "Schwartzberg identity mismatch",
    )
    return {
        "area_m2": area,
        "perimeter_m": perimeter,
        "minimum_bounding_radius_m": radius,
        "polsby_popper": polsby_popper,
        "reock": reock,
        "convex_hull_ratio": convex_hull_ratio,
        "schwartzberg": schwartzberg,
        "component_count": len(geometry.geoms)
        if geometry.geom_type == "MultiPolygon"
        else 1,
        "interior_ring_count": sum(
            len(polygon.interiors)
            for polygon in (
                geometry.geoms if geometry.geom_type == "MultiPolygon" else [geometry]
            )
        ),
    }


def dissolve_assignments(
    blocks: gpd.GeoDataFrame, assignments: dict[str, int]
) -> list[dict]:
    geoids = blocks["GEOID20"].astype(str)
    require(geoids.is_unique, "INPUT", "block GEOIDs are not unique")
    require(set(geoids) == set(assignments), "INPUT", "assignment and geometry universes differ")
    require(
        bool(np.all(~shapely.is_empty(blocks.geometry.array))),
        "GEOMETRY",
        "empty retained block geometry",
    )
    require(
        bool(np.all(shapely.is_valid(blocks.geometry.array))),
        "GEOMETRY",
        "invalid retained block geometry",
    )
    table = blocks[["GEOID20", "geometry"]].copy()
    table["district"] = table["GEOID20"].astype(str).map(assignments)
    rows = []
    for district, group in table.groupby("district", sort=True):
        geometry = shapely.union_all(group.geometry.array)
        rows.append(
            {
                "district": int(district),
                "block_count": len(group),
                **measure_geometry(geometry),
            }
        )
    require(rows, "INPUT", "no districts were dissolved")
    return rows


def summarize_plan(rows: list[dict], plan_family: str) -> dict:
    metric_names = (
        "polsby_popper",
        "reock",
        "convex_hull_ratio",
        "schwartzberg",
    )
    return {
        "plan_family": plan_family,
        "districts": len(rows),
        "district_metrics": rows,
        "unweighted_mean": {
            name: sum(row[name] for row in rows) / len(rows) for name in metric_names
        },
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
    blocks = gpd.read_file(block_path)[["GEOID20", "ALAND20", "geometry"]]
    source_block_count = len(blocks)
    blocks, excluded_water_only_blocks = select_land_blocks(blocks)
    blocks["GEOID20"] = blocks["GEOID20"].astype(str)
    blocks = blocks.to_crs(projection).sort_values("GEOID20").reset_index(drop=True)
    require(
        source_block_count == projection_diagnostics["source_blocks"],
        "INPUT",
        "Tier 1 and geometry source block counts differ",
    )
    require(
        excluded_water_only_blocks
        == projection_diagnostics["excluded_water_only_blocks"],
        "INPUT",
        "Tier 1 and geometry water exclusions differ",
    )
    require(set(comparator).issubset(benchmark_source), "INPUT", "NRS does not cover comparator universe")
    benchmark = {geoid: benchmark_source[geoid] for geoid in comparator}
    benchmark_rows = dissolve_assignments(blocks, benchmark)
    comparator_rows = dissolve_assignments(blocks, comparator)
    require(
        len(benchmark_rows) == len(comparator_rows) == 2,
        "INPUT",
        "Rhode Island district count is not two",
    )
    benchmark_summary = summarize_plan(benchmark_rows, "NRS-v0.3")
    comparator_summary = summarize_plan(
        comparator_rows, f"enacted-congressional-session-{expected_session}"
    )
    metric_names = (
        "polsby_popper",
        "reock",
        "convex_hull_ratio",
        "schwartzberg",
    )
    differences = {
        name: comparator_summary["unweighted_mean"][name]
        - benchmark_summary["unweighted_mean"][name]
        for name in metric_names
    }
    analysis = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "status": "pass",
        "state": state.upper(),
        "state_fips": state_fips,
        "census_year": year,
        "geometry_contract": {
            "atomic_unit": f"{year}-census-tabulation-block",
            "projection": projection,
            "source_blocks": source_block_count,
            "excluded_water_only_blocks": excluded_water_only_blocks,
            "retained_blocks": len(blocks),
            "dissolve": "GEOS unary union by complete block assignment",
            "perimeter": "complete dissolved boundary including multipart components and interior rings",
            "repair": "none",
            "comparator_geometry": "Tier 1 block projection, not original district polygon linework",
        },
        "benchmark": benchmark_summary,
        "comparator": comparator_summary,
        "comparator_minus_benchmark_unweighted_mean": differences,
        "unavailable_metrics": {
            "population": "Not part of the Tier 2 geometry contract.",
            "partisan": "Named elections and precinct-to-block inputs are not frozen.",
            "demographic_and_vra": "No frozen within-unit demographic allocation or legal analysis.",
            "ensemble": "No converged block-level ensemble exists for this comparison.",
        },
        "claim_boundary": CLAIM_BOUNDARY,
    }

    package_dir.mkdir(parents=True, exist_ok=True)
    analysis_path = package_dir / "analysis.json"
    analysis_path.write_text(json.dumps(analysis, indent=2) + "\n", encoding="utf-8", newline="\n")
    canonical_output_dir = display_output_dir or (
        f"docs/experiments/nrs-v0.3-bakeoff-geometry-{state.lower()}-{year}"
    )
    readme = f"""# NRS v0.3 {state.upper()} {year} Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD{expected_session} block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | {benchmark_summary['unweighted_mean']['polsby_popper']:.9f} | {comparator_summary['unweighted_mean']['polsby_popper']:.9f} | {differences['polsby_popper']:+.9f} |
| Exact Reock | {benchmark_summary['unweighted_mean']['reock']:.9f} | {comparator_summary['unweighted_mean']['reock']:.9f} | {differences['reock']:+.9f} |
| Convex-hull ratio | {benchmark_summary['unweighted_mean']['convex_hull_ratio']:.9f} | {comparator_summary['unweighted_mean']['convex_hull_ratio']:.9f} | {differences['convex_hull_ratio']:+.9f} |
| Schwartzberg | {benchmark_summary['unweighted_mean']['schwartzberg']:.9f} | {comparator_summary['unweighted_mean']['schwartzberg']:.9f} | {differences['schwartzberg']:+.9f} |

Both plans are dissolved from the same {len(blocks):,} retained Census block
polygons in `{projection}`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state {state.upper()} `
  --state-fips {state_fips} --year {year} --projection {projection} `
  --nrs-assignment {relative_path(nrs_path, root)} `
  --block-shapefile {relative_path(block_path, root)} `
  --comparator-source {relative_path(comparator_path, root)} `
  --comparator-state-column {comparator_state_column} `
  --comparator-district-column {comparator_district_column} `
  --comparator-session-column {comparator_session_column} `
  --expected-session {expected_session} --output-dir {canonical_output_dir}
python scripts/research/verify_nrs_bakeoff_geometry_slice.py {canonical_output_dir}
```

## Claim Boundary

{CLAIM_BOUNDARY}
"""
    readme_path = package_dir / "README.md"
    readme_path.write_text(readme, encoding="utf-8", newline="\n")

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
    code_paths = [PROTOCOL_PATH, ANALYZER_PATH, VERIFIER_PATH, TIER1_ANALYZER_PATH]
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
            "state": state.upper(),
            "state_fips": state_fips,
            "year": year,
            "projection": projection,
            "nrs_assignment": relative_path(nrs_path, root),
            "block_shapefile": relative_path(block_path, root),
            "comparator_source": relative_path(comparator_path, root),
            "comparator_state_column": comparator_state_column,
            "comparator_district_column": comparator_district_column,
            "comparator_session_column": comparator_session_column,
            "expected_session": expected_session,
            "display_output_dir": canonical_output_dir,
        },
        "claim_boundary": CLAIM_BOUNDARY,
    }
    (package_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
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
    write_package(
        root=Path.cwd(),
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
