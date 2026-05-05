# redist — Open Redistricting Platform

`redist` is an open-source redistricting platform for practitioners, researchers, and reform advocates. It draws districts, analyzes plans, compares them to enacted maps, verifies legal constraints, and produces court-ready audit trails — for any chamber, any state, any census year.

## Where to start

| Who you are | Where to start | Time |
|---|---|---|
| Court-appointed special master | [docs/quickstart/quickstart-special-master.md](docs/quickstart/quickstart-special-master.md) | 5–10 min |
| Academic researcher (parameter sweeps) | [docs/quickstart/quickstart-researcher.md](docs/quickstart/quickstart-researcher.md) | 10–15 min |
| §2 plaintiff's expert (post-Callais) | [docs/quickstart/quickstart-callais-expert.md](docs/quickstart/quickstart-callais-expert.md) | 30–60 min |
| State legislative staff | [docs/quickstart/quickstart-state-staff.md](docs/quickstart/quickstart-state-staff.md) | 5 min/iteration |
| Civic advocacy group | [docs/quickstart/quickstart-civic-advocate.md](docs/quickstart/quickstart-civic-advocate.md) | 15–30 min |

First time on a clean machine? Run **`bash bootstrap.sh`** (Linux/macOS) or **`bootstrap.bat`** (Windows) from the repo root. Target wall-clock: ≤ 10 minutes from `git clone` to first useful run.

## What this is — and is not

**Is:** an algorithmic redistricting engine with bisection-based district drawing, plan analysis (compactness, VRA, partisan, splits), and reproducibility-grade manifests suitable for court submission and public verification.

**Is not:** a GUI for interactive map drawing, a real-time multiplayer editor, an automated litigation predictor, or a partisan tool. The algorithm is partisan-blind by default; partisan-weighted bisection is opt-in and mutually exclusive with VRA-aware bisection per *Louisiana v. Callais* (608 U.S. ___, 2026-04-29) p.36.

---

## Quick start (after bootstrap)

```bash
# Build the binary (one-time; bootstrap.sh does this for you)
cargo build --release --manifest-path redist/Cargo.toml

# Download 2020 census data
redist fetch --year 2020

# Official proposal: all 50 states with ApportionRegions + county-sticky weights
redist build official_2020 --year 2020 --workers 8

# Verify the SHA audit chain (config → build → analysis → report)
redist label-verify official_2020 --year 2020

# Analyze and report
redist label-analyze official_2020 --year 2020 --types all
redist label-report  official_2020 --year 2020 --format json

# List all named plans and their pipeline status
redist ls
```

`configs/official_proposal.yml` contains the reference configuration for the federal statute proposal — ApportionRegions structure, county-sticky geographic weights, convergence-sweep seed search.

---

## The three-layer compositor

Every `redist` run is fully described by four independent choices:

| Layer | Flag | What it controls |
|-------|------|-----------------|
| **Engine** | `--metis-engine` | *Which* METIS implementation runs the graph partitioning |
| **Structure** | `--structure` | *How* the bisection tree is organized |
| **Weights** | `--weights-override` | *What* graph signal METIS optimises |
| **Search** | `--search` | *How* the seed space is explored |

These choices are orthogonal — any engine works with any structure, weights, and search strategy.

### METIS engines (`--metis-engine`)

Three implementations are available, selectable at runtime via `--metis-engine` or via `engine:` in the YAML config.

| Value | Aliases | Backend | Build requirement | Notes |
|-------|---------|---------|-------------------|-------|
| `c-ffi` | `c`, `metis-rs` | `metis` Rust crate → bundled C METIS source (`vendored`) | C compiler at build time; **no runtime lib needed** | **Default for normal builds.** Battle-tested; handles all k including prime k. |
| `redist-metis` | `rust` | `redist-metis` pure-Rust crate | None — no C at all | **Default for `--no-default-features` builds.** Fully portable. Known gaps below. |
| `gpmetis` | `subprocess` | External `gpmetis` binary on PATH | `gpmetis` installed | Reserved stub — returns `[CONFIG]` error today. Planned. |

