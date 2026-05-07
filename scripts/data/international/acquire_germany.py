"""
Germany (Bundeswahlkreise) data acquisition for E.6 international redistricting.

Germany (DE-WAHLKREIS):
  - 299 Wahlkreise (single-member FPTP seats) + list seats = 736 total Bundestag seats
  - ~8,300+ Gemeinden (municipalities) as base geographic units
  - Population: ~83.2M (2021 census, German citizens only for electoral purposes)
  - Balance tolerance: ±25% deviation from average (Bundeswahlgesetz §3)
  - Electoral system: Mixed-member proportional (MMP)
  - Census year: 2021

Data sources:
  - Boundaries: GADM v4.1 Germany level 3 (Gemeinden/Verwaltungsgemeinschaften)
    OR Destatis open data (ZENSUS 2021)
  - Population: Destatis ZENSUS 2021 small-area statistics
  - Electoral: Federal Returning Officer (bundeswahlleiter.de) Wahlkreiseinteilung

Note on German electoral geography:
  - Wahlkreise must respect Landkreis (county) boundaries where possible
  - Each Wahlkreis contains approximately one constituency averaging ~279K German citizens
  - 25% deviation allowed to account for population distribution constraints
  - Independent cities (kreisfreie Städte, similar to VA independent cities) are
    tracked separately from Landkreise in the split analysis

Usage:
    python scripts/data/international/acquire_germany.py
    python scripts/data/international/acquire_germany.py --check
    python scripts/data/international/acquire_germany.py --run

Outputs:
    outputs/international/germany/de_adjacency_2021.pkl
    outputs/international/germany/de_adjacency_2021.adj.bin
    outputs/international/germany/de_adjacency_2021_geoids.json
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
# Constants
# ---------------------------------------------------------------------------

GERMANY_TOTAL_POP_2021 = 83_200_000
GERMANY_TOTAL_POP_CITIZENS = 73_100_000  # German citizens only (electoral basis)
GERMANY_WAHLKREISE = 299
GERMANY_IDEAL_POP = GERMANY_TOTAL_POP_CITIZENS // GERMANY_WAHLKREISE  # ~244K

OUT_DIR = Path("outputs/international/germany")

# ---------------------------------------------------------------------------
# Data sources
# ---------------------------------------------------------------------------

GERMANY_SOURCES = {
    # GADM Germany level 3 (Gemeinden — ~11,000 units)
    "gadm_level3": "https://geodata.ucdavis.edu/gadm/gadm4.1/json/gadm41_DEU_3.json",
    # GADM Germany level 2 (Landkreise — ~400 units, faster for prototype)
    "gadm_level2": "https://geodata.ucdavis.edu/gadm/gadm4.1/json/gadm41_DEU_2.json",
    # Destatis ZENSUS 2021 Gemeinden population
    "destatis_pop": "https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Bevoelkerung/Zensus/Publikationen/Downloads-Zensus/zensus-daten-gemeinden-pdf.html",
}

# ---------------------------------------------------------------------------
# Boundary acquisition
# ---------------------------------------------------------------------------

def download_germany_boundaries(use_level: int = 2) -> gpd.GeoDataFrame:
    """
    Download Germany administrative boundaries.

    Level 2 (Landkreise + kreisfreie Städte, ~400 units): fast prototype
    Level 3 (Gemeinden, ~11,000 units): publication quality
    """
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    level_key = f"gadm_level{use_level}"
    cache_path = OUT_DIR / f"gadm41_DEU_{use_level}.geojson"

    if cache_path.exists():
        size_mb = cache_path.stat().st_size / 1e6
        print(f"  [CACHE] Using cached: {cache_path} ({size_mb:.1f} MB)")
        return gpd.read_file(cache_path)

    url = GERMANY_SOURCES[level_key]
    print(f"  Downloading GADM Germany level {use_level}...")
    print(f"  URL: {url}")

    try:
        resp = requests.get(url, timeout=300, stream=True)
        resp.raise_for_status()
        total = int(resp.headers.get("content-length", 0))
        chunks, downloaded = [], 0
        for chunk in resp.iter_content(chunk_size=65536):
            chunks.append(chunk)
            downloaded += len(chunk)
            if total:
                print(f"\r  Downloading... {downloaded/1e6:.1f}/{total/1e6:.1f} MB", end="")
        print()
        content = b"".join(chunks)
        cache_path.write_bytes(content)
        gdf = gpd.read_file(cache_path)
        print(f"  [OK] {len(gdf)} units downloaded")
        return gdf
    except Exception as e:
        print(f"  [WARN] Download failed: {e}")
        print("  Building Landkreis-level synthetic fallback...")
        return _build_synthetic_fallback()


def _build_synthetic_fallback() -> gpd.GeoDataFrame:
    """
    Construct a simplified 16-state (Bundesland) level GeoDataFrame as fallback.
    Germany bounding box: lon 5.87°E–15.04°E, lat 47.27°N–55.06°N
    """
    from shapely.geometry import box

    # Approximate Bundesland bounding boxes (lon_min, lat_min, lon_max, lat_max)
    bundeslaender = {
        "Baden-Württemberg":       (7.50, 47.53, 10.50, 49.79),
        "Bayern":                  (9.73, 47.27, 13.84, 50.56),
        "Berlin":                  (13.09, 52.34, 13.76, 52.68),
        "Brandenburg":             (11.27, 51.36, 14.77, 53.56),
        "Bremen":                  (8.48, 53.01, 8.99, 53.61),
        "Hamburg":                 (9.73, 53.39, 10.33, 53.96),
        "Hessen":                  (7.77, 49.40, 10.24, 51.66),
        "Mecklenburg-Vorpommern":  (10.59, 53.11, 14.41, 54.69),
        "Niedersachsen":           (6.65, 51.30, 11.60, 53.89),
        "Nordrhein-Westfalen":     (5.87, 50.32, 9.46, 52.53),
        "Rheinland-Pfalz":         (6.12, 49.12, 8.51, 50.94),
        "Saarland":                (6.36, 49.11, 7.40, 49.64),
        "Sachsen":                 (11.87, 50.17, 15.04, 51.68),
        "Sachsen-Anhalt":          (10.56, 51.04, 13.18, 53.04),
        "Schleswig-Holstein":      (7.87, 53.36, 11.31, 55.06),
        "Thüringen":               (9.87, 50.20, 12.65, 51.65),
    }

    # Approximate population share per Bundesland (from 2021 census)
    bundesland_pop_share = {
        "Nordrhein-Westfalen": 0.217, "Bayern": 0.154, "Baden-Württemberg": 0.132,
        "Niedersachsen": 0.096, "Hessen": 0.075, "Sachsen": 0.048,
        "Rheinland-Pfalz": 0.049, "Berlin": 0.043, "Schleswig-Holstein": 0.034,
        "Brandenburg": 0.030, "Sachsen-Anhalt": 0.027, "Thüringen": 0.026,
        "Hamburg": 0.022, "Mecklenburg-Vorpommern": 0.020, "Saarland": 0.012,
        "Bremen": 0.008,
    }

    rows = []
    for i, (name, (lon0, lat0, lon1, lat1)) in enumerate(bundeslaender.items()):
        pop_share = bundesland_pop_share.get(name, 1/16)
        pop = int(GERMANY_TOTAL_POP_CITIZENS * pop_share)
        rows.append({
            "GEOID": f"DE{i+1:04d}000000",
            "bundesland": name,
            "population": pop,
            "geometry": box(lon0, lat0, lon1, lat1),
        })

    gdf = gpd.GeoDataFrame(rows, crs="EPSG:4326")
    out = OUT_DIR / "de_bundeslaender_synthetic.geojson"
    gdf.to_file(out, driver="GeoJSON")
    print(f"  [OK] Built synthetic {len(gdf)}-Bundesland GeoDataFrame")
    print("  [WARN] Using Bundesland-level data — download GADM level 2/3 for publication")
    return gdf


# ---------------------------------------------------------------------------
# Population data
# ---------------------------------------------------------------------------

def attach_population(gdf: gpd.GeoDataFrame) -> gpd.GeoDataFrame:
    """Attach German citizen population data (electoral basis)."""
    if "population" in gdf.columns and gdf["population"].sum() > 0:
        return gdf

    # For GADM level 2: try to infer population from known Landkreis sizes
    # Full data: download from Destatis ZENSUS 2021
    n = len(gdf)
    avg_pop = GERMANY_TOTAL_POP_CITIZENS // n
    print(f"  [INFO] Using uniform population ({avg_pop:,} per unit).")
    print("  For accurate results: download ZENSUS 2021 from destatis.de")
    gdf["population"] = avg_pop
    return gdf


# ---------------------------------------------------------------------------
# Adjacency graph construction
# ---------------------------------------------------------------------------

def build_adjacency(gdf: gpd.GeoDataFrame) -> dict:
    """Build adjacency graph from GeoDataFrame."""
    try:
        gdf_proj = gdf.to_crs("EPSG:25832")  # UTM zone 32N (ETRS89, standard for Germany)
    except Exception:
        gdf_proj = gdf.to_crs("EPSG:32632")  # WGS84 UTM zone 32N fallback

    n = len(gdf_proj)
    geoids = gdf_proj["GEOID"].tolist() if "GEOID" in gdf_proj.columns else [f"DE{i:010d}" for i in range(n)]
    populations = gdf_proj["population"].tolist()

    print(f"  Building adjacency for {n} units...")

    adjacency = [[] for _ in range(n)]
    edge_weights = {}

    if n > 2000:
        # Vectorized spatial join for large datasets
        buffered = gdf_proj.copy()
        buffered["geometry"] = gdf_proj.geometry.buffer(1)
        joined = gpd.sjoin(
            buffered.reset_index().rename(columns={"index": "idx_left"}),
            buffered.reset_index().rename(columns={"index": "idx_right"})[["idx_right", "geometry"]],
            how="left", predicate="intersects"
        )
        for _, row in joined.iterrows():
            i, j = int(row["idx_left"]), int(row["idx_right"])
            if i >= j:
                continue
            shared = gdf_proj.geometry.iloc[i].boundary.intersection(gdf_proj.geometry.iloc[j].boundary)
            length = shared.length if not shared.is_empty else 0
            if length > 0:
                adjacency[i].append(j)
                adjacency[j].append(i)
                edge_weights[(i, j)] = length
    else:
        for i in range(n):
            for j in range(i + 1, n):
                try:
                    if gdf_proj.geometry.iloc[i].touches(gdf_proj.geometry.iloc[j]):
                        shared = gdf_proj.geometry.iloc[i].boundary.intersection(gdf_proj.geometry.iloc[j].boundary)
                        length = shared.length if not shared.is_empty else 1.0
                        if length > 0:
                            adjacency[i].append(j)
                            adjacency[j].append(i)
                            edge_weights[(i, j)] = length
                except Exception:
                    pass

    # Check for isolated units (enclaves like Berlin surrounded by Brandenburg)
    isolated = [i for i, nbrs in enumerate(adjacency) if not nbrs]
    if isolated:
        print(f"  [INFO] {len(isolated)} isolated units (enclaves). Adding nearest-neighbor bridges.")
        for iso in isolated:
            g = gdf_proj.geometry.iloc[iso]
            dists = sorted((gdf_proj.geometry.iloc[j].distance(g), j) for j in range(n) if j != iso)
            nearest = dists[0][1]
            adjacency[iso].append(nearest)
            adjacency[nearest].append(iso)
            edge_weights[(min(iso, nearest), max(iso, nearest))] = 1.0

    print(f"  [OK] {n} units, {len(edge_weights)} edges")

    return {
        "adjacency": adjacency,
        "vertex_weights": [int(p) for p in populations],
        "edge_weights": edge_weights,
        "index_to_geoid": {i: geoids[i] for i in range(n)},
    }


# ---------------------------------------------------------------------------
# .adj.bin conversion
# ---------------------------------------------------------------------------

def convert_to_bin(pkl_path: Path) -> Path | None:
    try:
        import redist_py
        with open(pkl_path, "rb") as f:
            d = pickle.load(f)
        adj = d["adjacency"]
        vw = [int(x) for x in d["vertex_weights"]]
        ew = {(min(i, j), max(i, j)): float(w) for (i, j), w in d.get("edge_weights", {}).items()}
        ig = {int(k): str(v) for k, v in d.get("index_to_geoid", {}).items()}
        bin_data = redist_py.adjacency_to_bin(adj, vw, ew, len(adj), len(ew))
        bin_path = pkl_path.with_suffix(".adj.bin")
        bin_path.write_bytes(bin_data)
        geoid_path = pkl_path.with_name("de_adjacency_2021_geoids.json")
        geoid_path.write_text(json.dumps({str(k): v for k, v in ig.items()}))
        print(f"  [OK] {bin_path} ({len(bin_data)/1e3:.1f} KB)")
        return bin_path
    except ImportError:
        print("  [WARN] redist_py not available — .adj.bin not generated")
        print("  Build with: cd redist/python/redist_py && maturin develop")
        return None


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Acquire Germany census data for E.6 experiments")
    parser.add_argument("--check", action="store_true", help="Check data sources only")
    parser.add_argument("--run", action="store_true", help="Also run redistricting after acquisition")
    parser.add_argument("--level", type=int, default=2, choices=[2, 3],
                        help="GADM level: 2=Landkreise (~400 units), 3=Gemeinden (~11K units)")
    args = parser.parse_args()

    print("=== Germany Data Acquisition (E.6 International Redistricting) ===")
    print(f"Output directory: {OUT_DIR.resolve()}\n")

    if args.check:
        print(f"Wahlkreise: {GERMANY_WAHLKREISE}")
        print(f"Electoral population (citizens): {GERMANY_TOTAL_POP_CITIZENS:,}")
        print(f"Ideal population per Wahlkreis: {GERMANY_IDEAL_POP:,}")
        print(f"Balance tolerance: ±25% (Bundeswahlgesetz §3)")
        print(f"GADM level {args.level}: {'~400 Landkreise' if args.level==2 else '~11,000 Gemeinden'}")
        return

    print(f"Step 1: Downloading GADM Germany level {args.level}...")
    gdf = download_germany_boundaries(use_level=args.level)
    if "GEOID" not in gdf.columns:
        col = next((c for c in gdf.columns if "GID" in c.upper()), None)
        if col:
            gdf["GEOID"] = gdf[col].apply(lambda x: f"DE{str(x).replace('.','')[:10].zfill(10)}")
        else:
            gdf["GEOID"] = [f"DE{i:010d}" for i in range(len(gdf))]

    print("\nStep 2: Attaching population...")
    gdf = attach_population(gdf)
    print(f"  Total pop: {gdf['population'].sum():,}")

    print("\nStep 3: Building adjacency...")
    graph_dict = build_adjacency(gdf)

    print("\nStep 4: Saving...")
    pkl_path = OUT_DIR / "de_adjacency_2021.pkl"
    with open(pkl_path, "wb") as f:
        pickle.dump(graph_dict, f)
    print(f"  [OK] {pkl_path}")

    bin_path = convert_to_bin(pkl_path)

    print(f"\n=== Germany Acquisition Complete ===")
    print(f"  Units: {len(graph_dict['adjacency'])}")
    print(f"  Edges: {len(graph_dict['edge_weights'])}")
    print(f"  Total pop: {sum(graph_dict['vertex_weights']):,}")
    print(f"  Target: {GERMANY_WAHLKREISE} Wahlkreise, ±25% tolerance")
    print(f"\nNext — run redistricting:")
    print(f"  redist state --state DE --year 2021 --version international \\")
    print(f"    --adjacency {bin_path or pkl_path} --state-name germany \\")
    print(f"    --districts {GERMANY_WAHLKREISE} --seats-per-district 1 \\")
    print(f"    --chamber parliamentary --balance-tolerance 25 --label germany_bundestag_2021")

    if args.run and bin_path:
        print("\nStep 5: Running redistricting...")
        redist = next((p for p in [
            Path("redist/target/release/redist.exe"),
            Path("redist/target/release/redist"),
        ] if p.exists()), None)
        if redist:
            result = subprocess.run([
                str(redist), "state",
                "--state", "DE", "--year", "2020", "--version", "international",
                "--adjacency", str(bin_path), "--state-name", "germany",
                "--districts", str(GERMANY_WAHLKREISE),
                "--chamber", "parliamentary", "--balance-tolerance", "25",
                "--label", "germany_bundestag_2021", "--force",
            ], capture_output=True, text=True)
            print(result.stderr)
        else:
            print("  [WARN] redist binary not found")


if __name__ == "__main__":
    main()
