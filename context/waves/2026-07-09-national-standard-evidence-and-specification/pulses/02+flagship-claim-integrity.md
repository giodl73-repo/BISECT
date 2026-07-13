---
pulse: 02
title: Flagship claim integrity
status: done
depends_on: 01
wave: national-standard-evidence-and-specification
validation_level: L1 claim audit
---

# Pulse 02 - Flagship Claim Integrity

## Purpose

Make the public front door and flagship papers describe the same bounded claim
and evidence posture as the canonical standard.

## Required Scope

- `README.md`
- `docs/PAPERS.md`
- `docs/legal/FAIRNESS_DOCTRINE.md`
- `research/tracks/A-synthesis/A.0+synthesis-metapaper/`
- `research/tracks/B-foundations/B.02+one-federal-law/`
- `research/tracks/C-validation/C.6+user-study/`
- `docs/vtrace/PAPER_EVIDENCE_INVENTORY.md`

## Deliverables

- [x] Reconcile paper and track counts.
- [x] Replace unconditional "cannot gerrymander" language with an
      execution-stage, input-bounded claim.
- [x] Align VRA opportunity, compliance, and legal-conclusion language.
- [x] Resolve C.6's study status and sample-size inconsistency.
- [x] Mark synthetic, preliminary, internal-review, and missing-real-evidence
      claims consistently.
- [x] Remove or replace placeholder flagship figures.

## Validation

Run claim searches, rebuild every changed paper, and run:

```powershell
git --no-pager diff --check
```

Result: passed 2026-07-09. A.0, B.02, and C.6 rebuilt successfully with
pdfLaTeX and BibTeX; published PDFs copied to `docs/papers/`.

## Evidence

- `context/waves/2026-07-09-national-standard-evidence-and-specification/panels/pulse-02-flagship-claim-review.md`
- `docs/vtrace/PAPER_EVIDENCE_INVENTORY.md`
- Revised A.0, B.02, and C.6 source and PDF artifacts

## Closure Rule

Closed at L1 claim-audit level. The reviewed flagship sources no longer present
synthetic, planned, preliminary, or internally reviewed evidence as completed
external empirical validation.
