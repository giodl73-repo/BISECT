---
wave: g-ensemble-evidence-packages
pulse: 01
status: done
depends_on: []
governing_roles:
  - DATUM
  - SCALE
  - COVENANT
  - LEDGER
  - BENCHMARK
---

# Pulse 01 - Evidence Inventory and Manifest Contract

## Mission

Inventory existing artifacts for G.1-G.3 ensemble claims and define the manifest
contract needed to validate future external ensemble/election/metric evidence
packages.

## Pre-implementation Scout

Run and record:

```powershell
rg -n "GerryChain|percentile|ensemble|Democratic seats|Polsby|Reock|edge-cut|diagnostic|R-hat|ESS|manifest|RPLAN|RCTX" research/tracks/G-ensemble docs data runs analysis reports crates
Get-ChildItem -Recurse -File data,runs,analysis,reports -ErrorAction SilentlyContinue | Select-Object FullName,Length | Sort-Object FullName
git --no-pager status --short
```

## Deliverables

- [x] Inventory existing candidate artifacts and classify them as present,
  missing, stale, or insufficient.
- [x] Define the evidence package manifest fields for G.1-G.3 claims.
- [x] Decide where the manifest schema/fixtures should live.
- [x] Add or update docs that describe what is and is not replayed.
- [x] Update `WAVE.md` and this pulse checklist.
- [x] Run validation and commit.

## Scout Results

- The shell `rg` command was unavailable in the interactive PowerShell PATH, so
  the same search was run with the built-in ripgrep tool over the required
  G-track, docs, data, runs, analysis, reports, and crate roots.
- The full `Get-ChildItem -Recurse` over `data`, `runs`, `analysis`, and
  `reports` was too large for an interactive pulse and was stopped. Targeted
  searches found paper claims, existing RPLAN/RCTX examples, and ensemble L2
  tests, but no dedicated G.1-G.3 external-trace evidence package.
- Existing G.1-G.3 paper text and scorecard rows classify the current state as
  boundary-scoped: external traces, election inputs, metric outputs, diagnostics,
  and plan/context packages are required before headline percentiles can be
  cited as final evidence.
- The contract lives in `bisect-ensemble::evidence_manifest` because the package
  validates ensemble evidence shape, while concrete RPLAN/RCTX files remain
  artifact roles referenced by hash.

## Implementation

- Added `GEnsembleEvidenceManifest` with `active` and `missing-evidence`
  statuses.
- Active packages require at least one hash-bound file and at least one
  verification command.
- Missing-evidence packages require explicit `{role, reason, next_step}` gaps
  and do not validate headline claims.
- Paths must be package-relative and portable; hashes must be 64 lowercase hex
  SHA-256 strings.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```
