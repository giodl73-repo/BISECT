# U.14 - Multi-Objective Selection

**Paper Type:** Multi-objective methodology  
**Status:** Planning  
**Track:** U - Search and Optimization  
**Code Home:** `bisect-pareto`, `bisect-smc`, `bisect-analysis`

## Research Question

How should practitioners choose among valid plans when objectives conflict and
the output is a frontier or weighted sample rather than one canonical plan?

## Hypotheses / Claims

- **H1:** Separating frontier generation from selected-plan audit makes multi-objective workflows reproducible.
- **H2:** Objective trade-offs should be reported as choices, not hidden inside a weighted scalar objective.
- **H3:** Selected plans can be audited without implying the whole frontier is complete or globally optimal.

Falsification: selected-plan packaging that cannot identify the frontier entry
or objective vector.

## Scope Boundary

- **In scope:** Pareto/SMC selection framing, objective vectors, selected plan audit.
- **Out of scope:** proving true Pareto completeness for large redistricting instances.
- **Generalizability claim:** selection discipline applies to BISECT frontier/sample workflows.

## Evaluation Plan

- Baselines: single weighted-sum objective, un-audited frontier export.
- Evidence: `bisect pareto` NDJSON, selected-frontier RPLAN package, objective metadata.
- Success criteria: every selected plan carries index, objectives, context, and audit certificate.

## Figures and Tables

- Objective-space frontier diagram.
- Selected-plan packaging flow.
- Trade-off disclosure table.

## Limitations

- Frontier quality depends on search budget.
- Objective choice is normative and must be disclosed.

## Panel Readiness Checklist

- [ ] `main.tex` and sections exist.
- [ ] Pareto and SMC roles are distinguished.
- [ ] Selected-plan audit limitations are explicit.
- [ ] P1 simulated feedback addressed.
