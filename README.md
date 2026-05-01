# redist — Open Redistricting Platform

`redist` is an open-source redistricting platform for practitioners, researchers, and reform advocates. It draws districts, analyzes plans, compares them to enacted maps, verifies legal constraints, and produces court-ready audit trails — for any chamber, any state, any census year.

## Where to start

| Who you are | Where to start | Time |
|---|---|---|
| Court-appointed special master | [docs/quickstart/quickstart-special-master.md](docs/quickstart/quickstart-special-master.md) | 5–10 min |
| Academic researcher (parameter sweeps) | [docs/quickstart/quickstart-researcher.md](docs/quickstart/quickstart-researcher.md) | 10–15 min |
| §2 plaintiff's expert (post-Callais) | [docs/quickstart/quickstart-callais-expert.md](docs/quickstart/quickstart-callais-expert.md) | 30–60 min |
| State legislative staff (Districtr backend) | [docs/quickstart/quickstart-state-staff.md](docs/quickstart/quickstart-state-staff.md) | 5 min/iteration |
| Civic advocacy group | [docs/quickstart/quickstart-civic-advocate.md](docs/quickstart/quickstart-civic-advocate.md) | 15–30 min |

First time on a clean machine? Run **`bash bootstrap.sh`** (Linux/macOS) or **`bootstrap.bat`** (Windows) from the repo root. Target wall-clock: ≤ 10 minutes from `git clone` to first useful run.

## What this is — and is not

**Is:** an algorithmic redistricting engine with bisection-based district drawing, plan analysis (compactness, VRA, partisan, splits), reproducibility-grade manifests, and a backend for Districtr/DRA round-tripping.

**Is not:** a GUI for interactive map drawing (use Districtr; we are the analytical backend), a real-time multiplayer editor, an automated litigation predictor, or a partisan tool. The algorithm itself is partisan-blind by default; partisan-weighted bisection is opt-in (Plan 03) and mutually exclusive with VRA-aware bisection per *Louisiana v. Callais* (608 U.S. ___, 2026-04-29) p.36.

---

## Quick start (after bootstrap)

```bash
# Build the binary (one-time; bootstrap.sh does this for you)
cargo build --release --manifest-path redist/Cargo.toml

# Download 2020 census data
redist fetch --year 2020

# Draw all 50 congressional districts (~15 seconds)
redist states --year 2020 --version V3 --output-dir outputs/V3 --workers 8

# Draw Washington's 98 house districts
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --label wa_house_draft1 \
  --balance-tolerance 5.0 --seed 42

# Analyze: compactness, partisan fairness, county splits, VRA
redist analyze --label wa_house_draft1 --year 2020 --version WA_Plans --types all

# Compare to enacted map
redist compare --plan-a wa_house_draft1 --enacted --year 2020 --version WA_Plans

# Generate commission report (HTML + JSON)
redist report --label wa_house_draft1 --year 2020 --version WA_Plans --format html json
```

---

## What it does

### Redistricting

Any chamber, any district count, any census year:

```bash
redist state --state WA --districts 98 --chamber house   # state house
redist state --state WA --districts 49 --chamber senate  # state senate
redist state --state WA --chamber congressional          # congressional (from config)
```

Multi-chamber suites with nesting validation (senate = 2 × house):

```bash
redist suite --state WA --year 2020 --version WA_Plans \
  --name wa_commission_v1 \
  --house-districts 98 --senate-districts 49 \
  --nest senate-in-house --seed 42
```

Three partition modes:
- **`edge-weighted`** (default) — compactness-optimized, follows geographic boundaries
- **`unweighted`** — pure population balance, no geometric optimization
- **`metis-vra`** — VRA-compliant, boosts edges between minority-heavy tracts

### Choosing resolution: tract vs block group

`redist` runs at census tract resolution by default (~4,000 people/unit). For state legislative maps with many districts, tract resolution may be too coarse:

| Resolution | Unit size | WA 98-district house | Recommendation |
|------------|-----------|----------------------|----------------|
| `tract` (default) | ~4,000 pop | 18 units/district | OK for ≤50 districts |
| `block_group` | ~1,200 pop | 54 units/district | Required for large chambers |

**Rule of thumb**: if `num_districts / state_tracts > 0.05` (fewer than 20 tracts per district), use block_group.

