# Plan: Callais Evidence Layer (Within-Party Racial Bloc Voting)

> **For agentic workers:** Do not execute until the spec at `docs/superpowers/specs/2026-04-30-callais-evidence-layer.md` v2 (with v2.1 patches) has been reviewed and approved. The 9-role re-review of 2026-04-30 lifted the SCALE BLOCK by adopting WLS+HC3, Holm-Bonferroni, cluster-bootstrap by county, VIF, robustness check, and the race-of-candidate provenance protocol — this plan implements those mitigations.

**Date:** 2026-04-30
**Spec:** `docs/superpowers/specs/2026-04-30-callais-evidence-layer.md`
**v2.1 tracking ref:** `docs/superpowers/specs/2026-04-30-v21-tracking.md`
**Goal:** Ship a defensible §2 within-party racial bloc voting analyzer behind `redist analyze --types bloc-voting`. The output JSON + summary must satisfy the *Callais* (608 U.S. ___, 2026-04-29) requirement that plaintiffs demonstrate within-party polarization that survives partisan controls — with full diagnostics, multiple-testing correction, cluster-aware uncertainty, and a chain-of-custody-grade race-of-candidate provenance pipeline that survives Daubert-style cross.

**Depends on:**
- Onboarding plan landed (`bootstrap.sh` so the LA walkthrough is reachable from a clean machine)
- Existing `partisan_shares.rs` TSV loader (read-only reuse)
- Existing `bootstrap_ci` machinery in `redist-analysis::partisan` (read-only reuse; new cluster bootstrap is a sibling, not a replacement)
- Election data registry at `scripts/data/elections/sources.json` (OpenElections fetcher already implemented; per-state schema mapping is a per-run input, not blocking)

**Blocks:**
- Court Submission Reports (consumes `bloc_voting.json` + the race-of-candidate reproducibility-zip artifact)
- `quickstart-callais-expert.md` (Onboarding Task 5.3) — the quickstart cannot land its end-to-end run before this plan ships

**v2.1 items addressed by this plan:**
- **S-02** — Holm family in Callais must include leave-one-out sensitivity-variant runs from Civic Bidirectional conflict resolution
- **B-02** (P0) — named L0 anchors: OLS coefficient ±0.02 on synthetic ground truth; Holm dominates raw on 30-test family; VIF>5 sets underpowered flag; `independently_verified=false` injects caveat
- **BD-R2** — race-of-candidate `attestation-doc` accepted formats, reproducibility-zip inclusion, `attestation_doc_sha256` + `attestation_doc_format` in `race_of_candidate_provenance` JSON

---

## Pre-Conditions

- `redist analyze --types <T>` dispatcher exists in `redist/crates/redist-cli/src/analyze.rs` and accepts new variants by extending the `AnalyzerType` enum in `redist/crates/redist-analysis/src/analyzer.rs`. No new dispatcher is needed.
- `redist-analysis` already depends on `csv`, `serde`, `serde_json`, `rand`. We will add a `linreg`-style WLS implementation in-tree (no external regression crate); SVD/Cholesky via `nalgebra` (already used elsewhere) is acceptable, but a from-scratch normal-equations solver for `p ≤ 5` predictors is sufficient and avoids adding a new dependency unless one is already present.
- The Callais p.36 mutex (VRA + partisan-weighting cannot both be on) lives in `runner.rs` and is **not** affected by this plan: bloc-voting analysis is post-hoc on a finished plan, not a redistricting mode.

---

## Task 1: New `bloc_voting` module — types, WLS+HC3 core, Holm

**Files:** `redist/crates/redist-analysis/src/bloc_voting.rs` (new), `redist/crates/redist-analysis/src/lib.rs`

This task lands the regression engine without any I/O or CLI surface.

