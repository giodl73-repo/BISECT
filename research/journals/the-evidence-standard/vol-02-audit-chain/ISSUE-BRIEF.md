---
journal: The Evidence Standard
volume: 2
title: "The Audit Chain"
status: seed
arc-position: 9 of 10 — establishes how to prove it wasn't manipulated
reader-promise: "A redistricting plan should come with provenance. This volume tests the SHA-256 audit chain from input data to final district assignments."
target-reader: Courts, opposing counsel who will challenge the plan's provenance, redistricting administrators
excluded-claims: No statistical inference (that's Vol 1). No algorithm design (that's The Districting Review).
terminal-connection: A candidate federal rule would need not just algorithmic process but auditable algorithmic process. This volume tests that compliance mechanism.
---

> Seed status: verification timing, Daubert framing, and statutory filing language require COVENANT and BOUNDARY review before publication.

## Reader Promise (expanded)

In redistricting litigation, map authenticity matters: was the submitted plan actually produced by the stated algorithm with the stated data? The Bisect audit chain is designed to make that question testable. The SHA-256 hash of the input file, adjacency graph construction, and final district assignments are recorded in the plan manifest. `bisect label-verify` reconstructs the chain from public data and confirms it. Runtime examples are candidates until the exact hardware, data, and command context are documented.

The Daubert standard for expert testimony makes reproducibility relevant. This volume auditions the argument that the audit chain supports admissibility and proposes model filing language for review.

## Slots (6-8 candidates, 2-3x pool)

| Slot | Role | Claim class | Candidate papers |
|------|------|-------------|-----------------|
| Editorial | The authenticity problem — why "trust us" doesn't work in court | interpretive | — |
| Formula | The three-level audit chain — download hash → adjacency hash → plan hash | formal/operational | R.4 (audit mechanisms), A.4 §verification |
| Verification demo | `bisect label-verify` for NC/TX/WI — 43/57/31 seconds | empirical | R.4 §label-verify-demo |
| Daubert analysis | All four criteria satisfied — testing, peer review, error rate, acceptance | formal | R.4 §daubert |
| TLS note | Certificate pinning not implemented — TLS-only limitation disclosed | operational | R.2 §tls-limitation |
| Model statute | DIA audit filing language — Secretary of State filing, 10-year retention, public verification | operational | R.4 §legal-language, P.1 §audit-provision |
| Archive note | The special master authenticity problem the audit chain replaces | archival | P.4 §special-master-precedent |

## Review lenses

- COMMONS: The 43-second verification demo is the front door — it makes the abstract concrete
- COVENANT: TLS limitation (no certificate pinning) must be disclosed, not buried in notes
- DATUM: R.4 and A.4 share content — assign R.4 as the primary source, A.4 as supporting
