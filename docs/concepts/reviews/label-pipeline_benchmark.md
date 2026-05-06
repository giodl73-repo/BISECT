# Review: label-pipeline.md
**Reviewer**: BENCHMARK (test-engineer)
**Date**: 2026-05-05

---

## What's accurate

The `redist label-verify` command is the core testable assertion in this guide, and its three-status output (VERIFIED / MISMATCH / MISSING) maps cleanly to assertions that a test suite can make. The guide correctly describes what each verb produces and reads: `redist build` writes `final_assignments.json` and `index.json`; `redist label-analyze` reads from `runs/` and writes to `analysis/`; `redist label-report` reads from `analysis/` and writes to `reports/`. These input/output contracts are specific enough to write integration tests against. The `redist mv` atomic-rename description — updating registry, all filesystem directories, `configs/{label}.yml`, and every `index.json` — is precise enough that a test could verify all five artifacts are updated and none are left in an inconsistent state.

---

## P1 — Required fixes

**No test coverage described for chain verification**: The guide presents `redist label-verify` as the chain integrity tool, but does not describe any test that would catch a regression in the chain itself. If the SHA-256 embedding in `index.json` were broken — e.g., the config hash is computed from the wrong file, or the binary truncates the hash — `label-verify` would silently return `MISMATCH` for every state, or worse, `VERIFIED` because the comparison is against a wrong hash embedded at build time. BENCHMARK asks: what test exists that would catch this bug? The guide should reference (or the codebase should contain) a test that: (1) builds a known config, (2) tampers with one byte of `final_assignments.json`, and (3) asserts that `label-verify` reports `MISMATCH`. Without this test, the chain verification is unverified infrastructure.

**`redist label-import` — no test for rejected plans documented**: The guide states that a plan violating the 0.5% population tolerance "will be rejected at import time." This is an important invariant. BENCHMARK asks: is there a test that constructs a deliberately imbalanced CSV, runs `redist label-import`, and asserts the rejection error? Without this test, the import validation is asserted only in prose, not in code. An import that silently accepts a 1.5%-imbalanced plan would corrupt the chain with a plan that misrepresents its population balance.

**`redist mv` — atomicity is untested by any documented test**: The atomic rename with rollback is described as a guarantee. BENCHMARK asks: what test verifies that a failure partway through the rename — e.g., the registry update succeeds but a filesystem rename fails — triggers rollback? Without a test that simulates mid-rename failure, the atomicity guarantee is a claim, not a measurement.

**Missing label registry validation tests**: The guide states that `redist config` will "reject a config with a mismatched name." This is a specific, testable invariant. BENCHMARK expects to find a unit test that creates a config file with a mismatched `name:` field and asserts the validator returns an error. If this test does not exist, the validation is undiscovered debt: any code change that removes or bypasses the check will not be caught by CI.

**`redist ls` output format**: The guide shows an exact example of `redist ls` tabular output with column headers `LABEL CONFIG BUILD ANALYSIS REPORTS`. This is a specific expected output format. If the output format changes — column order, spacing, added columns — any documentation consumer (scripts that parse this output, court staff following the guide's workflow) will silently break. A snapshot test for `redist ls --format` output would catch format regressions.

---

## P2 — Suggested improvements

The "Creating a new label" workflow at the end of the guide is a 5-step sequence that amounts to an end-to-end integration test scenario. This workflow should be referenced from the test suite as an acceptance scenario: a CI job that runs these five commands against a small test state (Vermont, Delaware) and asserts that `label-verify` returns `VERIFIED` for all links would give high confidence that the full label pipeline is functioning. The guide could include a note pointing to such a CI scenario.

The three status values for `label-verify` (VERIFIED / MISMATCH / MISSING) should be tested in isolation: a test for VERIFIED (hash matches), a test for MISMATCH (hash does not match after file modification), and a test for MISSING (stage not yet run). Three distinct tests, not one happy-path test that only exercises VERIFIED.

---

## Score: 2/4

The guide is clear about what the label pipeline does, but makes multiple guarantees — chain integrity, import validation, atomic rename — that appear to lack corresponding tests. A guide used to certify redistricting plans in legal proceedings must have machine-verifiable evidence for every guarantee it describes. The absence of documented tests for chain tampering detection and import validation is not a gap in the documentation; it is a gap in the proof that these features work.