```bash
# WA house 98D — tract fails, block_group works
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --label wa_house_draft1 \
  --resolution block_group --balance-tolerance 10.0 --seed 42

# TX house 150D — tract works (46 tracts/district)
redist state --state TX --year 2020 --version TX_Plans \
  --districts 150 --chamber house --label tx_house_draft1 \
  --balance-tolerance 10.0 --seed 42
```

Block group adjacency files are built with:
```bash
python scripts/data/geography/build_unit_adjacency.py \
  --state WA --year 2020 --resolution block_group
```

### Analysis

```bash
redist analyze --label wa_house_draft1 --types all
```

Produces per-district JSON for:
- **Demographic** — race/ethnicity breakdown, majority-minority flags
- **Political** — partisan lean (efficiency gap, mean-median, partisan bias with 95% CI)
- **Compactness** — Polsby-Popper, Reock, convex hull ratio
- **Contiguity** — verifies every district is a connected set of tracts
- **Splits** — county and municipal preservation (state-specific constitutional standard)
- **Urban** — largest city per district
- **VRA** — majority-minority district count and compliance

### Plan comparison

```bash
# Compare two generated plans
redist compare --plan-a wa_house_v1 --plan-b wa_house_v2

# Compare against currently enacted districts (downloads from Census if needed)
redist compare --plan-a wa_house_v1 --enacted
```

Reports Jaccard similarity, population equality, compactness, splits, and partisan metrics side-by-side. No "winner" declared — metrics presented neutrally for commission review.

### Commission reports

```bash
redist report --label wa_house_v1 --format html json pdf
```

10-section formal report:
1. Executive summary with pass/fail for each legal requirement
2. Population equality (per-district table + deviation histogram)
3. Geographic constraints (contiguity, county splits, municipal splits)
4. Partisan fairness (EG, MM, PB with 95% CI and academic citation)
5. Minority representation (VRA compliance)
6. Compactness (vs enacted, vs statewide average)
7. Comparison vs enacted plan
8. Nesting compliance (for multi-chamber suites)
9. **Full audit trail** — SHA-256 of all inputs, binary provenance, complete reproduction command
10. Maps (all types)

### Interoperability — RPLAN format

`redist` defines **RPLAN v0.1**, the first open redistricting plan interchange format. Export to or import from any tool:

```bash
# Export for other tools
redist export --label wa_house_v1 --format geojson shapefile gerrychain csv rplan

# Import a plan from DRA, PlanScore, or any GeoJSON source
redist import --file dra_alternative.geojson --state WA --year 2020 --label dra_alt

# Validate any RPLAN file
redist validate --file wa_house_v1.rplan
```

Compatible with: GerryChain v2.3+, Dave's Redistricting App, PlanScore, QGIS, ArcGIS, Maptitude.

---

## Research results

**Headline result (2020 Census):** mean Polsby-Popper compactness **0.361** (95% CI: 0.351-0.370), a **+22% improvement** over enacted 2020 congressional districts (PP 0.296). 37 of 44 multi-district states beat their own enacted maps. Illinois +174%, Louisiana +104%, New Hampshire +102%.

**VRA result (2020):** `metis-vra` mode achieves majority-minority district targets in Alabama (2 MM districts) and Georgia (7 MM districts, exceeding the 5-district target).

**State legislative results:** Using block group resolution (`--resolution block_group`), `redist` successfully draws state legislative maps for all 50 states including the hardest cases: Nevada (71% of population in Clark County), Louisiana (bayou geography, isolated tracts), Virginia (95 counties + 38 independent cities tracked separately), and Texas (150 state house districts — the largest state legislature in the country).

