# Review 3 — Moon Duchin
**Paper**: F.5: Compactness at State Scale — Algorithmic State Legislative Districts Outperform Congressional
**Round**: R2
**Score**: 3/4

## Response to Revision

**C1 (Abstract-body resolution effect inconsistency)** — Addressed. The abstract now reads "approximately 0.013 PP units (averaged across 2000/2010/2020 census years; 0.020 PP units for 2020 alone)." This correctly identifies both figures and their distinct computational bases. The body's decomposition table now consistently reports 0.013. The source of the discrepancy (different averaging periods across F.1 and F.5) is explained in a footnote. This was the primary internal error and it is now resolved.

**C2 (Regularity conditions not specified)** — Partially addressed. The paper now includes a note that states with single-congressional-district configurations (AK, WY, MT) are degenerate cases where the Proposition's prediction is reversed. However, the formal regularity conditions (rectifiable Jordan curve, bounded curvature, etc.) are still not stated. The Proposition still relies on informal language ("excluding states with highly fractal boundaries"). I would recommend either: (a) providing the formal conditions as a Definition, or (b) explicitly reframing the Proposition as a Conjecture supported by empirical evidence across 47 of 50 states. The current state — a Proposition with informal conditions — is mathematically unsatisfying.

**C3 (PP∞ limiting value)** — Addressed. A note has been added explaining that PP∞ represents the limiting compactness as districts become very small, with an empirical estimate from the data.

**C4 (Block-group resolution as measurement artifact)** — Addressed. Section 4.1 now explicitly notes that part of the PP improvement at block-group resolution may be a measurement effect (finer boundary tracing of the same districts) rather than a genuine geometric improvement, and acknowledges that disentangling these effects is not possible with the current pipeline.

## Assessment

The primary internal inconsistency is resolved and several secondary issues are addressed. The remaining concern (formal Proposition conditions, C2) is a mathematical rigor issue that I continue to flag. I maintain 3/4; the paper would reach 4/4 with either formal regularity conditions or an honest reframing as an empirically-supported conjecture.

**Score**: 3/4
**Recommendation**: Accept with minor revisions