**What "default" means** depends on how the binary was compiled:

```
cargo build                        → default engine: c-ffi       (C source bundled, no runtime dep)
cargo build --no-default-features  → default engine: redist-metis (zero C, pure Rust)
```

The binary self-describes: you never need to set `--metis-engine` unless you want to override the compiled-in default.

```bash
# Standard run — uses compiled-in default (c-ffi for release builds)
redist state --state IA --year 2020 --version ia_v1

# Explicit portable run — pure Rust, no C toolchain at runtime or build time
redist state --state VT --year 2020 --version vt_rust --metis-engine redist-metis

# Explicit c-ffi (useful when binary was built --no-default-features but c-ffi is now installed)
redist state --state PA --year 2020 --version pa_v1 --metis-engine c-ffi

# In YAML config:
# algorithm:
#   engine: redist-metis   # or c-ffi, gpmetis
```

#### Known gaps (as of v0.2.0)

| Gap | Affects | Workaround |
|-----|---------|-----------|
| `redist-metis` may stall on prime k (e.g. PA k=17, TX k=38) for large graphs | `redist-metis` engine, `prime-factor` structure | Use `c-ffi` engine for states with prime seat counts |
| `AreaSection` (`ratio-optimal-area`) requires ncon=2 dual constraint — not supported in `redist-metis` | `redist-metis` engine + `ratio-optimal-area` structure | Use `c-ffi` engine, or use `ratio-optimal` (GeoSection, ncon=1) instead |
| `gpmetis` subprocess not yet implemented | `gpmetis` engine | Use `c-ffi` or `redist-metis` |

These gaps are tracked; the `test-engine-redist-metis` CI gate surfaces them on every push.

### Structure modes (`--structure`)

| Value | Algorithm | Description |
|-------|-----------|-------------|
| `standard-bisect` | Standard | ⌊k/2⌋:⌈k/2⌉ binary split at every level (default) |
| `prime-factor` | **ApportionRegions** (B.11) | Split via prime factorization of k. k=14 → 7×2, k=17 (prime) → 9+8. Geographic completion of Huntington-Hill |
| `ratio-optimal` | **GeoSection** (B.8) | Try all ratios 1:(k-1)…⌊k/2⌋:⌈k/2⌉ at level 1; pick minimum normalised edge-cut |
| `ratio-optimal-area` | **AreaSection** (B.9) | GeoSection ratio sweep with ncon=2 dual constraint [population, area] |
| `ratio-optimal-vra` | **VRASection** (B.14) | GeoSection ratio sweep with minority geographic alignment score |
| `nway` | N-way direct | Direct k-way METIS partition (no bisection tree) |
| `compact-polsby` | CompactBisect | Polsby-Popper guided bisection |

### Weight modes (`--weights-override`)

| Value | Description |
|-------|-------------|
| `geographic` | Edge weight = shared boundary length (default for edge-weighted modes) |
| `county` | County-sticky: ×3.0 on county-interior edges, promotes county preservation |
| `unweighted` | Pure population balance, no geometric signal |
| `vra-aligned` | Minority-to-minority edges boosted; partisan data excluded |
| `proportional` | ncon=2 [population, D_votes]; ProportionalSection (B.12) |

### Search modes (`--search`)

| Value | Description |
|-------|-------------|
| `single` | One content-derived seed (SHA-256 of census release + state FIPS). Deterministic. |
| `multi` | Walk `--seeds N` seeds; pick minimum edge-cut. |
| `convergence` | Walk seeds from SHA-256 chain; stop after `--seeds T` (default 500) non-improving. Empirically sufficient for all 50 states (B.16). |

### Examples

