---
pulse: 31
title: Block-ensemble v2 Stage 0 readiness
status: complete
wave: nrs-2010-national-baseline
validation_level: local readiness passed; no process launched
---

# Pulse 31 - Block-Ensemble v2 Stage 0 Readiness

## Purpose

Close the final input, executable, capacity, and custody checks needed before a
separate decision to launch the six excluded v2 preflights.

## Result

- [x] Repair three retained resource-manifest hashes from repository CRLF-to-LF
      normalization; artifact semantics and all JSON values are unchanged.
- [x] Reverify the retained Pulse 24 resource package in full.
- [x] Build `block_trace` and `validate_block_input` in release mode and bind
      their local SHA-256 digests plus their source and toolchain identities.
- [x] Freshly validate NH, NM, and GA inputs and retain hash-bound audit records.
- [x] Confirm the v2 ledger is pristine and the package has no process artifacts.
- [x] Observe 106,446,356,480 free bytes against 8,589,934,592 required bytes.
- [x] Add adversarial checks for input, executable, capacity, and empty-package
      custody.
- [x] Complete a six-role readiness review and incorporate its claim boundaries.

## Decision

The package is locally ready for a separate Stage 0 launch decision. This pulse
does not itself authorize or start a process. Every future preflight must still
pass a fresh capacity admission immediately before process creation and retain
that immutable attempt record.

## Claim Boundary

This is author-machine custody and readiness evidence. It is not a reproducible
build attestation, a governed chain, a mixing assessment, or a statistical
result. The fresh audits bind the derived RCTX and assignment files; they do not
independently rederive those files from upstream Census sources.
