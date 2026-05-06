# Revision Plan — C.0 Multi-Faceted Validation Framework for Algorithmic Congressional Redistricting
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

### P1.1 — Separate metric stability from boundary stability (Karypis, Duchin)
**Issue**: The paper conflates PP metric stability (property of scores) with IoU boundary stability (property of geographic shapes). Courts care about boundary stability; the two concepts have different legal implications.
**Fix**: Introduce distinct terminology and subsections. "Metric temporal stability" (PP varies by ~10% over 20 years) and "boundary temporal stability" (IoU = 71% within geographic slices) should be clearly separated throughout Section 4 and the Property 2 definition.
**Target**: sections/04-temporal.tex, sections/06-synthesis.tex (Property 2 definition)

### P1.2 — Add CI for the variance ratio (sigma_geo / sigma_temp = 3.2x) (Liang)
**Issue**: The headline $3.2\times$ ratio is reported as a point estimate without a standard error or confidence interval. With 250 state-slice observations this is estimable.
**Fix**: Add bootstrap CI for the variance ratio across 250 state-slice time series. Also add an F-test (or Brown-Forsythe test) for the equality of geographic and temporal variances. Report CI in Table 1 (variance decomposition).
**Target**: sections/03-cross-census.tex (variance decomposition table and surrounding text)

### P1.3 — Qualify the DIA compliance language (Rodden, Stephanopoulos)
**Issue**: Section 6.3 states that the C-track "satisfies" DIA requirements R2 and R3. The evidence demonstrates compliance for specific samples (10/50 states at block level, 15 competitive states for partisan fairness), not for all 50 states under all conditions.
**Fix**: Replace "satisfies" with "provides empirical evidence toward satisfying" throughout Section 6.3. Add a sentence acknowledging the sample coverage gaps: "Full 50-state block-level validation (C.1) and extension to all multi-district states (C.5) remain to be completed."
**Target**: sections/06-synthesis.tex (DIA mapping table and surrounding text)

### P1.4 — Address the DIA's legal status (Stephanopoulos)
**Issue**: The DIA is cited throughout as if it were enacted law. Its actual status (model statute, proposed statute, or internal research framework) must be clearly stated.
**Fix**: Add a footnote or parenthetical at first mention of the DIA clarifying its legal status. If it is a model statute developed within this research program, state this explicitly. If it is a proposed federal statute, cite the bill number.
**Target**: sections/01-introduction.tex (first DIA citation), main.tex abstract

### P1.5 — Add VRA compliance as a fifth validated property or explain its absence (Stephanopoulos)
**Issue**: The four-property framework omits VRA compliance, which is the most frequently litigated redistricting claim. The 137 $\pm 1$ majority-minority district finding appears only as a parenthetical in Section 2.4.
**Fix**: Either (a) add a fifth property "VRA Stability" (majority-minority district counts are stable across resolutions, census years, and within the range of geographic variation) and add a brief section synthesising C.1's resolution finding and D-track findings; or (b) add a paragraph in Section 6.4 (limitations) explicitly acknowledging that VRA compliance is not a validated property and explaining why.
**Target**: sections/02-spatial-robustness.tex (Section 2.4), sections/06-synthesis.tex, sections/07-conclusion.tex

### P1.6 — Provide evidence or attribution for the temporal PP trend (Liang)
**Issue**: The improvement from PP = 0.412 (2000) to 0.441 (2020) is attributed to "refinements in tract boundary alignment," but no evidence for this attribution is provided.
**Fix**: Either cite Census Bureau documentation of boundary methodology changes between 2000 and 2020, or reframe the attribution as a hypothesis: "One plausible explanation is... An alternative explanation is population redistribution toward geographically simpler suburban areas." Avoid presenting an unsubstantiated causal claim.
**Target**: sections/03-cross-census.tex (Section 3.3)

---

## P2 — Suggested Improvements

### P2.1 — Add IoU distributional tail statistics (Liang)
**Issue**: The median IoU = 71% is reported but the tail behaviour (fraction of districts with IoU < 0.5) is absent.
**Suggestion**: Add a line to the IoU summary (Section 4.2) reporting the fraction of state-slices with IoU below 0.5 and whether any whole states show IoU < 0.5.
**Target**: sections/04-temporal.tex

### P2.2 — Move legal-use framing from conclusion to introduction (Stephanopoulos)
**Issue**: Section 7.2's sophisticated framing of how different C-track papers answer different legal challenges should orient readers from the beginning, not be revealed at the end.
**Suggestion**: Add 3--4 sentences in Section 1.1 previewing the legal-use structure: "A challenge that the algorithm fails at higher resolution is answered by C.1; a challenge that it is implicitly partisan is answered by C.5."
**Target**: sections/01-introduction.tex

### P2.3 — Correct or caveat the 2060 extrapolation (Rodden)
**Issue**: The linear extrapolation from 2 data points to gap closure "around 2060" is statistically unjustified and could be challenged in litigation.
**Suggestion**: Remove the specific date or replace with "if the current rate of reform continues, the algorithmic advantage in compactness would require several additional decades to narrow substantially --- a question the 2030 census will help address."
**Target**: sections/04-temporal.tex (Section 4.3.2)

### P2.4 — Clarify the competitive-state selection for EG analysis (Rodden)
**Issue**: The 15-state restriction is acknowledged in the limitations but not explained upfront in Section 5. Readers may take the 0.04/0.08 figures as nationally representative.
**Suggestion**: Add a sentence at the beginning of Section 5.2 noting that the analysis is restricted to states where EG is meaningful (>= 3 congressional districts) and acknowledging that the national picture includes many states with structural partisan dominance where EG comparisons are less informative.
**Target**: sections/05-partisan-fairness.tex
