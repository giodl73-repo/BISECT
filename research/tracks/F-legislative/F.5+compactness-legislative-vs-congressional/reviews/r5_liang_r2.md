# Review 5 — Christina Liang
**Paper**: F.5: Compactness at State Scale — Algorithmic State Legislative Districts Outperform Congressional
**Round**: R2
**Score**: 3/4

## Response to Revision

**C1 and C2 (PP mean inconsistency across papers)** — Addressed. The paper now explicitly explains that the 50-state tract PP mean in F.5 (0.388) differs from F.1 (0.381) because F.5 averages over three census years (2000/2010/2020) while F.1 reports 2020 only. The abstract now correctly reports both figures with their computational contexts. The cross-paper inconsistency is explained rather than hidden.

**C3 (Enacted map comparison data requirements)** — Not addressed. The source of the 35-state enacted map GIS data is still not specified. A replicator cannot reproduce Section 5 without knowing which 35 states are included and where the enacted shapefiles were obtained. This is the primary remaining reproducibility gap.

**C4 (Cross-census PP computation)** — Addressed. The paper now explicitly states that house seat counts (k) are held fixed at their 2020 values for all three census years, and that this is a valid design choice for cross-census comparison (holding k fixed isolates the effect of population distribution changes). The caveat that actual chamber sizes changed for some states between 2000 and 2020 is now disclosed.

## Assessment

The abstract-body consistency fix and the cross-census k-fixation disclosure are the right corrections. The enacted map data source gap remains. I am maintaining 3/4: the core paper is reproducible for the algorithmic results, but Section 5 (enacted map comparison) remains non-reproducible without the data source specification.

**Score**: 3/4
**Recommendation**: Accept with minor revisions
