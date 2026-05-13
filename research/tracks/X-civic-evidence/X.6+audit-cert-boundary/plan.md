# X.6 RAUDIT/RCERT Boundary

Goal: reserve reusable audit transcript and certification-action layers without
pulling them out of RCOUNT too early.

## Status

- [x] Define RAUDIT as reusable audit/recount/RLA transcript bundles.
- [x] Define RCERT as certification actions, signoffs, and evidence matrices.
- [x] Record that RAUDIT remains embedded in RCOUNT until it has a second
  production consumer.
- [x] Record that RCERT reads RCOUNT/RAUDIT/RLOG/RCHAIN instead of recomputing
  count arithmetic.
- [ ] Identify public certification source artifacts before RCERT schema work.
- [ ] Decide whether V-series audit transcripts need standalone RAUDIT packages.

## Specification

- `docs/specs/2026-05-13-downstream-evidence-boundaries.md`
- `docs/specs/2026-05-13-civic-evidence-layer-access-patterns.md`
- `docs/specs/2026-05-13-rcount-audit-algorithm-roadmap.md`

## Boundary Rule

RAUDIT can replay method evidence. RCERT can record official actions. Neither
owns the base count ledger, and neither should silently change RCOUNT verifier
results.
