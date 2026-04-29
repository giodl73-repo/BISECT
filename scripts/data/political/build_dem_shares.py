"""
Build per-tract Democratic vote share for `redist state --partition-mode partisan-weighted`.

Produces a TSV consumable by the Rust CLI. Format spec:
    docs/file-formats/partisan-shares.md

Two input modes:

  --mode direct
    Input CSV already has tract-level votes:
        geoid, dem_votes, rep_votes, [total_votes]
    Just computes dem_share = dem_votes / (dem_votes + rep_votes) and writes the TSV.
    This is what you have if precinct-to-tract aggregation was already done by
    another tool (MGGG OpenPrecincts post-processing, state SOS exports, etc.).

  --mode vap-disaggregate
    Input is county-level results + tract demographics. We disaggregate county
    votes to tracts proportional to voting-age population (VAP):
        tract_dem_votes = county_dem_votes * (tract_vap / county_vap)
    This is an approximation (assumes uniform turnout within a county) but is
    the standard fallback when precinct data isn't available.

Output:
    {output_dir}/dem_shares.tsv

Provenance:
    The output's leading comments record the script version, source files, and
    timestamp so a special master can reconstruct how the file was built.

Example invocations:

    # Direct mode (you already have tract-level results)
    python scripts/data/political/build_dem_shares.py \\
        --mode direct \\
        --state LA --year 2020 \\
        --input outputs/data/2020/elections/la_tract_results_2020.csv \\
        --output outputs/data/2020/partisan/la/dem_shares.tsv

    # VAP-disaggregation mode
    python scripts/data/political/build_dem_shares.py \\
        --mode vap-disaggregate \\
        --state LA --year 2020 \\
        --county-results outputs/data/2020/elections/la_county_2020.csv \\
        --tract-demographics outputs/data/2020/demographics/louisiana_demographics_2020.csv \\
        --output outputs/data/2020/partisan/la/dem_shares.tsv

Plan 03 (Callais 2026-04-29) — see docs/legal/CALLAIS_REFERENCE.md.
"""

from __future__ import annotations

import argparse
import csv
import datetime
import sys
from pathlib import Path
from typing import Dict, Iterable, Tuple

SCRIPT_VERSION = "1.0.0"
PROVENANCE_HEADER_LINES = 4  # number of `#` comment lines we always write


# ---------------------------------------------------------------------------
# Validators
# ---------------------------------------------------------------------------

def normalize_geoid(raw: str) -> str:
    """11-character TIGER tract GEOID with leading zeros."""
    s = str(raw).strip()
    if not s.isdigit():
        raise ValueError(f"GEOID must be all digits: {raw!r}")
    if len(s) > 11:
        raise ValueError(f"GEOID too long ({len(s)} chars): {raw!r}")
    return s.zfill(11)


def safe_dem_share(dem: float, rep: float) -> float:
    """dem / (dem + rep) with guard for zero-vote tracts (returns 0.5 = swing)."""
    total = dem + rep
    if total <= 0:
        return 0.5
    return dem / total


# ---------------------------------------------------------------------------
# Mode: direct
# ---------------------------------------------------------------------------

def build_from_tract_results(input_csv: Path) -> Dict[str, float]:
    """Read tract-level results CSV; return geoid -> dem_share."""
    shares: Dict[str, float] = {}
    with input_csv.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        if reader.fieldnames is None:
            raise ValueError(f"{input_csv}: empty file or missing header")
        required = {"geoid", "dem_votes", "rep_votes"}
        missing = required - set(reader.fieldnames)
        if missing:
            raise ValueError(f"{input_csv}: header missing columns: {sorted(missing)}")
        for row in reader:
            geoid = normalize_geoid(row["geoid"])
            dem = float(row["dem_votes"])
            rep = float(row["rep_votes"])
            if dem < 0 or rep < 0:
                raise ValueError(f"{input_csv}: negative votes for {geoid}")
            shares[geoid] = safe_dem_share(dem, rep)
    return shares


# ---------------------------------------------------------------------------
# Mode: vap-disaggregate
# ---------------------------------------------------------------------------

