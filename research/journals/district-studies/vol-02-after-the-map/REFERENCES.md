---
journal: District Studies
volume: 2
title: "After the Map"
status: audition-references
updated: 2026-05-09
---

# References

This is the audition reference layer for Vol. 2. It records which O-track
sources may support the scope-down lineup and which claims remain gated.

## Primary Sources

| Ref | Source | Used for | Publication status |
|-----|--------|----------|--------------------|
| [O0] | `research/tracks/O-outcomes-representation/O.0+outcomes-overview/` | Framework linking map geometry to downstream outcomes | Frame only; avoid proof language |
| [O1] | `research/tracks/O-outcomes-representation/O.1+electoral-competitiveness/` | Competitiveness outcome candidate | Strong audition source; exact values gated by source-chain and variance notes |
| [O2] | `research/tracks/O-outcomes-representation/O.2+voter-turnout/` | Turnout outcome candidate | Suggestive only; underpowered sample |
| [O3] | `research/tracks/O-outcomes-representation/O.3+legislative-polarization/` | Polarization outcome candidate | Projection only; no causal claim |
| [O4] | `research/tracks/O-outcomes-representation/O.4+constituent-distance/` | Constituent-distance anchor | Strongest preview anchor; exact values gated by routing/address provenance |
| [O5] | `research/tracks/O-outcomes-representation/O.5+representation-quality-index/` | RQI synthesis candidate | No legal-testimony or threshold language in District Studies |

## Review And Gate Sources

| Gate | Source | Effect |
|------|--------|--------|
| O-track panel review | `research/tracks/O-outcomes-representation/REVIEW_PANEL.md` | Ranks O.1/O.4 strongest; flags O.2/O.3/O.5 |
| Outcome claim audit | `reviews/outcome-claim-audit.md` | Blocks causal and legal overclaims |
| O.1 source audit | `reviews/o1-competitiveness-source-audit.md` | Blocks exact O.1 headline values |
| O.4 source audit | `reviews/o4-distance-source-audit.md` | Blocks exact O.4 travel-time values |
| Scope-down lineup | `decisions/scope-down-lineup.md` | Orders the issue around O.4, O.1, and cautionary synthesis |

## Citation Rule For Vol. 2

Article drafts should cite sources by bracketed reference ID, for example
`[O4]` or `[O1]`. Exact numeric values should stay out of public preview until
the corresponding source-chain gate is closed.
