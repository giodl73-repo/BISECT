"""Test exact GMPP computation for WI MEC winner."""
import sys, json, math
sys.path.insert(0, 'scripts')
from b7_exact_perimeters import get_geometry, load_adj, compute_gmpp_exact

code, name = "wi", "wisconsin"
n, adj, ew, vw = load_adj(f"outputs/V3/data/2020/adjacency/{code}_adjacency_2020.adj.bin")
areas, ext_perims = get_geometry(code)
print(f"Geometry loaded: {sum(1 for a in areas if a>0)} tracts with area, "
      f"{sum(1 for p in ext_perims if p>0)} with nonzero ext_perim")

# MEC winner (seed 891, level-1 bisection)
d1 = json.load(open("outputs/b7_WI_s891/2020/states/wisconsin/intermediate/depth_01/assignments.json"))
left_set  = set(i for i in range(n) if d1.get(str(i)) == 1)
right_set = set(i for i in range(n) if d1.get(str(i)) == 2)

gmpp, pp_l, pp_r = compute_gmpp_exact(left_set, right_set, adj, ew, areas, ext_perims)

# Compute perimeters of each half
def half_perim(verts):
    ext = sum(ext_perims[v] for v in verts)
    seen = set()
    cut = 0
    for u in verts:
        for v in adj[u]:
            if v not in verts:
                key = (min(u,v),max(u,v))
                if key not in seen:
                    seen.add(key); cut += ew.get(key,0)
    return ext, cut, ext+cut

A_l = sum(areas[v] for v in left_set)
A_r = sum(areas[v] for v in right_set)
ext_l, cut_l, P_l = half_perim(left_set)
ext_r, cut_r, P_r = half_perim(right_set)

print(f"\nWI MEC winner (seed 891) level-1 bisection:")
print(f"  Left half:  area={A_l/1e9:.2f}km2  border={ext_l/1000:.0f}km  cut={cut_l/1000:.0f}km  P={P_l/1000:.0f}km  PP={pp_l:.4f}")
print(f"  Right half: area={A_r/1e9:.2f}km2  border={ext_r/1000:.0f}km  cut={cut_r/1000:.0f}km  P={P_r/1000:.0f}km  PP={pp_r:.4f}")
print(f"  GMPP = {gmpp:.4f}  (in [0,1]: {0 <= pp_l <= 1 and 0 <= pp_r <= 1})")
print(f"\n  WI total area: {(A_l+A_r)/1e9:.0f} km2 (state is ~145746 km2)")
print(f"  Total state border: {(ext_l+ext_r)/1000:.0f} km")
print(f"  Level-1 cut: {cut_l/1000:.0f} km (both halves share same cut)")
