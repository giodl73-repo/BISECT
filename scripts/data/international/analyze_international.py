"""
Analysis script for international redistricting results (E.6 paper).

Computes key metrics from the plan outputs without requiring US TIGER data:
  - Population balance (max deviation, per-district stats)
  - Contiguity (BFS on adjacency graph)
  - Basic compactness proxy (Polsby-Popper using GADM geometries)
  - Summary table for LaTeX

Usage:
    python scripts/data/international/analyze_international.py ireland
    python scripts/data/international/analyze_international.py germany
    python scripts/data/international/analyze_international.py both
"""

import json
import pickle
import sys
from collections import defaultdict, deque
from pathlib import Path

try:
    import geopandas as gpd
    import numpy as np
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False


def load_assignments(label: str, version: str = "international", year: str = "2020") -> dict:
    """Load final_assignments.json for a plan label."""
    path = Path(f"outputs/{version}/{year}/plans/{label}/data/final_assignments.json")
    if not path.exists():
        # Try flat layout
        path = Path(f"outputs/{version}/{year}/plans/{label}/final_assignments.json")
    if not path.exists():
        raise FileNotFoundError(f"Assignments not found for {label}")
    with open(path) as f:
        raw = json.load(f)
    # Keys are unit indices (strings), values are district IDs (1-based ints)
    return {str(k): int(v) for k, v in raw.items()}


def load_pkl(pkl_path: Path) -> dict:
    """Load adjacency pickle."""
    with open(pkl_path, "rb") as f:
        return pickle.load(f)


def check_contiguity(assignments: dict, adjacency: list, num_districts: int) -> dict:
    """BFS contiguity check using adjacency list."""
    # Group units by district
    district_units = defaultdict(set)
    for unit_idx, district_id in assignments.items():
        district_units[district_id].add(int(unit_idx))

    results = {}
    all_contiguous = True

    for d in range(1, num_districts + 1):
        members = district_units.get(d, set())
        if not members:
            results[d] = {"contiguous": True, "components": 0, "units": 0}
            continue

        # BFS from any starting member
        start = next(iter(members))
        visited = {start}
        queue = deque([start])
        while queue:
            node = queue.popleft()
            if node < len(adjacency):
                for neighbor in adjacency[node]:
                    if neighbor in members and neighbor not in visited:
                        visited.add(neighbor)
                        queue.append(neighbor)

        components = 1 if visited == members else (
            # Count actual components
            _count_components(members, adjacency)
        )
        contiguous = (components == 1)
        if not contiguous:
            all_contiguous = False
        results[d] = {"contiguous": contiguous, "components": components, "units": len(members)}

    return {"all_contiguous": all_contiguous, "districts": results}


def _count_components(members: set, adjacency: list) -> int:
    """Count connected components among members."""
    remaining = set(members)
    components = 0
    while remaining:
        components += 1
        start = next(iter(remaining))
        visited = {start}
        queue = deque([start])
        while queue:
            node = queue.popleft()
            if node < len(adjacency):
                for neighbor in adjacency[node]:
                    if neighbor in remaining and neighbor not in visited:
                        visited.add(neighbor)
                        queue.append(neighbor)
        remaining -= visited
    return components


def compute_population_stats(assignments: dict, vertex_weights: list, num_districts: int) -> dict:
    """Compute population balance statistics."""
    district_pops = defaultdict(int)
    for unit_idx, district_id in assignments.items():
        idx = int(unit_idx)
        if idx < len(vertex_weights):
            district_pops[district_id] += vertex_weights[idx]

    total_pop = sum(vertex_weights)
    ideal = total_pop / num_districts

    deviations = {}
    for d in range(1, num_districts + 1):
        pop = district_pops.get(d, 0)
        dev = abs(pop - ideal) / ideal if ideal > 0 else 0.0
        deviations[d] = {"population": pop, "deviation_pct": dev * 100}

    max_dev = max(v["deviation_pct"] for v in deviations.values()) if deviations else 0
    mean_dev = sum(v["deviation_pct"] for v in deviations.values()) / len(deviations) if deviations else 0
    empty_districts = sum(1 for v in deviations.values() if v["population"] == 0)

    return {
        "total_pop": total_pop,
        "ideal_pop": ideal,
        "num_districts": num_districts,
        "max_deviation_pct": max_dev,
        "mean_deviation_pct": mean_dev,
        "empty_districts": empty_districts,
        "balance_valid_5pct": max_dev <= 5.0,
        "balance_valid_25pct": max_dev <= 25.0,
    }


def compute_compactness_gadm(assignments: dict, gdf_path: Path) -> dict:
    """Compute Polsby-Popper using GADM geometries."""
    if not HAS_NUMPY:
        return {"error": "numpy not available"}
    try:
        import geopandas as gpd
        from shapely.ops import unary_union
        import math

        gdf = gpd.read_file(gdf_path)
        # Project to equal-area for accurate metrics
        try:
            gdf_proj = gdf.to_crs("EPSG:3035")  # ETRS89 LAEA Europe
        except Exception:
            gdf_proj = gdf.to_crs("EPSG:32629")  # UTM zone 29N

        n = len(gdf_proj)
        # Group geometries by district
        district_geoms = defaultdict(list)
        for unit_idx, district_id in assignments.items():
            idx = int(unit_idx)
            if idx < n:
                district_geoms[district_id].append(gdf_proj.geometry.iloc[idx])

        pp_scores = []
        for d in sorted(district_geoms.keys()):
            geoms = district_geoms[d]
            merged = unary_union(geoms)
            if merged.is_empty:
                continue
            area = merged.area
            perimeter = merged.length
            if perimeter > 0:
                pp = 4 * math.pi * area / (perimeter ** 2)
                pp_scores.append(min(pp, 1.0))

        if pp_scores:
            return {
                "mean_polsby_popper": float(np.mean(pp_scores)),
                "min_polsby_popper": float(np.min(pp_scores)),
                "max_polsby_popper": float(np.max(pp_scores)),
                "num_districts_scored": len(pp_scores),
                "note": "computed using GADM dissolved geometries (prototype quality)",
            }
        else:
            return {"error": "no districts with geometry"}
    except Exception as e:
        return {"error": str(e)}