```bash
# Official proposal: ApportionRegions + county-sticky + convergence sweep
redist build official_2020 --year 2020 --workers 8
# (reads configs/official_2020.yml — structure: prime-factor, weights: county, search: convergence)

# GeoSection with 100 seeds per ratio
redist state --state NC --year 2020 --version nc_test \
  --structure ratio-optimal --search multi --seeds 100

# AreaSection for Illinois (area-balanced bisection, ±10% area swing)
redist state --state IL --year 2020 --version il_area \
  --structure ratio-optimal-area --area-swing 1.10 --search convergence

# Standard bisection with county weights
redist state --state WI --year 2020 --version wi_county \
  --partition-mode edge-weighted --weights-override county

# ApportionRegions direct (no config file)
redist state --state PA --year 2020 --version pa_pfr \
  --structure prime-factor --search convergence --seeds 600
```

---

## Label-based run management (Spec 7)

Plans are referenced by **labels** — short identifiers that resolve to all directory paths by convention. No path arguments needed in normal use.

```
runs/{label}/{year}/{state}/     ← build outputs
analysis/{label}/{year}/         ← analysis outputs
reports/{label}/{year}/          ← report outputs
```

Every stage writes an `index.json` with a SHA-256 that chains to the previous stage, forming a tamper-evident audit trail: **config → build → analysis → report**.

### Core verbs

```bash
# Build: draw all congressional districts for a named plan
redist build <label> [--year 2020] [--states IA TX CA] [--workers 8] [--force]

# Analyze: run metrics on a built plan
redist label-analyze <label> [--year 2020] [--types summary compactness political]

# Report: produce formatted output
redist label-report  <label> [--year 2020] [--format json html pdf]

# Verify: traverse the SHA chain and confirm integrity
redist label-verify  <label> [--year 2020]

# Inspect
redist ls                        # list all labels and their pipeline status
redist ls --json                 # machine-readable JSON
redist show <label>              # detailed status, paths, SHA chain
redist show <label> --json

# Rename (patches root index; data moves atomically)
redist mv <label> <new-label>

# Import an external plan (CSV or GeoJSON) as a label
redist label-import <label> --from plan.csv --year 2020 --format csv

# Compare two labels side-by-side
redist label-compare <label-a> <label-b> --year 2020
```

### Plan configuration (YAML)

Place a YAML config in `configs/<label>.yml`:

```yaml
name: official_2020
algorithm:
  structure: prime-factor        # ApportionRegions bisection tree
  weights: county                # county-sticky geographic weights
  search: convergence            # walk seeds until T non-improving
  convergence_threshold: 600
  balance_tolerance: 0.5
workers: 8
years: ["2020", "2010", "2000"]
```

Then run `redist build official_2020` — the config SHA is recorded in the build index.

---

## What it does

### Redistricting — any chamber, any district count

```bash
redist state --state WA --districts 98 --chamber house   # state house
redist state --state WA --districts 49 --chamber senate  # state senate
redist state --state WA --chamber congressional          # congressional (from config)
```

### Multi-state runs

```bash
# All 50 states, 8 workers
redist states --year 2020 --version V3 --workers 8

# Specific states only
redist states --year 2020 --version V3 --states VT DE AK WY --workers 4
```

### Analysis

```bash
redist label-analyze wa_house_draft1 --year 2020 --types all
```

Produces per-district JSON for:
- **Demographic** — race/ethnicity breakdown, majority-minority flags
- **Political** — partisan lean (efficiency gap, mean-median, partisan bias with 95% CI)
- **Compactness** — Polsby-Popper, Reock, convex hull ratio
- **Contiguity** — verifies every district is a connected set of tracts
- **Splits** — county and municipal preservation
- **Urban** — largest city per district
- **VRA** — majority-minority district count and compliance; bloc-voting regression (Callais p.36)
- **Proportionality** — seat-vote curve, Lorenz feasibility, Rodden gap

### Plan comparison

```bash
redist label-compare wa_house_v1 wa_house_v2 --year 2020
```

Reports Jaccard similarity, population equality, compactness, splits, and partisan metrics side-by-side. No "winner" declared — metrics presented neutrally for commission review.

### Commission reports

