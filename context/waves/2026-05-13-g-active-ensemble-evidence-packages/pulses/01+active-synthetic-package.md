---
wave: g-active-ensemble-evidence-packages
pulse: 01
status: in-progress
governing_roles:
  - DATUM
  - BENCHMARK
---

# Pulse 01 - Active Synthetic Package

## Mission

Add a docs-level active synthetic G.1-G.3 package with all required artifact
roles and a focused verifier test.

## Pre-implementation Scout

```powershell
rg -n "G\\.1|G\\.2|G\\.3|external trace|election|metric-output|missing-evidence" docs research\tracks\G-ensemble crates\bisect-ensemble
git --no-pager status --short
```

## Deliverables

- [x] Decide scope: active synthetic fixture, not real empirical result.
- [x] Add active G.1-G.3 package under `docs/examples/`.
- [x] Add verifier coverage in `bisect-ensemble`.
- [x] Run focused test.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble active_g1_g3_package_fixture_validates
git diff --check
```
