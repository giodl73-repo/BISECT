# Review: Cross-Census Temporal Stability
## Reviewer: Dr. Moon Duchin (Tufts University)
**Expertise**: Redistricting, gerrymandering, computational geometry
**Date**: 2026-02-08
**Score**: 3.0/4.0 (Accept)

---

## Overall Assessment

This paper tackles an important practical question: which algorithmic approach provides better continuity when redistricting must be redone every decade? The answer—that hierarchical recursive bisection offers marginally better stability—is useful for practitioners, though the effect size is smaller than I'd hoped.

**Strengths**:
1. **Practical relevance**: Real question faced by redistricting commissions
2. **Good state selection**: Southern states with VRA considerations
3. **Transparent reporting**: Authors don't hide the modest 1.1% effect
4. **Reproducible**: All data and code provided

**Concerns**:
1. **VRA implications underdeveloped**: How does stability affect minority representation?
2. **No normative framework**: Is stability always desirable?
3. **Missing political context**: Real redistricting involves partisan considerations
4. **Limited demographic analysis**: Only total minority population considered

---

## Major Issues (P1 - Blocking)

### P1.1: VRA Analysis is Superficial
**Issue**: Paper mentions "edge weighting (5x at 40% minority threshold)" but doesn't analyze how temporal stability interacts with VRA compliance. Critical questions unanswered:

1. Do stable districts maintain VRA compliance across decades?
2. If 2010 districts were VRA-compliant, are 2020 districts still compliant?
3. Does recursive bisection's stability help or hurt minority representation?

**Fix Required**: Add Section 4.4 "VRA Compliance Across Time":
- Show majority-minority district counts for 2010 vs 2020
- Analyze whether districts that stayed stable maintained MM status
- Discuss tradeoff: stability might lock in inadequate representation

**Evidence**: FINDINGS_SUMMARY.md shows Georgia went from 1 MM district (2010) to 7-8 MM districts (2020). This demographic shift requires boundary changes. Is stability even desirable here?

---

## Major Issues (P2 - Important)

### P2.1: Normative Framework Missing
**Issue**: Paper assumes stability is inherently good ("communities of interest see less disruption") without examining when disruption might be necessary or beneficial.

**Examples where disruption is good**:
- Demographic shifts require new MM districts (Georgia: 1 MM → 8 MM)
- Population growth concentrates in specific regions
- Correcting historical underrepresentation

**Recommendation**: Add Section 6.1 "When Stability Matters":
- Stability valuable when demographics stable
- Disruption necessary when demographics shift significantly
- Framework for balancing stability vs representational equity

---

### P2.2: Partisan Considerations Absent
**Issue**: Real redistricting is intensely partisan. Paper uses purely geographic algorithm with no partisan data. How do findings translate to partisan environment?

**Questions**:
- Do partisan mapmakers prefer stable or disruptive approaches?
- Would partisan actors exploit hierarchical structure predictability?
- Does stability advantage disappear under partisan manipulation?

**Recommendation**: Add Discussion section acknowledging partisan context and suggesting future work on partisan stability.

---

### P2.3: Community of Interest Metric Too Simple
**Issue**: Paper claims "communities of interest see 14 percentage points less disruption (22% vs 36% counties affected)" but uses county splits as proxy for communities of interest.

**Problems**:
- Counties are administrative units, not communities
- Many communities span multiple counties (metropolitan areas)
- Some counties contain multiple distinct communities

**Recommendation**: Either:
1. Use actual COI definitions (cities, metro areas, tribal lands)
2. Or explicitly label this as "county continuity" not COI disruption

---

### P2.4: Only Five States - Generalization Unclear
**Issue**: All five states are southern with specific demographic patterns:
- High Black minority populations (26-46%)
- Relatively low Hispanic populations (except Louisiana)
- Similar geographic characteristics (rural + urban patterns)

**Questions**:
- Does this generalize to Western states with different demographics?
- What about states with very dispersed minority populations?
- Urban-only states (New Jersey, Rhode Island)?

**Recommendation**: Add limitations section discussing generalization bounds.

---

## Minor Issues (P3 - Nice to Have)

