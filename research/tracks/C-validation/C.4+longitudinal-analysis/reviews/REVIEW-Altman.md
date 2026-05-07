> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Twenty Years of Congressional Redistricting

**Reviewer**: Micah Altman (MIT Libraries)
**Expertise**: Redistricting algorithms, computational methods, reproducibility
**Date**: 2026-02-08
**Round**: 1

---

## Overall Assessment

This paper demonstrates good longitudinal research design and makes meaningful contributions to understanding algorithmic redistricting over time. However, it falls short of computational science standards for reproducibility and transparency. Without access to code, data, and detailed computational specifications, the work cannot be independently verified or extended by other researchers.

**Verdict**: Accept with major revisions

---

## Score

**3.0 / 4.0** — Good paper but needs reproducibility improvements

---

## Major Issues

### M1. Reproducibility Crisis

**Issue**: Paper lacks essential computational details and does not provide code/data access:
- No link to GitHub repository or data archive
- METIS version specified (5.1.0) but not METIS parameters beyond α=5.0
- Census tract shapefiles mentioned but not sourced
- IoU computation method described conceptually but not algorithmically

**Evidence**: You claim "directly comparable district maps" (Section 1.2) but provide no mechanism for readers to access or reproduce these maps.

**Impact**: This violates computational reproducibility standards. Readers cannot:
- Verify your compactness scores
- Test sensitivity to parameter choices
- Extend analysis to 2030 census
- Compare your METIS implementation to alternatives

**Recommendation**:
1. Create public GitHub repository with:
   - METIS configuration files
   - Census tract preprocessing scripts
   - IoU computation code
   - District assignment CSVs (all 50 states, 3 years)
2. Archive data in Dataverse or Zenodo (permanent DOI)
3. Add "Data and Code Availability" section before bibliography
4. Follow FAIR principles (Findable, Accessible, Interoperable, Reusable)

---

### M2. Computational Workflow Underspecified

**Issue**: The paper describes high-level methodology but omits critical implementation details:

**Missing details**:
- Census tract-to-graph conversion (how are adjacencies computed?)
- METIS invocation (command-line flags? API calls?)
- Population data processing (how are 2000 tracts mapped to 2010/2020 boundaries?)
- IoU computation (which GIS library? Shapely? GeoPandas? PostGIS?)
- Runtime and hardware (how long did 50 states × 3 years take? What machine?)

**Impact**: Without these details, replication attempts will fail or produce different results due to implementation choices.

**Recommendation**: Add Methods Supplement with:
1. Complete computational pipeline diagram
2. Software versions (Python, GDAL, METIS bindings)
3. Hardware specs and runtime estimates
4. Pseudocode for graph construction and IoU calculation

---

### M3. Data Provenance Unclear

**Issue**: Census data sourcing is vague:
- "Census PL 94-171 redistricting files" (Section 3.1) — which release? Which URLs?
- "TIGER/Line shapefiles" — which year's TIGER files for which census?
- "Dave's Redistricting App shapefiles" (Section 3.3) — when downloaded? Which version?

**Evidence**: Census Bureau releases multiple versions of shapefiles (cartographic boundaries vs TIGER/Line). Each has different precision and topology. Your choice matters for compactness scores.

**Impact**: Different shapefile versions produce different compactness scores. Without exact data provenance, results are not reproducible.

**Recommendation**: Create Data Supplement with:
1. Table of all data sources with URLs and download dates
2. MD5 hashes of input files for verification
3. Scripts to download and preprocess raw data
4. Document any manual corrections or topology fixes

---

## Minor Issues

### m1. METIS as Black Box

METIS is proprietary (University of Minnesota license). For reproducibility, consider:
- Open-source alternatives (KaHIP, Scotch)
- Or document exact METIS version, compile flags, and provide Docker container

**Recommendation**: Test whether results are METIS-specific or graph-partitioning-general by comparing to KaHIP.

---

### m2. Statistical Software Not Specified

Compactness metrics (PP, IoU), statistical tests (t-test, ANOVA), correlations—computed how? R? Python? Stata?

**Recommendation**: Add "Statistical Analysis" subsection specifying software and packages (e.g., Python 3.11, scipy.stats, geopandas 0.14).

---

### m3. Figures Not Reproducible

Figures are described (Figure 1: choropleth map, Figure 2: box plots) but:
- No figure generation scripts
- No raw data files underlying figures
- No style files (matplotlib rc, ggplot themes)

**Recommendation**: Include figure generation scripts in repository. Readers should be able to run `make figures` and reproduce all visualizations.

---

## Strengths

1. **Clear methodology**: Algorithm description is mostly clear (METIS, PP, IoU)
2. **Comprehensive scope**: 50 states, 3 census years is impressive
3. **Longitudinal design**: Consistent methods across time is methodologically sound
4. **Open about limitations**: Discussion acknowledges single-algorithm limitation

---

## Recommendations

**Priority 1** (M1): Create public GitHub repository with code and data
**Priority 2** (M2): Add computational workflow documentation
**Priority 3** (M3): Document data provenance with URLs and hashes
**Priority 4** (m1-m3): Specify software versions and figure scripts

---

## Questions for Authors

1. Will you release code and data upon publication?
2. Have you tested reproducibility by having a colleague rerun your analysis?
3. Can you provide a Docker container for complete computational environment?

---

**Bottom Line**: Good research that advances redistricting methods, but falls short of modern computational reproducibility standards. Adding code/data access would substantially increase impact and enable follow-on research.
