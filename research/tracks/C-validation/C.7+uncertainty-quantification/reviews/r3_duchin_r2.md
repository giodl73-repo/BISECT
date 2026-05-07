---
reviewer: Moon Duchin
round: 2
score: 4
date: 2026-05-05
---

## Summary

The authors have addressed the most important of my three Round 1 P1 items: the Section 4.4 algebraic error is corrected, the abstract CI inconsistency is fixed, and the bootstrap exchangeability argument is substantively improved. The Section 4.4 correction is notable because the authors not only fixed the factual error ("is below the tract-level point estimate") but added the legally meaningful comparative statement ("but above the enacted plan mean PP (0.295), confirming that the algorithmic advantage is robust to resolution measurement error"). This is a better correction than a minimal fix would have been — it converts the error correction into a substantive result statement. My P1.1 (PP measurement error source conflation, specifically the unmeasured shapefile simplification error) is addressed indirectly through the bootstrap exchangeability argument's mention of two distinct error sources, but not through the separate-subsection treatment I requested.

## Evaluation of Revisions

**P1.3 (Section 4.4 arithmetic error)**: Fully resolved and improved. The corrected statement "The lower bound (0.438) is below the tract-level point estimate (0.441) but above the enacted plan mean PP (0.295), confirming that the algorithmic advantage is robust to resolution measurement error" is factually correct and adds interpretive value. This is the right approach to a correction: not just fixing the error but explaining what the correct result actually means.

**P1.1 (PP measurement error source conflation — shapefile simplification vs. building-block resolution)**: Partially addressed. The bootstrap validity paragraph in Section 2.3 correctly notes that the 50-state exchangeability assumption applies "if states are drawn from a common distribution of geographic complexity." This is adjacent to the measurement error source issue but does not resolve it. Section 4.1's description of two sources (shapefile resolution and building-block resolution) still treats the C.1 empirical ΔPP = 0.003 as capturing both sources. The request was for either (a) an empirical estimate of shapefile simplification error separately, or (b) an explicit acknowledgment that shapefile simplification is unmeasured and bounded by the fact that both algorithmic and enacted plans use the same TIGER shapefiles. Option (b) would be a two-sentence addition that fully resolves the concern. As it stands, the measurement error section remains conflated.

**P1.2 (VRA VAP fraction covariance)**: Not addressed. The delta method approximation for minority VAP in Section 6 still omits the covariance between minority coverage error and total coverage error. This remains a mathematical incompleteness.

**Abstract CI fix**: Fully resolved. The [+15%, +29%] three-source CI is the correct headline.

**Legal-relevance mapping (Section 7)**: A valuable addition. The three-item mapping linking each uncertainty source to a legal challenge category is clear and accurate. The Wesberry compliance framing for census uncertainty is exactly right.

## Remaining Concerns

The shapefile simplification error conflation (P1.1 option b) would require only two sentences in Section 4 to resolve — noting that simplification error affects both algorithmic and enacted plans identically (same TIGER shapefiles), so it cancels in the comparison. This is factually accurate and would fully resolve the measurement source conflation. The VRA covariance issue is a more substantive gap but is lower-priority for the paper's core claims.

## Score: 4 — Accept

The arithmetic error correction in Section 4.4 — both the factual fix and the improved interpretive statement — is the most important revision, and it is well executed. The abstract/body CI consistency fix resolves the most glaring inconsistency in the Round 1 paper. The remaining gaps (shapefile conflation, VRA covariance) are documented limitations that do not prevent acceptance.
