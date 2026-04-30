# Callais Evidence Layer: Within-Party Racial Bloc Voting
**Date:** 2026-04-30
**Status:** Proposed; pending review
**Closes gap for:** §2 plaintiff's expert post-Callais (★★★→★★★★★), academic researcher
**Depends on:** existing partisan-shares producer + redist-analysis::bootstrap_ci
**Estimated effort:** 5-7 days

## Why this exists

*Louisiana v. Callais* (2026-04-29, 608 U.S. ___) updated the *Gingles* second and third preconditions: plaintiffs must "provide an analysis that controls for party affiliation, showing that voters engage in racial-bloc voting that cannot be explained by partisan affiliation" (majority p. 30). Cross-party racial polarization (Black voters mostly Democrats, white voters mostly Republicans) no longer counts. **Within-party** racial polarization does — but proving it requires data and methodology our project does not currently provide.

Today the project has the infrastructure (race-blind illustrative maps, partisan-weighted alternatives, partisan metrics, bootstrap CI machinery) but not the analytical layer that turns data into §2 evidence. This spec closes that gap.

## Scope

### In scope

1. **Race-of-candidate annotation file format** — A registered file format mapping candidate names + cycles to race classifications (Black, white, Hispanic, Asian, etc.) used in §2 expert practice.

