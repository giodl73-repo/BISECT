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

Three implementations are available. The engine is also selectable in the YAML config via `engine:`.

| Value | Aliases | Backend | System requirement | Notes |
|-------|---------|---------|-------------------|-------|
| `c-ffi` | `c`, `metis-rs` | `metis` Rust FFI crate → C METIS library | `libmetis.so/.dll/.dylib` installed | **Default.** Battle-tested, handles all k values including prime k without stalling. |
| `redist-metis` | `rust` | Pure-Rust METIS (`redist-metis` crate) | None — no C dependency | Fully portable standalone binary. May stall on prime k for large graphs (>1000 tracts). Use when shipping `redist.exe` without system libs. |
| `gpmetis` | `subprocess` | External `gpmetis` binary | `gpmetis` on PATH | Reserved — returns a clear error today. Planned for environments where only the gpmetis binary is available. |

**For portable distribution**: build with `--metis-engine redist-metis` (or set `engine: redist-metis` in your config YAML). The resulting binary requires no installed METIS and runs on any machine.

**For production runs** (official proposal, 50-state sweeps): use the default `c-ffi`. It handles all edge cases including prime district counts like PA (k=17).

```bash
# Portable run — no libmetis required
redist state --state VT --year 2020 --version vt_test --metis-engine redist-metis

# Default C METIS FFI (explicit)
redist state --state PA --year 2020 --version pa_v1 --metis-engine c-ffi

# In YAML config
# algorithm:
#   engine: redist-metis   # or c-ffi (default), gpmetis (reserved)
```

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

## Research (B series — Algorithm track)

16 papers characterising the algorithm, its variants, and their properties. LaTeX sources under directory siblings of this file; see each paper's `main.pdf`.

### Foundations

**B.1 · Recursive Bisection for Congressional Redistricting** — graph-theoretic formulation, METIS bisection, unweighted baseline (50 states, 2020).

**B.2 · Edge-Weighted Recursive Bisection** — core contribution. Edges weighted by shared boundary length → minimising cut = minimising perimeter. Mean PP **0.361**, +22% over enacted 2020.

**B.3 · Single-Objective vs. Multi-Constraint METIS** — why single-objective outperforms: objectives don't compete.

**B.4 · Equivalence of Recursive and N-Way Bisection** — with edge weights, both converge to the same solution.

**B.5 · N-Way vs. Recursive Bisection (General)** — comprehensive comparison across chambers and district counts.

**B.6 · Computational Complexity** — runtime analysis for 50-state sweep; O(n log n) empirical.

**B.7 · Solution Space and Seed Sensitivity** — characterises the seed space; motivates convergence sweep.

### New algorithms (2026)

**B.8 · GeoSection — Ratio-Optimal First-Level Bisection** — tries all split ratios 1:(k-1)…⌊k/2⌋:⌈k/2⌉ at the first bisection; selects minimum isoperimetric-normalised edge-cut. NC: 5D/9R stable. Normalisation: EC/√(min(i,k−i)) prevents caterpillar pathology.

**B.9 · AreaSection — Dual Population-Area Constraint** — ncon=2 METIS [population, area], ubvec=[1.001, 1+swing]. 76% seat-outcome stability vs. standard bisection. Lorenz infeasibility filter. Regime boundary at area_swing=1.10.

**B.10 · Subdivision-Respecting Redistricting** — county-sticky weights: ×3.0 on county-interior edges, preserves county integrity without hard constraints.

**B.11 · ApportionRegions — Geographic Completion of Huntington-Hill** — prime-factor bisection tree. k=14=7×2 → two 7-district sub-problems. k=17 (prime) → 9+8 binary fallback. NC: 7D/7R (−0.7pp gap vs. 51.6% D vote share). Constitutional anchor: Art. I §2 apportionment → §4 manner. National 2020: 223D/209R.

**B.12 · ProportionalSection — Partisan Proportionality via Dual Constraint** — ncon=2 [population, D_votes]; proportionality paradox: competitive states have σ≈0 (target ≈ neutral). Rodden gap is Lorenz feasibility, not target calibration.

**B.13 · NestSection — Nested Multi-Chamber Redistricting** — spine-compatible bisection ensuring senate = 2 × house at each level.

**B.14 · VRASection — Minority Geographic Alignment** — GeoSection ratio sweep with geographic alignment score for minority-opportunity districts. Post-Callais disentanglement: VRA score is orthogonal to partisan signal.

**B.15 · StabilitySection — Cross-Census Stability** — bisection tree stability across 2000/2010/2020. Iowa identified as the most interesting unstable case (county population shifts break the 4=2×2 tree at one level).

**B.16 · ConvergenceSweep — Empirical Seed Sufficiency** — T=600 non-improving seeds is sufficient for all 50 states. Seed walk: SHA-256(census_release_id‖"DIA_SEED_V1"‖i). Deterministic statutory seed formula.

### Synthesis and policy bridge

**B.0 · Algorithm Design Overview — Bakeoff** — head-to-head comparison of all eight algorithm modes on the same six competitive states (WI, NC, GA, AZ, MN, NV). Key finding: the compactness–proportionality paradox. WI unweighted gives 4D/4R (proportional) but poor compactness; GeoSection gives 3D/5R (Republican-leaning) at higher compactness. The two objectives trade off systematically — no single mode dominates. Includes a Callais compliance table (VRA ⊕ partisan constraints, mutually exclusive per Callais p.36) and the cross-mode `redist analyze --types proportionality` comparison lens.

**B.02 · ApportionRegions: Redistricting as Geographic Completion of Huntington-Hill** — the one-sentence federal-law paper. Huntington-Hill (2 U.S.C. § 2a) allocates seats to states by priority sequence; the same priority sequence determines the prime factorization of k, which determines the ApportionRegions bisection tree. The algorithm is therefore derivable from existing statute — it is HH extended from "how many seats?" to "where are the districts?". Paper includes the county-weights connection (counties appear in the HH priority sequence for multi-county states) and the census-stability connection (B.15: the bisection tree is stable across census years when county populations are stable).

**B.16 · ConvergenceSweep — Empirical Seed Sufficiency** — answers: how many seeds are enough? Runs all 50 states with the convergence-sweep search strategy (walk seeds from SHA-256 chain, stop after T non-improving). Empirical result: T=600 is sufficient for all 50 states across all three census years — no state benefits from running beyond 600 consecutive non-improving seeds. Seed generation formula: `BLAKE3(census_release_id ‖ "DIA_SEED_V1" ‖ i)`. This formula is deterministic (given the census release ID), replicable by any verifier, and statute-safe (no human choice in seed selection). T=600 is the recommended statutory value in the model federal statute draft.

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
