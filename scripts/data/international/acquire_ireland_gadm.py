"""
Ireland GADM level 2 (counties) fallback acquisition for E.6.

Uses GADM v4.1 Ireland level 2 (26 counties) as the geographic units.
This is faster and more reliable than CSO Small Area boundaries.

Usage:
    python scripts/data/international/acquire_ireland_gadm.py
    python scripts/data/international/acquire_ireland_gadm.py --check
"""

import argparse
import json
import pickle
import sys
from pathlib import Path

try:
    import geopandas as gpd
    import requests
except ImportError as e:
    print(f"ERROR: Missing dependency: {e}")
    sys.exit(1)

OUT_DIR = Path("outputs/international/ireland")

IRELAND_TOTAL_POP_2022 = 5_123_536

# 26 counties with approximate populations (2022 census)
COUNTY_POPS = {
    "Carlow": 61_157,
    "Cavan": 82_771,
    "Clare": 129_549,
    "Cork": 613_294,
    "Donegal": 175_334,
    "Dublin": 1_458_154,
    "Galway": 284_081,
    "Kerry": 158_268,
    "Kildare": 246_977,
    "Kilkenny": 103_902,
    "Laois": 95_212,
    "Leitrim": 38_566,
    "Limerick": 220_749,
    "Longford": 50_071,
    "Louth": 145_491,
    "Mayo": 137_522,
    "Meath": 220_149,
    "Monaghan": 66_716,
    "Offaly": 83_402,
    "Roscommon": 73_238,
    "Sligo": 74_976,
    "Tipperary": 178_234,
    "Waterford": 116_176,
    "Westmeath": 97_590,
    "Wexford": 165_507,
    "Wicklow": 152_039,
}


def download_gadm_ireland() -> gpd.GeoDataFrame:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    url = "https://geodata.ucdavis.edu/gadm/gadm4.1/json/gadm41_IRL_2.json"
    cache = OUT_DIR / "gadm41_IRL_2.geojson"

    if cache.exists():
        print(f"  [CACHE] {cache} ({cache.stat().st_size/1e6:.1f} MB)")
        return gpd.read_file(cache)

    print(f"  Downloading GADM Ireland level 2 (counties)...")
    print(f"  URL: {url}")
    r = requests.get(url, timeout=120, stream=True)
    r.raise_for_status()

    total = int(r.headers.get("content-length", 0))
    chunks, downloaded = [], 0
    for chunk in r.iter_content(chunk_size=65536):
        chunks.append(chunk)
        downloaded += len(chunk)
        if total:
            print(f"\r  Downloading... {downloaded/1e6:.1f}/{total/1e6:.1f} MB", end="")
    print()

    content = b"".join(chunks)
    cache.write_bytes(content)
    gdf = gpd.read_file(cache)
    print(f"  [OK] {len(gdf)} counties downloaded")
    return gdf


def attach_population(gdf: gpd.GeoDataFrame) -> gpd.GeoDataFrame:
    """Attach population data.

    GADM level 2 for Ireland has 106 municipal districts, not 26 counties.
    We use uniform distribution: total pop / n_units.
    For publication-quality results, join Destatis/CSO municipal district populations.
    """
    n = len(gdf)
    avg = IRELAND_TOTAL_POP_2022 // n
    gdf = gdf.copy()
    gdf["population"] = avg
    print(f"  [OK] Uniform population: {avg:,} per unit, total: {gdf['population'].sum():,}")
    print(f"  [INFO] {n} municipal districts (GADM level 2 sub-county divisions)")
    print(f"  [INFO] For publication: join CSO 2022 municipal district population tables")
    return gdf


