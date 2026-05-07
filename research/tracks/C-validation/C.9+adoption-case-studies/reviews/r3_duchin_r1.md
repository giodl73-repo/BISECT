# Review 3 — Reviewer: Moon Duchin (Metric Geometry / Redistricting Reform)
**Paper:** C.9 — From Algorithm to Map: Implementation Case Studies for Three Adoption Pathways
**Round:** 1
**Score:** 3/4

## Summary

A practically useful paper that addresses the "last mile" problem in algorithmic redistricting: how does an algorithm's output become an adopted map? The three case study framework is well-conceived and the workflow table is the paper's most concrete contribution. My concerns are about reproducibility guarantees and the treatment of the adjustment authority.

## Strengths

The three-pathway framework (commission, statute, court-order) correctly identifies the main institutional channels for algorithmic redistricting adoption. The distinction between these pathways — who runs the algorithm, how adjustments are made, what counts as a successful challenge — is analytically useful and well-developed.

The court-order pathway analysis is the paper's strongest section. The argument that the algorithm output should be "presumed constitutional absent a specific showing of constitutional defect" is well-reasoned: the algorithm cannot gerrymander because it cannot see partisan data, and it satisfies the neutral criteria that every court has endorsed. This shifts the burden of proof in a legally significant way that the paper correctly characterizes.

The reproducibility discussion in Section 6.3 ("because the algorithm is deterministic given its inputs and parameters, the adopting authority can reproduce the output at any time") is accurate and important. Deterministic reproducibility is a genuine legal advantage of the algorithmic approach over human-drawn maps, where the mapmaker's mental process cannot be fully reconstructed.

## Weaknesses and Concerns

The reproducibility claim needs qualification. The redist system produces deterministic output given a specific seed, but the paper does not specify whether the adopted map is generated with a specific fixed seed (fully deterministic) or with a ConvergenceSweep across multiple seeds (the best-of-T result). These are different: a fixed-seed run produces a specific map deterministically, but a ConvergenceSweep may explore different seeds on different hardware or operating system environments due to floating-point non-determinism in METIS.

For legal purposes, the adopting authority needs to specify exactly which output is "the output" — not just the algorithm and parameters, but the specific seed used and the platform on which the algorithm was run. The paper should address this practical reproducibility question explicitly. Does the DIA framework specify a canonical platform and a canonical seed? If so, where?

The community-of-interest adjustment authority deserves more critical scrutiny. The paper provides guardrails (geographic justification, documented factual finding) but does not address the boundary of the adjustment authority: how many tracts can be moved under the adjustment authority before the adjusted map is no longer "the algorithm's output" but rather a human-drawn map using the algorithm as a starting point? If the Commission adjusts 1% of tracts, the output is still largely algorithmic. If it adjusts 15%, the algorithmic starting point has been substantially replaced. The paper should establish a quantitative threshold for when adjustments exceed the adjustment authority and require the commission to restart from the algorithm.

The workflow table (Table 1) is useful but the "Public comment" and "Adjustment" rows obscure a timing problem. The paper specifies a "60-day public hearing period" for commission adjustments. But the redistricting timeline (census data available in approximately April after the census year, maps needed for the following election year) puts pressure on this timeline. For the 2020 cycle, maps needed to be ready for candidate filing deadlines in early 2022. A 60-day public comment period plus an adjustment period plus legal challenge time may be incompatible with election administration requirements. The paper should address the timeline constraints for each pathway.

## Minor Issues

- The paper mentions that the VRASection threshold "can be set to reflect the CVAP (Citizen Voting Age Population) percentages required by VRA analysis." This is important: using CVAP rather than total population for VRA purposes is the legally correct standard under *Hayden v. Pataki* and related cases. The paper should cite the relevant VRA case law establishing the CVAP standard.
- The conclusion's claim that "adoption of algorithmic redistricting at scale would likely follow the pattern of any significant institutional reform: early adopters in states with independent commissions" is plausible but speculative. The paper could strengthen this by pointing to specific states that have recently established or reformed redistricting commissions as candidates for early adoption.

## Recommendation

Accept with minor revisions. Clarify the seed/reproducibility question (fixed seed vs. ConvergenceSweep), establish a quantitative threshold for when adjustments exceed the algorithm's starting-plan authority, and address the redistricting timeline constraints.
