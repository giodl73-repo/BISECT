---
skill: roles-check
topic: wasm-manifest-reader-width
date: 2026-10-27
roles_used: 6
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

Standard single-agent application of installed role lenses, not independent
signatories. Artifact: four read-only manifest consumers, shared ILP binding,
native/browser comparisons, builder registration and staged package. Selected
BENCHMARK for measurement, COVENANT for provenance, LEDGER for compatibility,
VAULT for source integrity, SURVEY for practitioner controls and TRENCH for
structural failures. Algorithm and legal-policy rules are not changed.

| Role / ID | Finding | Severity | Reference | Recommendation / disposition |
|---|---|---|---|---|
| BENCHMARK B1 | Gate depended on a historical generated fixture under target. | P2 | test_manifest_reader_width.mjs | Resolved: derive it from tracked example files, verify hashes and retain fixture identities. Both corrected native runs pass. |
| BENCHMARK B2 | A local manual test would not guard ordinary builds. | P2 | both preview/legacy builders | Resolved by registering native comparisons and optional browser checks, with source pins and receipts. Whole-builder execution is still unproven. |
| BENCHMARK B3 | Native dry-run checks cover selected branches, not complete Verify or Report execution. | P3 | 18 CLI outcomes/profile | Preserve the exact scope; do not accept broader parents. |
| COVENANT C1 | Existing native reference binaries are hashed but were not rebuilt by this gate. | P2 | native/cli receipt pins | Disclose this distinction; current native CLI compile and six binding tests pass separately. Reproducible release provenance remains open. |
| COVENANT C2 | Mutable working paths alone cannot retain the tested identities. | P2 | closure requirement | Retain exact source/runtime/package evidence and independently read every mapped blob before accepting this checkpoint. |
| COVENANT C3 | A valid certificate binding does not authenticate the producer or establish legal validity. | P3 | operation scope strings | Preserve existing disclosures and source hashes. |
| LEDGER L1 | wasm32 usize rejected native 64-bit metadata and silently removed summary availability. | P2 | four readers, shared schema | Resolved with the shared u64 read-only view; all 13 fields reach successful downstream checks. |
| LEDGER L2 | Native and adapter invalid-number diagnostics name different Rust types. | P3 | native comparison helper | Normalize only expected usize/u64 spelling, retain raw native output and compare all other fields exactly. |
| LEDGER L3 | Native execution aliases must keep their original schema/domain. | P3 | PlanManifest / ManifestIlpBindings aliases | Retained usize aliases and explicit native Verify binding; CLI compile passes. |
| VAULT V1 | Parsing metadata must not grant paths or increase selected-file budgets. | P2 | selected-byte adapters | Existing safe-path, collision, byte and file limits remain in force; no ambient filesystem/URL access added. |
| VAULT V2 | Numeric roundtripping through JS would invalidate exact source identity. | P2 | browser requests / saved sources | Resolved for these paths: original manifest bytes remain encoded and exact; actual worker requests and saved sources match references. |
| VAULT V3 | Hashes prove identity rather than authenticity or malware resistance. | P3 | package and source receipts | Keep the evidence claim bounded; no new privacy/security certification. |
| SURVEY S1 | Open must not reactivate imported verification results or execute automatically. | P2 | 72 workbench cases/profile | Resolved: no worker run on Open; imported download controls inactive until explicit Run. |
| SURVEY S2 | Results need usable exports and repeatable saved inputs. | P3 | verification receipts, summary table/CSV, projects | Actual downloads and source-preserving Save match native references. Discovery retains its result in the saved project. |
| SURVEY S3 | Project-file inputs do not prove a granted OS directory picker. | P3 | browser scope | Retain this separate host/API acceptance gap. |
| TRENCH T1 | A wide ILP count could be truncated or rejected before native mismatch ordering. | P2 | shared ILP binding / actual audit fixtures | Resolved: checked conversion and comparison, exact mismatch through u64::MAX, certificate stage remains not-run after failure. |
| TRENCH T2 | Fixing selected readers could be mistaken for a global portability guarantee. | P2 | PP-72 / execution-coupled readers | Added the distinct host-width pitfall as mitigated; replay/allocation consumers remain individually reviewable and open. |
| TRENCH T3 | Staging hardlinks can corrupt a prior package if overwritten in place. | P2 | staged package / browser start/end manifest pins | Mutable destinations were unlinked before replacement; preserve linked binaries and sealed package bytes. Independent retained readback is required. |

Roles reviewed: 6. P1 blockers: 0; P2 issues: 11; P3 notes: 7.
Verdict: APPROVED-WITH-CONDITIONS for the bounded implementation/local package.
No parent, full builder, host matrix, reproducible release or deployment signoff.

Top finding: metadata domain portability must remain separate from allocation and
execution authority. BENCHMARK, LEDGER and TRENCH agree that successful downstream
checks and exact failure ordering are needed, rather than parser success alone.

Amendments:

1. Replace the historical target fixture with tracked, hash-checked examples
   (done; corrected native comparisons and browser runs pass).
2. Register affected tests in both builders and preserve partial broader states
   (done; syntax checks pass, complete builder run remains open).
3. Seal and independently verify the complete bounded evidence/package inventory;
   run ordinary ledger and negative full-parity gates before closure. Broader
   requirements remain open regardless of this checkpoint's results.
