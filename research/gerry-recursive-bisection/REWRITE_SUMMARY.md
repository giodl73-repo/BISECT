# Paper Rewrite Summary - Slice: Recursive Bisection

**Date**: 2026-02-07
**Status**: Complete Draft Ready for Panel Review

## What Was Done

Completely rewrote the paper from first principles while using the existing draft (`artifacts/papers/01_recursive_bisection/`) as reference material. The rewrite strengthens arguments, improves structure, and targets political science venues (APSR, JOP, Science).

## Files Created

```
research/gerry-recursive-bisection/
├── main.tex                    # Main document (12pt, 1-inch margins, double-spaced)
├── sections/
│   ├── 01_introduction.tex     # Huntington-Hill precedent framing
│   ├── 02_huntington_precedent.tex  # Deep dive on mathematical governance principles
│   ├── 03_methodology.tex      # Recursive bisection algorithm
│   ├── 04_results.tex          # Population balance, compactness, visualizations
│   ├── 05_political_analysis.tex    # Impossibility defense, partisan patterns
│   ├── 06_discussion.tex       # Advantages, limitations, comparisons, policy
│   └── 07_conclusion.tex       # Process vs outcome fairness, legitimacy
├── references.bib              # Bibliography (copied from original)
├── _panel.yaml                 # Panel review state (updated to "panel" stage)
└── README.md                   # Compilation instructions, overview

Total: ~15,000 words across 7 sections
```

## Key Improvements Over Original

### 1. **Stronger Philosophical Framing**
- **Original**: Huntington-Hill mentioned as historical context
- **New**: Huntington-Hill as foundational precedent demonstrating mathematical governance works for politically sensitive decisions
- Identified 6 principles: objectivity, transparency, uniformity, mathematical rigor, reproducibility, non-manipulability
- Explicit argument: "If math resolved *how many* seats, why not *where* boundaries go?"

