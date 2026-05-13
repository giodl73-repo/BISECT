# Simulated Review: CANVASS

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The paper correctly treats precincts as administrative objects rather than
permanent natural units. It also keeps the right legal boundary: RCOUNT can
verify declared reporting-unit lineage, but it does not decide whether a
precinct change was lawful, wise, or politically neutral.

## Major Issues

1. **Administrative events need more concrete sources.** The paper names
   authority and explanation, but should say what source artifacts might support
   lineage: precinct board orders, county GIS updates, statements of votes,
   voter-assignment files, or public precinct-change notices.

2. **The split/merge story needs a visual ledger.** CANVASS wants a reader to
   see P-004 becoming P-004A/P-004B and P-007/P-008 becoming P-078, not only read
   a table of ids.

3. **Jurisdiction variation is understated.** Some jurisdictions use precincts,
   split precincts, vote centers, or central-count batches differently. V.2
   should state that lineage adapters must preserve local reporting conventions.

## Minor Issues

- Add a sentence that precinct lineage is not voter movement.
- Mention board/court/legal authority without requiring a single artifact name.
- Clarify that a passing lineage check is not approval of a precinct change.

## Strengths

- Excellent "precincts are not eternal atoms" framing.
- Clear separation between lineage evidence and political interpretation.
- Good failure distinction between bad lineage and stale plans.
