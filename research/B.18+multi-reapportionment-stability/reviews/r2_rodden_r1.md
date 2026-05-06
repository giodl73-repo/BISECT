# Review 2 — Reviewer: Jonathan Rodden (Political Geography / Geographic Sorting)
**Paper:** B.18 — Redistricting Stability Across Reapportionment Cycles
**Round:** 1
**Score:** 2/4

## Summary

This paper makes a technically interesting contribution on the algorithmic structure of reapportionment-induced disruption, but it substantially underweights the political geography implications of its main finding. The claim that "reapportionment disruption is less than MCMC mixing" is technically true but misleading as a statement about political stability. The paper needs to address what the boundary disruptions mean for constituent representation and electoral geography, not just for algorithm structure.

## Strengths

The prime factorization framework for predicting boundary disruption is original and technically sound. The observation that prime-to-prime or composite-to-composite transitions produce less disruption than composite-to-prime or prime-to-composite transitions is a genuine insight that prior work on redistricting algorithms has not explicitly analyzed. This is a contribution worth preserving.

The comparison to political redistricting disruption (35–55% of tracts vs. ≤23% algorithmically) is the paper's most politically significant finding. If the worst-case algorithmic reapportionment (Texas, d_Ham = 0.23) is less disruptive than typical political redistricting, this is a strong argument for the DIA approach.

## Weaknesses and Concerns

The paper is almost entirely structural — it analyzes bisection trees and Hamming distances — and almost entirely silent on the political geography of where the boundary disruptions occur. This is a significant gap.

For the Texas case, the paper notes that the composite-to-prime transition "most affects the boundary between the two halves of the 2-level tree (the 'spine' of the current 38-district map)" and "districts near the state's centroid." In Texas politics, the centroid of the state is roughly the I-35 corridor from San Antonio through Austin to Dallas-Fort Worth — one of the most electorally contested geographic regions in the country, with rapidly shifting suburban populations. A 23% tract-level disruption concentrated in this region is not politically neutral in the way the paper implies.

The paper should analyze where the d_Ham disruption is geographically concentrated for each state. Are the disrupted tracts in urban cores, suburban rings, or rural areas? Are they in electorally competitive regions or safe partisan areas? The answer matters enormously for whether the disruption is "politically neutral" as the statutory framing implies.

A related concern: the paper's claim that "algorithmic redistricting is far more stable than political redistricting" conflates two different types of instability. Political redistricting instability is intentional — mapmakers change boundaries to achieve partisan goals. Algorithmic reapportionment instability is structural — the tree topology changes because the arithmetic changes. These are qualitatively different, and the Hamming distance comparison obscures the distinction. A 23% algorithmic disruption concentrated in competitive swing-district corridors is politically more significant than a 35% political redistricting disruption concentrated in already-safe partisan areas.

The California analysis is inadequately developed. The paper notes that the 52→50 seat change produces a "structural" tree change (both composite, but different factorizations). But California is the largest state, with the most complex geographic diversity, and the paper provides only a brief characterization of the California case. The political implications of the factorization change from 52 = 4×13 to 50 = 2×5² deserve more analysis.

## Minor Issues

- The "incumbent protection analysis" in Section 5.4 is too brief. The claim that "at most 23% of incumbents could be moved to a different district" provides an upper bound, but the paper should give a lower bound based on how often incumbents reside in tracts that are in the same district under both tree structures.
- The note in Section 6.2 that "the previous map is a useful starting point for the new map" is inconsistent with the DIA's fresh-recomputation design (Section 6.1). The paper should clarify whether it is recommending that the DIA use the previous map as a warm start (which would contradict Section 6.1) or merely making an observation about the information content of the previous map.
- The open question about "optimal transition strategies" (Section 7.2) should note that any transition-minimizing objective would require the algorithm to access the previous map, which creates potential for gaming the transition objective in politically motivated ways.

## Recommendation

Major revisions required. The paper must analyze the geographic and political distribution of boundary disruptions for at least the Texas and California cases before the "less disruptive than political redistricting" claim can be evaluated fairly.
