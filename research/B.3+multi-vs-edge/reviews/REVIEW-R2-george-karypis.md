# Review Round 2: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: Graph partitioning algorithms, METIS development, multi-constraint optimization
**Round**: 2 (Revision Review)
**Date**: 2026-05-02

## Summary of Revisions Received

The authors have substantially revised the paper in response to Round 1 concerns. I specifically raised four issues: (1) incorrect multi-constraint `tpwgts` specification; (2) calculation errors in Section 3.1.2; (3) asymmetric configuration counts; and (4) lack of statistical rigor. I evaluate each in turn.

## P1 Resolution Assessment

### P1-1: Multi-Constraint Implementation Bug — RESOLVED, STRENGTHENS PAPER

This was my primary concern in Round 1. I am satisfied by the resolution. The authors confirmed the `tpwgts` bug, corrected the formula, and re-ran all multi-constraint experiments. The corrected implementation produces 30.0% success rate (down from 35.0% with the bug), which actually *strengthens* the paper's central conclusion: edge-weighted now wins by 17.9 percentage points rather than 12.9. This is the ideal outcome — implementation correction validates the hypothesis rather than refuting it.

The fact that fixing the bug further widened the performance gap is a meaningful result in itself. It confirms that the original (buggy) implementation was inadvertently *helping* multi-constraint, and the corrected version exposes the true limitation more clearly. I am satisfied with this resolution.

### P1-2: Section 3.1.2 Calculation Errors — RESOLVED

The confused 129%/258% impossibility calculation has been removed and replaced with a formal constraint tightness definition (τ_c = ε). The quantification of the 60–800× tightness ratio difference is appropriate and the section is now coherent. This is what I asked for.

### P1-3: Asymmetric Configuration Counts — RESOLVED

The authors expanded multi-constraint from 4 to 28 configs per state (140 total), achieving a genuine 140-vs-140 balanced comparison. The restated finding is: MC 35.7% (50/140) vs EW 47.9% (67/140), gap 12.1 pp. More important to me is the state-dependency finding — MC completely fails in AL/LA/SC across all 28 parameter values. This is a strong and fair result.

### P1-4: Statistical Rigor — RESOLVED (substantially)

Section 5.6 adds Wilson 95% CI for both methods, a χ²(1)=4.243 p=0.039 significance test, and — critically — a 30-seed per-state variance table showing SD=0 for all states. The deterministic nature of outcomes (zero variance across seeds) is actually more useful than I anticipated: it means the configuration-level χ² test is appropriate rather than potentially confounded by seed noise. The Phase 2 140-run population estimates tighten the CI upper bound for zero-success states to 2.7%, which is compelling.

## Remaining Concerns

### Remaining P2-1: METIS Implementation Details (Partially Addressed)

I see some improved specificity in the experimental description, but the full parameter specifications I requested — complete command lines, explicit `-ncuts`, `-contig`, and `-ufactor` values — are still not provided. For a result claiming METIS-specific behavior, the reader should be able to replicate the exact invocations. I want supplementary material with the complete command-line template before final acceptance.

### New Concern: p=0.039 is Marginal

The χ² test is significant at α=0.05 but not at α=0.01. For a paper making a strong algorithmic claim ("outperforms"), this is a somewhat weak margin. The authors should note this explicitly and point readers to the stronger evidence: the zero-variance per-state results, which are far more compelling than the marginal config-level p-value. The state-level success rate (80% vs 40%) needs no significance test to be convincing — the prior for "both methods fail in exactly 3/5 states at random" is clearly not the right null. I would like the paper to lead with the state-level evidence more forcefully.

### Remaining P2-6: Mechanism Validation Still Absent

The ablation studies I suggested — testing ubvec=[1.005, 1.005] to rule out Hypothesis C (edge-weighting succeeds due to METIS objective function clustering) — were not performed. I understand the experimental burden, but the constraint conflict mechanism remains empirically supported rather than mechanistically proven. This is acceptable for an empirical paper, but the claims in Section 3 should be calibrated accordingly.

### Remaining P3-1: Georgia Anomaly

The Georgia non-monotonic ubvec result (7 MM at 1.3, 5 MM at 1.5) still lacks a satisfying explanation. One sentence of speculation is not enough.

## Score and Recommendation

**Score: 3/4 — Accept with Minor Revisions**

The revision addresses my two blocking concerns: the implementation bug and the asymmetric comparison. The statistical additions are well-executed. The paper now presents a credible and fair empirical case for edge-weighted superiority. Remaining issues — complete METIS command lines, more forceful framing of state-level evidence, Georgia anomaly explanation — are minor in the context of a substantially improved paper.

I am prepared to recommend acceptance once (1) supplementary material provides complete METIS invocation details, and (2) the abstract and Section 5.1 are rewritten to lead with state-level results (80% vs 40%) rather than the marginal config-level p-value.

**Verdict**: Accept with minor revisions.
