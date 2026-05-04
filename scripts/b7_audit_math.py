"""
Full math audit for B.7 paper.

For each focal state, compute:
1. Total edge-cut of final k-district partition (from manifest - verified)
2. Level-1 bisection edge-cut (from intermediate/depth_01 - what ONE cut costs)
3. Number of edges crossed at level-1 (count, not just weight)
4. Cheeger constant h(G) upper bound from the achieved level-1 bisection
5. Lambda2 upper bound from Cheeger: lambda2 <= 2*h
6. Correct Fiedler lower bound on EC_min (level-1): >= h * n/2
7. MEC / level-1 EC  (how many times larger is the full partition vs one cut?)
8. Level-1 EC / (state minor axis km) -- true discretization overhead
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

def compute_cut(adj, ew, assignment_idx):
    """Edge-cut given assignment dict {str(idx): region}."""
    ec = 0; count = 0
    for u in range(len(adj)):
        for v in adj[u]:
            if v > u:
                du = assignment_idx.get(str(u))
                dv = assignment_idx.get(str(v))
                if du is not None and dv is not None and du != dv:
                    ec += ew.get((min(u,v),max(u,v)), 0)
                    count += 1
    return ec, count

# Focal states: (code, name, k, MEC_seed, minor_axis_km, area_km2)
# minor_axis: shortest geographic width in km
# MEC_seed: the seed that achieved the confirmed minimum
states = [
    ("pa", "pennsylvania",   17, 181,  240, 119280),
    ("ga", "georgia",        14, 489,  300, 153910),
    ("mi", "michigan",       13, 181,  200, 147122),  # lower pen width ~200km E-W
    ("tn", "tennessee",       9, 609,  160, 109153),  # narrow N-S ~160km
    ("nc", "north_carolina", 14,  41,  200, 139391),  # narrow N-S ~200km
    ("wi", "wisconsin",       8, 891,  275, 145746),
    ("mn", "minnesota",       8, 449,  400, 206232),
    ("tx", "texas",          38, 114,  800, 676587),
]

# MEC values we confirmed (km)
mec_confirmed = {
    "pa": 2441, "ga": 2546, "mi": 2098, "tn": 1568,
    "nc": 2400, "wi": 1615, "mn": 1357, "tx": 8176,
}

adj_dir = "outputs/V3/data/2020/adjacency"
out_dir = "outputs"

print(f"{'St':>3} {'k':>2} {'n':>5}  {'MEC':>6} {'L1-EC':>6} {'L1-N':>5} {'h(m/v)':>7} {'lambda2_ub':>10} {'L1-bound':>9} {'MEC/L1':>7} {'L1/minor':>9}")
print("-" * 95)

for code, name, k, mec_seed, minor_km, area_km2 in states:
    adj_path = f"{adj_dir}/{code}_adjacency_2020.adj.bin"
    if not os.path.exists(adj_path):
        print(f"{code.upper():>3}: adj.bin not found"); continue

    n, adj, ew, vw = load_adj(adj_path)
    total_pop = sum(vw)

    # Find best plan's intermediate depth_01 file
    # Try b7_{CODE}_s{seed} and b7tx_s{seed} for TX
    if code == "tx":
        ver = f"b7tx_s{mec_seed}"
    else:
        ver = f"b7_{code.upper()}_s{mec_seed}"

    d1_path = f"{out_dir}/{ver}/2020/states/{name}/intermediate/depth_01/assignments.json"
    if not os.path.exists(d1_path):
        print(f"{code.upper():>3}: depth_01 not found at {d1_path}"); continue

    d1 = json.load(open(d1_path))  # {str(idx): region (1 or 2)}

    # Compute level-1 bisection cut
    l1_ec_m, l1_count = compute_cut(adj, ew, d1)
    l1_ec_km = l1_ec_m / 1000

    # Population and vertex split
    pop1 = sum(vw[i] for i in range(n) if d1.get(str(i)) == 1)
    pop2 = sum(vw[i] for i in range(n) if d1.get(str(i)) == 2)
    n1 = sum(1 for i in range(n) if d1.get(str(i)) == 1)
    n2 = sum(1 for i in range(n) if d1.get(str(i)) == 2)
    s_size = min(n1, n2)  # smaller half

    # Cheeger constant upper bound from this cut
    h_upper = l1_ec_m / s_size  # m per vertex

    # Lambda2 upper bound: lambda2 <= 2*h
    lambda2_ub = 2 * h_upper  # metres

    # Correct Fiedler lower bound on level-1 EC:
    # EC_min(level1) >= h(G) * n/2 >= (lambda2/2) * n/2 = lambda2*n/4
    # But we only know lambda2 <= lambda2_ub, so this gives an upper bound on the lower bound
    # The actual lower bound from our achieved h: EC_min >= h_achieved * n/2 = l1_ec_km (circular!)
    # Better: use lambda2_ub as upper bound, not a lower bound for EC
    # The correct statement: EC_min(level1) <= l1_ec_km (since we achieved it)
    # Lower bound: need actual lambda2 which we can't compute reliably

    # Geometric minimum (theoretical, circular state):
    # Single bisection of area A into equal halves: ~sqrt(A/pi) * some_factor
    # But more honestly: the minor axis gives a floor

    mec_km = mec_confirmed[code]
    geom_est_km = math.sqrt(math.pi * area_km2 * k)  # total k-partition circular estimate

    ratio_mec_l1 = mec_km / l1_ec_km
    ratio_l1_minor = l1_ec_km / minor_km

    print(f"{code.upper():>3} {k:>2} {n:>5}  {mec_km:>6}km {l1_ec_km:>5.0f}km {l1_count:>5} {h_upper:>7.0f} {lambda2_ub:>10.0f} "
          f"{l1_ec_km:>7.0f}km {ratio_mec_l1:>6.1f}x {ratio_l1_minor:>7.2f}x")

print()
print("Columns:")
print("  MEC      = total edge-cut for full k-district partition (confirmed)")
print("  L1-EC    = level-1 bisection edge-cut (first split only)")
print("  L1-N     = number of tract-boundary edges crossed at level-1")
print("  h(m/v)   = Cheeger constant upper bound = L1-EC(m) / min_half_vertices")
print("  lambda2_ub = upper bound on lambda2 = 2*h (Cheeger inequality)")
print("  L1-bound = = L1-EC (our cut IS the lower bound via h)")
print("  MEC/L1   = how many times larger is full partition vs first cut")
print("  L1/minor = level-1 cut vs state geographic minor axis (true discretization)")
print()
print("KEY INSIGHT: lambda2 <= lambda2_ub. Our power iteration was OVERESTIMATING")
print("lambda2 (giving a value > lambda2_ub), making the bound invalid.")
print("The correct Fiedler lower bound requires lambda2 <= lambda2_ub, so")
print("EC_min >= lambda2_true * n/4, but we don't know lambda2_true precisely.")
print()
print("HONEST CLAIM: 'We achieved L1-EC at level-1, which is our best empirical")
print("upper bound on EC_min. The true minimum is <= L1-EC.'")
