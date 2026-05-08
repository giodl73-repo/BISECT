# F-Track Panel Review — Batch 1
**Date**: 2026-05-07
**Reviewer Panel**: R1 Karypis (algorithms), R2 Rodden (political science), R3 Duchin (math/redistricting), R4 Stephanopoulos (law), R5 Liang (ML/reproducibility)
**Scale**: 0–4 per reviewer; Average ≥ 3.0 = Accept, ≥ 2.5 = Minor Revision, ≥ 2.0 = Major Revision, < 2.0 = Reject

---

## F.0 — State Legislative Overview

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 3.5 | Resolution selection rule (k/n > 0.05) is well-motivated; the k/n_bg = 0.57 for New Hampshire is correctly flagged as borderline. Runtime table is specific and testable. The O(n²·log n) degradation claim at k/n → 1 is stated without proof; would benefit from a citation or brief derivation. |
| R2 Rodden | 3.0 | Overview correctly frames state legislative redistricting as politically more consequential than congressional; 23-state Republican trifecta claim (2020) is accurate by public records. Preview figures ($\overline{PP}$ = 0.401 vs. 0.361) are stated correctly as forward references. |
| R3 Duchin | 3.0 | The resolution selection rule derivation is principled and the 20-units-per-district heuristic is well-stated. The footnote about New Hampshire requiring block resolution but using block-group as a compromise is appropriately qualified. The nesting success rate preview (40/49) is internally consistent with F.2's detailed findings. |
| R4 Stephanopoulos | 3.5 | *Wesberry v. Sanders* and *Reynolds v. Sims* correctly distinguished (±0 for congressional, ±5% for state legislative). *Karcher v. Daggett* not mentioned but is appropriate context for the congressional standard; not a blocker. All cited cases accurate. |
| R5 Liang | 3.0 | Overview paper correctly limits itself to framing and forward references rather than presenting unreproduced data. Runtime table is concrete and testable. Storage estimates (4 GB block-group, 40 GB block) are specific. Single seed noted; 5-seed sensitivity described as case-study scope only. |

**Average: 3.2 → Accept**

### Priority Issues

**P1 — Blockers**
- None.

**P2 — Should Fix**
- P2-1: The O(n²·log n) runtime claim for the degenerate k/n → 1 regime is stated in Section 6 without citation or derivation. Add either a reference to a METIS complexity analysis or a brief explanation.
- P2-2: Table 1 column header "$k/n$" is ambiguous without clarifying the denominator is tracts specifically; the body text distinguishes tract vs. block-group ratios but the table header does not.

**P3 — Optional**
- P3-1: The `bisect` CLI commands throughout use correct binary name. No issues.
- P3-2: The 40/49 bicameral nesting success rate previewed here—confirm this matches F.2's final count of 40 compatible states once F.2 is finalised.

---

## F.1 — Single-Chamber All 50 States

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 3.5 | The 50-state sweep is a substantive algorithmic contribution. Resolution selection rule correctly applied (12 block-group states, 38 tract states). Population balance within ±0.5% for all 50 states is independently verifiable. Runtime measurements are specific (8s to 420s range; median 67s). The New Hampshire case (k/n_bg = 0.57) is correctly identified as sub-optimal and flagged as a limitation. |
| R2 Rodden | 3.0 | The comparison of enacted vs. algorithmic maps on county splits is credible and important. The conclusion that gerrymandered states (PA, WI, OH, NC) show 30–41% county-split reductions while commission states show 5–12% is a plausible and testable hypothesis. The causal claim ("reflects partisan cracking") should be stated as an interpretation rather than a finding. |
| R3 Duchin | 3.5 | Strong paper. Single-seed limitation is properly documented and the 5-seed sensitivity check for 4 case studies (identical partisan outcomes, PP within ±0.005) is appropriate. The limitations section is honest and complete: block-group ceiling for NH/WY/VT/SD/ND, no VRA assessment, 12 missing enacted maps. |
| R4 Stephanopoulos | 3.0 | *Reynolds v. Sims* standard (±5%) vs. *Wesberry* (±0.5%) correctly distinguished and applied. The claim that algorithmic maps achieve *Wesberry* precision for state legislative plans is accurately qualified as "tighter than legally required." No invented cases. |
| R5 Liang | 3.0 | All 50 states use seed 42 as primary. Five-seed sensitivity for 4 case studies reported. The block-group prerequisite script is documented. Population balance figures are specific and verifiable from census data. The statistical tests for county splits (t=4.2, p<0.001) and population balance (t=3.8, p<0.001) are appropriate for the comparison. No variance reported on the $\overline{PP}$ = 0.381 (tract) and 0.401 (BG) headline figures. |

