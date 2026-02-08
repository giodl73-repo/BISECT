# Gerrymandering Research Portfolio

## Overview
This directory contains research papers on algorithmic redistricting, focusing on VRA compliance, graph partitioning methods, and practical tradeoffs in congressional redistricting.

## Papers

### Paper 1: Edge-Weighted VRA Compliance ✅ COMPLETE
**Directory:** `gerry-vra-compliance/`
**Status:** Complete (7 sections, 3 visualizations, analysis notebook)
**Key Finding:** Edge-weighted single-objective optimization achieves 80% success rate vs 40% for multi-constraint

**Files:**
- `main.tex` - Complete paper
- `sections/*.tex` - 6 sections (intro, background, methodology, results, discussion, conclusion)
- `scripts/*.py` - 3 visualizations (approach comparison, ablation heatmap, tradeoff scatter)
- `analysis/edge_weighting_analysis.ipynb` - Statistical analysis
- `results/edge_weighting_ablation_study.csv` - 140 experimental configurations

**Data Status:** ✅ All 5 states tested with edge-weighting (Alabama, Georgia, Mississippi, Louisiana, South Carolina)

---

### Paper 2: N-Way vs Recursive Bisection ✅ COMPLETE
**Directory:** `gerry-nway-vs-recursive/`
**Status:** Complete (7 sections, references)
**Key Finding:** N-way achieves +4.3 points better minority concentration but recursive bisection offers transparency/auditability advantages

**Files:**
- `main.tex` - Complete paper structure
- `sections/*.tex` - 7 sections including theory, methodology, results, discussion, conclusion
- `references.bib` - Citations (METIS, graph partitioning literature)

**Data Status:** ⚠️ **PARTIAL** - Only Alabama has complete comparison data
- Alabama (k=7): Recursive 43.0%, N-way 47.3%, Adaptive 46.1%
- Georgia, Mississippi, Louisiana, South Carolina: Need n-way vs recursive comparison

**Data Gap to Fill:** Run comprehensive n-way vs recursive comparison for 4 states (see Paper 2 PLAN.md)

---

### Paper 3: Adaptive Recursive Bisection 📋 PLANNED
**Directory:** `gerry-adaptive-bisection/`
**Status:** Planning complete, experiments pending
**Research Question:** Can data-driven tree selection improve recursive bisection while preserving transparency?

**Key Contributions:**
1. Adaptive algorithm that chooses tree structure based on data
2. Quantify improvement over predetermined trees (expected +3 points)
3. Show why adaptive still inferior to n-way (constraint by binary structure)

**Data Requirements:**
- Predetermined tree results for all 5 states (6 trees for k=7)
- Adaptive tree results for all 5 states
- Comparison to n-way baseline (from Paper 2)

**See:** `gerry-adaptive-bisection/PLAN.md` for detailed experiments and timeline

---

### Paper 4: Multi-Constraint vs Edge-Weighted 📋 PLANNED
**Directory:** `gerry-multi-vs-edge/`
**Status:** Planning complete, experiments pending
**Research Question:** Why does edge-weighted single-objective dramatically outperform multi-constraint?

**Key Contributions:**
1. Theoretical explanation: constraint conflict in multi-constraint
2. Empirical validation: 40% → 80% success rate improvement
3. Generalization: When to use each approach beyond redistricting

**Data Requirements:**
- Multi-constraint results for all 5 states (currently only Alabama tested)
- Constraint conflict test: vary ubvec to test theory
- Head-to-head comparison with edge-weighted (from Paper 1)

**See:** `gerry-multi-vs-edge/PLAN.md` for detailed experiments and timeline

---

### Paper 5: The 42% Threshold 📋 PLANNED
**Directory:** `gerry-threshold-analysis/`
**Status:** Planning complete, experiments pending
**Research Question:** What state-level minority percentage is required for VRA compliance to be geographically feasible?

**Key Contributions:**
1. Empirical threshold identification: ~40-42% for moderate clustering
2. Feasibility metric: $F = \frac{\text{state\\_minority} \times k}{t} \times C$
3. Geographic vs algorithmic limits distinction
4. Policy tool for courts/legislatures

