# Expert Reviewer Database - Redistricting Research

> **AI Simulation Disclosure**: This database supports an AI-simulated peer review
> system. The named researchers are **not** actual reviewers of this work. Their
> names, affiliations, and expertise areas are used to construct AI personas that
> emulate the *perspective and priorities* each researcher is known for, based on
> their published work, public talks, and documented research philosophy. No
> endorsement, affiliation, or participation by these individuals is implied.
> All reviews generated from these personas are synthetic outputs produced by a
> large language model (Claude, Anthropic).

A comprehensive pool of 13 expert reviewers for AI-simulated paper reviews of redistricting algorithm research. This specialized panel covers graph algorithms, political science, constitutional law, GIS, and optimization.

**Scope**: Congressional Redistricting via Graph Partitioning
**Core Topics**: METIS recursive bisection, compactness metrics, political neutrality, constitutional compliance

---

## Reviewer Categories

### Category: Graph Algorithms & Partitioning
*For papers on: graph partitioning, METIS, recursive bisection, edge-weighted algorithms*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **George Karypis** | University of Minnesota | METIS, graph partitioning, multilevel algorithms | What's the partitioning quality vs speed tradeoff? |
| **Ümit V. Çatalyürek** | Georgia Tech | Hypergraph partitioning, parallel algorithms | How does this scale with graph size? |
| **Bruce Hendrickson** | Sandia National Labs | Graph partitioning, load balancing, spectral methods | What are the theoretical guarantees? |

### Category: Political Science & Redistricting
*For papers on: gerrymandering, electoral systems, political geography, representation*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Jonathan Rodden** | Stanford | Political geography, gerrymandering, representation | How does geographic clustering affect partisan outcomes? |
| **Jowei Chen** | University of Michigan | Automated redistricting, compactness, neutrality | How does this compare to other automated methods? |
| **Moon Duchin** | Rutgers | Gerrymandering, metric geometry, fairness | What are the mathematical fairness properties? |
| **Nicholas Stephanopoulos** | Harvard Law | Efficiency gap, partisan symmetry, legal standards | Does this meet judicial standards for neutrality? |

### Category: Constitutional Law & Legal Standards
*For papers on: constitutional compliance, one-person-one-vote, Voting Rights Act*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Richard Pildes** | NYU Law | Election law, constitutional doctrine, VRA | Does this comply with constitutional requirements? |
| **Heather Gerken** | Yale Law | Election law, voting rights, federalism | What are the normative implications for democracy? |

### Category: GIS & Geospatial Analysis
*For papers on: spatial algorithms, census geography, geographic data processing*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Michael Goodchild** | UC Santa Barbara | GIS theory, spatial analysis, geography | How are geographic adjacency and topology handled? |
| **May Yuan** | University of Texas | Spatial algorithms, census data, temporal GIS | How does census tract resolution affect results? |

### Category: Optimization & Operations Research
*For papers on: combinatorial optimization, constraint satisfaction, heuristics*

| Name | Affiliation | Expertise | Key Question |
|------|-------------|-----------|--------------|
| **Cynthia A. Phillips** | Sandia National Labs | Combinatorial optimization, graph algorithms | What's the approximation quality? |
| **William J. Cook** | University of Waterloo | Combinatorial optimization, exact algorithms | Are there better bounds on solution quality? |

---

## Venue-Specific Selection Guides

### Political Science Venues (APSR, AJPS, JOP)
Focus: Political implications, representation, democratic theory
**Recommended reviewers**: Jonathan Rodden, Jowei Chen, Moon Duchin, Nicholas Stephanopoulos, Richard Pildes

### Law Reviews (Harvard, Yale, Stanford)
Focus: Constitutional compliance, legal standards, judicial precedents
**Recommended reviewers**: Richard Pildes, Heather Gerken, Nicholas Stephanopoulos, Moon Duchin

### Computer Science Venues (KDD, AAAI, SODA)
Focus: Algorithms, computational complexity, optimization
**Recommended reviewers**: George Karypis, Ümit Çatalyürek, Bruce Hendrickson, Cynthia Phillips, William Cook

