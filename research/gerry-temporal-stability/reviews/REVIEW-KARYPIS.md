# Review: Cross-Census Temporal Stability
## Reviewer: Dr. George Karypis (University of Minnesota)
**Expertise**: METIS algorithm, graph partitioning, multilevel algorithms
**Date**: 2026-02-08
**Score**: 3.5/4.0 (Strong Accept)

---

## Overall Assessment

This paper makes a significant empirical contribution by being the first to systematically quantify temporal stability differences between recursive bisection and direct k-way partitioning methods. As the creator of METIS, I find the technical implementation sound and the research question highly relevant to practitioners who must perform redistricting every decade.

**Key Strengths**:
1. **Novel research question**: Temporal stability has been assumed but never measured
2. **Correct METIS usage**: Authors properly use RecursiveBisection class vs gpmetis k-way
3. **Appropriate scale**: 5 states × 2 methods × 2 years = sufficient statistical power
4. **Honest effect size**: 1.1% improvement is modest but real and consistently observed

**Key Limitations**:
1. **Abstract oversells findings**: Claims "80% tract retention vs 70%" but actual results show 71.6% vs 72.4%
2. **Missing algorithmic analysis**: Why does hierarchical structure provide stability? Lacks theoretical explanation
3. **Limited generalization**: Only southern states with specific demographic patterns

---

## Major Issues (P1 - Blocking)

### P1.1: Abstract Does Not Match Results
**Issue**: Abstract claims "80% tract retention versus 70% for n-way partitioning (+14 percentage point improvement)" but Table 1 shows:
- Recursive: 71.6% disruption (28.4% retention)
- N-way: 72.4% disruption (27.6% retention)
- Actual difference: 0.8 percentage points, not 14

**Fix Required**: Rewrite abstract with actual numbers. The 1.1% stability advantage (71.6% vs 72.4% disruption) is still publishable but must be stated accurately.

**Evidence**: FINDINGS_SUMMARY.md shows correct numbers; abstract is inconsistent.

---

### P1.2: Recursive Bisection Implementation Not Validated
**Issue**: Paper claims to use "true recursive bisection" but doesn't verify the hierarchical tree structure is actually created. Need to show:
- Tree depth matches log₂(k) for k districts
- Parent-child splits are preserved across rounds
- Edge cuts at each level of hierarchy

**Fix Required**: Add Section 3.5 "Implementation Validation" showing:
- Alabama's 7-district tree has depth 3 (log₂(7) ≈ 2.8)
- Dendrogram showing hierarchical structure for one state
- Verification that early splits remain stable

**Rationale**: Without this, readers can't trust the "hierarchical" claim.

---

## Major Issues (P2 - Important)

### P2.1: Missing Computational Complexity Analysis
**Issue**: Paper mentions "N-way is 60x faster" but doesn't explain:
- Why the speed difference?
- What's the O(n) complexity for each method?
- At what scale does the 60x gap matter?

**Recommendation**: Add complexity analysis:
- Recursive: O(n log k) for k districts
- N-way: O(n k) but with better constants
- For k=7 to k=14 (typical district counts), difference is negligible

---

### P2.2: Census Tract Boundary Changes Underexplored
**Issue**: 26% of tracts were redrawn between 2010-2020. This causes unavoidable disruption regardless of method. Paper doesn't adequately separate:
- Disruption from algorithmic differences (what you measure)
- Disruption from census boundary changes (uncontrollable)

**Recommendation**: Add sensitivity analysis:
- Restrict to stable tracts only (73.9% coverage)
- Show whether 1.1% advantage holds when excluding boundary-changed tracts

---

### P2.3: No Comparison to Human Redistricting
**Issue**: 71-72% disruption seems extremely high. What's the baseline?
- How much do actual redistricting commissions change districts?
- Is algorithmic redistricting more or less stable than human-drawn plans?

**Recommendation**: Add one state comparison to actual 2010→2020 congressional maps.

---

## Minor Issues (P3 - Nice to Have)

### P3.1: Visualization Needs Improvement
**Current**: Bar charts comparing methods
**Better**: Side-by-side maps showing 2010 vs 2020 districts for one state, with tract-level coloring showing stability

---

### P3.2: Missing Related Work on Ensemble Methods
**Issue**: GerryChain and other ensemble methods generate thousands of plans. How does temporal stability compare?

**Recommendation**: Add paragraph in Related Work discussing ensemble stability.

---

### P3.3: Performance Metric is Confusing
**Issue**: "Population disruption rate" could mean % disrupted or % stable. Current usage (71.6% = bad) is counter-intuitive.

**Recommendation**: Use "Population Stability Rate" (28.4% = good) or explicitly define lower-is-better upfront.

---

## Detailed Technical Comments

### METIS Parameter Validation
✅ **Correct**: Use of `RecursiveBisection` class with `nparts` parameter
✅ **Correct**: Edge weights (5x at 40% threshold) applied consistently
✅ **Correct**: ufactor=5 (0.5% imbalance tolerance)
⚠️ **Concern**: niter=100 is high. Standard is 10. Justify this choice.

### Statistical Significance
The paper claims 1.1% advantage is "statistically meaningful" but provides no significance test. With n=5 states, this is marginal. Consider:
- Paired t-test on 5 state differences
- Bootstrap confidence intervals
- Report p-value and confidence interval

### Generalization Claims
Paper title says "Over Decades" but only covers 2010-2020 (one decade). Consider:
- Either add 2000 data (2000→2010→2020)
- Or revise title to "Across the 2010-2020 Decade"

---

## Recommendations for Revision

### Tier 1 (P1 - Must Fix Before Accept)
1. **Rewrite abstract** with correct numbers (71.6% vs 72.4%, not 80% vs 70%)
2. **Add implementation validation** (Section 3.5) proving hierarchical structure
3. **Provide statistical significance test** (paired t-test or bootstrap)

### Tier 2 (P2 - Strongly Recommended)
1. Add computational complexity analysis
2. Separate census boundary changes from algorithmic disruption
3. Compare to one real-world redistricting baseline

### Tier 3 (P3 - Would Strengthen)
1. Improve visualizations (side-by-side maps)
2. Discuss ensemble method stability
3. Clarify "disruption" vs "stability" terminology

---

## Recommendation

**Score: 3.5/4.0 (Strong Accept with Minor Revisions)**

This paper addresses a genuinely novel question and provides the first empirical evidence on temporal stability of partitioning methods. The technical implementation is sound and the findings are credible, though modest in magnitude.

The abstract overclaims (P1.1) must be fixed—this is a dealbreaker for integrity. The implementation validation (P1.2) is necessary to support the paper's central claim about hierarchical structure benefits.

With these P1 items addressed, this is a solid contribution to the redistricting and graph partitioning literature. The 1.1% effect may be small, but it's the first measurement of this phenomenon, which has high value for the field.

**Venue Fit**: ACM-KDD is appropriate given the algorithmic comparison and empirical methodology. This bridges graph algorithms and societal applications effectively.

---

## Meta-Review Notes

**Author Rebuttal Needed On**:
- Why abstract numbers differ from results (P1.1)
- Whether hierarchical tree structure was actually verified (P1.2)
- Statistical significance of 1.1% difference with n=5 states

**Potential Accept Conditions**:
- Fix abstract to match results
- Add implementation validation section
- Provide significance test with confidence intervals
