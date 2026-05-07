# Plan: New Algorithm Papers — B.25 Moving-Knife + B.26 Multi-Objective Pareto

**Date**: 2026-05-07  
**Status**: Draft — pending review  
**Context**: After surveying the redistricting software landscape, two genuinely novel algorithmic approaches were identified that are not yet in our portfolio and represent different paradigms from our current work.

---

## Algorithm 1: Moving-Knife Algorithm (B.25)

### Background

Puppe & Tasnadi (2026, *Public Choice*) adapt the fair-division "moving-knife" protocol to redistricting. Instead of minimising edge cuts (METIS) or geographic distance (CVD), MKA sweeps a line across the geographic space to maximise Reock compactness — the legally recognised minimum-enclosing-circle measure.

### Key insight

MKA answers a different question than our current algorithms:
- METIS/SA: "minimise boundary length" (edge-cut compactness)
- CVD: "minimise geographic distance to seeds" (proximity compactness)
- **MKA: "maximise roundness" (Reock compactness)** — the most defensible in court

### Integration with AreaSection (B.9)

AreaSection already has a "direction" concept (ratio-optimal bisection orientation). MKA provides the Reock-optimal direction geometrically; AreaSection then enforces population + area balance along that axis. The hybrid MKA-AreaSection addresses two objectives simultaneously:
- MKA: natural geographic compactness axis
- AreaSection: dual population + area balance

### Task structure

```
#160 Spec: B.25 Moving-Knife → panel review ≥3.0/4
    ↓
#161 Implement SplitStrategy::MovingKnife
     - split_subgraph_mka(): sweep n_orientations, Welzl Reock, post-hoc rebalance
     - Requires: tract_centroids (already in LoadedGraph from CVD Phase 2)
     - CLI: --structure moving-knife --mka-orientations 180 --mka-metric reock
    ↓
#162 MKA-AreaSection hybrid
     - split_subgraph_mka_direction(): returns optimal angle θ*
     - AreaSection uses θ* as warm-start for ratio direction
     - CLI: --structure area-section --area-section-init moving-knife
    ↓
#163 Write + panel-review B.25 paper
     - Cites Puppe & Tasnadi (2026) as theoretical foundation
     - Compare MKA vs METIS vs CVD-Geo vs SA on Reock for NC/FL/WA
     - MKA-AreaSection hybrid beats both standalone methods
```

### Implementation notes

**Welzl's algorithm** for minimum enclosing circle: O(n) expected time. Each orientation evaluation is O(n) for projection + sort + Welzl. Total: O(n_orientations × n) per bisection node.

**n_orientations=180** (every 1°): covers the full half-circle, deterministic. For production speed, 36 orientations (every 5°) is likely sufficient.

**Population balance**: geometric sweep won't produce 50/50 population splits. Same 200-iter boundary-swap rebalance as CVD and BFS.

**Reock vs Polsby-Popper**: Reock = district_area / minimum_enclosing_circle_area. Polsby-Popper = 4πA/P². MKA optimises Reock directly; PP is reported for comparison. `--mka-metric [reock|polsby]` selects which metric the sweep maximises.

---

## Algorithm 2: Multi-Objective Pareto Redistricting (B.26)

### Background

All current redistricting optimisation uses **weighted-sum objectives**: minimise w1×EC + w2×partisan_deviation + w3×VRA_deficit. The weight choice is opaque and arbitrary — practitioners tune weights until they get a plan they like, which is legally defensible only if the weights are pre-committed.

**Pareto-front redistricting** is honest about trade-offs: it produces a *frontier* of plans where no plan is better on ALL objectives simultaneously. Practitioners then choose from the frontier with full transparency about what each choice costs.

### NSGA-II for redistricting

Non-dominated Sorting Genetic Algorithm II (Deb et al. 2002), adapted:
- **Population**: N redistricting plans (Vec<u32> assignments)
- **Objectives**: EC (compactness), D-seats (partisan), minority_vap_deficit (VRA)
- **Non-dominated sort**: rank plans by Pareto dominance, keep non-dominated front
- **Crossover**: merge two parent plans by swapping a connected subregion (like ReCom)
- **Mutation**: single boundary-tract flip (reuse Flip chain logic)

### Legal value

The Pareto frontier enables a powerful legal argument: "the enacted plan is Pareto-dominated — there exists a plan that is better on ALL redistricting criteria simultaneously." If true, this is direct evidence that the enacted plan is sub-optimal by the legislature's own stated criteria.

### SMC connection

SMC already produces a weighted sample from the uniform distribution over plans. Projecting SMC particles into objective space (EC, D-seats, VRA) gives an approximate Pareto frontier without running a GA. This "SMC-Pareto" is potentially faster and better-calibrated. B.26 compares NSGA-II vs SMC-Pareto.

### Task structure

