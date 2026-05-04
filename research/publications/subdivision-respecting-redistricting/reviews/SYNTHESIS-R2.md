# Round 2 Synthesis — B.10 Subdivision-Respecting Redistricting

**Round**: 2
**Avg score**: 2.6 / 4
(Ullman 2.5, Liang 2.5, Polikarpova 3.0, Steinhardt 2.5, Zhang 2.5)
**Decision**: Conditional accept — Round 3 required with two blocking items.

---

## Progress from Round 1

All five Round 1 P1 items are resolved:
- **P1-A**: Monotonicity Proposition correctly scoped to exact solver (✓)
- **P1-B**: 25-seed ensembles for 7 focal states, Wilcoxon tests, p-values (✓)
- **P1-C**: All 44 states have EC in Table 1 (✓)
- **P1-D**: Partisan section uses ensemble data; GA anomaly explained (✓)

P2-A/B/C also resolved. The paper has moved from avg 1.5 to avg 2.6 — significant progress.

---

## New P1 Items (blocking for acceptance)

### P1-R2-1: Produce Pareto frontier figure
**Raised by**: Ullman, Liang, Steinhardt, Zhang (4/5 reviewers)

The data exists in tabular form (α∈{0,0.5,1,2,5,10,20} for 5 states). A figure showing the splits(α) vs. EC(α) curve for GA, NC, TX simultaneously would:
- Show the knee shape is consistent across states (or identify exceptions)
- Demonstrate that α=5 is a stable recommendation across geographies
- Satisfy the visual intuition that a table cannot convey

**Resolution**: Generate a Python script reading `b10_multiseed.csv`, plot splits vs. EC for 3 focal states, annotate the α=5 knee. Save to `figures/pareto_frontier.pdf`.

### P1-R2-2: Restrict multi-level claims or validate
**Raised by**: Ullman, Polikarpova, Steinhardt (3/5 reviewers)

The paper describes an algorithm over 4 levels (county, MCD, place, VTD) but experiments validate only county-level α. A triple-boundary tract accumulates weight up to (1 + α_county + α_mcd + α_place), which at α=5 per level is 16× base weight — potentially overwhelming any geographic signal.

**Resolution options (choose one)**:
A. Restrict all claims and the algorithm description to county-level α. Add explicit "Multi-level extension" subsection with the formula and a note that empirical validation is future work.
B. Run a single state (GA or NC) with α_county=α_mcd=1 and show the composition produces the expected behaviour (fewer both-county-and-MCD splits, no population balance failure).

Option A is faster and cleaner.

---

## P2 Items for Round 3 (non-blocking)

- **P2-R2-1**: Report SD alongside ranges in Table 2 (Steinhardt)
- **P2-R2-2**: Normalize EC by α=0 baseline for cross-state comparability (Steinhardt)
- **P2-R2-3**: Extend ensemble to more states, or explicitly bound the 7-state generalizability claim (Liang)
- **P2-R2-4**: Discuss VRA interaction — compositional safety with D.0 VRA mode (Zhang)

---

## Path to Acceptance

P1-R2-1 (figure): 1 hour — data exists, just needs plotting.
P1-R2-2 (multi-level scope): 30 minutes — restrict claims in §3.3 and §6.

After resolving both: expected score 3.0–3.2 → accept.
