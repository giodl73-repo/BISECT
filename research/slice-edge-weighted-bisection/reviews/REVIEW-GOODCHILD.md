# Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Michael Goodchild (UC Santa Barbara)
**Expertise**: GIS theory, spatial analysis, geography
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

This paper applies graph partitioning to congressional redistricting with appropriate attention to geographic realities: boundary length computation, water crossings, islands, and projection choices. The core contribution—weighting graph edges by actual boundary lengths—is a sensible way to incorporate geometric information into topological optimization. The empirical results (56% compactness improvement) validate the approach.

From a GIS perspective, the paper handles spatial data competently: Queen contiguity for adjacency, R-tree spatial indexing, Shapely geometric operations, and state-specific projections (California Albers, Texas Conic) show geographic awareness. The bridge connection strategy for water crossings (county-based median weights) is pragmatic if not theoretically principled.

However, the paper has notable geographic limitations. Most critically, it treats census tract boundaries as fixed geometric primitives when they are social constructs with embedded biases (e.g., highways, railroad tracks, historical redlining). Optimizing over fixed tract boundaries may perpetuate these biases rather than creating truly optimal districts. Additionally, the paper doesn't engage with fundamental geographic questions: What is a "compact" district from a functional geography perspective? Does geometric compactness (Polsby-Popper) correspond to meaningful accessibility or communities of interest?

The work is technically sound within its chosen framework and makes a valuable contribution to automated redistricting. For a computational venue, it's acceptable; for a geography or GIS journal, it needs deeper engagement with spatial theory and alternative geographic representations.

## Score

**Score**: 3/4 — **Accept** (for computational venues); **2/4 — Weak Accept** (for GIS venues)

## Major Issues (Blocking)

### M1: Census Tracts as Fixed Boundaries—Embedded Biases Not Addressed

The paper treats census tract boundaries as fixed inputs for optimization. However, tract boundaries are social constructs that often follow problematic divisions:
- Highways and railroads (historical barriers)
- Redlining boundaries (discriminatory housing policies)
- Industrial zones vs residential areas
- Arbitrary administrative decisions

Optimizing over fixed tracts may *perpetuate* these biases. Key questions:
- How often do census tract boundaries align with natural communities?
- Do tract boundaries systematically separate racial/ethnic groups?
- Would block-level optimization (finer granularity) reduce boundary artifacts?

At minimum:
- Discuss tract boundary limitations
- Compare tract-level to block-level results for 2-3 pilot states
- Acknowledge that optimal compactness over tracts ≠ optimal geographic districting

### M2: Geometric vs Functional Compactness Conflated

The paper optimizes Polsby-Popper (geometric compactness: shape regularity) but doesn't consider *functional compactness*—ease of representation and constituent access. Key geographic principles:
- **Accessibility**: Can constituents reach representative's office?
- **Travel time**: Are districts contiguous in travel-time space, not just Euclidean space?
- **Functional regions**: Do districts respect commuting patterns, media markets, watersheds?

A geometrically compact district (circular) may be functionally poor if it crosses mountain ranges or bisects a metro area. Include:
- Discussion of geometric vs functional compactness
- Acknowledgment that Polsby-Popper doesn't measure functional properties
- Pilot study using travel-time distances instead of Euclidean distances

### M3: Projection Choices Not Justified

The paper uses state-specific projections (California Albers EPSG:3310, Texas Conic EPSG:3083) for boundary length computation. This is better than lat/lon but still introduces distortion:
- Equal-area projections (Albers) preserve area but distort shape/distance
- Do projection artifacts affect compactness measurements?
- Would different projections (conformal, equidistant) change results?

Include sensitivity analysis showing results are robust to projection choice, or use geodesic distances (ellipsoidal calculations) to eliminate projection dependence.

## Minor Issues

### m1: Queen Contiguity Not Justified

The paper uses Queen contiguity (shared edge or vertex) but doesn't justify this vs alternatives:
- **Rook contiguity**: Shared edge only (more restrictive)
- **Distance-based**: Adjacency within threshold distance
- **Functional adjacency**: Connected by major roads

Why Queen? Does Rook contiguity produce different compactness? Show results are robust to adjacency definition.

### m2: R-tree Spatial Index Not Optimized

Section 3.2.1 uses R-tree for candidate neighbor queries. R-tree performance depends on tree construction parameters (max entries per node, split strategy). Are these tuned? Would quadtree or other spatial indices be more efficient?

### m3: Point Adjacency (0.1m) Geographically Questionable

Tracts sharing only a vertex receive 0.1m edge weight. Geographically, single-point contact is often ambiguous:
- Surveying imprecision (±10m common)
- Datum shifts between census years
- Rounding in coordinate storage

Is 0.1m meaningful given measurement uncertainty? Should point adjacencies be excluded entirely (Rook contiguity)?

### m4: Bridge Edges Heuristic—Alternatives Not Explored

