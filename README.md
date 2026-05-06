# redist — Algorithmic Redistricting

`redist` draws congressional and state-legislative districts that are compact, population-balanced, reproducible to the byte, and derivable from first principles — for any chamber, any state, any census year.

## How it works

**In plain terms:** treat a state as a map of small census-tract neighborhoods. Two tracts are connected if they share a border; the connection is *stronger* the longer that border is. The algorithm cuts the map in half by snipping the weakest set of connections that produces two equal-population halves — which naturally avoids long, jagged borders. Repeat recursively until you have exactly N districts.

Formally: tracts are nodes; edge weight = shared boundary length. Minimising the total weight of edges cut directly minimises total district perimeter, so **Polsby–Popper compactness goes up automatically** — without ever being told to optimise it.

**No political or racial data enters the algorithm at any stage.** VRA mode uses demographics only for edge weighting; it is mutually exclusive with partisan-weighted mode per *Louisiana v. Callais* (2026).

### Bisection rounds — Minnesota (8 districts, 3 rounds)

| Round 1 (1 → 2) | Round 2 (2 → 4) | Round 3 (4 → 8) |
| :---: | :---: | :---: |
| ![](docs/figures/minnesota_round_1.png) | ![](docs/figures/minnesota_round_2.png) | ![](docs/figures/minnesota_round_3.png) |

### Bisection rounds — Alabama (7 districts, 3 rounds)

| Round 1 (1 → 2) | Round 2 (2 → 4) | Round 3 (4 → 7) |
| :---: | :---: | :---: |
| ![](docs/figures/alabama_round_1.png) | ![](docs/figures/alabama_round_2.png) | ![](docs/figures/alabama_round_3.png) |

## Why it's fair

Bisection is fair because it **eliminates the choice**. When you split something in half, neither side gets to pick which half is theirs — the cut is determined by equal-population geometry, not by who benefits. Repeating that recursively means every district is the product of a series of neutral halvings, not a single optimised design.

It's also **transparent**. You can watch each round in the dashboard and see exactly how a 52-district California map emerges from 6 rounds of bisection. No black box — just a sequence of cuts.

**Procedural fairness is a stronger claim than substantive fairness.** "This map is fair" is contested; "this map is the byte-identical output of running this published algorithm on these published inputs" is verifiable.

---

## Who it's for

| Who you are | Start here | Time |
|---|---|---|
| Court-appointed special master | [docs/quickstart/quickstart-special-master.md](docs/quickstart/quickstart-special-master.md) | 5–10 min |
| Academic researcher | [docs/quickstart/quickstart-researcher.md](docs/quickstart/quickstart-researcher.md) | 10–15 min |
| §2 plaintiff's expert (post-Callais) | [docs/quickstart/quickstart-callais-expert.md](docs/quickstart/quickstart-callais-expert.md) | 30–60 min |
| State legislative staff | [docs/quickstart/quickstart-state-staff.md](docs/quickstart/quickstart-state-staff.md) | 5 min/iteration |
| Civic advocacy group | [docs/quickstart/quickstart-civic-advocate.md](docs/quickstart/quickstart-civic-advocate.md) | 15–30 min |

**[→ All 66 research papers with PDFs](docs/PAPERS.md)**

First time? Run **`bash bootstrap.sh`** (Linux/macOS) or **`bootstrap.bat`** (Windows) — ≤ 10 minutes from `git clone` to first run.

---

## Quick start

```bash
# Build the binary
cargo build --release --manifest-path redist/Cargo.toml

# Download 2020 census data
redist fetch --year 2020

# Draw all 50 congressional maps (~15 seconds)
redist build official_2020 --year 2020 --workers 8

# Analyze: compactness, VRA, partisan lean, splits
redist label-analyze official_2020 --year 2020 --types all

# Generate a report
redist label-report official_2020 --year 2020 --format html json
```

---

## What it does

**Draw districts** — any chamber, any district count, any census year:

```bash
redist state --state WA --districts 98 --chamber house    # 98-seat state house
redist state --state WA --chamber congressional           # congressional (from config)
redist states --year 2020 --workers 8                     # all 50 states in parallel
```

