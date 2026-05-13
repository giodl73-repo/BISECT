# V.13 Plan

## Thesis

Minerva and Athena are round-based ballot-polling RLA methods used in modern
audit software and state reports. RCOUNT should replay their published risk
measurements when the artifact set is sufficient.

## Atlas

- `docs/algorithm-atlas/v13-minerva-athena-ballot-polling.md`

## Landed

- [x] RI Rep. 28 adapter records `MINERVA` metadata.
- [x] RI Rep. 28 adapter validates sampled-ballot rows against retrieval rows
  and writes a source-summary transcript.
- [x] RI Rep. 28 adapter emits a package-level `minerva-ballot-polling-v1`
  `AuditAlgorithmRun` with `Boundary` decision.
- [x] `rcount replay-audit-algorithms` reports Minerva/Athena as explicit
  boundary-only method surfaces rather than generic unsupported methods.
- [x] `rcount-stats` replays a scoped round-one, two-candidate Minerva
  tail-ratio check with exact binomial tails.
- [x] `synthetic_minerva_round_one_package` verifies, round-trips, and replays
  through the CLI with known risk.
- [x] Published-risk drift on the final round step is detected by
  `replay-audit-algorithms`.
- [x] `AuditSampleStep.round_index` can carry explicit Minerva round
  boundaries without breaking older single-round packages.
- [x] `synthetic_minerva_multi_round_package` verifies, round-trips, and
  replays round one as continue and round two as stop.
- [x] `synthetic_athena_boundary_package` verifies, round-trips, and reports a
  method-specific boundary through `replay-audit-algorithms`.

## Implementation Tasks

- [x] Add synthetic Minerva positive fixture with known risk.
- [x] Add published-risk drift negative fixture.
- [x] Add package schema for explicit round boundaries when multiple Minerva
  rounds are present.
- [x] Add multi-round continue/stop fixture.
- [ ] Replace RI Rep. 28 boundary replay with computed risk once public source
  rows expose sufficient per-round ballot evidence.
- [x] Add Athena replay fixture or retain a documented Athena boundary fixture
  if the method inputs remain absent.

## Claim Boundary

RCOUNT can now replay synthetic one-round and multi-round two-candidate
Minerva packages from ballot observations. It still must not claim to recompute
Rhode Island's Minerva stopping decision, because that public fixture is
preserved as a package-level method run without the ballot-observation sequence
needed for round replay.
