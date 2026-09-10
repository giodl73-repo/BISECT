# Research pause and resume checkpoint

Date: 2026-09-10. Status: good research checkpoint, not a finished districting
standard or publication-ready claim of optimality/neutrality.

Git handoff: checkpoint branch `research/districting-checkpoint-2026-09-10`,
based on `f7250e72`. At pause time remote main had advanced to `e0764d2a`
with 13 additional commits, including overlapping runner changes. Integrating
those changes and rechecking affected source-bound artifacts is a separate
resume task; this checkpoint does not overwrite remote main.

Subsequent user-authorized integration is recorded in
[main integration](2026-09-10-main-integration.md). The checkpoint commit
`81f396f5` remains the source snapshot for historical hash-bound reports.

## What is complete in this round

- Certified sequential-ceremony implementation, bounded procedural bakeoff,
  aligned A.0/B.0/B.1/B.02/U.21 paper stack and local reviews from the preceding round.
- Three-cycle national replay and 2020 national geographic/geometry baseline
  checks, recorded in the external verification and experiment artifacts.
- Development roadmap and local `.roles` review with explicit evidence gates.
- County-response diagnostics: fixed DFS, five roots, population bands,
  county-aware trees, connected single-block descent, longer budgets, adjacent swaps.
- Experiment READMEs, manifests, results, tests, move replays and main research
  indices record favorable and unfavorable outcomes. Latest county-search
  diagnostics have not yet been incorporated throughout the academic manuscripts.

## Current empirical position

RI connected single-block search reduced split counties from four to three,
including without county weighting. Weighting changed assignments and costs,
but this does not establish that weighting caused the county-count reduction.
With a 500-move allowance, all six arms stopped after 271–283 moves, within
the fixed 1000-ppm child-population band and connected through the source graph.
Adjacent opposite-label block swaps accepted no moves from those final states.
These are local results, not global optima or evidence of nationwide improvement.

Latest result: [adjacent swaps](../experiments/ri-adjacent-swaps-2020/README.md).
Underlying controls: [longer-budget search](../experiments/ri-boundary-budget-2020/README.md).
Program: [development roadmap](2026-09-09-districting-rule-development-roadmap.md).

## Resume here

1. Freeze a new move family: nonadjacent boundary exchanges or connected-group
   transfers. Preserve current starts, population limits and weighted objectives
   for the search-only comparison; retain all failed/incomplete cells.
2. Compare direct county-fragment scoring as a separate objective change.
3. Package and independently check the improved two-label RI assignments, then
   extend to complete multi-district trees with final-leaf population and
   descendant-feasibility checks. Current diagnostics are not full-plan bakeoffs.
4. Promote supported findings into manuscripts and repeat consistency/referee
   review only after the relevant evidence gate is met.

Do not increase population bands, select a production alpha, or claim legal
compliance merely because a local experiment stalls. Production profiles were
not changed by the county-search diagnostic sequence.

## Reproduction and storage caveats

The source contexts and large replay corpus remain under
`M:\DATA_VAULT\projects\apportionment`; they are not included in Git. New
manifests intentionally record local absolute paths and exact source/input
hashes. A different machine needs the vault mounted appropriately or a new,
explicitly versioned path-mapping/replay procedure. Never overwrite frozen
reports simply to accommodate a new path or changed source file.

Hashes and replay checks are not independent external witnesses. Current
move verifiers share helpers with their generators. Ceremony timestamps and
receipts remain test/opaque evidence without a real external witness adapter.
The prior CLI suite passed serially; parallel tests have known global-cwd races.
Long national regeneration is already recorded and need not be rerun merely
to resume this bounded search work.