```
#164 Spec: B.26 Multi-Objective Pareto → panel review ≥3.0/4
     - Design decision: NSGA-II full GA vs SMC-Pareto projection
     - Three objectives: EC, partisan, VRA
     - Output format: NDJSON Pareto frontier
    ↓
#165 Implement redist-pareto (→ bisect-pareto) crate
     - NSGA-II with ReCom-style crossover + Flip-style mutation
     - Rayon parallelism for objective evaluation
     - CLI: bisect pareto --state NC --year 2020 --population 100 --generations 200
    ↓
#166 Write + panel-review B.26 paper
     - Compare: NSGA-II vs SMC-Pareto vs ILP-only vs VRA-only
     - Key result: enacted plan Pareto-dominance test
     - Legal framing: "no plan can be better on all criteria simultaneously"
```

### Implementation notes

**NSGA-II crossover for redistricting**: take a connected subregion from parent plan A, keep the rest from parent plan B. Unlike ReCom which uses spanning trees for bisection, Pareto crossover needs to handle arbitrary k-district plans, making contiguity harder to guarantee. Options:
1. Accept invalid plans and repair via BFS rebalance (simplest)
2. Use spanning-tree crossover (complex but contiguity-safe)
3. Restrict crossover to single-district swaps (safe but low diversity)

**SMC-Pareto alternative**: after running SMC for NC (N=5000 particles), project each particle onto (EC, D-seats, minority_VAP) objective space. The non-dominated front of this projection is an approximate Pareto frontier without a GA. Much faster but less customizable — the objectives must be computable from a plan and minority data, which they are.

**New crate `bisect-pareto`**: separate from `bisect-smc` because the algorithmic paradigm (GA population evolution) is different from SMC (sequential importance resampling). They can call common utilities (plan evaluation, NDJSON output).

---

## Combined review checklist

### B.25 Moving-Knife

- [ ] Spec: geometric sweep definition on discrete tract graph is precise
- [ ] Spec: Reock = district_area / MEC_area correctly defined
- [ ] Spec: Welzl's algorithm cited (Welzl 1991)
- [ ] Spec: connection to AreaSection direction parameter clearly specified
- [ ] Spec: n_orientations default (180) and computational cost stated
- [ ] Implementation: tract_centroids dependency on CVD Phase 2 documented
- [ ] Implementation: MKA-AreaSection hybrid correctly propagates θ*
- [ ] Paper: Puppe & Tasnadi 2026 cited correctly
- [ ] Paper: legal defensibility of Reock maximisation discussed

### B.26 Multi-Objective Pareto

- [ ] Spec: NSGA-II vs SMC-Pareto design decision resolved
- [ ] Spec: three objectives defined (EC, D-seats, VRA_deficit) with formulas
- [ ] Spec: crossover validity guarantee specified
- [ ] Spec: output NDJSON format compatible with existing bisect NDJSON conventions
- [ ] Spec: "Pareto dominance as gerrymandering evidence" framing legally reviewed
- [ ] Implementation: crate boundary between bisect-pareto and bisect-smc clear
- [ ] Paper: SMC-Pareto comparison included
- [ ] Paper: enacted-plan dominance test framing reviewed by Stephanopoulos role

---

## Dependencies on existing work

| Task | Depends on |
|------|-----------|
| #161 MKA implement | LoadedGraph.tract_centroids (#150 ✓), albers_project() (#150 ✓) |
| #162 MKA-AreaSection | AreaSection (B.9 ✓), MKA direction output (#161) |
| #165 Pareto crate | redist-smc/bisect-smc (#157 ✓), Flip chain (G.8 ✓) |
| B.25 paper (#163) | #161 ✓, #162 ✓ |
| B.26 paper (#166) | #165 ✓ |

---

## Open questions for review

**B.25 MKA:**
1. Should MKA be a pure Structure layer (stand-alone SplitStrategy) or always used as an AreaSection initialiser? Or both as separate modes?
2. For the hybrid MKA-AreaSection: does the AreaSection spec need to be updated to accept an external direction parameter? (Currently ratio is computed internally)
3. Is Reock the right metric to maximise, or should MKA maximise PP? Reock is easier (Welzl), PP requires perimeter computation.
4. How does MKA handle non-convex subgraphs where no straight-line sweep cleanly separates tracts (e.g., a donut-shaped region)?

**B.26 Pareto:**
1. Should the Pareto output be integrated into the `bisect ensemble` command (alongside SMC) or be a new `bisect pareto` subcommand?
2. How many generations/population size is needed for NC k=14 to produce a meaningful frontier? (Estimate: 200 generations × 100 population = 20,000 objective evaluations × NC ~2200 tracts ≈ 30 minutes)
3. Should D-seats be discrete (integer seat count) or continuous (expected partisan seat share)? Discrete creates Pareto ties; continuous is smoother but harder to interpret.
4. Should VRA be a constraint (all plans must satisfy VRA — equivalent to running VRA-Aware MCMC as the mutation operator) or an objective (plans may violate VRA but are ranked lower)? Constraint is legally safer but limits frontier diversity.
