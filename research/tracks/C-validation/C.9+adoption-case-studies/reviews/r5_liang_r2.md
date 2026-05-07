# Review 5 — Reviewer: Christina Liang (Computational Social Science / Implementation Research)
**Paper:** C.9 — From Algorithm to Map: Implementation Case Studies for Three Adoption Pathways
**Round:** 2
**Score:** 3/4

## Summary

The revision addresses my principal concerns: the comparative summary table across the three pathways is now present, the contracting/conflict-of-interest question is addressed, and the unresolvable forward citations are now marked explicitly. I maintain my score at 3/4. The paper is now a well-structured and honest implementation guide.

## Addressed Issues

The comparative summary table (Section 5.4) is the most important addition from my perspective. The three-column comparison (who runs the algorithm / adjustment mechanism / challenge standard) across the three pathways makes the comparative argument explicit rather than requiring the reader to extract it from the workflow table. The table is clean and directly useful.

The contracting/conflict-of-interest question (Section 6.2, my Round 1 concern about whether it is appropriate for the redistricting commission to contract with the algorithm developer) is now addressed: the paper recommends that adopting jurisdictions establish a third-party auditing role (separate from the algorithm developer) to verify that the algorithm was run correctly. The proposal is reasonable and addresses the core conflict-of-interest concern.

The forward citations are now consistently marked: critical claims that depend on unavailable papers are either (a) supported by inline evidence, or (b) marked as "forthcoming" with the specific claim they are being used to support. The *dellaLibera2026vra* citation for VRASection performance is now marked "forthcoming — aligned with the methodology in D.5," which is honest.

The plausible early adopter states are now named (Michigan, Colorado, Wisconsin, Virginia), with brief descriptions of the institutional barriers each faces. This is the specificity I requested.

## Remaining Concerns

The generalizability to smaller states (Wyoming, Montana, small delegations) is acknowledged in a footnote rather than a dedicated discussion. The paper correctly notes that the three case studies represent larger, more institutionally sophisticated states, but the footnote is brief. A paragraph discussing how smaller states with limited GIS capacity would implement the workflow (perhaps through regional compacts or federal assistance) would complete the generalizability claim.

The benchmark for data preparation time (now specified as a 16-core workstation) is better than before, but the paper should note whether cloud computing options (AWS, Azure) are appropriate for redistricting work — the data sensitivity question (census tract population data is public, but the algorithm's deposition log may be sensitive) deserves brief attention.

## Minor Issues

- The adjustment threshold discussion (5%/10% thresholds) is cross-referenced from Section 2.3 to 6.3, which is appropriate.
- The three-pathway comparison table cross-references the workflow table correctly.
- The paper correctly notes that Wyoming's single-seat status means no redistricting is required, removing it from the scope.

## Recommendation

Accept. The comparative table and the conflict-of-interest discussion are genuine improvements. The paper is now a structured and honest implementation guide.
