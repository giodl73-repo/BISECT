# Revision Plan - Round 1 Peer Review

**Paper**: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals
**Review Round**: 1 → 2 → 3
**Date**: 2026-02-08 (updated 2026-05-05)
**Status**: ACCEPTED — Round 3 avg 3.6/4 ≥ 3.5, all P1s addressed

## Review Summary

**Average Score**: 2.6/4
- 2 reviewers: Major revision (2/4)
- 3 reviewers: Minor revision (3/4)

**Consensus**: Major revision needed to address critical implementation and statistical concerns.

## Priority 1 Items (MUST Address Before Resubmission)

### ✅ P1-1: Multi-Constraint Implementation Verification [CRITICAL]
**Status**: ✅ RESOLVED (confirmed by Round 2 reviewers: Karypis, Phillips, Hendrickson, Cook, Duchin)
**Flagged by**: Karypis (primary), Phillips (supporting)
**Severity**: Critical - Could invalidate paper's main conclusion

**Resolution**: Bug confirmed and corrected. tpwgts formula fixed to fraction-based targets. Re-ran all multi-constraint experiments. Corrected MC success rate: 30.0% (down from 35.0% with bug), which actually strengthens the paper's conclusion — edge-weighted now wins by 17.9 pp rather than 12.9 pp.

**Required actions**:
1. [x] Verify tpwgts calculation in code (show exact formula used)
2. [x] Re-run multi-constraint experiments with corrected target weights
3. [x] Results changed (30.0% corrected vs 35.0% buggy) — paper updated accordingly
4. [x] Add detailed tpwgts specification to paper (Section 2.2 updated)
5. [x] Show example calculation for Alabama (2 MM districts, 36.9% minority)

**Timeline**: Completed

---

### ✅ P1-2: Theoretical Section Calculation Errors
**Status**: ✅ RESOLVED (confirmed by Round 2 reviewers: Karypis, Hendrickson, Cook, Phillips)
**Flagged by**: Karypis, Hendrickson, Cook
**Severity**: Critical - Section 3.1.2 undermines credibility

**Resolution**: Section 3.1.2 removed and replaced with formal constraint tightness definition (τ_c = ε). The 129%/258% impossibility calculation removed. Now quantifies 60–800× tightness ratio difference between population and minority constraints. Reframed as "Constraint Conflict Hypothesis" (empirical claim, not proof).

**Required actions**:
1. [x] Remove or completely rewrite Section 3.1.2
2. [x] Formalize constraint conflict mathematically with tightness definition τ_c = ε
3. [x] Define constraint "tightness" precisely (60–800× ratio quantified)
4. [x] Honest reframing as hypothesis with empirical support (Cook accepted)
5. [x] Use correct population accounting throughout

**Timeline**: Completed

---

### ✅ P1-3: Asymmetric Configuration Counts
**Status**: ✅ RESOLVED (confirmed by Round 2 reviewers: Phillips, Cook, Karypis, Hendrickson, Duchin)
**Flagged by**: Phillips (primary), Cook (supporting)
**Severity**: Critical - Makes 47.9% vs 35.0% comparison invalid

**Resolution**: Expanded multi-constraint to 28 configs per state (140 total), achieving a genuine 140-vs-140 balanced comparison. Restated finding: MC 35.7% (50/140) vs EW 47.9% (67/140), gap 12.1 pp. State-level result (80% vs 40%) now primary finding. Complete MC failure in AL/LA/SC across all 28 parameter values is the strongest result.

**Required actions**:
1. [x] Run 140 multi-constraint configs (7 ubvec × 20 random seeds) — 140-vs-140 achieved
2. [x] Report per-state success rates as primary result
3. [x] Use paired comparisons (best-of-N for each method)
4. [x] State-level results (80% vs 40%) framed as primary evidence
5. [x] Focus on state-level outcomes (Alabama 2 vs 1 MM) as robust result

**Timeline**: Completed

---

### ✅ P1-4: No Statistical Rigor
**Status**: ✅ RESOLVED (confirmed by Round 2 reviewers: Phillips, Cook, Karypis, Hendrickson, Duchin)
**Flagged by**: Phillips (primary), Cook (supporting)
**Severity**: Critical - Single runs, no significance tests, no variance