**Analyse plans** — compactness, VRA compliance, partisan lean, county splits, contiguity:

```bash
redist label-analyze my_plan --year 2020 --types all
```

**Compare plans** — side-by-side metrics, Jaccard similarity, partisan differences:

```bash
redist label-compare plan_v1 plan_v2 --year 2020
```

**Verify integrity** — SHA-256 audit chain from config → build → analysis → report:

```bash
redist label-verify official_2020 --year 2020
```

**Generate reports** — HTML, JSON, or PDF with full provenance:

```bash
redist label-report official_2020 --year 2020 --format html json pdf
```

---

## Results at a glance

**2020 Census, 50 states, 435 congressional districts:**

| Metric | Algorithmic | Enacted | Change |
|--------|-------------|---------|--------|
| Mean Polsby–Popper | **0.361** | 0.296 | **+22%** |
| States beating enacted maps | 37 / 44 | — | |
| Partisan outcome (ApportionRegions) | 223D / 209R | — | Tracks 51.6% D vote share |

North Carolina ($k=14=7\times 2$): ApportionRegions gives **7D/7R** vs. standard bisection 5D/9R — structure, not criterion, drives partisan outcomes.

**[View all dashboards →](https://giodl73-repo.github.io/REDIST/)** — 2020, 2010, VRA results, round-by-round bisection maps.

---

## Track record

The same algorithm, unchanged, on 2010 data: Polsby–Popper **0.320** — only 10% lower than 2020, despite a decade of demographic change. Geographic structure is stable; political environments aren't.

---

## The case for a federal statute

*Rucho v. Common Cause* (2019) closed the federal-court door on partisan gerrymandering. Roberts' opinion explicitly invited Congress to act.

This project's thesis ([B.02](docs/papers/B.02+one-federal-law.pdf)): the gap can be closed by a statute structurally analogous to **2 U.S.C. § 2a** (Huntington-Hill apportionment). Congress prescribes the algorithm; states execute it; any citizen can verify the output byte-identically. **ApportionRegions** is the algorithmic proposal — the only redistricting method derivable from the existing apportionment statute.

Drafts: [`docs/legal/`](docs/legal/) — bill text, policy memo, one-pager, and state-court companion ([`FAIRNESS_DOCTRINE.md`](docs/legal/FAIRNESS_DOCTRINE.md)).

---

## Research papers

66 papers across eight tracks. All PDFs are pre-built.
**[→ Full index at docs/PAPERS.md](docs/PAPERS.md)**

### Track A — Synthesis
| | |
|--|--|
| [A.0](docs/papers/A.0+synthesis-metapaper.pdf) | National-scale demonstration: 50 states, three census decades |
| [A.1](docs/papers/A.1+portfolio-guide.pdf) | Portfolio guide: how the tracks relate, reading order |
| [A.2](docs/papers/A.2+portfolio-summary.pdf) | Portfolio summary: all major findings in one document |

### Track B — Algorithm (all 18 papers have PDFs)
| | |
|--|--|
| [B.0](docs/papers/B.0+algorithm-design-overview.pdf) | Bakeoff: compactness–proportionality paradox across 8 algorithm modes |
| [B.02](docs/papers/B.02+one-federal-law.pdf) | One federal law: ApportionRegions as the HH intrastate extension |
| [B.1](docs/papers/B.1+recursive-bisection.pdf) | Recursive bisection baseline (50 states, 2020) |
| [B.2](docs/papers/B.2+edge-weighted-bisection.pdf) | Edge-weighted bisection: +22% compactness, mean PP 0.361 |
| [B.3](docs/papers/B.3+multi-vs-edge.pdf) | Single-objective vs. multi-constraint METIS |
| [B.4](docs/papers/B.4+adaptive-bisection.pdf) | Equivalence of recursive and n-way bisection under edge weighting |
| [B.5](docs/papers/B.5+nway-vs-recursive-general.pdf) | N-way vs. recursive: general comparison across all chambers |
| [B.6](docs/papers/B.6+computational-complexity.pdf) | Computational complexity: O(n log² n), empirically O(n¹·⁰⁷) |
| [B.7](docs/papers/B.7+solution-space-and-seed-sensitivity.pdf) | Seed sensitivity: CV < 2% for 96% of states |
| [B.8](docs/papers/B.8+geosection-ratio-optimal-bisection.pdf) | GeoSection: ratio-optimal first bisection, isoperimetric normalisation |
| [B.9](docs/papers/B.9+areasection-dual-population-area-constraint.pdf) | AreaSection: dual [population, area] constraint, 76% seat stability |
| [B.10](docs/papers/B.10+subdivision-respecting-redistricting.pdf) | County-sticky weights: 34% fewer county splits at 3% compactness cost |
| [B.11](docs/papers/B.11+apportion-regions.pdf) | ApportionRegions: prime-factor tree, NC 7D/7R, national 223D/209R |
| [B.12](docs/papers/B.12+proportional-section.pdf) | ProportionalSection: proportionality paradox and Rodden-gap finding |
| [B.13](docs/papers/B.13+nestsection-nested-multi-chamber.pdf) | NestSection: senate = 2 × house spine compatibility |
| [B.14](docs/papers/B.14+vrasection-minority-opportunity-bisection.pdf) | VRASection: minority alignment, post-Callais disentanglement |
| [B.15](docs/papers/B.15+stabilitysection-cross-census-stability.pdf) | StabilitySection: tree stability 2000–2020 |
| [B.16](docs/papers/B.16+convergence-sweep.pdf) | ConvergenceSweep: T=600 sufficient, statutory seed formula |
| [B.17](docs/papers/B.17+parameter-sensitivity.pdf) | Parameter sensitivity: partisanship insensitive to tuning |
| [H.0](docs/papers/H.0+percentile-sweep.pdf) | PercentileSweep — statutory choice of legal posture |
| [H.1](docs/papers/H.1+bisection-ensemble.pdf) | BisectionEnsemble — local ReCom at each bisection node |
| [H.2](docs/papers/H.2+redist-ensemble.pdf) | redist-ensemble — Rust ReCom at 2500× speed |

### Track C — Validation
| | |
|--|--|
| [C.1](docs/papers/C.1+maup-sensitivity.pdf) | MAUP sensitivity: robust across 130× unit-count range |
| [C.2](docs/papers/C.2+cross-census-validation.pdf) | Cross-census validation: compactness varies only ~10% across decades |
| [C.3](docs/papers/C.3+temporal-stability.pdf) | Temporal stability: geographic clustering drives predictability |
| [C.4](docs/papers/C.4+longitudinal-analysis.pdf) | 20-year longitudinal analysis: enacted maps improving, gap narrowing |
| [C.5](docs/papers/C.5+efficiency-gap-analysis.pdf) | Efficiency gap: near-zero gaps as byproduct of geometric neutrality |

### Track D — Voting Rights Act
| | |
|--|--|
| [D.0](docs/papers/D.0+vra-compliance.pdf) | VRA compliance via vra-aligned edge weighting |
| [D.1](docs/papers/D.1+threshold-analysis.pdf) | The 42% threshold: geographic limits of VRA compliance |
| [D.2](docs/papers/D.2+nway-vs-recursive-vra.pdf) | N-way vs. recursive for VRA-compliant redistricting |
| [D.3](docs/papers/D.3+compactness-tradeoff.pdf) | VRA–compactness tradeoff: real but bounded |

### Track E — Experimental Extensions
| | |
|--|--|
| [E.1](docs/papers/E.1+multi-member-districts.pdf) | Multi-member districts: reduces compactness–VRA tension |
| [E.2](docs/papers/E.2+county-representation.pdf) | Direct county representation: eliminates district-drawing |
| [E.3](docs/papers/E.3+national-redistricting.pdf) | National redistricting without state boundaries |
| [E.4](docs/papers/E.4+partisan-similarity-districts.pdf) | Partisan similarity districts: geographic sorting creates safe seats |
| [E.5](docs/papers/E.5+party-based-allocation.pdf) | Partisan fairness through algorithmic districting |
| [E.6](docs/papers/E.6+international-applications.pdf) | International applications: parliamentary single-member systems |

---

## Reference

### Named plans — the label workflow

Plans are referenced by **labels** (short names like `official_2020`) that resolve to all paths by convention. Every stage writes a SHA-256 that chains to the previous stage: **config → build → analysis → report**.

```bash
redist build <label>              # draw districts
redist label-analyze <label>      # run metrics
redist label-report  <label>      # generate report
redist label-verify  <label>      # verify SHA chain
redist ls                         # list all plans
redist show <label>               # plan details
redist mv <old> <new>             # rename
redist label-compare <a> <b>      # side-by-side diff
```

Config file (`configs/<label>.yml`):
```yaml
name: official_2020
algorithm:
  structure: prime-factor     # ApportionRegions
  weights: county             # county-sticky
  search: convergence         # walk seeds until T non-improving
  convergence_threshold: 600
  balance_tolerance: 0.5
workers: 8
years: ["2020", "2010", "2000"]
```

### Algorithm options

Every run is defined by four orthogonal choices:

| Layer | Flag | Options |
|-------|------|---------|
| **Engine** | `--metis-engine` | `c-ffi` (default), `redist-metis` (portable, no C), `gpmetis` (planned) |
| **Structure** | `--structure` | `standard-bisect`, `prime-factor`, `ratio-optimal`, `ratio-optimal-area`, `ratio-optimal-vra`, `nway`, `compact-polsby` |
| **Weights** | `--weights-override` | `geographic` (default), `county`, `unweighted`, `vra-aligned`, `proportional` |
| **Search** | `--search` | `single` (deterministic), `multi` (N seeds), `convergence` (walk until T non-improving) |

**Engine note:** the default engine depends on how the binary was compiled.
`cargo build` → `c-ffi` (bundled C, fast, handles all k).
`cargo build --no-default-features` → `redist-metis` (pure Rust, portable, no C toolchain needed).

See [`docs/REDIST_CLI.md`](docs/REDIST_CLI.md) for the complete flag reference.

---

## Getting the data

```bash
redist fetch --year 2020 --workers 8    # download census data
redist fetch --year all                  # all three census years
redist fetch --year 2020 --release       # pre-built adjacency files (requires gh auth login)
```

---

## Project structure

```
redist/crates/
  redist-cli/       # All commands (build, state, fetch, analyze, map, …)
  redist-apportion/ # ApportionRegions compositor, METIS engine dispatch
  redist-core/      # Bisection, edge-weighting, FIPS, population balance
  redist-data/      # TIGER reading, adjacency, .adj.bin serialization
  redist-analysis/  # Compactness, VRA, political, demographic, bloc-voting
  redist-map/       # SVG→PNG map rendering
  redist-metis/     # Pure-Rust METIS (portable engine)
configs/            # YAML plan configs
docs/papers/        # 34 compiled PDFs (committed)
research/           # LaTeX sources for all papers
docs/legal/         # Model federal statute, fairness doctrine
docs/quickstart/    # Persona-specific quickstart guides
```

---

## Documentation

- **[`docs/REDIST_CLI.md`](docs/REDIST_CLI.md)** — complete CLI reference
- **[`docs/PAPERS.md`](docs/PAPERS.md)** — all 66 research papers with PDFs
- **[`docs/legal/`](docs/legal/)** — model statute, fairness doctrine
- **[`docs/quickstart/`](docs/quickstart/)** — persona guides
- **[`docs/CHANGELOG.md`](docs/CHANGELOG.md)** — version history

---

## Interactive dashboards

| Dashboard | Year | Link |
|-----------|------|------|
| 2020 results | 2020 | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_2020.html) |
| 2010 results | 2010 | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_2010.html) |
| 2000 results | 2000 | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_2000.html) |
| VRA (2020) | 2020 | [Open →](https://giodl73-repo.github.io/REDIST/dashboard_vra.html) |

---

## License

[MIT](LICENSE) © 2026 Gio Della-Libera
