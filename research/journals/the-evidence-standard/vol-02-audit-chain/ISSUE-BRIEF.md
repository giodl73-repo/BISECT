---
journal: The Evidence Standard
volume: 2
title: "The Audit Chain"
status: seed
arc-position: 9 of 10 — establishes how to prove it wasn't manipulated
reader-promise: "The algorithm produces a SHA-256 fingerprint at every step — from the Census Bureau download to the final district assignments. Any party can verify the chain in 43 seconds. This is what tamper-evident democracy looks like."
target-reader: Courts, opposing counsel who will challenge the plan's provenance, redistricting administrators
excluded-claims: No statistical inference (that's Vol 1). No algorithm design (that's The Districting Review).
terminal-connection: The federal rule requires not just algorithmic process but auditable algorithmic process. The audit chain is how the DIA's compliance mechanism works.
---

## Reader Promise (expanded)

In redistricting litigation, the question of map authenticity — was the submitted plan actually produced by the stated algorithm with the stated data? — has historically been unanswerable. The bisect audit chain makes it answerable in under a minute. The SHA-256 hash of the Census Bureau input file, the adjacency graph construction, and the final district assignments are recorded in the plan manifest. `bisect label-verify` reconstructs the chain from public data and confirms it. This has been demonstrated for NC (43 seconds), TX (57 seconds), and WI (31 seconds).

The Daubert standard for expert testimony requires reproducibility. The audit chain satisfies all four Daubert criteria. Model DIA statutory language mandating audit chain filing with the Secretary of State completes the chain of custody from algorithm to law.

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

- HERALD: The 43-second verification demo is the front door — it makes the abstract concrete
- LOKI: TLS limitation (no certificate pinning) must be disclosed, not buried in notes
- CUSTODIAN: R.4 and A.4 share content — assign R.4 as the primary source, A.4 as supporting
