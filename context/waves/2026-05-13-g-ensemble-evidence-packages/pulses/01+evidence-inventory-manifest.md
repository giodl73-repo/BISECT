---
wave: g-ensemble-evidence-packages
pulse: 01
status: todo
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

- [ ] Inventory existing candidate artifacts and classify them as present,
  missing, stale, or insufficient.
- [ ] Define the evidence package manifest fields for G.1-G.3 claims.
- [ ] Decide where the manifest schema/fixtures should live.
- [ ] Add or update docs that describe what is and is not replayed.
- [ ] Update `WAVE.md` and this pulse checklist.
- [ ] Run validation and commit.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```
