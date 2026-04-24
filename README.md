# Apportionment

Every ten years, after the census, each state redraws its congressional districts. The process is notoriously political — legislators choose their voters, not the other way around. Salamander-shaped districts, packed minorities, cracked communities. It's been this way for two centuries.

And computers have made it worse. Modern redistricting software lets mapmakers calculate the partisan effect of moving a single city block from one district to another, optimizing the map to a precision that was impossible by hand. The problem isn't that we need a computer to solve it. Computers are already being used — to gerrymander more precisely than ever before.

The question is which **method** to use. And the fairest method turns out to be the oldest one in the book: **divide it in half.**

Split the state into two equal halves by population. Split each half again. Keep going until you have the right number of districts. It's the same principle as cutting a pizza, splitting an inheritance, or dividing a deck of cards — nobody gets to choose which half is theirs, so nobody gets an advantage. The shape of each piece is determined entirely by geography.

**Headline result (2020 Census):** mean Polsby–Popper compactness **0.367**, a **+56% improvement** over the unweighted baseline and **+20% over enacted 2020 congressional districts**. 37 of 50 states beat their own enacted maps on compactness. Illinois improves +174%, Louisiana +104%, New Hampshire +102%.

**[View interactive results dashboard →](https://giodl73-repo.github.io/DEXTRO/dashboard.html)** — all 50 states, 435 districts, round-by-round bisection maps.

---

## How it works

The algorithm treats each state as a graph. Census tracts are nodes; two tracts are connected if they share a border. The weight on each edge is the length of that shared border — so cuts through long shared boundaries cost more than cuts through short ones.

Then it bisects. METIS, a high-performance graph partitioner, splits the state into two halves of roughly equal population. Each half is split again. This continues for `⌈log₂ N⌉` rounds until there are exactly `N` districts. Minnesota's 8 districts take 3 rounds. California's 52 take 6.

Because the algorithm minimizes the total weight of the edges it cuts — and edge weight equals shared boundary length — it is directly minimizing the total perimeter of the resulting districts. Shorter perimeter at fixed area means more compact, more sensible shapes. The Polsby–Popper score (the standard compactness metric) goes up automatically, without ever being told to.

**No political or racial data enters the algorithm at any stage.** The inputs are geometry and population, nothing else.

## Why bisection

Bisection is fair because it eliminates the choice. When you split something in half, neither side gets to pick which half is theirs — the cut is determined by the constraint (equal population) and the geometry, not by who benefits. Repeating that process recursively means every district is the product of a series of neutral halvings, not a single optimized design.

It's also transparent. You can watch each round of splitting in the dashboard and see exactly how a 52-district California map emerges from 6 rounds of splitting. There's no black box — just a sequence of cuts, each one as fair as splitting a pizza.

## Track record

The same algorithm, unchanged, run on 2010 Census data produces Polsby–Popper **0.320** — only 10% lower than the 2020 result, despite vastly different political environments, different state legislatures, and a decade of demographic change. Geographic structure drives the outcome, not political opportunity.

The gap between algorithmic and enacted districts has also been shrinking: from 2010 to 2020, enacted maps improved as redistricting reform spread across states. The algorithm's advantage narrowed by roughly half. That's the reform working — and a good sign that the benchmark this project sets is a meaningful one.

---

## Figures

### Bisection rounds — Minnesota (8 districts, 3 rounds)
| Round 1 (1 → 2) | Round 2 (2 → 4) | Round 3 (4 → 8) |
| :---: | :---: | :---: |
| ![](docs/figures/minnesota_round_1.png) | ![](docs/figures/minnesota_round_2.png) | ![](docs/figures/minnesota_round_3.png) |

### Bisection rounds — Alabama (7 districts, 3 rounds)
| Round 1 | Round 2 | Round 3 |
| :---: | :---: | :---: |
| ![](docs/figures/alabama_round_1.png) | ![](docs/figures/alabama_round_2.png) | ![](docs/figures/alabama_round_3.png) |

---

## Research

Three papers document the method and results in full. Sources live under [`artifacts/papers/`](artifacts/papers/) and compile to PDF via each paper's `compile.bat` / `compile.sh`.

### Paper 1 — Introducing Recursive Bisection to Redistricting
[`artifacts/papers/01_recursive_bisection/`](artifacts/papers/01_recursive_bisection/)

Baseline method. Represents a state's census tracts as a graph, then applies METIS recursive bisection under a population-balance constraint to produce districts from only population and adjacency. Evaluated on all 50 states (2020 Census) with a mean population deviation of 2.79%.

### Paper 2 — Edge-Weighted Recursive Bisection for Compact Congressional Redistricting
[`artifacts/papers/02_edge_weighted_bisection/`](artifacts/papers/02_edge_weighted_bisection/)

Core contribution. Weights graph edges by actual shared boundary length, so minimizing the weighted cut directly minimizes district perimeter. Achieves mean Polsby–Popper **0.367** nationally (+56% vs. unweighted, +20% vs. enacted 2020). Introduces county-based bridge edges for water crossings.

### Paper 3 — Algorithmic Congressional Redistricting via Edge-Weighted Recursive Bisection
[`artifacts/papers/03_combined_recursive_bisection/`](artifacts/papers/03_combined_recursive_bisection/)

Consolidated treatment with cross-census validation. Demonstrates 10% variation across census years as evidence that geography — not politics — drives performance. Quantifies the shrinking gap between algorithmic and enacted districts from 2010 to 2020.

### Also under `artifacts/`
- **Presentation**: [`presentations/edge_weighted_bisection/`](artifacts/presentations/edge_weighted_bisection/) — conference-style deck
- **Guides**: [`guides/edge_weighted_bisection/`](artifacts/guides/edge_weighted_bisection/) (layman's guide) and [`guides/command_reference/`](artifacts/guides/command_reference/)

---

## Quick Start

```bash
pip install -r requirements.txt
```

On Windows, `pymetis` may need `conda install -c conda-forge metis` first.

### Run the full pipeline

```bash
# All 3 census years in parallel (2-4h)
run -v v1

# Single year
run -y 2020 -v v1

# Specific states
run -y 2020 -v v1 -st MN AL

# Test run (outputs under outputs/dev/)
runtest -y 2020 -v test -st VT
```

`run` and `runtest` are `doskey` aliases for `run_redistricting.bat` and `run_test.bat`. See `run -h` for all flags.

### Run a single state

```bash
python scripts/pipeline/run_state_redistricting.py --state CA --year 2020
```

### Build the dashboard

```bash
python scripts/web/generate_master_dashboard.py
python scripts/web/embed_maps_dashboard.py   # → docs/dashboard.html (self-contained)
```

## Data

Inputs, per census year (2000, 2010, 2020):
- **Geometry**: TIGER/Line tract shapefiles
- **Population**: P.L. 94-171 redistricting files
- **Places**: TIGER/Line places shapefiles (for city labels)

All census-year data lives under `data/{year}/` (gitignored — ~55GB). Download with:

```bash
python scripts/data/download_orchestrator.py --stages redistricting --year 2020
```

## Project Structure

```
apportionment/
├── src/apportionment/       # Library: partition/, data/, visualization/
├── scripts/                 # Executables: pipeline/, data/, political/, demographic/, compactness/, web/
├── artifacts/               # Papers, presentations, guides (LaTeX sources)
├── data/{year}/             # Raw census data (gitignored)
├── outputs/                 # Pipeline outputs (gitignored)
├── docs/                    # Human-facing documentation + dashboard
├── context/                 # Developer / AI-assistant docs
└── tests/                   # unit/, integration/, e2e/ (215 tests, ~24s)
```

## Constraints

- Population: within ±0.5% of state target per district
- Contiguity: all districts connected (enforced by METIS `-contig`)
- Compactness: edge-cut minimization = perimeter minimization = Polsby–Popper maximization
- **No political or racial data used at any stage**

## Documentation

- [`docs/RECURSIVE_BISECTION.md`](docs/RECURSIVE_BISECTION.md) — algorithm walkthrough
- [`docs/GETTING_STARTED.md`](docs/GETTING_STARTED.md) — install + first run
- [`docs/CHANGELOG.md`](docs/CHANGELOG.md) — version history
- [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) — workflow + git practices