- [ ] **1.1** Create `bloc_voting.rs`. Define types: `Precinct { id, county_fips, total_votes, candidate_share, pct_minority, pct_dem_baseline }`, `RegressionFit { coefficients: HashMap<String, Coef>, n, n_clusters, r_squared, vif_pct_minority_vs_baseline, vif_underpowered_flag, residuals: Vec<f64> }`, `Coef { estimate, stderr_hc3, ci_95_cluster, standardized_beta, p_value_raw, p_value_holm }`.
- [ ] **1.2** Implement `fit_wls(precincts, predictors)` using normal equations with weights = `total_votes`: solve `(Xᵀ W X) β = Xᵀ W y`. For numerical stability, center `X` columns before solve and back out the intercept. p ≤ 4 predictors → small-matrix solver fine.
- [ ] **1.3** Implement `hc3_stderr(X, W, residuals)` — White's HC3 sandwich estimator: `(XᵀWX)⁻¹ Xᵀ W diag(eᵢ²/(1-hᵢᵢ)²) W X (XᵀWX)⁻¹`, where `hᵢᵢ` are the diagonal of the hat matrix. This is the spec's "HC3 robust SE" line in `regression.specification`.
- [ ] **1.4** Implement `compute_vif(X, predictor_idx)` — VIF for one predictor against all others: regress predictor on the rest, return `1 / (1 - R²)`. Set `vif_underpowered_flag = vif > 5.0`. Spec line 215 + B-02 anchor 3.
- [ ] **1.5** Implement `holm_bonferroni(p_values: &[(String, f64)]) -> HashMap<String, f64>` — step-down: sort raw p ascending; corrected `pᵢ = max over j ≤ i of (m - j + 1) · pⱼ`; clamp to ≤ 1.0. Family is whatever the caller passes — for Callais the caller assembles the family per Task 3.
- [ ] **1.6** L0 unit tests for each: synthetic 100-row dataset where (a) WLS recovers known β=0.61 within ±0.02 (B-02 anchor 1); (b) HC3 SE within ±10% of bootstrap-derived SE; (c) VIF for two collinear predictors > 5.0 sets the flag (B-02 anchor 3); (d) Holm result on a hand-computed 5-test family matches by-hand math; (e) Holm dominates raw on a 30-test family (B-02 anchor 2 — at least one raw-significant test is no longer significant after correction, demonstrating the dominance relation `p_holm ≥ p_raw` holds elementwise).
- [ ] **1.7** Re-export from `lib.rs`: `pub use bloc_voting::{fit_wls, hc3_stderr, compute_vif, holm_bonferroni, RegressionFit, Coef, Precinct, BlocVotingError};`.

**Exit:** `cargo test -p redist-analysis bloc_voting::tests` green; all four B-02 L0 anchors named and asserted by name in `#[test]` function names (e.g. `test_b02_anchor1_ols_coefficient_within_002`).

---

## Task 2: Cluster bootstrap by county

**Files:** `redist/crates/redist-analysis/src/bloc_voting.rs`

The naive precinct bootstrap (already in `partisan::bootstrap_ci`) ignores spatial structure; SCALE blocker fix mandates resampling at the county level.

- [ ] **2.1** Implement `cluster_bootstrap(precincts, county_fn, n_samples, seed) -> ClusterCi` that resamples counties with replacement (n_clusters draws), then includes ALL precincts in each sampled county. For each resample, refit WLS and record coefficient vector.
- [ ] **2.2** Compute percentile CIs at the configured level (default 0.95). Also compute the *naive* (precinct-level) CI for the same family; record both. If `|naive_high - cluster_high| > 0.05` or `|naive_low - cluster_low| > 0.05`, set `ci_naive_vs_cluster_diverged = true` (spec line 90).
- [ ] **2.3** Use `SmallRng::seed_from_u64(seed)` from `rand` (already a dep). Default seed 42; configurable via `--bootstrap-seed`.
- [ ] **2.4** Performance gate: 4082 precincts × 64 clusters × B=10000 must complete in ≤ 60s on a developer laptop. If not, surface a `WARNING: cluster bootstrap took {n}s — consider --bootstrap-samples 2000` line; do not silently truncate.
- [ ] **2.5** L0 test: synthetic dataset with intentionally-correlated within-county residuals — assert cluster CI is wider than naive CI (the whole point of the correction). Second test: i.i.d. data — assert cluster CI ≈ naive CI within 5%.

