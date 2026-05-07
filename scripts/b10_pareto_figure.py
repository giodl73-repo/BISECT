"""Generate Pareto frontier figure for B.10 paper."""
import csv, math, sys
from pathlib import Path

# Read multiseed results
csv_path = Path("outputs/b10_multiseed/multiseed_results.csv")
if not csv_path.exists():
    print(f"ERROR: {csv_path} not found", file=sys.stderr); sys.exit(1)

from collections import defaultdict
data = defaultdict(lambda: defaultdict(list))
with open(csv_path) as f:
    for row in csv.DictReader(f):
        data[row['state']][float(row['alpha'])].append(
            (int(row['county_splits']), int(row['ec_km'])))

focal = ['GA', 'NC', 'TX', 'PA', 'CA']
alphas = sorted({a for st in data for a in data[st]})

# Build Pareto table: mean splits and mean EC per (state, alpha)
pareto = {}
for st in focal:
    pareto[st] = {}
    for a in alphas:
        if a in data[st] and data[st][a]:
            pts = data[st][a]
            pareto[st][a] = (
                sum(p[0] for p in pts)/len(pts),
                sum(p[1] for p in pts)/len(pts)
            )

# Try matplotlib; fall back to ASCII if unavailable
try:
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    import matplotlib.ticker as ticker

    fig, axes = plt.subplots(1, 2, figsize=(11, 4.5))
    colors = {'GA':'#e41a1c','NC':'#377eb8','TX':'#4daf4a','PA':'#984ea3','CA':'#ff7f00'}
    markers = {'GA':'o','NC':'s','TX':'^','PA':'D','CA':'v'}

    # Left: splits vs alpha
    ax = axes[0]
    for st in focal:
        xs = sorted(pareto[st])
        ys = [pareto[st][a][0] for a in xs]
        ax.plot(xs, ys, marker=markers[st], color=colors[st], label=st, linewidth=1.8)
    ax.axvline(x=5, color='gray', linestyle='--', linewidth=1, label=r'$\alpha=5$')
    ax.set_xlabel(r'Stickiness parameter $\alpha$', fontsize=11)
    ax.set_ylabel('Mean county splits (25 seeds)', fontsize=11)
    ax.set_title('County splits vs. $\\alpha$', fontsize=12)
    ax.legend(fontsize=9)
    ax.set_xscale('symlog', linthresh=0.5)
    ax.xaxis.set_major_formatter(ticker.ScalarFormatter())
    ax.set_xticks([0, 0.5, 1, 2, 5, 10, 20])

    # Right: splits vs EC (Pareto frontier)
    ax2 = axes[1]
    for st in focal:
        xs = sorted(pareto[st])
        ec_vals = [pareto[st][a][1] for a in xs]
        sp_vals = [pareto[st][a][0] for a in xs]
        ax2.plot(ec_vals, sp_vals, marker=markers[st], color=colors[st], label=st, linewidth=1.8)
        # Annotate alpha=5 point
        if 5 in pareto[st]:
            ax2.annotate(r'$\alpha$=5', xy=(pareto[st][5][1], pareto[st][5][0]),
                        xytext=(5, 5), textcoords='offset points', fontsize=7,
                        color=colors[st])
    ax2.set_xlabel('Mean edge cut (km, log scale)', fontsize=11)
    ax2.set_ylabel('Mean county splits', fontsize=11)
    ax2.set_title('Pareto frontier: splits vs. edge cut', fontsize=12)
    ax2.set_xscale('log')
    ax2.legend(fontsize=9)

    plt.tight_layout()
    out = Path("research/publications/subdivision-respecting-redistricting/figures/pareto_frontier.pdf")
    out.parent.mkdir(exist_ok=True)
    plt.savefig(out, bbox_inches='tight')
    # Also save PNG for quick preview
    plt.savefig(out.with_suffix('.png'), dpi=150, bbox_inches='tight')
    print(f"Saved: {out}")
    print(f"Saved: {out.with_suffix('.png')}")

except ImportError:
    # ASCII fallback: print the table
    print("matplotlib not available — printing ASCII Pareto table\n")
    header = f"{'State':>5} {'alpha':>6} {'splits':>8} {'ec_km':>8}"
    print(header); print('-'*len(header))
    for st in focal:
        for a in sorted(pareto[st]):
            sp, ec = pareto[st][a]
            print(f"{st:>5} {a:>6.1f} {sp:>8.1f} {ec:>8.0f}")
        print()
