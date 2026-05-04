# Review: R-1 Percy Liang
**Paper**: The Solution Space of Minimum-Edge-Cut Redistricting: Seed Sensitivity and Partisan Variance
**Date**: 2026-05-01
**Score**: 2.5 / 4

---

## Summary

A methodologically interesting paper that combines algorithmic analysis with empirical political science. The evaluation design is careful in places but has reproducibility gaps and missing baselines that prevent confident interpretation of the key claims. The central empirical finding — MEC convergence for PA — rests on a single state and is contradicted by slower convergence in other states.

---

## Strong Points

1. **SHA-256 verification of partition uniqueness** is the right approach and produces an unambiguous result: 1,100 seeds, 1,100 distinct plans for PA. This is exactly the kind of falsifiable, reproducible claim that I want to see.

2. **The 50-state proportionality sweep** (Task 1) is a clean, complete empirical exercise. The state-level CSV with proper available/unavailable flags (AK, HI excluded for missing data) shows methodological care.

3. **The WI three-way comparison** (MEC 2D, CompactBisect 3D, proportional 4D) is the paper's strongest empirical result. Clear, interpretable, and directly proves the paper's hierarchy claim.

---

## Concerns and Weaknesses

### P1: Critical

**P1.1 — PA is one state. The convergence claim should not generalize without cross-state evidence.**

The introduction claims "the seed is irrelevant in a precisely verifiable sense" but this claim is demonstrated primarily for PA. The cross-state evidence (§4 evaluation) shows dramatically different convergence profiles:
- PA: last improvement at seed 181 (of 1,100)
- GA: last improvement at seed 190 (of 200) — 95% of seeds after last improvement is 10, NOT convergent
- MI: last improvement at seed 47 (of 76) — only 29 seeds after last improvement, clearly not converged

The claim that the seed is irrelevant would require demonstrating convergence in ALL states. GA and MI data directly contradict the convergence claim when only 200 seeds were run. The paper should either: (a) run more seeds for GA/MI until convergence is demonstrated, or (b) temper the headline claim to "seed is irrelevant for PA and other converged states" and describe the open question for states where convergence is not yet established.

**P1.2 — CompactBisect baseline comparison uses a debug build, not a release build.**

The paper reports CompactBisect results (WI: 3D/5R, NC: 5D/9R, PA: 7D/10R) from runs using the debug binary (the text says "50 seeds/level, debug binary"). Debug builds of METIS-based code can produce different results from optimized builds due to floating-point ordering. The results should be reproduced with the release binary before they are publishable findings.

### P2: Significant

**P2.1 — The seed sweep does not report confidence intervals.**

The introduction lists "$\sim$1\% of the observed best" as the certificate bound, but this is an informal claim. For PA, the claim is that the running minimum (2,441km) is within 1% of the true global minimum. But 1% of 2,441km = 24.4km. What is the width of the 95% CI from the fitted GEV model? This should be reported explicitly. For states where convergence is slower, the CI may span multiple seat boundaries.

**P2.2 — The empirical calibration in §3.2 is outdated relative to §4.**

The convergence subsection (§3.2) reports PA calibration based on 100 seeds (improvements at seeds {1,2,3,5,12,18,34}, last at seed 34). But §4 reports the full 1,100-seed result (last improvement at seed 181). These should be consistent. The text in §3.2 should be updated to reflect the 1,100-seed result, and the "5,000 seeds needed" estimate should be revised.

**P2.3 — MEC protocol is compared against CompactBisect but not against random selection.**

The paper shows MEC > random (implicitly) and CompactBisect ≥ MEC for some states, but does not show a baseline of "what does a randomly selected plan look like?" PA's 1,100 random seeds have a distribution (6D: 9%, 7D: 54%, 8D: 35%, 9D: 2%). The MEC plan is 8D — this IS better than the modal plan (7D) for PA. But is it better than the median? Better than a random draw? These comparisons would make the MEC protocol's value clearer.

**P2.4 — Texas failure rate (4%) is not analyzed.**

The evaluation mentions that 4 of 100 TX seeds (seeds 1, 2, 6, 16) produced population balance violations. This 4% failure rate is not explained. Are these seeds structurally different? Does the failure rate increase with district count? For a statutory algorithm, a 4% failure rate means roughly 2 of the 50 states would produce invalid plans per run — significant for legal purposes.

### P3: Minor

**P3.1 —** AK and HI are excluded from the proportionality sweep for missing election data, but this is only noted in the code (available=false). The paper should explicitly state these exclusions in §4 with a note on why (election data absent from presidential_by_tract.csv) and what the best available source would be.

**P3.2 —** The PP computation in §3 uses WGS84 (unprojected) coordinates, as noted in the crs_note field. The paper should state this explicitly and bound the resulting error (PP in WGS84 is compressed for east-west districts at high latitudes). For states like ND, ID, OR with significant east-west extent, the WGS84 PP values may be systematically off by 5-15%.

---

## Verdict

The paper needs PA's convergence claim to be supported by additional states before the headline claim holds. The immediate fix is to either (a) run more seeds for GA and MI, or (b) narrow the headline to PA. The CompactBisect comparison needs to be re-run with the release binary. I would accept this paper after major revisions addressing P1.1 and P1.2.