**Exit:** Two L0 tests pass; performance run on synthetic 4000×60 hits under 60s.

---

## Task 3: Disentanglement orchestration + robustness check + Holm family with leave-one-out (S-02)

**Files:** `redist/crates/redist-analysis/src/bloc_voting.rs`

- [ ] **3.1** Implement `run_bloc_voting(inputs: BlocVotingInputs) -> BlocVotingResult` that, per analyzed candidate:
  1. Computes precinct-level `pct_black` (or `pct_minority` per `--minority-group`), `pct_dem_baseline`, `candidate_share`, total votes
  2. Calls `fit_wls` + `hc3_stderr` + `compute_vif`
  3. Calls `cluster_bootstrap`
  4. Repeats the fit under three alternative baselines per spec §10: `statewide_dem_share`, `district_dem_share`, `prior_primary_dem_share`. Records the min/max race coefficient across the four (primary + three robustness baselines).
- [ ] **3.2** **Holm family construction (S-02 — critical):** the family for Holm correction is **not** just the raw p-values from this state-cycle's analyzed candidates. It must include:
  - One p-value per candidate per primary baseline (the headline run)
  - One p-value per candidate per *robustness* baseline (3 alternates)
  - One p-value per candidate per *leave-one-out sensitivity variant* — when Civic Bidirectional conflict-resolution produces N alternative race-of-candidate annotation sets (each leaving one curator out), each set's race coefficient p-value joins the family

  Document the family-size formula in code comments: `m = n_candidates × (1 baseline + 3 robustness + n_loo_variants)`. This is the SCALE-blocker-lifting bargain — it inflates the correction but is the price of treating sensitivity variants as part of the inference, not as a side eye.
- [ ] **3.3** Race coefficient is "robust" iff its Holm-corrected p < α under *all* baselines AND all LOO variants. Set `robustness_check.race_coefficient_significant_under_all` accordingly. Default α = 0.05; configurable via `--alpha`.
- [ ] **3.4** Effect size: standardized β per candidate = `coefficient · (sd(predictor) / sd(response))`, computed on the weighted (WLS) variance. Report alongside raw β.
- [ ] **3.5** Annotation-perturbation sensitivity (spec §"Sensitivity analysis (DATUM)"): for any candidate where flipping a single annotation (Black ↔ white, etc.) changes the race coefficient by more than `--annotation-sensitivity` (default 0.10 → 10%), enumerate all single-flip permutations and report `robust_under_annotation_perturbation: bool`. Combinatoric explosion is bounded by candidate count, not precinct count, so this is cheap.
- [ ] **3.6** L0 test: synthetic dataset with three baselines that all yield significant race coefficient → assert `robustness_check.race_coefficient_significant_under_all == true`. Synthetic dataset where only one baseline is significant → assert false.

**Exit:** Orchestration glues Tasks 1+2; the Holm family explicitly includes LOO variants when supplied; one L0 test for each robustness branch.

---

## Task 4: Race-of-candidate file format, parser, and provenance struct (BD-R2)

**Files:** `redist/crates/redist-analysis/src/bloc_voting.rs`, `docs/file-formats/race-of-candidate.md` (new)