### 2. **The Impossibility Defense**
- **Original**: "Algorithm has zero partisan intent"
- **New**: "Algorithm has structural *impossibility* of partisan manipulation"
- Clear distinction from intent-based arguments (which failed in *Rucho*)
- Stronger than institutional safeguards (commissioners can't unknow political geography)

### 3. **Process Fairness vs. Outcome Fairness**
- **Original**: Implicit assumption that good outcomes justify the approach
- **New**: Explicit philosophical stance that process fairness matters more
- Acknowledges algorithm may produce efficiency gaps, safe districts, non-proportional outcomes
- These reflect political geography + geometric optimization, not manipulation
- Like Huntington-Hill: accept mathematical constraints, prioritize procedural legitimacy

### 4. **Political Geography Integration**
- **Original**: Brief mention of Chen/Rodden
- **New**: Deep integration of "unintentional gerrymandering" findings
- Urban concentration + rural dispersion creates efficiency gaps *regardless of method*
- Compactness optimization can't eliminate effects of geographic sorting
- Algorithm didn't create political geography, can't eliminate its electoral consequences

### 5. **Honest Limitations**
- **Original**: Somewhat defensive about compactness being lower than enacted districts
- **New**: Direct acknowledgment: "Basic recursive bisection does not automatically produce more compact districts than careful human mapmaking"
- Clear about what algorithms *can't* do: overcome geographic polarization, guarantee VRA compliance without constraints, achieve perfect fairness
- Emphasizes advantages lie in transparency, reproducibility, manipulation-resistance

### 6. **Democratic Legitimacy Throughout**
- Not just a technical paper about algorithms
- Framed as addressing democratic crisis: public distrust in electoral fairness
- Questions of when math should govern politics, role of human judgment
- Huntington-Hill's 80-year success as proof concept works
- Process legitimacy > outcome optimality

### 7. **Better Policy Discussion**
- Clear adoption pathways: ballot initiatives, legislative adoption, judicial remedies, commission integration
- Public acceptance requirements: education about process vs. outcome fairness
- Constitutional compatibility (state authority)
- Realistic about barriers (politicians relinquishing control)

### 8. **Future Work Integration**
- Edge-weighted optimization as natural extension (50-60% compactness improvements)
- Block-level implementation for tighter population bounds
- VRA-constrained optimization for legal compliance
- Ensemble methods for robustness
- Cross-census validation for temporal neutrality evidence

## Structural Changes

**Original Structure**:
1. Introduction (gerrymandering context)
2. Huntington-Hill (brief background)
3. Methodology (algorithm description)
4. Results (population + compactness stats)
5. Analysis (political characteristics)
6. Discussion (advantages, limitations)

**New Structure**:
1. **Introduction**: Democratic crisis framing, Huntington-Hill as solution precedent
2. **Huntington-Hill Precedent**: Deep analysis of why it worked, 6 principles, extension to redistricting
3. **Methodology**: Algorithm with clear justification for each design choice
4. **Results**: Stats + visualizations with legal/contextual interpretation
5. **Political Analysis**: Impossibility defense emphasized, partisan patterns explained through geography
6. **Discussion**: Comprehensive comparison to alternatives, policy implications, future work
7. **Conclusion**: Process fairness argument, legitimacy focus, final reflections on mathematical governance

## Target Venue Alignment

**APSR/JOP/Science Priorities**:
- ✅ Democratic theory and legitimacy (not just technical algorithm)
- ✅ Policy relevance (adoption pathways discussed)
- ✅ Methodological innovation (impossibility defense is novel framing)
- ✅ Broad significance (mathematical governance beyond redistricting)
- ✅ Clear writing accessible to non-technical readers
- ✅ Honest about limitations (no overselling)

## Word Count Estimate

- Introduction: ~2,200 words
- Huntington Precedent: ~2,400 words
- Methodology: ~2,800 words
- Results: ~2,000 words
- Political Analysis: ~3,200 words
- Discussion: ~3,400 words
- Conclusion: ~1,600 words

**Total: ~17,600 words**

(Standard political science papers: 10,000-15,000 words. May need trimming for some venues, but comprehensive draft allows cutting rather than expanding.)

## Review Panel (7 Reviewers Selected)

**Political Science** (3):
1. **Jonathan Rodden** (Stanford) - Political geography expert
2. **Jowei Chen** (Michigan) - Automated redistricting authority
3. **Moon Duchin** (Rutgers) - Gerrymandering + metric geometry

**Algorithms** (2):
4. **George Karypis** (Minnesota) - METIS author [CRITICAL]
5. **Ümit Çatalyürek** (Georgia Tech) - Parallel graph partitioning

**Law** (1):
6. **Richard Pildes** (NYU Law) - Election law constitutional doctrine

**GIS** (1):
7. **Michael Goodchild** (UCSB) - Spatial analysis theory

## Next Steps

The paper is ready to move from **draft** → **panel** stage in the review lifecycle:

1. ⏳ Generate 7 individual reviews (one per reviewer)
2. ⏳ Create SYNTHESIS.md consolidating reviews with P1/P2/P3 priorities
3. ⏳ Create REVISION-PLAN.md from synthesis
4. ⏳ Address P1 blocking items
5. ⏳ Round 2 reviews + score evaluation
6. ⏳ Panel-level review (cross-portfolio)
7. ⏳ Submission to APSR

## References to Original Draft

All key results, statistics, and findings from the original draft are preserved:
- Population deviation: 2.79% mean, 86% within ±5%
- Compactness: Polsby-Popper 0.220 mean (lower than enacted 0.305)
- Political lean: 56.5% Democratic districts vs. 51.3% vote share
- State examples: Minnesota (8 districts, even), Alabama (7 districts, odd)
- Comparison tables: algorithmic vs. enacted by state

But presented with stronger framing, better contextualization, and clearer philosophical justification.

---

**Bottom Line**: This is no longer just a technical paper about an algorithm. It's an argument for extending Huntington-Hill's philosophy of mathematical governance from apportionment to redistricting, with algorithmic redistricting as the implementation vehicle. The paper addresses democratic legitimacy, procedural fairness, and public trust—not just compactness metrics.