County-based median boundary length for water crossings is heuristic. Geographic alternatives:
- **Shortest geodesic distance** between disconnected components
- **Ferry routes or bridges** (actual transportation infrastructure)
- **Functional connectivity** (commuting data, phone call patterns)

Why median boundary length? This is a strange choice—it relates landform boundaries to water crossings without clear geographic logic.

### m5: Island States (Hawaii, Alaska) Deserve Deeper Analysis

Hawaii (2 districts, 8 islands) and Alaska (1 district, hundreds of islands) have unique geography. The paper mentions them briefly (+177% improvement for Hawaii) but doesn't analyze:
- Which islands are grouped together?
- Do districts respect island boundaries or span islands?
- Are inter-island bridge weights reasonable?

Include detailed case studies with maps showing island groupings.

### m6: Geographic Scale Effects Not Analyzed

Do compactness improvements vary by geographic scale?
- Urban vs rural states (New Jersey vs Montana)
- Small vs large states (Delaware vs Alaska)
- High vs low population density

Analyze whether edge weighting benefits are scale-dependent or uniform.

### m7: Boundary Following Not Evaluated

The paper claims districts "follow natural geographic features (rivers, roads)" (Section 5.1, 5.4) but never validates this. Do district boundaries actually align with rivers, roads, or other geographic features more than enacted plans?

Quantify boundary alignment:
- Overlay district boundaries with river/road networks
- Measure percentage of boundary length that coincides with geographic features
- Compare to enacted districts

## Strengths

1. **Geographic awareness**: State-specific projections, Queen contiguity, water crossing handling shows practical GIS competence.

2. **Spatial data pipeline**: R-tree indexing, Shapely geometric operations, intersection classification is technically sound.

3. **National scope**: 50-state evaluation demonstrates method handles diverse geographies (islands, deserts, mountains, urban areas).

4. **Computational efficiency**: 10-30 seconds preprocessing per state is reasonable for boundary computation.

5. **Reproducibility**: Detailed geometric operations (EPSG codes, Shapely methods) enable replication.

6. **Handles complexity**: Bridge connections, point adjacencies, adaptive weight scaling addresses real-world geographic challenges.

## Questions for Authors

1. **Tract boundary biases**: How often do census tract boundaries perpetuate problematic divisions (highways, redlining)? Would block-level optimization reduce these artifacts?

2. **Functional compactness**: Can you measure travel-time compactness instead of Euclidean? Do geometrically compact districts cross natural barriers (mountains, rivers)?

3. **Projection sensitivity**: Would different projections (equidistant, conformal) or geodesic distances change compactness results materially?

4. **Boundary feature alignment**: Do district boundaries actually follow rivers/roads more than enacted plans? Can this be quantified?

5. **Queen vs Rook**: Does contiguity definition affect results? Would Rook produce different compactness?

6. **Bridge weights**: Why median boundary length for water crossings? Would shortest geodesic distance or actual ferry routes be more geographically principled?

7. **Scale effects**: Do compactness improvements depend on state size, density, or urban/rural character?

8. **Island groupings**: For Hawaii and Alaska, which islands end up in the same districts? Does this make geographic sense?

## Recommendations

- **Discuss tract boundary limitations**: Acknowledge that tracts are social constructs; optimizing over them may perpetuate biases. Compare tract-level to block-level for pilot states.

- **Add functional compactness analysis**: Compute travel-time distances for 2-3 states; show geometric compactness doesn't guarantee functional compactness.

- **Projection sensitivity test**: Rerun 3-5 states with different projections or geodesic distances; show robustness.

- **Quantify boundary feature alignment**: Measure overlap with rivers/roads/geographic features; validate "natural feature following" claim.

- **Justify Queen contiguity**: Compare to Rook for representative states; show results are robust.

- **Alternative bridge weights**: Test shortest geodesic distance vs median boundary length; justify current choice.

- **Scale analysis**: Stratify states by size/density/urban-rural; show whether improvements are scale-dependent.

- **Island case studies**: Provide detailed maps for Hawaii and Alaska showing island groupings; discuss geographic sensibility.

- **Geographic theory**: Engage with functional geography literature; discuss what "compact" means from a geographic (not just geometric) perspective.

---

**Verdict**: Accept with Minor Revisions (for computational venues); Major Revisions (for GIS venues)

**Confidence**: High — I have extensive experience with GIS theory, spatial analysis, and geographic representation. This paper is technically competent on spatial data handling but lacks depth on geographic theory. For computational venues (KDD, AAAI), the current technical treatment suffices with minor additions (projection sensitivity, boundary feature validation). For GIS venues (IJGIS, GeoInformatica, AGILE), substantial additions are needed: tract boundary bias discussion, functional compactness analysis, geographic scale effects, and engagement with spatial representation theory. The authors should tailor revisions to their target venue's expectations.