**Average: 3.2 → Accept**

### Priority Issues

**P1 — Blockers**
- None.

**P2 — Should Fix**
- P2-1: The causal claim in Section 6 ("consistent with the hypothesis that enacted maps in gerrymander-prone states deliberately crack communities") should be framed as an interpretation, not a finding. Add language such as "our results are consistent with" or "a plausible interpretation is."
- P2-2: No standard deviation reported for $\overline{PP}$ = 0.381 (tract) or 0.401 (block-group) in the main results table. Add SD or 95% CI to permit the reader to assess the precision of these estimates.

**P3 — Optional**
- P3-1: The conclusion paragraph "bisect state --chamber house --resolution block_group" command is a clean contribution to replication. Consider adding the full prerequisite build command as well.
- P3-2: Table 3 (runtime) is in the results section but runtime results appear again in a descriptive paragraph; minor redundancy.

---

## F.2 — Bicameral Nesting

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 3.5 | NestSection algorithm is well-described and the spine construction is mathematically rigorous. The proposition (spine triviality when gcd=1) is correctly stated. The compactness penalty measurement design (independent vs. NestSection, edge-cut ratio) is sound and reproducible. |
| R2 Rodden | 3.0 | The partisan effect analysis (Table 4) covers 30 of 40 compatible states "with available precinct data" but the other 10 are not addressed. The mean 0.3-seat shift is credible but the sample is incomplete. The Wisconsin 43D→42D change is isolated and noted correctly. |
| R3 Duchin | 3.5 | The gcd-based compatibility classification is a clean mathematical contribution. The senate PP < house PP observation (senate = house − 0.028 on average) is explained correctly as a consequence of merging. The 40-state success rate is clean and internally consistent with F.0's preview. |
| R4 Stephanopoulos | 3.5 | State constitutional examples (California Prop 11, Iowa statute) correctly described. The legal discussion of nesting requirements and court enforcement (California, Iowa) is accurate. The note on incompatible states (9 states) and the constitutional amendment required for each is appropriate. |
| R5 Liang | 3.0 | Spine construction is deterministic (no stochastic elements) for compatible states. The compactness penalty analysis compares two deterministic runs. The precinct data source (Kuriwaki 2023) is cited. 30 of 40 compatible states covered for partisan analysis; remaining 10 should be noted as a limitation more prominently. |

**Average: 3.3 → Accept**

### Priority Issues

**P1 — Blockers**
- None.

**P2 — Should Fix**
- P2-1: The partisan effect analysis covers only 30 of 40 compatible states ("with available precinct data"). The other 10 are not mentioned in the limitations. Add a limitations note and acknowledge that the 0.3-seat average may shift when all 40 states are included.
- P2-2: The senate compactness penalty scaling claim ("each additional house district per senate district adds approximately +0.8% edge-cut penalty") is stated without a regression or fitted model. Either fit a simple regression or qualify as approximate.

**P3 — Optional**
- P3-1: The multi-level nesting extension (county → state leg → congressional) described as "ongoing research" in the conclusion would benefit from a brief problem statement.
- P3-2: Table 2 (NestSection examples) has Pennsylvania listed as incompatible (gcd=1) in Table 3 but not in Table 2; Table 2 would benefit from including at least one incompatible state for contrast.

---

