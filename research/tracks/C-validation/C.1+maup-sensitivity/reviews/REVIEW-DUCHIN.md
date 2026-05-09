# Review — C.1: Spatial Resolution and Algorithmic Redistricting (MAUP)
**Reviewer**: Moon Duchin (Gerrymandering, metric geometry, redistricting methodology)
**Round**: 1
**Score**: 3/4
**Verdict**: Minor Revision

## Summary

C.1 addresses an important and underexplored question: does the bisect algorithm's spatial resolution choice affect its outputs enough to matter? The answer — no, within 2–3% for compactness and ±1 seat for seat counts — is reassuring and well-supported for the five focus states. The MAUP framing is appropriate; the structural argument (graph-edge-cut optimisation is insensitive to unit-size changes that preserve adjacency topology) is insightful and partially explains the finding. My concerns are about the seat-count stability claim and the limited state sample.

## Strengths
- The structural explanation for MAUP robustness is the paper's most important theoretical contribution: bisect optimises edges, not polygon geometry, so unit-size changes that don't add new adjacencies don't change the optimisation landscape significantly. This should be elevated to a theorem or proposition.
- The five-state selection (NC, TX, WI, FL, VA) is geographically and demographically diverse — covering competitive states, VRA-important states, and geographically varied settings.
- The monotonic PP pattern and its log-linear theoretical prediction are well-matched.

## Concerns
- **Seat-count stability with single seeds**: The claim that "seat counts are stable within ±1 seat" across resolutions is important but measured at a single seed per state per resolution. With a different seed, the plan may differ, and a different plan might show more than ±1 seat difference across resolutions. Report seat counts for 5+ seeds at each resolution to verify that the ±1 bound holds across seeds.
- **Adjacency topology change**: The paper's structural argument assumes "unit-size changes that preserve adjacency topology." But county-to-tract changes do add adjacencies (a single county is replaced by many tracts with internal adjacencies), which changes the optimization landscape. The paper should acknowledge that the stability finding is empirical, not fully explained by the adjacency-preservation argument, since the topology does change between county and tract levels.
- **Partisan implications not tested**: The paper tests PP compactness and seat counts but not partisan efficiency gap or minority representation across resolutions. A hostile expert could argue that resolution choice changes partisan outcomes even if seat counts are stable. A brief check of efficiency gap stability across resolutions would strengthen the paper.

## Required Changes (P1/P2)
- **P1**: Report seat counts for at least 5 seeds per state-resolution combination and show that ±1 bound holds in aggregate.
- **P2**: Elevate the structural explanation to a formal Proposition: "If resolution change preserves adjacency topology, then the minimum edge-cut solution is preserved up to boundary-tract rounding." Acknowledge that the county-to-tract change doesn't exactly preserve topology and add an empirical explanation for the observed stability.
- **P2**: Add a brief efficiency gap comparison across resolutions for the five focus states.
