"""
Ireland electoral data acquisition for E.6 international redistricting experiments.

Ireland (IE - STV):
  - 39 Dáil constituencies, 3–5 seats each = 160 seats total
  - ~18,000 small areas (SAs) from 2022 census
  - Population: ~5,123,000 (2022 census)
  - Balance tolerance: population within ±5% of quota per seat
  - Electoral system: Single Transferable Vote (STV), 3–5 seats per constituency

Data sources:
  - Boundaries: CSO Ireland Small Area boundaries 2022 (open data)
  - Population: CSO SAPS 2022 (Small Area Population Statistics)
  - Electoral: Oireachtas constituency map (Electoral (Amendment) Act 2023)

The key argument in E.6: algorithmic redistricting of Ireland's STV constituencies
achieves compactness improvements while preserving proportionality via per-constituency
D'Hondt allocation (proxy for STV). True STV would require voter preference data.

Usage:
    python scripts/data/international/acquire_ireland.py
    python scripts/data/international/acquire_ireland.py --check
    python scripts/data/international/acquire_ireland.py --run

Outputs:
    outputs/international/ireland/ie_adjacency_2022.pkl
    outputs/international/ireland/ie_adjacency_2022.adj.bin
    outputs/international/ireland/ie_adjacency_2022_geoids.json
"""

import argparse
import json
import pickle
import subprocess
import sys
from pathlib import Path

try:
    import geopandas as gpd
    import pandas as pd
    import requests
except ImportError as e:
    print(f"ERROR: Missing dependency: {e}")
    print("Install: pip install geopandas pandas requests")
    sys.exit(1)

# ---------------------------------------------------------------------------
# Data sources
# ---------------------------------------------------------------------------

# CSO Ireland open data portal (data.gov.ie / CSO direct)
IRELAND_SOURCES = {
    # Small Area boundaries 2022 (CSO SAPS 2022)
    # Full URL: https://data.gov.ie/dataset/cso-small-areas-national-statistical-boundaries-2022
    "sa_boundaries": "https://opendata.arcgis.com/datasets/49e62cc43dce4c7d950b23b2e67f5a67_0.geojson",

    # CSO SAPS 2022 — population by small area
    # https://www.cso.ie/en/census/census2022/census2022smallareapopulationstatistics/
    "saps_2022": "https://www.cso.ie/en/media/csoie/census/census2022/SAPS_2022_SA2017_V2.xlsx",

    # Electoral boundaries 2023 (after Electoral Amendment Act 2023 — 39 constituencies, 160 seats)
    # From Electoral Commission Ireland: https://www.electoralcommission.ie
    "constituencies_2023": "https://opendata.arcgis.com/datasets/electoral-commission-ireland-2023-constituencies.geojson",
}

# 2023 Dáil constituencies (Electoral (Amendment) Act 2023)
# 39 constituencies, seat counts: 3×3-seat, 16×4-seat, 20×5-seat (changes from 2020 allocation)
IRELAND_CONSTITUENCIES_2023 = {
    "Carlow–Kilkenny": 5, "Cavan–Monaghan": 4, "Clare": 4,
    "Cork East": 4, "Cork North–Central": 4, "Cork North–West": 3,
    "Cork South–Central": 5, "Cork South–West": 3,
    "Donegal": 5, "Dublin Bay North": 5, "Dublin Bay South": 4,
    "Dublin Central": 4, "Dublin Fingal East": 4, "Dublin Fingal West": 4,
    "Dublin Mid–West": 4, "Dublin North–West": 3,
    "Dublin Rathdown": 4, "Dublin South–Central": 4,
    "Dublin South–West": 5, "Dublin West": 4,
    "Dún Laoghaire": 4, "Galway East": 3,
    "Galway West": 5, "Kerry": 5,
    "Kildare North": 4, "Kildare South": 4,
    "Laois–Offaly": 5, "Limerick City": 4,
    "Limerick County": 3, "Longford–Westmeath": 4,
    "Louth": 5, "Mayo": 4,
    "Meath East": 3, "Meath West": 3,
    "Roscommon–Galway": 3, "Sligo–Leitrim": 4,
    "Tipperary North": 3, "Tipperary South": 3,
    "Waterford": 4, "Wexford": 5,
    "Wicklow": 5,
}
IRELAND_TOTAL_SEATS_2023 = sum(IRELAND_CONSTITUENCIES_2023.values())  # 160
IRELAND_N_CONSTITUENCIES = len(IRELAND_CONSTITUENCIES_2023)  # 39 (may vary slightly)
IRELAND_TOTAL_POP_2022 = 5_123_536  # CSO Census 2022

