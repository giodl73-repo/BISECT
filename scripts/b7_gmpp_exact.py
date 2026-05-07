"""
GMPP convergence with exact TIGER perimeters — corrected analysis.
Replaces the approximate circular perimeter with exact projected perimeters.
"""
import sys, json, math, os
sys.path.insert(0, 'scripts')
from b7_exact_perimeters import get_geometry, load_adj, compute_gmpp_exact

def run_gmpp_exact(code, name, k, max_seeds=1000):
    adj_path = f"outputs/V3/data/2020/adjacency/{code}_adjacency_2020.adj.bin"
    n, adj, ew, vw = load_adj(adj_path)
    areas, ext_perims = get_geometry(code)
    if not areas:
        print(f"{code.upper()}: geometry unavailable"); return

    results = []
    run_min_ec = float('inf')
    run_max_gmpp = 0.0
    last_ec_impr = 0
    last_gmpp_impr = 0

    for seed in range(1, max_seeds + 1):
        ver = f"b7tx_s{seed}" if code == "tx" else f"b7_{code.upper()}_s{seed}"
        mpath = f"outputs/{ver}/2020/states/{name}/manifest.json"
        d1path = f"outputs/{ver}/2020/states/{name}/intermediate/depth_01/assignments.json"
        ppath  = f"outputs/{ver}/2020/states/{name}/analysis/proportionality.json"

        if not (os.path.exists(mpath) and os.path.exists(d1path)): continue

        ec = json.load(open(mpath)).get('edge_cut', 0) / 1000
        if ec <= 0: continue

        d1 = json.load(open(d1path))
        left_set  = set(i for i in range(n) if d1.get(str(i)) == 1)
        right_set = set(i for i in range(n) if d1.get(str(i)) == 2)
        if not left_set or not right_set: continue

        gmpp, pp_l, pp_r = compute_gmpp_exact(left_set, right_set, adj, ew, areas, ext_perims)
        d_seats = json.load(open(ppath)).get('dem_seats', 0) if os.path.exists(ppath) else 0

        ec_new   = ec   < run_min_ec
        gmpp_new = gmpp > run_max_gmpp
        if ec_new:   run_min_ec = ec;     last_ec_impr = seed
        if gmpp_new: run_max_gmpp = gmpp; last_gmpp_impr = seed

        results.append(dict(seed=seed, ec=ec, gmpp=gmpp,
                            pp_l=pp_l, pp_r=pp_r,
                            ec_new=ec_new, gmpp_new=gmpp_new, d=d_seats))

    n_done = len(results)
    mec_w  = min(results, key=lambda r: r['ec'])
    gmpp_w = max(results, key=lambda r: r['gmpp'])

    # Correlation
    ecs = [r['ec'] for r in results]; gmpps = [r['gmpp'] for r in results]
    me = sum(ecs)/n_done; mg = sum(gmpps)/n_done
    cov = sum((e-me)*(g-mg) for e,g in zip(ecs,gmpps))
    ve  = sum((e-me)**2 for e in ecs); vg = sum((g-mg)**2 for g in gmpps)
    corr = cov/math.sqrt(ve*vg) if ve*vg > 0 else 0

    print(f"\n=== {code.upper()} ({k}D) — {n_done} seeds | EXACT TIGER perimeters ===")
    print(f"MEC convergence:  best={run_min_ec:.0f}km  last=seed {last_ec_impr}  tail={n_done-last_ec_impr}")
    print(f"GMPP convergence: best={run_max_gmpp:.4f}  last=seed {last_gmpp_impr}  tail={n_done-last_gmpp_impr}")
    print(f"MEC winner:  seed={mec_w['seed']}  ec={mec_w['ec']:.0f}km  gmpp={mec_w['gmpp']:.4f}  D={mec_w['d']}  PP=({mec_w['pp_l']:.3f},{mec_w['pp_r']:.3f})")
    print(f"GMPP winner: seed={gmpp_w['seed']}  ec={gmpp_w['ec']:.0f}km  gmpp={gmpp_w['gmpp']:.4f}  D={gmpp_w['d']}  PP=({gmpp_w['pp_l']:.3f},{gmpp_w['pp_r']:.3f})")
    print(f"Same plan?   {'YES' if mec_w['seed'] == gmpp_w['seed'] else 'NO'}")
    agree = "YES" if mec_w['d'] == gmpp_w['d'] else f"NO: MEC={mec_w['d']}D GMPP={gmpp_w['d']}D"
    print(f"Outcomes agree?  {agree}")
    print(f"Corr(EC,GMPP) = {corr:.3f}  ({'aligned' if corr < 0 else 'opposed'})")

    return results

if __name__ == '__main__':
    for code, name, k, seeds in [
        ("wi","wisconsin",8,1000),
        ("pa","pennsylvania",17,200),
        ("nc","north_carolina",14,199),
    ]:
        run_gmpp_exact(code, name, k, seeds)