## F.3 — Multi-Resolution High-k Chambers

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 4.0 | This is the strongest methods paper in the F track. The resolution selection rule is rigorously derived from three independent considerations (balance feasibility, compactness optimisation richness, empirical calibration). Empirical validation on three states spanning the threshold is well-designed. The O(n log n) runtime analysis is correctly applied and the 3–4× factor is mechanistically justified. |
| R2 Rodden | 3.5 | The MAUP analysis (resolution → partisan outcomes) is a valuable and underexplored contribution. The finding that high-k/n chambers can shift partisan outcomes by ±2–4 seats due to resolution choice has direct policy implications that are clearly stated. |
| R3 Duchin | 3.5 | The Central Limit Theorem argument for why 20 units/district enables ±0.5% balance is sound. The compactness optimisation richness argument (O(n^m / k^m) configurations) is informal but appropriate for a heuristic derivation. The recommendation table for all 50 states is a major practical contribution. |
| R4 Stephanopoulos | 3.0 | Legal framing is minimal but appropriate: the MAUP discussion correctly frames resolution choice as a substantive decision with partisan implications, not merely technical. No legal citations are required in this methods paper. |
| R5 Liang | 3.5 | The three-state comparison design is well-controlled (same algorithm, same seed, varying resolution). The recommendation table is complete and reproducible from publicly available Census TIGER data. The prerequisite build command is documented. The r≈3.7–4.0× runtime ratio is confirmed across all three states, consistent with theoretical prediction. |

**Average: 3.5 → Accept**

### Priority Issues

**P1 — Blockers**
- None.

**P2 — Should Fix**
- P2-1: The compactness optimisation richness argument uses O(n^m / k^m) without defining what "configuration" means precisely. Add a sentence clarifying that this counts the number of distinct connected subgraphs of size m, and note this is an order-of-magnitude argument, not an exact count.
- P2-2: The MAUP results for Texas (+1 seat from block-group) and California (0 seats) use Kuriwaki (2023) precinct returns but the interpolation method is not described. Add a sentence on the interpolation approach.

**P3 — Optional**
- P3-1: The recommendation table uses "BL" for block resolution but the CLI flag is `--resolution block`; confirm the table notation matches the actual flag value.
- P3-2: Vermont (k/n = 0.806) is listed as "BL" (block resolution required) but F.1 uses block-group for Vermont as a compromise; make explicit that the table reflects the strict rule and F.1 the practical compromise.

---

## F.4 — State Criteria Variation

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 3.0 | The parameter mapping (legal criteria → YAML configuration) is well-designed and the five YAML examples are specific and testable. The claim that `coi_weights: census_places` satisfies community-of-interest requirements is asserted without validation data; this is appropriate for a framework paper but should be flagged. |
| R2 Rodden | 3.0 | The five-type taxonomy is a useful framework. The claim that Arizona's AIRC commission criterion "promotes competitive elections" is correctly described and the algorithmic comparison (mean margin 7.2 pp algorithmic vs. 9.4 pp enacted) is a concrete finding. Source for this comparison should be cited or a forward reference given. |
| R3 Duchin | 3.5 | The legal analysis is sophisticated and the mapping from legal criteria to algorithmic parameters is well-grounded. The observation that "structural partisan blindness" is a stronger claim than "procedural partisan neutrality" is a genuine contribution to the redistricting law literature. |
| R4 Stephanopoulos | 4.0 | Exceptional legal precision throughout. All constitutional citations are accurate: *Wesberry*, *Karcher v. Daggett*, *Thornburg v. Gingles*, *Shaw v. Reno*, *Reynolds v. Sims*, Florida Amendment 6 (Art. III §20), Arizona Prop 106, Michigan Proposal 2, Iowa's LSB, *League of Women Voters v. Commonwealth* (PA 2018). No invented cases. The distinction between mandatory and discretionary criteria is clearly and correctly drawn. |
| R5 Liang | 2.5 | The five YAML configurations are well-specified but not empirically validated. The Arizona competitive elections claim (7.2 vs. 9.4 pp margin) cites no source. The claim that Pennsylvania's court-ordered map achieved PP mean ~0.47 is given as a benchmark but is not sourced. The certify_partisan_blind and certify_incumbent_blind flags described in the Florida config appear to be proposed features, not implemented ones; this should be stated explicitly. |

**Average: 3.2 → Accept**

### Priority Issues

**P1 — Blockers**
- None.

**P2 — Should Fix**
- P2-1: The Arizona competitive elections comparison (7.2 pp vs. 9.4 pp) is stated in the legal analysis section without a source. Add either a citation to the companion paper where this is computed or a forward reference.
- P2-2: The `certify_partisan_blind` and `certify_incumbent_blind` output flags in the Florida config appear to be proposed CLI features, not currently implemented. Add a note clarifying the implementation status; do not present proposed features as existing ones.
- P2-3: The Pennsylvania PP ~0.47 benchmark attributed to the court-ordered LWV map should be sourced (either a companion paper result or a citation to the court record).

