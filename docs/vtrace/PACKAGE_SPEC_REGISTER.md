# Package Specification Register

## Scope

This register resolves DREQ-004 for the internal VTRACE baseline by routing
RPLAN, RCOUNT, RCTX, and RHIST schema and canonicalization questions to their
current package specs, crate constants, verifier paths, and change-control
triggers.

This is a control artifact. It does not restate full schemas, create a new
package version, publish package evidence, close public interoperability gates,
or upgrade the current `internal_engineering_baseline_only` posture.

Source controls:

- `REQUIREMENTS.md` DREQ-004.
- `PACKAGE_BOUNDARIES.md` PKG-004 through PKG-006.
- `INTERFACES.md` IF-003 through IF-005.
- `WORK_PACKAGES.md` WP-004.
- `IMPORT_COMPATIBILITY.md` DCR-005 package/interoperability posture.

## Register

| Family | Boundary / interface | Current spec source | Version identity | Hash shape | Verifier path | Evidence posture |
|---|---|---|---|---|---|---|
| RPLAN | PKG-004 / IF-003 | `docs/specs/2026-05-10-rplan-v0.2-schema.md`; `crates/rplan-core`; `crates/rplan-io`; `crates/rplan-audit` | `RPLAN_V02 = "0.2"`; `DISTRICT_PLAN_SCHEMA_VERSION = "district-plan-v1"`; audit certificate schema `audit-certificate-v1`; legal profile schema `legal-profile-v1` | Plan/context canonical bytes and audit certificate content hashes in `rplan-audit`; RPLAN import compatibility bounded by `rplan-io` v0.2 plus v0.1 compatibility input | `rplan audit --plan <file> ...`; `rplan verify-certificate --certificate <file> --plan <file> [--context <file>]`; library functions `audit_plan_with_lineage` and `verify_audit_certificate` | L1 crate/package tests plus DCR-001 fixture-backed label import; broader public exchange claims require selected fixtures or package review. |
| RCOUNT | PKG-005 / IF-004 | `docs/specs/2026-05-12-rcount-substrate.md`; `docs/specs/2026-05-12-rcount-incubation.md`; `docs/specs/2026-05-13-rcount-audit-algorithm-roadmap.md`; `crates/rcount-core`; `crates/rcount-io`; `crates/rcount-audit` | `RCOUNT_VERSION = "0.1-draft"`; audit transcript schema `rcount-audit-transcript-v1`; algorithm IDs such as `rcount-sha256-modulo-v1`, `colorado-rule-25-comparison-v1`, and `california-public-rla-v1` | Domain-separated SHA-256 prefixes for source, record, file, package, status event, proof, RLA manifest, and RLA sample records (`RCOUNT_*_V1`) | `rcount verify <package_dir>`; `rcount replay-audit-algorithms <package_dir>`; `rcount aggregate-districts <package_dir> --plan <path>`; import commands for statement CSV, NIST CDF JSON, and RI 2024 Rep 28 RLA fixtures | L1 synthetic/package tests preserve election semantics; jurisdiction/vendor replay and public golden package promotion remain selected L2 work. |
| RCTX | PKG-006 / IF-005 | `docs/specs/2026-05-13-rctx-boundary.md`; `crates/rctx-core` | `RCTX_VERSION = "0.1"` | Domain-separated SHA-256 prefixes for crosswalk, crosswalk set, and package records (`RCTX_*_V1`) | Library functions `verify_package` and `verify_crosswalk_input`; downstream verifier use through RPLAN/RCOUNT/RHIST package tests until a dedicated CLI is selected | L1 core tests cover source index, graph, crosswalk, and claim-boundary semantics; public context packages require fixture promotion or downstream scenario review. |
| RHIST | PKG-006 / IF-005 | `docs/specs/2026-05-13-rhist-boundary.md`; `docs/specs/2026-05-13-rhist-implementation.md`; `crates/rhist-core`; `crates/rhist-io`; `crates/rhist-cli` | `RHIST_VERSION = "0.1"` | Domain-separated SHA-256 package prefix `RHIST_PACKAGE_V1`; package IO verifies source files and package hashes | `rhist verify <package_dir>`; library functions `verify_package`, `verify_package_hash`, and `verify_source_files` | L1 core/IO/CLI tests cover unit history, lineage, crosswalks, source references, and claim boundaries; public historical lineage claims require selected source fixtures. |

## Change-control triggers

Update this register in the same change as any of the following:

1. A package version, schema version, audit-certificate version, transcript
   version, status vocabulary, or algorithm identifier changes.
2. A canonicalization or hash prefix changes.
3. A verifier command, verifier output status, or package audit acceptance rule
   changes.
4. A package family adds a public fixture, public evidence package, external
   adapter, or downstream compatibility claim.
5. A package boundary moves between RPLAN, RCOUNT, RCTX, RHIST, BISECT, or the
   shared kernel crates.

## Claim and custody rules

| Rule | Disposition |
|---|---|
| Full schemas remain in package specs and crates | This register is a routing surface, not a duplicate schema definition. |
| Hash compatibility requires named version and prefix | Version-unknown or prefix-unknown package artifacts are findings, not compatibility evidence. |
| Verifier success is package-scope only | A package audit result does not imply legal plan approval, official election certification, public release readiness, or court admissibility. |
| Public package examples need custody review | Synthetic fixtures may be committed with verifier path and negative coverage; externally sourced packages need VAULT/source disposition. |
| Broader interoperability requires selected evidence | DCR-001/DCR-005 cover only named fixture-backed import surfaces unless a later pulse promotes more fixtures. |

## Current DREQ-004 disposition

DREQ-004 is controlled at L1 by this register. Exact package schemas and
canonicalization algorithms remain owned by the package specs and implementation
constants above; this register supplies the operator-facing map from VTRACE
requirements to those sources of truth.

No public package publication, external interoperability expansion, legal/court
readiness, official certification, or S6 readiness upgrade is claimed.