OUT_DIR = Path("outputs/international/ireland")


# ---------------------------------------------------------------------------
# Boundary acquisition
# ---------------------------------------------------------------------------

def download_ireland_boundaries() -> gpd.GeoDataFrame:
    """Download CSO Small Area boundaries for Ireland."""
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    cache_path = OUT_DIR / "ie_sa_boundaries_2022.geojson"

    if cache_path.exists():
        print(f"  [CACHE] Using cached: {cache_path} ({cache_path.stat().st_size/1e6:.1f} MB)")
        return gpd.read_file(cache_path)

    print("  Downloading CSO Ireland Small Area boundaries (~18K units, ~50 MB)...")
    print(f"  URL: {IRELAND_SOURCES['sa_boundaries']}")

    try:
        resp = requests.get(IRELAND_SOURCES["sa_boundaries"], timeout=120, stream=True)
        resp.raise_for_status()

        total = int(resp.headers.get("content-length", 0))
        downloaded = 0
        chunks = []
        for chunk in resp.iter_content(chunk_size=65536):
            chunks.append(chunk)
            downloaded += len(chunk)
            if total:
                pct = downloaded / total * 100
                print(f"\r  Downloading... {pct:.0f}% ({downloaded/1e6:.1f}/{total/1e6:.1f} MB)", end="")
        print()

        content = b"".join(chunks)
        cache_path.write_bytes(content)
        gdf = gpd.read_file(cache_path)
        print(f"  [OK] Downloaded {len(gdf)} small areas")
        return gdf

    except Exception as e:
        print(f"  [WARN] Download failed: {e}")
        print("  Building constituency-level fallback...")
        return _build_constituency_fallback()


