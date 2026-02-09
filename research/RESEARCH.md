# Apportionment Research Papers

**Module**: Congressional redistricting via METIS recursive bisection
**Papers**: 6
**Author**: Giovanni De Luca

---

## Paper Inventory

| # | Directory | Title | PDF | Venue Target | Score | Tier |
|---|-----------|-------|-----|-------------|-------|------|
| 1 | [00+synthesis-metapaper](00+synthesis-metapaper/) | Algorithmic Objectivity for Congressional Redistricting: A National-Scale Demonstration | [PDF](docs/00+synthesis-metapaper.pdf) | Science | — | — |
| 2 | [13+national-redistricting](13+national-redistricting/) | National Redistricting Without State Boundaries: Quantifying the Geometric Cost of Federalism | [PDF](docs/13+national-redistricting.pdf) | Comparative Political Studies | — | — |
| 3 | [11+maup-sensitivity](11+maup-sensitivity/) | Spatial Resolution and Algorithmic Redistricting: A Test of the Modifiable Areal Unit Problem | [PDF](docs/11+maup-sensitivity.pdf) | IJGIS | — | — |
| 4 | [12+longitudinal-analysis](12+longitudinal-analysis/) | Twenty Years of Congressional Redistricting: A Longitudinal Analysis of Algorithmic Redistricting Across Three Census Decades | [PDF](docs/12+longitudinal-analysis.pdf) | Science | — | — |
| 5 | [17+multi-member-districts](17+multi-member-districts/) | Multi-Member Districts and Proportional Representation: Exploring Alternatives to Single-Member Redistricting | [PDF](docs/17+multi-member-districts.pdf) | Electoral Studies | — | — |
| 6 | [15+party-based-allocation](15+party-based-allocation/) | Partisan Fairness through Algorithmic Districting: An Alternative to Proportional Representation | [PDF](docs/15+party-based-allocation.pdf) | Electoral Studies | — | — |

---

## Paper Dependency Graph

```
00+synthesis-metapaper (Science)
├─ Synthesizes all 10 papers
└─ Targets highest-impact interdisciplinary venue
```

---

## Review Status

| Paper | Stage | Round | Reviewers | Score | Verdict |
|-------|-------|-------|-----------|-------|---------|

---

## Build

```bash
make all          # Build all papers
make dist         # Copy PDFs to docs/
make clean        # Remove build artifacts

# Single paper
make -C <paper-dir> pdf
```

---

*Apportionment research module — established February 2026*
*Papers: 6*
