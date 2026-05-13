---
wave: r-package-completion
pulse: 03
status: done
governing_roles:
  - rcount-core
  - rcount-audit
  - rcount-io
  - rcount-cli
---

# Pulse 03 - Analytic And Observable Boundary Methods

## Mission

Add V.20 Bayesian tabulation and V.21 SOBA observable-ballot surfaces as honest
boundary methods. These methods preserve analytic or privacy/linkage evidence;
they do not alter RCOUNT certification math.

## Delivered

- [x] V.20 Bayesian prior and likelihood ids.
- [x] V.20 posterior winner probability, posterior risk, seed, and draw count.
- [x] V.20 negative coverage for impossible posterior risk.
- [x] V.20 audit replay boundary: analytic, not risk-limiting replay.
- [x] V.21 SOBA observable-ballot assertion surface.
- [x] V.21 sample-step linkage to anonymized inclusion proofs.
- [x] V.21 missing-opening negative coverage.
- [x] V.21 audit replay boundary: linkage recorded, comparison risk deferred.
- [x] Core, IO, audit, CLI, atlas, research plan, active goal, and roadmap
  updated.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-stats -p rcount-core -p rcount-io -p rcount-audit -p rcount-district -p rcount-cli
git diff --check
```

Observed: focused tests passed. `git diff --check` only reported existing CRLF
normalization warnings.

## Carry Forward

- V.20 hand-computable posterior replay.
- V.20 calibrated-risk boundary fixture.
- V.21 CVR/human-observation mismatch fixture feeding V.14 comparison math.

