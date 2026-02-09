# Apportionment Research Papers

**Module**: Congressional redistricting via METIS recursive bisection
**Papers**: 2
**Author**: Giovanni De Luca

---

## Paper Inventory

| # | Directory | Title | PDF | Venue Target | Score | Tier |
|---|-----------|-------|-----|-------------|-------|------|
| 1 | [00+synthesis-metapaper](00+synthesis-metapaper/) | Algorithmic Objectivity for Congressional Redistricting: A National-Scale Demonstration | [PDF](docs/00+synthesis-metapaper.pdf) | Science | — | — |
| 2 | [13+national-redistricting](13+national-redistricting/) | National Redistricting Without State Boundaries: Quantifying the Geometric Cost of Federalism | [PDF](docs/13+national-redistricting.pdf) | Comparative Political Studies | — | — |

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
*Papers: 2*