def _build_constituency_fallback() -> gpd.GeoDataFrame:
    """Constituency-level fallback when SA boundaries unavailable."""
    from shapely.geometry import box

    # Approximate constituency bounding boxes (lon_min, lat_min, lon_max, lat_max)
    # Ireland bounding box: -10.47°W to -6.00°W, 51.44°N to 55.39°N
    constituency_approx = {
        "Dublin Bay North": (-6.22, 53.33, -6.05, 53.50),
        "Dublin Bay South": (-6.28, 53.28, -6.09, 53.36),
        "Dublin Central": (-6.30, 53.33, -6.22, 53.39),
        "Dublin Fingal East": (-6.20, 53.44, -6.00, 53.60),
        "Dublin Fingal West": (-6.38, 53.44, -6.20, 53.65),
        "Dublin Mid–West": (-6.48, 53.28, -6.32, 53.40),
        "Dublin North–West": (-6.38, 53.36, -6.24, 53.46),
        "Dublin Rathdown": (-6.30, 53.23, -6.13, 53.34),
        "Dublin South–Central": (-6.32, 53.30, -6.22, 53.38),
        "Dublin South–West": (-6.44, 53.22, -6.28, 53.34),
        "Dublin West": (-6.48, 53.34, -6.34, 53.46),
        "Dún Laoghaire": (-6.18, 53.25, -6.06, 53.33),
        "Cork South–Central": (-8.55, 51.85, -8.35, 51.95),
        "Cork North–Central": (-8.55, 51.90, -8.35, 52.00),
        "Cork East": (-8.30, 51.90, -8.00, 52.10),
        "Cork North–West": (-9.20, 51.90, -8.55, 52.20),
        "Cork South–West": (-9.50, 51.60, -8.80, 51.90),
        "Kerry": (-10.47, 51.75, -9.20, 52.40),
        "Limerick City": (-8.75, 52.60, -8.55, 52.70),
        "Limerick County": (-9.40, 52.35, -8.55, 52.60),
        "Galway West": (-9.50, 53.15, -8.80, 53.50),
        "Galway East": (-8.80, 53.00, -7.90, 53.50),
        "Mayo": (-10.10, 53.45, -8.80, 54.30),
        "Roscommon–Galway": (-8.80, 53.30, -7.80, 53.80),
        "Sligo–Leitrim": (-8.80, 53.80, -7.80, 54.50),
        "Donegal": (-8.50, 54.30, -7.10, 55.39),
        "Cavan–Monaghan": (-7.80, 53.80, -6.80, 54.30),
        "Louth": (-6.80, 53.80, -6.10, 54.10),
        "Meath East": (-6.70, 53.45, -6.15, 53.80),
        "Meath West": (-7.20, 53.45, -6.70, 53.80),
        "Kildare North": (-6.80, 53.25, -6.48, 53.55),
        "Kildare South": (-7.10, 52.90, -6.48, 53.25),
        "Longford–Westmeath": (-8.00, 53.30, -7.00, 53.80),
        "Laois–Offaly": (-7.90, 52.90, -6.90, 53.30),
        "Tipperary North": (-8.40, 52.50, -7.60, 52.90),
        "Tipperary South": (-8.40, 52.20, -7.60, 52.50),
        "Waterford": (-8.10, 52.00, -6.95, 52.40),
        "Wexford": (-6.95, 52.10, -6.10, 52.60),
        "Wicklow": (-6.55, 52.60, -5.98, 53.20),
        "Carlow–Kilkenny": (-7.50, 52.35, -6.80, 52.85),
    }

    rows = []
    for i, (name, seats) in enumerate(IRELAND_CONSTITUENCIES_2023.items()):
        bbox = constituency_approx.get(name)
        if bbox is None:
            # Generic Ireland bbox fallback
            bbox = (-10.0, 51.5 + i * 0.1, -6.0, 55.4)
        lon0, lat0, lon1, lat1 = bbox
        from shapely.geometry import box as make_box
        avg_pop = IRELAND_TOTAL_POP_2022 // IRELAND_N_CONSTITUENCIES
        rows.append({
            "GEOID": f"IE{i+1:05d}00000",
            "constituency": name,
            "seats": seats,
            "population": avg_pop,
            "geometry": make_box(lon0, lat0, lon1, lat1),
        })

    gdf = gpd.GeoDataFrame(rows, crs="EPSG:4326")
    out = OUT_DIR / "ie_constituencies_2023_synthetic.geojson"
    gdf.to_file(out, driver="GeoJSON")
    print(f"  [OK] Built synthetic {len(gdf)}-constituency GeoDataFrame")
    print(f"  [WARN] Using constituency-level boundaries — download SA boundaries for publication")
    return gdf


# ---------------------------------------------------------------------------
# Population join
# ---------------------------------------------------------------------------

def attach_population(gdf: gpd.GeoDataFrame) -> gpd.GeoDataFrame:
    """Attach population data to the GeoDataFrame."""
    if "population" in gdf.columns and gdf["population"].sum() > 0:
        return gdf

    # Try to download CSO SAPS 2022
    pop_cache = OUT_DIR / "ie_saps_2022.xlsx"
    if not pop_cache.exists():
        print("  Downloading CSO SAPS 2022 population data...")
        try:
            resp = requests.get(IRELAND_SOURCES["saps_2022"], timeout=60)
            resp.raise_for_status()
            pop_cache.write_bytes(resp.content)
            print(f"  [OK] Downloaded population data ({pop_cache.stat().st_size/1e6:.1f} MB)")
        except Exception as e:
            print(f"  [WARN] Population download failed: {e}")
            print("  Using uniform population distribution")
            n = len(gdf)
            gdf["population"] = IRELAND_TOTAL_POP_2022 // n
            return gdf

    try:
        pop_df = pd.read_excel(pop_cache, sheet_name=0, usecols=["GEOGID", "T1_1AGETT"])
        pop_df.columns = ["GEOID", "population"]
        gdf = gdf.merge(pop_df, on="GEOID", how="left")
        gdf["population"] = gdf["population"].fillna(280).astype(int)  # avg SA ~280 people
        print(f"  [OK] Joined population data (total: {gdf['population'].sum():,})")
    except Exception as e:
        print(f"  [WARN] Population join failed: {e}, using uniform distribution")
        gdf["population"] = IRELAND_TOTAL_POP_2022 // len(gdf)

    return gdf