def analyze_country(country: str, label: str, pkl_path: Path, gadm_path: Path, num_districts: int):
    print(f"\n{'='*60}")
    print(f"{country.upper()}")
    print(f"{'='*60}")

    # Load data
    graph = load_pkl(pkl_path)
    adjacency = graph["adjacency"]
    vertex_weights = graph["vertex_weights"]
    n_units = len(adjacency)
    n_edges = len(graph["edge_weights"])

    print(f"Graph: {n_units} units, {n_edges} edges")

    try:
        assignments = load_assignments(label)
        print(f"Assignments loaded: {len(assignments)} units -> {len(set(assignments.values()))} districts")
    except FileNotFoundError as e:
        print(f"  ERROR: {e}")
        return {}

    # Population stats
    pop_stats = compute_population_stats(assignments, vertex_weights, num_districts)
    print(f"\nPopulation Balance:")
    print(f"  Total pop: {pop_stats['total_pop']:,}")
    print(f"  Ideal per district: {pop_stats['ideal_pop']:,.0f}")
    print(f"  Max deviation: {pop_stats['max_deviation_pct']:.1f}%")
    print(f"  Mean deviation: {pop_stats['mean_deviation_pct']:.1f}%")
    print(f"  Balance valid (5%): {pop_stats['balance_valid_5pct']}")
    print(f"  Balance valid (25%): {pop_stats['balance_valid_25pct']}")
    if pop_stats["empty_districts"] > 0:
        print(f"  WARNING: {pop_stats['empty_districts']} empty districts")

    # Contiguity
    cont = check_contiguity(assignments, adjacency, num_districts)
    non_contig = [d for d, r in cont["districts"].items() if not r["contiguous"]]
    print(f"\nContiguity:")
    print(f"  All contiguous: {cont['all_contiguous']}")
    if non_contig:
        print(f"  Non-contiguous districts: {non_contig}")

    # Compactness
    if gadm_path.exists():
        print(f"\nCompactness (GADM geometry):")
        comp = compute_compactness_gadm(assignments, gadm_path)
        if "error" in comp:
            print(f"  Error: {comp['error']}")
        else:
            print(f"  Mean Polsby-Popper: {comp['mean_polsby_popper']:.3f}")
            print(f"  Min PP: {comp['min_polsby_popper']:.3f}")
            print(f"  Max PP: {comp['max_polsby_popper']:.3f}")
    else:
        print(f"\nCompactness: GADM file not found at {gadm_path}")
        comp = {}

    # Save analysis JSON
    result = {
        "country": country,
        "label": label,
        "data_source": "GADM v4.1 level 2",
        "units": n_units,
        "edges": n_edges,
        "num_districts": num_districts,
        "population": pop_stats,
        "contiguity": {
            "all_contiguous": cont["all_contiguous"],
            "num_non_contiguous": len(non_contig),
        },
        "compactness": comp,
    }

    out_dir = Path(f"outputs/international/{country.lower()}/analysis")
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / f"{label}_analysis.json"
    with open(out_path, "w") as f:
        json.dump(result, f, indent=2)
    print(f"\n[OK] Analysis saved to {out_path}")

    return result


def print_latex_row(result: dict):
    """Print a LaTeX table row for the paper."""
    c = result.get("country", "?")
    units = result.get("units", 0)
    edges = result.get("edges", 0)
    nd = result.get("num_districts", 0)
    pop = result.get("population", {})
    cont = result.get("contiguity", {})
    comp = result.get("compactness", {})

    max_dev = pop.get("max_deviation_pct", 0)
    all_cont = "Yes" if cont.get("all_contiguous", False) else "No"
    mean_pp = comp.get("mean_polsby_popper", None)
    pp_str = f"{mean_pp:.3f}" if mean_pp is not None else "N/A"

    print(f"\nLaTeX row for {c}:")
    print(f"  {c} & {units} & {edges} & {nd} & {max_dev:.1f}\\% & {all_cont} & {pp_str} \\\\")


def main():
    countries = sys.argv[1:] if len(sys.argv) > 1 else ["both"]
    if countries == ["both"]:
        countries = ["ireland", "germany"]

    results = {}

    if "ireland" in countries:
        results["ireland"] = analyze_country(
            country="ireland",
            label="ireland_dail_2022",
            pkl_path=Path("outputs/international/ireland/ie_adjacency_2022.pkl"),
            gadm_path=Path("outputs/international/ireland/gadm41_IRL_2.geojson"),
            num_districts=43,
        )

    if "germany" in countries:
        results["germany"] = analyze_country(
            country="germany",
            label="germany_bundestag_2021",
            pkl_path=Path("outputs/international/germany/de_adjacency_2021.pkl"),
            gadm_path=Path("outputs/international/germany/gadm41_DEU_2.geojson"),
            num_districts=299,
        )

    print(f"\n{'='*60}")
    print("LATEX TABLE ROWS")
    print(f"{'='*60}")
    for r in results.values():
        if r:
            print_latex_row(r)


if __name__ == "__main__":
    main()
