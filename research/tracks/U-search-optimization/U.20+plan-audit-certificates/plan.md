# U.20 - Plan Audit Certificates

**Paper Type:** Audit/certification infrastructure  
**Status:** Round-1 review addressed
**Track:** U - Search and Optimization / audit  
**Code Home:** `rplan-core`, `rplan-io`, `rplan-audit`, `bisect verify`, `rplan-cli`

## Research Question

Can BISECT use a neutral plan/context/audit artifact family as the fixed point
for heterogeneous redistricting algorithms?

## Hypotheses / Claims

- **H1:** RPLAN and RCTX separate plan identity from graph/population context while preserving canonical unit order.
- **H2:** Audit certificates can verify declared constraints and algorithm lineage without depending on a specific solver.
- **H3:** Final-plan workflows across construction, exact, local-search, and Pareto families can converge into the same verify/report path.

Falsification: algorithm-specific certificate fields leaking into generic plan
identity, unverifiable certificates, or inconsistent unit-order semantics.

## Scope Boundary

- **In scope:** RPLAN/RCTX schema, legal profiles, audit certificates, lineage, verifier integration.
- **Out of scope:** full legal compliance determination, political fairness proof, geometry-heavy certification beyond current profiles.
- **Generalizability claim:** RPLAN is designed as a tool-neutral interchange artifact with BISECT adapters.

## Evaluation Plan

- Baselines: ad hoc plan JSON, solver-local manifests, final assignments only.
- Evidence: rplan-core/io/audit tests, bisect verify tests, final-plan sidecars across algorithm families.
- Evidence needed: public examples and tamper/failure-mode catalog.

## Figures and Tables

- RPLAN fixed-point pipeline.
- Certificate schema anatomy.
- Algorithm-lineage consumer table.

## Limitations

- A certificate verifies a profile, not all law.
- Some constraints require data not present in every RCTX.
- External tools must preserve canonical unit ordering.

## Panel Readiness Checklist

- [x] `main.tex` and sections exist.
- [x] RPLAN/RCTX distinction is clear.
- [x] Audit semantics and non-semantics are explicit.
- [x] P1 simulated feedback addressed.