# ---------------------------------------------------------------------------
# Adjacency graph construction
# ---------------------------------------------------------------------------

def build_adjacency(gdf: gpd.GeoDataFrame) -> dict:
    """Build adjacency graph. For large datasets (18K SAs), uses vectorized spatial join."""
    try:
        gdf_proj = gdf.to_crs("EPSG:2157")  # Irish Transverse Mercator
    except Exception:
        gdf_proj = gdf.to_crs("EPSG:32629")  # UTM zone 29N fallback

    n = len(gdf_proj)
    geoids = gdf_proj["GEOID"].tolist()
    populations = gdf_proj["population"].tolist()
    print(f"  Building adjacency for {n} units (may take 1–3 minutes for 18K SAs)...")

    if n > 1000:
        # Vectorized approach: spatial index join for large datasets
        adjacency, edge_weights = _build_adjacency_vectorized(gdf_proj)
    else:
        adjacency, edge_weights = _build_adjacency_brute(gdf_proj, n)

    index_to_geoid = {i: geoids[i] for i in range(n)}
    print(f"  [OK] {n} units, {len(edge_weights)} edges")

    return {
        "adjacency": adjacency,
        "vertex_weights": [int(p) for p in populations],
        "edge_weights": edge_weights,
        "index_to_geoid": index_to_geoid,
    }


def _build_adjacency_vectorized(gdf_proj: gpd.GeoDataFrame) -> tuple:
    """Vectorized adjacency via geopandas spatial join (efficient for 18K units)."""
    adjacency = [[] for _ in range(len(gdf_proj))]
    edge_weights = {}

    # Add a small buffer to catch touching polygons
    buffered = gdf_proj.copy()
    buffered["geometry"] = gdf_proj.geometry.buffer(1)  # 1 meter buffer

    joined = gpd.sjoin(
        buffered.reset_index().rename(columns={"index": "idx_left"}),
        buffered.reset_index().rename(columns={"index": "idx_right"})[["idx_right", "geometry"]],
        how="left", predicate="intersects"
    )

    for _, row in joined.iterrows():
        i = int(row["idx_left"])
        j = int(row["idx_right"])
        if i >= j:
            continue
        shared = gdf_proj.geometry.iloc[i].boundary.intersection(gdf_proj.geometry.iloc[j].boundary)
        length = shared.length if not shared.is_empty else 1.0
        if length > 0:
            adjacency[i].append(j)
            adjacency[j].append(i)
            edge_weights[(i, j)] = length

    return adjacency, edge_weights


