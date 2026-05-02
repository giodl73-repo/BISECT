"""
B.7: GMPP convergence analysis — parallel to MEC convergence.

For each seed, compute the geometric-mean Polsby-Popper of the level-1
bisection: GMPP = sqrt(PP(left_half) * PP(right_half))

PP(S) = 4*pi*A(S) / P(S)^2  where:
  A(S) = sum of TIGER ALAND (land area, m^2) for tracts in S
  P(S) = sum of external perimeters of tracts in S
        + sum of shared boundary weights for edges crossing S boundary

External perimeter of tract v:
  ext_perim(v) = total_perim(v) - sum(shared_edge_weights(v))
  total_perim(v) ~ 2*sqrt(pi*ALAND(v))  [circular approximation]

Then track: running maximum GMPP across seeds.
Compare to running minimum MEC for same seeds.
"""
import struct, json, math, os

def load_adj(path):
    with open(path,'rb') as f: data=f.read()
    pos=4; _=struct.unpack_from('<I',data,pos)[0]; pos+=4
    n=struct.unpack_from('<I',data,pos)[0]; pos+=4; pos+=4
    vw=[]
    for _ in range(n): vw.append(struct.unpack_from('<q',data,pos)[0]); pos+=8
    adj=[]
    for _ in range(n):
        nb=struct.unpack_from('<I',data,pos)[0]; pos+=4
        nbrs=[struct.unpack_from('<I',data,pos+i*4)[0] for i in range(nb)]; pos+=nb*4
        adj.append(nbrs)
    nw=struct.unpack_from('<I',data,pos)[0]; pos+=4
    ew={}
    for _ in range(nw):
        u=struct.unpack_from('<I',data,pos)[0]; pos+=4
        v=struct.unpack_from('<I',data,pos)[0]; pos+=4
        w=struct.unpack_from('<d',data,pos)[0]; pos+=8
        ew[(min(u,v),max(u,v))]=w
    return n, adj, ew, vw

def load_tiger_areas(code, year="2020"):
    """Load ALAND per tract from TIGER shapefile via the adj.bin geoids."""
    geoid_path = f"outputs/V3/data/2020/adjacency/{code}_adjacency_2020_geoids.json"
    if not os.path.exists(geoid_path): return {}
    idx2geoid = json.load(open(geoid_path))

    # Try to find TIGER .dbf for ALAND — read directly from shapefile attributes
    # We use the units file as a proxy since it has population+geoid
    # Actually load from the tract summary CSV if available
    # Fallback: approximate area from population density (rough)
    # For now return empty — use circular approx from adj topology
    return {}

def compute_ext_perimeters_approx(n, adj, ew, vertex_areas_m2):
    """
    External perimeter per vertex using circular approximation.
    ext_perim(v) = 2*sqrt(pi*area(v)) - sum(shared edges of v)
    Clamped to >= 0.
    """
    ext = []
    for v in range(n):
        a = vertex_areas_m2[v]
        total_perim = 2 * math.sqrt(math.pi * a) if a > 0 else 0
        shared = sum(ew.get((min(v,u), max(v,u)), 0) for u in adj[v])
        ext.append(max(total_perim - shared, 0))
    return ext

def compute_gmpp(left_set, right_set, adj, ew, vertex_areas, vertex_ext_perims):
    """Compute geometric-mean PP of a bisection (left, right)."""
    def subgraph_pp(verts):
        A = sum(vertex_areas[v] for v in verts)
        ext = sum(vertex_ext_perims[v] for v in verts)
        boundary = sum(ew.get((min(u,v),max(u,v)),0)
                      for u in verts for v_nb in [adj[u]] for v in v_nb
                      if v not in verts and (min(u,v),max(u,v)) in ew)
        # Deduplicate boundary edges
        seen = set()
        boundary = 0
        for u in verts:
            for v in adj[u]:
                if v not in verts:
                    key = (min(u,v), max(u,v))
                    if key not in seen:
                        seen.add(key)
                        boundary += ew.get(key, 0)
        P = ext + boundary
        if P <= 0: return 0
        return 4 * math.pi * A / (P * P)

    pp_l = subgraph_pp(left_set)
    pp_r = subgraph_pp(right_set)
    return math.sqrt(pp_l * pp_r), pp_l, pp_r