**Data Requirements:**
- Complete VRA results for all 5 states (edge-weighted + multi-constraint)
- Geographic clustering metrics (Moran's I, clustering index)
- Synthetic state generation for threshold curve analysis

**See:** `gerry-threshold-analysis/PLAN.md` for detailed experiments and timeline

---

### Paper 6: Compactness-VRA Tradeoff 📋 PLANNED
**Directory:** `gerry-compactness-tradeoff/`
**Status:** Planning complete, experiments pending
**Research Question:** What is the quantitative tradeoff between compactness and VRA compliance?

**Key Contributions:**
1. Pareto frontier mapping across parameter space
2. Surprising finding: Alabama achieves 2 MM districts with BETTER compactness (-1.4% edge cut)
3. Debunk myth: VRA-compactness conflict is algorithm artifact, not inherent
4. Policy tool: Transparent tradeoff quantification

**Data Requirements:**
- Baseline compactness (no VRA goals) for all 5 states
- Geometric metrics (Polsby-Popper, Reock, convex hull) for all configurations
- Pareto frontier identification for each state

**See:** `gerry-compactness-tradeoff/PLAN.md` for detailed experiments and timeline

---

### Paper 7: Temporal Stability 📋 PLANNED
**Directory:** `gerry-temporal-stability/`
**Status:** Planning complete, experiments pending
**Research Question:** Do recursive bisection's hierarchical structures provide greater temporal stability across census cycles?

**Key Contributions:**
1. Temporal stability measurement across 2010→2020 transition
2. Hierarchical structure preservation demonstrated
3. Performance-stability tradeoff quantification (+4.3 points performance vs +14 points stability)
4. Predictions for 2030 redistricting

**Data Requirements:**
- 2010 census redistricting for all 5 states (n-way + recursive)
- Census tract relationship files (2010→2020 mapping)
- Stability metrics: tract reassignment, population disruption, boundary stability

**See:** `gerry-temporal-stability/PLAN.md` for detailed experiments and timeline

---

## Research Dependencies

### Paper Completion Order

**Priority 1: Fill Data Gaps for Paper 2**
- Run n-way vs recursive comparison for Georgia, Mississippi, Louisiana, South Carolina
- Complete Paper 2 with full 5-state results
- Estimated time: 1 week

**Priority 2: Papers 3-4 (Build on Papers 1-2)**
- Paper 3 (Adaptive): Depends on Paper 2 baseline
- Paper 4 (Multi vs Edge): Depends on Paper 1 edge-weighted results
- Can run in parallel
- Estimated time: 2 weeks each

**Priority 3: Papers 5-6 (Cross-Paper Analysis)**
- Paper 5 (Threshold): Requires Papers 1 + 4 (edge-weighted + multi-constraint results)
- Paper 6 (Compactness): Requires Papers 1 + 2 + 4 (all methods)
- Can run in parallel after Papers 1-4 complete
- Estimated time: 3 weeks each

**Priority 4: Paper 7 (Multi-Census)**
- Paper 7 (Temporal): Requires 2010 census data + Papers 2-3 (n-way + recursive methods)
- Most data-intensive (need 2010 redistricting)
- Estimated time: 4 weeks

### Data Collection Status

| State | Edge-Weighted (Paper 1) | N-Way (Paper 2) | Recursive (Paper 2) | Multi-Constraint (Paper 4) |
|-------|------------------------|-----------------|---------------------|---------------------------|
| Alabama | ✅ Complete | ✅ Complete | ✅ Complete | ⚠️ Partial |
| Georgia | ✅ Complete | ❌ Need | ❌ Need | ❌ Need |
| Mississippi | ✅ Complete | ❌ Need | ❌ Need | ❌ Need |
| Louisiana | ✅ Complete | ❌ Need | ❌ Need | ❌ Need |
| South Carolina | ✅ Complete | ❌ Need | ❌ Need | ❌ Need |

**Next Action:** Fill data gaps for Papers 2 & 4 (n-way, recursive, multi-constraint for 4 states)

---

## Shared Resources

### Code
- **Edge-weighting implementation:** `scripts/pipeline/test_edge_weighting_comprehensive.py`
- **METIS wrapper:** `src/apportionment/partition/metis_wrapper.py`
- **Recursive bisection:** `src/apportionment/partition/recursive_bisection.py`

### Data
- **Census tracts (2020):** `outputs/data/2020/units/{state}/`
- **Demographics (2020):** `outputs/data/2020/demographics/{state}/`
- **Adjacency (2020):** `outputs/data/2020/adjacency/{state}/`
- **Census tracts (2010):** `data/2010/tiger/tracts/{state}/` (need to process)

### Results
- **Edge-weighting ablation:** `gerry-vra-compliance/results/edge_weighting_ablation_study.csv` (140 configs)
- **Multi-constraint:** Need to create comprehensive CSV
- **N-way vs recursive:** Stored in Paper 2 LaTeX tables (need to extract to CSV)

---

## Timeline

### Phase 1: Complete Papers 1-2 (Weeks 1-2)
- **Week 1:** Fill Paper 2 data gaps (n-way + recursive for 4 states)
- **Week 2:** Update Paper 2 with complete results, compile LaTeX

### Phase 2: Papers 3-4 (Weeks 3-6)
- **Weeks 3-4:** Paper 3 experiments + writing (adaptive bisection)
- **Weeks 5-6:** Paper 4 experiments + writing (multi-constraint comparison)

### Phase 3: Papers 5-6 (Weeks 7-12)
- **Weeks 7-9:** Paper 5 experiments + writing (threshold analysis + synthetic states)
- **Weeks 10-12:** Paper 6 experiments + writing (compactness-VRA tradeoff)

### Phase 4: Paper 7 (Weeks 13-16)
- **Weeks 13-14:** 2010 census data processing + redistricting
- **Weeks 15-16:** Temporal stability analysis + writing

**Total Estimated Time:** 16 weeks (4 months) for all 7 papers

---

## Submission Strategy

### Target Venues

**Paper 1 (Edge-Weighted):**
- Primary: *Political Analysis* (methodology focus)
- Alternative: *Journal of Computational Social Science*

**Paper 2 (N-Way vs Recursive):**
- Primary: *SIAM Journal on Applied Mathematics* (algorithm comparison)
- Alternative: *ACM Transactions on Spatial Algorithms and Systems*

**Papers 3-4 (Adaptive, Multi vs Edge):**
- Primary: Conference submission (e.g., *KDD*, *SIAM Data Mining*)
- Alternative: Workshop papers

**Papers 5-6 (Threshold, Compactness):**
- Primary: *Election Law Journal* (policy implications)
- Alternative: *Journal of Politics* (empirical findings)

**Paper 7 (Temporal Stability):**
- Primary: *Political Geography* (temporal analysis focus)
- Alternative: *Electoral Studies*

### Bundling Strategy

**Option A: Monograph**
- Compile all 7 papers into book manuscript
- Target: Cambridge University Press, MIT Press

**Option B: Journal Special Issue**
- Coordinate special issue on "Algorithmic Redistricting"
- Lead paper (Paper 1) + invited submissions

**Option C: Independent Submissions**
- Submit each paper separately to best-fit venue
- Cite previous papers in sequence

---

## Contact & Collaboration

For questions, data requests, or collaboration inquiries related to this research portfolio, see individual paper READMEs or PLAN.md files for specific contact information.

---

## License

All code, data, and paper drafts in this directory are [LICENSE TBD - specify license for research materials].

---

## Changelog

- **2026-02-07**: Created research portfolio index, PLAN.md files for Papers 3-7
- **2026-02-07**: Completed Papers 1-2 (edge-weighted VRA, n-way vs recursive)
- **2026-02-06**: Initial Paper 1 breakthrough (edge-weighting achieves 2 MM districts in Alabama)