**[View all dashboards](https://giodl73-repo.github.io/REDIST/)** — 2020, 2010, VRA results. All 50 states, 435 districts, round-by-round bisection maps.

---

## The bigger picture: a federal statute analogous to Huntington-Hill

The Supreme Court's *Rucho v. Common Cause* (2019) decision held that partisan-gerrymandering claims are nonjusticiable in federal court — there is no "judicially manageable standard" for federal courts to apply. But Roberts' opinion explicitly invited Congress and the States to act through positive law: state constitutions, state commissions, and **federal legislation** (HR 1 was cited by name).

Two of those paths — state-court litigation and state commissions — are working but slow. The third — single-issue federal legislation — has not been seriously attempted.

This project's working thesis is that the gap *Rucho* opened can be closed by a federal statute structurally analogous to **2 U.S.C. § 2a** (Huntington-Hill apportionment). Congress prescribes one deterministic algorithm; states execute it; any person can verify the output byte-identically.

Drafts now live under [`docs/legal/`](docs/legal/):

- **[`MODEL_FEDERAL_STATUTE.md`](docs/legal/MODEL_FEDERAL_STATUTE.md)** — bill text in U.S. Code drafting style (v0.1, post 5-role review). Defines tracts, mandates recursive bisection on the tract adjacency graph, requires reproducibility artifacts (`final_assignments.json` + `manifest.json` with input SHA-256s), preserves VRA § 2 as a deviation from baseline with published *Gingles* justification, and creates a private right of action for citizens to verify byte-identity.
- **[`STATUTE_RATIONALE.md`](docs/legal/STATUTE_RATIONALE.md)** — policy memo: why Congress (Rucho's invitation), why reproducibility (every other "fair maps" standard imports discretion), why bisection (uniquely writeable, executable, constitutionally defensible), why states execute and not Census (Elections Clause cleanliness + local knowledge + politics), why now (the implementation is mature; post-Callais clarifies disentanglement).
- **[`STATUTE_ONE_PAGER.md`](docs/legal/STATUTE_ONE_PAGER.md)** — 90-second version for staff briefings.
- **[`PARTISAN_OPTIONS.md`](docs/legal/PARTISAN_OPTIONS.md)** — how the project's four partisan-input postures (partisan-blind / partisan-balanced / partisan-similarity / party-overlapping) fit together. The Federal Statute prohibits inputs (2)–(4) for federal congressional districts; they remain available for state legislative districts and civic counter-proposals. The new `redist analyze --types proportionality` metric is the cross-cutting comparison lens.

The architecture in one paragraph: **Algorithm and parameters → federal (Director of Census + expert panel under § 107). Execution → state. VRA § 2 deviations → state, with published *Gingles* justification. Verification → any citizen, byte-identically, with private right of action.** Federal courts get jurisdiction; a successful suit substitutes the algorithm's output for the State's published map. This is the same posture as VRA § 2 today, applied to mechanical reproducibility instead of substantive fairness.

The reference implementation under § 107(a) of the model bill is what this project ships: `redist` produces the `manifest.json` + `final_assignments.json` schemas the bill requires, with byte-stable AEA-compliant replication packages (`redist analyze --paper-mode`), bloc-voting analysis for VRA § 2 (`redist analyze --types bloc-voting`), and Callais-disentangled partisan-weighted mode for state-court fairness claims. **The technical risk is zero — the algorithm has been in production scientific computing since METIS shipped in 1997. The work that remains is political: coalition assembly around a freestanding, single-issue bill.**

The state-court companion ([`FAIRNESS_DOCTRINE.md`](docs/legal/FAIRNESS_DOCTRINE.md)) is the strategy for *today*. The federal-statute drafts are the strategy for the **2030 census cycle**.

---

## Why bisection is fair

Bisection is fair because it eliminates the choice. When you split something in half, neither side gets to pick which half is theirs — the cut is determined by the constraint (equal population) and the geometry, not by who benefits. Repeating that process recursively means every district is the product of a series of neutral halvings, not a single optimized design.

It's also transparent. You can watch each round of splitting in the dashboard and see exactly how a 52-district California map emerges from 6 rounds of splitting. There's no black box — just a sequence of cuts, each one as fair as splitting a pizza.

Procedural fairness is a stronger property than the substantive-fairness claims federal courts have rejected post-*Rucho*. "This map is fair" is contested; "this map is the byte-identical output of running this published algorithm on these published inputs" is verifiable. That's the property the federal-statute drafts above are built on.

## How it works

**The picture, in plain terms:** treat the state as a map of small neighborhoods. Two neighborhoods are connected if they share a border, and the connection is "stronger" the longer that shared border is. The algorithm cuts the map in half by snipping the weakest set of connections that produces two halves with equal population — which naturally avoids long, jagged borders.

Stated formally, the algorithm treats each state as a graph. Census tracts are nodes; two tracts are connected if they share a border. The weight on each edge is the length of that shared border — so cuts through long shared boundaries cost more than cuts through short ones.

Then it bisects. METIS, a high-performance graph partitioner, splits the state into two halves of roughly equal population. Each half is split again. This continues for `⌈log₂ N⌉` rounds until there are exactly `N` districts. Minnesota's 8 districts take 3 rounds. California's 52 take 6.

Because the algorithm minimizes the total weight of the edges it cuts — and edge weight equals shared boundary length — it is directly minimizing the total perimeter of the resulting districts. Shorter perimeter at fixed area means more compact, more sensible shapes. The Polsby–Popper score (the standard compactness metric) goes up automatically, without ever being told to.

**No political or racial data enters the algorithm at any stage.** The inputs are geometry and population, nothing else.

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

Core algorithmic contribution. Weights edges by shared boundary length so minimizing the cut directly minimizes district perimeter. Achieves mean Polsby–Popper **0.361** (95% CI: 0.351–0.370), a **+22% improvement** over enacted 2020 maps (PP 0.296).

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

Introduces the `metis-vra` edge-weighted formulation: minority-to-minority tract edges receive higher weights (adaptive formula: `max(3.0, 10.0*(1-0.7*f))`) so METIS naturally preserves minority communities without multi-constraint vertex weights. Tests VRA Section 2 compliance on covered states. This is the algorithm behind the V4 pipeline run.

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
python setup_data.py --outputs v4    # VRA edge-weighted (metis-vra), 50 states, 2020

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

`run` and `runtest` are `doskey` aliases configured by `setup_env.bat`. See `run -h` for all flags.

### Run a single state

```bash
python scripts/pipeline/run_state_redistricting.py --state CA --year 2020
```

### Build the dashboard

```bash
python scripts/web/deploy_docs.py --version V3 --year 2020 --out dashboard_2020.html
python scripts/web/deploy_docs.py --version V4 --year 2020 --out dashboard_vra.html
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
├── research/                # Research papers (LaTeX sources + PDFs, A–E tracks)
├── artifacts/               # Earlier papers, presentations, guides
├── design/                  # Architecture plans: rust-port/, pitfalls/
├── data/{year}/             # Raw census data (gitignored)
├── outputs/                 # Pipeline outputs (gitignored)
├── docs/                    # Human-facing documentation + GitHub Pages dashboards
├── context/                 # Developer / AI-assistant docs
└── tests/                   # unit/ (~1000 tests), integration/ (~730 tests), e2e/
```

## Constraints

- Population: within ±0.5% of state target per district
- Contiguity: all districts connected
- Compactness: edge-cut minimization = perimeter minimization = Polsby–Popper maximization
- **No political or racial data used at any stage** (VRA mode uses demographic data only for edge weighting, not for population balance)

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
| VRA (2020) | 2020 | Edge-weighted minority (metis-vra) | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_vra.html) |

To regenerate after a new pipeline run:
```bash
python scripts/web/deploy_docs.py --version V3 --year 2020 --out dashboard_2020.html
python scripts/web/deploy_docs.py --version V3 --year 2010 --out dashboard_2010.html
python scripts/web/deploy_docs.py --version V3 --year 2000 --out dashboard_2000.html
python scripts/web/deploy_docs.py --version V4 --year 2020 --out dashboard_vra.html
```

## Rust CLI (recommended for production runs)

The `redist` binary is a complete Rust rewrite of the redistricting pipeline — **~213× faster** than Python
for full 50-state runs (55 min → 15.5 s). It is the recommended entry point for running redistricting.

```bash
cargo build --release --manifest-path redist/Cargo.toml

redist fetch --year 2020                              # Download census data
python scripts/data/generate_adj_bin.py --year 2020  # Convert adjacency to fast format
redist states --year 2020 --version V3 \
  --output-dir outputs/V3 --workers 8                # All 50 states in ~15 s
```

See [`docs/REDIST_CLI.md`](docs/REDIST_CLI.md) for full command reference.

**Architecture** (`redist/crates/`): `redist-core` (algorithm) · `redist-data` (adjacency, TIGER) ·
`redist-cli` (binary) · `redist-analysis` (compactness, VRA) · PyO3 bindings for Python interop.

The Python pipeline (`scripts/pipeline/`) remains available for development and analysis work.
Phase plan and benchmarks: [`design/rust-port/`](design/rust-port/).

## License

[MIT](LICENSE) © 2026 Gio Della-Libera
