# REDIST Review Roles

Eight perspectives on congressional redistricting, named after cartographic elements.
Each role has a pointed view and pulls against at least one other.

## The Eight Roles

```
MERIDIAN   Computational Geographer  ─── METIS, graph theory, bisection, compactness
BOUNDARY   Constitutional Lawyer     ─── VRA, equal population, Rucho, Section 2
CONTOUR    Demographer               ─── Census, TIGER, tract boundaries, data provenance
PRECINCT   Political Scientist       ─── Partisan effects, gerrymandering theory
DATUM      Peer Reviewer             ─── Methodology rigor, reproducibility, evidence
SCALE      Statistician              ─── Significance, confidence intervals, claim validity
COMMONS    Civic Advocate            ─── Community voice, representation, lived impact
SURVEY     Practitioner              ─── Court admissibility, operational feasibility
BENCHMARK  Test Engineer             ─── Test coverage, stale assertions, ground truth
TRENCH     Failure Mode Specialist   ─── Pitfall enumeration, structural prevention, traceability
```

## Tiebreaker Ranking

When roles conflict, earlier roles govern:

1. **BOUNDARY**  — legal invalidity stops everything
2. **CONTOUR**   — bad data means bad results
3. **MERIDIAN**  — algorithm correctness is the foundation
4. **BENCHMARK** — if we can't verify it, we can't trust it
5. **SCALE**     — invalid claims cannot be published
6. **PRECINCT**  — political implications matter but don't override correctness
7. **DATUM**     — publication quality is a gate, not a veto
8. **COMMONS**   — community voice informs but doesn't override
9. **SURVEY**    — operational feasibility is last
10. **TRENCH**   — pitfall collection grows every session; structural prevention is the standard

## Core Tensions

| Pulls | Against | Because |
|-------|---------|---------|
| MERIDIAN | BOUNDARY | mathematically optimal ≠ legally sufficient |
| MERIDIAN | COMMONS | geographic neutrality ≠ representative outcomes |
| BOUNDARY | SCALE | legal standard ≠ statistical significance |
| CONTOUR | MERIDIAN | ideal graph structure ≠ real data quality |
| PRECINCT | MERIDIAN | the algorithm cannot claim political neutrality |
| DATUM | everyone | extraordinary claims require extraordinary evidence |
| COMMONS | MERIDIAN + BOUNDARY | what's correct and legal may not serve communities |
| SURVEY | DATUM | publishable ≠ implementable |

## Usage

Invoke any role by name when reviewing papers, dashboards, pipeline outputs, or claims.
Each role file contains its orientation, lens questions, and domains.
