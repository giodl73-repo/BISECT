# Apportionment

Every ten years, after the census, each state redraws its congressional districts. The process is notoriously political — legislators choose their voters, not the other way around. Salamander-shaped districts, packed minorities, cracked communities. It's been this way for two centuries.

And computers have made it worse. Modern redistricting software lets mapmakers calculate the partisan effect of moving a single city block from one district to another, optimizing the map to a precision that was impossible by hand. The problem isn't that we need a computer to solve it. Computers are already being used — to gerrymander more precisely than ever before.

The question is which **method** to use. And the fairest method turns out to be the oldest one in the book: **divide it in half.**

Split the state into two equal halves by population. Split each half again. Keep going until you have the right number of districts. It's the same principle as cutting a pizza, splitting an inheritance, or dividing a deck of cards — nobody gets to choose which half is theirs, so nobody gets an advantage. The shape of each piece is determined entirely by geography.

**Headline result (2020 Census):** mean Polsby–Popper compactness **0.367**, a **+56% improvement** over the unweighted baseline and **+20% over enacted 2020 congressional districts**. 37 of 50 states beat their own enacted maps on compactness. Illinois improves +174%, Louisiana +104%, New Hampshire +102%.

**[View all dashboards →](https://giodl73-repo.github.io/REDIST/)** — 2020, 2010, VRA results. All 50 states, 435 districts, round-by-round bisection maps.

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

20 papers across five tracks. All PDFs are pre-built and open directly from the links below. LaTeX sources live under [`research/`](research/) and [`artifacts/papers/`](artifacts/papers/).

---

### Track A — Synthesis

**A.0 · Algorithmic Objectivity for Congressional Redistricting: A National-Scale Demonstration**
[PDF](research/A.0+synthesis-metapaper/main.pdf) · [Source](research/A.0+synthesis-metapaper/)

Synthesis metapaper. Frames the full research program: why bisection is the right method, what the national-scale results show, and what the implications are for redistricting reform. Covers all 50 states across three census decades.

**A.1 · Research Portfolio Guide**
[PDF](research/A.1+portfolio-guide/guide.pdf) · [Source](research/A.1+portfolio-guide/)

Reader's guide to the paper portfolio — how the tracks relate, what order to read them in, and how findings build on each other.

**A.2 · Portfolio Summary**
[PDF](research/A.2+portfolio-summary/summary.pdf) · [Source](research/A.2+portfolio-summary/)

One-document summary of all major findings across the portfolio for readers who want the headlines without the full papers.

---

### Track B — Algorithm

**B.1 · From Apportionment to Boundary Design: Recursive Bisection for Congressional Redistricting**
[PDF](research/B.1+recursive-bisection/main.pdf) · [Source](research/B.1+recursive-bisection/)

Baseline method paper. Establishes the graph-theoretic formulation, METIS recursive bisection under population balance, and unweighted baseline results on all 50 states (2020 Census).

**B.2 · Edge-Weighted Recursive Bisection for Compact Congressional Districts**
[PDF](research/B.2+edge-weighted-bisection/main.pdf) · [Source](research/B.2+edge-weighted-bisection/)

Core algorithmic contribution. Weights edges by shared boundary length so minimizing the cut directly minimizes district perimeter. Achieves mean Polsby–Popper **0.367** (+56% vs. unweighted, +20% vs. enacted 2020 maps).

**B.3 · Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Redistricting**
[PDF](research/B.3+multi-vs-edge/main.pdf) · [Source](research/B.3+multi-vs-edge/)

Theoretical and empirical comparison of edge-weighted (single-objective) vs. multi-constraint METIS. Shows why single-objective formulation produces better compactness: the objectives don't compete.

**B.4 · Edge-Weighting Makes Method Selection Irrelevant: Complete Equivalence of Recursive and N-Way Bisection**
[PDF](research/B.4+adaptive-bisection/main.pdf) · [Source](research/B.4+adaptive-bisection/)

Proves that once edge weights are introduced, recursive bisection and direct n-way partitioning converge to the same solution. Method choice doesn't matter — the weighting does.

---

### Track C — Validation

**C.1 · Spatial Resolution and Algorithmic Redistricting: MAUP Sensitivity Analysis**
[PDF](research/C.1+maup-sensitivity/main.pdf) · [Source](research/C.1+maup-sensitivity/)

Tests whether results depend on the choice of geographic unit (census tracts vs. block groups vs. blocks). Finds the algorithm is robust across 130× range in unit count — geography drives outcomes, not resolution.

**C.2 · Cross-Census Validation for Congressional Redistricting Algorithms**
[PDF](research/C.2+cross-census-validation/main.pdf) · [Source](research/C.2+cross-census-validation/)

Validates the algorithm on 2000, 2010, and 2020 Census data. Compactness scores vary only ~10% across decades, confirming that stable geography — not political cycles — drives performance.

**C.3 · Cross-Census Temporal Stability**
[PDF](research/C.3+temporal-stability/main.pdf) · [Source](research/C.3+temporal-stability/)

Deep analysis of why the algorithm is temporally stable. Identifies the geographic clustering properties that make bisection outcomes predictable across different political environments.

**C.4 · Twenty Years of Congressional Redistricting: A Longitudinal Analysis**
[PDF](research/C.4+longitudinal-analysis/main.pdf) · [Source](research/C.4+longitudinal-analysis/)

Tracks algorithmic vs. enacted district quality from 2000 to 2020. Quantifies the shrinking gap as redistricting reform spread across states — and shows the algorithm's benchmark is a meaningful target.

**C.5 · Measuring Partisan Fairness in Algorithmic Redistricting: Efficiency Gap Analysis**
[PDF](research/C.5+efficiency-gap-analysis/main.pdf) · [Source](research/C.5+efficiency-gap-analysis/)

Applies the efficiency gap metric to algorithmic districts. Shows that purely geographic bisection produces near-zero efficiency gaps in most states — fairness as a byproduct of geometric neutrality.

---

### Track D — Voting Rights Act

**D.0 · Voting Rights Act Compliance Through Edge-Weighted Graph Partitioning**
[PDF](research/D.0+vra-compliance/main.pdf) · [Source](research/D.0+vra-compliance/)

Introduces the `metis-vra` multi-constraint formulation (2D vertex weights: population + minority VAP). Tests VRA Section 2 compliance on covered states. This is the algorithm behind the V4 pipeline run.

**D.1 · The 42% Threshold: Geographic Limits of VRA Compliance Through Algorithmic Redistricting**
[PDF](research/D.1+threshold-analysis/main.pdf) · [Source](research/D.1+threshold-analysis/)

Discovers the critical empirical finding: states where minority population exceeds ~42% statewide achieve all statutory majority-minority district targets with principled methods. Below that threshold, geography makes compliance impossible without sacrificing compactness.

**D.2 · N-Way vs. Recursive Bisection for VRA-Compliant Redistricting**
[PDF](research/D.2+nway-vs-recursive-vra/main.pdf) · [Source](research/D.2+nway-vs-recursive-vra/)

Compares n-way and recursive approaches specifically for minority district formation. N-way's global optimization concentrates minority population slightly better, but neither method overcomes geographic constraints in low-minority states.

**D.3 · Quantifying the Voting Rights Act–Compactness Tradeoff**
[PDF](research/D.3+compactness-tradeoff/main.pdf) · [Source](research/D.3+compactness-tradeoff/)

Measures the exact compactness cost of VRA compliance in borderline states. Finds the tradeoff is real but bounded — even in Alabama, the compactness penalty is modest relative to typical enacted maps.

---

### Track E — Experimental Extensions

> These papers explore structural alternatives to the current redistricting system — different units, different scopes, different rules. They are forward-looking and speculative rather than empirical validations of the core method.

**E.1 · Multi-Member Districts and Proportional Representation**
[PDF](research/E.1+multi-member-districts/main.pdf) · [Source](research/E.1+multi-member-districts/)

Extends the bisection algorithm to multi-member districts. Shows that 3- and 5-member districts dramatically reduce the compactness–VRA tension by allowing minority communities to elect representatives without requiring majority-concentration districts.

**E.2 · Direct County Representation: An Alternative to Congressional Redistricting**
[PDF](research/E.2+county-representation/main.pdf) · [Source](research/E.2+county-representation/)

Proposes using counties as the unit of congressional representation, weighted by population. Eliminates district-drawing entirely. Evaluates compactness, county integrity, and VRA implications.

**E.3 · National Redistricting Without State Boundaries**
[PDF](research/E.3+national-redistricting/main.pdf) · [Source](research/E.3+national-redistricting/)

Asks what would happen if congressional districts could cross state lines. Applies the algorithm nationally — treating the entire US as one graph — and examines the resulting 435-district map.

**E.4 · Partisan Similarity Districts: Algorithmic Safe Seats**
[PDF](research/E.4+partisan-similarity-districts/main.pdf) · [Source](research/E.4+partisan-similarity-districts/)

Investigates whether geographic bisection inadvertently creates partisan safe seats by clustering politically similar communities. Finds that compactness and partisan homogeneity are correlated — a natural consequence of geographic sorting.

**E.5 · Partisan Fairness Through Algorithmic Districting**
[PDF](research/E.5+party-based-allocation/main.pdf) · [Source](research/E.5+party-based-allocation/)

Evaluates whether algorithmic redistricting produces partisan-fair outcomes without intentionally targeting fairness. Compares to proportional representation benchmarks and examines outlier states.

---

### Presentation & Guides

- **Conference presentation**: [PDF](artifacts/presentations/edge_weighted_bisection/presentation.pdf) · [Source](artifacts/presentations/edge_weighted_bisection/)
- **Layman's guide**: [PDF](artifacts/guides/edge_weighted_bisection/laymen_guide.pdf) · [Source](artifacts/guides/edge_weighted_bisection/)
- **Command reference**: [PDF](artifacts/guides/command_reference/command_reference.pdf) · [Source](artifacts/guides/command_reference/)

The three papers in [`artifacts/papers/`](artifacts/papers/) are earlier standalone versions of B.1, B.2, and B.1+B.2 combined; the `research/` track versions above supersede them.

---

## Getting the Data

Data is distributed via GitHub Releases — no Git LFS, no account required beyond `gh auth login`.

```bash
pip install -r requirements.txt

# Download curated inputs (~740MB: elections, demographics, baseline, adjacency graphs)
python setup_data.py --inputs

# Download pre-computed results (skip running the pipeline)
python setup_data.py --outputs v3    # edge-weighted, 50 states, 2020
python setup_data.py --outputs v4    # VRA multi-constraint, 50 states, 2020

# Download both outputs
python setup_data.py --all
```

Requires the [GitHub CLI](https://cli.github.com/) (`gh auth login` once).

### What's in each release

| Release | Contents | Size |
|---------|----------|------|
| [`data-inputs-v1`](https://github.com/giodl73-repo/REDIST/releases/tag/data-inputs-v1) | Elections, demographics, baseline enacted districts, metro areas, MAUP adjacency graphs | 736MB |
| [`outputs-v3`](https://github.com/giodl73-repo/REDIST/releases/tag/outputs-v3) | V3 edge-weighted results: district CSVs, compactness, political analysis | 3MB |
| [`outputs-v4`](https://github.com/giodl73-repo/REDIST/releases/tag/outputs-v4) | V4 VRA results: district CSVs with minority compliance analysis | 3MB |

### Raw Census data (~55GB)

TIGER/Line shapefiles and PL 94-171 redistricting files are downloaded directly from the US Census Bureau — no redistribution needed since they're already public:

```bash
python scripts/data/download_orchestrator.py --stages redistricting --year 2020
python scripts/data/download_orchestrator.py --stages redistricting --year 2010
```

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

### Concepts
Short guides for understanding the core ideas — each has a TL;DR followed by a deep dive.

- [`docs/concepts/recursive-bisection.md`](docs/concepts/recursive-bisection.md) — why bisection, how METIS splits the graph, the binary tree structure
- [`docs/concepts/edge-weighted-bisection.md`](docs/concepts/edge-weighted-bisection.md) — how boundary-length weights improve compactness automatically
- [`docs/concepts/polsby-popper.md`](docs/concepts/polsby-popper.md) — the compactness metric: formula, benchmarks, intuition
- [`docs/concepts/vra-compliance.md`](docs/concepts/vra-compliance.md) — VRA majority-minority districts, the 42% threshold, the V4 algorithm
- [`docs/concepts/census-data.md`](docs/concepts/census-data.md) — TIGER shapefiles, PL 94-171 files, GEOID format, downloading
- [`docs/concepts/pipeline-stages.md`](docs/concepts/pipeline-stages.md) — the five pipeline stages and how they connect

### Reference
- [`docs/RECURSIVE_BISECTION.md`](docs/RECURSIVE_BISECTION.md) — detailed algorithm walkthrough with pseudocode
- [`docs/PIPELINE_OUTPUTS.md`](docs/PIPELINE_OUTPUTS.md) — every file the pipeline writes (per-state and national)
- [`docs/GETTING_STARTED.md`](docs/GETTING_STARTED.md) — install + first run
- [`docs/CHANGELOG.md`](docs/CHANGELOG.md) — version history
- [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) — workflow + git practices

## Interactive Dashboards

All dashboards are self-contained — no server needed, open directly in a browser.

| Dashboard | Census year | Algorithm | Link |
|-----------|-------------|-----------|------|
| 2020 Results | 2020 | Edge-weighted bisection | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_2020.html) |
| 2010 Results | 2010 | Edge-weighted bisection | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_2010.html) |
| 2000 Results | 2000 | Edge-weighted bisection | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_2000.html) |
| VRA (2020) | 2020 | Multi-constraint minority | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_vra.html) |

To regenerate after a new pipeline run:
```bash
python scripts/web/deploy_docs.py --version V3 --year 2020 --out dashboard_2020.html
python scripts/web/deploy_docs.py --version V3 --year 2010 --out dashboard_2010.html
python scripts/web/deploy_docs.py --version V3 --year 2000 --out dashboard_2000.html
python scripts/web/deploy_docs.py --version V4 --year 2020 --out dashboard_vra.html
```

## License

[MIT](LICENSE) © 2026 Gio Della-Libera