def run_gmpp_convergence(code, name, k, max_seeds=1000, tiger_aland=None):
    adj_path = f"outputs/V3/data/2020/adjacency/{code}_adjacency_2020.adj.bin"
    if not os.path.exists(adj_path):
        print(f"{code.upper()}: adj.bin not found"); return

    n, adj, ew, vw = load_adj(adj_path)

    # Load TIGER areas — try from a units CSV or use ALAND from shapefile
    # For now: approximate area per tract from total state area / n_tracts
    # This is rough — we'll use TIGER if available
    tiger_path = f"data/2020/tiger/tracts/tl_2020_{get_fips(code)}_tract"
    aland_map = load_aland_from_dbf(tiger_path, code)

    # Build per-vertex area array aligned to adj.bin index
    geoid_path = f"outputs/V3/data/2020/adjacency/{code}_adjacency_2020_geoids.json"
    idx2geoid = json.load(open(geoid_path)) if os.path.exists(geoid_path) else {}
    geoid2idx = {g: int(i) for i, g in idx2geoid.items()}

    vertex_areas = [aland_map.get(idx2geoid.get(str(i), ''), 0) for i in range(n)]
    missing_areas = sum(1 for a in vertex_areas if a == 0)
    if missing_areas > n // 2:
        # Fallback: uniform area estimate from state land area
        state_areas_km2 = {"wi":145746,"pa":119280,"nc":139391,"ga":153910,
                          "mi":147122,"tn":109153,"mn":206232,"tx":676587}
        avg_area_m2 = state_areas_km2.get(code, 100000) * 1e6 / n
        vertex_areas = [avg_area_m2] * n

    vertex_ext_perims = compute_ext_perimeters_approx(n, adj, ew, vertex_areas)

    # Process each seed
    results = []
    run_min_ec = float('inf')
    run_max_gmpp = 0.0
    last_ec_impr = 0
    last_gmpp_impr = 0

    for seed in range(1, max_seeds + 1):
        ver = f"b7_{code.upper()}_s{seed}"
        mpath = f"outputs/{ver}/2020/states/{name}/manifest.json"
        d1path = f"outputs/{ver}/2020/states/{name}/intermediate/depth_01/assignments.json"
        ppath = f"outputs/{ver}/2020/states/{name}/analysis/proportionality.json"

        if not (os.path.exists(mpath) and os.path.exists(d1path)):
            continue

        ec = json.load(open(mpath)).get('edge_cut', 0) / 1000
        if ec <= 0: continue

        d1 = json.load(open(d1path))
        left_set = set(i for i in range(n) if d1.get(str(i)) == 1)
        right_set = set(i for i in range(n) if d1.get(str(i)) == 2)

        gmpp, pp_l, pp_r = compute_gmpp(left_set, right_set, adj, ew, vertex_areas, vertex_ext_perims)

        d_seats = 0
        if os.path.exists(ppath):
            d_seats = json.load(open(ppath)).get('dem_seats', 0)

        ec_new = ec < run_min_ec
        gmpp_new = gmpp > run_max_gmpp

        if ec_new: run_min_ec = ec; last_ec_impr = seed
        if gmpp_new: run_max_gmpp = gmpp; last_gmpp_impr = seed

        results.append({'seed': seed, 'ec': ec, 'gmpp': gmpp,
                        'pp_l': pp_l, 'pp_r': pp_r,
                        'ec_new': ec_new, 'gmpp_new': gmpp_new, 'd': d_seats})

    # Report
    n_done = len(results)
    print(f"\n=== {code.upper()} ({k}D) — {n_done} seeds ===")
    print(f"MEC:  best={run_min_ec:.0f}km  last_impr=seed {last_ec_impr}  "
          f"tail={n_done - last_ec_impr}")
    print(f"GMPP: best={run_max_gmpp:.4f}  last_impr=seed {last_gmpp_impr}  "
          f"tail={n_done - last_gmpp_impr}")

    # Do MEC-best and GMPP-best come from same seed?
    mec_seed = min(results, key=lambda r: r['ec'])
    gmpp_seed = max(results, key=lambda r: r['gmpp'])
    print(f"MEC winner:  seed={mec_seed['seed']} ec={mec_seed['ec']:.0f}km "
          f"gmpp={mec_seed['gmpp']:.4f} D={mec_seed['d']}")
    print(f"GMPP winner: seed={gmpp_seed['seed']} ec={gmpp_seed['ec']:.0f}km "
          f"gmpp={gmpp_seed['gmpp']:.4f} D={gmpp_seed['d']}")
    print(f"Same plan? {'YES' if mec_seed['seed'] == gmpp_seed['seed'] else 'NO'}")

    # Correlation between EC and GMPP
    ecs = [r['ec'] for r in results]
    gmpps = [r['gmpp'] for r in results]
    n_r = len(results)
    mean_ec = sum(ecs)/n_r; mean_gm = sum(gmpps)/n_r
    cov = sum((e-mean_ec)*(g-mean_gm) for e,g in zip(ecs,gmpps))
    var_ec = sum((e-mean_ec)**2 for e in ecs)
    var_gm = sum((g-mean_gm)**2 for g in gmpps)
    corr = cov/math.sqrt(var_ec*var_gm) if var_ec*var_gm > 0 else 0
    print(f"Corr(EC, GMPP) = {corr:.3f}  "
          f"({'lower EC -> higher GMPP' if corr < 0 else 'lower EC -> lower GMPP'})")

    return results

