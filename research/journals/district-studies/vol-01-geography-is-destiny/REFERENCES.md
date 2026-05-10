---
journal: District Studies
volume: 1
title: "Geography Is Destiny"
status: preview-references
updated: 2026-05-09
---

# References

This is the preview reference layer for Vol. 1. It points to the source
artifacts used by the public preview and records which claims remain gated.

## Primary Sources

| Ref | Source | Used for | Publication status |
|-----|--------|----------|--------------------|
| [B12] | `research/tracks/B-algorithm/B.12+proportional-section/` | Geography/proportionality mechanism; qualitative state-type frame | Use qualitatively only; exact table values and paper-run parameters remain gated |
| [G1] | `research/tracks/G-ensemble/G.1+gerrychain-congressional-comparison/` | Cautious ensemble-position example | Use ESS-safe language only; no exact percentile claim |
| [L6] | `research/tracks/L-partisan-fairness/L.6+proportionality-majoritarianism/` | Supporting explanation of proportionality versus majoritarian outcomes | Supporting source only |
| [B11] | `research/tracks/B-algorithm/B.11+apportion-regions/` | Deferred support for NC/TX and ApportionRegions context | Not a headline Vol. 1 source |
| [L1] | `research/tracks/L-partisan-fairness/L.1+efficiency-gap/` | Metric definition and sign-convention context | No result-bearing numeric claims |
| [C5] | `research/tracks/C-validation/C.5+efficiency-gap-analysis/` | Deferred measurement candidate | No result-bearing numeric claims |

## Review And Gate Sources

| Gate | Source | Effect |
|------|--------|--------|
| B.12 source-chain audit | `reviews/b12-reproducibility-and-scope-audit.md` | Blocks exact B.12 table values |
| B.12 implementation provenance | `reviews/b12-implementation-provenance-note.md` | Blocks inference from current defaults to paper-run parameters |
| G.1 SCALE/DATUM review | `reviews/g1-scale-datum-review.md` | Permits "near the middle" / preliminary ensemble language |
| L.1 efficiency-gap sign audit | `reviews/l1-efficiency-gap-sign-audit.md` | Blocks L.1/C.5 result-bearing claims until post-fix output provenance closes |
| Scope-down decision | `decisions/scope-down-path.md` | Keeps Vol. 1 diagnostic and qualitative |
| Final lock review | `reviews/final-lock-review.md` | Holds issue at provisional public preview |

## Citation Rule For Vol. 1

Article drafts should cite sources by bracketed reference ID, for example
`[B12]` or `[G1]`, and should not cite exact numeric claims unless the
corresponding gate above is closed.
