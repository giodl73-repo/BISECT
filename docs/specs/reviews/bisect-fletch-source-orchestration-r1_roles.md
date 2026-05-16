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

## Post-Implementation Role Pass

scope: committed implementation slice in `crates/bisect-cli/src/fletch.rs`,
`crates/bisect-cli/src/fetch.rs`, CLI/docs updates, and FLETCH dependency wiring.

| Role | Finding | Status |
|---|---|---|
| MERIDIAN | Source acquisition is kept outside graph construction and redistricting computation. FLETCH cache hits do not alter adjacency, weights, population balance, or METIS semantics. | Pass |
| BENCHMARK | Focused tests cover FLETCH registry classification and existing fetch behavior. `bisect fletch-sources --gate` and an isolated local HTTP fetch smoke exercise the new seam. Full `bisect-cli` lib tests still have unrelated pre-existing `.bisect` registry failures. | Pass with known baseline caveat |
| LEDGER | The handoff CSV records source family, cache target, acquisition mode, adapter-required status, and the claim boundary. `data/` remains ignored, so the ledger is generated rather than versioned. | Pass |
| COVENANT | Source bytes are cached under `data/.fletch`, but BISECT does not claim that downloaded bytes validate a plan. Court/audit claims remain with build/analyze/report artifacts. | Pass |
| TRENCH | Review found two failure modes: BISECT processing failures deleting FLETCH cache objects, and Windows-only test path literals. Both were fixed before push. | Pass |
| COMMONS | Shared fetch infrastructure reduces duplicated acquisition logic without shifting public-interest or redistricting interpretation into FLETCH. | Pass |

Remaining explicit boundary: release adjacency and election downloads are still
adapter-required. Moving them into generic FLETCH needs a separate source/auth
design, not a silent change to this slice.

