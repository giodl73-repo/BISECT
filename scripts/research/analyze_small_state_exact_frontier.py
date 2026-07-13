#!/usr/bin/env python3
"""Analyze and verify the Rhode Island block-level exact benchmark frontier."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from decimal import Decimal, localcontext
from pathlib import Path
from typing import Any


SCHEMA = "exact-canonical-small-state-frontier-v1"
STATE = "rhode_island"
STATE_CODE = "RI"
STATE_FIPS = "44"
YEAR = 2020
DISTRICTS = 2
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
RUST_REFERENCE = ROOT / "crates/bisect-ilp/src/canonical.rs"
BLOCK_ROOT = ROOT / "data/2020/tiger/blocks/tl_2020_44_tabblock20"
SOURCE_PATHS = [
    BLOCK_ROOT / "tl_2020_44_tabblock20.cpg",
    BLOCK_ROOT / "tl_2020_44_tabblock20.dbf",
    BLOCK_ROOT / "tl_2020_44_tabblock20.prj",
    BLOCK_ROOT / "tl_2020_44_tabblock20.shp",
    BLOCK_ROOT / "tl_2020_44_tabblock20.shx",
    ROOT / "data/2020/redistricting/ri2020.pl/rigeo2020.pl",
    ROOT / "data/2020/redistricting/ri2020.pl/ri000012020.pl",
]
ADJACENCY_PATTERNS = [
    "data/2020/**/*ri*block*adj*",
    "data/2020/**/*rhode*block*adj*",
    "data/2020/**/*ri*block*.rctx",
    "data/2020/**/*44*block*adj*",
    "data/2020/**/*44*block*.rctx",
    "data/2020/**/tl_2020_44*adj*",
    "data/2020/**/tl_2020_44*.rctx",
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_record(path: Path) -> dict[str, Any]:
    return {
        "path": path.as_posix(),
        "bytes": path.stat().st_size,
        "sha256": sha256(path),
    }


def rust_reference_contract() -> tuple[str, int]:
    source = RUST_REFERENCE.read_text(encoding="utf-8")
    model = re.search(r'pub const EXACT_MODEL_ID: &str = "([^"]+)";', source)
    limit = re.search(r"pub const EXACT_ENUMERATION_LIMIT: usize = (\d+);", source)
    if not model or not limit:
        raise SystemExit("cannot read exact reference constants from canonical.rs")
    return model.group(1), int(limit.group(1))


def search_scale(unit_count: int, unit_limit: int) -> dict[str, Any]:
    exponent = unit_count - 1
    with localcontext() as context:
        context.prec = 60
        log10_two = Decimal(2).ln() / Decimal(10).ln()
        candidate_log10 = Decimal(exponent) * log10_two
        billion_per_second_year_log10 = candidate_log10 - (
            Decimal(1_000_000_000 * 31_557_600).ln() / Decimal(10).ln()
        )
        reference_log10 = Decimal((1 << (unit_limit - 1)) - 1).ln() / Decimal(
            10
        ).ln()
    return {
        "candidate_formula": f"2^{exponent}-1",
        "candidate_decimal_digits": int(candidate_log10) + 1,
        "candidate_log10": format(candidate_log10, ".30f"),
        "reference_limit_candidates": (1 << (unit_limit - 1)) - 1,
        "candidate_ratio_to_reference_log10": format(
            candidate_log10 - reference_log10, ".30f"
        ),
        "years_at_one_billion_candidates_per_second_log10": format(
            billion_per_second_year_log10, ".30f"
        ),
    }


def find_adjacency_artifacts() -> list[str]:
    found: set[str] = set()
    for pattern in ADJACENCY_PATTERNS:
        found.update(
            path.relative_to(ROOT).as_posix()
            for path in ROOT.glob(pattern)
            if path.is_file()
        )
    return sorted(found)


def analyze() -> dict[str, Any]:
    import geopandas as gpd

    from scripts.data.census.parse_pl94171_blocks_2020 import (
        parse_2020_geo_file,
        parse_2020_pop_file,
    )

    for path in SOURCE_PATHS:
        if not path.is_file():
            raise SystemExit(f"missing required source: {path}")
    shapefile = BLOCK_ROOT / "tl_2020_44_tabblock20.shp"
    tiger = gpd.read_file(shapefile)
    pl_geo = parse_2020_geo_file(SOURCE_PATHS[-2])
    pl_population = parse_2020_pop_file(SOURCE_PATHS[-1])
    merged = pl_geo.merge(pl_population, on="LOGRECNO", how="left")
    tiger_ids = set(tiger["GEOID20"].astype(str))
    pl_ids = set(pl_geo["GEOID"].str.removeprefix("7500000US"))
    unit_count = len(tiger)
    model_id, unit_limit = rust_reference_contract()
    adjacency_artifacts = find_adjacency_artifacts()
    return {
        "schema_version": SCHEMA,
        "status": "blocked",
        "benchmark": {
            "state": STATE,
            "state_code": STATE_CODE,
            "state_fips": STATE_FIPS,
            "year": YEAR,
            "districts": DISTRICTS,
            "statutory_unit": "2020 census tabulation block",
        },
        "source_files": [
            {
                **source_record(path),
                "path": path.relative_to(ROOT).as_posix(),
            }
            for path in SOURCE_PATHS
        ],
        "observed_instance": {
            "tiger_block_rows": unit_count,
            "tiger_unique_geoids": int(tiger["GEOID20"].nunique()),
            "geometry_null_count": int(tiger.geometry.isna().sum()),
            "geometry_empty_count": int(tiger.geometry.is_empty.sum()),
            "pl_block_rows": len(pl_geo),
            "pl_missing_population_count": int(merged["POP100"].isna().sum()),
            "population_total": int(merged["POP100"].sum()),
            "positive_population_blocks": int((merged["POP100"] > 0).sum()),
            "zero_population_blocks": int((merged["POP100"] == 0).sum()),
            "tiger_without_pl_count": len(tiger_ids - pl_ids),
            "pl_without_tiger_count": len(pl_ids - tiger_ids),
        },
        "exact_reference": {
            "model_id": model_id,
            "reference_source": RUST_REFERENCE.relative_to(ROOT).as_posix(),
            "reference_source_sha256": sha256(RUST_REFERENCE),
            "supported_k": 2,
            "unit_limit": unit_limit,
            "state_unit_count": unit_count,
            "units_above_limit": unit_count - unit_limit,
            **search_scale(unit_count, unit_limit),
        },
        "adjacency_custody": {
            "search_patterns": ADJACENCY_PATTERNS,
            "matching_artifacts": adjacency_artifacts,
            "block_rctx_available": any(
                artifact.endswith(".rctx") for artifact in adjacency_artifacts
            ),
        },
        "blockers": [
            {
                "kind": "model-bound",
                "detail": (
                    f"The E0 exact model accepts at most {unit_limit} units; "
                    f"Rhode Island has {unit_count} statutory blocks."
                ),
            },
            {
                "kind": "compute",
                "detail": (
                    "Under the E0 exhaustive proof contract, every symmetry-reduced "
                    "assignment is enumerated; "
                    f"the Rhode Island search is 2^{unit_count - 1}-1 candidates."
                ),
            },
            {
                "kind": "input-custody",
                "detail": (
                    "Block geometry and population custody is complete, but no matching "
                    "block adjacency or block RCTX artifact is present."
                ),
            },
        ],
        "disallowed_substitutions": [
            "tract-level exact claim",
            "inhabited-block-only universe",
            "heuristic plan labeled exact",
            "optimality-gap result labeled optimal",
        ],
        "claim_boundary": (
            "This is a reproducible real-data blocker report, not an exact State "
            "certificate. It does not construct block adjacency or replay a solver."
        ),
    }


def verify_report(report: dict[str, Any], check_sources: bool) -> None:
    if report.get("schema_version") != SCHEMA or report.get("status") != "blocked":
        raise SystemExit("unsupported or non-blocked frontier report")
    observed = report["observed_instance"]
    exact = report["exact_reference"]
    if exact["state_unit_count"] != observed["tiger_block_rows"]:
        raise SystemExit("frontier unit-count mismatch")
    if exact["units_above_limit"] != exact["state_unit_count"] - exact["unit_limit"]:
        raise SystemExit("frontier unit-limit arithmetic mismatch")
    if exact["candidate_formula"] != f"2^{exact['state_unit_count'] - 1}-1":
        raise SystemExit("frontier candidate formula mismatch")
    expected_scale = search_scale(exact["state_unit_count"], exact["unit_limit"])
    for key, value in expected_scale.items():
        if exact.get(key) != value:
            raise SystemExit(f"frontier search-scale mismatch: {key}")
    if observed["tiger_block_rows"] != observed["pl_block_rows"]:
        raise SystemExit("TIGER and PL block counts differ")
    if observed["positive_population_blocks"] + observed["zero_population_blocks"] != (
        observed["pl_block_rows"]
    ):
        raise SystemExit("population block partition mismatch")
    if check_sources:
        rebuilt = analyze()
        if report != rebuilt:
            raise SystemExit("frontier report differs from current source analysis")


def package_manifest(report_path: Path, manifest_path: Path) -> dict[str, Any]:
    analyzer_path = Path("scripts/research/analyze_small_state_exact_frontier.py")
    return {
        "schema_version": "exact-canonical-small-state-frontier-package-v1",
        "package_id": "ri-2020-block-exact-frontier",
        "status": "blocked-real-data-frontier",
        "files": [
            {
                "path": report_path.name,
                "sha256": sha256(report_path),
            }
        ],
        "analyzer_path": analyzer_path.as_posix(),
        "analyzer_sha256": sha256(analyzer_path),
        "verification_commands": [
            (
                "python scripts/research/analyze_small_state_exact_frontier.py "
                f"verify-package {manifest_path.as_posix()}"
            ),
            (
                "python scripts/research/analyze_small_state_exact_frontier.py "
                f"verify-package {manifest_path.as_posix()} --check-sources"
            ),
        ],
        "claim_boundary": (
            "Hash-bound Rhode Island 2020 block-level blocker report; "
            "not an exact certificate or solver result."
        ),
    }


def verify_package(manifest_path: Path, check_sources: bool) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema_version") != (
        "exact-canonical-small-state-frontier-package-v1"
    ):
        raise SystemExit("unsupported frontier package schema")
    if manifest.get("status") != "blocked-real-data-frontier":
        raise SystemExit("frontier package status drift")
    analyzer_path = ROOT / manifest["analyzer_path"]
    if manifest.get("analyzer_sha256") != sha256(analyzer_path):
        raise SystemExit("frontier analyzer source hash mismatch")
    files = manifest.get("files", [])
    if len(files) != 1:
        raise SystemExit("frontier package file inventory mismatch")
    report_path = manifest_path.parent / files[0]["path"]
    if files[0]["sha256"] != sha256(report_path):
        raise SystemExit("frontier report hash mismatch")
    verify_report(json.loads(report_path.read_text(encoding="utf-8")), check_sources)


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    analyze_parser = subparsers.add_parser("analyze")
    analyze_parser.add_argument("--output", type=Path, required=True)
    analyze_parser.add_argument("--manifest", type=Path)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("report", type=Path)
    verify_parser.add_argument("--check-sources", action="store_true")
    package_parser = subparsers.add_parser("verify-package")
    package_parser.add_argument("manifest", type=Path)
    package_parser.add_argument("--check-sources", action="store_true")
    args = parser.parse_args()
    if args.command == "analyze":
        report = analyze()
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        if args.manifest:
            args.manifest.write_text(
                json.dumps(package_manifest(args.output, args.manifest), indent=2) + "\n",
                encoding="utf-8",
            )
        print("Small-State exact frontier analysis: BLOCKED")
    elif args.command == "verify":
        report = json.loads(args.report.read_text(encoding="utf-8"))
        verify_report(report, args.check_sources)
        print("Small-State exact frontier verification: PASS")
    else:
        verify_package(args.manifest, args.check_sources)
        print("Small-State exact frontier package verification: PASS")


if __name__ == "__main__":
    main()