- [ ] **4.1** Implement CSV parser for the v2 schema: required columns `candidate_name, party, race, curator, curator_credentials, curator_attestation_date, source, independently_verified` plus the v2.1 BD-R2 additions: `attestation_doc_path` (relative path to the attestation document inside the reproducibility zip), `attestation_doc_format` (one of `pdf|docx|md|txt`), `attestation_doc_sha256` (computed at import; fail if file missing).
- [ ] **4.2** Schema validation: `race ∈ {Black, white, Hispanic, Asian, Native, other}` case-sensitive — any other value is a hard `[INPUT]` error per `docs/error-conventions.md` conventions. `curator` and `curator_attestation_date` non-empty. `independently_verified` parses to bool.
- [ ] **4.3** When `independently_verified == false`, the `BlocVotingResult` injection logic (Task 6) must:
  (a) set `race_of_candidate_provenance.annotations_independently_verified = false`
  (b) prepend the caveat string `"[CAVEAT — annotations not independently verified]"` to `draft_interpretation`

  This is B-02 anchor 4. Test must assert both fields after a non-verified import.
- [ ] **4.4** Build `RaceOfCandidateProvenance` struct: `source_file` (relative path inside the reproducibility zip), `source_sha256` (SHA-256 of the CSV content, computed at run time per spec line 113), `curator`, `curator_attestation_date`, `annotations_independently_verified`, `schema_version: 1`, plus BD-R2: `attestation_doc_path`, `attestation_doc_format`, `attestation_doc_sha256`.
- [ ] **4.5** Multi-curator dispute support: when multiple rows exist for the same `(candidate_name, party)`, the parser does not error — it returns `Vec<AnnotationSet>` keyed by curator. The orchestrator (Task 3) iterates and emits one analysis per curator; the JSON output is an array under `parallel_curator_runs`.
- [ ] **4.6** Write `docs/file-formats/race-of-candidate.md` documenting:
  - The schema (column-by-column)
  - Allowed `race` values and why the list is closed
  - `attestation_doc` accepted formats: `pdf`, `docx`, `md`, `txt` (no HTML — too easy to fake; no images — not extractable text). Doc must contain at minimum the curator name, credential, attestation date, and a per-candidate sentence justifying the classification.
  - The reproducibility-zip inclusion contract (Task 5)
  - Worked example for the LA 2020 Democratic presidential primary
- [ ] **4.7** L0 tests: well-formed CSV parses; bad `race` value errors with `[INPUT]` prefix; missing `attestation_doc_path` errors; computed `attestation_doc_sha256` matches `sha256sum` on the same fixture file; multi-curator file produces 2-element vec.

**Exit:** Format spec doc lints clean; parser tests cover all four error branches and the multi-curator path; BD-R2 fields populated.

---

## Task 5: Reproducibility-zip inclusion contract for the race-of-candidate bundle

**Files:** `redist/crates/redist-cli/src/bloc_voting_cmd.rs` (new), `redist/crates/redist-cli/src/report_cmd.rs` (small edit at the bundling site)

- [ ] **5.1** When the bloc-voting analyzer runs, copy two artifacts into the plan's `analysis/bloc_voting/` directory:
  - The race-of-candidate CSV at the path it was read from (preserving filename, recording `source_sha256`)
  - The attestation document(s) — one per row's `attestation_doc_path` — into `analysis/bloc_voting/attestations/{candidate_slug}.{ext}`

  Both copies are verified post-write: re-read, recompute SHA-256, assert match.
- [ ] **5.2** Edit `report_cmd.rs` so that when assembling the reproducibility zip (Court Submission Report path), the entire `analysis/bloc_voting/` subtree is included. Without this, `independently_verified=false` is irrelevant — *no* artifact is verifiable absent the documents. This is the BD-R2 / BOUNDARY-blocker line: "must be included in the reproducibility package emitted by `redist report`."
- [ ] **5.3** Add an L0 test in `report_cmd.rs` that builds a fake plan with a `bloc_voting/` artifact tree and asserts the assembled zip contains every file under it.

**Exit:** A `redist report --label la_2020 --format zip` output, when unzipped, contains the race-of-candidate CSV and attestation docs at predictable paths; SHA-256s in the manifest match the bytes on disk.

---

## Task 6: Output JSON schema, narrative summary, and `bloc_voting.json` writer

