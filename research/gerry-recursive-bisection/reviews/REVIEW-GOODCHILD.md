# Review: Recursive Bisection for Congressional Redistricting

**Reviewer**: Michael Goodchild (UC Santa Barbara)
**Expertise**: GIS theory, spatial analysis, geographic information science
**Date**: 2026-02-07
**Round**: 1

---

## Overall Assessment

This paper applies graph partitioning to congressional redistricting with reasonable attention to geographic realities—a welcome contrast to purely mathematical approaches that ignore spatial structure. The water-based adjacency handling (Section 3.2) shows sophisticated understanding of geographic connectivity challenges in island and peninsula states.

The philosophical framing around Huntington-Hill and "impossibility defense" is interesting, though less relevant to GIS research. From a spatial analysis perspective, I'm interested in how well the algorithm respects actual geographic features, preserves natural communities, and handles edge cases in Census geographic data.

However, the paper has significant gaps in geographic analysis: (1) no examination of how algorithm interacts with natural and built geography, (2) missing discussion of coordinate reference systems and projection impacts, (3) inadequate treatment of Census geographic anomalies, and (4) limited engagement with GIS literature on spatial optimization.

## Score: 3.0/4.0

**Accept with Revisions (Geographic Analysis)**

The technical work is competent and the results demonstrate feasibility. With additional spatial analysis depth—particularly on geographic feature preservation, projection handling, and Census data quality—this would be suitable for publication in venues bridging GIS and political science (IJGIS applications, PLOS One, Political Geography).

## Major Issues (Must Address)

### M1. Geographic Feature Preservation Unanalyzed

**Issue**: You optimize edge cuts without examining how district boundaries relate to actual geographic features—rivers, mountain ranges, highways, state parks, urban areas.

**Why this matters**: Traditional redistricting respects natural and built features that define communities. Rivers often form natural boundaries; mountain ranges separate regions; highways divide neighborhoods. Your algorithm sees only abstract graph edges, not what those edges represent geographically.

**Specific questions**:

1. **Water features**: How often do district boundaries cross rivers vs. follow them? Natural boundaries often align with river courses—does your algorithm discover this?

2. **Mountain ranges**: Do districts respect topographic barriers (Cascades, Rockies, Appalachians) or bisect them arbitrarily?

3. **Urban areas**: Do districts keep cities intact or split them? (This has both political and community-of-interest implications)

4. **Transportation networks**: Do boundaries follow highways/major roads (which often form psychological boundaries) or cut across them?

5. **Public lands**: Do districts split state/national parks, forests, or BLM lands? (These areas have no voters but affect district shape)

**Current treatment**: No analysis of geographic feature preservation.

**Recommendation**: Add Section 4.4 "Geographic Feature Preservation Analysis":

1. **Water boundary alignment**: For states with major rivers (Mississippi, Ohio, Missouri, Columbia), calculate percentage of district boundaries that follow river courses. Compare to enacted districts.

2. **Topographic analysis**: Overlay district boundaries on elevation data, measure how often boundaries cross mountain ranges vs. follow valleys. (Use USGS digital elevation models)

3. **Urban coherence**: For 20-30 major cities, determine whether algorithmic districts keep cities intact or split them. Compare split rates to enacted plans.

4. **Transportation alignment**: Overlay boundaries on highway networks, measure alignment (do boundaries follow highways?).

5. **Case studies**: Show 2-3 states where geographic features clearly influence district shapes (CO: mountain ranges, MI: Great Lakes, FL: Everglades).

This geographic analysis would demonstrate your algorithm respects natural geographic structure, not just abstract graph properties.

### M2. Coordinate Reference Systems and Projection Issues

**Issue**: Section 3.2 mentions "state-specific coordinate reference systems (e.g., California Albers EPSG:3310)" but doesn't discuss projection impacts on distance calculations, area measurements, or compactness metrics.

**Geographic realities**:

1. **Projection distortion**: All map projections distort distance, area, or shape. Albers Equal-Area preserves area but distorts shape; Mercator preserves shape but distorts area. Your compactness metrics (Polsby-Popper, Reock) depend on accurate area and perimeter—which projection matters.

2. **State-specific vs. national projections**: You mention state-specific projections for county bridging. But do you use different projections for different states? Or a single national projection? Each choice affects comparability.

3. **Multi-state comparison**: If California uses Albers (EPSG:3310) and Texas uses Lambert Conformal Conic (EPSG:3082), compactness scores aren't directly comparable. Area and perimeter calculations differ.

4. **Alaska special cases**: Alaska spans 60° longitude. Any projection creates massive distortion. How do you handle this?

**Current treatment**: One sentence mentioning EPSG:3310 without discussing implications.

**Recommendation**: Add subsection 3.2.1 "Coordinate Reference Systems and Projection Considerations":

1. **Projection catalog**: List which projection you use for each state and why (presumably state-specific NAD83 projections)

2. **Distortion analysis**: For states spanning large areas (AK, TX, CA), quantify projection distortion (area distortion, distance distortion). Does this affect compactness calculations significantly?

3. **Sensitivity analysis**: For 2-3 states, compute compactness metrics using different projections (Albers, Lambert, Web Mercator). Show variation is <5% or acknowledge uncertainty.

4. **Alaska handling**: Explain specific approach for Alaska given extreme projection challenges. (Maybe use Alaska Albers EPSG:3338?)

This attention to projection details demonstrates GIS sophistication and ensures metrics are spatially meaningful.

### M3. Census Geographic Data Quality Issues

