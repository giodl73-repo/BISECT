# Track N — Population Counting Panel Review
**Date**: 2026-05-09 | **Round**: 1
**Papers**: N.0–N.5 (6 papers)
**Reviewers**: Pildes, Duchin, Rodden, Stephanopoulos, Liang

## Paper Scores (Round 1)

| Paper | Title | Score | Verdict |
|-------|-------|-------|---------|
| N.0 | Population Counting Overview | 3.2/4 | Conditional Accept |
| N.1 | Prison Gerrymandering | 3.4/4 | Conditional Accept |
| N.2 | College Students | 2.8/4 | Conditional Accept |
| N.3 | Noncitizen VAP | 3.2/4 | Conditional Accept |
| N.4 | Military Overseas | 2.8/4 | Conditional Accept |
| N.5 | 50-State Comparison | 3.6/4 | Conditional Accept (strong) |
| **Track Mean** | | **3.2/4** | |

## Module Score: 8.2/10

## Post-Cleanup Status
- **N.2**: The 180-tract estimate is now tied to the home-address sensitivity workflow in the abstract and Section 4, so it reads as a derived point estimate rather than a bare assertion.
- **N.3**: The 15--20 seat shift estimate is now framed with uncertainty bounds in the body and with a tract-level point-estimate caveat in the divergence section.
- **N.4**: The civilian_only / \texttt{--exclude-gq-types 210} mode remains Phase 2 by design; it is already labeled that way in the paper text.
- **N.5**: The 50-state table now carries prison-adjusted values across the full state set, so the flagship comparison is no longer a pilot-only result.

## Track-Level Strengths
- N.5 is the definitive reference paper on population definition choice in redistricting — the first 50-state empirical comparison across four balance metrics. The statutory recommendation (total population + prison adjustment only) is carefully argued and legally grounded.
- N.1's three-state case study (NY reformed, PA/TX not) provides compelling evidence that prison gerrymandering is a solved problem administratively even if not federally.
- N.3's Evenwel analysis correctly distinguishes congressional (Article I total population) from state legislative (Evenwel opens citizen VAP) contexts — a distinction most practitioners get wrong.

## P1 Items by Paper
- **N.2**: The 180-tract national estimate for student-counting impact needs a derivation or citation. As stated it's an assertion.
- **N.3**: The 15-20 seat shift estimate from citizen VAP redistricting needs uncertainty bounds — the point estimate without CI is not sufficient for litigation use.
- **N.4**: The civilian_only balance metric (\texttt{--exclude-gq-types 210}) is described as a planned bisect feature but not yet implemented. Label clearly as Phase 2.
- **N.5**: Extend from 5-state pilot to all 50 states for the prison-adjusted column — currently the 50-state table has missing data for some states.

## P2 Items
- N.0: Add a recommendation matrix: "If X population definition is mandated, use Y bisect flag." (Pildes)
- N.1: Extend to 2030 — the Census Bureau has announced a prison-adjusted redistricting file for 2030; note this makes N.1 a solved problem by 2030. (Rodden)
- N.3: Add Evenwel Thomas/Alito concurrence significance — their view that citizen VAP may be constitutionally required at state level is more legally live than the majority suggests. (Pildes)

## Next Action
Source draft is now internally consistent. The remaining work is publication packaging: PDF rebuilds, bibliography refresh, and a final pass over the public research index once the compiled artifacts exist.
