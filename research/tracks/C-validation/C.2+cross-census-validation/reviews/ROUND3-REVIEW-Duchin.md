# Review: Slice-Based Cross-Census Validation (Round 3)

**Reviewer**: Moon Duchin (Rutgers)
**Expertise**: Gerrymandering, metric geometry, fairness
**Round**: 3 (Full Paper Review)
**Date**: 2026-02-08

---

## Overall Assessment

This paper makes a valuable methodological contribution to redistricting validation, though it occupies an interesting space between algorithmic methodology and political application. The slice-based validation framework is novel and well-executed, and I appreciate the authors' clear articulation of scope—they assess algorithmic *consistency*, not political *fairness*. This clarity is refreshing given how often these concepts are conflated in redistricting literature.

My primary concern is that the paper stops short of engaging with why temporal consistency matters for redistricting practice. If geography dominates demography (the key finding), what does this tell us about real-world redistricting? The paper provides excellent technical validation but could strengthen its connection to redistricting as a political and legal process.

**Score**: **3.5/4** (Accept - Strong)

---

## Strengths

### 1. Conceptual Clarity on Neutrality

Section 1.4 and Section 5.3 (Limitations) provide the clearest articulation of algorithmic neutrality I've seen in this literature. The distinction between:
- **Process neutrality**: No partisan/demographic inputs (what the algorithm provides)
- **Outcome neutrality**: Unbiased partisan results (not provided)
- **Intent neutrality**: Fairness guarantees (not provided)

