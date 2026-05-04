# Review: VRASection: Geographic Alignment Score for Minority Opportunity District Bisection
# Round 2

**Reviewer**: George Karypis (University of Minnesota, METIS developer)
**Expertise**: Graph partitioning, multilevel algorithms, METIS design, parallel graph algorithms, sparse matrix algorithms
**Round**: 2
**Date**: 2026-05-02

## Summary of Revisions

The authors have addressed Issue 1 (decoupled optimization bias) with a new remark
that acknowledges the gap between the compactness-optimal alignment and the achievable
alignment at each ratio, and characterizes the bias as conservative (underselling
minority alignment). Issues 2 and 3 appear to have been addressed via clarifying text,
though the authors have not provided the EC_norm distribution I requested. The legal
issues are outside my expertise and are addressed by the other reviewers.

## Score: 3.5/4

**My score**: 3.5/4 — The decoupled optimization acknowledgment is well-handled and
the conservative-bias framing is correct. The remaining gap is the absence of the
EC_norm distribution confirming the max(·,1) floor is inactive, and the seed stability
data. These are verifiable claims; the paper should verify them.

## What Changed and Whether It Works

### Issue 1 (Decoupled Optimization): Addressed — Good Resolution

The new remark explaining that the alignment score is evaluated on the
compactness-optimal bisection (not the alignment-optimal bisection) is the right
acknowledgment. The key insight — that this makes VRASection *conservative* in the
legally important direction, because the algorithm is less likely to select a
minority-aligned ratio than a joint-optimal version would — is correct and is the
right framing for both the algorithmic and legal contexts.

To see why this is conservative: if the compactness-optimal bisection at ratio 2:5 has
*A* = 0.42, and the alignment-optimal bisection at ratio 2:5 has *A* = 0.55, then the
true Pareto-optimal score at ratio 2:5 is lower (better) than what the paper reports.
VRASection selects 2:5 despite underestimating its alignment quality. This means the
algorithm is biased toward the GeoSection result, not away from it. For Shaw
predominance purposes, this is legally favorable: the algorithm is systematically
less race-responsive than it could be.

The deferral of joint optimization to future work is appropriate. A full Pareto-frontier
exploration would require 2N seeds per ratio (N compactness-minimizing, N
alignment-maximizing), doubling the METIS call budget. That is a paper-scope change,
not a revision item.

### Issue 2 (max(·,1) Floor): Acknowledged, Unverified

The revision appears to have added text noting that the max(·,1) floor is inactive for
the six test states (where EC_norm values are "well above 1"). I cannot confirm this
from the paper's current empirical data, because the paper does not report the
EC_norm distribution across all candidate ratios for all six states. For Alabama,
EC_norm values of 430.1 (1:6) and 448.7 (2:5) confirm the floor is inactive for those
ratios. For the other states, I am taking this on faith.

The request from Round 1 was to report the EC_norm distribution, which would make this
verifiable. That distribution is not in the paper.

### Issue 3 (Seed Count Stability): Acknowledged, Unverified

The revision adds a note that the 2:5 vs. 1:6 score gap for Alabama (322.5 vs. 350.2,
a gap of 27.7 points) provides sufficient margin for stability across seed draws. This
is a reasonable inference — a 27.7-point gap in the selection score is unlikely to be
bridged by seed-to-seed variance — but it remains an inference. The standard deviation
of alignment scores across the 50 seeds at ratio 2:5 is still not reported.

I note the CLI output reports 20 seeds per ratio, while §4.1 states 50 seeds. This
discrepancy is not resolved in the revision. Which is correct?

## Remaining Issues

### Issue A: EC_norm Distribution Not Reported
**Severity**: Medium
The claim that the max(·,1) floor is never active should be verified empirically.
Table 2 should add a column for the EC_norm value at the winning ratio for each state,
or a supplementary table should list EC_norm at all candidate ratios. For any state
with $k \leq 4$ (Mississippi, $k = 4$), the ratio scan covers only 2 candidate ratios
and EC_norm values may be lower.

### Issue B: Seed Count Discrepancy (20 vs. 50)
**Severity**: Medium
The abstract and §4.1 state $N = 50$ seeds per ratio; the CLI output in §4.2 reports
"20 seeds per ratio." This was flagged in Round 1 and remains unresolved. The correct
value should be stated consistently throughout the paper.

### Issue C: Alignment Score Variance Across Seeds
**Severity**: Low-medium
The seed stability argument (27.7-point score gap) is plausible but the underlying
claim — that alignment scores are stable across seed draws — is not empirically
demonstrated. A table showing the mean and standard deviation of the alignment score
across the 50 (or 20) seeds for Alabama's 2:5 ratio would close this gap.

## Minor Notes

- The balance tolerance propagation question (Issue 3 minor, Round 1) is not addressed.
  The recursion calls GeoSection with `ufactor = 1 + δ/k` where k is the sub-problem
  size. This is internally consistent with GeoSection's convention; the paper should
  state this explicitly.
- The clarification on Algorithm 1 line 1 (k=1 base case and the recursion calling
  GeoSection on i*=1 sub-problems) has not been added. This is a minor pseudocode
  documentation issue.
- The normalization justification for √min(i, k-i) is now present in §3.2.
  This closes the minor notation issue from Round 1.

## Questions for Authors

1. Is the max(·,1) floor ever active in the six test states? Please report EC_norm
   at all candidate ratios for Mississippi ($k = 4$, the smallest $k$ in the
   evaluation set) to verify.

2. What is the minimum $w_\text{vra}$ at which the Alabama ratio shifts from 1:6 to
   2:5? This calibration would support the default parameter choice.

3. Which seed count is correct: 20 or 50? The CLI output and the §4.1 parameter
   description are inconsistent.

## Recommendation

Accept with minor revisions. The decoupled optimization acknowledgment is well-handled
and the conservative-bias framing is correct. Resolve the seed count discrepancy,
add the EC_norm distribution for the six test states to verify the floor is inactive,
and add alignment score variance data for Alabama. These are straightforward additions
that would complete the empirical characterization the paper claims to provide.
