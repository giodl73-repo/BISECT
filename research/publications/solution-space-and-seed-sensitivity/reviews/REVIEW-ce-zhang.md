# Review: R-30 Ce Zhang
**Paper**: The Solution Space of Minimum-Edge-Cut Redistricting: Seed Sensitivity and Partisan Variance
**Date**: 2026-05-01
**Score**: 3.0 / 4

---

## Summary

The paper's data infrastructure is generally solid — TIGER files with SHA-256 manifests, a proportionality metric with explicit data-quality flags, and a bug fix for the national vote-share contamination. But several data quality issues affect the main results, and the election data provenance is insufficiently documented for a paper making legal claims.

---

## Strong Points

1. **The proportionality bug fix (national vs. state vote share)** is exactly the kind of data-centric correction that changes results. The paper correctly identifies that the national election CSV caused every state to show the same Dem vote share (52.2%). The fix — filter to assigned tracts only — is correct and the before/after comparison (VT: 52.2% → 68.3%) is convincing. Documenting this fix is valuable.

2. **The available flag for missing data (AK, HI)** is correct practice. Rather than silently outputting zero-vote plans for these states, the analyzer marks them unavailable. The method section should describe this flag and its implications for the 50-state analysis.

3. **The TIGER data pipeline** (adj.bin with SHA-256 manifests, version-controlled input files) is appropriately documented. The reproducibility artifacts satisfy the basic requirements for a legally-relevant paper.

---

## Concerns and Weaknesses

### P1: Critical

**P1.1 — The election data source and quality are undocumented.**

The proportionality metric uses `data/2020/elections/presidential_by_tract.csv` as the vote source. The paper never states: where this data comes from, who produced it, what methodology was used to apportion precinct-level votes to census tracts, or what its accuracy is. This is not a minor documentation gap — it directly affects the core metric.

Presidential vote-to-tract apportionment is non-trivial. Precincts and census tracts have different boundaries. The standard methods (population-weighted interpolation, geographic overlay) produce different results and can have errors of 5-15% for individual tracts in boundary zones. For a paper making claims like "WI has 50.3% Dem statewide vote share" that are used to compute the proportionality gap (which the paper uses to evaluate CompactBisect), the election data source must be cited and its quality characterized.

What is the source? VEST (Voting and Election Science Team)? MGGG? DistrictBuilder? Fekrazad (the paper mentions DOI 10.7910/DVN/Z8TSH3 in the onboarding plan but not in this paper)? State that it is the Fekrazad dataset, cite it, and report the validation statistics (R² with precinct-level reported totals, tract-level error distribution).

**P1.2 — The circular approximation for tract external perimeters is not validated.**

The CompactBisect implementation approximates each tract's external perimeter as 2√(πA) − Σ(shared boundary lengths). The circular approximation can be significantly wrong for elongated or irregular tracts. For example, a tract with shape ratio 4:1 has perimeter 2(4a + a) = 10a but circular approximation gives 2√(π·4a²) ≈ 7.1a — an underestimate of 30%. For coastal tracts with complex boundaries, the approximation error can exceed 100%.

This matters because: (a) PP is computed from the approximated external perimeters, (b) the Fiedler certificate's GMPP upper bound uses these perimeters, and (c) the CompactBisect selection at each level uses these PPs to choose bisections. An inaccurate approximation means CompactBisect is optimizing a corrupted objective.

Recommendation: compute exact external perimeters from TIGER shapefile polygon coordinates (exact coordinate-based perimeters are available in the adj.bin build pipeline). If approximate perimeters must be used, validate them against exact values for a sample of tracts and report the error distribution.

### P2: Significant

**P2.1 — Data sensitivity for the election CSV is not analyzed.**

The paper makes empirical claims that depend entirely on `presidential_by_tract.csv`. But the quality of this file varies across states. Some states have detailed precinct-level data enabling accurate tract-level apportionment; others have county-level data that forces population-weighted imputation with large errors.

The paper should either: (a) report per-state data quality indicators (e.g., fraction of tracts where vote data was imputed from county vs. precinct level) and condition the analysis accordingly, or (b) use a uniform high-quality data source for all states and document it explicitly.

For instance, if the election data for deep-red states (AR, OK, MS) was imputed from county-level results, the reported Dem vote shares may be systematically biased, and the proportionality gap analysis for those states is unreliable.

**P2.2 — The Hawaii and Alaska data gaps should be quantified.**

HI and AK are flagged as `available=false` due to missing election data. But AK is a single-at-large district and HI has 2 districts — their inclusion/exclusion matters for the national totals (228D/204R). The paper reports the 48-state total but should also report what the full 50-state total would be under the most plausible assumption for AK (R, given Don Young won in 2020) and HI (D, both seats reliably Dem). This would give a full 50-state estimate rather than the 48-state approximation.

**P2.3 — The race condition in the multi-state sweep is not documented.**

The background sweep (scripts/b7_sweep.ps1) runs 50 states × 200 seeds sequentially with output buffering. The sweep was interrupted and restarted multiple times during data collection. The paper should document the data collection protocol more precisely: which seeds were run for which states, whether any states were re-run, and whether the convergence data is from a clean sequential run or multiple partial runs.

### P3: Minor

**P3.1 —** The WGS84 coordinate system is used for PP computation (as noted in `crs_note` in the output files). For states with significant east-west extent (ND, OR, WA), PP in WGS84 is compressed by approximately cos²(latitude) ≈ 0.8 relative to equal-area projection. This systematic understatement of PP doesn't affect relative comparisons within a state, but affects the absolute PP values reported in Table 4.2 and would affect cross-state PP comparisons.

**P3.2 —** The `final_assignments.json` format stores GEOID → district mappings. The paper should state that this is the canonical output format and define the schema (11-char GEOID, 1-based district numbering) so that third parties can independently verify the proportionality computations.

---

## Verdict

The data infrastructure is better than average for a legal/political science paper. The election data provenance gap (P1.1) is the most serious issue — without knowing the source and quality of the vote data, the proportionality metric's claims are ungrounded. Fix this (add citation + validation stats), switch to exact TIGER perimeters (P1.2), and document the data collection protocol (P2.3). I would accept after these revisions.
