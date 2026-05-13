---
wave: r-package-completion
pulse: 01
status: done
governing_roles:
  - rcount-core
  - rcount-stats
  - rcount-audit
  - rcount-io
  - rcount-cli
---

# Pulse 01 - RCOUNT Audit Replay Substrate

## Mission

Retrofit the completed V.12 through V.18 audit algorithm work into the wave
ledger. This pulse records the substrate already landed: replayable BRAVO,
Minerva, Kaplan-Markov/MACRO, ALPHA, batch comparison, and boundary-preserving
Athena and stratified/hybrid coordinator runs.

## Delivered

- [x] V.12 BRAVO ballot-polling replay.
- [x] V.13 Minerva round-one and multi-round replay with Athena boundary.
- [x] V.14 Kaplan-Markov comparison replay and MACRO design fields.
- [x] V.15 ALPHA fixed-bet bounded martingale replay.
- [x] V.17 stratified/hybrid coordinator boundary surface.
- [x] V.18 batch comparison records, linkage, derivation helper, and replay.
- [x] Core, IO, audit, CLI, stats, atlas, and roadmap docs updated.

## Validation

```powershell
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-stats -p rcount-core -p rcount-io -p rcount-audit -p rcount-district -p rcount-cli
```

## Carry Forward

- External public validation is still required for V.13 and V.14 where public
  artifacts expose the method-specific inputs.
- V.16 SHANGRLA normalization remains the future unifying assertion language.
- V.17 combined-risk replay waits on explicit combining-rule validation.

