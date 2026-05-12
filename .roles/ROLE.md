# REDIST Review Roles

Eight perspectives on congressional redistricting, named after cartographic elements.
Each role has a pointed view and pulls against at least one other.

## The Sixteen Roles

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
CANVASS    Election Certifier        ─── Canvass workflow, certification status, recount/adjudication lineage
TALLY      Voting Systems Engineer   ─── CVRs, ballot manifests, batch accounting, vendor export semantics
VAULT      Crypto & Privacy Reviewer ─── Ballot secrecy, inclusion proofs, canonical hashes, threat models
```

### New roles added 2026-04-26 (practitioner toolkit expansion)

**WARD** fills the gap between federal constitutional law (BOUNDARY) and state-specific redistricting requirements — balance tolerance by chamber type, county preservation clauses that vary by state constitution, nesting ratios.

**COVENANT** fills the gap between methodology rigor (DATUM) and legal evidence admissibility — what a special master actually needs, binary provenance, chain of custody for computational plans.

**LEDGER** fills the gap in format standards and ecosystem compatibility — GeoJSON RFC 7946 conformance, GerryChain schema versions, RPLAN design, Census TIGER naming conventions.

### New roles added 2026-05-12 (RCOUNT expansion)

**CANVASS** fills the gap between legal theory and election-office practice:
unofficial returns, canvassed totals, recounts, amended certifications,
provisional adjudication, cure periods, and canvassing-board decisions.

**TALLY** fills the gap between file interop and voting-system semantics:
CVRs, ballot manifests, scanner batches, contest definitions, overvotes,
undervotes, write-ins, duplicated ballots, and vendor export meaning.

**VAULT** fills the gap between public verification and ballot secrecy:
domain-separated hashes, canonical commitments, inclusion proofs, coercion
resistance, small-cell privacy, and explicit security non-goals.

## Tiebreaker Ranking

When roles conflict, earlier roles govern:

1. **BOUNDARY**  — legal invalidity stops everything
2. **WARD**      — state constitutional violations stop everything (jurisdiction-specific)
3. **COVENANT**  — chain of custody failure makes evidence inadmissible
4. **CANVASS**   — election certification status is a legal/workflow fact, not computed by wish
5. **VAULT**     — ballot secrecy and coercion resistance are hard stops
6. **CONTOUR**   — bad data means bad results
7. **MERIDIAN**  — algorithm correctness is the foundation
8. **TALLY**     — voting-system semantics must survive normalization
9. **BENCHMARK** — if we can't verify it, we can't trust it
10. **SCALE**    — invalid claims cannot be published
11. **PRECINCT** — political implications matter but don't override correctness
12. **DATUM**    — publication quality is a gate, not a veto
13. **COMMONS**  — community voice informs but doesn't override
14. **LEDGER**   — format incompatibility silently breaks practitioner workflows
15. **SURVEY**   — operational feasibility is last
16. **TRENCH**   — pitfall collection grows every session; structural prevention is the standard

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
| CANVASS | COVENANT + LEDGER | lawful election records evolve; a clean hash is not a canvass |
| VAULT | COMMONS + COVENANT | transparency cannot become a vote-choice receipt |
| TALLY | LEDGER + BENCHMARK | format compatibility must preserve voting-system semantics |

## Usage

Invoke any role by name when reviewing papers, dashboards, pipeline outputs, or claims.
Each role file contains its orientation, lens questions, and domains.