2. **Within-party racial bloc voting analysis** — Per-precinct: did Black voters and white voters vote differently in the same party's primary? Computed via:
   - Ecological inference (King's method or RxC variant) restricted to a single party's primary
   - Or two-by-two ecological regression on (Black share, candidate-X share) over precincts within the party

3. **Disentanglement regression** — Per-district or per-state model:
   ```
   candidate_minority_pref_share ~ pct_minority + pct_dem_baseline + ε
   ```
   Where `candidate_minority_pref_share` is the share for the minority-preferred candidate, `pct_minority` is the racial composition, `pct_dem_baseline` is a partisan baseline from prior cycles. The Callais standard requires the `pct_minority` coefficient remain significant after controlling for partisan baseline.

4. **CLI surface** — New subcommand `redist analyze --types bloc-voting` that:
   - Loads precinct-level primary results + tract demographics + race-of-candidate file
   - Runs the disentanglement regression
   - Returns coefficients, standard errors, and bootstrap CIs
   - Writes `analysis/bloc_voting.json` and `analysis/bloc_voting_summary.md`

5. **Documentation** — A `docs/file-formats/race-of-candidate.md` spec, an updated `docs/REDIST_CLI.md` section, and an example end-to-end run for Louisiana 2020 primaries (uses the existing OpenElections fetcher).

### Out of scope (deferred to future work)

- Multi-cycle robustness (Callais requires "current data" but doesn't bound how many cycles)
- Causal inference (we provide associational, not causal — courts ask for the former)
- Survey-based race attribution (we use registry; voter file integration is a separate spec)
- Catalist / L2 commercial voter file integration (paid, out of scope for an open project)

## Inputs

| Input | Format | Source |
|---|---|---|
| Precinct election results | OpenElections per-state CSV (or any source the registry covers) | existing election-source registry |
| Tract demographics | `data/{year}/demographics/{state}_demographics_{year}.csv` | existing pipeline |
| Race-of-candidate annotation | `data/elections/race_of_candidate/{year}-{election}.csv` (NEW) | manual or community-curated |
| Adjacency / GEOID mapping | existing `.adj.bin` | existing pipeline |
| Tract↔precinct alignment (optional, for high-fidelity) | per-state assignment CSV | OpenElections + GIS overlay |

## Outputs

```
outputs/{version}/states/{state_name}/analysis/
├── bloc_voting.json              # full machine-readable results
└── bloc_voting_summary.md        # plain-English summary suitable for expert report
```

`bloc_voting.json` schema:

```json
{
  "analyzer": "bloc-voting",
  "available": true,
  "state": "louisiana",
  "year": "2020",
  "election": "presidential-primary",
  "party": "democratic",
  "method": "ecological-regression-with-party-control",
  "ecology": {
    "n_precincts": 4082,
    "candidates_analyzed": ["Biden", "Sanders", "Warren"],
    "race_of_candidate": {"Biden": "white", "Sanders": "white", "Warren": "white"}
  },
  "regression": {
    "model": "candidate_X_share ~ pct_black + pct_dem_baseline",
    "candidate": "Biden",
    "coefficients": {
      "intercept": {"estimate": 0.42, "stderr": 0.03, "ci_95": [0.36, 0.48]},
      "pct_black":   {"estimate": 0.61, "stderr": 0.04, "ci_95": [0.53, 0.69]},
      "pct_dem_baseline": {"estimate": -0.12, "stderr": 0.05, "ci_95": [-0.22, -0.02]}
    },
    "n": 4082,
    "r_squared": 0.71,
    "callais_significance": {
      "race_coefficient_significant_after_party_control": true,
      "p_value": 0.0001,
      "interpretation": "Black voters' candidate preference within the Democratic primary differs from white Democratic voters' preference at p < 0.001, holding partisan baseline constant. Satisfies Callais Gingles 2/3 within-party racial bloc voting test."
    }
  },
  "provenance": { /* same provenance block as other outputs */ }
}
```

## Algorithmic approach

### Method 1 (default): Ecological regression with party control

For each cycle's primary results in a chosen party:
1. Compute `pct_black` per precinct from tract-aggregated demographics
2. Compute `pct_dem_baseline` per precinct from prior-cycle general election share
3. Compute `candidate_share` per precinct for each named candidate
4. Fit OLS: `candidate_share ~ pct_black + pct_dem_baseline`
5. Bootstrap CIs by resampling precincts (B=10000)
6. Report per-candidate coefficients and the Callais significance flag

### Method 2 (research): RxC ecological inference

For each precinct, model the joint distribution of (race × candidate-vote) using King's RxC method (or a simpler 2×2 variant). Returns posterior means + credible intervals for "P(votes for candidate X | Black voter)" within the chosen party.

We ship Method 1 first because it's simpler, defensible, and the Callais language explicitly contemplates regression-style controls. Method 2 is a future research mode.

## CLI surface

```
redist analyze --label vt_test --types bloc-voting [options]

Options:
  --party <DEM|REP>             which party's primary to analyze (default: DEM)
  --election <NAME>             e.g., presidential-primary, senate-primary (default: presidential-primary)
  --year <YYYY>                 election year
  --race-of-candidate <PATH>    CSV mapping candidate -> race
  --partisan-baseline <PATH>    CSV with prior-cycle partisan baseline
  --bootstrap-samples <N>       default 10000
  --method <ols|rxc>            default ols (Method 1)
```

## Tests

- L0: pure-function unit tests for OLS, bootstrap, regression with synthetic data
- L1: end-to-end test on synthetic 100-precinct dataset where ground truth is known
- L2: skipped-by-default acceptance test that runs against Louisiana 2020 primary (requires OpenElections clone + race-of-candidate annotation)

## Risks

| Risk | Mitigation |
|---|---|
| Race-of-candidate annotation is subjective and contested | Document source of each annotation; allow user override; ship a curated default for major cycles only |
| Ecological inference assumptions don't hold in some districts | Output diagnostics (residual analysis, leverage); honest caveat in summary |
| Different ecological-inference methods give different answers | Default to OLS for transparency; offer RxC as opt-in |
| Bootstrap CI undercoverage with small precincts | Document minimum precinct count; refuse to run below threshold (configurable) |

## Definition of done

- `redist analyze --types bloc-voting` runs end-to-end on Louisiana 2020 with real data
- Output JSON validates against schema
- Plain-English summary reads as defensible §2 testimony language
- L0/L1 tests passing
- Tutorial in docs/REDIST_CLI.md walks through one full run
