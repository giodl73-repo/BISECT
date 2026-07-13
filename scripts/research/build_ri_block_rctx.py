#!/usr/bin/env python3
"""Build and verify the RI 2020 land-boundary block RCTX frontier."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
SCHEMA = "certified-recursive-ri-block-rctx-frontier-v1"
SCRIPT_PATH = Path("scripts/research/build_ri_block_rctx.py")
SHAPEFILE = Path(
    "data/2020/tiger/blocks/tl_2020_44_tabblock20/tl_2020_44_tabblock20.shp"
)
PL_GEO = Path("data/2020/redistricting/ri2020.pl/rigeo2020.pl")
PL_POP = Path("data/2020/redistricting/ri2020.pl/ri000012020.pl")
BRIDGE_RULE_SOURCE = Path("crates/bisect-data/src/bridge.rs")
BRIDGE_WEIGHT_SOURCE = Path("crates/bisect-data/src/adjacency.rs")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_hash(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def components(adjacency: list[list[dict[str, Any]]]) -> list[list[int]]:
    seen: set[int] = set()
    result = []
    for start in range(len(adjacency)):
        if start in seen:
            continue
        stack = [start]
        seen.add(start)
        component = []
        while stack:
            unit = stack.pop()
            component.append(unit)
            for edge in adjacency[unit]:
                neighbor = edge["to"]
                if neighbor not in seen:
                    seen.add(neighbor)
                    stack.append(neighbor)
        result.append(sorted(component))
    return sorted(result, key=len, reverse=True)


def add_island_bridges(
    adjacency: list[list[dict[str, Any]]],
    geometries: Any,
    unit_ids: list[str],
    land_weights: list[int],
) -> list[dict[str, Any]]:
    graph_components = components(adjacency)
    if len(graph_components) <= 1:
        return []
    main = graph_components[0]
    centroids = [(point.x, point.y) for point in geometries.centroid]
    median_land_weight = sorted(land_weights)[len(land_weights) // 2]
    records = []
    for component in graph_components[1:]:
        for island_index in component:
            county = unit_ids[island_index][:5]
            same_county = [index for index in main if unit_ids[index][:5] == county]
            candidates = same_county or main
            x0, y0 = centroids[island_index]
            mainland_index = min(
                candidates,
                key=lambda index: (
                    (centroids[index][0] - x0) ** 2 + (centroids[index][1] - y0) ** 2,
                    index,
                ),
            )
            distance = (
                (centroids[mainland_index][0] - x0) ** 2
                + (centroids[mainland_index][1] - y0) ** 2
            ) ** 0.5
            adjacency[island_index].append(
                {
                    "to": mainland_index,
                    "kind": "bridge",
                    "weight": float(median_land_weight),
                }
            )
            adjacency[mainland_index].append(
                {
                    "to": island_index,
                    "kind": "bridge",
                    "weight": float(median_land_weight),
                }
            )
            records.append(
                {
                    "island_geoid": unit_ids[island_index],
                    "mainland_geoid": unit_ids[mainland_index],
                    "same_county": bool(same_county),
                    "centroid_distance_meters": round(distance, 3),
                    "weight_millimeters": median_land_weight,
                }
            )
    for neighbors in adjacency:
        neighbors.sort(key=lambda edge: edge["to"])
    return records


def build(rctx_path: Path, report_path: Path, manifest_path: Path) -> None:
    import geopandas as gpd
    import pyproj
    import shapely

    from scripts.data.census.parse_pl94171_blocks_2020 import (
        parse_2020_geo_file,
        parse_2020_pop_file,
    )

    tiger = (
        gpd.read_file(ROOT / SHAPEFILE)
        .sort_values("GEOID20")
        .reset_index(drop=True)
        .to_crs(5070)
    )
    pl_geo = parse_2020_geo_file(ROOT / PL_GEO)
    pl_pop = parse_2020_pop_file(ROOT / PL_POP)
    population = (
        pl_geo.merge(pl_pop, on="LOGRECNO", how="left")
        .assign(GEOID=lambda frame: frame["GEOID"].str.removeprefix("7500000US"))
        .set_index("GEOID")["POP100"]
        .astype(int)
        .to_dict()
    )
    unit_ids = tiger["GEOID20"].astype(str).tolist()
    missing_population = sorted(set(unit_ids) - set(population))
    extra_population = sorted(set(population) - set(unit_ids))
    if missing_population or extra_population:
        raise SystemExit(
            "TIGER/PL block GEOID mismatch: "
            f"{len(missing_population)} missing populations, "
            f"{len(extra_population)} extra population records"
        )
    populations = [population[unit_id] for unit_id in unit_ids]
    boundaries = tiger.geometry.boundary
    candidate_pairs = tiger.sindex.query(tiger.geometry, predicate="intersects")
    adjacency: list[list[dict[str, Any]]] = [[] for _ in unit_ids]
    edge_count = 0
    boundary_millimeters = 0
    land_weights = []
    for left, right in zip(candidate_pairs[0], candidate_pairs[1], strict=True):
        left = int(left)
        right = int(right)
        if left >= right:
            continue
        shared_length = float(boundaries.iloc[left].intersection(boundaries.iloc[right]).length)
        if shared_length <= 1e-6:
            continue
        weight = max(1, round(shared_length * 1000))
        adjacency[left].append({"to": right, "kind": "boundary", "weight": float(weight)})
        adjacency[right].append({"to": left, "kind": "boundary", "weight": float(weight)})
        edge_count += 1
        boundary_millimeters += weight
        land_weights.append(weight)
    for neighbors in adjacency:
        neighbors.sort(key=lambda edge: edge["to"])

    land_components = components(adjacency)
    bridge_records = add_island_bridges(
        adjacency, tiger.geometry, unit_ids, land_weights
    )
    final_components = components(adjacency)

    unit_index = {
        "unit_kind": "block",
        "state": "RI",
        "year": 2020,
        "canonical_order": "sorted-geoid",
        "unit_ids": unit_ids,
        "source_id": "ri-2020-tiger-pl-block-county-bridged-adjacency",
    }
    unit_index["unit_universe_hash"] = canonical_hash(unit_index)
    source_hashes = {
        "tiger_block_shp": f"sha256:{sha256(ROOT / SHAPEFILE)}",
        "tiger_block_dbf": f"sha256:{sha256((ROOT / SHAPEFILE).with_suffix('.dbf'))}",
        "tiger_block_shx": f"sha256:{sha256((ROOT / SHAPEFILE).with_suffix('.shx'))}",
        "pl_geo": f"sha256:{sha256(ROOT / PL_GEO)}",
        "pl_population": f"sha256:{sha256(ROOT / PL_POP)}",
        "bridge_rule_source": f"sha256:{sha256(ROOT / BRIDGE_RULE_SOURCE)}",
        "bridge_weight_rule_source": f"sha256:{sha256(ROOT / BRIDGE_WEIGHT_SOURCE)}",
    }
    graph = {"edge_semantics": "undirected", "adjacency": adjacency}
    context_projection = {
        "units": unit_index,
        "graph": graph,
        "populations": populations,
        "source_hashes": source_hashes,
    }
    rctx = {
        "rctx_version": "0.1",
        "context_hash": canonical_hash(context_projection),
        **context_projection,
    }
    rctx_path.parent.mkdir(parents=True, exist_ok=True)
    rctx_path.write_text(
        json.dumps(rctx, ensure_ascii=False, separators=(",", ":")),
        encoding="utf-8",
    )

    component_rows = [
        {
            "unit_count": len(component),
            "population": sum(populations[index] for index in component),
            "first_geoid": unit_ids[component[0]],
            "last_geoid": unit_ids[component[-1]],
        }
        for component in land_components
    ]

    report = {
        "schema_version": SCHEMA,
        "status": "blocked",
        "state": "rhode_island",
        "year": 2020,
        "districts": 2,
        "rctx": {
            "path": rctx_path.relative_to(ROOT).as_posix(),
            "bytes": rctx_path.stat().st_size,
            "sha256": sha256(rctx_path),
            "context_hash": rctx["context_hash"],
            "unit_universe_hash": unit_index["unit_universe_hash"],
            "committed": False,
        },
        "source_hashes": source_hashes,
        "geometry_toolchain": {
            "geopandas": gpd.__version__,
            "shapely": shapely.__version__,
            "geos": shapely.geos_version_string,
            "pyproj": pyproj.__version__,
            "proj": pyproj.proj_version_str,
            "crs": "EPSG:5070",
        },
        "graph": {
            "unit_count": len(unit_ids),
            "land_edge_count": edge_count,
            "bridge_edge_count": len(bridge_records),
            "final_edge_count": edge_count + len(bridge_records),
            "total_boundary_weight_millimeters": boundary_millimeters,
            "land_component_count": len(land_components),
            "land_components": component_rows,
            "final_component_count": len(final_components),
            "bridge_rule": {
                "algorithm": (
                    "each non-main-component unit connects to nearest same-county "
                    "main-component unit; fallback to nearest main-component unit"
                ),
                "weight_rule": "median land-boundary weight",
                "source_path": BRIDGE_RULE_SOURCE.as_posix(),
                "source_sha256": sha256(ROOT / BRIDGE_RULE_SOURCE),
                "weight_source_path": BRIDGE_WEIGHT_SOURCE.as_posix(),
                "weight_source_sha256": sha256(ROOT / BRIDGE_WEIGHT_SOURCE),
            },
            "bridge_records": bridge_records,
            "adjacency_rule": "positive shared boundary length after EPSG:5070 projection",
        },
        "population": {
            "total": sum(populations),
            "positive_blocks": sum(value > 0 for value in populations),
            "zero_blocks": sum(value == 0 for value in populations),
            "theoretical_minimum_scaled_deviation": sum(populations) % 2,
        },
        "proof_frontier": {
            "bounded_unit_limit": 24,
            "state_unit_count": len(unit_ids),
            "opb_assignment_variables": len(unit_ids),
            "opb_cut_variables": edge_count + len(bridge_records),
            "opb_total_variables_before_connectivity_encoding": len(unit_ids)
            + edge_count
            + len(bridge_records),
            "static_connectivity_nogood_formula": (
                f"up to 2^{len(unit_ids) - 1} canonical equal-seat assignments"
            ),
            "roundingsat_available": shutil.which("roundingsat") is not None,
            "veripb_available": shutil.which("veripb") is not None,
            "external_proof_smoke_verified": (
                ROOT / "docs/examples/proof-toolchain-smoke/manifest.json"
            ).is_file(),
            "discovery_solver_availability": {
                "scip": shutil.which("scip") is not None,
                "highs": shutil.which("highs") is not None,
            },
            "compact_parent_depth": {
                "depth_bits": (len(unit_ids) - 1).bit_length(),
                "boolean_variables": (
                    len(unit_ids)
                    + edge_count
                    + len(bridge_records)
                    + 2 * len(unit_ids)
                    + 4 * (edge_count + len(bridge_records))
                    + 2 * len(unit_ids) * (len(unit_ids) - 1).bit_length()
                ),
                "base_constraints": (
                    3
                    + 4 * (edge_count + len(bridge_records))
                    + 2
                    * (
                        1
                        + 4 * len(unit_ids)
                        + 6 * (edge_count + len(bridge_records))
                    )
                ),
                "encoding": "one root per child, one parent per non-root unit, binary acyclic depth",
                "bounded_smoke_proof_verified": True,
            },
        },
        "blockers": [
            "compact parent/depth model is polynomial but still requires production-scale solver integration",
            "no production discovery solver is installed",
            "proof toolchain is smoke-tested but not integrated into the production runner",
        ],
        "claim_boundary": (
            "Local uncommitted county-bridged RCTX custody using the established BISECT "
            "island rule and a quantified proof frontier; not a certified split or scalable proof."
        ),
    }
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-recursive-ri-frontier-package-v1",
        "package_id": "ri-2020-certified-root-frontier",
        "status": "blocked",
        "files": [
            {"path": report_path.name, "sha256": sha256(report_path)}
        ],
        "builder_path": SCRIPT_PATH.as_posix(),
        "builder_sha256": sha256(ROOT / SCRIPT_PATH),
        "verification_commands": [
            (
                "python scripts/research/build_ri_block_rctx.py verify "
                f"{manifest_path.relative_to(ROOT).as_posix()}"
            ),
            (
                "python scripts/research/build_ri_block_rctx.py verify "
                f"{manifest_path.relative_to(ROOT).as_posix()} --check-rctx"
            ),
        ],
        "claim_boundary": report["claim_boundary"],
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print("RI block RCTX frontier build: BLOCKED")


def verify(manifest_path: Path, check_rctx: bool) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["schema_version"] != "certified-recursive-ri-frontier-package-v1":
        raise SystemExit("unsupported RI certified frontier package")
    if manifest["builder_sha256"] != sha256(ROOT / manifest["builder_path"]):
        raise SystemExit("RI frontier builder hash mismatch")
    report_path = manifest_path.parent / manifest["files"][0]["path"]
    if manifest["files"][0]["sha256"] != sha256(report_path):
        raise SystemExit("RI frontier report hash mismatch")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if report["schema_version"] != SCHEMA or report["status"] != "blocked":
        raise SystemExit("RI frontier report posture drift")
    if (
        report["graph"]["unit_count"] != 25_649
        or report["graph"]["land_component_count"] != 2
        or report["graph"]["final_component_count"] != 1
    ):
        raise SystemExit("RI block graph frontier drift")
    if sum(row["unit_count"] for row in report["graph"]["land_components"]) != 25_649:
        raise SystemExit("RI component unit-count mismatch")
    if sum(row["population"] for row in report["graph"]["land_components"]) != 1_097_379:
        raise SystemExit("RI component population mismatch")
    if report["graph"]["bridge_edge_count"] != 64:
        raise SystemExit("RI bridge-edge count mismatch")
    if check_rctx:
        rctx_path = ROOT / report["rctx"]["path"]
        if not rctx_path.is_file() or sha256(rctx_path) != report["rctx"]["sha256"]:
            raise SystemExit("RI local RCTX custody mismatch")
        rctx = json.loads(rctx_path.read_text(encoding="utf-8"))
        projection = {
            key: rctx[key]
            for key in ("units", "graph", "populations", "source_hashes")
        }
        if rctx["context_hash"] != canonical_hash(projection):
            raise SystemExit("RI local RCTX context hash mismatch")
    print("RI block RCTX frontier verification: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    build_parser = subparsers.add_parser("build")
    build_parser.add_argument("--rctx", type=Path, required=True)
    build_parser.add_argument("--report", type=Path, required=True)
    build_parser.add_argument("--manifest", type=Path, required=True)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("manifest", type=Path)
    verify_parser.add_argument("--check-rctx", action="store_true")
    args = parser.parse_args()
    if args.command == "build":
        build(ROOT / args.rctx, ROOT / args.report, ROOT / args.manifest)
    else:
        verify(ROOT / args.manifest, args.check_rctx)


if __name__ == "__main__":
    main()
