# Role Review: BISECT FLETCH Source Orchestration

reviewer: ROLE PANEL
date: 2026-05-15
scope: `docs/specs/2026-05-15-bisect-fletch-source-orchestration.md`
roles: MERIDIAN, SCALE, BENCHMARK, LEDGER, COVENANT, TRENCH, COMMONS

## Verdict

Approved for a staged migration, provided FLETCH does not absorb redistricting
semantics or change BISECT output paths.

## Role Findings

| Role | Score | Finding |
|---|---:|---|
| MERIDIAN | 3.5/4 | The seam is correct: FLETCH acquires source bytes, while BISECT retains state/year/type expansion and derived artifact logic. |
| SCALE | 3.2/4 | A reusable cache root helps repeated fetches. Large Census ZIP streaming remains important and is preserved through FLETCH object caching plus local extraction. |
| BENCHMARK | 3.4/4 | The non-mutating `fletch-sources --gate` provides a concrete readiness check before changing fetch behavior. Existing fetch tests should remain the regression floor. |
| LEDGER | 3.6/4 | The handoff CSV makes generic-ready and adapter-required sources explicit. It correctly records that downloads do not validate downstream claims. |
| COVENANT | 3.2/4 | Public data provenance improves, but court/evidence claims must still be made by BISECT reports and audit artifacts, not by FLETCH cache hits. |
| TRENCH | 3.3/4 | Preserve BISECT's done markers, local paths, `--force`, and 404 soft-skip behavior. Do not force release assets through generic HTTP in this slice. |
| COMMONS | 3.0/4 | Shared infrastructure reduces duplicated fetch code without changing the civic/redistricting interpretation layer. |

Overall: 23.2 / 28. Approved.

## Required Follow-Ups

1. Keep GitHub release adjacency as adapter-required until release URLs and auth
   behavior can be represented without weakening existing `gh` semantics.
2. Keep LODES and ACS 404 behavior soft; absence of a source is data availability,
   not a fatal redistricting error.
3. Do not treat `data/.fletch` cache presence as proof that BISECT outputs are
   complete or legally/admissibly valid.

