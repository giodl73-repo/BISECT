# Revision Plan - The 42% Threshold
**Paper**: The 42% Threshold: Geographic Limits of VRA Compliance Through Algorithmic Redistricting
**Round**: 1 → 2
**Date**: 2026-02-08
**Status**: Major Revisions Required (2.6/4 average score)

---

## Executive Summary

**Gate Status**: ✅ PASSED (avg 2.6/4, no scores <2.0)
**Recommendation**: Major revisions required before acceptance
**Timeline**: 3-4 weeks for P1 items, +1-2 weeks for high-value P2 items

**Core Finding**: The 42% threshold finding is robust and valuable. Revisions focus on **framing** (empirical vs legal standard) and **statistical rigor** (small N=5, confidence intervals), not methodology.

**Critical Path**: Address all 5 P1 blocking issues → P2.1-P2.3 for statistical strength → Resubmit for Round 2 reviews

---

## Priority 1: Blocking Issues (MUST ADDRESS)

### P1.1: State-level vs District-level Analysis Mismatch
**Reviewers**: Pildes (primary), Stephanopoulos
**Effort**: 2-3 days
**Status**: [ ] Not Started

**Required Actions**:
1. **Add Section 2.3**: "Scope and Modeling Assumptions"
   - Clarify proportional targets are modeling assumptions, not legal requirements
   - Explain distinction between "Can X% minority create Y% MM districts?" (research question) vs "Must states create MM districts proportional to minority %?" (legal requirement)
   - Estimated: 2-3 pages

2. **Reframe Results Section** (Section 4):
   - Replace "VRA compliance achieved" → "Target MM count achieved"
   - Replace "VRA threshold" → "Empirical feasibility pattern"
   - Throughout: Specify "under proportionality assumption"

3. **Expand Discussion 5.3** (Legal Implications):
   - Add 2-3 paragraphs: "From Empirical Findings to Legal Doctrine"
   - Address how courts might use threshold (one factor vs dispositive)
   - Distinguish empirical regularity from self-executing legal standard

4. **Revise Conclusion**:
   - Replace "geographic reality" → "empirical patterns that may inform legal analysis"
   - Add: "Courts will evaluate district-by-district feasibility; our state-level threshold provides contextual guidance"

---

### P1.2: Proportionality Assumption Lacks Legal Foundation
**Reviewers**: Pildes, Gerken, Stephanopoulos
**Effort**: 3-4 days (includes re-running experiments)
**Status**: [ ] Not Started

**Required Actions**:
1. **Add explicit statement in Methodology** (Section 3):
   ```
   "We adopt proportional representation as a modeling assumption to detect
   systematic patterns in VRA feasibility. This is not a legal requirement—
   Section 2 prohibits dilution where compact minorities exist, not proportional
   allocation of MM districts. However, proportionality provides a principled
   baseline for cross-state comparison."
   ```

2. **Sensitivity Analysis** (NEW: Section 4.9):
   - Re-run edge-weighted optimization targeting [proportional - 1] MM districts
   - Example: Alabama (proportional=2) → test target=1
   - Show how threshold changes: "42% for proportional representation; ~38% for [proportional-1]"
   - Create new table showing sensitivity to MM targets
   - Estimated: 1 day computation + 1 day analysis

3. **Discussion Addition** (Section 5.4):
   - "Our proportionality assumption likely provides upper bound on VRA requirements.
     Actual doctrine might require fewer MM districts, lowering the threshold."
   - Discuss cases where courts mandated <proportional MM districts

4. **Throughout paper**:
   - Replace "VRA threshold" → "feasibility threshold under proportional representation"
   - Clarify findings apply to proportionality goals, not minimum compliance

---

### P1.3: Insufficient Engagement with Gingles Three-Prong Test
**Reviewers**: Pildes (primary), Gerken
**Effort**: 2 days
**Status**: [ ] Not Started

**Required Actions**:
1. **Add Section 2.2**: "Gingles Framework and Study Scope"
   - Explain three Gingles preconditions:
     1. Sufficiently large and geographically compact minority (THIS PAPER)
     2. Politically cohesive minority (NOT addressed algorithmically)
     3. Racially polarized voting (NOT addressed algorithmically)
   - Clarify this paper addresses only prong 1
   - 1-2 pages

2. **Limitations Section** (6.x - NEW subsection):
   - Title: "Incomplete VRA Framework"
   - "Our analysis addresses only the first Gingles precondition. Full VRA compliance
     requires demonstrating political cohesion and racially polarized voting, which
     are case-specific and cannot be determined algorithmically."
   - Prominent placement (move to 6.1 from current position)

