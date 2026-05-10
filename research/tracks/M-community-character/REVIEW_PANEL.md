# Track M — Community Character Panel Review
**Date**: 2026-05-09 | **Round**: 1
**Papers**: M.0–M.8 (9 papers)
**Reviewers**: Duchin, Rodden, Stephanopoulos, Karypis, Liang

## Paper Scores (Round 1)

| Paper | Title | Score | Verdict |
|-------|-------|-------|---------|
| M.0 | Framework + Legal Grounding | 3.0/4 | Conditional Accept |
| M.1 | Economic Character (LODES) | 3.2/4 | Conditional Accept |
| M.2 | Land Use (NLCD) | 2.8/4 | Conditional Accept |
| M.3 | Housing Character (ACS) | 2.8/4 | Conditional Accept |
| M.4 | Commuting Shed (LODES OD) | 3.0/4 | Conditional Accept |
| M.5 | Topographic Features (3DEP) | 3.0/4 | Conditional Accept |
| M.6 | Administrative Zone Co-membership | 3.0/4 | Conditional Accept |
| M.7 | Transit Accessibility (GTFS) | 2.8/4 | Conditional Accept |
| M.8 | Composite Index | 3.2/4 | Conditional Accept |
| **Track Mean** | | **3.0/4** | |

## Module Score: 7.8/10

## Track-Level Strengths
- The M-track is the only body of work in the redistricting literature to systematically operationalise seven distinct community-of-interest signals in a unified computational framework. This is a genuine research contribution.
- All M.1–M.7 papers satisfy the track contract: similarity metric in [0,1], data source documented, NC/WI/TX empirical effect, proportionality gap delta, §5 Legal Usage. Consistency across papers is exceptional.
- M.8's composite index addresses the litigation risk of single-signal vulnerability directly — this is the most legally impactful paper.

## Cross-Paper Issues
1. **Signal correlation**: M.1 (economic) and M.4 (commuting shed) are likely correlated — tracts with similar economic character often have similar commuting patterns. M.8 should address whether the composite index double-counts these correlated signals or weights them appropriately.
2. **Geographic coverage**: M.7 (transit) has only 28–51% coverage across states (zero in rural areas). M.8 must address how the composite handles missing signals without penalising rural tracts.
3. **Proportionality gap**: All papers report proportionality gap change vs. the −6.5pp NC baseline. The −6.5pp baseline should be cited to T.5 consistently — some papers cite it, others do not.

## P1 Items by Paper
- **M.0**: The legal citation to *Shaw v. Reno* (1993) as authority for communities of interest is correct but incomplete — *Shaw* was about racial classification, not communities of interest per se. Add *Thornburg v. Gingles* §2 communities-of-interest language and the relevant state cases (California Prop 11 (2008), Colorado Amendment Y (2018)).
- **M.2 (NLCD)**: The NLCD land use classification (21 categories) is reduced to 6 for similarity. Justify this aggregation — why not 4 or 10? Provide sensitivity analysis.
- **M.7 (GTFS)**: Coverage limitation (18% of population in areas with no transit) is acknowledged but M.8 does not specify how missing M.7 scores are handled in the composite. Clarify: is transit treated as zero (disadvantaging urban-adjacent rural) or as N/A (excluded from composite for that tract)?
- **M.8**: The composite weighting (equal weights) is asserted without justification. Add sensitivity analysis: how do composite rankings change if economic character (M.1) is weighted 2× or if transit (M.7) is excluded?

## P2 Items
- M.0: Cross-reference D.5's bloc voting methodology — communities of interest and bloc voting are related concepts in VRA litigation.
- M.4 + M.1: Address signal correlation explicitly in M.8 or add a correlation analysis appendix.
- M.5: The watershed HUC-8 level is justified but HUC-6 (larger basins) and HUC-10 (smaller) should be mentioned as sensitivity cases.

## Next Action
M.8 P1 (composite weighting sensitivity) and M.7 P1 (missing transit handling) are the highest-priority items — they affect the composite's legal defensibility. M.0 legal citations should be updated to include state constitutional authority beyond Shaw.
