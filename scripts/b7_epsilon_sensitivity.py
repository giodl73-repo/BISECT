"""
Epsilon sensitivity analysis for CompactBisect (B.7 P2-III).
For each seed, the near-minimum candidate set C_epsilon = {plans with EC <= (1+epsilon)*EC_min}.
The CompactBisect selection criterion picks the plan in C_epsilon with max GMPP.
Here we approximate: since we only have the FINAL k-partition plans (not level-1 only),
we use the final plan's EC as a proxy and report what partisan outcome
the minimum-EC plan in each epsilon-band produces.

This answers: "does the CompactBisect outcome change as epsilon varies?"
"""
import json, os, math

def run_epsilon_sensitivity(code, name, k, max_seeds=1000):
    results = []
    for seed in range(1, max_seeds + 1):
        ver = f"b7_{code.upper()}_s{seed}"
        mpath = f"outputs/{ver}/2020/states/{name}/manifest.json"
        ppath = f"outputs/{ver}/2020/states/{name}/analysis/proportionality.json"
        if not (os.path.exists(mpath) and os.path.exists(ppath)): continue
        ec = json.load(open(mpath)).get('edge_cut', 0) / 1000
        p  = json.load(open(ppath))
        if ec <= 0 or not p.get('available', True): continue
        results.append({'seed': seed, 'ec': ec, 'd': p['dem_seats'], 'gap': p['proportionality_gap_pp']})

    if not results: return

    ec_min = min(r['ec'] for r in results)
    print(f"\n{code.upper()} ({k}D, {len(results)} seeds) | EC_min = {ec_min:.0f}km")
    print(f"{'epsilon':>8} {'pool_size':>10} {'MEC_d':>8} {'pool_D_range':>15} {'outcome_stable?':>16}")
    print("-" * 65)

    for eps in [0.01, 0.02, 0.05, 0.10, 0.20]:
        threshold = ec_min * (1 + eps)
        pool = [r for r in results if r['ec'] <= threshold]
        d_vals = set(r['d'] for r in pool)
        mec_plan = min(pool, key=lambda r: r['ec'])
        d_range = f"{min(d_vals)}D-{max(d_vals)}D" if len(d_vals) > 1 else f"{list(d_vals)[0]}D"
        stable = "YES" if len(d_vals) == 1 else f"NO ({len(d_vals)} outcomes)"
        print(f"  {eps:>6.0%} {len(pool):>10} {mec_plan['d']:>7}D {d_range:>15}  {stable:>15}")

if __name__ == '__main__':
    for code, name, k, seeds in [
        ("WI", "wisconsin",       8, 1000),
        ("PA", "pennsylvania",   17, 1100),
        ("NC", "north_carolina", 14,  200),
    ]:
        run_epsilon_sensitivity(code, name, k, seeds)
