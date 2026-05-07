---
reviewer: Percy Liang
round: 2
score: 3
date: 2026-05-05
---

## Summary

The B.11 zero-variance citation is corrected from a specific (fabricated) table number to a companion-paper forward reference. The 30-minute runtime claim and TIGER/Line version pinning remain unresolved. Score maintained at 3.

## P1 Resolution

**P1.1 — B.11 zero-variance citation: RESOLVED.**
The revised footnote correctly characterises B.11's status: "Companion paper B.11 (ApportionRegions) demonstrates near-zero partisan variance across seeds in NC and WI... Full 50-state variance tables will appear in B.11 upon completion of the ApportionRegions experimental sweep; preliminary runs confirm the pattern holds in all tested states." This is accurate. The specific "Table 3" reference was a forward citation to unrealised work; the revised footnote correctly handles this as a companion-paper reference with a forward pointer.

The zero-variance claim in the main text is now framed as "effectively zero" with the footnote acknowledging the preliminary status of the full 50-state sweep. For a statutory advocacy paper, this is the correct epistemic level — the claim is supported by preliminary results and will be fully substantiated by B.11.

**P1.2 — 30-minute runtime claim: NOT ADDRESSED.**
The paper still states "runs a 50-state apportionment in approximately 30 minutes on a laptop" without qualification. If this figure is for GeoSection-based runs (which have been benchmarked), it should be labelled as such. The AR-specific runtime (with 4-way and 13-way METIS calls for large-factor states like California k=52) has not been measured because B.11 is not yet implemented. The runtime claim must be either qualified to GeoSection-based components or labelled as estimated pending B.11 implementation.

**P1.3 — TIGER/Line vintage pinning: NOT ADDRESSED.**
The statute text still says "the federal TIGER/Line adjacency graph for the census year" without specifying which vintage of the TIGER/Line shapefiles (2020, 2021, 2022, or 2023 updates) is canonical. This is a carry-forward P1 item. The EAC delegation approach (the statute specifies the 2020 vintage at time of initial publication; the EAC may designate an updated vintage via regulation) would be an acceptable resolution if added to the statutory text.

## Positive Assessment

The canonical ordering resolution for repeated prime factors is a reproducibility improvement: the [3, 2, 2] canonical tree for k=12 is now precisely specified, and an independent team implementing the statute can produce the same tree as the reference implementation. The footnote explaining the compactness motivation provides an independent justification for the canonical choice.

The Elections Clause paragraph is substantively important for the paper's legal coherence. The argument that algorithm-mandate is "procedural regulation" under Smiley's "broadest sense" reading is the correct legal framing and will reduce the paper's vulnerability to federalism challenges.

The B.11 footnote correction is the right approach to a forward-citation problem. "Companion paper B.11 demonstrates near-zero variance in NC and WI" is a statement that is true (based on preliminary runs) and appropriately hedged; the full 50-state claim is correctly deferred to B.11.

## Remaining Reproducibility Gap

The most significant reproducibility gap — no public repository URL or TIGER/Line vintage specification — carries forward. A practitioner who reads the DIA statutory text and wants to verify the map for their state cannot do so without a public URL for the redist binary and a specific TIGER/Line vintage year. For a statute whose validity rests on independent verifiability ("any court, expert, or citizen can independently reproduce the map"), this is a structural requirement. It must be resolved before the paper is submitted to a law review.

## Score: 3 / 4 — Minor Revision

The B.11 citation fix is the most important change in this revision. The runtime and TIGER/Line vintage carry forward as P1 items. The paper's computational design remains the strongest in the B-series for reproducibility, but the gap between the design and the implementation (B.11 not yet complete) limits what can currently be claimed.
