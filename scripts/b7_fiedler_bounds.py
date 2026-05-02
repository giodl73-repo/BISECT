"""Compute Fiedler lower bounds for confirmed MEC states (B.7 paper)."""
import json, struct, math, os, sys

def load_adj_bin(path):
    with open(path, 'rb') as f: data = f.read()
    pos = 4
    _ = struct.unpack_from('<I', data, pos)[0]; pos += 4  # version
    n = struct.unpack_from('<I', data, pos)[0]; pos += 4
    pos += 4  # n_edges_hdr
    for _ in range(n): pos += 8  # vertex weights (i64)
    adj = []
    for _ in range(n):
        nb = struct.unpack_from('<I', data, pos)[0]; pos += 4
        nbrs = [struct.unpack_from('<I', data, pos+i*4)[0] for i in range(nb)]
        pos += nb * 4
        adj.append(nbrs)
    nw = struct.unpack_from('<I', data, pos)[0]; pos += 4
    ew = {}
    for _ in range(nw):
        u = struct.unpack_from('<I', data, pos)[0]; pos += 4
        v = struct.unpack_from('<I', data, pos)[0]; pos += 4
        w = struct.unpack_from('<d', data, pos)[0]; pos += 8
        ew[(min(u,v), max(u,v))] = w
    return n, adj, ew

def fiedler(n, adj, ew, iters=150, tol=1e-6):
    """Deflated power iteration for lambda_2 of weighted Laplacian."""
    deg = [sum(ew.get((min(i,j),max(i,j)),0) for j in adj[i]) for i in range(n)]
    def L(v): return [deg[i]*v[i] - sum(ew.get((min(i,j),max(i,j)),0)*v[j] for j in adj[i]) for i in range(n)]
    def proj(v): m=sum(v)/len(v); return [x-m for x in v]
    def nrm(v): s=math.sqrt(sum(x*x for x in v)); return [x/s for x in v] if s>1e-14 else v
    # lambda_max via power iter on L restricted to 1-perp
    v = nrm(proj([math.sin(i+1) for i in range(n)]))
    lmax = 0
    for _ in range(iters):
        lv = L(v); rq = sum(a*b for a,b in zip(v,lv))
        if abs(rq-lmax)<tol: break
        lmax = rq; v = nrm(proj(lv))
    if lmax < 1e-6: lmax = max(deg)
    # lambda_2 via power iter on (lmax*I - L) on 1-perp
    u = nrm(proj([math.cos(i+1) for i in range(n)]))
    dom = 0
    for _ in range(iters):
        lu = L(u); w = proj([lmax*u[i]-lu[i] for i in range(n)])
        rq = sum(a*b for a,b in zip(u,w))
        if abs(rq-dom)<tol: break
        dom = rq; u = nrm(w)
    return max(lmax-dom, 0)

confirmed = [
    ("pa","pennsylvania",   17, 8, -3.54,  2441000),
    ("ga","georgia",        14, 7, -0.13,  2546000),
    ("mi","michigan",       13, 7,  2.43,  2098000),
    ("tn","tennessee",       9, 1,-27.06,  1568000),
    ("nc","north_carolina", 14, 5,-13.60,  2400000),
    ("wi","wisconsin",       8, 2,-25.32,  1689000),
    ("tx","texas",          38,15, -7.70,  8176000),  # b7tx_ fresh sweep
    ("mn","minnesota",       8, 3,-16.14,  1357000),
]

adj_dir = "outputs/V3/data/2020/adjacency"
print(f"{'State':<5} {'k':>3} {'n':>5}  {'MEC(km)':>8} {'Bound(km)':>9} {'Ratio':>6}  {'MEC plan':<10} {'Gap(pp)':>8}")
print("-" * 60)
for code, name, k, d, gap, mec_m in confirmed:
    p = f"{adj_dir}/{code}_adjacency_2020.adj.bin"
    if not os.path.exists(p):
        print(f"{code.upper()}: adj.bin not found at {p}")
        continue
    n, adj, ew = load_adj_bin(p)
    lam2 = fiedler(n, adj, ew)
    lb_m = lam2 * n / 4
    ratio = mec_m / lb_m if lb_m > 0 else 999
    print(f"{code.upper():<5} {k:>3} {n:>5}  {mec_m/1000:>8.0f} {lb_m/1000:>9.0f} {ratio:>6.2f}x  {d}D/{k-d}R      {gap:>+8.2f}")
print()
print("Ratio = MEC / Fiedler-bound. Closer to 1.0 = tighter certificate.")
print("EC_min >= lambda2 * n/4  (Cheeger, unnormalized Laplacian)")
