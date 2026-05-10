---
reviewer: ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH
spec: Algorithm Family Roadmap for T/U Refactor
round: 1
date: 2026-05-10
score: 3
---

# Role Review: Algorithm Family Roadmap

## Summary

The roadmap is directionally correct: it separates construction families from
search/optimization families, keeps existing crates where they already have
ownership, and introduces new crates only where solver/runtime boundaries justify
them. The strongest decision is putting shared certification first
(`bisect-constraints` / U.20), because branch-and-cut, clustering repair,
flow construction, local search, and evolutionary search all need the same
validity witnesses.

Round 1 identified several blocking specification gaps. The roadmap has already
been patched for the high-confidence fixes: legal parameterization, data
provenance, output schema versioning, dependency direction, solver callback
distinction, and richer plan/objective interfaces.

Decision: **minor revision before implementation**. The roadmap can guide work,
but each family still needs its own implementation spec before code starts.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3/4 | Legal constraints now parameterized; still needs concrete legal profiles before certification implementation. |
| WARD | 3/4 | State/chamber subdivision rules are acknowledged; needs first-class state legal rule table later. |
| COVENANT | 3/4 | Audit metadata and solver provenance added; needs exact certificate schema in U.20 spec. |
| CONTOUR | 3/4 | GEOID/source vintage requirements added; needs explicit data loader ownership per new crate. |
| MERIDIAN | 3/4 | Algorithm/crate placement is mostly right; spectral and flow need sharper algorithmic definitions in their own specs. |
| BENCHMARK | 3/4 | L0/L1 acceptance criteria present; each family still needs fixture definitions. |
| SCALE | 3/4 | Comparison metrics noted; needs statistical evaluation plan for algorithm comparisons. |
| PRECINCT | 3/4 | Partisan objectives included; needs guardrails against declaring political neutrality from algorithm class alone. |
| DATUM | 3/4 | Scope is honest roadmap/spec, not results; needs falsification/evaluation criteria per paper. |
| COMMONS | 2/4 | Community impact appears only through subdivision/split constraints; needs explicit COI and explainability hooks. |
| LEDGER | 3/4 | Versioned outputs added; needs concrete RPLAN/certificate schema references. |
| SURVEY | 3/4 | CLI surfaces and rollout order are usable; needs runtime/dependency estimates before adoption claims. |
| TRENCH | 3/4 | Failure-mode prevention is improved; needs pitfall entries once implementation begins. |
| **Average** | **3.0/4** | Suitable as a roadmap; not yet an implementation-ready spec for any one family. |

## Consolidated P1 Items

### P1-A: U.20 certificate schema must be specified before any new algorithm crate

Raised by: BOUNDARY, WARD, COVENANT, BENCHMARK, LEDGER, TRENCH

The roadmap correctly puts U.20 first, but implementation must not start with an
open-ended `Certificate` struct. The U.20 spec must define:

- schema version
- plan hash and assignment schema version
- legal profile applied
- source-data hashes
- binary/git/solver provenance
- district-level population and contiguity witnesses
- failure/warning codes
- JSON examples for valid and invalid plans

Without this, later crates will emit incompatible certificates.

### P1-B: Legal profiles cannot be free text

Raised by: BOUNDARY, WARD

The roadmap now says constraints are parameterized by state, chamber, and year.
The implementation spec must define the profile format. Required fields:

- `jurisdiction`
- `chamber`
- `population_tolerance`
- `contiguity_required`
- `county_split_rule`
- `municipal_split_rule`
- `nesting_rule`
- `vra_policy`

Do not hardcode congressional assumptions into the shared validator.

### P1-C: Plan identity must be stable across crates

Raised by: CONTOUR, LEDGER, COVENANT

The patched `DistrictPlan` includes a unit index, which is the right direction.
The U.20 and `bisect-core::plan` specs must define whether assignment order is:

- source file order
- sorted GEOID order
- explicit `unit_ids[i]` order

The recommended answer is explicit `unit_ids[i]` order with a stable canonical
sort for exports. Every crate must reject plans whose assignment length does not
match the unit index.

### P1-D: External solver modes need disclosure and fallback rules

Raised by: COVENANT, SURVEY, MERIDIAN

Branch-and-cut, branch-and-price, and flow construction may use external solvers
or solver features that vary by backend. Each spec must state:

- supported solver backends
- whether lazy callbacks are required
- fallback behavior when unavailable
- how solver version is recorded
- how time limits and optimality gaps affect legal/audit claims