**Files:** `redist/crates/redist-analysis/src/bloc_voting.rs`, `redist/crates/redist-cli/src/bloc_voting_cmd.rs`, `redist/crates/redist-analysis/schemas/bloc_voting.schema.json` (new)

- [ ] **6.1** Implement `serde::Serialize` on the result struct producing exactly the spec §"`bloc_voting.json` schema" shape, including:
  - top-level `analyzer: "bloc-voting"`, `available`, `state`, `year`, `election`, `party`, `method`
  - `ecology` block with `n_precincts`, `candidates_analyzed`, `race_of_candidate` map
  - `regression` block with `model`, `specification` (the literal string `"WLS weighted by precinct vote count; HC3 robust SE; cluster-bootstrap by county B=10000"`), `candidate`, `diagnostics` (the three flags), `coefficients` (intercept + per-predictor with `estimate`, `stderr_hc3`, `ci_95_cluster`, `standardized_beta`), `n_precincts`, `n_clusters`, `r_squared`, `p_values` (raw + Holm for each predictor), `robustness_check`, `ecology_caveat` (verbatim from spec line 110), `draft_interpretation`
  - `race_of_candidate_provenance` block with the BD-R2 additions
  - `provenance` block matching other analyzers (build commit, rustc, args)
- [ ] **6.2** Write the JSON Schema file at `redist-analysis/schemas/bloc_voting.schema.json` that the writer's output must validate against. Include the schema path in `provenance.schema_path`.
- [ ] **6.3** Implement `write_bloc_voting_summary(result, path)` producing `bloc_voting_summary.md` with:
  - Plain-English `[DRAFT — expert witness should rewrite]` block (verbatim spec line 110 template, with values filled in)
  - Diagnostics table (VIF, cluster-vs-naive divergence, n_clusters)
  - Robustness table (4 rows: primary + 3 baselines)
  - The fixed `ecology_caveat`
  - The fixed annotation-verification caveat when `independently_verified=false`
- [ ] **6.4** L0 test: round-trip — fit on synthetic data, serialize, validate against the JSON schema, parse back into the struct. Asserts schema and writer agree.

**Exit:** Both the JSON and the Markdown summary are produced for a known synthetic input; JSON validates against the committed schema.

---

## Task 7: CLI surface — `redist analyze --types bloc-voting` + flags

**Files:** `redist/crates/redist-analysis/src/analyzer.rs`, `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/analyze.rs`, `redist/crates/redist-cli/src/bloc_voting_cmd.rs`

