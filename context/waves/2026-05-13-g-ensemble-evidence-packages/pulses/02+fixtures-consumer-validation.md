---
wave: g-ensemble-evidence-packages
pulse: 02
status: done
depends_on:
  - 01
governing_roles:
  - DATUM
  - LEDGER
  - BENCHMARK
---

# Pulse 02 - Fixtures and Consumer Validation

## Mission

Add concrete positive and negative fixtures for the G ensemble evidence manifest
and ensure a consumer can verify referenced file hashes.

## Pre-implementation Scout

```powershell
rg -n "GEnsembleEvidenceManifest|g-ensemble-evidence-manifest|validate_referenced_file_hashes" crates\bisect-ensemble docs\file-formats context\waves\2026-05-13-g-ensemble-evidence-packages
git --no-pager status --short
```

## Deliverables

- [x] Add a positive synthetic active package fixture.
- [x] Add a negative hash-mismatch package fixture.
- [x] Add consumer validation that recomputes referenced file hashes.
- [x] Add focused positive and negative tests.
- [x] Update wave docs and checklist.
- [x] Run validation and commit.

## Scout Results

- The manifest contract now exists in `crates/bisect-ensemble/src/evidence_manifest.rs`.
- No real G.1-G.3 external trace package was introduced in this pulse.
- Fixtures are synthetic and only validate package mechanics; they are not
  empirical evidence for G.1-G.3 headline percentile claims.

## Implementation

- Added `validate_referenced_file_hashes` to recompute SHA-256 values for
  referenced package files.
- Added `active-smoke` and `bad-hash` fixtures under
  `crates/bisect-ensemble/src/fixtures/g_ensemble/`.
- Added positive fixture hash validation and negative hash-mismatch tests.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```
