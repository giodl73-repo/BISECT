# X.5 RSTAT/RROLL Boundary

Goal: reserve layers for statistical analysis and aggregate eligibility
universes without confusing either one with count certification.

## Status

- [x] Define RSTAT as statistical and forensic analysis outputs.
- [x] Define RROLL as aggregate registration, eligibility, and ballot-style
  universes.
- [x] Record that RSTAT must not change verifier pass/fail results.
- [x] Record that RROLL must not publish private voter-level rolls.
- [ ] Define privacy/disclosure thresholds before fixtures.
- [ ] Build RSTAT only after enough normalized RCOUNT/RHIST inputs exist.

## Specification

- `docs/specs/2026-05-13-downstream-evidence-boundaries.md`
- `docs/specs/2026-05-13-civic-evidence-layer-access-patterns.md`

## Boundary Rule

RSTAT can identify patterns that need explanation. RROLL can declare aggregate
eligibility universes. Neither certifies an election.
