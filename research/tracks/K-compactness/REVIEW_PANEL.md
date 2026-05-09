# Track K — Compactness Panel Review
**Date**: 2026-05-09 | **Round**: 1
**Papers**: K.0–K.7 (8 papers)
**Reviewers**: Polsby, Duchin, Karypis, Stephanopoulos, Chen

## Paper Scores (Round 1)

| Paper | Title | Score | Verdict |
|-------|-------|-------|---------|
| K.0 | Taxonomy Overview | 3.0/4 | Conditional Accept |
| K.1 | Polsby-Popper | 3.2/4 | Conditional Accept |
| K.2 | Reock | 2.8/4 | Conditional Accept |
| K.3 | Convex Hull | 2.8/4 | Conditional Accept |
| K.4 | Schwartzberg | 3.0/4 | Conditional Accept |
| K.5 | Length-Width | 2.6/4 | Conditional Accept (borderline) |
| K.6 | Population-Weighted | 3.2/4 | Conditional Accept |
| K.7 | Composite Court Guide | 3.6/4 | Conditional Accept (strong) |
| **Track Mean** | | **3.0/4** | |

## Module Score: 7.8/10

## Track-Level Strengths
- K.7 is the track's crown jewel — the multi-metric composite court guide fills a genuine practitioner gap. No prior document synthesises all six major compactness metrics into a single expert-witness reference with litigation strategy guidance.
- K.1's empirical grounding (PP variation across 50 states, relationship to enacted maps) is particularly strong.
- K.6's population-weighted compactness is the most methodologically innovative paper — it addresses the well-known limitation of area-based metrics in sparse rural states.

## P1 Items by Paper
- **K.2 (Reock)**: The minimum bounding circle computation is stated without specifying the algorithm. Multiple algorithms exist (Welzl, Megiddo) with different numerical properties. State which is used in the bisect implementation.
- **K.3 (Convex Hull)**: The paper uses convex hull ratio (area of district / area of convex hull) but does not address the known issue that non-simply-connected districts (donut-shaped) can have hull ratios > 1. Address or exclude non-simply-connected configurations.
- **K.5 (Length-Width)**: The definition of "length" and "width" is not standardized across jurisdictions. Some courts use bounding box; others use minimum-width calipers. The paper should present both definitions and note which is implemented.
- **K.7**: The composite profile lacks a weighting scheme. The paper presents six metrics as equal contributors but does not justify equal weighting vs. case-law-driven weighting. Add a sensitivity analysis showing how composite rankings change under different weights.

## P2 Items
- K.0: Cross-reference A's COMPACTNESS-BASELINE-TABLE.md — the canonical reference for program-wide compactness figures.
- K.7: Add a decision tree: "If opposing counsel cites metric X to argue your map is not compact, use metric Y to respond because…"

## Next Action
K.7 P1 (weighting sensitivity) is highest priority — it is the paper most likely to face cross-examination on the weighting choice. K.2 Reock algorithm specification is a reproducibility requirement.
