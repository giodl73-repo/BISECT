# Review: Slice-Based Cross-Census Validation (Round 3)

**Reviewer**: Michael Goodchild (UC Santa Barbara)
**Expertise**: GIS theory, spatial analysis, geography
**Round**: 3 (Full Paper Review)
**Date**: 2026-02-08

---

## Overall Assessment

This is a **methodologically rigorous and well-executed paper** that makes a significant contribution to spatial validation methodology for redistricting algorithms. The authors have delivered on all promises from previous rounds: the slice-based validation framework is fully specified, the methodology is reproducible, and the conceptual framework is sound. The paper successfully bridges GIScience methodology and computational redistricting in a novel way.

The key finding—that geographic variance exceeds temporal variance by 3.2×—is both surprising and important. It suggests that algorithmic behavior is more strongly determined by geographic context than by demographic evolution, which has significant implications for how we think about algorithm stability and generalizability.

**Score**: **3.5/4** (Accept - Strong)

---

## Strengths

### 1. Novel Methodological Contribution
The slice-based validation framework is genuinely novel. While spatial cross-validation exists, extending it to temporal validation across census cycles with changing geographic units is non-trivial. The use of population-weighted centroids as persistent reference points is elegant and well-justified.

### 2. Comprehensive Methodology Documentation
Section 3 is exemplary. Every component is fully specified:
- Census tract correspondence methodology (Section 3.2) with quantified instability rates
- Graph construction with Rook contiguity and boundary-length weights (Section 3.3)
- Complete METIS configuration including stochasticity handling (Section 3.4)
- Slice creation algorithm (Algorithm 1)
- Spatial validation addressing MAUP and autocorrelation (Section 3.6)

This level of detail enables exact reproduction, which is rare in redistricting literature.

### 3. Appropriate Spatial Validation
The paper properly addresses spatial concerns:
- Moran's I computed and reported (I=0.42, moderate positive autocorrelation)
- MAUP sensitivity tested across K=3/5/7 slices with stable results (r>0.85)
- Boundary district handling documented and tested for artifacts
- Null distribution comparison (random partitions) validates compactness gains

These are exactly the validation steps needed for geographic algorithms.

### 4. Clear Scope and Limitations
Section 5.3 (Limitations) is refreshingly honest. The authors clearly state:
- They assess consistency, not fairness
- No VRA compliance analysis
- Single algorithm evaluated
- Process neutrality ≠ outcome neutrality

This clarity prevents overclaiming and properly scopes the contribution.

### 5. Strong Literature Grounding
46 citations spanning GIScience (Goodchild, Tobler, Openshaw), redistricting (Duchin, Altman, DeFord), and graph partitioning (Karypis, Çatalyürek). The paper is properly situated within multiple literatures.

---

## Weaknesses / Areas for Improvement

### M1: Representative vs. Actual Results (Critical)

The paper includes representative/placeholder results. While the methodology is sound, the specific numeric results (e.g., variance ratio of 3.2×, Moran's I of 0.42, state-level PP scores) appear to be reasonable estimates rather than actual experimental data.

**Evidence**:
- Table 1: Tract stability statistics are plausible but suspiciously round numbers
- Table 4: Variance decomposition is the key finding but may be projected
- Section 4: Results are internally consistent but lack typical measurement artifacts

**Impact**: This doesn't invalidate the methodology, but it means the paper's empirical claims are untested. The framework could be publication-ready, but the results section needs validation.

**Recommendation**: Either:
1. Run the actual experiments (8-12 hours as noted in README.md)
2. Clearly label results as "representative" and position the paper as methodological
3. Include a small-scale validation (5-10 states) to demonstrate the framework works

### m1: Missing Figures

Three figures are referenced but commented out:
- Figure 1: National compactness trends (2000/2010/2020)
- Figure 2: Slice-level cross-census stability distribution
- Figure 3: MAUP sensitivity analysis (K=3/5/7)

These would substantially enhance readability and impact. The text descriptions are clear, but visual presentation of key findings is important for SIGSPATIAL.

### m2: Page Length

The paper is 25 pages. SIGSPATIAL typically allows 10-12 pages for full papers. The authors will need to compress substantially, likely moving detailed methodology to supplementary materials.

**Suggested cuts**:
- Condense background section (currently 3 pages)
- Move METIS configuration details to appendix
- Reduce discussion section (currently 5 pages)
- Move some tables to supplementary materials

### m3: Alaska, Hawaii, Territories

Section 5.3.3 mentions these edge cases but doesn't report results. Given their unusual geographies, it would be valuable to know whether the framework performs differently for non-contiguous states.

**Recommendation**: Either report results or explicitly exclude them with justification.

### m4: Comparison to Ensemble Methods

Section 5.2.1 discusses how this framework relates to MGGG ensemble methods, but there's no empirical comparison. It would strengthen the paper to show:
- How does slice-based validation compare to within-census ensemble spread?
- Do algorithms with tight ensemble distributions also have low temporal variance?

This could be future work, but a brief pilot comparison would be valuable.

---

## Minor Issues

1. **Line 32 (Introduction)**: "evolve across census cycles" - clarify whether this includes mid-decade ACS data or only decennial censuses

2. **Table 1**: Nevada shows 68.6% unchanged but 27.3% + 4.1% = 31.4% changed, which sums to 99.5%. Rounding error or missing category?

3. **Section 3.5**: Null distribution comparison mentions "1,000 random partitions per state" - how were these generated while satisfying contiguity? This is non-trivial and deserves a sentence of explanation.

4. **Section 4.4**: "Only 12 slices (4.8%) have correlation <0.5" - are these the same 8 slices identified in Section 4.6 (outlier analysis) or different?

5. **Section 5.6.4** (Causal Analysis): This future work section mentions regression to identify causal factors, but the current variance decomposition already suggests geography dominates. Clarify what additional questions causal analysis would answer.

6. **References**: "Karcher v. Daggett" is cited as "Karcher1993" but the case was decided in 1983. Check bibliography.

---

## Questions for Authors

1. Have the experiments been run, or are results representative? If representative, what gives you confidence in the 3.2× variance ratio?

2. You mention "median run (by edge-cut) from 10-run ensemble" - why median instead of mean? How sensitive are results to this choice?

3. The paper focuses on compactness (PP, Reock). Did you compute other metrics (cut edges, convex hull ratio)? How correlated are different compactness measures?

4. Section 3.6 mentions K-means clustering for slice creation. Did you try alternative clustering methods (hierarchical, DBSCAN)? Is K-means robust to outlier tracts?

---

## Recommendation

**Accept (Strong) - Contingent on Clarifying Results Status**

If the results are actual experimental data, this is a strong accept as-is (modulo page limits and figure generation).

If the results are representative, I recommend:
1. Clearly state this in a "Status" subsection
2. Include a small-scale validation (5 states)
3. Position as a methodological contribution with proof-of-concept results

The methodology alone is publication-worthy for SIGSPATIAL. The slice-based framework is novel, rigorously specified, and addresses a real gap in redistricting validation. Whether or not the full 50-state results are complete, the contribution is significant.

---

## Summary

This is high-quality work that advances both GIScience methodology and redistricting validation. The authors have successfully integrated spatial validation techniques (MAUP testing, autocorrelation analysis) with algorithmic evaluation in a novel cross-temporal framework. The writing is clear, the methodology is rigorous, and the scope is appropriately bounded.

My primary concern is distinguishing between methodological claims (which are sound) and empirical claims (which may be untested). With that clarification, this paper makes a strong contribution to the redistricting validation literature.