**P3 — Optional**
- P3-1: The `coi_weights: census_places` parameter is used in three YAML configs but what this means concretely (which Census designation, what weight modifier δ is applied) is not specified. A brief description would help implementers.
- P3-2: The classification of Vermont as Type I (permissive) with comment "1 at-large district" is correct; however, Vermont also has state constitutional language about compactness. The paper's scope note (congressional only) covers this, but a footnote acknowledging Vermont's state legislative criteria would prevent confusion.

---

## F.5 — Compactness: Legislative vs. Congressional

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 3.5 | The mathematical derivation of the O(1/√k) scaling law is rigorous given the stated regularity conditions. The hexagonal packing argument is a standard result. The proposition (eq. 3) is stated with appropriate regularity conditions and the empirical fit (c ≈ 0.120, predicted Δ = 0.030 vs. observed 0.027) is a genuine validation. |
| R2 Rodden | 3.5 | The three-census-year analysis (2000, 2010, 2020) demonstrating cross-census stability (0.026, 0.027, 0.028 PP advantage) is a strong robustness check. The state-level variation pattern (largest advantages in TX, PA, NH) is consistent with the theoretical prediction and well-explained. |
| R3 Duchin | 3.5 | The decomposition into scale effect and shape effect is the paper's central contribution and is well-executed. The distinction between "smaller districts are more compact" (PP/geometry) and "the algorithm performs better at smaller scale" (algorithmic claim) is maintained carefully throughout. The regularity conditions excluding fractal boundaries (Alaska, Florida coast) are appropriately noted. |
| R4 Stephanopoulos | 3.0 | This paper does not primarily engage with legal criteria and therefore does not require extensive legal analysis. The observation that Iowa's compactness statute is satisfied more readily at state legislative scale is accurate and legally relevant. No citations to legal cases are required here. |
| R5 Liang | 3.0 | Multi-census analysis (2000, 2010, 2020) is a strong reproducibility contribution. The full 50-state table (Table 1) with congressional and house PP at tract and block-group resolution is verifiable. VRA mode disabled for this comparison is correctly noted. The enacted map comparison covers 35 states; the other 15 are noted as unavailable. |

**Average: 3.3 → Accept**

### Priority Issues

**P1 — Blockers**
- None.

**P2 — Should Fix**
- P2-1: The proposition in Section 2 uses "minimum-edge-cut partition" which the algorithm produces, but the O(1/√k) scaling law is derived for uniform hexagonal tilings. The connection between the two—that METIS minimum-edge-cut approximates hexagonal tilings in expectation—is assumed but not demonstrated. Add a sentence acknowledging this gap.
- P2-2: The enacted map comparison (Section 4) uses "GIS data for 35 states" but the source is not cited. Specify the shapefile source (e.g., NCSL, state redistricting databases, Census Bureau).

**P3 — Optional**
- P3-1: Maryland consistently shows lowest absolute PP at both scales (0.302 congressional, 0.334 house). A brief case study on why Maryland is an outlier would strengthen the paper's geographic analysis.
- P3-2: The O(1/√k) relationship implies PP → PP_∞ as k → ∞. What is PP_∞ for METIS on US geographies? The paper fits c but not PP_∞ explicitly.

---

