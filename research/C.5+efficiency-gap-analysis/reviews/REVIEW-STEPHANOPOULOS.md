# Review: Measuring Partisan Fairness in Algorithmic Redistricting

**Reviewer**: Nicholas O. Stephanopoulos (Harvard Law School)
**Expertise**: Efficiency gap metric (co-creator), partisan gerrymandering law
**Date**: 2026-02-08
**Venue**: American Political Science Review

---

## Overall Assessment

This paper makes an important empirical contribution to the redistricting literature by providing the first national-scale application of the efficiency gap metric to algorithmic redistricting. The finding that algorithmic plans exhibit -3.2% efficiency gap while enacted plans show +5.1% is precisely the kind of benchmark courts and reformers need. However, the paper's treatment of the efficiency gap metric itself requires substantial strengthening, and the legal implications section needs more nuance.

**Verdict**: Weak Accept (major revisions needed)
**Score**: **3/4**

---

## Major Issues

### 1. Incomplete Treatment of Efficiency Gap Limitations (P1)

The paper uses efficiency gap as if it's an uncontroversial partisan fairness standard, but you don't adequately address the Supreme Court's skepticism in *Gill v. Whitford* and subsequent scholarly critiques. The Court questioned whether efficiency gap captures all forms of gerrymandering and whether it's too sensitive to factors unrelated to manipulation.

**Specific concerns:**
- No discussion of the difference between "cracking" and "packing" beyond Section 5's brief mention
- Missing analysis of how close elections affect efficiency gap calculations (small vote shifts can produce large EG changes)
- No acknowledgment that EG assumes uniform swing, which may not hold

**Required revision:** Add a dedicated subsection in Section 2 (Background) addressing efficiency gap limitations and why it remains useful despite *Gill* critiques. Discuss when EG is vs. isn't an appropriate metric (competitive states yes, landslide states problematic).

### 2. Threshold Problem for "Substantial Bias" (P1)

You repeatedly invoke the "+7% threshold" as evidence of substantial partisan bias (e.g., page 4: "12 of 45 enacted plan observations exceed +7%"). Where does this threshold come from? In *Stephanopoulos & McGhee* (2015), we proposed 7-8% as a *durability* threshold (bias likely to persist across elections), not as a legal standard for unconstitutionality.

**Problem:** Courts have never adopted 7% as a bright-line rule, and *Rucho* eliminated federal justiciability entirely. State courts have used different standards.

**Required revision:**
1. Clarify that 7% is a proposed *durability* threshold, not a legal standard
2. Discuss state-by-state variation in constitutional standards (PA uses "reasonably related," NC banned gerrymandering entirely)
3. Present your algorithmic baselines as providing state-specific benchmarks rather than universal thresholds

### 3. Proportionality Analysis Needs Connecting to EG (P1)

The new proportionality subsection (Section 4.6) is excellent and addresses a major gap, but it's currently disconnected from the efficiency gap framework. The relationship between EG and proportionality is: EG ≈ 2×(seat share - vote share) for competitive elections.

**Required additions:**
1. Show mathematically how your -3.2% algorithmic EG implies ~56% Democratic seats for 52% vote share
2. Explain why EG and proportionality sometimes give different signals (geographic concentration affects both but differently)
3. Connect the mean-median difference analysis to wasted votes explicitly

---

## Minor Issues

### 4. Missing Voter Turnout Analysis (P2)

Efficiency gap assumes equal turnout across districts, but urban districts (Democratic) often have lower turnout than suburban/rural districts (Republican). This means Democrats' wasted votes may be less "real" than the metric suggests.

**Suggested addition:** Brief discussion in Section 5 about how turnout differentials might affect EG calculations. If you have precinct-level data, show EG with and without turnout weighting.

### 5. Temporal Stability Section Too Brief (P2)

Figure 3 and the temporal stability analysis (Section 4.4) show that EG is stable across 2016-2020, which is crucial for establishing that these patterns aren't artifacts of specific elections. But this gets only one page. This deserves more development.

**Suggested expansion:**
- Quantify temporal stability using coefficient of variation or standard deviation
- Discuss implications: if EG varied wildly across elections, it wouldn't be useful for courts evaluating plans
- Compare temporal stability of algorithmic vs enacted plans (do enacted plans show MORE stability, suggesting deliberate entrenchment?)

### 6. Seats-Votes Curves Missing Citation (P3)

The seats-votes analysis in Section 4.6.2 is solid, but you don't cite the foundational work on seats-votes curves (Tufte, Gelman & King, etc.). The elasticity concept especially needs proper attribution.

**Fix:** Add 2-3 citations to the canonical seats-votes literature.

---

## Positive Aspects

1. **National scale**: The comprehensive 50-state, 3-year analysis is unprecedented and valuable
2. **Regional variation**: Rust Belt vs Sunbelt comparison reveals important patterns
3. **Algorithmic baseline**: Using genuinely neutral algorithms (can't access partisan data) provides clean counterfactual
4. **Legal relevance**: Clear framing for courts considering state constitutional challenges
5. **Honesty about limits**: Acknowledging that algorithms can't eliminate geographic bias is important

---

## Specific Recommendations for Revision

### Section 2 (Background)
- Add subsection "2.3 Efficiency Gap: Utility and Limitations"
- Discuss *Gill v. Whitford* Court skepticism
- Explain why EG remains useful for comparative benchmarks even if not constitutional standard

### Section 4 (Results)
- Replace "7% threshold" language with "7% durability threshold from Stephanopoulos & McGhee"
- Add statistical significance tests for all major findings (t-tests, confidence intervals)
- Expand temporal stability analysis with quantitative measures

### Section 5 (Discussion)
- Add subsection on turnout differentials and EG
- Connect proportionality findings to efficiency gap framework explicitly
- Discuss state-specific constitutional standards rather than federal threshold

### Section 6 (Conclusion)
- Clarify legal takeaways: these are benchmarks for state courts, not federal standards
- Emphasize that algorithmic plans don't "solve" gerrymandering but reduce it

---

## Questions for Authors

1. Do you have block-level or precinct-level election data, or are you using estimates? This affects EG precision.
2. Have you tested sensitivity to district compactness relaxation? What if you allow slightly less compact districts?
3. Could you compute EG for hybrid approaches (algorithmic base + minor adjustments for communities of interest)?

---

## Verdict Justification

This paper provides exactly the kind of empirical benchmark the redistricting field needs, and the national scale is impressive. The algorithmic baseline approach is methodologically sound and legally useful. However, the treatment of the efficiency gap metric itself is insufficiently critical, and the legal implications need more nuance given *Rucho* and state constitutional variation.

With revisions addressing the P1 issues—especially efficiency gap limitations, threshold clarification, and proportionality-EG connection—this will be a strong contribution to APSR. The data and analysis are fundamentally sound; the framing needs work.

**Recommendation**: Major revisions required, but accept with revisions.
