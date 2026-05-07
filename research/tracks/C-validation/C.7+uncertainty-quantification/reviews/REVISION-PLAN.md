# Revision Plan — C.7 Statistical Uncertainty in Algorithmic Redistricting
Round 1 avg: 3.0/4

## Score Summary

| Reviewer | Score | Verdict |
|----------|-------|---------|
| Karypis  | 3/4   | Minor Revision |
| Rodden   | 3/4   | Minor Revision |
| Duchin   | 3/4   | Minor Revision |
| Stephanopoulos | 3/4 | Minor Revision |
| Liang    | 3/4   | Minor Revision |
| **Average** | **3.0/4** | **Minor Revision** |

**Gate Status**: PASSED (avg 3.0 >= 2.5, no score < 2.0)

---

## P1 — Required Fixes

### P1.1 — Fix abstract/body CI inconsistency for the +22% claim (Karypis, Liang)
**Issue**: Abstract states the 95% CI is "[+18%, +26%]" while the body's Table 7.4 reports "[+15%, +29%]" for all three sources. The abstract CI corresponds to seed+resolution only, not all three.
**Fix**: Correct the abstract to cite [+15%, +29%] (all three sources), or revise to "[+18%, +26%] (accounting for seed variance and resolution uncertainty; the census undercount component is small and conservative)" with explicit parenthetical.
**Target**: main.tex abstract, sections/07-synthesis.tex (synthesis paragraph)

### P1.2 — Argue the exchangeability assumption for the bootstrap CI (Karypis)
**Issue**: The bootstrap CI for PP treats 10,000 METIS seed realizations as exchangeable draws, but this needs to be argued. The CI is for the expected PP of a random seed, not specifically for the DIA seed.
**Fix**: Add a paragraph in Section 2.3 distinguishing three interpretations: (a) CI for the DIA seed's output (not bootstrap --- the DIA seed is fixed), (b) CI for the expected output of a uniformly random seed (the bootstrap CI), (c) CI for a future DIA seed in a different application. Clarify that the paper reports interpretation (b) and argue why this is the relevant quantity for legal proceedings (it characterizes what any seed, including the DIA seed, is likely to produce).
**Target**: sections/02-seed-variance.tex (Section 2.3 and the Proposition)

### P1.3 — Separate shapefile simplification error from building-block resolution error (Duchin)
**Issue**: Section 4 treats the empirical $\Delta PP = 0.003$ from C.1 (building-block resolution, i.e., tracts vs. blocks) as capturing all PP measurement error, but shapefile Douglas-Peucker simplification error is a separate, unmeasured source.
**Fix**: Add a subsection 4.5 ("Shapefile simplification error") that either (a) estimates this source by comparing different-tolerance TIGER shapefiles for a sample of states (the paper reports doing this in 4.6 for shapefile vintage --- extend to tolerance), or (b) explicitly acknowledges this as an unquantified source and bounds it by noting that the TIGER redistricting shapefiles use a fixed tolerance and are the same for both algorithmic and enacted plans (so the comparison is unaffected even if the absolute values are not).
**Target**: sections/04-polsby-popper.tex

### P1.4 — Document the PES tract-level calibration for sigma_epsilon (Liang)
**Issue**: The $\sigma_\epsilon \approx 0.015$ calibration is attributed to "Census Bureau's tract-level undercount estimates from the 2020 PES experimental microdata," but official PES provides state-level and demographic-group estimates, not tract-level estimates.
**Fix**: Specify the exact PES product used and the methodology for deriving tract-level estimates. If small-area estimation was used, describe the model. If 0.015 is extrapolated from published state-level SDs, explain the extrapolation. Report the uncertainty in $\sigma_\epsilon$ itself and propagate it through the delta method CI.
**Target**: sections/03-census-sampling.tex (Section 3.4)

### P1.5 — Correct the Section 4.4 CI lower-bound description (Duchin)
**Issue**: Section 4.4 states "the lower bound (0.438) is above the tract-level point estimate (0.441)." This is factually incorrect: 0.438 < 0.441. The lower bound is below the point estimate, as expected.
**Fix**: Correct to: "The lower bound (0.438) is below the tract-level point estimate (0.441) but above the enacted plan mean PP (0.295), confirming that the algorithmic advantage is robust to resolution measurement error."
**Target**: sections/04-polsby-popper.tex (Section 4.4)

### P1.6 — Add legal-relevance mapping for the three uncertainty sources (Stephanopoulos)
**Issue**: Courts challenging algorithmic redistricting for different reasons will emphasise different uncertainty sources (seed: determinism claims; census: population equality claims; resolution: compactness measurement claims). The paper provides no guidance on this mapping.
**Fix**: Add a table or subsection in Section 1 or Section 8.2 mapping each uncertainty source to the category of legal challenge it addresses, following the model of C.0's Section 6.3 DIA compliance mapping.
**Target**: sections/01-introduction.tex or sections/08-conclusion.tex

---

## P2 — Suggested Improvements

### P2.1 — Add enacted plan EG CIs for the 50-state table (Rodden)
**Issue**: Table 5.5 reports algorithmic EG with bootstrap CIs but enacted EG as point estimates only.
**Suggestion**: Add enacted EG cross-election SDs (using 2016, 2018, 2020 elections) and compute enacted plan CIs by the same method used for algorithmic plans.
**Target**: sections/05-partisan-metrics.tex (Table 5.5)

### P2.2 — Report electoral uncertainty sensitivity for the EG CIs (Rodden)
**Issue**: Section 5.2's three-election electoral uncertainty estimate may substantially understate true decadal electoral uncertainty.
**Suggestion**: Add a sensitivity table showing joint EG CIs if electoral uncertainty is doubled (SD = 1.8%) or tripled (SD = 2.7%). Even at doubled uncertainty, the direction should remain robust.
**Target**: sections/05-partisan-metrics.tex

### P2.3 — Acknowledge TopDown Algorithm noise as a potential fourth uncertainty source (Duchin, Liang)
**Issue**: 2020 PL 94-171 data includes differential privacy noise from the TopDown Algorithm that is not mentioned as an uncertainty source.
**Suggestion**: Add a sentence in Section 3.1 acknowledging that the 2020 Census PL 94-171 data includes TopDown noise injection and noting whether this source is (a) captured by the PES calibration, (b) separate and bounded, or (c) excluded with justification (e.g., redistricting data uses noise-controlled vintage).
**Target**: sections/03-census-sampling.tex

### P2.4 — Add CI interpretation guidance for lay audiences (Stephanopoulos)
**Issue**: Frequentist CI interpretation is technically correct but may be misunderstood by judges and opposing counsel.
**Suggestion**: Add a footnote or boxed sidebar explaining the frequentist interpretation and suggesting robust litigation phrasing ("the data are consistent with values in the range [X, Y] at 95% confidence levels").
**Target**: sections/01-introduction.tex or sections/08-conclusion.tex
