#!/usr/bin/env python3
"""Freeze the 50-State 2010 PL/TIGER/RCTX custody inventory."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from config_2010 import STATE_CONFIG_2010  # noqa: E402


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
SCRIPT = Path("scripts/research/inventory_national_2010.py")


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


def count_2010_blocks(path: Path) -> int | None:
    if not path.is_file():
        return None
    count = 0
    with path.open("rb") as source:
        for line in source:
            if len(line) >= 11 and line[8:11] == b"750":
                count += 1
    return count


def main() -> None:
    rows = []
    for code, config in sorted(STATE_CONFIG_2010.items()):
        lower = code.lower()
        fips = FIPS[code]
        pl_dir = ROOT / f"data/2010/redistricting/{lower}2010.pl"
        shape_base = (
            ROOT / f"data/2010/tiger/blocks/tl_2010_{fips}_tabblock10/"
            f"tl_2010_{fips}_tabblock10"
        )
        archive = (
            ROOT / f"data/2010/tiger/archives/tl_2010_{fips}_tabblock10.zip"
        )
        rctx = ROOT / f"data/2010/certified/{lower}_blocks_2010.rctx"
        geo = pl_dir / f"{lower}geo2010.pl"
        sources = {
            "pl_geography": file_record(geo),
            "pl_population_segment_1": file_record(pl_dir / f"{lower}000012010.pl"),
            "pl_population_segment_2": file_record(pl_dir / f"{lower}000022010.pl"),
            "pl_packing_list": file_record(
                pl_dir / f"{lower}2010.pl.prd.packinglist.txt"
            ),
            "tiger_archive": file_record(archive),
            "tiger_shp": file_record(shape_base.with_suffix(".shp")),
            "tiger_dbf": file_record(shape_base.with_suffix(".dbf")),
            "tiger_shx": file_record(shape_base.with_suffix(".shx")),
        }
        pl_ready = all(sources[key]["ready"] for key in (
            "pl_geography", "pl_population_segment_1",
            "pl_population_segment_2", "pl_packing_list",
        ))
        tiger_extracted_ready = all(sources[key]["ready"] for key in (
            "tiger_shp", "tiger_dbf", "tiger_shx",
        ))
        tiger_custody_ready = bool(sources["tiger_archive"]["ready"])
        rows.append({
            "state": code,
            "name": config["name"],
            "fips": fips,
            "districts": config["districts"],
            "block_count": count_2010_blocks(geo),
            "pl_custody_ready": pl_ready,
            "tiger_ready": tiger_custody_ready,
            "tiger_custody_ready": tiger_custody_ready,
            "tiger_extracted_ready": tiger_extracted_ready,
            "tiger_source_url": (
                "https://www2.census.gov/geo/tiger/TIGER2010/TABBLOCK/2010/"
                f"tl_2010_{fips}_tabblock10.zip"
            ),
            "rctx_ready": rctx.is_file(),
            "rctx_path": rctx.relative_to(ROOT).as_posix(),
            "rctx_bytes": rctx.stat().st_size if rctx.is_file() else None,
            "sources": sources,
        })

    missing_pl = [row["state"] for row in rows if not row["pl_custody_ready"]]
    missing_tiger = [row["state"] for row in rows if not row["tiger_ready"]]
    missing_rctx = [row["state"] for row in rows if not row["rctx_ready"]]
    total_blocks = sum(row["block_count"] or 0 for row in rows)
    report = {
        "schema_version": "certified-national-2010-input-inventory-v1",
        "status": "ready" if not missing_pl and not missing_tiger and not missing_rctx else "incomplete",
        "census_year": 2010,
        "state_count": len(rows),
        "district_count": sum(row["districts"] for row in rows),
        "total_blocks": total_blocks,
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
            "Hash-bound local 2010 source and context custody inventory. "
            "An incomplete status is expected until TIGER block geometry and all RCTX files exist; "
            "this report makes no district-generation claim."
        ),
    }
    out = ROOT / "docs/experiments/nationwide-2010"
    out.mkdir(parents=True, exist_ok=True)
    report_path = out / "inventory.json"
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-national-2010-input-inventory-package-v1",
        "package_id": "nationwide-2010-input-inventory",
        "status": report["status"],
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "builder_path": SCRIPT.as_posix(),
        "builder_sha256": sha256(ROOT / SCRIPT),
        "claim_boundary": report["claim_boundary"],
    }
    manifest_path = out / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(
        f"2010 inventory: {report['pl_custody_ready_states']}/50 PL, "
        f"{report['tiger_ready_states']}/50 TIGER, "
        f"{report['existing_rctx_states']}/50 RCTX, {total_blocks} blocks"
    )


if __name__ == "__main__":
    main()
