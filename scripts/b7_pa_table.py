import json, os
counts = {}; min_ec = {}; max_ec = {}; sum_ec = {}
for seed in range(1, 1101):
    ppath = f"outputs/b7_PA_s{seed}/2020/states/pennsylvania/analysis/proportionality.json"
    mpath = f"outputs/b7_PA_s{seed}/2020/states/pennsylvania/manifest.json"
    if not (os.path.exists(ppath) and os.path.exists(mpath)): continue
    p = json.load(open(ppath)); m = json.load(open(mpath))
    if not p.get('available', True): continue
    d = p['dem_seats']; ec = m.get('edge_cut', 0) / 1000
    if ec <= 0: continue
    counts[d] = counts.get(d, 0) + 1
    if d not in min_ec or ec < min_ec[d]: min_ec[d] = ec
    if d not in max_ec or ec > max_ec[d]: max_ec[d] = ec
    sum_ec[d] = sum_ec.get(d, 0) + ec

total = sum(counts.values())
print(f"PA 1,100-seed distribution ({total} valid seeds):")
for d in sorted(counts):
    r = 17 - d; gap = 100*(d/17 - 0.506)
    mean = sum_ec[d]/counts[d]
    print(f"  {d}D/{r}R ({gap:+.1f}pp): n={counts[d]} ({100*counts[d]/total:.0f}%) | EC min={min_ec[d]:.0f} mean={mean:.0f} max={max_ec[d]:.0f}km")