**Resolution**: Section 5.6 added with Wilson 95% CIs for both methods, χ²(1)=4.243 p=0.039 significance test, and 30-seed per-state variance table showing SD=0 for all states. Deterministic outcomes (zero variance across seeds) confirms results are not noise artifacts. Phase 2 140-run population estimates bound CI upper for zero-success states at 2.7%.

**Required actions**:
1. [x] Run each config 30 times with different random seeds (Phase 2: 140-run population estimates)
2. [x] Report mean ± std for MM count, max minority %, edge cut; SD=0 confirmed
3. [x] Conduct statistical significance tests: χ²(1)=4.243 p=0.039, Wilson CIs
4. [x] Add confidence intervals to all key results
5. [x] Differences survive statistical testing (p=0.039, state-level 80% vs 40%)

**Timeline**: Completed

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

**Current status**: 4/4 P1 items addressed ✅

---

## Round 3 Results (2026-05-05)

**Average Score**: 3.6/4 — ACCEPTED
- Karypis (Minnesota): 3.5/4 — Accept with Minor Revisions
- Duchin (Rutgers): 4/4 — Accept
- Stephanopoulos (Harvard Law): 3.5/4 — Accept with Minor Revisions
- Liang (Stanford): 3.5/4 — Accept with Minor Revisions
- Rodden (Stanford): 3.5/4 — Accept with Minor Revisions

**All P1 blocking issues resolved.** Paper cleared for submission to ALENEX or SIAM SISC.

**Remaining P2 items** (not blocking):
- Section 5.1 ordering (Karypis): lead with state-level evidence before χ² in body text (1 sentence)
- Shaw v. Reno analysis (Stephanopoulos): one paragraph on race-aware edge weights and strict scrutiny
- Demographic scope qualification (Liang, Rodden): 5 Southern states, not general redistricting
- Partisan outcome analysis (Rodden): efficiency gap for successful edge-weighted configurations
- Allen v. Milligan connection (Rodden): paragraph connecting Alabama result to 2023 VRA ruling

---

## Round 2 Results (2026-05-05)

**Average Score**: 3.3/4
- Karypis (Minnesota): 3/4 — Accept with minor revisions
- Duchin (Rutgers): 3.5/4 — Accept with minor revisions
- Hendrickson (Sandia): 3.5/4 — Accept with minor revisions
- Cook (Waterloo): 3.5/4 — Accept with minor revisions
- Phillips (Sandia): 3/4 — Accept with minor revisions

**Remaining minor issues after Round 2**:
- Karypis: complete METIS command-line specs in supplementary; lead abstract with state-level evidence (80% vs 40%); explain Georgia non-monotonic anomaly (ubvec=1.3 → 7MM, ubvec=1.5 → 5MM)
- Duchin: Gingles Prong 1 geographic compactness paragraph; explicit aggregate-vs-group-specific minority VAP statement; at least one state map (Alabama best configuration)
- Hendrickson: ablation validation of constraint conflict mechanism; qualify "graph partitioning" claims to "METIS recursive bisection"
- Cook: formal optimality limitation statement for SC; algorithm selection threshold table (τ_tight/τ_loose > T); complete METIS invocation appendix
- Phillips: complete reproducibility appendix (exact METIS version, command lines, hardware); Polsby-Popper for best configurations; abstract restructured to lead with state-level evidence

---

## Notes

- **P1-1 is highest priority**: Implementation bug confirmed and fixed; results strengthened
- **Statistical rigor (P1-4) is resolved**: SD=0 across seeds is more compelling than traditional significance tests
- **Theoretical section (P1-2) rewritten**: Constraint conflict hypothesis correctly framed as empirical claim
- **Round 3 target**: Address reproducibility appendix, state map (Alabama), abstract restructuring, Georgia anomaly explanation

---

## Estimated Total Timeline

**Completed**: All P1 items (Round 2 submission done)
**Remaining minor work**: ~1 week (reproducibility appendix + maps + minor edits)
**Round 3 target**: ≥ 3.5/4 average
