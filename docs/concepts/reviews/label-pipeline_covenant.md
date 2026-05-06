# Review: label-pipeline.md
**Reviewer**: COVENANT (audit-evidence-expert)
**Date**: 2026-05-05

---

## What's accurate

The SHA-256 four-stage audit chain diagram is structurally correct: config → build output → analysis output → report manifest forms a tamper-evident chain where each stage hashes the prior stage's primary artifact. The `redist label-verify` command's three-status output (VERIFIED / MISMATCH / MISSING) is a well-designed chain-verification surface, and the guide correctly states that any MISMATCH "should be investigated before the plan is used in a legal proceeding" — exactly the right framing. The `redist mv` atomic-rename with rollback is important for maintaining chain integrity during administrative label changes, and the guide correctly describes it as atomic with rollback. The `redist label-import` section correctly notes that imported civic counter-proposals receive the same `index.json` structure as built plans, which preserves the chain structure for third-party plan submissions. The `submission_type: civic-counter-proposal` field is the right disclosure mechanism.

---

## P1 — Required fixes

**SHA chain: upstream Census data not hashed**: The guide describes the chain as beginning with `configs/{label}.yml`. This means the census input data — the actual TIGER shapefiles and demographic files downloaded from the Census Bureau — is not part of the SHA-256 chain. A court-appointed special master reviewing this chain can verify that the config was not modified and that the build was not tampered with, but cannot verify that the input data used was the authentic Census Bureau release. Opposing counsel can argue that the chain proves only internal consistency, not provenance from a certified government source. The guide should either (a) document how Census input file SHAs are recorded and where they appear in the chain, or (b) explicitly disclose that the chain begins at the config, not the Census source, and explain what separate process provides Census data provenance.

**Section "`redist label-report`", Typst path**: The guide states "For installation, see `redist/docs/typst-templates/README.md`." The actual path is `redist/crates/redist-report/typst-templates/README.md`. A broken path in the court-submission workflow is a procedural failure point: a legal staff member following this guide to produce a PDF report will not find the Typst installation instructions and may conclude the feature is unavailable or broken. This is a straightforward factual error that must be corrected.

**Section "`redist label-verify`"**: The guide does not specify what the binary version recorded in `index.json` represents — whether it is the `redist` binary version, a Cargo workspace version, or a Git commit hash. For court use, the binary version is a critical chain link: if the same source code can produce a different binary, the binary version alone does not identify the computation. The guide should state that `index.json` records the `redist` binary version string AND the build commit hash (if available), so that independent verifiers can obtain and run the same binary.

**Section "Config YAML format"**: The guide states that `redist config validate` will "reject a config with a mismatched name." However, the guide also says the label pipeline spec is in `docs/superpowers/specs/`, which is incorrect per project standards — specs live in `docs/specs/`. This stale path does not directly affect audit integrity, but a user following the internal cross-reference to understand the design rationale for the name-matching requirement will be sent to the wrong location.

**`redist label-import` — contiguity and population check**: The guide says a plan that "fails contiguity or violates the 0.5% population tolerance will be rejected at import time." This is critical for chain integrity: an imported plan that passes internal validation but was generated from different population data than the built plans would have different population totals. The guide should specify which population source is used for import validation (the same Census tract file used in the build, identified by its SHA?) and what error message or verification step is produced when population totals differ because of different source data vintage.

---

## P2 — Suggested improvements

The `.redist` registry file is mentioned as storing all labels but is never described structurally — is it a TOML file, a JSON file, a SQLite database? For a court-facing tool, the format of every metadata artifact should be documented so that independent parties can read it without running `redist` itself.

The guide would benefit from a "Chain integrity FAQ" section addressing the most common adversarial scenarios: (1) What happens if a build is re-run for the same label? (2) Does re-running overwrite the existing chain or create a new one? (3) Can two different build runs of the same config produce different chains? These are the exact questions opposing counsel will ask.

---

## Score: 2/4

The guide describes the SHA chain mechanism correctly at the structural level, but has three substantive gaps that matter for legal use: Census data provenance is not in the chain, the binary version record is underspecified, and the Typst path is broken. These are not cosmetic issues — they are the difference between a chain that survives adversarial scrutiny and one that can be challenged on procedural grounds before the merits are reached.
