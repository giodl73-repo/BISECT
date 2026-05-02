"""
Fiedler value (lambda_2) for all focal states using scipy ARPACK.

ARPACK (Arnoldi/Lanczos) with shift-invert is the standard robust method
for sparse eigenvalue problems. It handles clustered eigenvalues correctly,
unlike deflated power iteration which underconverges when lambda_2 ~ lambda_3.

Validation: lambda_2 * n/4 must be <= the achieved level-1 bisection EC.
If it exceeds the achieved EC, the bound is invalid (we have a bug).
"""
import struct, json, math, os
import numpy as np
from scipy.sparse import csr_matrix
from scipy.sparse.linalg import eigsh

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

def build_sparse_laplacian(n, adj, ew):
    """Build sparse weighted Laplacian L = D - W."""
    rows, cols, data = [], [], []
    degree = [0.0] * n
    for (u, v), w in ew.items():
        degree[u] += w
        degree[v] += w
        rows.append(u); cols.append(v); data.append(-w)
        rows.append(v); cols.append(u); data.append(-w)
    for i in range(n):
        rows.append(i); cols.append(i); data.append(degree[i])
    return csr_matrix((data, (rows, cols)), shape=(n, n))

def compute_fiedler(n, adj, ew):
    """
    Compute lambda_2 (Fiedler value) of weighted Laplacian via ARPACK.
    Uses shift-invert mode (sigma=small_positive) to find smallest eigenvalues.
    Returns (lambda2, lambda3) for diagnosing clustering.
    """
    L = build_sparse_laplacian(n, adj, ew)
    # Request 4 eigenvalues: 0 (constant), lambda2, lambda3, lambda4
    # sigma=1e-6 enables shift-invert: finds eigenvalues near 0 efficiently
    try:
        lam = eigsh(L, k=4, which='LM', sigma=1e-6,
                    return_eigenvectors=False, tol=1e-10, maxiter=10000)
        lam_sorted = sorted(abs(l) for l in lam)
        # lam_sorted[0] ~ 0 (constant eigenvector, lambda_1=0)
        lambda2 = lam_sorted[1]
        lambda3 = lam_sorted[2]
        return lambda2, lambda3
    except Exception as e:
        print(f"  eigsh failed: {e}")
        return 0.0, 0.0

def compute_level1_cut(n, adj, ew, vw, ver, name):
    """Load depth_01 intermediate and compute level-1 bisection edge-cut."""
    d1_path = f"outputs/{ver}/2020/states/{name}/intermediate/depth_01/assignments.json"
    if not os.path.exists(d1_path):
        return None, None, None
    d1 = json.load(open(d1_path))
    ec = sum(ew[(min(u,v),max(u,v))]
             for u in range(n) for v in adj[u] if v > u
             if d1.get(str(u)) != d1.get(str(v)))
    count = sum(1 for u in range(n) for v in adj[u] if v > u
                if d1.get(str(u)) != d1.get(str(v)))
    n1 = sum(1 for i in range(n) if d1.get(str(i)) == 1)
    n2 = sum(1 for i in range(n) if d1.get(str(i)) == 2)
    return ec, count, min(n1, n2)

# Focal states: (code, name, k, MEC_seed, MEC_km)
states = [
    ("pa", "pennsylvania",   17, 181,  2441),
    ("ga", "georgia",        14, 489,  2546),
    ("mi", "michigan",       13, 181,  2098),
    ("tn", "tennessee",       9, 609,  1568),
    ("nc", "north_carolina", 14,  41,  2400),
    ("wi", "wisconsin",       8, 891,  1615),
    ("mn", "minnesota",       8, 449,  1357),
    ("tx", "texas",          38, 114,  8176),
]

adj_dir = "outputs/V3/data/2020/adjacency"

print(f"{'St':>3} {'k':>2} {'n':>5}  {'lambda2':>9} {'lambda3':>9} {'L2/L3':>6}  "
      f"{'Bound(km)':>9} {'L1-EC(km)':>9} {'valid?':>7}  {'Ratio':>7}")
print("-" * 85)

for code, name, k, mec_seed, mec_km in states:
    adj_path = f"{adj_dir}/{code}_adjacency_2020.adj.bin"
    if not os.path.exists(adj_path):
        print(f"{code.upper():>3}: not found"); continue

    n, adj, ew, vw = load_adj(adj_path)

    # Compute lambda_2 via scipy ARPACK
    print(f"{code.upper():>3} computing lambda2...", end='\r')
    lambda2, lambda3 = compute_fiedler(n, adj, ew)

    # Fiedler lower bound on minimum balanced bisection
    fiedler_bound_km = lambda2 * n / 4 / 1000

    # Level-1 achieved cut
    ver = f"b7tx_s{mec_seed}" if code == "tx" else f"b7_{code.upper()}_s{mec_seed}"
    l1_ec_m, l1_count, s_size = compute_level1_cut(n, adj, ew, vw, ver, name)
    l1_ec_km = l1_ec_m / 1000 if l1_ec_m else None

    # Validation: bound must be <= achieved (it's a lower bound)
    valid = "OK" if (l1_ec_km and fiedler_bound_km <= l1_ec_km + 1) else "BUG"
    ratio = fiedler_bound_km / l1_ec_km if l1_ec_km else 0
    cluster = lambda2 / lambda3 if lambda3 > 0 else 0

    print(f"{code.upper():>3} {k:>2} {n:>5}  {lambda2:>9.1f} {lambda3:>9.1f} {cluster:>6.3f}  "
          f"{fiedler_bound_km:>9.0f} {(l1_ec_km or 0):>9.0f} {valid:>7}  {ratio:>7.3f}")

print()
print("lambda2/lambda3: clustering ratio. Near 1 = clusters eigenvalues")
print("  (previous power iteration underconverged when this is near 1)")
print("Bound = lambda2 * n/4 (lower bound on min balanced bisection EC)")
print("valid = OK if bound <= achieved level-1 cut (as required for a lower bound)")
print("Ratio = Bound/L1-EC (1.0 = tight; < 1 = loose; > 1 = BUG)")