```bash
redist label-report wa_house_v1 --year 2020 --format html json pdf
```

10-section formal report with full audit trail — SHA-256 of all inputs, binary provenance, complete reproduction command.

---

## Research results

**Headline result (2020 Census):** mean Polsby-Popper compactness **0.361** (95% CI: 0.351–0.370), a **+22% improvement** over enacted 2020 congressional districts (PP 0.296). 37 of 44 multi-district states beat their own enacted maps.

**ApportionRegions (B.11) — 2020 sweep:** 223D/209R national (51.6% D). NC k=14=7×2 → **7D/7R** (near-perfectly proportional, −0.7pp gap) vs. standard bisection 5D/9R (−13.6pp). Structure, not criterion, drives partisan outcomes.

**GeoSection (B.8) — 50-state sweep:** 5D/9R stable for NC across all tested seeds. Algorithm characterised: ratio selectivity follows isoperimetric EC/√(min(i,k−i)) normalisation; 6D/8R not achievable for NC without multi-level area constraint.

**AreaSection (B.9) — 50-state sweep:** 76% of seat-distribution outcomes stable vs. standard bisection. Lorenz infeasibility filter identifies states (GA, concentrated) where equal-area splits are geographically impossible. Dual-constraint regime boundary at area_swing=1.10 (B.9 §4).

**ConvergenceSweep (B.16):** T=600 non-improving seeds is empirically sufficient for all 50 states. Seed space explored via SHA-256 chain `BLAKE3(census_release_id‖"DIA_SEED_V1"‖i)`.

