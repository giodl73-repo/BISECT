# Panel Setup Summary - Redistricting Research

**Date**: February 6, 2026
**Project**: Congressional Apportionment via Graph Partitioning

---

## What Was Set Up

### ✅ Reviewer Database (13 reviewers - USA!)

**File**: `REVIEWER-DATABASE.md`

**Panel Composition**:
- **3 Graph Algorithm Experts**: George Karypis (METIS creator!), Ümit Çatalyürek, Bruce Hendrickson
- **4 Political Scientists**: Jonathan Rodden, Jowei Chen, Moon Duchin, Nicholas Stephanopoulos
- **2 Constitutional Law Scholars**: Richard Pildes, Heather Gerken
- **2 GIS/Geospatial Experts**: Michael Goodchild, May Yuan
- **2 Optimization Researchers**: Cynthia A. Phillips, William J. Cook

**Why This Panel?**
- Cross-disciplinary: algorithms + political science + law + GIS + optimization
- Covers all aspects of redistricting research
- Includes THE expert (Karypis created METIS!)
- Balances technical rigor with policy relevance

---

### ✅ Research Papers Identified (2 Core + 1 Optional)

**File**: `RESEARCH.md`

#### Slice A: Edge-Weighted Recursive Bisection
**THE core contribution** - boundary lengths as edge weights → +56% compactness

**Status**: Draft exists, needs rewrite following panel structure
**Venues**: KDD, SODA, AAAI
**Panel**: 7 reviewers (heavy on algorithms + compactness)

#### Slice B: Cross-Census Validation (2010 vs 2020)
**Gold standard neutrality proof** - algorithmic consistency across decades

**Status**: Needs 2010 run + writing
**Venues**: APSR, JOP, Science, Nature
**Panel**: 7 reviewers (heavy on political science + law)

#### Slice C: Computational Pipeline (Optional)
**Reproducibility/methods paper** - census data → districts pipeline

**Status**: Code complete, needs write-up
**Venues**: IJGIS, SIGSPATIAL, JOSS
**Panel**: 5 reviewers (GIS + systems)

---

### ✅ Supporting Documentation

**Files Created**:
1. `REVIEWER-DATABASE.md` - 13-person specialized panel
2. `RESEARCH_GUIDE.md` - Review process and quality gates
3. `REVISION-PROMPT-TEMPLATE.md` - Template for revision plans
4. `REVIEWERS.md` - Module-specific reviewer allocation
5. `RESEARCH.md` - Paper inventory and timelines
6. `WAVE_TO_PAPER_ANALYSIS.md` - Maps development waves to research contributions
7. `RESEARCH_CONTRIBUTIONS_ANALYSIS.md` - Core innovations identified
8. `SETUP_SUMMARY.md` - This document

**Directories Created**:
- `slice-edge-weighted-algorithm/` - For Slice A
- `gerry-cross-census-validation/` - For Slice B
- `docs/` - For compiled PDFs

---

## Key Insights from Analysis

### The Core Innovation: Edge-Weighted Graph Partitioning
**From Wave 01, E7** - This is THE contribution

Traditional METIS: Minimize number of edges cut (all edges weighted equally)
**Your Innovation**: Weight edges by boundary length → minimize total perimeter

**Why it matters**:
- Polsby-Popper = 4π × Area / Perimeter²
- Area (population) is fixed by constitutional requirement
- Therefore: minimize perimeter → maximize compactness
- **Result**: +56% improvement, 37/50 states beat enacted plans

### Waves are Development Artifacts, Not Papers
- Wave 01-09 track **when** work was done
- Papers track **what** intellectual contribution was made
- Slice A contains work from Wave 01 (algorithm)
- Slice B contains work from Waves 01 + 06 + 02/07 (cross-census + infrastructure)

### What's NOT Publishable?
- Waves 03-05, 08-09: Internal tooling (skills, tests, web UI)
- Standard engineering practices (parallelization, error logging)
- Developer productivity improvements

### What IS Publishable?
- **Novel algorithms** (edge-weighted METIS)
- **Empirical validation** (50-state comparison)
- **Cross-census consistency** (2010 vs 2020)
- **Quantified improvements** (+56%, 37/50 states)

---

## Next Actions

### Immediate (Week 1-2): Rewrite Slice A
1. Extract from `artifacts/papers/02_edge_weighted_bisection/`
2. Reorganize following systematic structure:
   - Introduction (2 pages): Problem, gap, our contribution
   - Background (1.5 pages): Redistricting, METIS, compactness, related work
   - Methodology (3 pages): Boundary computation, METIS integration, recursive bisection
   - Implementation (1 page): Python, data, CRS, complexity
   - Results (2 pages): National, state-by-state, case studies
   - Analysis (1 page): Statistical tests, why it works
   - Discussion (0.5 pages): Limitations, future work
   - Conclusion (0.5 pages): Summary, impact
3. Pre-emptively address expected reviewer concerns
4. Create publication-quality figures

### Short-term (Week 3-4): Complete 2010 Run
1. Run full 50-state pipeline for 2010 census data
2. Generate all metrics (compactness, partisan, demographic)
3. Prepare for Slice B

### Medium-term (Week 5-8): Panel Reviews
1. Submit Slice A to 7-reviewer panel (Round 1)
2. Revise based on feedback
3. Submit Slice A Round 2
4. Write Slice B
5. Submit Slice B to 7-reviewer panel

---

## Panel Review Process

