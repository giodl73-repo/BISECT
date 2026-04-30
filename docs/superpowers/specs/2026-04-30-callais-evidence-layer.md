# Callais Evidence Layer: Within-Party Racial Bloc Voting
**Date:** 2026-04-30
**Updated:** 2026-04-30 (v2 — incorporates SCALE BLOCK + BOUNDARY/MERIDIAN/DATUM/COVENANT/TRENCH revisions)
**Status:** Revised; pending re-review
**Closes gap for:** §2 plaintiff's expert post-Callais (★★★→★★★★★), academic researcher
**Depends on:** existing partisan-shares producer + redist-analysis::bootstrap_ci
**Estimated effort:** 7-9 days (v2: more rigor)

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
    "specification": "WLS weighted by precinct vote count; HC3 robust SE; cluster-bootstrap by county B=10000",
    "candidate": "Biden",
    "diagnostics": {
      "vif_pct_black_vs_pct_dem_baseline": 3.2,
      "vif_underpowered_flag": false,
      "ci_naive_vs_cluster_diverged": false
    },
    "coefficients": {
      "intercept": {"estimate": 0.42, "stderr_hc3": 0.03, "ci_95_cluster": [0.36, 0.48]},
      "pct_black": {"estimate": 0.61, "stderr_hc3": 0.04, "ci_95_cluster": [0.51, 0.71], "standardized_beta": 0.58},
      "pct_dem_baseline": {"estimate": -0.12, "stderr_hc3": 0.05, "ci_95_cluster": [-0.24, 0.00]}
    },
    "n_precincts": 4082,
    "n_clusters": 64,
    "r_squared": 0.71,
    "p_values": {
      "pct_black_raw": 0.0001,
      "pct_black_holm_corrected": 0.0007,
      "pct_dem_baseline_raw": 0.018,
      "pct_dem_baseline_holm_corrected": 0.054
    },
    "robustness_check": {
      "baselines_tested": ["statewide_dem_share", "district_dem_share", "prior_primary_dem_share"],
      "race_coefficient_significant_under_all": true,
      "race_coefficient_min_estimate": 0.55,
      "race_coefficient_max_estimate": 0.64
    },
    "ecology_caveat": "This analysis assumes ecological inference validity; ecological fallacy cannot be ruled out without individual-level data. Coefficients describe per-precinct associations, not individual-voter behavior.",
    "draft_interpretation": "[DRAFT — expert witness should rewrite] Black-share has a positive, large, and robust association with Biden's vote share within the Democratic primary, after controlling for partisan baseline. β=0.61 (95% cluster-bootstrap CI [0.51, 0.71]; Holm-corrected p=0.0007). Robust under three alternative partisan-baseline definitions."
  },
  "race_of_candidate_provenance": {
    "source_file": "data/elections/race_of_candidate/2020-presidential-primary.csv",
    "source_sha256": "<computed at run time>",
    "curator": "<from CSV header — required>",
    "curator_attestation_date": "<from CSV header — required>",
    "annotations_independently_verified": false,
    "schema_version": 1
  },
  "provenance": { /* same provenance block as other outputs */ }
}
```

## Algorithmic approach (revised v2 per SCALE/MERIDIAN)

### Method 1 (default): Weighted ecological regression with party control

For each cycle's primary results in a chosen party:
1. Compute `pct_black` per precinct from tract-aggregated demographics
2. Compute `pct_dem_baseline` per precinct from prior-cycle general election share
3. Compute `candidate_share` per precinct for each named candidate
4. **Compute precinct weights** = total_votes_in_precinct (handles 100× size variance — SCALE blocker fix)
5. **Diagnostic gate**: compute Variance Inflation Factor for `pct_black` vs `pct_dem_baseline`. If VIF > 5, output a "underpowered for causal claims" flag in the JSON; do not block the run (MERIDIAN recommendation).
6. Fit **WLS** (weighted by precinct vote count) with **HC3 robust standard errors** (heteroskedasticity correction):
   `candidate_share ~ pct_black + pct_dem_baseline`, weights = precinct vote count
7. **Cluster-bootstrap by county** (B=10000) — resample counties with replacement, then all precincts in each sampled county. Naive precinct resampling ignores spatial structure (SCALE blocker fix). Report both naive and cluster CIs; flag divergence > 0.05.
8. **Multiple-testing correction** (SCALE blocker): all p-values for a (state, cycle) pair are corrected via Holm-Bonferroni step-down. The corrected p-value is reported alongside the raw p-value. Default α = 0.05, configurable via `--alpha`.
9. **Robustness check** (BOUNDARY recommendation): re-run with three alternative partisan baselines (statewide D-share, district D-share, prior-cycle primary D-share). Report whether the race coefficient remains significant under all three. A "robust" result is significant under all baselines after Holm correction.
10. Report per-candidate **coefficients with 95% CI** + **effect size** (standardized β) + plain-English interpretation. **Do not output a binary significance flag** (SCALE blocker fix).

### Method 2 (research): RxC ecological inference

For each precinct, model the joint distribution of (race × candidate-vote) using King's RxC method (or a simpler 2×2 variant). Returns posterior means + credible intervals for "P(votes for candidate X | Black voter)" within the chosen party.

We ship Method 1 first because it's simpler, defensible, and the Callais language explicitly contemplates regression-style controls. Method 2 is a future research mode.

**Caveat language for both methods:** outputs include this fixed disclaimer: *"This analysis assumes ecological inference validity; ecological fallacy cannot be ruled out without individual-level data. Coefficients describe per-precinct associations, not individual-voter behavior."*

### Confidence-interval level (standardized v2)

All CIs in this spec are **95% percentile bootstrap CIs** unless explicitly noted. The CI level is configurable via `--ci-level <0.0..1.0>`; default 0.95.

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

## Race-of-candidate annotation: full provenance protocol (v2)

The race-of-candidate file is the most contested data input in this analysis. v2 specifies a chain-of-custody protocol per BOUNDARY/COVENANT/DATUM/TRENCH consensus:

### File format

`data/elections/race_of_candidate/{cycle}.csv` with required columns:

```
candidate_name,party,race,curator,curator_credentials,curator_attestation_date,source,independently_verified
"Joseph Biden",DEM,white,"Jane Doe","Ph.D. Political Science",2026-04-30,"https://en.wikipedia.org/wiki/Joe_Biden",true
```

### Schema validation at import

- `race` field must be one of: `Black, white, Hispanic, Asian, Native, other` (case-sensitive). Any other value → import refuses with a clear error.
- `curator` and `curator_attestation_date` required (no nulls).
- `independently_verified` is a boolean; if `false`, the bloc-voting JSON output sets `annotations_independently_verified: false` and the draft interpretation includes the caveat "annotations not independently verified."

### Sensitivity analysis (DATUM recommendation)

For close-margin annotations (any candidate where the regression coefficient changes by more than 10% under a single annotation flip), the analysis runs all permutations and reports a "robust under annotation perturbation" flag. Default tolerance configurable via `--annotation-sensitivity <FLOAT>`.

### Reproducibility-package inclusion (BOUNDARY blocker)

The race-of-candidate CSV **must be included** in the reproducibility package emitted by `redist report` (see Court Submission Reports spec v2). Without it, the analysis is not Daubert-defensible.

### Dispute-resolution

When two curators disagree on an annotation, the file format supports multiple rows per candidate (one per curator). The analysis runs once per curator's annotation set; results from each are reported side by side. The expert witness picks which to defend.

## Risks (v2)

| Risk | Mitigation |
|---|---|
| Race-of-candidate annotation is subjective and contested | Schema-validated import; required curator attestation; sensitivity analysis; reproducibility-package inclusion; dispute-resolution via parallel curator runs |
| Ecological inference assumptions don't hold in some districts | Output VIF, leverage, residual diagnostics; fixed caveat in every summary |
| Different ecological-inference methods give different answers | Default to WLS+HC3 for transparency; document expected divergence from RxC; offer RxC as opt-in for sensitivity |
| Bootstrap CI undercoverage with small precincts | Cluster-bootstrap by county; minimum precinct count gate (`--min-precincts`, default 50); refuse with clear error below |
| Multiple-testing inflation across 15-30 candidate × cycle × race tests | Holm-Bonferroni step-down correction reported alongside raw p-values; family defined as (state × cycle × party) |
| Multicollinearity between race and party | VIF diagnostic; if VIF > 5, "underpowered for causal claims" flag in JSON |
| Auto-generated interpretation prose creates expert-witness liability | Output is `draft_interpretation` with explicit `[DRAFT — expert witness should rewrite]` prefix; expert writes the final language with JSON as audit trail |

## Definition of done

- `redist analyze --types bloc-voting` runs end-to-end on Louisiana 2020 with real data
- Output JSON validates against schema
- Plain-English summary reads as defensible §2 testimony language
- L0/L1 tests passing
- Tutorial in docs/REDIST_CLI.md walks through one full run