### GIS Venues (IJGIS, GeoInformatica, ACM SIGSPATIAL)
Focus: Geospatial algorithms, census data, spatial analysis
**Recommended reviewers**: Michael Goodchild, May Yuan, Ümit Çatalyürek, Moon Duchin

### Interdisciplinary Venues (Science, Nature, PNAS)
Focus: Broad impact, methodological innovation, policy relevance
**Recommended reviewers**: Moon Duchin, George Karypis, Jonathan Rodden, Michael Goodchild, Richard Pildes

---

## Paper Type to Reviewer Selection

### Type: Core Algorithm Paper (METIS, Graph Partitioning)
**Must have**: George Karypis, Ümit Çatalyürek, Bruce Hendrickson
**Add for optimization**: Cynthia Phillips, William Cook
**Add for political context**: Jowei Chen

### Type: Compactness & Neutrality Paper
**Must have**: Moon Duchin, Jowei Chen, Nicholas Stephanopoulos
**Add for algorithms**: George Karypis
**Add for legal standards**: Richard Pildes

### Type: Political Analysis Paper
**Must have**: Jonathan Rodden, Jowei Chen, Nicholas Stephanopoulos
**Add for math**: Moon Duchin
**Add for law**: Richard Pildes, Heather Gerken

### Type: Data Pipeline & Infrastructure Paper
**Must have**: Michael Goodchild, May Yuan, Ümit Çatalyürek
**Add for algorithms**: George Karypis
**Add for optimization**: Cynthia Phillips

### Type: Constitutional Compliance Paper
**Must have**: Richard Pildes, Heather Gerken, Nicholas Stephanopoulos
**Add for political science**: Jonathan Rodden, Jowei Chen
**Add for metrics**: Moon Duchin

---

## Reviewer Expertise Tags

Quick reference for expertise matching:

| Tag | Reviewers |
|-----|-----------|
| `graph-partitioning` | George Karypis, Ümit Çatalyürek, Bruce Hendrickson |
| `metis` | George Karypis, Bruce Hendrickson |
| `political-geography` | Jonathan Rodden, Jowei Chen |
| `gerrymandering` | Moon Duchin, Jowei Chen, Nicholas Stephanopoulos |
| `constitutional-law` | Richard Pildes, Heather Gerken |
| `voting-rights` | Richard Pildes, Heather Gerken, Nicholas Stephanopoulos |
| `gis` | Michael Goodchild, May Yuan |
| `census-data` | May Yuan, Michael Goodchild |
| `optimization` | Cynthia Phillips, William Cook |
| `compactness` | Moon Duchin, Jowei Chen |
| `fairness-metrics` | Moon Duchin, Nicholas Stephanopoulos |
| `spatial-algorithms` | Michael Goodchild, Ümit Çatalyürek |

---

## Adding New Reviewers

When adding reviewers from new papers or citations:

1. **Identify expertise category** from their top papers
2. **Find their "key question"** - what do they always ask?
3. **Add affiliation** - current as of paper date
4. **Tag with expertise** for quick lookup
5. **Note venue preference** if known

Template:
```markdown
| **Name** | Affiliation | Expertise | Key Question |
```

---

## Special Considerations for Redistricting Research

### The 13-Person Panel (USA!)
This reviewer database contains exactly 13 reviewers, mirroring the number of original U.S. colonies and providing comprehensive coverage across:
- 3 graph algorithm/METIS experts
- 4 political science/redistricting experts
- 2 constitutional law experts
- 2 GIS/geospatial experts
- 2 optimization/operations research experts

### Cross-Disciplinary Nature
Redistricting research requires reviewers who understand:
- **Technical**: Graph algorithms, computational geometry, optimization
- **Political**: Electoral systems, representation, partisan effects
- **Legal**: Constitutional standards, Voting Rights Act, judicial precedents
- **Geographic**: Spatial adjacency, census data, administrative boundaries

Select panels that balance these perspectives based on paper focus.

---

*Last updated: February 2026*
*Total reviewers: 13*
