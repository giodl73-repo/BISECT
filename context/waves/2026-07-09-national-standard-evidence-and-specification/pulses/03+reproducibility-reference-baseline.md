---
pulse: 03
title: Reproducibility reference baseline
status: done
depends_on: 01
wave: national-standard-evidence-and-specification
validation_level: L2 reference replay
---

# Pulse 03 - Reproducibility Reference Baseline

## Purpose

Create one release-candidate-quality replay demonstrating that a named source
commit, toolchain, input manifest, and canonical configuration produce the
expected reference assignments and audit chain.

## Deliverables

- [x] Pin an exact Rust toolchain version.
- [x] Correct `docs/REPRODUCIBLE_BUILD.md` to match embedded METIS,
      implemented provenance, and current verification commands.
- [x] Select a small canonical reference state and configuration.
- [x] Record source, compiler, dependency, census, TIGER, adjacency, config,
      and output hashes.
- [x] Run the replay in a clean worktree or container.
- [x] Publish the reference manifest and expected assignment hash without
      committing prohibited raw census data.

## Validation

- Targeted Rust tests for provenance, manifests, and verification.
- Two independent clean executions with identical assignment hashes.
- `git --no-pager diff --check`.

Results:

- Targeted config, build-index, output, provenance, and manifest-verification
  tests passed.
- Two clean-source Rhode Island executions produced identical raw and canonical
  assignment hashes.
- Both label chains reported `VERIFIED`.
- `git --no-pager diff --check` passed.

## Evidence

- `docs/fixtures/nrs-reference-v0.1/reference_manifest.json`
- `docs/fixtures/nrs-reference-v0.1/README.md`
- `context/waves/2026-07-09-national-standard-evidence-and-specification/panels/pulse-03-reproducibility-review.md`

## Closure Rule

Closed as an internal clean-source functional reference replay. The documented
base commit, runtime overlay, config, toolchain, input hashes, and commands
reconstruct the tested source and expected assignment. External replication,
block-level conformance, and public data custody remain explicit carry-forwards.
