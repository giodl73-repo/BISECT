# Review — C.1: Spatial Resolution and Algorithmic Redistricting (MAUP)
**Reviewer**: Jonathan Rodden (Political geography, gerrymandering, electoral systems)
**Round**: 1
**Score**: 3/4
**Verdict**: Minor Revision

## Summary

C.1 provides an important robustness check on the bisect pipeline: does the spatial resolution choice matter for algorithmic redistricting outputs? The finding that it doesn't (within 2–3% for PP, ±1 seat) is reassuring and helps establish that the pipeline's conclusions are not artefacts of the tract-level choice. The political geography perspective is partially covered by the seat-count stability result, but the paper misses an opportunity to connect the MAUP finding to the geographic-sorting literature — which would explain why resolution choices that don't alter geographic sorting patterns produce stable partisan outcomes.

## Strengths
- The five-state selection covers the geographic diversity needed: WI (compact, inland), TX (large, diverse), FL (coastal, elongated), NC (mixed), VA (urban-rural gradient).
- The 50-state summary extends beyond the five focus states and is an important contribution.
- The structural explanation (edge-cut optimisation + adjacency topology) is well-motivated.

## Concerns
- **Geographic sorting and resolution**: The paper explains MAUP stability via the edge-cut argument but doesn't note that geographic sorting is a resolution-invariant phenomenon. Regardless of whether you draw districts at the tract or block-group level, Democratic voters are concentrated in urban cores — and compact redistricting reflects this geography regardless of the resolution granularity. Adding a paragraph connecting MAUP robustness to geographic sorting would strengthen the paper's connection to the political geography literature.
- **What's NOT stable**: The paper focuses on what's stable (PP, seat counts) but doesn't discuss what resolution does change: the precise location of district boundaries (which tracts/block-groups are included in which district). For VRA analysis, whether a specific community is in or out of a district can depend on resolution choice even when the overall seat count is stable. This is worth noting.
- **Policy relevance**: The key question for practitioners is: does resolution choice matter enough to require specification in statute? The paper should conclude with a recommendation: "Resolution should be specified in statute as the standard census unit for the applicable chamber size, with finer resolutions for state legislative maps (as specified in F.3)."

## Required Changes (P1/P2)
- **P2**: Add a paragraph connecting MAUP robustness to geographic sorting — the same underlying partisan geography is captured at any resolution, explaining why seat counts are stable even as boundaries shift.
- **P2**: Add a brief discussion of what resolution DOES change: community membership, exact boundary placement, and sub-district minority concentrations.
- **P2**: End with a statutory recommendation: specify resolution in statute, defaulting to the finest resolution supporting population equality.
