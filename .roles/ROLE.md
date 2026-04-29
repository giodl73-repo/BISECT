# REDIST Review Roles

Eight perspectives on congressional redistricting, named after cartographic elements.
Each role has a pointed view and pulls against at least one other.

## The Thirteen Roles

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
WARD       Subdivision Law Expert    ─── State constitutional preservation, county/municipal law by state
COVENANT   Audit & Evidence Expert   ─── Chain of custody, binary provenance, expert witness standards
LEDGER     Standards & Interop       ─── File format standards, GeoJSON/RPLAN/GerryChain compatibility
```

### New roles added 2026-04-26 (practitioner toolkit expansion)

**WARD** fills the gap between federal constitutional law (BOUNDARY) and state-specific redistricting requirements — balance tolerance by chamber type, county preservation clauses that vary by state constitution, nesting ratios.

**COVENANT** fills the gap between methodology rigor (DATUM) and legal evidence admissibility — what a special master actually needs, binary provenance, chain of custody for computational plans.

**LEDGER** fills the gap in format standards and ecosystem compatibility — GeoJSON RFC 7946 conformance, GerryChain schema versions, RPLAN design, Census TIGER naming conventions.

## Tiebreaker Ranking

When roles conflict, earlier roles govern:

1. **BOUNDARY**  — legal invalidity stops everything
2. **WARD**      — state constitutional violations stop everything (jurisdiction-specific)
3. **COVENANT**  — chain of custody failure makes evidence inadmissible
4. **CONTOUR**   — bad data means bad results
5. **MERIDIAN**  — algorithm correctness is the foundation
6. **BENCHMARK** — if we can't verify it, we can't trust it
7. **SCALE**     — invalid claims cannot be published
8. **PRECINCT**  — political implications matter but don't override correctness
9. **DATUM**     — publication quality is a gate, not a veto
10. **COMMONS**  — community voice informs but doesn't override
11. **LEDGER**   — format incompatibility silently breaks practitioner workflows
12. **SURVEY**   — operational feasibility is last
13. **TRENCH**   — pitfall collection grows every session; structural prevention is the standard

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
