---
wave: g-ensemble-evidence-packages
pulse: 03
status: done
depends_on:
  - 01
  - 02
governing_roles:
  - DATUM
  - COVENANT
  - LEDGER
---

# Pulse 03 - Missing-Evidence Package

## Mission

Create an explicit package for the currently missing G.1-G.3 external ensemble
evidence so paper and ledger updates can cite a concrete artifact gap.

## Pre-implementation Scout

```powershell
rg -n "external-trace package needed|election/ensemble model boundary|metric/trace boundary|G.1-G.3|G.1|G.2|G.3" docs\PAPERS.md docs\papers\ALGORITHM-PAPER-SCORECARD.md context\waves\2026-05-13-g-ensemble-evidence-packages research\tracks\G-ensemble
git --no-pager status --short
```

## Deliverables

- [x] Add a `missing-evidence` package for G.1-G.3.
- [x] Hash-bind a scout summary or equivalent diagnostic artifact.
- [x] Validate the package through the manifest consumer.
- [x] Update wave docs and checklist.
- [x] Run validation and commit.

## Scout Results

- `docs/PAPERS.md` still lists G.1 as needing an external-trace package, G.2 as
  an election/ensemble boundary, and G.3 as a metric/trace boundary.
- Scorecard and paper-source searches show boundary language but no
  hash-bound G.1-G.3 package that validates the headline trace/election/metric
  claims.

## Implementation

- Added
  `docs/examples/g-ensemble-evidence-packages/G.1-G.3+missing-evidence/manifest.json`.
- Hash-bound `scout-summary.json` as the package diagnostic artifact.
- Added `missing_evidence_package_fixture_validates` coverage in
  `bisect-ensemble`.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```
