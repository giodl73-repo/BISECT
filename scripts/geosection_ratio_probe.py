"""
GeoSection Phase 1: Empirical validation of ratio search.

For each focal state, try all split ratios at the first bisection level.
Run N seeds per ratio. Find the minimum MEC per ratio.
Plot: does a non-50/50 ratio give a lower MEC than the standard split?

If yes: the state has a "natural" geographic ratio.
If the natural ratio also produces a different partisan outcome: interesting.
If insensitive to ratio: standard 50/50 is already optimal.
"""
import struct, json, math, os, subprocess, tempfile, sys

# ── Load adjacency ──────────────────────────────────────────────────────────

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

# ── Write METIS graph file ──────────────────────────────────────────────────

def write_metis_graph(n, adj, ew, vw, path):
    """Write METIS graph format with vertex and edge weights."""
    n_edges = len(ew)
    with open(path, 'w') as f:
        # Header: n_vertices n_edges format (011 = vertex+edge weights)
        f.write(f"{n} {n_edges} 011\n")
        for i in range(n):
            # vertex weight (population)
            parts = [str(vw[i])]
            # neighbors with edge weights
            for j in adj[i]:
                w = ew.get((min(i,j),max(i,j)), 1)
                parts.append(f"{j+1} {int(round(w))}")  # 1-indexed
            f.write(' '.join(parts) + '\n')

# ── Run METIS bisection with target ratio ────────────────────────────────────

def run_metis_bisection(graph_file, left_frac, seed, gpmetis='gpmetis'):
    """
    Run gpmetis bisection (k=2) with target partition weights.
    left_frac = fraction of population in left half.
    Returns partition list (0 or 1 per vertex), or None on failure.
    """
    right_frac = 1.0 - left_frac
    tpwgts = f"{left_frac:.6f},{right_frac:.6f}"

    with tempfile.TemporaryDirectory() as tmp:
        import shutil
        tmp_graph = os.path.join(tmp, 'graph.metis')
        shutil.copy(graph_file, tmp_graph)

        cmd = [gpmetis, tmp_graph, '2',
               '-seed', str(seed),
               '-tpwgts', tpwgts,
               '-ufactor', '5']

        result = subprocess.run(cmd, capture_output=True, text=True, cwd=tmp)
        if result.returncode != 0:
            return None

        part_file = tmp_graph + '.part.2'
        if not os.path.exists(part_file):
            return None

        with open(part_file) as f:
            parts = [int(line.strip()) for line in f if line.strip()]
        return parts

# ── Compute edge-cut from partition ─────────────────────────────────────────

def compute_ec(parts, adj, ew):
    ec = 0
    for u, nbrs in enumerate(adj):
        for v in nbrs:
            if v > u and parts[u] != parts[v]:
                ec += ew.get((min(u,v),max(u,v)), 0)
    return ec

# ── Main ratio probe ─────────────────────────────────────────────────────────

def probe_ratios(code, name, k, n_seeds=100):
    adj_path = f"outputs/V3/data/2020/adjacency/{code}_adjacency_2020.adj.bin"
    if not os.path.exists(adj_path):
        print(f"{code.upper()}: adj.bin not found"); return

    n, adj, ew, vw = load_adj(adj_path)
    total_pop = sum(vw)

    # Write METIS graph once
    with tempfile.NamedTemporaryFile(suffix='.metis', delete=False, mode='w') as tf:
        graph_file = tf.name
    write_metis_graph(n, adj, ew, vw, graph_file)

    # Find gpmetis
    gpmetis = None
    for candidate in ['gpmetis', 'bin/gpmetis.exe', 'bin/gpmetis',
                       'C:/tools/metis/bin/gpmetis.exe']:
        if subprocess.run(['where' if os.name=='nt' else 'which',
                          candidate.split('/')[-1]],
                         capture_output=True).returncode == 0:
            gpmetis = candidate; break
    if gpmetis is None:
        # Try direct path
        for p in ['bin/gpmetis.exe']:
            if os.path.exists(p): gpmetis = p; break
    if gpmetis is None:
        print("gpmetis not found — using redist state directly instead")
        os.unlink(graph_file)
        return probe_ratios_via_redist(code, name, k, n_seeds)

    print(f"\n=== {code.upper()} k={k}, {n_seeds} seeds/ratio ===")
    print(f"{'Ratio':<8} {'frac':>6} {'min_EC':>8} {'best_seed':>9} {'n_valid':>7}")
    print("-" * 45)

    results = {}
    for i in range(1, k//2 + 1):
        left_frac = i / k
        min_ec = float('inf')
        best_seed = -1
        n_valid = 0

        for seed in range(1, n_seeds + 1):
            parts = run_metis_bisection(graph_file, left_frac, seed, gpmetis)
            if parts is None or len(parts) != n: continue
            n_valid += 1
            ec = compute_ec(parts, adj, ew)
            if ec < min_ec:
                min_ec = ec
                best_seed = seed

        ratio_label = f"{i}:{k-i}"
        results[i] = {'ratio': ratio_label, 'frac': left_frac,
                      'min_ec': min_ec, 'best_seed': best_seed,
                      'n_valid': n_valid}
        print(f"{ratio_label:<8} {left_frac:>6.3f} {min_ec/1000:>7.0f}km {best_seed:>9} {n_valid:>7}")

    # Find natural ratio
    natural = min(results.values(), key=lambda r: r['min_ec'])
    standard_ec = results.get(k//2, {}).get('min_ec', float('inf'))
    print(f"\nNatural ratio: {natural['ratio']} at {natural['min_ec']/1000:.0f}km")
    print(f"Standard {k//2}:{k//2 if k%2==0 else k-k//2} ratio: {standard_ec/1000:.0f}km")
    if natural['min_ec'] < standard_ec:
        savings = (standard_ec - natural['min_ec'])/1000
        pct = savings / (standard_ec/1000) * 100
        print(f"GeoSection saves {savings:.0f}km ({pct:.1f}%) vs standard split")
    else:
        print("Standard split IS the natural ratio (no savings)")

    os.unlink(graph_file)
    return results

def probe_ratios_via_redist(code, name, k, n_seeds=50):
    """Fallback: use redist state with different partition modes."""
    print(f"Using redist state for {code.upper()} (gpmetis not directly accessible)")
    # We can use the existing b7_WI_s* runs to get the 50/50 baseline
    # and need to add a new mode for other ratios
    print("Note: Only 50/50 data available from existing runs.")
    print("Full ratio probe requires GeoSection implementation in Rust.")

if __name__ == '__main__':
    # Start with WI (k=8) — 4 ratios: 1:7, 2:6, 3:5, 4:4
    probe_ratios("wi", "wisconsin", 8, n_seeds=100)
    # Then TN (k=9) — 4 ratios: 1:8, 2:7, 3:6, 4:5
    probe_ratios("tn", "tennessee", 9, n_seeds=100)
    # Then PA (k=17) — 8 ratios
    probe_ratios("pa", "pennsylvania", 17, n_seeds=50)
