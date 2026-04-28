"""
Run Germany redistricting with various configurations.

Usage:
    python scripts/data/international/run_germany_only.py
    python scripts/data/international/run_germany_only.py --districts 16 --label germany_16
    python scripts/data/international/run_germany_only.py --districts 299 --label germany_bundestag_2021
    python scripts/data/international/run_germany_only.py --use-level3
"""

import argparse
import subprocess
import sys
from pathlib import Path

REDIST = Path("redist/target/release/redist.exe")
if not REDIST.exists():
    REDIST = Path("redist/target/release/redist")


def run_germany(districts: int, label: str, adj_path: Path, tolerance: float = 25.0):
    """Run Germany redistricting."""
    print(f"=== Germany Redistricting: {districts} districts, tol={tolerance}% ===")
    print(f"  Adjacency: {adj_path} ({adj_path.stat().st_size} bytes)")

    cmd = [
        str(REDIST), "state",
        "--state", "DE",
        "--year", "2020",
        "--version", "international",
        "--adjacency", str(adj_path),
        "--state-name", "germany",
        "--districts", str(districts),
        "--chamber", "parliamentary",
        "--balance-tolerance", str(tolerance),
        "--label", label,
        "--force",
    ]
    print(f"\n[CMD] {' '.join(cmd)}\n")
    result = subprocess.run(cmd, text=True)
    print(f"\nExit code: {result.returncode}")
    return result.returncode == 0


def download_gadm_level3() -> Path:
    """Download GADM Germany level 3 (Gemeinden, ~11K units)."""
    import requests
    import geopandas as gpd

    out = Path("outputs/international/germany")
    out.mkdir(parents=True, exist_ok=True)
    cache = out / "gadm41_DEU_3.geojson"

    if cache.exists() and cache.stat().st_size > 1e6:
        print(f"  [CACHE] {cache} ({cache.stat().st_size/1e6:.1f} MB)")
        return cache

    url = "https://geodata.ucdavis.edu/gadm/gadm4.1/json/gadm41_DEU_3.json"
    print(f"  Downloading GADM Germany level 3 (~150MB, may take several minutes)...")
    r = requests.get(url, timeout=600, stream=True)
    r.raise_for_status()

    total = int(r.headers.get("content-length", 0))
    chunks, downloaded = [], 0
    for chunk in r.iter_content(chunk_size=65536):
        chunks.append(chunk)
        downloaded += len(chunk)
        if total:
            print(f"\r  {downloaded/1e6:.1f}/{total/1e6:.1f} MB", end="")
    print()
    cache.write_bytes(b"".join(chunks))
    print(f"  [OK] Downloaded {cache.stat().st_size/1e6:.1f} MB")
    return cache


