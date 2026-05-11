# Goal: RPLAN Golden Package Corpus and Verifier Bridge

**Status:** Active - ready to begin  
**Date:** 2026-05-11  
**Owner:** BISECT / RPLAN integration  
**Depends on:** `docs/specs/2026-05-11-algorithm-family-paper-writing-goal.md`

## Goal

Turn the RPLAN/RCTX/audit substrate into a reusable evidence layer by creating
a small golden package corpus, wiring verifier behavior through both public
CLIs, expanding negative fixtures, and upgrading the U.16-U.20 evidence notes
where the examples now demonstrate executable behavior.

The target is not bigger algorithms. The target is a stable final-plan package
contract that every algorithm family can land on.

## Stage 1 - Package Contract

- [ ] Define the canonical package directory shape:
      `plan.rplan`, `context.rctx`, `audit-certificate.json`, `manifest.json`,
      optional method transcript, and optional solve/search report.
- [ ] Add a short package schema note under `docs/file-formats/` or
      `docs/examples/`.
- [ ] Decide whether package manifests stay example-local for now or become a
      crate-owned schema later.
- [ ] Ensure package docs distinguish file integrity, certificate verification,
      audit result, and legal/policy non-claims.

## Stage 2 - U.20 Negative Fixture Catalog

- [x] Add initial public valid 3x3 package.
- [x] Add CLI coverage for missing context, plan tamper, context hash change,
      and canonical unit-order mismatch.
- [ ] Add profile mismatch fixture.
- [ ] Add stale RCTX fixture.
- [ ] Add unsupported-constraint or missing-input fixture.
- [ ] Add lineage-reserved-field or broken-lineage fixture if exposed through a
      public package surface.
- [ ] Document each failure mode with expected command and expected failure
      class.

## Stage 3 - Golden Package Corpus By Family

Add one tiny public package per family, with tests that verify the package:

- [ ] T.14 spectral construction package.
- [ ] T.15 capacity-constrained clustering package.
- [ ] T.16 hierarchical regionalization package.
- [ ] T.17 flow construction package.
- [ ] U.16 branch-and-cut package.
- [ ] U.17 branch-and-price package.
- [ ] U.18 local-search improvement package with parent/child lineage.
- [ ] U.19 selected-frontier package.
- [ ] U.20 audit-certificate reference package.

Acceptance for each package:

- [ ] Package files are committed under `docs/examples/` or a documented
      fixture corpus path.
- [ ] Package has a manifest with SHA-256 file hashes.
- [ ] `rplan verify-certificate` accepts the package.
- [ ] If the package comes from a method family, lineage metadata names the
      producer family and method.
- [ ] Documentation states what the package does and does not prove.

## Stage 4 - Verifier Bridge

- [ ] Compare `rplan verify-certificate` and existing `bisect verify` behavior.
- [ ] Add or update `bisect verify` so it can consume the same package shape, or
      document why `rplan` is the neutral verifier and `bisect` delegates.
- [ ] Add L1 tests showing both public surfaces agree on at least one package.
- [ ] Update CLI help/docs with the fixed-point workflow:
      algorithm output -> package -> certificate verification -> report.

## Stage 5 - Evidence Upgrades

- [ ] Update U.16-U.20 manuscript evidence tables where public packages now
      exist.
- [ ] Rebuild affected PDFs and update `docs/PAPERS.md` notes if evidence level
      changes.
- [ ] Update `docs/concepts/algorithm-family-layer-cake.md` with the golden
      package corpus if the taxonomy changes.
- [ ] Update `docs/NEXT_SESSION.md` with any remaining empirical/package work.

## Stage 6 - Review, Commit, Push

- [ ] Run focused Rust tests for touched crates.
- [ ] Run `git diff --check`.
- [ ] Commit coherent slices.
- [ ] Push to `origin/main`.

## Suggested `/goal`

```text
/goal Build the RPLAN golden package corpus and verifier bridge described in docs/specs/2026-05-11-rplan-golden-package-corpus-goal.md: define the package contract, expand U.20 negative fixtures, add one tiny verified package per T/U algorithm family, bridge rplan/bisect verification where appropriate, update evidence tables and docs, verify, commit, and push stage by stage.
```