- [ ] **7.1** Add `BlocVoting` to `AnalyzerType` enum (do **not** add to `all_concrete()` — like `Compactness` and `Partisan`, this is opt-in via `--types bloc-voting`). Wire `name() => "bloc-voting"`.
- [ ] **7.2** Extend `AnalyzeArgs` in `args.rs` with the spec's CLI surface:
  - `--party <DEM|REP>` (default `DEM`)
  - `--election <NAME>` (default `presidential-primary`)
  - `--candidate-race-csv <PATH>` (the spec's `--race-of-candidate`; rename to `--candidate-race-csv` for unambiguous reading at trial — short doc note on this rename in the help)
  - `--partisan-baseline <PATH>` (CSV with prior-cycle baseline)
  - `--bootstrap-samples <N>` (default `10000`)
  - `--method <ols|rxc>` (default `ols`; `rxc` returns `not-yet-implemented` per spec — Method 2 is research mode)
  - `--minority-group <black|hispanic|asian>` (default `black`)
  - `--alpha <FLOAT>` (default `0.05`)
  - `--annotation-sensitivity <FLOAT>` (default `0.10`)
  - `--ci-level <FLOAT>` (default `0.95`)
  - `--min-precincts <N>` (default `50`; refuses with clear `[INPUT]` error below this count per spec risk row 4)
- [ ] **7.3** In `analyze.rs::run_analyze`, add the `AnalyzerType::BlocVoting` match arm. Delegates to `bloc_voting_cmd::run_bloc_voting_cmd(...)` which: loads precincts (via OpenElections per `sources.json` registry — `fetcher: null` rows surface a `[INPUT]` error directing the user to the registry doc), loads tract demographics (existing `data/{year}/demographics/{state}_demographics_{year}.csv`), loads partisan baseline (reuse `partisan_shares.rs` for the TSV path), loads the race-of-candidate CSV, computes precinct-level `pct_minority` via tract→precinct overlay (existing pipeline), runs the orchestrator, writes JSON + summary, runs Task 5's reproducibility-zip staging.
- [ ] **7.4** Help text (`redist analyze --help`) lists `bloc-voting` with a one-line description and a `See: docs/file-formats/race-of-candidate.md` pointer.
- [ ] **7.5** Help text states explicitly that `--types bloc-voting --stdout` is permitted (single-type rule; consistent with `analyze.rs` line 228 invariant).
- [ ] **7.6** L1 integration test in `redist-cli/src/integration_pipeline_tests.rs`: end-to-end on a 100-precinct synthetic fixture under `tests/fixtures/bloc_voting_synthetic/`, asserting JSON exit 0 and B-02 anchor 1 (coefficient within ±0.02 of ground truth).

**Exit:** `redist analyze --label la_2020 --types bloc-voting --candidate-race-csv data/elections/race_of_candidate/2020-presidential-primary.csv --party DEM` runs to completion on the synthetic L1 fixture.

---

## Task 8: Tests — explicitly named L0 anchors, L1, L2

**Files:** `redist/crates/redist-analysis/src/bloc_voting.rs` (`#[cfg(test)] mod tests`), `redist/crates/redist-cli/tests/bloc_voting_l1.rs` (new), `tests/acceptance/test_bloc_voting_louisiana.py` (new)

The four B-02 anchors must be present as named tests; they are the SCALE-block-lifting receipts.

- [ ] **8.1** `test_b02_anchor1_ols_coefficient_within_002()` — synthetic ground truth where `pct_black` coefficient is exactly 0.61. Fit recovers within ±0.02 (test asserts `(estimate - 0.61).abs() <= 0.02`).
- [ ] **8.2** `test_b02_anchor2_holm_dominates_raw_on_30test_family()` — construct a 30-test family with raw p-values spanning 0.0001..0.5. Assert: (a) Holm-corrected p ≥ raw p elementwise; (b) at least one test that was raw-significant (p < 0.05) is no longer significant after Holm; (c) the smallest raw p remains significant. This proves Holm "dominates" raw in the spec's sense.
- [ ] **8.3** `test_b02_anchor3_vif_above_5_sets_underpowered_flag()` — synthetic predictors with correlation 0.95 → VIF > 5 → `vif_underpowered_flag == true`. Counterpart: correlation 0.10 → flag false.
- [ ] **8.4** `test_b02_anchor4_independently_verified_false_injects_caveat()` — load a race-of-candidate CSV with `independently_verified=false`, run the pipeline, assert (a) `race_of_candidate_provenance.annotations_independently_verified == false`; (b) `draft_interpretation.starts_with("[CAVEAT — annotations not independently verified]")`.
- [ ] **8.5** L1 (already in Task 7.6): synthetic 100-precinct end-to-end → JSON validates against schema → `redist doctor` would not flag.
- [ ] **8.6** L2 acceptance test `tests/acceptance/test_bloc_voting_louisiana.py`, marked `@pytest.mark.network @pytest.mark.slow` — clones OpenElections-LA, runs the 2020 Democratic presidential primary analysis, asserts: exit code 0; n_precincts > 3000; `regression.specification` contains the literal "cluster-bootstrap by county"; reproducibility zip contains the race-of-candidate CSV. Wired into nightly CI per the roadmap CI strategy.
- [ ] **8.7** Cluster-bootstrap correctness test from Task 2.5 also lives here under `test_cluster_bootstrap_widens_ci_under_within_county_correlation`.

**Exit:** `cargo test -p redist-analysis bloc_voting` and `cargo test -p redist-cli bloc_voting_l1` green; nightly CI green on the L2 test.

---

## Task 9: Documentation — REDIST_CLI, race-of-candidate format, Louisiana walkthrough hook

**Files:** `docs/REDIST_CLI.md`, `docs/file-formats/race-of-candidate.md` (Task 4.6), `examples/louisiana-callais-walkthrough/` (consumed by Onboarding plan; this task lands the bloc-voting step inside it), `docs/CHANGELOG.md`, `CLAUDE.md`

- [ ] **9.1** `docs/REDIST_CLI.md` — new section "Within-Party Bloc Voting (Callais Evidence)" documenting:
  - The full CLI surface from Task 7.2
  - The disentanglement model in plain English, including why Holm is computed on the *combined* family per S-02
  - The `ecology_caveat` (verbatim) and the expert-witness liability disclaimer (output is `draft_interpretation`, not finished testimony)
  - Pointer to `docs/file-formats/race-of-candidate.md` and `docs/legal/CALLAIS_REFERENCE.md`
- [ ] **9.2** Add the bloc-voting step to `examples/louisiana-callais-walkthrough/run.sh` (created by Onboarding plan Task 5.3 / DoD). Two-line addition: after `redist analyze --types all`, run `redist analyze --types bloc-voting --candidate-race-csv ...`. Update the walkthrough's `checksums.json` to include `analysis/bloc_voting.json` SHA. Note: the bloc-voting JSON contains build-commit-sensitive provenance; pin via `REDIST_BUILD_COMMIT` (B-07 lives in Deposition Prep plan but this walkthrough must work with it).
- [ ] **9.3** `docs/CHANGELOG.md` entry citing Callais (608 U.S. ___, 2026-04-29) and v2.1 items addressed.
- [ ] **9.4** `CLAUDE.md` "Recent Changes" entry; "Common Commands" gets one canonical bloc-voting invocation example.

**Exit:** Markdown-lint clean; the LA walkthrough's L2 acceptance run includes the bloc-voting step and its checksum is pinned.

---

## Definition of Done

- `cargo test -p redist-analysis bloc_voting` green; all four B-02 L0 anchors named exactly per Task 8 (`test_b02_anchor{1,2,3,4}_*`)
- `redist analyze --types bloc-voting --candidate-race-csv <PATH>` runs end-to-end on the LA 2020 fixture in nightly CI
- `bloc_voting.json` validates against `schemas/bloc_voting.schema.json` and contains: `regression.specification` matching the spec's exact string; `regression.diagnostics` populated (VIF + flag + ci-divergence flag); `regression.p_values` containing both raw and Holm-corrected entries for every predictor
- `bloc_voting_summary.md` reads as defensible §2 testimony, prefixed with the `[DRAFT — expert witness should rewrite]` marker, and includes the `ecology_caveat` verbatim
- Race-of-candidate CSV + every attestation document is included in the `redist report --format zip` output, with SHA-256s in the manifest matching bytes on disk (BD-R2)
- Holm family for Callais provably includes leave-one-out sensitivity-variant runs from Civic Bidirectional conflict resolution (S-02), documented in code comments at the family-construction site
- `docs/file-formats/race-of-candidate.md` exists, lists `attestation_doc_format` accepted values (`pdf|docx|md|txt`), and links from REDIST_CLI.md
- LA walkthrough fixture runs the bloc-voting step and pins its output SHA in `checksums.json`
- `independently_verified=false` injects the documented caveat into `draft_interpretation` (B-02 anchor 4) — verified by L0 test
- VIF > 5 sets `vif_underpowered_flag` (B-02 anchor 3) — verified by L0 test
- Holm dominates raw on the canonical 30-test family (B-02 anchor 2) — verified by L0 test
- OLS coefficient on synthetic ground truth recovers within ±0.02 (B-02 anchor 1) — verified by L0 test

---

## Risks

| Risk | Mitigation |
|---|---|
| WLS+HC3 from-scratch implementation has numerical edge cases (rank-deficient X, near-zero weights) | Center predictors before solve; refuse if any weight ≤ 0; refuse if condition number > 1e8 with a `[INTERNAL]` error and a "predictors collinear; try fewer covariates" hint |
| Cluster bootstrap B=10000 is too slow on real LA-scale data | Performance gate in Task 2.4 surfaces a warning; if a real LA run breaches the budget, lower the L1 default to `--bootstrap-samples 2000` and document the trade-off in REDIST_CLI.md |
| Holm family size explodes when LOO variants are large (e.g., 8 curators × 5 candidates × 4 baselines = 160) inflating corrections to the point that nothing is significant | This is a *feature*, not a bug, of the SCALE-block-lifting bargain — but document it in REDIST_CLI.md so the expert witness understands when to defend a smaller curator pool |
| Race-of-candidate annotation files leak personal opinion into court submissions | Schema-validated; `attestation_doc` requires curator credential text; multi-curator dispute path produces parallel runs — the expert chooses which to defend, the JSON surfaces both |
| `independently_verified=false` caveat is ignored by downstream consumers | Caveat is prepended to the `draft_interpretation` string itself, not just a side flag — it is impossible to render the interpretation without seeing it |
| OpenElections per-state schema variance breaks the precinct loader | Per-state schema mapping is the consumer's responsibility (per `sources.json` notes); we ship a `--schema-mapping <PATH>` escape hatch and document the LA mapping inline in the walkthrough; OpenElections fetcher is not in scope for this plan |
| `fetcher: null` rows in `sources.json` surface unhelpfully when a user picks a state with no automated fetcher | `[INPUT]` error names the registry path and the manual-fetch alternative; D-03 (`schema_version` + content-hash on `sources.json`) is owned by the Court Reports plan, not this one |
| RxC method (Method 2) is more rigorous than WLS+HC3 in some literatures and a savvy opposing expert raises it | Spec defers RxC to research mode; `--method rxc` returns `not-yet-implemented` with a pointer to `docs/legal/CALLAIS_REFERENCE.md` documenting the deferral; do not pretend to ship it |
| Attestation-doc PDF could embed JavaScript or active content | Whitelist `pdf|docx|md|txt`; SHA-256 is computed on the *file bytes* not extracted text — tamper-evident regardless of content; we do not parse the PDF, only hash and bundle it |

---

## Out of Scope

- Method 2 (RxC ecological inference) implementation — spec defers to future research mode; CLI accepts `--method rxc` but returns `not-yet-implemented`
- Multi-cycle robustness across more than the four baselines specified (primary + 3 alternates) — Callais requires "current data" without bounding cycle count; we ship four
- Causal inference — explicitly associational per spec §"Out of scope"
- Survey-based race attribution / voter-file integration (Catalist, L2) — separate spec, not addressed here
- Auto-fetching OpenElections data inside the bloc-voting command — fetcher is `redist fetch --source openelections`, run separately
- Fixing `fetcher: null` rows in `sources.json` — registry hygiene is owned by D-03 in the Court Reports plan
- The `--label` rename to `--plan-label` (M-01) — owned by Plan Comparison + Deposition plans; this plan continues to use `--label` consistent with other analyze subcommands until that rename lands
- Consuming Civic Bidirectional conflict-resolution outputs directly — this plan accepts LOO variants as an *input* (via repeated `--candidate-race-csv` invocations or a multi-curator CSV per Task 4.5); the upstream conflict-resolution pipeline that produces them is owned by the Civic Bidirectional plan
- Daubert pre-flight — separate concern, owned by the Deposition Prep + Court Reports plans
