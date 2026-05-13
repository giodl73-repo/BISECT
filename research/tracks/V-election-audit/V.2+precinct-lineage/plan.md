# V.2 Plan: Precinct Lineage Across Elections

## Question

How can a public verifier compare election counts across cycles when precincts
split, merge, rename, or move under new plans?

## Claims

1. Reporting-unit lineage is a first-class evidence object, not a label-matching
   convenience.
2. Split and merge events let multi-election comparisons preserve continuity
   without pretending precinct ids are stable.
3. RPLAN-linked district aggregation must bind the RCOUNT package hash, plan
   hash, and unit universe hash for each cycle.
4. Negative cases should distinguish bad lineage, stale plans, and tampered
   source packages.

## Evidence

- `precinct-split-lineage` fixture.
- `multi-election-harness` L2 transcript.
- Negative fixtures: `bad-lineage`, `stale-plan`, and `tampered-2028-source`.
