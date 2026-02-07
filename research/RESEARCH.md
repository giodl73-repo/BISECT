# Slice Research Papers

**Project**: Slice (Congressional Apportionment via Graph Partitioning)
**Author**: Gio Della-Libera
**Papers**: 3 slices

---

## Paper Inventory

| Slice | Directory | Title | Source | Venue Target | Status |
|-------|-----------|-------|--------|-------------|--------|
| **Recursive Bisection** | [slice-recursive-bisection](slice-recursive-bisection/) | Recursive Bisection for Congressional Redistricting: Extending Huntington-Hill to Boundary Design | artifacts/papers/01_* | APSR / JOP / Science | Draft exists |
| **Edge-Weighted** | [slice-edge-weighted-bisection](slice-edge-weighted-bisection/) | Edge-Weighted Recursive Bisection for Compact Congressional Districts | artifacts/papers/02_* | KDD / SODA / AAAI | Draft exists |
| **Cross-Census** | [slice-cross-census-validation](slice-cross-census-validation/) | Algorithmic Redistricting Across Two Decades: Evidence for Geographic Neutrality | artifacts/papers/03_* | APSR / JOP / Science | Partial draft |

---

## Slice Dependencies

```
[Recursive Bisection] (foundational - the algorithm)
     |
     +──→ [Edge-Weighted] (enhancement - +56% compactness)
     |
     +──→ [Cross-Census] (validation - neutrality proof)
```

---

## Slice 1: Recursive Bisection

**THE Foundational Algorithm**

### Core Contribution
Applying recursive graph bisection to congressional redistricting, extending Huntington-Hill's mathematical objectivity from apportionment (how many seats) to boundary design (where lines go).

### Key Innovations
1. **Hierarchical graph partitioning model** for redistricting
2. **Recursive algorithm** handling arbitrary district counts (odd/even via unequal splits)
3. **Water-based adjacency** via county-based bridge connections
4. **Huntington-Hill philosophy extension**: "If mathematical objectivity resolved *how many* seats, why not *where* boundaries go?"

### Results
- All 435 districts across 50 states
- Mean population deviation: 2.79%
- Geographic contiguity by construction
- Zero partisan intent by design

### Target Venues
- **Primary**: APSR (American Political Science Review), JOP (Journal of Politics), Science
- **Alternative**: Nature, PNAS, Political Analysis

### Reviewer Panel (7 reviewers)
- **Political Science** (3): Jonathan Rodden, Jowei Chen, Moon Duchin
- **Algorithms** (2): George Karypis, Ümit Çatalyürek
- **Law** (1): Richard Pildes
- **GIS** (1): Michael Goodchild

### Status
- **Existing Draft**: `artifacts/papers/01_recursive_bisection/recursive_bisection.tex`
- **Action**: Rewrite following panel structure
- **Data**: ✅ Complete (2020 run)

---

## Slice 2: Edge-Weighted Bisection

**The Compactness Enhancement**

### Core Contribution
Edge weights = boundary lengths → METIS minimizes total perimeter → +56% compactness improvement

### Key Innovation
First application of METIS with geometric boundary lengths as edge weights to redistricting. Handles water crossings, islands, point adjacencies.

### Results
- **+56% compactness** over baseline (0.367 vs 0.235 Polsby-Popper)
- **+20% better** than enacted 2020 districts (0.367 vs 0.305)
- **37 of 50 states** exceed enacted compactness
- **Partisan states**: Illinois +174%, Louisiana +104%, New Hampshire +102%

### Target Venues
- **Primary**: KDD (data mining), SODA (algorithms), AAAI
- **Alternative**: IJCAI, ICML (applications track)

### Reviewer Panel (7 reviewers)
- **Algorithms** (3): George Karypis (MUST HAVE), Ümit Çatalyürek, Bruce Hendrickson
- **Political Science** (2): Moon Duchin, Jowei Chen
- **GIS** (1): Michael Goodchild
- **Optimization** (1): Cynthia Phillips

### Status
- **Existing Draft**: `artifacts/papers/02_edge_weighted_bisection/edge_weighted_bisection.tex`
- **Action**: Rewrite following panel structure
- **Data**: ✅ Complete (2020 run)

---

## Slice 3: Cross-Census Validation

**The Neutrality Proof**

### Core Contribution
Identical algorithm on 2010 vs 2020 census → only 10.3% variation → proves geographic (not political) factors drive performance

