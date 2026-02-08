# Revision Plan - Round 1 Peer Review

**Paper**: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals
**Review Round**: 1
**Date**: 2026-02-08
**Status**: Major Revision Required

## Review Summary

**Average Score**: 2.6/4
- 2 reviewers: Major revision (2/4)
- 3 reviewers: Minor revision (3/4)

**Consensus**: Major revision needed to address critical implementation and statistical concerns.

## Priority 1 Items (MUST Address Before Resubmission)

### ✅ P1-1: Multi-Constraint Implementation Verification [CRITICAL]
**Status**: ❌ Not addressed
**Flagged by**: Karypis (primary), Phillips (supporting)
**Severity**: Critical - Could invalidate paper's main conclusion

**Issue**: Karypis (METIS creator) suspects multi-constraint target weights may be fundamentally wrong—potentially asking for 60% of TOTAL minority population per MM district rather than proper fraction-based targets. This would make achieving 2+ MM districts mathematically impossible regardless of algorithm.

**Required actions**:
1. ☐ Verify tpwgts calculation in code (show exact formula used)
2. ☐ Re-run multi-constraint experiments with corrected target weights if needed
3. ☐ If results change significantly, rewrite paper accordingly
4. ☐ Add detailed tpwgts specification to paper (currently missing from Section 2.2)
5. ☐ Show example calculation for Alabama (2 MM districts, 36.9% minority)

**Timeline**: 2-3 weeks (includes re-running experiments if needed)

---

### ✅ P1-2: Theoretical Section Calculation Errors
**Status**: ❌ Not addressed
**Flagged by**: Karypis, Hendrickson, Cook
**Severity**: Critical - Section 3.1.2 undermines credibility

**Issue**: Section 3.1.2 "Theoretical Prediction" contains multiple calculation errors and unclear reasoning. Reviewers note confusion about VAP vs total population, impossible percentages (129%, 258%), and unclear constraint conflict formalization.

