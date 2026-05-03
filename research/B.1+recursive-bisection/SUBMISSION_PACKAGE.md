# B.1 Submission Package — APSR

**Paper**: Recursive Bisection for Congressional Redistricting: Extending Huntington-Hill to Boundary Design
**Target venue**: American Political Science Review (APSR)
**Panel avg**: 3.64/4 (Round 2 — Strong Accept, 3 perfect scores: Chen, Karypis, Pildes)
**Status**: Ready for submission

## Submission Checklist

- [ ] APSR author guidelines reviewed (https://www.cambridge.org/core/journals/american-political-science-review/information/instructions-for-contributors)
- [ ] Word count verified (APSR limit: 12,000 words for research articles)
- [ ] Blind review copy prepared (remove author name, acknowledgments)
- [ ] Supplementary materials package: data, code, replication archive
- [ ] Cover letter written
- [ ] Co-author conflict of interest declarations

## Cover Letter Draft

Dear Editors,

We submit for consideration "Recursive Bisection for Congressional Redistricting:
Extending Huntington-Hill to Boundary Design."

This paper introduces Prime-Factor Redistricting (PFR), a deterministic algorithm
that extends the Huntington-Hill apportionment method — the constitutional procedure
already used to allocate seats among states — to the intrastate problem of drawing
district boundaries. PFR requires zero additional degrees of freedom beyond the
apportionment formula: the prime factorization of each state's seat count uniquely
determines a hierarchical bisection tree, and minimum-edge-cut graph partitioning
on the federal TIGER adjacency graph draws the boundaries.

The paper demonstrates that PFR produces zero partisan variance across 480 random
seeds on all tested states — the partisan outcomes are properties of the geography,
not of seed choice. This resolves the "why this seed?" objection that arises under
any seed-based redistricting proposal. The paper also shows that PFR produces
137 majority-minority districts nationally versus 68 enacted (net +69 advantage),
satisfying the Gingles criteria without any explicit racial targeting.

We believe this work is of direct relevance to APSR's readership in political science,
law, and electoral studies, given its implications for the Redistricting Integrity Act
proposal and current state court litigation on algorithmic redistricting.

## Replication Archive
All code, data, and run logs are available at:
https://github.com/giodl73-repo/REDIST
See research/A.4+replication-materials/ for the canonical replication script.
