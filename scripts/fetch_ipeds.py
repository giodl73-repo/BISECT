"""
fetch_ipeds.py — Download IPEDS enrollment data for N.2 (college student population).

Downloads IPEDS Institutional Characteristics + Fall Enrollment tables,
geocodes campus addresses to census tract, and outputs a parquet file
with columns: [tract_geoid, campus_name, state, enrollment_total, enrollment_fall].

Usage:
    python scripts/fetch_ipeds.py --year 2020 --out data/college/ipeds_2020.parquet

Data source:
    NCES IPEDS: https://nces.ed.gov/ipeds/datacenter/DataFiles.aspx
    - HD{year}.csv: Institutional Characteristics (name, address, lat/lon)
    - EF{year}A.csv: Fall Enrollment by institution

Census geocoding:
    Uses FCC Area API (free, no key required) to map lat/lon to census tract GEOID.
    Rate-limited to 30 req/sec per FCC API terms of service.
"""

import argparse
import logging
import time
import urllib.request
import json
import zipfile
import io
import pandas as pd
from pathlib import Path

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
log = logging.getLogger(__name__)

IPEDS_BASE = "https://nces.ed.gov/ipeds/datacenter/data"
FCC_API = "https://geo.fcc.gov/api/census/block/find"


def download_ipeds_file(year: int, table: str, out_dir: Path) -> Path:
    """Download IPEDS zip file and extract the first CSV."""
    url = f"{IPEDS_BASE}/{table}{year}.zip"
    out_path = out_dir / f"{table}{year}.csv"
    if out_path.exists():
        log.info(f"Using cached {out_path}")
        return out_path

    log.info(f"Downloading {url}")
    with urllib.request.urlopen(url) as resp:
        data = resp.read()

    with zipfile.ZipFile(io.BytesIO(data)) as z:
        csv_names = [n for n in z.namelist() if n.upper().endswith(".CSV")]
        if not csv_names:
            raise RuntimeError(f"No CSV in {url}")
        csv_name = csv_names[0]
        with z.open(csv_name) as f:
            out_path.write_bytes(f.read())

    log.info(f"Wrote {out_path}")
    return out_path


def geocode_to_tract(lat: float, lon: float, retries: int = 3) -> str | None:
    """Use FCC Area API to get census tract GEOID for a lat/lon."""
    url = (
        f"{FCC_API}?latitude={lat}&longitude={lon}&format=json&censusYear=2020"
    )
    for attempt in range(retries):
        try:
            with urllib.request.urlopen(url, timeout=10) as resp:
                data = json.loads(resp.read())
            block = data.get("Block", {})
            fips = block.get("FIPS", "")
            if len(fips) >= 11:
                return fips[:11]  # tract = state(2) + county(3) + tract(6)
        except Exception as e:
            log.warning(f"Geocode attempt {attempt+1} failed: {e}")
            time.sleep(1.0)
    return None


def fetch_ipeds(year: int, out_path: Path) -> None:
    """Main entry point."""
    cache_dir = out_path.parent / "ipeds_cache"
    cache_dir.mkdir(parents=True, exist_ok=True)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    # Download institutional characteristics (has lat/lon)
    hd_path = download_ipeds_file(year, "HD", cache_dir)
    ef_path = download_ipeds_file(year, "EF", cache_dir)

    log.info("Loading institutional characteristics...")
    hd = pd.read_csv(
        hd_path,
        usecols=["UNITID", "INSTNM", "STABBR", "LATITUDE", "LONGITUD", "ICLEVEL"],
        encoding="latin-1",
        low_memory=False,
    )
    hd = hd[hd["ICLEVEL"].isin([1, 2])]  # 4-year and 2-year only
    hd = hd.dropna(subset=["LATITUDE", "LONGITUD"])
    hd = hd[(hd["LATITUDE"].abs() < 90) & (hd["LONGITUD"].abs() < 180)]

    log.info("Loading fall enrollment...")
    ef = pd.read_csv(
        ef_path,
        usecols=["UNITID", "EFYTOTLT"],  # EFYTOTLT = total fall enrollment
        encoding="latin-1",
        low_memory=False,
    )
    ef = ef.groupby("UNITID")["EFYTOTLT"].sum().reset_index()

    # Join
    df = hd.merge(ef, on="UNITID", how="left")
    df["EFYTOTLT"] = df["EFYTOTLT"].fillna(0).astype(int)

    # Geocode to tract (rate-limited)
    log.info(f"Geocoding {len(df)} campuses to census tracts (this takes ~{len(df)//30} seconds)...")
    tracts = []
    for i, row in df.iterrows():
        if i % 100 == 0:
            log.info(f"  {i}/{len(df)}")
        tract = geocode_to_tract(row["LATITUDE"], row["LONGITUD"])
        tracts.append(tract)
        time.sleep(0.034)  # 30 req/sec FCC limit

    df["tract_geoid"] = tracts
    df = df.dropna(subset=["tract_geoid"])

    result = df.rename(columns={
        "INSTNM": "campus_name",
        "STABBR": "state",
        "EFYTOTLT": "enrollment_total",
    })[["tract_geoid", "campus_name", "state", "enrollment_total", "LATITUDE", "LONGITUD"]]

    result.to_parquet(out_path, index=False)
    log.info(f"Wrote {len(result)} campuses to {out_path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--year", type=int, default=2020)
    parser.add_argument("--out", default="data/college/ipeds_2020.parquet")
    args = parser.parse_args()
    fetch_ipeds(args.year, Path(args.out))
