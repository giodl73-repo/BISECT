---
skill: discover-hypothesis
topic: ri-population-band
date: 2026-09-10
confidence: 55
verdict: OPEN
---

# Hypothesis, frozen before the new run

Claim (technical): In the 2020 RI five-tree candidate family, at least one of
the declared population bands will admit multiple physical cuts and a positive
county penalty will change the boundary-minimizing physical winner set.

Falsification: No band has multiple eligible cuts, or every tested positive
penalty retains the alpha-zero winning set in every nonempty band.
Confidence: 55/100, a modest technical hunch, not a probability estimate.

Prior: The user expects county preservation to be tunable. Existing diagnostics
show strict population minimization removes all physical choice in this
five-root RI search. That is evidence about filtering, not about population
bands or final county-fragment improvements.

Evidence thresholds: two or more eligible physical cuts establish selection
opportunity; a changed winner set at any positive alpha establishes score
response; neither establishes fewer split counties. Bias risk: dismissing an
unchanged winner as an insufficiently broad band after inspecting results.
These bands will not be widened within this experiment.

## Two tests in this round

1. Enumerate every tree-edge cut under five fixed roots. If no band has at
   least two physical cuts, the candidate-opportunity component is falsified.
2. Score all eligible cuts at alpha 0, 0.5, 1, 2, 4, 8. If all nonempty bands
   retain their alpha-zero winner set, the weight-response component is falsified.

Both tests run now on local hashed inputs. Stop the investigation sequence
after this bounded experiment if either component fails; retain the complete
experiment table. Surviving both tests would leave unresolved transfer to other
States, completed trees, county fragments, certification, and legal requirements.

## Fixed design and scope (15 checks)

1. RI 2020 only; no confirmation cohort access.
2. Source context hash must match the preceding root diagnostic.
3. Five roots use floor(j*(n-1)/4), j=0..4, in sorted GEOID order.
4. DFS uses sorted neighbors and marks parent on push.
5. All proper subtrees are eligible for population testing, not just minima.
6. RI has two seats; each child target is parent population divided by two.
7. Relative child-deviation bands are 10, 100 and 1000 parts per million.
8. Eligibility uses integer arithmetic: abs(2*p-P)*1000000 <= band*P.
9. Both children have the same absolute deviation in this two-seat case.
10. Alpha is additive; scoring uses exact fractions without rounding.
11. Bridges retain their original weights; all retained units count.
12. Zero-population units remain in physical identity and county counts.
13. Complementary cuts and duplicate trees are deduplicated by physical labels.
14. All tied winning physical cuts are reported; no post-hoc winner selection.
15. Empty bands are reported as infeasible in this candidate family, not globally.

Investigation sequence: local enumeration and controlled rescoring, followed by
interpretation against the two earlier diagnostics. Public web evidence cannot
answer this code/input-specific measurement; no legal conclusion is attempted.
Further causal/coherence/synthesis work is deferred until this test completes.

Sharpening: scope is RI/five trees, thresholds are explicit winner-set changes,
and the null is unchanged winner sets across all tested alphas within each band.
This hypothesis does not assert a reduction in county fragments.