### P3.1: Comparison to Actual 2010-2020 Redistricting
**Missing**: How stable were actual congressional maps?
**Would strengthen**: Compare algorithmic stability to real plans for Alabama, Georgia, etc.

---

### P3.2: Temporal Predictions Not Testable Until 2030
**Issue**: Paper claims recursive bisection will make 2030 redistricting easier.
**Problem**: Can't verify this for 4 more years.

**Recommendation**: Add 2000 data to show 2000→2010 stability, making the temporal claim more credible.

---

### P3.3: No Discussion of Court Challenges
**Issue**: District stability matters because court challenges to new maps are common. Stable districts might face less legal scrutiny.

**Recommendation**: Add paragraph discussing litigation implications of stable vs disruptive redistricting.

---

## Detailed Comments

### Methodology Strengths
✅ **Good**: Edge weighting approach (5x at 40%) aligns with VRA goals
✅ **Good**: Common tract identification handles census boundary changes
✅ **Good**: District mapping algorithm (2010 → 2020) is well-designed

### Methodology Concerns
⚠️ **Problem**: "Best district mapping" (2010 D₁ → 2020 D₂) assumes districts should correspond. But if demographics shift, new district 2020 D₂ might intentionally represent different population than 2010 D₁.

This assumption biases toward stability. Consider alternative metric: "population whose district changed in character" rather than "changed in number."

### Results Interpretation

**Tract Reassignment Rate** (71.2% for both methods):
- This seems extremely high and weakens the stability claim
- If 71% of tracts change districts, is there really any stability?
- The 1.1% population disruption difference is meaningful only if baseline isn't near-total disruption

**State-by-State Variation**:
- Recursive wins 4/5 states but only by 0.3-1.9%
- Georgia is anomaly (n-way slightly better)
- This suggests state-specific factors matter more than method

---

## Recommendations for Revision

### Tier 1 (P1 - Must Fix)
1. **Add VRA compliance analysis**: Show how stability affects minority representation
2. **Add normative framework**: Discuss when stability is/isn't desirable

### Tier 2 (P2 - Strongly Recommended)
1. Acknowledge partisan context and limitations
2. Revise community of interest claims (use real COIs or relabel as county continuity)
3. Add limitations section on generalization
4. Explain why 71% baseline reassignment isn't near-total disruption

### Tier 3 (P3 - Would Strengthen)
1. Compare to actual 2010-2020 congressional maps
2. Add 2000 data for three-census longitudinal view
3. Discuss litigation implications

---

## Recommendation

**Score: 3.0/4.0 (Accept)**

This paper makes a useful contribution to redistricting methodology by quantifying temporal stability differences. The finding that hierarchical methods provide modest stability advantages is valuable for practitioners, even though the effect is smaller than one might hope.

The key limitation is insufficient engagement with VRA and representational equity considerations (P1.1, P2.1). Redistricting isn't just about geographic optimization—it's about ensuring fair representation. The paper needs to address whether stability helps or hinders this goal.

With VRA analysis and normative framework added, this becomes a solid contribution. The modest effect size (1.1%) is appropriately characterized, and the transparency about limitations is commendable.

**Venue Fit**: ACM-KDD is reasonable but consider political science venues (APSR, AJPS) given the policy implications. The algorithmic focus fits KDD, but the societal impact is the real contribution.

---

## Questions for Author Rebuttal

1. **VRA interaction**: Does recursive bisection's stability help or hurt minority representation in changing demographics?

2. **Baseline interpretation**: Is 71% tract reassignment considered "stable" or "unstable" compared to typical redistricting?

3. **Generalization**: Would findings hold in Western states with dispersed Hispanic populations?

4. **Practical adoption**: Have you discussed findings with actual redistricting commissions? Would they use this?

---

## Ethical Considerations

This research could influence real redistricting decisions affecting millions of people. Authors should add:
- Statement on potential partisan misuse
- Discussion of representational equity tradeoffs
- Acknowledgment that pure algorithmic redistricting is not currently used in practice

**Overall**: Good contribution with real-world relevance. Fix VRA/normative issues and this is publishable.
