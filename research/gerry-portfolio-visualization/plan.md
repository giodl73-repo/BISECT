# Portfolio-Level Visualization — Plan

**Artifact Type**: PP3.3 (Optional Enhancement)
**Goal**: Single figure showing all 10 papers' contributions and dependencies
**Estimated Effort**: 4 hours
**Status**: Not started

---

## Objective

Create a visual representation of the research program architecture that helps readers understand:
- How papers relate to each other (dependencies)
- The flow from foundational → extensions → investigations
- Key findings from each paper at a glance
- Entry points for different reader interests

---

## Format Options

### Option 1: Directed Graph (LaTeX/TikZ)
```
                   ┌──────────────────────────┐
                   │  recursive-bisection     │
                   │  • Impossibility defense │
                   │  • 8.4/10 score (Tier A) │
                   └──────────┬───────────────┘
                              │
                 ┌────────────┴────────────┐
                 ▼                         ▼
      ┌──────────────────┐       ┌──────────────────┐
      │ edge-weighted    │       │ vra-compliance   │
      │ • +2.1% compact  │       │ • +69 MM surplus │
      │ • 8.2/10 (A-)    │       │ • 8.0/10 (A-)    │
      └────────┬─────────┘       └────────┬─────────┘
               │                           │
      ┌────────┴────────┐         ┌───────┴────────┐
      ▼                 ▼         ▼                ▼
[compactness]  [temporal]  [threshold]  [cross-census]
```

**Advantages**: Clear dependencies, hierarchical structure visible
**Implementation**: LaTeX with TikZ, or Python with networkx + matplotlib

### Option 2: Swimlane Diagram
Organize papers by research theme/track:
- **Track 1: Core Method** (recursive-bisection)
- **Track 2: Optimization** (edge-weighted, adaptive-bisection)
- **Track 3: VRA Compliance** (vra-compliance, threshold-analysis, compactness-tradeoff)
- **Track 4: Validation** (cross-census-validation, temporal-stability)
- **Track 5: Alternatives** (multi-vs-edge, nway-vs-recursive)

**Advantages**: Shows thematic organization, easier to follow research threads
**Implementation**: Draw.io, PowerPoint, or LaTeX with custom layout

### Option 3: Interactive Web Visualization
D3.js force-directed graph with:
- Nodes = papers (sized by score, colored by tier)
- Edges = citations/dependencies
- Hover = show paper title, key finding, score
- Click = link to PDF

**Advantages**: Interactive exploration, scales well, shareable
**Implementation**: HTML + D3.js or Plotly

---

## Required Content Per Paper

For each of the 10 papers, include:
1. **Short name** (e.g., "recursive-bisection")
2. **Key finding** (1 bullet point, <10 words)
3. **Score and tier** (e.g., "8.4/10, Tier A")
4. **Primary venue** (e.g., "APSR / Science")
5. **Dependencies** (which papers it builds on)

---

## Dependency Map (from Panel Review)

```
Foundation:
  └─ recursive-bisection ← core method, impossibility defense

Primary Extensions:
  ├─ edge-weighted-bisection ← add compactness optimization
  └─ vra-compliance ← add demographic awareness

Systematic Investigations:
  ├─ adaptive-bisection ← parameter sensitivity, tree structure
  ├─ temporal-stability ← cross-census validation
  ├─ cross-census-validation ← tract correspondence methodology
  ├─ compactness-tradeoff ← VRA-compactness Pareto frontier
  ├─ threshold-analysis ← VRA feasibility limits (42%)
  ├─ multi-vs-edge ← why single-objective wins
  └─ nway-vs-recursive ← algorithmic alternatives
```

---

## Design Specifications

**Dimensions**:
- Single page (8.5" × 11" portrait or 11" × 17" landscape)
- High resolution (300 DPI for publication)
- Colorblind-friendly palette

**Colors** (suggested):
- **Tier A/A-**: Green shades (#2E7D32, #66BB6A)
- **Tier B+**: Blue shades (#1976D2, #42A5F5)
- **Tier B/B-**: Gray shades (#616161, #9E9E9E)
- **Foundations**: Bold border
- **Extensions**: Dashed border
- **Investigations**: Solid border

**Typography**:
- Title: 16pt bold
- Paper names: 12pt
- Key findings: 10pt
- Scores: 9pt italics

---

## Deliverables

1. **Source file**: `portfolio-architecture.tex` or `portfolio-architecture.py`
2. **Output**: `portfolio-architecture.pdf` (high-res)
3. **Web version** (optional): `portfolio-architecture.html`
4. **README**: Brief explanation of how to regenerate/customize

---

## Usage Context

This visualization will be used in:
- Paper submissions (as supplementary figure)
- Grant proposals (showing research program scope)
- Conference presentations (overview slide)
- README.md (orient new readers)
- Portfolio guide document (see gerry-portfolio-guide/)

---

## Implementation Steps

1. **Data collection**: Extract paper metadata from `_panel.yaml` files
2. **Graph construction**: Define nodes (papers) and edges (dependencies)
3. **Layout algorithm**: Position nodes hierarchically (top = foundation, bottom = leaves)
4. **Rendering**: Generate PDF with labels, scores, key findings
5. **Iteration**: Review with 2-3 colleagues, adjust layout/colors
6. **Export**: Final PDF + source code

---

## Next Actions

- [ ] Choose visualization format (Option 1/2/3)
- [ ] Extract paper metadata from all `_panel.yaml` files
- [ ] Create initial draft layout
- [ ] Get feedback from advisor/collaborators
- [ ] Finalize and integrate into portfolio materials

---

**Created**: 2026-02-08
**Panel Reference**: REVIEW_PANEL.md Section V, PP3.1
**Related Artifacts**: gerry-portfolio-guide/, README.md
