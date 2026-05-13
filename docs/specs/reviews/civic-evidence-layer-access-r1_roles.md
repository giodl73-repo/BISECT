# Role Review: Civic Evidence Layer Access Patterns

reviewer: ROLE PANEL  
date: 2026-05-13  
scope:
`docs/specs/2026-05-13-civic-evidence-layer-access-patterns.md`,
`docs/specs/2026-05-13-rctx-boundary.md`,
`docs/specs/2026-05-13-rhist-boundary.md`,
`docs/specs/2026-05-13-downstream-evidence-boundaries.md`  
roles: BOUNDARY, COVENANT, CONTOUR, LEDGER, BENCHMARK, SCALE, TRENCH, CANVASS,
TALLY, VAULT, IDENTITY, LINEAGE, CARTOGRAPHY, CUSTODY, STATS, COMPOSITION

## Verdict

Approved as the implementation gate for package layering. The specs now explain
who writes each fact, who reads it, and which packages are explicitly deferred.

The most important outcome is that RCOUNT can continue without absorbing
RCTX/RHIST/RSTAT/RCHAIN responsibilities. The second most important outcome is
that `rctx-core` extraction is delayed until access pressure is real.

## Role Scores

| Role | Score | Finding |
|---|---:|---|
| BOUNDARY | 3.7/4 | Ownership and prohibited access are explicit. Add per-package claim-boundary schemas before implementation. |
| COVENANT | 3.4/4 | Read modes and source snapshots are clear. Hash domains still need final names for deferred packages. |
| CONTOUR | 3.7/4 | RCTX/RMAP split is now concrete enough for engineering decisions. |
| LEDGER | 3.5/4 | Common package interface is useful without forcing identical internals. Manifest schemas remain future work. |
| BENCHMARK | 3.3/4 | RHIST fixture ladder is concrete. Deferred packages need L0 fixtures before crate work. |
| SCALE | 3.4/4 | RSTAT is isolated from verifier pass/fail and certification. Good guardrail. |
| TRENCH | 3.5/4 | Build order protects RCOUNT from premature package sprawl. |
| CANVASS | 3.3/4 | RCERT/RAUDIT/RCOUNT boundaries are now readable. |
| TALLY | 3.4/4 | RCOUNT remains the count ledger owner; audit and certification read it. |
| VAULT | 3.2/4 | RROLL and RSTAT privacy warnings are present. Needs concrete disclosure thresholds later. |
| IDENTITY | 3.8/4 | RCTX remains the sole canonical unit identity layer. |
| LINEAGE | 3.8/4 | RHIST access patterns are strong and prevent RCOUNT/RPLAN duplication. |
| CARTOGRAPHY | 3.7/4 | RMAP owns presentation and is not needed for machine verification. |
| CUSTODY | 3.4/4 | RLOG/RCHAIN split avoids inferring custody from logs or counts. Needs real source examples before modeling. |
| STATS | 3.6/4 | RSTAT claim language blocks anomaly-as-certification mistakes. |
| COMPOSITION | 3.6/4 | RCASE is correctly limited to child-package composition. |

Overall: 56.3 / 64. Approved for planning; only RCTX and RHIST are cleared for
near-term implementation planning.

## Required Fixes Before Crate Work

### F1: RHIST Package Spec

Promote the RHIST boundary into an implementation spec before writing code:

- exact manifest fields;
- exact NDJSON record schemas;
- verifier error taxonomy;
- package-hash projection;
- L0 positive and negative fixtures.

### F2: RCTX Crosswalk Decision

Do not add ad hoc crosswalks separately to RCOUNT and RHIST. Pick one of:

- add crosswalk records to existing RPLAN-owned `.rctx` code as a transitional
  measure;
- create a small `rctx-core` only for shared crosswalk/context validation;
- keep crosswalks in RHIST first, but make their schema byte-compatible with
  RCTX.

Preferred path: keep existing `.rctx` stable, implement RHIST first, and only
extract `rctx-core` when RCOUNT needs direct crosswalk validation.

### F3: RCOUNT Access Notes

Update RCOUNT planning docs to say:

- current-count ledgers stay in RCOUNT;
- cross-cycle unit lineage belongs to RHIST;
- district aggregation binds RCOUNT, RPLAN, RCTX, and crosswalk hashes;
- audit transcripts remain embedded until RAUDIT has a second consumer.

## Deferred Implementation List

Do not implement yet:

- RMAP, until rendered/public map products become a deliverable;
- RAUDIT, until audit transcripts need to stand apart from RCOUNT;
- RCERT, until certification source artifacts are in hand;
- RSTAT, until enough normalized RCOUNT/RHIST data exists;
- RLOG/RCHAIN, until real log/custody artifacts force schemas;
- RROLL, until aggregate registration or ballot-style universes enter scope;
- RCASE, until child package claim boundaries are stable.

## Approved Next Step

Write the RHIST implementation-ready spec, then add the tiny positive and
negative fixture plan. Continue RCOUNT only through access patterns that remain
compatible with RHIST and RCTX.
