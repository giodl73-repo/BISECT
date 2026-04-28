"""
Analyze Germany level 3 redistricting results.
"""
import json
import pickle
from pathlib import Path
from collections import defaultdict, deque
import sys


def load_pkl(pkl_path):
    with open(pkl_path, "rb") as f:
        return pickle.load(f)


def load_assignments(label, version="international", year="2020"):
    path = Path(f"outputs/{version}/{year}/plans/{label}/data/final_assignments.json")
    if not path.exists():
        raise FileNotFoundError(f"Not found: {path}")
    with open(path) as f:
        return {str(k): int(v) for k, v in json.load(f).items()}


def check_contiguity_bfs(assignments, adjacency, num_districts):
    district_units = defaultdict(set)
    for unit_idx, district_id in assignments.items():
        district_units[district_id].add(int(unit_idx))

    all_contiguous = True
    non_contig = []
    for d in range(1, num_districts + 1):
        members = district_units.get(d, set())
        if not members:
            continue
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
        if visited != members:
            all_contiguous = False
            non_contig.append(d)
    return all_contiguous, non_contig


def compute_pop_stats(assignments, vertex_weights, num_districts):
    district_pops = defaultdict(int)
    for unit_idx, district_id in assignments.items():
        idx = int(unit_idx)
        if idx < len(vertex_weights):
            district_pops[district_id] += vertex_weights[idx]

    total = sum(vertex_weights)
    ideal = total / num_districts
    deviations = []
    empty = 0
    for d in range(1, num_districts + 1):
        pop = district_pops.get(d, 0)
        if pop == 0:
            empty += 1
        dev = abs(pop - ideal) / ideal * 100 if ideal > 0 else 0
        deviations.append(dev)

    return {
        "total_pop": total,
        "ideal_pop": ideal,
        "max_dev_pct": max(deviations),
        "mean_dev_pct": sum(deviations) / len(deviations),
        "empty_districts": empty,
        "balance_25pct": max(deviations) <= 25.0,
    }


def compute_pp(assignments, gadm_path, num_districts):
    try:
        import geopandas as gpd
        from shapely.ops import unary_union
        import math

        print("  Loading GADM L3 geometry...")
        gdf = gpd.read_file(gadm_path)
        try:
            gdf_proj = gdf.to_crs("EPSG:25832")
        except Exception:
            gdf_proj = gdf.to_crs("EPSG:32632")

        n = len(gdf_proj)
        district_geoms = defaultdict(list)
        for unit_idx, district_id in assignments.items():
            idx = int(unit_idx)
            if idx < n:
                district_geoms[district_id].append(gdf_proj.geometry.iloc[idx])

        pp_scores = []
        print(f"  Computing PP for {len(district_geoms)} districts...")
        for d in sorted(district_geoms.keys()):
            geoms = district_geoms[d]
            merged = unary_union(geoms)
            if not merged.is_empty:
                area = merged.area
                perimeter = merged.length
                if perimeter > 0:
                    pp = min(4 * math.pi * area / perimeter ** 2, 1.0)
                    pp_scores.append(pp)

        if pp_scores:
            return {
                "mean_pp": sum(pp_scores) / len(pp_scores),
                "min_pp": min(pp_scores),
                "max_pp": max(pp_scores),
                "n": len(pp_scores),
            }
        return {"error": "no PP scores"}
    except Exception as e:
        return {"error": str(e)}


def main():
    print("=== Germany L3 Analysis ===")

    pkl_path = Path("outputs/international/germany/de_adjacency_2021_l3.pkl")
    gadm_path = Path("outputs/international/germany/gadm41_DEU_3.geojson")

    print("Loading adjacency...")
    graph = load_pkl(pkl_path)
    adjacency = graph["adjacency"]
    vertex_weights = graph["vertex_weights"]
    n_units = len(adjacency)
    n_edges = len(graph["edge_weights"])
    print(f"  Units: {n_units}, Edges: {n_edges}")

    print("Loading assignments...")
    try:
        assignments = load_assignments("germany_bundestag_2021")
        print(f"  {len(assignments)} units -> {len(set(assignments.values()))} districts")
    except FileNotFoundError as e:
        print(f"  ERROR: {e}")
        sys.exit(1)

    print("Population balance...")
    pop = compute_pop_stats(assignments, vertex_weights, 299)
    print(f"  Total pop: {pop['total_pop']:,}")
    print(f"  Ideal/district: {pop['ideal_pop']:,.0f}")
    print(f"  Max deviation: {pop['max_dev_pct']:.1f}%")
    print(f"  Balance valid (25%): {pop['balance_25pct']}")
    if pop['empty_districts'] > 0:
        print(f"  WARNING: {pop['empty_districts']} empty districts")

    print("Contiguity...")
    all_cont, non_contig = check_contiguity_bfs(assignments, adjacency, 299)
    print(f"  All contiguous: {all_cont}")
    if non_contig:
        print(f"  Non-contiguous ({len(non_contig)}): {non_contig[:10]}")

    print("Compactness (PP)...")
    comp = compute_pp(assignments, gadm_path, 299)
    if "error" in comp:
        print(f"  Error: {comp['error']}")
    else:
        print(f"  Mean PP: {comp['mean_pp']:.3f}")
        print(f"  Min PP: {comp['min_pp']:.3f}")
        print(f"  Max PP: {comp['max_pp']:.3f}")

    # Save
    result = {
        "country": "germany",
        "label": "germany_bundestag_2021",
        "data_source": "GADM v4.1 level 3 (Gemeinden/Verwaltungsgemeinschaften)",
        "units": n_units,
        "edges": n_edges,
        "num_districts": 299,
        "population": pop,
        "contiguity": {"all_contiguous": all_cont, "num_non_contiguous": len(non_contig)},
        "compactness": comp,
    }

    out_dir = Path("outputs/international/germany/analysis")
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / "germany_bundestag_2021_analysis.json"
    with open(out_path, "w") as f:
        json.dump(result, f, indent=2)
    print(f"\n[OK] Saved to {out_path}")

    max_dev = pop["max_dev_pct"]
    mean_pp = comp.get("mean_pp", None)
    pp_str = f"{mean_pp:.3f}" if mean_pp is not None else "N/A"
    cont_str = "Yes" if all_cont else "No"
    print(f"\nLaTeX row:")
    print(f"  Germany (MMP) & {n_units} & {n_edges} & 299 & {max_dev:.1f}\\% & {cont_str} & {pp_str} \\\\")


if __name__ == "__main__":
    main()
