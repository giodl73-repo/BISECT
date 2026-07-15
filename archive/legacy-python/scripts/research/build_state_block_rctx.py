#!/usr/bin/env python3
"""Build a connected 2020 block RCTX for a parameterized State."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

from build_ri_block_rctx import add_island_bridges, canonical_hash, components


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path("scripts/research/build_state_block_rctx.py")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(
    state_code: str,
    state_fips: str,
    state_name: str,
    rctx_path: Path,
    report_path: Path,
    manifest_path: Path,
) -> None:
    import geopandas as gpd
    import pyproj
    import shapely

    from scripts.data.census.parse_pl94171_blocks_2020 import (
        parse_2020_geo_file,
        parse_2020_pop_file,
    )

    lower = state_code.lower()
    shape = Path(
        f"data/2020/tiger/blocks/tl_2020_{state_fips}_tabblock20/"
        f"tl_2020_{state_fips}_tabblock20.shp"
    )
    pl_dir = Path(f"data/2020/redistricting/{lower}2020.pl")
    pl_geo_path = pl_dir / f"{lower}geo2020.pl"
    pl_pop_path = pl_dir / f"{lower}000012020.pl"
    bridge_source = Path("crates/bisect-data/src/bridge.rs")
    weight_source = Path("crates/bisect-data/src/adjacency.rs")

    tiger = (
        gpd.read_file(ROOT / shape)
        .sort_values("GEOID20")
        .reset_index(drop=True)
        .to_crs(5070)
    )
    pl_geo = parse_2020_geo_file(ROOT / pl_geo_path)
    pl_pop = parse_2020_pop_file(ROOT / pl_pop_path)
    population = (
        pl_geo.merge(pl_pop, on="LOGRECNO", how="left")
        .assign(GEOID=lambda frame: frame["GEOID"].str.removeprefix("7500000US"))
        .set_index("GEOID")["POP100"]
        .astype(int)
        .to_dict()
    )
    unit_ids = tiger["GEOID20"].astype(str).tolist()
    if set(unit_ids) != set(population):
        raise SystemExit("TIGER/PL block GEOID mismatch")
    populations = [population[unit_id] for unit_id in unit_ids]
    boundaries = tiger.geometry.boundary
    candidate_pairs = tiger.sindex.query(tiger.geometry, predicate="intersects")
    adjacency: list[list[dict[str, Any]]] = [[] for _ in unit_ids]
    land_weights = []
    for left, right in zip(candidate_pairs[0], candidate_pairs[1], strict=True):
        left, right = int(left), int(right)
        if left >= right:
            continue
        shared = float(boundaries.iloc[left].intersection(boundaries.iloc[right]).length)
        if shared <= 1e-6:
            continue
        weight = max(1, round(shared * 1000))
        adjacency[left].append(
            {"to": right, "kind": "boundary", "weight": float(weight)}
        )
        adjacency[right].append(
            {"to": left, "kind": "boundary", "weight": float(weight)}
        )
        land_weights.append(weight)
    for neighbors in adjacency:
        neighbors.sort(key=lambda edge: edge["to"])
    land_components = components(adjacency)
    bridges = add_island_bridges(adjacency, tiger.geometry, unit_ids, land_weights)
    final_components = components(adjacency)
    if len(final_components) != 1:
        raise SystemExit("State block graph remains disconnected")

    units = {
        "unit_kind": "block",
        "state": state_code,
        "year": 2020,
        "canonical_order": "sorted-geoid",
        "unit_ids": unit_ids,
        "source_id": f"{lower}-2020-tiger-pl-block-county-bridged-adjacency",
    }
    units["unit_universe_hash"] = canonical_hash(units)
    source_hashes = {
        "tiger_block_shp": f"sha256:{sha256(ROOT / shape)}",
        "tiger_block_dbf": f"sha256:{sha256((ROOT / shape).with_suffix('.dbf'))}",
        "tiger_block_shx": f"sha256:{sha256((ROOT / shape).with_suffix('.shx'))}",
        "pl_geo": f"sha256:{sha256(ROOT / pl_geo_path)}",
        "pl_population": f"sha256:{sha256(ROOT / pl_pop_path)}",
        "bridge_rule_source": f"sha256:{sha256(ROOT / bridge_source)}",
        "bridge_weight_rule_source": f"sha256:{sha256(ROOT / weight_source)}",
    }
    projection = {
        "units": units,
        "graph": {"edge_semantics": "undirected", "adjacency": adjacency},
        "populations": populations,
        "source_hashes": source_hashes,
    }
    rctx = {
        "rctx_version": "0.1",
        "context_hash": canonical_hash(projection),
        **projection,
    }
    rctx_path.parent.mkdir(parents=True, exist_ok=True)
    rctx_path.write_text(
        json.dumps(rctx, ensure_ascii=False, separators=(",", ":")),
        encoding="utf-8",
    )
    report = {
        "schema_version": "certified-state-block-rctx-v1",
        "status": "ready",
        "state": state_name,
        "state_code": state_code,
        "state_fips": state_fips,
        "year": 2020,
        "rctx_path": rctx_path.relative_to(ROOT).as_posix(),
        "rctx_bytes": rctx_path.stat().st_size,
        "rctx_sha256": sha256(rctx_path),
        "context_hash": rctx["context_hash"],
        "unit_universe_hash": units["unit_universe_hash"],
        "unit_count": len(unit_ids),
        "population_total": sum(populations),
        "land_edge_count": len(land_weights),
        "land_component_count": len(land_components),
        "bridge_edge_count": len(bridges),
        "final_component_count": len(final_components),
        "geometry_toolchain": {
            "geopandas": gpd.__version__,
            "shapely": shapely.__version__,
            "geos": shapely.geos_version_string,
            "pyproj": pyproj.__version__,
            "proj": pyproj.proj_version_str,
            "crs": "EPSG:5070",
        },
        "claim_boundary": "Hash-bound connected block context; not a district certificate.",
    }
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-state-block-rctx-package-v1",
        "package_id": f"{lower}-2020-block-rctx",
        "status": "ready",
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "builder_path": SCRIPT.as_posix(),
        "builder_sha256": sha256(ROOT / SCRIPT),
        "claim_boundary": report["claim_boundary"],
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(
        f"{state_code} block RCTX: {len(unit_ids)} units, "
        f"{len(land_weights) + len(bridges)} edges, {len(bridges)} bridges"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--state-code", required=True)
    parser.add_argument("--state-fips", required=True)
    parser.add_argument("--state-name", required=True)
    parser.add_argument("--rctx", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    build(
        args.state_code.upper(),
        args.state_fips,
        args.state_name,
        ROOT / args.rctx,
        ROOT / args.report,
        ROOT / args.manifest,
    )


if __name__ == "__main__":
    main()