def build_adjacency(gdf: gpd.GeoDataFrame) -> dict:
    """Build adjacency graph using touches predicate."""
    try:
        gdf_proj = gdf.to_crs("EPSG:2157")  # Irish Transverse Mercator
    except Exception:
        gdf_proj = gdf.to_crs("EPSG:32629")

    n = len(gdf_proj)
    print(f"  Building adjacency for {n} units (brute-force O(n^2), n small)...")

    # GEOID from GID_2 or synthetic
    geoid_col = next((c for c in gdf_proj.columns if "GID_2" in c.upper()), None)
    if geoid_col:
        geoids = [f"IE{str(v).replace('.','')[:10].zfill(10)}" for v in gdf_proj[geoid_col]]
    else:
        geoids = [f"IE{i:010d}" for i in range(n)]

    populations = list(gdf_proj["population"])
    adjacency = [[] for _ in range(n)]
    edge_weights = {}

    for i in range(n):
        for j in range(i + 1, n):
            try:
                # Use both touches and very small buffer to catch shared edges
                gi = gdf_proj.geometry.iloc[i]
                gj = gdf_proj.geometry.iloc[j]
                if gi.touches(gj) or gi.buffer(10).intersects(gj.buffer(10)):
                    shared = gi.boundary.intersection(gj.boundary)
                    length = shared.length if not shared.is_empty else 0.0
                    if length < 1.0:
                        # Check with buffer intersection for near-touching
                        length = 1.0 if gi.buffer(50).intersects(gj.buffer(50)) else 0.0
                    if length > 0:
                        adjacency[i].append(j)
                        adjacency[j].append(i)
                        edge_weights[(i, j)] = length
            except Exception:
                pass

    # Fix isolated units
    isolated = [i for i, nbrs in enumerate(adjacency) if not nbrs]
    if isolated:
        print(f"  [INFO] {len(isolated)} isolated units — adding nearest-neighbor bridges")
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


def save_outputs(graph_dict: dict) -> tuple:
    """Save .pkl and .adj.bin files."""
    pkl_path = OUT_DIR / "ie_adjacency_2022.pkl"
    with open(pkl_path, "wb") as f:
        pickle.dump(graph_dict, f)
    print(f"  [OK] {pkl_path}")

    geoid_path = OUT_DIR / "ie_adjacency_2022_geoids.json"
    ig = graph_dict.get("index_to_geoid", {})
    geoid_path.write_text(json.dumps({str(k): v for k, v in ig.items()}))
    print(f"  [OK] {geoid_path}")

    bin_path = None
    try:
        import redist_py
        adj = graph_dict["adjacency"]
        vw = [int(x) for x in graph_dict["vertex_weights"]]
        ew = {(min(i, j), max(i, j)): float(w) for (i, j), w in graph_dict["edge_weights"].items()}
        bin_data = redist_py.adjacency_to_bin(adj, vw, ew, len(adj), len(ew))
        bin_path = OUT_DIR / "ie_adjacency_2022.adj.bin"
        bin_path.write_bytes(bin_data)
        print(f"  [OK] {bin_path} ({len(bin_data)} bytes)")
    except ImportError:
        print("  [WARN] redist_py not available")

    return pkl_path, bin_path


def main():
    parser = argparse.ArgumentParser(description="Ireland GADM county-level acquisition (E.6 fallback)")
    parser.add_argument("--check", action="store_true", help="Check only")
    parser.add_argument("--force", action="store_true", help="Re-download")
    args = parser.parse_args()

    print("=== Ireland GADM Level 2 Acquisition (E.6 county-level fallback) ===")

    if args.check:
        print(f"Counties: 26, Total pop: {IRELAND_TOTAL_POP_2022:,}")
        print(f"Source: GADM v4.1 gadm41_IRL_2.json")
        print(f"Output: {OUT_DIR.resolve()}")
        return

    if args.force:
        for f in (OUT_DIR / "gadm41_IRL_2.geojson", OUT_DIR / "ie_adjacency_2022.pkl",
                  OUT_DIR / "ie_adjacency_2022.adj.bin"):
            if f.exists():
                f.unlink()
                print(f"  Removed {f}")

    print("Step 1: Download GADM Ireland level 2...")
    gdf = download_gadm_ireland()
    print(f"  Columns: {list(gdf.columns)}")

    print("\nStep 2: Attach population...")
    gdf = attach_population(gdf)

    print("\nStep 3: Build adjacency...")
    graph_dict = build_adjacency(gdf)

    print("\nStep 4: Save outputs...")
    pkl_path, bin_path = save_outputs(graph_dict)

    adj_arg = bin_path or pkl_path
    print(f"\n=== Ireland GADM Acquisition Complete ===")
    print(f"  Data: GADM v4.1 level 2 (counties)")
    print(f"  Units: {len(graph_dict['adjacency'])}")
    print(f"  Edges: {len(graph_dict['edge_weights'])}")
    print(f"  Total pop: {sum(graph_dict['vertex_weights']):,}")
    print(f"\nNext - run redistricting:")
    print(f"  redist state --state IE --year 2020 --version international \\")
    print(f"    --adjacency {adj_arg} --state-name ireland \\")
    print(f"    --districts 43 --seats-per-district 4 \\")
    print(f"    --chamber parliamentary --balance-tolerance 5 \\")
    print(f"    --label ireland_dail_2022 --force")


if __name__ == "__main__":
    main()