3. **Terminology change throughout**:
   - Replace "VRA compliance" → "geographic feasibility" OR "Gingles prong 1 satisfaction"
   - Abstract, intro, results, discussion, conclusion all need updates
   - Use find-replace carefully to avoid breaking meaning

4. **Discussion 5.3** (Legal Implications):
   - Add paragraph relating algorithmic compactness to legal compactness
   - Address: "Courts consider community of interest, traditional boundaries, not just edge-cut"
   - Justify: "METIS compactness correlates with traditional compactness measures"

---

### P1.4: Sample Size (N=5) Undermines Generalizability Claims
**Reviewers**: All reviewers (Pildes, Duchin primary)
**Effort**: 5-7 days (if expanding sample) OR 1-2 days (if reframing only)
**Status**: [X] COMPLETE (2026-02-08)

**Resolution**: Expanded sample to ALL 43 multi-district states (far exceeding recommended 5-10 additional states)
- 645 total configurations (43 states × 5 weights × 3 thresholds)
- Statistical validation: r=0.78, t=7.39, p<1e-08
- Geographic diversity: All US regions represented
- Updated all sections: abstract, methodology, results, discussion, conclusion, limitations

**Decision Point**: Expand sample or reframe claims?

**Option A: Expand Sample (RECOMMENDED if time allows) ✓ COMPLETED**
- Add 5-10 additional states spanning 30-55% minority
- Target states: North Carolina (NC), Texas (TX), Florida (FL), Virginia (VA), Maryland (MD)
- Prioritize states with existing VRA litigation (NC, TX, FL)
- Estimated: 2 days computation + 2 days analysis + 1 day write-up

**Option B: Reframe as Preliminary Study (FALLBACK)**
1. **Title change**:
   - Current: "The 42% Threshold: Geographic Limits..."
   - Revised: "The 42% Threshold: Preliminary Evidence for Geographic Limits..." OR
   - Add subtitle: "A Five-State Exploratory Analysis"

2. **Throughout paper** (find-replace):
   - "the 42% threshold" → "evidence suggests a ~42% threshold"
   - "geographic reality" → "preliminary findings indicate"
   - "universal threshold" → "empirical pattern requiring validation"

3. **Add confidence interval** (Section 4.x):
   - Bootstrap or jackknife resampling to estimate: "42% threshold (95% CI: 38-46%)"
   - Show sensitivity to individual states (e.g., excluding Alabama)
   - New subsection: "Threshold Robustness Analysis"

4. **Conclusion addition**:
   - "Validation with 15-20 additional states across diverse geographies is essential
     to confirm the 42% threshold's generalizability. Future work should test whether
     this pattern holds in Southwestern states (Hispanic majorities), urban-dominated
     states, and states with multi-racial populations."

**Recommendation**: Pursue Option A if 1 week available for computation; otherwise Option B

---

### P1.5: Geographic Heterogeneity Not Adequately Addressed
**Reviewers**: Rodden (primary), Duchin
**Effort**: 2-3 days
**Status**: [ ] Not Started

**Required Actions**:
1. **Add Section 3.6**: "Geographic Characterization of Study States"
   - Describe each state's spatial structure:
     * Mississippi: Rural Black Belt concentration (linear north-south)
     * Georgia: Major metro (Atlanta) + Black Belt (rural south)
     * Louisiana: Linear river geography (Mississippi Delta concentration)
     * South Carolina: Coastal vs inland divide
     * Alabama: Major metro (Birmingham/Mobile) + Black Belt
   - Explain how geography affects redistricting feasibility
   - 1-2 pages + potentially a map figure

2. **Expand Moran's I Discussion** (Section 4.4):
   - Current: Single Moran's I value per state
   - Add: Distinguish types of clustering:
     * Metropolitan concentration (GA, AL with major cities)
     * Regional concentration (MS Black Belt)
     * Linear/river patterns (LA)
   - Discuss implications: "Metropolitan concentration may enable MM districts at lower
     state percentages due to density; regional concentration requires balancing geographic
     dispersion"

3. **Results Section** (4.x - new subsection):
   - Title: "Geographic Structure and Threshold Variation"
   - Analyze: Do states with metro concentration (GA, AL) behave differently than
     rural states (MS, SC)?
   - Alabama case study: High Moran's I (0.716) from Birmingham/Mobile metros enables
     success at 36.9% minority despite being below 42% threshold

4. **Discussion 5.x** (new subsection):
   - "The 42% threshold represents average across diverse geographies. States with
     major metropolitan areas concentrating minority populations may achieve feasibility
     at lower percentages (37-40%), while rural states with dispersed populations may
     require higher percentages (44-46%)."