### P1-E: Shared objective functions need missing-data behavior

Raised by: CONTOUR, DATUM, SCALE

The roadmap now says objectives must return structured missing-input errors.
Each objective must define its required inputs and whether it can operate as:

- exact metric
- proxy metric
- unavailable

For example, compactness proxy from graph edge cut is not the same as geometric
Polsby-Popper or Reock; the output must label which one was used.

## P2 Items

### P2-A: Add canonical synthetic fixtures

Raised by: BENCHMARK, TRENCH

Before implementation, define canonical fixtures:

- path graph
- two-clique bridge graph
- 3x3 or 4x4 grid graph
- disconnected invalid plan fixture
- county-boundary toy graph
- impossible population capacity fixture

These should live under a shared test helper so every algorithm crate can reuse
the same ground truth.

### P2-B: Add runtime and dependency budget table

Raised by: SURVEY

The roadmap should eventually include expected runtime class and dependency
burden for each family. This matters for adoption:

- pure Rust
- external solver required
- geometry required
- block-group/tract only
- practical n/k limits

### P2-C: Add comparison/evaluation design

Raised by: SCALE, DATUM, PRECINCT

The roadmap says which families to add but not how they will be compared. The
family papers should share an evaluation matrix:

- compactness metrics
- population deviation
- contiguity failures/repairs
- county/municipal splits
- VRA opportunity districts where relevant
- partisan seat outcomes where relevant
- runtime and determinism
- sensitivity to seeds/parameters

### P2-D: Add community-of-interest hooks

Raised by: COMMONS

The roadmap should connect construction/search methods to Track M community
character weights and explicit COI reporting. Especially for clustering and
regionalization, the spec should distinguish administrative subdivision
preservation from lived community cohesion.

### P2-E: Add pitfall tracking when implementation begins

Raised by: TRENCH

When the first implementation PR starts, add pitfall records for:

- plan assignment order drift
- solver fallback silently changing method
- certificate schema drift
- geometry/projection mismatch
- repair step breaking legal constraints
- objective proxy mislabeled as exact metric

## Role Notes

### BOUNDARY

The roadmap now avoids treating population balance and contiguity as generic
quality metrics. The remaining legal risk is implementation drift: if
`bisect-constraints` defaults to congressional rules but is used for state
legislative plans, it will certify the wrong thing.

### WARD

State constitutional rules must not be collapsed into "split count." The
roadmap correctly calls for jurisdiction/chamber/year profiles, but the profile
schema is now a prerequisite.

### COVENANT

Putting certificate generation first is correct. The certificate must capture
external solver versions and fallback paths. A plan produced by "branch-and-cut"
with lazy callbacks is materially different evidence from a plan produced by
iterative solve-separate-resolve.

### CONTOUR

The unit index addition is necessary. The next spec must pin source vintage and
hash behavior for PL 94-171, TIGER, ACS, election data, and projection files.

### MERIDIAN

The crate allocation is sensible. Spectral partitioning can start in
`bisect-apportion`; clustering and flow deserve crates. Branch-and-price should
not be buried in `--search` because its solver lifecycle is not a search
compositor.

### BENCHMARK

The acceptance criteria are good but not testable enough yet. Each algorithm
family spec must name canonical fixtures and exact assertions.

### SCALE

The roadmap should not let future papers claim "algorithm X is better" without
a comparison design. Each family needs evaluation metrics and uncertainty/scope
rules.

### PRECINCT

Partisan objectives are supported, but the roadmap should avoid implying that
algorithmic neutrality guarantees political neutrality. U.19 and U.14 should
carry that warning.

### DATUM

As a roadmap, the document is appropriately scoped. As soon as it becomes paper
specs, each paper needs falsification criteria: what result would show the
family is not worth adopting?

### COMMONS

The weak spot is communities of interest. Clustering/regionalization naturally
touch COI, but the roadmap currently frames this mostly as subdivision
preservation. Add Track M integration in the family specs.

### LEDGER

Output versioning is now present. The next concrete spec should define RPLAN and
certificate schema versions before new outputs proliferate.

### SURVEY

The CLI split is practical: ordinary construction modes stay in `--structure`,
heavy exact solvers go under `bisect exact`, and improvement starts from
`bisect improve`. Adoption claims still need runtime and dependency budgets.

### TRENCH

The roadmap now prevents several predictable failures structurally. The next
step is to add pitfall records once implementation starts so the failures stay
visible after this review.