**[View all dashboards](https://giodl73-repo.github.io/REDIST/)** — 2020, 2010, VRA results. All 50 states, 435 districts, round-by-round bisection maps.

---

## The bigger picture: a federal statute analogous to Huntington-Hill

The Supreme Court's *Rucho v. Common Cause* (2019) held that partisan-gerrymandering claims are nonjusticiable in federal court. Roberts' opinion explicitly invited Congress and the States to act through positive law.

This project's working thesis (B.02): the gap *Rucho* opened can be closed by a federal statute structurally analogous to **2 U.S.C. § 2a** (Huntington-Hill apportionment). Congress prescribes one deterministic algorithm; states execute it; any person can verify the output byte-identically. **ApportionRegions is the algorithmic proposal** — it is the only redistricting method derivable from the apportionment statute itself (HH seat allocation → prime-factor factorization of k → bisection tree).

Drafts live under [`docs/legal/`](docs/legal/):
- **[`MODEL_FEDERAL_STATUTE.md`](docs/legal/MODEL_FEDERAL_STATUTE.md)** — bill text in U.S. Code drafting style (v0.1, post 5-role review)
- **[`STATUTE_RATIONALE.md`](docs/legal/STATUTE_RATIONALE.md)** — policy memo: why Congress, why reproducibility, why bisection, why now
- **[`STATUTE_ONE_PAGER.md`](docs/legal/STATUTE_ONE_PAGER.md)** — 90-second version for staff briefings
- **[`FAIRNESS_DOCTRINE.md`](docs/legal/FAIRNESS_DOCTRINE.md)** — state-court companion for partisan-gerrymandering litigation post-*Rucho*

The architecture: **Algorithm and parameters → federal. Execution → state. VRA § 2 deviations → state, with published *Gingles* justification. Verification → any citizen, byte-identically, with private right of action.**

---

## Why bisection is fair

Bisection is fair because it eliminates the choice. When you split something in half, neither side gets to pick which half is theirs — the cut is determined by the constraint (equal population) and the geometry, not by who benefits. Repeating that process recursively means every district is the product of a series of neutral halvings, not a single optimised design.

It's also transparent. You can watch each round of splitting in the dashboard and see exactly how a 52-district California map emerges from 6 rounds. There's no black box — just a sequence of cuts, each one as fair as splitting a pizza.

Procedural fairness is a stronger property than the substantive-fairness claims federal courts have rejected post-*Rucho*. "This map is fair" is contested; "this map is the byte-identical output of running this published algorithm on these published inputs" is verifiable.

## How it works

**In plain terms:** treat the state as a map of small neighborhoods connected where they share a border. Connection strength = length of that shared border. The algorithm cuts the map in half by snipping the weakest set of connections that produces two halves with equal population — which naturally avoids long, jagged borders.

Formally: census tracts are nodes; two tracts are connected if they share a border; edge weight = shared boundary length. METIS splits the state into two equal-population halves. Each half is split again. This continues for ⌈log₂ N⌉ rounds until there are exactly N districts (or, under ApportionRegions, until the prime-factor tree is exhausted).

Because the algorithm minimises total edge-cut weight — and weight equals shared boundary length — it directly minimises total district perimeter. Polsby–Popper goes up automatically.

**No political or racial data enters the algorithm at any stage.** VRA mode uses demographic data only for edge weighting, not population balance; it is mutually exclusive with partisan-weighted mode per Callais.

### Bisection rounds — Minnesota (8 districts, 3 rounds)

| Round 1 (1 → 2) | Round 2 (2 → 4) | Round 3 (4 → 8) |
| :---: | :---: | :---: |
| ![](docs/figures/minnesota_round_1.png) | ![](docs/figures/minnesota_round_2.png) | ![](docs/figures/minnesota_round_3.png) |

### Bisection rounds — Alabama (7 districts, 3 rounds)

| Round 1 (1 → 2) | Round 2 (2 → 4) | Round 3 (4 → 7) |
| :---: | :---: | :---: |
| ![](docs/figures/alabama_round_1.png) | ![](docs/figures/alabama_round_2.png) | ![](docs/figures/alabama_round_3.png) |

## Track record

The same algorithm, unchanged, on 2010 Census data: Polsby–Popper **0.320** — only 10% lower than the 2020 result, despite a decade of demographic change. Geographic structure drives the outcome.

---

## Research papers

28 papers across five tracks. PDFs are pre-built and committed to [`docs/papers/`](docs/papers/). LaTeX sources live under [`research/`](research/). Papers marked *(draft)* have no compiled PDF yet.

Compile all papers locally: `cd research && make docs`

---

### Track A — Synthesis

**A.0 · Algorithmic Objectivity for Congressional Redistricting: A National-Scale Demonstration** [[PDF](docs/papers/A.0+synthesis-metapaper.pdf)]
Synthesis metapaper. Frames the full research program: why bisection is the right method, what the national-scale results show, and what the implications are for redistricting reform. Covers all 50 states across three census decades.

**A.1 · Research Portfolio Guide** *(draft)*
Reader's guide to the paper portfolio — how the tracks relate, what order to read them in, and how findings build on each other.

**A.2 · Portfolio Summary** *(draft)*
One-document summary of all major findings across the portfolio for readers who want the headlines without the full papers.

---

### Track B — Algorithm

**B.0 · Algorithm Design Overview — Bakeoff** [[PDF](docs/papers/B.0+algorithm-design-overview.pdf)]
Head-to-head comparison of all eight algorithm modes (WI, NC, GA, AZ, MN, NV). Key finding: compactness–proportionality paradox. Includes Callais compliance table.

**B.02 · ApportionRegions: Redistricting as Geographic Completion of Huntington-Hill** [[PDF](docs/papers/B.02+one-federal-law.pdf)]
The one-sentence federal-law paper. HH (2 U.S.C. § 2a) priority sequence determines the prime factorization of k → the ApportionRegions bisection tree. Algorithm is derivable from existing statute.

**B.1 · Recursive Bisection for Congressional Redistricting** [[PDF](docs/papers/B.1+recursive-bisection.pdf)]
Baseline method paper. Graph-theoretic formulation, METIS recursive bisection under population balance, unweighted baseline results on all 50 states (2020 Census).

**B.2 · Edge-Weighted Recursive Bisection for Compact Congressional Districts** [[PDF](docs/papers/B.2+edge-weighted-bisection.pdf)]
Core contribution. Edges weighted by shared boundary length → minimising cut = minimising perimeter. Mean PP **0.361** (+22% over enacted 2020).

**B.3 · Single-Objective vs. Multi-Constraint METIS** [[PDF](docs/papers/B.3+multi-vs-edge.pdf)]
Why single-objective outperforms multi-constraint: objectives don't compete.

**B.4 · Equivalence of Recursive and N-Way Bisection** [[PDF](docs/papers/B.4+adaptive-bisection.pdf)]
With edge weights, both converge to the same solution. Method choice doesn't matter — the weighting does.

**B.5 · N-Way vs. Recursive Bisection (General)** *(draft)*
Comprehensive comparison across chambers and district counts.

**B.6 · Computational Complexity** *(draft)*
Runtime analysis for 50-state sweep; O(n log n) empirical.

**B.7 · Solution Space and Seed Sensitivity** *(draft)*
Characterises the seed space; motivates convergence sweep.

**B.8 · GeoSection — Ratio-Optimal First-Level Bisection** [[PDF](docs/papers/B.8+geosection-ratio-optimal-bisection.pdf)]
Tries all split ratios 1:(k-1)…⌊k/2⌋:⌈k/2⌉ at the first bisection; selects minimum isoperimetric-normalised edge-cut. NC: 5D/9R stable. Normalisation: EC/√(min(i,k−i)) prevents caterpillar pathology.

**B.9 · AreaSection — Dual Population-Area Constraint** [[PDF](docs/papers/B.9+areasection-dual-population-area-constraint.pdf)]
ncon=2 METIS [population, area]. 76% seat-outcome stability vs. standard bisection. Lorenz infeasibility filter. Regime boundary at area_swing=1.10.

**B.10 · Subdivision-Respecting Redistricting** *(draft)*
County-sticky weights: ×3.0 on county-interior edges, preserves county integrity without hard constraints.

**B.11 · ApportionRegions — Geographic Completion of Huntington-Hill** *(draft)*
Prime-factor bisection tree. k=14=7×2 → two 7-district sub-problems. k=17 (prime) → 9+8 binary fallback. NC: 7D/7R (−0.7pp gap). National 2020: 223D/209R.

**B.12 · ProportionalSection — Partisan Proportionality via Dual Constraint** *(draft)*
ncon=2 [population, D_votes]; proportionality paradox: competitive states have σ≈0. Rodden gap is Lorenz feasibility, not target calibration.

**B.13 · NestSection — Nested Multi-Chamber Redistricting** [[PDF](docs/papers/B.13+nestsection-nested-multi-chamber.pdf)]
Spine-compatible bisection ensuring senate = 2 × house at each level.

**B.14 · VRASection — Minority Geographic Alignment** [[PDF](docs/papers/B.14+vrasection-minority-opportunity-bisection.pdf)]
GeoSection ratio sweep with geographic alignment score for minority-opportunity districts. Post-Callais disentanglement: VRA score is orthogonal to partisan signal.

**B.15 · StabilitySection — Cross-Census Stability** [[PDF](docs/papers/B.15+stabilitysection-cross-census-stability.pdf)]
Bisection tree stability across 2000/2010/2020. Iowa: the most interesting unstable case (county population shifts break the 4=2×2 tree).

**B.16 · ConvergenceSweep — Empirical Seed Sufficiency** [[PDF](docs/papers/B.16+convergence-sweep.pdf)]
T=600 non-improving seeds is sufficient for all 50 states. Seed walk: SHA-256(census_release_id‖"DIA_SEED_V1"‖i). Deterministic statutory seed formula.

---

### Track C — Validation

**C.1 · Spatial Resolution and Algorithmic Redistricting: MAUP Sensitivity Analysis** [[PDF](docs/papers/C.1+maup-sensitivity.pdf)]
Tests whether results depend on the choice of geographic unit. Finds the algorithm is robust across 130× range in unit count — geography drives outcomes, not resolution.

**C.2 · Cross-Census Validation** [[PDF](docs/papers/C.2+cross-census-validation.pdf)]
Validates on 2000, 2010, and 2020 Census data. Compactness varies only ~10% across decades.

**C.3 · Cross-Census Temporal Stability** [[PDF](docs/papers/C.3+temporal-stability.pdf)]
Deep analysis of why the algorithm is temporally stable. Identifies geographic clustering properties that make bisection outcomes predictable.

**C.4 · Twenty Years of Congressional Redistricting: A Longitudinal Analysis** [[PDF](docs/papers/C.4+longitudinal-analysis.pdf)]
Tracks algorithmic vs. enacted district quality from 2000 to 2020. Quantifies the shrinking gap as redistricting reform spread.

**C.5 · Measuring Partisan Fairness: Efficiency Gap Analysis** [[PDF](docs/papers/C.5+efficiency-gap-analysis.pdf)]
Applies the efficiency gap metric to algorithmic districts. Near-zero efficiency gaps in most states — fairness as a byproduct of geometric neutrality.

---

### Track D — Voting Rights Act

**D.0 · VRA Compliance Through Edge-Weighted Graph Partitioning** [[PDF](docs/papers/D.0+vra-compliance.pdf)]
Introduces the `vra-aligned` edge-weighted formulation. Tests VRA Section 2 compliance on covered states.

**D.1 · The 42% Threshold: Geographic Limits of VRA Compliance** [[PDF](docs/papers/D.1+threshold-analysis.pdf)]
Critical empirical finding: states where minority population exceeds ~42% statewide achieve all statutory majority-minority district targets with principled methods.

**D.2 · N-Way vs. Recursive Bisection for VRA-Compliant Redistricting** [[PDF](docs/papers/D.2+nway-vs-recursive-vra.pdf)]
Compares n-way and recursive approaches for minority district formation.

**D.3 · Quantifying the VRA–Compactness Tradeoff** [[PDF](docs/papers/D.3+compactness-tradeoff.pdf)]
Measures the exact compactness cost of VRA compliance. Finds the tradeoff is real but bounded.

**D.4 · Legal Implementation Framework** *(draft)*
How the algorithmic approach maps to existing VRA § 2 litigation standards and post-Callais requirements.

---

### Track E — Experimental Extensions

**E.1 · Multi-Member Districts and Proportional Representation** [[PDF](docs/papers/E.1+multi-member-districts.pdf)]
Extends bisection to multi-member districts. 3- and 5-member districts dramatically reduce the compactness–VRA tension.

**E.2 · Direct County Representation** [[PDF](docs/papers/E.2+county-representation.pdf)]
Proposes using counties as the unit of congressional representation, weighted by population. Eliminates district-drawing entirely.

**E.3 · National Redistricting Without State Boundaries** [[PDF](docs/papers/E.3+national-redistricting.pdf)]
Asks what would happen if congressional districts could cross state lines. Applies the algorithm nationally — 435-district map of the entire US.

**E.4 · Partisan Similarity Districts: Algorithmic Safe Seats** [[PDF](docs/papers/E.4+partisan-similarity-districts.pdf)]
Investigates whether geographic bisection inadvertently creates partisan safe seats by clustering politically similar communities.

**E.5 · Partisan Fairness Through Algorithmic Districting** [[PDF](docs/papers/E.5+party-based-allocation.pdf)]
Evaluates whether algorithmic redistricting produces partisan-fair outcomes without intentionally targeting fairness.

**E.6 · International Applications** [[PDF](docs/papers/E.6+international-applications.pdf)]
Applies the redistricting algorithm to parliamentary systems in other countries with single-member districts.

---

## Getting the Data

```bash
# Download 2020 census data
redist fetch --year 2020 --workers 8

# Download all three census years
redist fetch --year all --workers 8

# Convert adjacency pkl files to fast native format (one-time after download)
python scripts/data/generate_adj_bin.py --year 2020
```

Pre-built adjacency files from GitHub Releases (requires `gh auth login`):
```bash
redist fetch --year 2020 --release
```

| Release | Contents | Size |
|---------|----------|------|
| [`data-inputs-v1`](https://github.com/giodl73-repo/REDIST/releases/tag/data-inputs-v1) | Elections, demographics, baseline enacted districts, adjacency graphs | 736 MB |
| [`outputs-v3`](https://github.com/giodl73-repo/REDIST/releases/tag/outputs-v3) | V3 edge-weighted results (all 50 states) | 3 MB |

---

## Project structure

```
redist/               # Rust workspace — production binary
  crates/
    redist-cli/       # Binary: all commands (build, state, states, fetch, analyze, map, …)
    redist-apportion/ # ApportionRegions compositor, split.rs (C METIS primary, Rust shadow)
    redist-core/      # Bisection, edge-weighting, FIPS, population balance
    redist-data/      # TIGER reading, adjacency build, .adj.bin serialization
    redist-analysis/  # Compactness, VRA, political, demographic, bloc-voting, ensemble
    redist-map/       # Native SVG→PNG map rendering (resvg, Liberation Sans embedded)
    redist-report/    # Commission report generation (JSON/HTML/PDF stub)
    redist-metis/     # Pure-Rust METIS implementation (shadow oracle)
configs/              # YAML plan configs (configs/{label}.yml)
runs/                 # Build outputs (gitignored, created by redist build)
analysis/             # Analysis outputs (gitignored)
reports/              # Report outputs (gitignored)
data/{year}/          # Raw census data (gitignored, ~55 GB)
outputs/              # Legacy pipeline outputs (gitignored)
docs/                 # REDIST_CLI.md (canonical CLI reference), legal/, quickstart/
docs/specs/           # Spec documents (Spec 7: label run management, etc.)
notebooks/            # Research notebooks (5 stubs with runtime-budget metadata)
scripts/              # Python: data download, election sources, dashboard generation
archive/              # Forensic reference: archive/python-pipeline-final/ (sealed)
context/              # Developer / AI-assistant docs
```

---

## Algorithm constraints

- **Population**: within ±0.5% of state target per district (congressional one-person-one-vote)
- **Contiguity**: all districts are connected subgraphs
- **Compactness**: edge-cut minimisation = perimeter minimisation = Polsby–Popper maximisation
- **No political or racial data** in the standard algorithm; VRA mode and proportional mode are opt-in and mutually exclusive

---

## Documentation

- [`docs/REDIST_CLI.md`](docs/REDIST_CLI.md) — complete CLI reference (all commands, all flags)
- [`docs/specs/2026-05-04-spec7-run-manifest.md`](docs/specs/2026-05-04-spec7-run-manifest.md) — Spec 7: label-based run management
- [`docs/legal/`](docs/legal/) — model federal statute, fairness doctrine, partisan options
- [`docs/quickstart/`](docs/quickstart/) — persona-specific quickstart guides
- [`docs/error-conventions.md`](docs/error-conventions.md) — `[INPUT]`/`[CONFIG]`/`[NETWORK]`/`[INTERNAL]` error categories
- [`docs/CHANGELOG.md`](docs/CHANGELOG.md) — version history
- [`context/ARCHITECTURE.md`](context/ARCHITECTURE.md) — system architecture
- [`context/CODING_PATTERNS.md`](context/CODING_PATTERNS.md) — Rust coding conventions

---

## Interactive dashboards

| Dashboard | Census year | Algorithm | Link |
|-----------|-------------|-----------|------|
| 2020 Results | 2020 | Edge-weighted bisection | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_2020.html) |
| 2010 Results | 2010 | Edge-weighted bisection | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_2010.html) |
| 2000 Results | 2000 | Edge-weighted bisection | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_2000.html) |
| VRA (2020) | 2020 | VRA edge-weighted | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_vra.html) |

---

## License

[MIT](LICENSE) © 2026 Gio Della-Libera
