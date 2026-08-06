#!/usr/bin/env python3
"""Freeze the 50-State Census 2000 PL/TIGER/RCTX custody inventory."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from config_2000 import STATE_CONFIG_2000  # noqa: E402


FIPS = {
    "AL": "01", "AK": "02", "AZ": "04", "AR": "05", "CA": "06",
    "CO": "08", "CT": "09", "DE": "10", "FL": "12", "GA": "13",
    "HI": "15", "ID": "16", "IL": "17", "IN": "18", "IA": "19",
    "KS": "20", "KY": "21", "LA": "22", "ME": "23", "MD": "24",
    "MA": "25", "MI": "26", "MN": "27", "MS": "28", "MO": "29",
    "MT": "30", "NE": "31", "NV": "32", "NH": "33", "NJ": "34",
    "NM": "35", "NY": "36", "NC": "37", "ND": "38", "OH": "39",
    "OK": "40", "OR": "41", "PA": "42", "RI": "44", "SC": "45",
    "SD": "46", "TN": "47", "TX": "48", "UT": "49", "VT": "50",
    "VA": "51", "WA": "53", "WV": "54", "WI": "55", "WY": "56",
}
SCRIPT = Path("scripts/research/inventory_national_2000.py")
TIGER_BASE_URL = "https://www2.census.gov/geo/tiger/TIGER2010/TABBLOCK/2000"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(8 * 1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def file_record(path: Path) -> dict[str, object]:
    relative = path.relative_to(ROOT).as_posix()
    if not path.is_file():
        return {"path": relative, "ready": False, "bytes": None, "sha256": None}
    return {
        "path": relative,
        "ready": True,
        "bytes": path.stat().st_size,
        "sha256": sha256(path),
    }


def inspect_geography(path: Path) -> tuple[int | None, list[str]]:
    if not path.is_file():
        return None, []
    blocks = 0
    counties: set[str] = set()
    with path.open("rb") as source:
        for line in source:
            if len(line) < 66:
                continue
            if line[8:11] == b"750":
                blocks += 1
            elif line[8:11] == b"050":
                county = line[31:34]
                if len(county) == 3 and county.isdigit():
                    counties.add(county.decode("ascii"))
    return blocks, sorted(counties)


def main() -> None:
    rows = []
    for code, config in sorted(STATE_CONFIG_2000.items()):
        lower = code.lower()
        fips = FIPS[code]
        geography = ROOT / f"data/2000/redistricting/{lower}geo.upl"
        block_count, counties = inspect_geography(geography)
        archive_dir = ROOT / f"data/2000/tiger/archives/{lower}"
        block_dir = ROOT / f"data/2000/tiger/blocks/{lower}"
        archives = []
        components = []
        urls = []
        for county in counties:
            stem = f"tl_2010_{fips}{county}_tabblock00"
            archives.append(file_record(archive_dir / f"{stem}.zip"))
            urls.append(f"{TIGER_BASE_URL}/{stem}.zip")
            for extension in ("shp", "dbf", "shx"):
                components.append(file_record(block_dir / f"{stem}.{extension}"))
        archive_ready = bool(counties) and all(record["ready"] for record in archives)
        extracted_ready = bool(components) and all(record["ready"] for record in components)
        rctx = ROOT / f"data/2000/certified/{lower}_blocks_2000.rctx"
        rows.append({
            "state": code,
            "name": config["name"],
            "fips": fips,
            "districts": config["districts"],
            "block_count": block_count,
            "county_count": len(counties),
            "county_fips": counties,
            "pl_custody_ready": geography.is_file(),
            "tiger_ready": archive_ready,
            "tiger_custody_ready": archive_ready,
            "tiger_extracted_ready": extracted_ready,
            "tiger_source_urls": urls,
            "rctx_ready": rctx.is_file(),
            "rctx_path": rctx.relative_to(ROOT).as_posix(),
            "rctx_bytes": rctx.stat().st_size if rctx.is_file() else None,
            "sources": {
                "pl_geography_and_population": file_record(geography),
                "tiger_archives": archives,
                "tiger_components": components,
            },
        })

    missing_pl = [row["state"] for row in rows if not row["pl_custody_ready"]]
    missing_tiger = [row["state"] for row in rows if not row["tiger_ready"]]
    missing_rctx = [row["state"] for row in rows if not row["rctx_ready"]]
    total_blocks = sum(row["block_count"] or 0 for row in rows)
    report = {
        "schema_version": "certified-national-2000-input-inventory-v1",
        "status": "ready" if not missing_pl and not missing_tiger and not missing_rctx else "incomplete",
        "census_year": 2000,
        "state_count": len(rows),
        "district_count": sum(row["districts"] for row in rows),
        "total_blocks": total_blocks,
        "county_archive_count": sum(row["county_count"] for row in rows),
        "pl_custody_ready_states": len(rows) - len(missing_pl),
        "tiger_ready_states": len(rows) - len(missing_tiger),
        "existing_rctx_states": len(rows) - len(missing_rctx),
        "missing_pl_states": missing_pl,
        "missing_tiger_states": missing_tiger,
        "missing_rctx_states": missing_rctx,
        "states": sorted(rows, key=lambda row: (row["block_count"] or 10**9, row["state"])),
        "batch_order": [
            row["state"]
            for row in sorted(rows, key=lambda row: (row["block_count"] or 10**9, row["state"]))
            if not row["rctx_ready"]
        ],
        "claim_boundary": (
            "Hash-bound local Census 2000 source and context custody inventory. "
            "Official county TIGER/Line archives are required for geometry custody; extracted "
            "members are reproducible working files. This report makes no district-generation claim."
        ),
    }
    out = ROOT / "docs/experiments/nationwide-2000"
    out.mkdir(parents=True, exist_ok=True)
    report_path = out / "inventory.json"
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-national-2000-input-inventory-package-v1",
        "package_id": "nationwide-2000-input-inventory",
        "status": report["status"],
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "builder_path": SCRIPT.as_posix(),
        "builder_sha256": sha256(ROOT / SCRIPT),
        "claim_boundary": report["claim_boundary"],
    }
    manifest_path = out / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(
        f"2000 inventory: {report['pl_custody_ready_states']}/50 PL, "
        f"{report['tiger_ready_states']}/50 TIGER, "
        f"{report['existing_rctx_states']}/50 RCTX, {total_blocks} blocks, "
        f"{report['county_archive_count']} county archives"
    )


if __name__ == "__main__":
    main()