## F.6 — VRA Compliance at State Legislative Scale

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 3.0 | VRASection algorithm is described clearly at the procedural level. The three-phase (Identify, Aggregate, Verify) structure is implementable. The claim that "seed variation affects only METIS partitioning of non-minority-concentrated tracts" (Section 6) is asserted but not proven; this is the mechanism by which seed stability is explained. |
| R2 Rodden | 3.0 | The scale advantage argument (state house provides more MM district opportunities than congressional for same minority population) is well-motivated and empirically supported. The South Carolina finding (0 congressional MM districts, 28 state house MM districts) is the headline result and is mechanistically explained. |
| R3 Duchin | 3.5 | The Callais disentanglement methodology is correctly applied and the regression results (Table 3) are clean and theoretically predicted. The distinction between mandatory minority population threshold (42%) and the scale-invariance claim is well-grounded. The finding that MM district compactness penalty is smaller at state house scale (5.7% vs. 10.3%) is a genuine and important result. |
| R4 Stephanopoulos | 3.5 | *Thornburg v. Gingles* three-precondition framework correctly applied. *Shaw v. Reno* and *Louisiana v. Callais* correctly described. *Allen v. Milligan* implicitly present (the five covered states are the *Allen*/*Callais* states). *Miller v. Johnson* cited in the introduction correctly. No invented cases. The categorical satisfaction of Callais disentanglement argument for algorithmic maps is legally sophisticated and correct. |
| R5 Liang | 2.5 | The headline finding (5/5 covered states at state house scale) is reported as seed 42 only. Section 6 explicitly labels the seed sensitivity as "Phase 2 note" with results pending. The abstract and conclusion treat the 5/5 result as established, but the paper explicitly acknowledges it is a single-seed observation. The Callais regression (Table 3) has no reported R² or model fit statistic. |

**Average: 3.1 → Accept (Minor Revision Recommended)**

### Priority Issues

**P1 — Blockers**
- None.

**P2 — Should Fix**
- P2-1: The abstract and conclusion state "VRASection achieves majority-minority districts in all 5 covered states at state house scale" without qualification. Section 6 explicitly notes these are single-seed results pending 5-seed validation. The abstract and conclusion must include this qualification until Phase 2 is complete (e.g., "in single-seed analysis; multi-seed validation pending").
- P2-2: Table 3 (Callais regression) reports coefficient estimates and HC3 standard errors but no model-level fit statistic (R², pseudo-R², or observation count). Add N (number of tract-pair observations) and an overall model fit measure.

**P3 — Optional**
- P3-1: The "Phase 2 note" framing for seed sensitivity (Section 6) is unusual for a paper submitted for panel review. Either complete the Phase 2 runs before submitting for review, or move the entire seed sensitivity discussion to a Future Work section.
- P3-2: The Georgia result (55 state house MM districts) is the largest in Table 2 but receives no case study treatment. A brief discussion of the Georgia geography would illustrate the scale advantage argument concretely.

---

## Batch Summary

| Paper | R1 | R2 | R3 | R4 | R5 | Avg | Verdict | P1 Count | Top P1 Issue |
|-------|----|----|----|----|-----|-----|---------|----------|--------------|
| F.0 Overview | 3.5 | 3.0 | 3.0 | 3.5 | 3.0 | **3.2** | Accept | 0 | — |
| F.1 Single-Chamber 50 | 3.5 | 3.0 | 3.5 | 3.0 | 3.0 | **3.2** | Accept | 0 | — |
| F.2 Bicameral Nesting | 3.5 | 3.0 | 3.5 | 3.5 | 3.0 | **3.3** | Accept | 0 | — |
| F.3 Multi-Resolution | 4.0 | 3.5 | 3.5 | 3.0 | 3.5 | **3.5** | Accept | 0 | — |
| F.4 Criteria Variation | 3.0 | 3.0 | 3.5 | 4.0 | 2.5 | **3.2** | Accept | 0 | — |
| F.5 Compactness Scale | 3.5 | 3.5 | 3.5 | 3.0 | 3.0 | **3.3** | Accept | 0 | — |
| F.6 VRA State Leg | 3.0 | 3.0 | 3.5 | 3.5 | 2.5 | **3.1** | Accept | 0 | — |

### Track-Level Notes
- The F track is the most coherent track in the portfolio. All seven papers accept. F.3 is the strongest (3.5 avg); it makes a clean methods contribution that will survive independent replication.
- F.6 has no P1 blockers but has a meaningful P2-1: the abstract and conclusion overclaim single-seed results as established findings. This is a minor but important correction before submission.
- F.4 is the track's legal standout (R4 = 4.0), with accurate and sophisticated legal analysis across all 50 states. R5's concern about unvalidated YAML configurations is legitimate but addressable without major revision.
- The cross-track dependency structure is healthy: F.1 correctly depends on F.3 for the resolution rule; F.2 depends on B.13 for NestSection; F.6 depends on D.1 for the 42% threshold. All cross-references are correctly attributed.
- No `redist` binary name errors found in F-track papers (F.6 correctly uses `bisect fetch`). F.0 through F.6 consistently use `bisect`.