def build_from_county_disaggregation(
    county_results_csv: Path,
    tract_demographics_csv: Path,
) -> Dict[str, float]:
    """
    Disaggregate county-level votes to tracts using VAP shares.

    county_results_csv columns: county_fips (5 digits), dem_votes, rep_votes
    tract_demographics_csv columns: GEOID, county_fips (5 digits), vap (or voting_age_pop)

    Returns geoid -> dem_share. Tracts whose county isn't in the results get 0.5 (swing).
    """
    # Load county results
    county_votes: Dict[str, Tuple[float, float]] = {}
    with county_results_csv.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        if not {"county_fips", "dem_votes", "rep_votes"} <= set(reader.fieldnames or ()):
            raise ValueError(
                f"{county_results_csv}: header must have county_fips, dem_votes, rep_votes"
            )
        for row in reader:
            cfips = str(row["county_fips"]).strip().zfill(5)
            county_votes[cfips] = (float(row["dem_votes"]), float(row["rep_votes"]))

    # Load tract demographics, group VAP by county
    tract_vap: Dict[str, Tuple[str, float]] = {}  # geoid -> (county_fips, vap)
    county_total_vap: Dict[str, float] = {}
    with tract_demographics_csv.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        cols = set(reader.fieldnames or ())
        if "GEOID" not in cols and "geoid" not in cols:
            raise ValueError(f"{tract_demographics_csv}: header must have GEOID")
        vap_col = next((c for c in ("vap", "voting_age_pop", "VAP") if c in cols), None)
        if vap_col is None:
            raise ValueError(
                f"{tract_demographics_csv}: header must have one of vap, voting_age_pop, VAP"
            )
        geoid_col = "GEOID" if "GEOID" in cols else "geoid"
        for row in reader:
            geoid = normalize_geoid(row[geoid_col])
            cfips = geoid[:5]  # first 5 chars of tract GEOID = county FIPS
            vap = float(row[vap_col]) if row[vap_col] else 0.0
            tract_vap[geoid] = (cfips, vap)
            county_total_vap[cfips] = county_total_vap.get(cfips, 0.0) + vap

    # Disaggregate
    shares: Dict[str, float] = {}
    for geoid, (cfips, vap) in tract_vap.items():
        if cfips not in county_votes or county_total_vap.get(cfips, 0) <= 0:
            shares[geoid] = 0.5  # swing default for unmappable
            continue
        cd, cr = county_votes[cfips]
        # Allocation: fraction = tract_vap / county_vap; dem_share is the same
        # ratio at the county level (uniform turnout assumption), so we just
        # use the county-level share.
        shares[geoid] = safe_dem_share(cd, cr)
    return shares


# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------

def write_tsv(shares: Dict[str, float], out: Path, provenance: Dict[str, str]) -> None:
    """Write canonical partisan-shares TSV with provenance comments."""
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8", newline="\n") as f:
        for k, v in provenance.items():
            f.write(f"# {k}: {v}\n")
        f.write("geoid\tdem_share\n")
        for geoid in sorted(shares):
            share = shares[geoid]
            if not (0.0 <= share <= 1.0):
                raise ValueError(f"computed share {share} for {geoid} out of [0,1]")
            f.write(f"{geoid}\t{share:.6f}\n")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("--mode", choices=["direct", "vap-disaggregate"], required=True)
    p.add_argument("--state", required=True, help="State code, e.g. LA")
    p.add_argument("--year", required=True, help="Election year, e.g. 2020")
    p.add_argument("--output", required=True, help="Output TSV path")
    p.add_argument("--input", help="(direct mode) tract-level results CSV")
    p.add_argument("--county-results", help="(vap-disaggregate mode) county results CSV")
    p.add_argument("--tract-demographics",
                   help="(vap-disaggregate mode) tract demographics CSV with GEOID + VAP")
    args = p.parse_args(argv)

    if args.mode == "direct":
        if not args.input:
            p.error("--mode direct requires --input")
        shares = build_from_tract_results(Path(args.input))
        sources = {"input": args.input}
    else:  # vap-disaggregate
        if not (args.county_results and args.tract_demographics):
            p.error("--mode vap-disaggregate requires --county-results and --tract-demographics")
        shares = build_from_county_disaggregation(
            Path(args.county_results), Path(args.tract_demographics)
        )
        sources = {
            "county_results": args.county_results,
            "tract_demographics": args.tract_demographics,
        }

    provenance = {
        "producer": f"scripts/data/political/build_dem_shares.py v{SCRIPT_VERSION}",
        "mode": args.mode,
        "state": args.state,
        "year": args.year,
        "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        **sources,
    }
    write_tsv(shares, Path(args.output), provenance)
    print(f"[OK] Wrote {len(shares)} tracts -> {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