def build_adjacency_level3(gdf_path: Path) -> Path:
    """Build adjacency for Germany level 3 Gemeinden."""
    import pickle, json
    import geopandas as gpd

    try:
        import redist_py
        has_redist_py = True
    except ImportError:
        has_redist_py = False

    out = Path("outputs/international/germany")
    pkl_path = out / "de_adjacency_2021_l3.pkl"

    if pkl_path.exists():
        print(f"  [CACHE] {pkl_path}")
        adj_bin = pkl_path.with_suffix(".adj.bin")
        if adj_bin.exists():
            return adj_bin
        with open(pkl_path, "rb") as f:
            graph_dict = pickle.load(f)
    else:
        print(f"  Loading {gdf_path}...")
        gdf = gpd.read_file(gdf_path)
        n = len(gdf)
        print(f"  {n} units")

        # Set GEOID
        if "GEOID" not in gdf.columns:
            col = next((c for c in gdf.columns if "GID" in c.upper()), None)
            if col:
                gdf["GEOID"] = gdf[col].apply(lambda x: f"DE{str(x).replace('.','')[:10].zfill(10)}")
            else:
                gdf["GEOID"] = [f"DE{i:010d}" for i in range(n)]

        # Uniform population
        avg_pop = 73_100_000 // n
        gdf["population"] = avg_pop

        # Project
        try:
            gdf_proj = gdf.to_crs("EPSG:25832")
        except Exception:
            gdf_proj = gdf.to_crs("EPSG:32632")

        geoids = [f"DE{i:010d}" for i in range(n)]
        populations = list(gdf_proj["population"])

        print(f"  Building adjacency for {n} units (vectorized sjoin)...")
        adjacency = [[] for _ in range(n)]
        edge_weights = {}

        buffered = gdf_proj.copy()
        buffered["geometry"] = gdf_proj.geometry.buffer(1)
        buffered = buffered.reset_index().rename(columns={"index": "idx"})

        joined = gpd.sjoin(
            buffered[["idx", "geometry"]].rename(columns={"idx": "idx_left"}),
            buffered[["idx", "geometry"]].rename(columns={"idx": "idx_right"}),
            how="left", predicate="intersects"
        )

        import math
        batch = 0
        for _, row in joined.iterrows():
            i, j = int(row["idx_left"]), int(row["idx_right"])
            if i >= j:
                continue
            try:
                shared = gdf_proj.geometry.iloc[i].boundary.intersection(gdf_proj.geometry.iloc[j].boundary)
                length = shared.length if not shared.is_empty else 0.0
                if length > 0:
                    adjacency[i].append(j)
                    adjacency[j].append(i)
                    edge_weights[(i, j)] = length
            except Exception:
                pass
            batch += 1
            if batch % 10000 == 0:
                print(f"\r  Processed {batch} pairs, {len(edge_weights)} edges found...", end="")
        print()

        # Fix isolated
        isolated = [i for i, nbrs in enumerate(adjacency) if not nbrs]
        if isolated:
            print(f"  {len(isolated)} isolated units — adding bridges")
            for iso in isolated:
                g = gdf_proj.geometry.iloc[iso]
                dists = sorted((gdf_proj.geometry.iloc[j].distance(g), j) for j in range(n) if j != iso)
                nearest = dists[0][1]
                adjacency[iso].append(nearest)
                adjacency[nearest].append(iso)
                edge_weights[(min(iso, nearest), max(iso, nearest))] = 1.0

        graph_dict = {
            "adjacency": adjacency,
            "vertex_weights": [int(p) for p in populations],
            "edge_weights": edge_weights,
            "index_to_geoid": {i: geoids[i] for i in range(n)},
        }
        with open(pkl_path, "wb") as f:
            pickle.dump(graph_dict, f)
        print(f"  [OK] {pkl_path}")

    if has_redist_py:
        adj = graph_dict["adjacency"]
        vw = [int(x) for x in graph_dict["vertex_weights"]]
        ew = {(min(i, j), max(i, j)): float(w) for (i, j), w in graph_dict["edge_weights"].items()}
        bin_data = redist_py.adjacency_to_bin(adj, vw, ew, len(adj), len(ew))
        bin_path = pkl_path.with_suffix(".adj.bin")
        bin_path.write_bytes(bin_data)
        print(f"  [OK] {bin_path} ({len(bin_data)/1e3:.1f} KB)")
        return bin_path
    else:
        print("  [WARN] redist_py not available for .adj.bin conversion")
        return pkl_path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--districts", type=int, default=299)
    parser.add_argument("--label", default="germany_bundestag_2021")
    parser.add_argument("--tolerance", type=float, default=25.0)
    parser.add_argument("--use-level3", action="store_true",
                        help="Download and use GADM level 3 (~11K Gemeinden)")
    args = parser.parse_args()

    if not REDIST.exists():
        print(f"ERROR: redist binary not found at {REDIST}")
        sys.exit(1)

    if args.use_level3:
        print("=== Germany Level 3 (Gemeinden) Pipeline ===")
        gdf_path = download_gadm_level3()
        adj_path = build_adjacency_level3(gdf_path)
    else:
        adj_path = Path("outputs/international/germany/de_adjacency_2021.adj.bin")
        if not adj_path.exists():
            adj_path = Path("outputs/international/germany/de_adjacency_2021.pkl")
        if not adj_path.exists():
            print("ERROR: adjacency not found")
            sys.exit(1)

    ok = run_germany(args.districts, args.label, adj_path, args.tolerance)

    if ok:
        print("\n=== Analysis ===")
        cmd = [
            str(REDIST), "analyze",
            "--label", args.label,
            "--year", "2020",
            "--version", "international",
            "--types", "contiguity",
            "--allow-imbalance",
            "--allow-noncontiguous",
            "--force",
        ]
        print(f"[CMD] {' '.join(cmd)}\n")
        result = subprocess.run(cmd, text=True)
        print(f"Analysis exit code: {result.returncode}")

        # Run Python analysis
        import subprocess as sp
        sp.run([sys.executable, "scripts/data/international/analyze_international.py", "germany"], text=True)
    else:
        print(f"\n[FAIL] Germany redistricting failed")


if __name__ == "__main__":
    main()
