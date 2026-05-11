# U.18 - Large-Neighborhood Search

**Paper Type:** Heuristic improvement implementation  
**Status:** Round-1 review addressed
**Track:** U - Search and Optimization  
**Code Home:** `bisect-local-search`, `bisect-cli::improve_cmd`  
**CLI Surface:** `bisect improve`; staged `--search lns` and `--search tabu`

## Research Question

How can local-search and future LNS/tabu methods improve existing audited plans
while preserving validity and recording lineage?

## Hypotheses / Claims

- **H1:** A one-move boundary improvement can reduce edge cut while preserving contiguity and population validity on fixtures.
- **H2:** No-improvement and staged-method paths return structured, reproducible statuses.
- **H3:** Improved plans can be audited as descendants of an existing RPLAN input.

Falsification: invalid improved plans, nondeterministic summaries, or missing
lineage linking input and output.

## Scope Boundary

- **In scope:** one-move baseline, summary lineage, RPLAN input/output audit.
- **Out of scope:** full LNS/tabu performance claims until advanced methods are implemented.
- **Generalizability claim:** this establishes the improvement-family contract.

## Evaluation Plan

- Baselines: original plan, no-op local search, future tabu/LNS.
- Evidence: L0 improvement/no-op/staged-method tests and L1 CLI/RPLAN sidecar test.
- Evidence needed: empirical distribution of accepted moves and objective deltas.

## Figures and Tables

- Boundary-move diagram.
- Improvement summary schema.
- Before/after fixture table.

## Limitations

- One-move local search can get stuck in local optima.
- Advanced LNS/tabu methods are scaffolded but not yet full empirical baselines.

## Panel Readiness Checklist

- [x] `main.tex` and sections exist.
- [x] Improvement and no-op cases are both documented.
- [x] Parent/child audit lineage is shown.
- [x] P1 simulated feedback addressed.