**Issue**: You treat Census TIGER/Line shapefiles as ground truth but don't discuss well-known data quality issues that affect redistricting.

**TIGER/Line problems**:

1. **Sliver polygons**: Topology errors create tiny gaps or overlaps between tract boundaries (10^-6 area). Do these create spurious adjacencies or non-adjacencies?

2. **Precision inconsistencies**: TIGER coordinates have varying precision. Coastlines may not align perfectly with tract boundaries. How do you handle this?

3. **Water boundaries**: TIGER treats water bodies inconsistently—sometimes as polygons, sometimes as absences. Does this affect island detection or adjacency computation?

4. **Year-to-year changes**: Census tract boundaries change between censuses (splits, mergers). How would your algorithm handle this for longitudinal analysis?

5. **Point adjacencies**: Some tracts touch only at single points (corner adjacency). These are mathematically valid but geographically tenuous—are these real adjacencies?

**Current treatment**: Assumes TIGER data is perfect.

**Recommendation**: Add subsection 3.2.2 "Census Geographic Data Processing":

1. **Topology cleaning**: Describe preprocessing steps (snapping vertices, removing slivers, validating polygons). Do you use GeoPandas? PostGIS? GEOS?

2. **Adjacency validation**: Report how many tract pairs have shared boundaries <10 meters (likely topology errors). Do you filter these?

3. **Corner adjacency policy**: State explicitly whether you include corner-only adjacencies (queen contiguity) or just edge-sharing (rook contiguity). Justify choice.

4. **Invalid geometry handling**: Section 4.2 mentions 2 districts excluded "due to invalid geometries." What caused this? How did you detect it? How common are TIGER geometry errors?

5. **Quality metrics**: Report TIGER data quality statistics (number of slivers removed, boundaries snapped, geometries validated).

This demonstrates you're aware of real-world GIS data challenges and handle them appropriately.

## Minor Issues (Should Address)

### m1. Spatial Autocorrelation of Compactness

**Compactness varies with geography** (urban areas more compact, rural areas less compact). But is this variation:
- Random? (Geography-driven)
- Clustered? (Some regions systematically more compact than others)

**Spatial analysis**: Moran's I test for spatial autocorrelation of compactness scores. Are low-compactness districts clustered geographically? (Would suggest geography, not algorithm, drives variation)

**Recommendation**: Brief analysis showing compactness patterns are spatially autocorrelated (expected given geographic constraints).

### m2. Scale Dependency of Compactness Metrics

**Polsby-Popper and Reock are scale-dependent**:
- Small districts in dense urban areas: Easier to be compact
- Large districts in rural areas: Harder to achieve compactness (long roads, dispersed population)

**Question**: Do you account for scale differences when comparing compactness across districts?

**Recommendation**: Normalize compactness by district area or population density, showing urban districts aren't unfairly advantaged in comparisons.

### m3. Island Bridging Distance Distribution

**Section 3.2**: County-based bridges connect islands. But:
- How far are these bridges? (100m? 10km? 100km?)
- Distribution of bridge lengths?
- Which states require longest bridges?

**Concern**: If bridges span 100+ km (Alaska Aleutians?), are resulting "contiguous" districts geographically meaningful?

**Recommendation**: Table showing bridge statistics per state (number of bridges, mean/median/max distance). Discuss whether long bridges create legally-contiguous but geographically-absurd districts.

### m4. Engagement with Spatial Optimization Literature

**You cite METIS (CS)** but not spatial optimization literature from geography/GIS:

- **Openshaw & Rao** (1995): Automated zone design
- **Macmillan & Pierce** (2006): Geodemographic classification
- **Jones et al.** (2011): Geographic regionalization algorithms

**These algorithms** explicitly handle spatial contiguity, compactness, and attribute similarity—directly relevant to redistricting.

**Recommendation**: Add 2-3 paragraphs in Section 6.2 comparing your approach to geographic regionalization methods. Are they similar? Different? Complementary?

## Strengths

1. **Water-based adjacency**: Sophisticated handling of island connectivity challenges—shows geographic awareness

2. **State-specific projections**: Recognition that projection matters (though underexplored)

3. **Tract-level resolution**: Reasonable compromise between blocks (too fine) and counties (too coarse)

4. **Computational efficiency**: 2.5 hours enables iteration and scenario analysis

5. **Practical applicability**: Could actually be implemented with real Census data

## Recommendation

**Accept with Revisions (Geographic Analysis)**

The paper demonstrates a computationally sound approach to redistricting with awareness of basic geographic challenges (islands, projections, Census data). Results are credible and methodology is reproducible.

To strengthen for GIS venues (IJGIS, Computers Environment & Urban Systems, Applied Geography):

1. **M1**: Analyze how districts relate to actual geographic features (rivers, mountains, cities, highways)
2. **M2**: Comprehensive discussion of projection impacts on compactness metrics
3. **M3**: Address Census TIGER data quality issues and preprocessing steps

These additions would demonstrate spatial analysis sophistication beyond basic graph partitioning, showing you understand geographic realities that constrain redistricting in practice.

For political science venues (APSR, JOP), current geographic treatment may suffice. But for any venue where geographers review papers, deeper spatial analysis is necessary.

---

**Final note**: The intersection of computational methods, political science, and geographic information science is rich research territory. Your paper bridges these fields reasonably well, but deeper engagement with spatial analysis would strengthen it. The question "how well do algorithmic districts respect natural and built geography?" is central to evaluating whether they're superior to human-drawn maps. Answer it empirically, and you'll have a compelling contribution.