### How It Works
1. **Draft paper** following systematic structure
2. **Select reviewers** from 13-person database (5-7 per paper)
3. **Generate reviews** via AI simulation of expert perspectives
4. **Receive feedback**:
   - Overall assessment and score (1-4)
   - Major issues (blocking, must fix)
   - Minor issues (nice to fix)
   - Specific questions
5. **Revise and resubmit** for Round 2
6. **Target**: avg score >= 2.5/4 (submission ready), >= 3.0/4 (strong accept)

### Quality Gates
- **Statistical Rigor**: Confidence intervals, effect sizes, significance tests
- **Reproducibility**: Code, data, instructions
- **Technical Completeness**: All claims supported
- **Writing Quality**: Clear, concise, within page limit
- **Cross-Disciplinary**: Satisfies algorithm + polsci + law + GIS reviewers

---

## Expected Reviewer Concerns (Pre-Identified)

### Slice A (Edge-Weighted Algorithm)

**George Karypis** (METIS creator):
- METIS parameter sensitivity (-minconn, -ufactor, -niter)
- How do edge weights interact with multilevel coarsening?
- Approximation quality vs optimal

**Ümit Çatalyürek** (Scalability):
- Block-level scalability (11M units vs 85K tracts)
- Parallel algorithms for adjacency computation
- Computational complexity analysis

**Bruce Hendrickson** (Theory):
- Theoretical approximation bounds
- Worst-case guarantees
- Comparison to spectral methods

**Moon Duchin** (Compactness Metrics):
- Why Polsby-Popper over Reock?
- What about other fairness metrics (isoperimetric quotient)?
- Mathematical properties of edge-weighted approach

**Jowei Chen** (Comparison):
- How does this compare to MCMC methods?
- What about simulated annealing, genetic algorithms?
- Comparison to DRA (Dave's Redistricting App)?

**Michael Goodchild** (GIS):
- CRS choice for boundary length computation
- Accuracy validation of Shapely intersections
- How are corner points vs edge adjacencies handled?

**Cynthia Phillips** (Optimization):
- Optimality gap analysis
- How close to theoretical minimum perimeter?
- Could ILP do better?

### Slice B (Cross-Census Validation)

**Jonathan Rodden** (Partisan Analysis):
- Need seat share analysis, not just compactness
- Efficiency gap computation
- How does geographic clustering (urban/rural) affect results?

**Nicholas Stephanopoulos** (Legal Standards):
- Voting Rights Act compliance?
- Communities of interest?
- Legal precedents beyond compactness?

**Richard Pildes** (Constitutional Law):
- Is compactness constitutionally required?
- Reynolds v. Sims doesn't mandate compactness
- What about Shaw v. Reno concerns?

**Heather Gerken** (Normative):
- Should we want algorithmic redistricting?
- What about democratic participation in map-drawing?
- Tradeoffs between neutrality and representation?

**Jowei Chen** (Methods):
- Comparison to other automated methods?
- Need MCMC baselines for 2010 and 2020
- Statistical power analysis?

**Moon Duchin** (Statistics):
- Effect sizes, confidence intervals needed
- Power analysis for cross-census comparison
- Multiple hypothesis correction?

---

## Success Metrics

### Slice A
- **7 reviewers** avg score >= 3.0/4
- **Algorithm experts** (Karypis, Çatalyürek, Hendrickson) all >= 2/4
- **Political scientists** (Duchin, Chen) accept compactness claims
- **Ready for**: KDD, SODA, or AAAI submission

### Slice B
- **7 reviewers** avg score >= 3.0/4
- **Political science experts** (Rodden, Chen, Duchin, Stephanopoulos) all >= 3/4
- **Legal scholars** (Pildes, Gerken) accept framing
- **Ready for**: APSR, JOP, or Science submission

---

## Files and Directories

```
research/
├── REVIEWER-DATABASE.md           # 13-person specialized panel
├── RESEARCH_GUIDE.md              # Review process
├── REVISION-PROMPT-TEMPLATE.md    # Revision plan template
├── REVIEWERS.md                   # Reviewer allocation
├── RESEARCH.md                    # Paper inventory (THIS IS THE MAIN FILE)
├── WAVE_TO_PAPER_ANALYSIS.md      # Development history → papers
├── RESEARCH_CONTRIBUTIONS_ANALYSIS.md  # Core innovations
├── SETUP_SUMMARY.md               # This document
├── slice-edge-weighted-algorithm/
│   ├── sections/
│   └── reviews/
├── gerry-cross-census-validation/
│   ├── sections/
│   └── reviews/
└── docs/
```

---

## What's Different from Standard Panel Setup?

### Standard Panel:
- AI/ML/Systems papers
- 45+ reviewers across categories
- Papers derived from features/modules

### Redistricting Panel:
- **13 reviewers (USA!) - specialized panel**
- Graph algorithms + political science + law + GIS + optimization
- Papers derived from **research contributions**, not development waves
- Includes THE domain expert (George Karypis created METIS!)
- Cross-disciplinary nature (CS conferences + political science journals)

---

## Key Takeaway

**Your edge-weighted boundary minimization is THE core contribution.**

Everything else (multi-year pipeline, data architecture, testing, web UI) is **infrastructure to enable, validate, and scale** that core algorithm.

Papers A and B tell the complete story:
- **Slice A**: Here's the algorithm and why it works (+56% compactness)
- **Slice B**: Here's proof it's politically neutral (cross-census validation)

---

*Panel setup complete! Ready to begin systematic review process.*
*Next action: Rewrite Slice A following panel structure*