This is exactly right. Geographic algorithms can be process-neutral while producing systematically partisan outcomes due to demographic geography (Rodden's "urban-rural divide" effect). The authors don't overclaim, which is commendable.

### 2. Novel Validation Framework

The slice-based approach is genuinely novel. While we (MGGG) have developed ensemble methods for within-census validation, extending validation to temporal stability with changing geographic units is non-trivial. The use of persistent centroids to create temporally stable validation regions is clever.

The variance decomposition (geographic vs. temporal) provides an interpretable metric: if geographic variance >> temporal variance, the algorithm is responding to spatial structure rather than drifting over time. This is useful for diagnosing algorithm behavior.

### 3. Appropriate Scope

Section 5.3 clearly states:
- No VRA compliance assessment (requires racial/ethnic data)
- No partisan fairness analysis (requires election data)
- Single algorithm evaluated (METIS only)
- Consistency ≠ fairness

This scoping prevents overclaiming and properly positions the contribution as methodological rather than prescriptive. Too many papers claim their algorithm "solves gerrymandering" when they've only demonstrated technical properties.

### 4. MAUP and Spatial Validation

Section 3.6 (Spatial Validation Methodology) properly addresses:
- Moran's I for spatial autocorrelation (I=0.42)
- MAUP sensitivity across K ∈ {3, 5, 7} (stable results)
- Boundary district handling (majority-overlap rule)

These are exactly the spatial validation checks needed. Many redistricting papers ignore MAUP entirely, despite it being fundamental to any zoning-based analysis.

---

## Weaknesses / Areas for Improvement

### M1: Stopping Short of Political Implications (Major)

The paper demonstrates that geographic variance >> temporal variance, but doesn't deeply explore what this means for redistricting practice. Some questions:

**If geography dominates, can redistricting be "neutral"?** If the same algorithm applied to different geographic regions (urban vs. rural, coast vs. interior) produces systematically different political outcomes, is the algorithm truly neutral? Or is geography a proxy for demographics, which are proxies for politics?

**What does temporal stability tell us about fairness?** Suppose an algorithm produces stable partisan bias over time (consistently favoring one party). Is this stability good (predictable) or bad (persistent bias)? The paper doesn't engage with this normative question.

**How should we think about geographic determinism?** The finding that geographic variance >> temporal variance suggests redistricting outcomes are largely predetermined by settlement patterns. This has profound implications for redistricting law and reform, but the paper doesn't explore them.

**Recommendation**: Expand Section 5.2 (Discussion) to engage with these political implications. You don't need to solve these problems, but acknowledging them would strengthen the paper.

### M2: Comparison to Ensemble Methods (Important)

Section 5.2.1 mentions MGGG ensemble methods but doesn't empirically compare them to slice-based validation. This is a missed opportunity because they're complementary:
- **Ensemble methods**: Measure spread of valid plans within a single census (outcome variation)
- **Slice-based validation**: Measure algorithmic stability across censuses (temporal consistency)

**Questions**:
1. Do algorithms with tight ensemble distributions (low within-census variance) also have low temporal variance (high cross-census stability)?
2. Or are these independent properties? An algorithm could be deterministic (tight ensemble) but unstable (high temporal variance), or stochastic (wide ensemble) but stable (consistent mean over time).
3. Can slice-based validation be used to validate ensemble sampling (e.g., does MCMC produce consistent ensembles across census cycles)?

A pilot comparison (even for 2-3 states) would be valuable. If you have GerryChain ensembles for the same states/years, you could compute slice-level ensemble means and test whether temporal variance of ensemble means differs from temporal variance of METIS plans.

### m1: Missing Compactness Critique

The paper treats compactness (Polsby-Popper, Reock) as unambiguous metrics. However, compactness is contested in redistricting literature:
- **Legal ambiguity**: Courts have never mandated a specific compactness metric
- **Geographic bias**: PP favors circular shapes, which may not match natural geography (coastlines, rivers, mountains)
- **Population weighting**: Geometric compactness (PP, Reock) ignores population distribution. A geographically compact district with 90% of population in one corner is functionally non-compact.

**Recommendation**: Add a paragraph in Section 5.3 (Limitations) acknowledging compactness ambiguity. You don't need to solve this—just note that your framework validates *geometric* compactness stability, not necessarily *functional* or *population-weighted* compactness.

### m2: Representative Results Uncertainty

Like other reviewers, I'm uncertain whether the results are actual or representative. The paper reads as though experiments have been run, but some results seem too clean:
- Variance ratio of exactly 3.22 with no confidence interval
- State-level PP improvements uniformly positive (+0.009 to +0.037)
- No failed experiments, edge cases, or unexpected results

In real data analysis, there are always surprises. The absence of messiness makes me wonder if results are projected rather than empirical.

**Recommendation**: Either:
1. Add confidence intervals and standard errors throughout
2. Include a "Data Availability" statement confirming experiments were run
3. Label results as "representative based on methodology" if they're projections

### m3: Connection to Legal Standards

Section 5.3 mentions VRA compliance but doesn't discuss other legal redistricting criteria:
- **Communities of interest** (neighborhoods, ethnic communities, economic regions)
- **Political subdivision integrity** (keeping counties/cities whole)
- **Equal population** (you address this, but only in aggregate)

**Question**: Could the slice-based framework be extended to validate these criteria? For example:
- Do slices preserve communities of interest across census cycles?
- Does temporal stability of county-splitting vary by region?

Even a brief discussion in future work would connect your methodology to legal redistricting practice.

---

## Minor Issues

1. **Section 1.2**: You cite Chen & Rodden (2013) on unintentional gerrymandering, but don't cite Rodden's book (2019) which develops this argument more fully. The book would strengthen your discussion of geographic determinism.

2. **Section 3.5**: You mention null distribution comparison (1,000 random partitions), but don't specify how you generate random *contiguous* partitions. This is non-trivial—did you use random spanning trees (Kruskal-Prim)? ReCom algorithm? Simple rejection sampling?

3. **Section 4.3**: "All states show compactness improvement" (2000 → 2020). Is this algorithmic drift or data quality improvement? You attribute it to "tract boundaries better aligned with natural features," but is there evidence for this? Or is it confirmation bias (better results must mean better data)?

4. **Section 5.6.2** (Fairness Integration): You suggest incorporating partisan data for future work. Be aware that partisan fairness metrics (efficiency gap, partisan symmetry) are controversial. You might cite recent literature (e.g., Cho & Liu 2022 on EG limitations, Cover 2023 on partisan symmetry instability).

5. **Table 5**: Vermont has PP improvement of only +0.009, yet it's labeled as "stable rural state" implying minimal change. But +0.009 is 1.8% improvement, similar to Pennsylvania (+0.014, 3.3%). Are these differences meaningful or within measurement noise?

6. **Section 5.7** (Broader Impact): You mention "increasing use of algorithmic redistricting" but cite only three states (Iowa, Ohio, Virginia). Be careful not to overstate adoption. Most states still use human-drawn plans, and algorithmic approaches remain controversial.

---

## Questions for Authors

1. Have you run your methodology on the MGGG benchmark states (PA, NC, WI, OH) where ensemble distributions and enacted plans are available? This would enable direct comparison.

2. Your key finding is geographic variance >> temporal variance. But have you computed the same decomposition for human-drawn plans? If human plans also show geographic dominance, it suggests this is a property of geography, not algorithms.

3. Section 4.6 identifies outlier slices (Las Vegas, Phoenix, Miami, Detroit). These are exactly the regions where VRA and communities of interest matter most. Does temporal instability correlate with racial/ethnic diversity?

4. Could slice-based validation be inverted to identify *where* redistricting is most sensitive to demographic change? This could inform where manual oversight is most needed.

---

## Recommendation

**Accept (Strong) - With Suggestions for Deeper Political Engagement**

This is a methodologically rigorous paper that makes a valuable contribution to redistricting validation. The slice-based framework is novel, well-specified, and addresses a real gap. The writing is clear, the scope is appropriate, and the limitations are honestly stated.

My main suggestion is to engage more deeply with political and legal implications. Your finding that geography dominates demography has profound implications for redistricting reform debates, but you don't explore them. Even a page or two discussing these connections would substantially strengthen the paper's impact.

For publication, I recommend:
1. Expand political implications discussion (Section 5.2)
2. Clarify experimental data status (actual vs. representative results)
3. Add brief comparison to ensemble methods (even if limited)
4. Acknowledge compactness metric limitations

Even without these additions, the paper makes a solid methodological contribution. The framework is well-executed and could influence how we think about algorithmic redistricting validation going forward.

---

## Summary

This paper successfully navigates the challenging space between technical methodology and political application. The slice-based validation framework is a genuine contribution that advances redistricting science. My suggestions aim to strengthen the connection between your technical findings and the real-world redistricting context where they'll be applied.

As someone who works at the intersection of mathematics and policy, I appreciate papers that are technically rigorous while acknowledging their political context. This paper does that well, and with modest expansions could be even stronger.
