> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied.

---

# Review Round 3: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: Percy Liang (Stanford University)
**Expertise**: Machine learning, AI evaluation, algorithmic systems, responsible AI, benchmarking
**Round**: 3 (First review)
**Date**: 2026-05-05

## Summary

First review of this paper. I focus on evaluation methodology, experimental design, and the reliability of the algorithm comparison claims.

## Strengths

**1. The experimental design is commendably fair and transparent.**

The discovery and correction of the tpwgts implementation bug is an exemplary case of scientific integrity. Most papers would not voluntarily reveal an implementation error, especially one that disadvantaged the baseline (making the comparison appear less favorable to the paper's hypothesis). Here, correcting the bug actually strengthened the conclusions — the corrected multi-constraint performance (30.0%) is worse than the buggy version (35.0%), widening the edge-weighted advantage. This demonstrates that the paper's results are not dependent on a favorable implementation choice; if anything, the uncorrected implementation was giving multi-constraint too much credit.

**2. The zero-variance finding across 30 seeds per state is methodologically significant.**

Demonstrating SD=0 for all five test states across 30 seeds establishes that the state-level outcomes are deterministic: a given (algorithm, parameter) combination produces the same outcome every time. This is more meaningful than a traditional significance test because it shows the effect is categorical, not continuous — multi-constraint fails in Alabama not 95% of the time but 100% of the time. For practitioners, this means no seed choice can rescue a failing configuration, which is a strong and actionable result.

**3. The balanced design (140-vs-140) resolves the core methodological concern.**

The original paper compared 140 edge-weighted configs against 4 multi-constraint configs, which was unfair because more configurations increase the probability of finding at least one success. The 140-vs-140 balanced design in the revision eliminates this confound. The final comparison is apples-to-apples: both methods get the same number of parameter-space explorations.

## Weaknesses

**1. The experiment uses only 5 states — generalization is unclear.**

The paper's headline result (multi-constraint fails in 3 of 5 states) is based on a very small sample of states. It is entirely possible that the 3/5 failure rate is specific to the demographic structure of the chosen states (Alabama, Georgia, Louisiana, Mississippi, South Carolina — all Southern states with Black minority populations in the 27-50% range). States with different demographic patterns (Latino majority in Southwest, Asian concentration in Hawaii, urban Black population in Northern states) may show different results. The paper's title claims results about "asymmetric redistricting goals" in general, but the evidence is from one demographic cluster.

**2. The constraint conflict mechanism is unvalidated by ablation.**

The paper argues that constraint conflict (tight population balance at ±0.5% dominating loose minority tolerance at ±10-1000%) explains multi-constraint's failure. This is a reasonable hypothesis, but the paper does not perform the ablation that would validate it: running multi-constraint with both constraints tight (e.g., ubvec=[1.005, 1.005]) or both loose, and showing that symmetric tightness enables success that asymmetric tightness does not. Without this ablation, the mechanism could equally well be explained by METIS's internal heuristics preferring certain constraint structures, or by the specific coarsening decisions made at the graph scale of the test states.

**3. The evaluation could benefit from a power analysis.**

With n=140 configurations per method and a 12.1 percentage point gap, the statistical power for detecting the observed effect is high. However, the paper does not report the minimum detectable effect size for the chosen design, which would help readers calibrate how large the true effect needs to be for the comparison to be credible. A brief power analysis note in the statistical methods section would strengthen the methodological transparency.

**4. No error analysis for the Alabama map.**

The Alabama schematic map is appreciated as a visualization, but it is generated from tract centroids, not polygon boundaries. The caption should acknowledge that the centroid-based representation may differ from the actual district shape in ways that matter geographically (e.g., centroids near water boundaries may appear inside adjacent tracts). This is a standard geographic data limitation that should be documented.

## P1 Items

None blocking.

## P2 Items

- **Geographic/demographic scope qualification**: Add one paragraph acknowledging that the 5-state experiment covers only Southern states with Black minority populations and that generalization to other demographic structures (Latino, Asian, Northern urban Black) is unvalidated.

- **Mechanism ablation**: Add a paragraph in the Discussion acknowledging that the constraint conflict mechanism is supported by evidence but not causally validated by ablation experiments (symmetric tightness vs. asymmetric tightness). Recommend this as future work.

- **Map caption clarification**: Acknowledge centroid-based approximation and its geographic limitations.

## Score

**Score: 3.5/4 — Accept with Minor Revisions**

The experimental design is sound after the balanced comparison correction. The zero-variance finding is methodologically strong. The geographic scope limitation (5 Southern states) is the most significant generalization concern. With a paragraph acknowledging demographic scope limitations and mechanism validation gaps, the paper would be a clean contribution to the applied algorithms literature.

**Recommendation**: Accept with minor revisions. Geographic scope qualification is the priority addition.
