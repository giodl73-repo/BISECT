---
reviewer: Moon Duchin
round: 3
score: 3
date: 2026-05-05
---

## Summary

Round 3 makes two targeted changes: the Rodden-null denominator explanation and the Data Availability statement. Neither change touches my open P1 items (strong-inference test methodology, Proposition 1 proof limitations). Score remains at 3. The paper continues to be a useful B-series synthesis but has methodological gaps that should be resolved before external publication.

## R3 Changes: Assessment

**Rodden-null denominator.**
The explanation is correctly structured and well-motivated. The $k=1$ exclusion is uncontroversial; the 9-invariant-state exclusion is reasonable, though I note that "identical partisan outcomes under all tested modes" is an empirical claim that should cite the specific B-series runs that establish invariance. The current text does not provide a citation or data source for the invariant-state classification. Since the 9 states are not enumerated, a reader cannot independently verify this claim. I flag this as P2: enumerate the 9 invariant states in a footnote.

From a mathematical standpoint, I also note that "identical partisan outcomes under all tested modes" is a strong empirical claim — it means the seat count is insensitive to the entire tested mode space in these states. This is a meaningful result and should be presented as a finding, not as a denominator-construction detail. A brief sentence noting that "these 9 states are empirically mode-invariant — a result that itself supports the geographic-sorting thesis" would strengthen the paper.

**Data Availability statement.**
The repository URL (`https://github.com/apportionment-research/redist`) resolves Liang's core concern. The statement that adjacency files are available as "GitHub Release assets, with SHA-256 hashes recorded in the plan manifests" is exactly the right structure: it connects the public data release to the paper's internal hash-chain verification mechanism. This is a substantial improvement.

## Remaining P1 Items (unchanged from R2)

**P1.2 — Strong-inference test procedure.**
The procedure still compares GeoSection (no minority signal) against VRASection (w_vra = 0.40) to establish that "political goals could have been achieved with better minority outcomes." My objection stands: this conflates algorithm selection with parameter variation. The correct operationalisation holds the algorithm (VRASection) fixed and varies w_vra from 0 to 0.40, demonstrating that the w_vra = 0 baseline produces fewer MM districts than w_vra = 0.40. The current procedure cannot isolate whether the MM-count improvement comes from the ratio-selection mechanism (VRASection) or from the alignment signal (w_vra > 0). For a court applying the Callais strong-inference framework, this distinction matters.

**P1.3 — Proposition 1 proof (integrity vs. provenance).**
The proof still states "the SHA-256 hash of the adjacency graph verifies the data provenance." This conflates integrity verification (the file has not changed since it was hashed) with provenance (the file was derived from the TIGER/Line source by a documented process). I continue to require: "The SHA-256 hash verifies file integrity; provenance from the TIGER/Line source to the adjacency graph file requires separate chain-of-custody documentation." This is a one-sentence correction that should be made before external publication.

## Score: 3 / 4 — Minor Revision

The Rodden-null fix and the Data Availability statement are genuine improvements. The two carry-forward P1 items require attention for external publication. The paper is publication-ready for the B-series internal track.