---

## Priority 2: Important Issues (SHOULD ADDRESS for Strong Paper)

### P2.1: Add Confidence Intervals and Statistical Significance Tests
**Reviewers**: Stephanopoulos, Duchin
**Effort**: 1-2 days
**Value**: HIGH - Strengthens statistical claims significantly

**Actions**:
- Bootstrap resampling for threshold estimate confidence interval
- Hypothesis test for r=0.88 correlation significance
- Jackknife sensitivity analysis (excluding each state individually)
- Add to Results Section 4.6

---

### P2.2: Address METIS Stochasticity with Multiple Runs
**Reviewers**: Pildes, Stephanopoulos, Duchin
**Effort**: 2-3 days
**Value**: HIGH - Critical for legal applicability

**Actions**:
- Run 10 trials per configuration for Louisiana (borderline case)
- Report success rate with confidence intervals: "Louisiana achieves target in 42.9%
  of configurations; with 10 runs per config, success probability is 38.5% ± 5.2%"
- Addresses legal question: "Does VRA require multiple optimization attempts?"

---

### P2.3: Algorithm-Dependency Analysis
**Reviewers**: Pildes, Stephanopoulos
**Effort**: 2 days
**Value**: MEDIUM - Clarifies scope of findings

**Actions**:
- Explicit comparison: Edge-weighted (42% threshold) vs Multi-constraint (~47% threshold)
- Discuss: Is 42% fundamental geographic truth or edge-weighted-specific?
- Recommendation: "42% threshold applies to current algorithmic state-of-the-art;
  future improvements may lower threshold modestly but cannot eliminate it due to
  proportionality constraints"

---

### P2.4-P2.8: [Other P2 items - see SYNTHESIS.md for full list]

---

## Priority 3: Nice to Have (Polish, Not Blocking)

- P3.1: Multi-year temporal stability (test on 2010 data)
- P3.2: Census resolution impact analysis
- P3.3: 45% vs 50% MM threshold sensitivity
- P3.4: Comparison to existing quantitative standards
- P3.5-P3.12: [See SYNTHESIS.md]

---

## Revision Timeline

### Week 1: Core P1 Issues
- Days 1-2: P1.1 (state vs district framing)
- Days 3-4: P1.2 (proportionality assumption + sensitivity analysis)
- Days 5: P1.3 (Gingles three-prong framework)

### Week 2: Statistical Rigor
- Days 1-2: P1.5 (geographic heterogeneity analysis)
- Days 3-5: P1.4 (expand sample to N=10 OR reframe + confidence intervals)

### Week 3: P2 High-Value Items
- Days 1-2: P2.1 (confidence intervals, hypothesis tests)
- Days 3-4: P2.2 (multiple METIS runs for borderline cases)
- Day 5: P2.3 (algorithm-dependency clarification)

### Week 4: Integration & Polish
- Days 1-2: Integrate all revisions, ensure consistency
- Day 3: Update all figures/tables with new analyses
- Days 4-5: Comprehensive proofreading, check all cross-references

**Total Estimated Effort**: 3-4 weeks (20-25 working days)

---

## Success Criteria for Round 2

**Minimum (Advance to Ready Stage)**:
- All P1 items addressed with clear documentation
- Average score ≥2.5/4 in Round 2 reviews
- No individual scores <2.0/4

**Target (Strong Acceptance)**:
- All P1 items + P2.1-P2.3 addressed
- Average score ≥3.0/4 in Round 2 reviews
- At least 3/5 reviewers score 3/4 or higher

---

## Tracking Completion

Update checkboxes as items completed:
- [ ] P1.1: State vs district-level framing
- [ ] P1.2: Proportionality assumption clarification
- [ ] P1.3: Gingles three-prong engagement
- [X] P1.4: Sample size expansion (N=43 states, 645 configs) - COMPLETE 2026-02-08
- [ ] P1.5: Geographic heterogeneity analysis
- [ ] P2.1: Confidence intervals
- [ ] P2.2: Multiple METIS runs
- [ ] P2.3: Algorithm-dependency

**Completion**: 1/8 critical items (12.5%)

---

## Notes for Authors

**What Survives**: The core 42% threshold finding is robust and valuable—no reviewer questions its empirical validity.

**What Needs Work**: Framing and statistical foundation. The paper over-claims (presents empirical pattern as legal standard) and under-supports (N=5 without confidence intervals).

**Good News**: These are fixable issues requiring reframing and additional analysis, not fundamental methodological flaws.

**Recommendation**: Focus on P1 items first, then high-value P2 items (P2.1-P2.3). The paper can achieve strong acceptance with these revisions.