### Key Innovation
Cross-census validation as gold standard for political neutrality claims. Demonstrates temporal stability across different political environments.

### Results
- **2020**: 0.367 PP (algorithmic) vs 0.305 (enacted)
- **2010**: 0.320 PP (algorithmic) vs 0.353 (enacted)
- **Only 10.3% variation** between census years
- **50% gap reduction** from 2010→2020 (measures reform effectiveness)

### Target Venues
- **Primary**: APSR, JOP, Science, Nature
- **Alternative**: Political Analysis, Election Law Journal, PNAS

### Reviewer Panel (7 reviewers)
- **Political Science** (4): Jonathan Rodden, Jowei Chen, Moon Duchin, Nicholas Stephanopoulos
- **Law** (2): Richard Pildes, Heather Gerken
- **Algorithms** (1): George Karypis (consistency check)

### Status
- **Existing Draft**: Partial in `artifacts/papers/03_combined_recursive_bisection/`
- **Data Status**:
  - ✅ 2020: Complete
  - ⏳ 2010: Tract data ready, needs full redistricting run (~1-2 weeks)
  - ⏳ 2000: Awaiting NHGIS shapefiles
- **Action**: Complete 2010 run, then write

---

## Review Timeline

### Phase 1: Slice 1 (Recursive Bisection)
**Week 1-2**: Rewrite from `artifacts/papers/01_*`
**Week 3**: Internal review
**Week 4**: Panel review Round 1 (7 reviewers)
**Week 5**: Revisions + Round 2

### Phase 2: Slice 2 (Edge-Weighted)
**Week 1-2**: Rewrite from `artifacts/papers/02_*`
**Week 3**: Internal review
**Week 4**: Panel review Round 1 (7 reviewers)
**Week 5**: Revisions + Round 2

### Phase 3: Slice 3 (Cross-Census)
**Week 1-2**: Complete 2010 redistricting run
**Week 3-4**: Write slice (cross-census analysis)
**Week 5**: Internal review
**Week 6**: Panel review Round 1 (7 reviewers)
**Week 7**: Revisions + Round 2

---

## Success Criteria

### Slice 1 (Recursive Bisection)
- **Political science reviewers** accept Huntington-Hill extension argument
- **Algorithm reviewers** accept METIS application to redistricting
- **Legal reviewers** accept constitutional framing
- **Target**: avg score >= 3.0/4 for APSR/JOP/Science

### Slice 2 (Edge-Weighted)
- **Algorithm reviewers** (Karypis, Çatalyürek, Hendrickson) accept edge weight approach
- **Political science reviewers** accept compactness improvements
- **Target**: avg score >= 3.0/4 for KDD/SODA/AAAI

### Slice 3 (Cross-Census)
- **Political science reviewers** accept neutrality evidence
- **Legal scholars** approve policy implications
- **Target**: avg score >= 3.0/4 for APSR/JOP/Science

---

## What Makes Each Slice Publishable?

### Slice 1: The Foundation
- **Novel framing**: Extends Huntington-Hill philosophy to boundaries
- **Algorithmic contribution**: Recursive bisection with unequal splits
- **Engineering contribution**: Water-based adjacency handling
- **Policy relevance**: Mathematical objectivity for redistricting

### Slice 2: The Enhancement
- **Novel algorithm**: Edge weights = boundary lengths (first application to redistricting)
- **Quantified improvement**: +56% over baseline, +20% over enacted
- **National scale**: All 50 states, 435 districts
- **Handles complexity**: Water crossings, islands, variable state sizes

### Slice 3: The Proof
- **Cross-census validation**: Gold standard for neutrality claims
- **Temporal stability**: 10.3% variation across 20 years
- **Reform measurement**: Quantifies impact of redistricting reforms
- **Policy relevant**: Provides baseline for detecting gerrymandering

---

## Next Steps

1. ✅ Panel infrastructure setup
2. ✅ Contribution analysis (3 slices identified)
3. ⏳ Rewrite Slice 1 following panel structure
4. ⏳ Rewrite Slice 2 following panel structure
5. ⏳ Complete 2010 run for Slice 3
6. ⏳ Write Slice 3
7. ⏳ Submit all 3 slices to panel review

---

*Project: Slice - Congressional Apportionment via Graph Partitioning*
*Papers: 3 slices (recursive-bisection, edge-weighted-bisection, cross-census-validation)*
*Panel: 13 reviewers specialized for redistricting research*