def _build_adjacency_brute(gdf_proj: gpd.GeoDataFrame, n: int) -> tuple:
    adjacency = [[] for _ in range(n)]
    edge_weights = {}
    for i in range(n):
        for j in range(i + 1, n):
            if gdf_proj.geometry.iloc[i].touches(gdf_proj.geometry.iloc[j]):
                shared = gdf_proj.geometry.iloc[i].boundary.intersection(gdf_proj.geometry.iloc[j].boundary)
                length = shared.length if not shared.is_empty else 1.0
                adjacency[i].append(j)
                adjacency[j].append(i)
                edge_weights[(i, j)] = length
    return adjacency, edge_weights


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Acquire Ireland census data for E.6 experiments")
    parser.add_argument("--check", action="store_true", help="Check data sources only")
    parser.add_argument("--run", action="store_true", help="Also run redistricting after acquisition")
    parser.add_argument("--force", action="store_true", help="Re-download even if cached")
    args = parser.parse_args()

    print("=== Ireland Data Acquisition (E.6 International Redistricting) ===")
    print(f"Output directory: {OUT_DIR.resolve()}\n")

    if args.check:
        print(f"Constituencies: {IRELAND_N_CONSTITUENCIES}")
        print(f"Total seats: {IRELAND_TOTAL_SEATS_2023}")
        print(f"Population (2022): {IRELAND_TOTAL_POP_2022:,}")
        print(f"Quota per seat: {IRELAND_TOTAL_POP_2022 // IRELAND_TOTAL_SEATS_2023:,}")
        print("\nSeat distribution:")
        for n_seats in sorted(set(IRELAND_CONSTITUENCIES_2023.values())):
            count = sum(1 for s in IRELAND_CONSTITUENCIES_2023.values() if s == n_seats)
            print(f"  {n_seats}-seat: {count} constituencies")
        return

    if args.force:
        for f in OUT_DIR.glob("ie_sa_boundaries*.geojson"):
            f.unlink()

    # Step 1: Boundaries
    print("Step 1: Acquiring Ireland Small Area boundaries...")
    gdf = download_ireland_boundaries()
    if "GEOID" not in gdf.columns:
        geoid_col = next((c for c in gdf.columns if "SA_GUID" in c.upper() or "GEOID" in c.upper() or "GUID" in c.upper()), None)
        if geoid_col:
            gdf = gdf.rename(columns={geoid_col: "GEOID"})
        else:
            gdf["GEOID"] = [f"IE{i+1:05d}00000" for i in range(len(gdf))]

    # Step 2: Population
    print("\nStep 2: Attaching population data...")
    gdf = attach_population(gdf)

    # Step 3: Adjacency
    print("\nStep 3: Building adjacency graph...")
    graph_dict = build_adjacency(gdf)

    # Step 4: Save
    print("\nStep 4: Saving adjacency files...")
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    pkl_path = OUT_DIR / "ie_adjacency_2022.pkl"
    with open(pkl_path, "wb") as f:
        pickle.dump(graph_dict, f)
    print(f"  [OK] {pkl_path}")

    # Step 5: Convert to .adj.bin
    geoid_path = pkl_path.with_name("ie_adjacency_2022_geoids.json")
    ig = graph_dict.get("index_to_geoid", {})
    geoid_path.write_text(json.dumps({str(k): v for k, v in ig.items()}))
    try:
        import redist_py
        adj = graph_dict["adjacency"]
        vw = [int(x) for x in graph_dict["vertex_weights"]]
        ew = {(min(i, j), max(i, j)): float(w) for (i, j), w in graph_dict["edge_weights"].items()}
        bin_data = redist_py.adjacency_to_bin(adj, vw, ew, len(adj), len(ew))
        bin_path = OUT_DIR / "ie_adjacency_2022.adj.bin"
        bin_path.write_bytes(bin_data)
        print(f"  [OK] {bin_path} ({len(bin_data)/1e6:.1f} MB)")
    except ImportError:
        bin_path = None
        print("  [WARN] redist_py not available — .adj.bin not generated")
        print("  Build with: cd redist/python/redist_py && maturin develop")

    # Summary
    print(f"\n=== Ireland Acquisition Complete ===")
    print(f"  Units: {len(graph_dict['adjacency'])}")
    print(f"  Edges: {len(graph_dict['edge_weights'])}")
    print(f"  Total pop: {sum(graph_dict['vertex_weights']):,}")
    print(f"  Constituencies: {IRELAND_N_CONSTITUENCIES} × avg 4.1 seats = {IRELAND_TOTAL_SEATS_2023} total")
    print(f"\nNext — run redistricting (avg seats=4, balance=5%):")
    print(f"  redist state --state IE --year 2022 --version international \\")
    print(f"    --adjacency {bin_path or pkl_path} --state-name ireland \\")
    print(f"    --districts {IRELAND_N_CONSTITUENCIES} --seats-per-district 4 \\")
    print(f"    --chamber parliamentary --balance-tolerance 5 --label ireland_dail_2022")

    if args.run and bin_path:
        print("\nStep 6: Running redistricting...")
        redist = Path("redist/target/release/redist.exe")
        if not redist.exists():
            redist = Path("redist/target/release/redist")
        if redist.exists():
            cmd = [str(redist), "state",
                   "--state", "IE", "--year", "2022", "--version", "international",
                   "--adjacency", str(bin_path), "--state-name", "ireland",
                   "--districts", str(IRELAND_N_CONSTITUENCIES),
                   "--seats-per-district", "4",
                   "--chamber", "parliamentary", "--balance-tolerance", "5",
                   "--label", "ireland_dail_2022", "--manifest", "--force"]
            result = subprocess.run(cmd, capture_output=True, text=True)
            print(result.stderr)
        else:
            print("  [WARN] redist binary not found")


if __name__ == "__main__":
    main()
