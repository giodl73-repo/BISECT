# Role Review: Shared Math, Statistics, And Graph Kernel Crates

reviewer: ROLE PANEL
date: 2026-05-13
scope: `docs/specs/2026-05-13-shared-math-graph-kernels.md`
roles: MERIDIAN, SCALE, BENCHMARK, LEDGER, COVENANT, TRENCH, COMMONS,
ROUTE-OPTIMIZATION-METHODOLOGIST, ROUTE-TRAFFIC-ENGINEER, ROUTE-SCOPE-KEEPER,
ROUTE-NUMERACY-CHECKER

## Verdict

Approved for a wave, with a narrow first slice.

The right first move is `rgraph-core`, not a broad math/statistics monorepo.
Both BISECT and ROUTE already contain graph-heavy logic, and ROUTE has concrete
local implementations of weighted Dijkstra and Brandes edge betweenness that are
good extraction candidates. `rstat-core` is also justified, but it should follow
after the graph API proves the shared-crate pattern.

## Role Scores

| Role | Score | Finding |
|---|---:|---|
| MERIDIAN | 3.6/4 | Correctly centers graph correctness, tie handling, and deterministic traversal. Requires tiny synthetic graph fixtures before any domain migration. |
| SCALE | 3.2/4 | `rstat-core` boundary is useful and non-certifying. Needs explicit quantile/interpolation and uncertainty conventions when that wave starts. |
| BENCHMARK | 3.5/4 | Acceptance criteria require positive and negative graph tests. Add regression tests before replacing route or BISECT local code. |
| LEDGER | 3.1/4 | Boundary between algorithm crate and package formats is clear. Needs versioned API notes once crates become external dependencies. |
| COVENANT | 3.0/4 | Determinism and no hidden RNG are good evidence rules. Keep transcript/hash ownership in domain crates. |
| TRENCH | 3.4/4 | Avoids monolith and avoids premature extraction of all algorithms. Main risk is half-migrated duplicate implementations drifting apart. |
| COMMONS | 2.8/4 | General kernels do not encode public-interest outcomes. Domain crates must still own civic interpretation and harm analysis. |
| ROUTE Optimization Methodologist | 3.6/4 | Strong separation of objective/constraint domain logic from reusable graph primitives. Requires rejected-path/counterfactual support only in domain layers. |
| ROUTE Traffic Engineer | 3.1/4 | Shared shortest-path and redundancy kernels are useful, but route-specific capacity and operational constraints must stay in ROUTE. |
| ROUTE Scope Keeper | 3.7/4 | Spec stays within algorithm infrastructure. It does not drift into corridor design or redistricting package semantics. |
| ROUTE Numeracy Checker | 3.0/4 | Unit and weight policy must be explicit: graph weights are abstract costs; miles, travel time, and population penalties cannot be silently mixed. |

Overall: 36.0 / 44. Approved as an implementation-planning spec.

## Required Follow-Ups

### F1: Start With `rgraph-core`

Do not begin with `rstat-core`, `rmath-core`, and `ropt-core` all at once. The
first wave should build the smallest graph crate that can replace one real
duplicated implementation.

### F2: Define Weight Semantics

Every graph API must state whether edge weights are distances, costs, capacities,
or abstract non-negative weights. Negative and non-finite weights must have a
typed error path or explicit skip policy.

### F3: Preserve Domain Ownership

Route scoring, redistricting legality, RCOUNT audit semantics, and package
hashing must stay out of shared algorithm crates. The shared crates compute
generic values; domain crates interpret them.

### F4: Add Migration Tests Before Removing Local Code

Before replacing ROUTE's local Brandes/Dijkstra routines or BISECT connectivity
helpers, add compatibility tests that prove the new crate reproduces current
synthetic expectations.

### F5: Defer Standalone Repository Until Two Consumers Exist

Incubating inside `apportionment` is acceptable. Moving to a standalone shared
repository should wait until one BISECT consumer and one ROUTE consumer compile
against the same API.

## Wave Recommendation

Create a wave with pulses:

1. `rgraph-core` crate skeleton, trait boundary, typed errors, and shortest-path
   tests.
2. Brandes edge-betweenness implementation with route synthetic compatibility
   fixtures.
3. BISECT graph-only consumer or diagnostic helper using `rgraph-core`.
4. ROUTE adapter/consumer branch or documented blocker if cross-repo dependency
   policy needs a separate step.
5. Documentation update and extraction decision: keep incubated, publish shared
   repo, or continue local wrappers.

`rstat-core` should become its own later wave after the graph extraction proves
the pattern.