**Required actions**:
1. ☐ Remove or completely rewrite Section 3.1.2
2. ☐ Formalize constraint conflict mathematically (as Hendrickson suggests)
3. ☐ Define constraint "tightness" precisely (ratio of tolerance to target?)
4. ☐ Provide clear bounds analysis (Cook's request)
5. ☐ Use correct population accounting throughout

**Timeline**: 1-2 weeks

---

### ✅ P1-3: Asymmetric Configuration Counts
**Status**: ❌ Not addressed
**Flagged by**: Phillips (primary), Cook (supporting)
**Severity**: Critical - Makes 47.9% vs 35.0% comparison invalid

**Issue**: Comparing 140 edge-weighted configs vs 4 multi-constraint configs is fundamentally unfair. More configs = more chances to succeed, inflating edge-weighted success rate.

**Required actions**:
1. ☐ Run 140 multi-constraint configs (7 ubvec × 20 random seeds) OR reduce edge-weighted to 4
2. ☐ Report per-state success rates instead of aggregate (current Table 2 obscures this)
3. ☐ Use paired comparisons (best-of-N for each method)
4. ☐ Downplay "47.9% vs 35.0%" headline if unfair comparison persists
5. ☐ Focus on state-level outcomes (Alabama 2 vs 1 MM) which is robust result

**Timeline**: 2 weeks (if running new experiments) OR 1 week (if reframing analysis)

---

### ✅ P1-4: No Statistical Rigor
**Status**: ❌ Not addressed
**Flagged by**: Phillips (primary), Cook (supporting)
**Severity**: Critical - Single runs, no significance tests, no variance

**Issue**: Each configuration run once with fixed seed. No statistical testing, confidence intervals, or variance analysis. Cannot distinguish signal from noise.

**Required actions**:
1. ☐ Run each config 10-30 times with different random seeds
2. ☐ Report mean ± std for MM count, max minority %, edge cut
3. ☐ Conduct statistical significance tests (t-tests, Mann-Whitney U, chi-square)
4. ☐ Add confidence intervals to all plots
5. ☐ Check if 2.6/4 vs 2/4 differences survive statistical testing

**Timeline**: 3-4 weeks (re-running all experiments with multiple seeds)

---

## Priority 2 Items (Should Address - Significantly Strengthens Paper)

### P2-1: Missing METIS Implementation Details
**Flagged by**: Karypis, Hendrickson
**Status**: ❌ Not addressed

**Actions**:
- ☐ Report METIS version used
- ☐ Show exact command-line flags for both methods
- ☐ Specify coarsening/refinement parameters
- ☐ Document any preprocessing (e.g., removing disconnected components)

**Timeline**: 1 day

---

### P2-2: Oversimplified VRA Legal Standards
**Flagged by**: Duchin
**Status**: ❌ Not addressed

**Actions**:
- ☐ Clarify: 50% is threshold, not legal requirement (Gingles test more complex)
- ☐ Acknowledge coalition districts (minority + allies) vs pure majority-minority
- ☐ Cite recent Supreme Court cases (Brnovich, Allen v. Milligan)
- ☐ Note legal standards vary by state and election type

**Timeline**: 2-3 days

---

### P2-3: Aggregate Minority Definition
**Flagged by**: Duchin
**Status**: ❌ Not addressed

**Actions**:
- ☐ Acknowledge aggregation of all non-white groups is legally/politically contentious
- ☐ Report results for Black-only vs aggregate minorities (at least for Alabama)
- ☐ Discuss coalition formation realities
- ☐ Note Census 2020 allows multiracial identification (complicates definition)

**Timeline**: 1 week (if re-analyzing with Black-only data)

---

### P2-4: Geographic Compactness Analysis
**Flagged by**: Duchin, Cook
**Status**: ❌ Not addressed

**Actions**:
- ☐ Calculate Polsby-Popper and Reock scores (gold standards in redistricting)
- ☐ Generate district maps for Alabama showing 2 MM vs 1 MM visually
- ☐ Verify edge cut correlates with geographic compactness
- ☐ Acknowledge edge cut is proxy, not direct measure

**Timeline**: 1 week (requires tract geometries)

---

### P2-5: Narrow Scope vs Generalization Claims
**Flagged by**: All reviewers
**Status**: ❌ Not addressed

**Actions**:
- ☐ Tone down claims of broad applicability (only tested METIS + redistricting)
- ☐ Remove or heavily qualify Section 6.1 "When to Use..." guidance
- ☐ Add caveats: findings may not hold for other partitioners (ParMETIS, Zoltan)
- ☐ Acknowledge untested application domains need empirical validation

**Timeline**: 1 day

---

### P2-6: Missing Ablation Studies
**Flagged by**: Hendrickson, Cook
**Status**: ❌ Not addressed

**Actions**:
- ☐ Test hypothesis: run edge-weighted with NO population balance constraint
- ☐ Test: multi-constraint with only population constraint (1D weights)
- ☐ Isolate effect of constraint conflict vs edge weighting vs METIS behavior
- ☐ Strengthen causal claims with controlled experiments

**Timeline**: 1 week

---

### P2-7: No Bounds Analysis
**Flagged by**: Cook, Hendrickson
**Status**: ❌ Not addressed

**Actions**:
- ☐ Provide bounds on solution quality (edge cut optimality)
- ☐ Compare to LP relaxation or other lower bounds if feasible
- ☐ Quantify how far results are from theoretical optimum
- ☐ Discuss METIS approximation guarantees

**Timeline**: 1-2 weeks (may require new analysis)

---

## Priority 3 Items (Nice to Have - Polish)

### P3-1 through P3-10
[See SYNTHESIS.md for full list - includes figure improvements, writing polish, additional citations, etc.]

**Timeline**: 1-2 weeks total

---

## Recommended Revision Strategy

### Phase 1: Verify Foundation (Weeks 1-3)
**Focus**: P1-1 first - this is make-or-break
1. Audit multi-constraint implementation
2. Re-run experiments if needed
3. Decide whether to proceed or pivot paper

### Phase 2: Statistical Rigor (Weeks 4-7)
**Focus**: P1-3 and P1-4
1. Design balanced experimental protocol (equal configs or paired comparisons)
2. Run multiple seeds for all configurations
3. Conduct significance testing

### Phase 3: Theory Cleanup (Weeks 8-9)
**Focus**: P1-2
1. Rewrite Section 3.1.2 with correct math
2. Formalize constraint conflict theory
3. Add bounds analysis (P2-7)

### Phase 4: Enhancements (Weeks 10-12)
**Focus**: P2 items
1. Add METIS details (P2-1)
2. Fix VRA legal standards (P2-2, P2-3)
3. Add compactness metrics and maps (P2-4)
4. Tone down generalization claims (P2-5)

### Phase 5: Polish (Weeks 13-14)
**Focus**: P3 items
1. Figure improvements
2. Writing polish
3. Response letter

---

## Expected Outcome

**If P1 + most P2 addressed**:
- Round 2 score: 3.0-3.5/4
- Recommendation: Accept with minor revisions
- Likely outcome: Acceptance after addressing final P3 items

**If only P1 addressed (P2 mostly skipped)**:
- Round 2 score: 2.5-3.0/4
- Recommendation: Another major revision
- Risk: Reviewers may lose patience if P2 items ignored

**If P1-1 reveals flawed implementation**:
- May need to pivot paper entirely or withdraw
- Check this FIRST before investing effort elsewhere

---

## Gate to Next Stage

**Requirements to advance from `revision` to `recheck`**:
- ✅ All 4 P1 items marked as addressed
- ✅ Detailed response letter documenting each fix
- ✅ Updated manuscript with changes highlighted

**Current status**: 0/4 P1 items addressed

---

## Notes

- **P1-1 is highest priority**: If multi-constraint implementation is wrong, results may completely change
- **Statistical rigor (P1-4) is non-negotiable**: Phillips and Cook will reject without it
- **Theoretical section (P1-2) undermines credibility**: Multiple reviewers noted this hurts paper's impact
- **Don't ignore VRA concerns (P2-2, P2-3)**: Duchin's domain expertise is important for redistricting validity

---

## Estimated Total Timeline

**Minimum (P1 only)**: 8-10 weeks
**Recommended (P1 + P2)**: 12-14 weeks
**Maximum (P1 + P2 + P3)**: 14-16 weeks

**Resubmission target**: Late May 2026 (if starting now)