def get_fips(code):
    fips = {"wi":"55","pa":"42","nc":"37","ga":"13","mi":"26",
            "tn":"47","mn":"27","tx":"48"}
    return fips.get(code, "00")

def load_aland_from_dbf(tiger_dir, code):
    """Load ALAND per GEOID from TIGER .dbf file."""
    import struct
    dbf_path = f"{tiger_dir}/tl_2020_{get_fips(code)}_tract.dbf"
    if not os.path.exists(dbf_path): return {}
    aland_map = {}
    try:
        with open(dbf_path, 'rb') as f:
            header = f.read(32)
            n_records = struct.unpack_from('<I', header, 4)[0]
            header_size = struct.unpack_from('<H', header, 8)[0]
            record_size = struct.unpack_from('<H', header, 10)[0]
            # Read field descriptors
            fields = []
            f.seek(32)
            while True:
                field_rec = f.read(32)
                if not field_rec or field_rec[0] == 0x0D: break
                name = field_rec[:11].rstrip(b'\x00').decode('ascii','replace')
                typ = chr(field_rec[11])
                length = field_rec[16]
                fields.append((name, typ, length))
            f.seek(header_size)
            for _ in range(n_records):
                row = f.read(record_size)
                if not row or row[0] == 0x2A: continue  # deleted
                pos = 1
                rec = {}
                for fname, ftype, flen in fields:
                    val = row[pos:pos+flen].decode('ascii','replace').strip()
                    rec[fname] = val
                    pos += flen
                geoid = rec.get('GEOID', rec.get('GEOIDFQ',''))[-11:]
                aland = int(rec.get('ALAND','0') or '0')
                if geoid: aland_map[geoid] = float(aland)
    except Exception as e:
        pass
    return aland_map

if __name__ == '__main__':
    for code, name, k in [("wi","wisconsin",8), ("pa","pennsylvania",17),
                           ("nc","north_carolina",14)]:
        run_gmpp_convergence(code, name, k, max_seeds=200)
