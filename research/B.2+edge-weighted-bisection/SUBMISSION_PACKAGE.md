# B.2 Submission Package — KDD 2026

**Paper**: Edge-Weighted Recursive Bisection for Compact Congressional Districts
**Target venue**: KDD 2026 (ACM SIGKDD Conference on Knowledge Discovery and Data Mining)
**Panel avg**: 3.71/4 (Round 2 — unanimous 4/4 acceptance after P2R2 items resolved)
**Status**: Ready for submission

## Submission Checklist

- [ ] KDD 2026 author guidelines reviewed (https://kdd.org/kdd2026/)
- [ ] Paper formatted per ACM proceedings template (double-column, 9-page limit)
- [ ] Supplementary materials: algorithm pseudocode, extended results tables
- [ ] Blind review copy prepared
- [ ] Cover letter / abstract submission
- [ ] Ethics checklist (KDD requirement)

## Cover Letter Draft

Dear Program Chairs,

We submit "Edge-Weighted Recursive Bisection for Compact Congressional Districts"
to KDD 2026 in the Applied Data Science track.

This paper demonstrates that encoding geographic boundary length as METIS edge
weights — where longer shared borders between census tracts receive higher weights —
produces congressional district maps that are simultaneously more compact, more
population-balanced, and more reproducible than maps produced by unweighted
bisection or VRA multi-constraint partitioning.

The key empirical finding is a performance inversion: edge-weighted single-objective
partitioning achieves VRA compliance (majority-minority districts meeting Gingles
criteria) in 4 of 5 tested states, while the purpose-built multi-constraint
formulation achieves compliance in only 2 of 5 — despite using minority population
explicitly as a balance constraint. We explain this inversion via the constraint
conflict mechanism: when population and minority-concentration constraints differ
by 60-800×, METIS's refinement phase is dominated by the tighter constraint,
preventing the weaker one from influencing the solution.

The result has practical implications for any graph partitioning problem where
multiple soft constraints interact: single-objective optimization with appropriately
weighted objectives consistently outperforms explicit multi-constraint formulations
when constraint scales differ by large factors.

## Replication Archive
https://github.com/giodl73-repo/REDIST
All 160 experiments reproducible via scripts/run_b2_experiments.py
