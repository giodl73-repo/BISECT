# Spec: PercentileSweep — Targeting Specific Positions in the Redistricting Feasible Space

**Status**: Proposed  
**Date**: 2026-05-06  
**Depends on**: B.16 (ConvergenceSweep), redist-ensemble (ReCom crate, parallel spec)  
**Motivation**: ConvergenceSweep finds the minimum-edge-cut plan (compactness extremum, 0.1–0.7th percentile of ReCom ensemble for most states). PercentileSweep generalises this to target any percentile — enabling statutory choice of legal posture.

---

## The problem with the extremum

ConvergenceSweep produces plans at the **compactness extremum** of the feasible space: more compact than 99–100% of valid plans in most states (WI: 0.2th pct, GA: 0.1th pct, PA: 0.2th pct). This is legally defensible ("no neutral plan is more compact") but:

1. Compactness correlates with Republican-lean in geographically sorted states (the Rodden effect). Being at the 0.1th percentile may inadvertently produce a partisan tilt as a side effect of geometric neutrality.
2. Some legal standards — particularly state constitutional "representative maps" requirements — implicitly expect plans near the 50th percentile of the feasible distribution.
3. A statutory algorithm that always produces extremum plans may be challenged as "over-optimizing" for a particular geometric criterion.

PercentileSweep gives the legislature or commission a knob: **choose your legal posture**.

---

## Algorithm design

### Core idea: sort T plans by edge cut, pick rank p·T

The SHA-256 seed formula from B.16 provides a deterministic sequence of T seeds. Instead of picking the minimum-EC plan, pick the plan at a specified rank:

```
PercentileSweep(p, T):
  seeds = [SHA-256(census_release_id || "DIA_SEED_V1" || i) for i in 0..T]
  plans = [bisect(seed) for seed in seeds]
  sorted_plans = sort(plans, key=edge_cut)
  return sorted_plans[floor(p * T)]
```

**Special cases:**
- p=0.0, T=600: equivalent to ConvergenceSweep (minimum EC)
- p=0.5, T=101: MedianSweep (median within bisection seed space)
- p=1.0, T=101: MaximumSweep (maximum EC = least compact valid plan)

### Relationship between bisection space and ReCom ensemble space

The bisection seed space and the ReCom ensemble space are **different distributions**:

| State | Bisection min EC | Bisection median EC* | ReCom median EC |
|-------|-----------------|---------------------|-----------------|
| NC | 0.0973 | ~0.097 | 0.0970 |
| WI | 0.0651 | ~0.075 | 0.0866 |
| GA | 0.0854 | ~0.095 | 0.1017 |
| PA | 0.0857 | ~0.093 | 0.1005 |

*Estimated from B.7 seed variance analysis (CV < 2% implies narrow bisection distribution)

NC is the exception: bisection space and ReCom space converge (geographic constraint dominates). For WI/GA/PA, the bisection algorithm consistently finds plans well below the ReCom ensemble median.

**Implication**: `MedianSweep(T=101)` does NOT produce a plan at the 50th percentile of the ReCom ensemble. It produces a plan at the median of the bisection seed space — still well below the ReCom median for most states.

### TargetedSweep: calibrated to the ReCom ensemble

To genuinely target the ReCom ensemble's p-th percentile:

```
Calibration (one-time per state):
  EC_target(p) = p-th percentile of ReCom ensemble (from redist-ensemble)

TargetedSweep(p, T):
  plans = [bisect(seed_i) for i in 0..T]
  return argmin_{plan} |EC(plan) - EC_target(p)|
```

**Statutory implementation**: The DIA would specify EC_target values (published by the Census Bureau from a reference ensemble run), and TargetedSweep would select the bisection plan closest to the target. This requires a one-time ensemble calibration step (computationally trivial with Rust ReCom).

---

## CLI surface

```bash
# MedianSweep: 101 seeds, pick median EC plan
redist state --state WI --search percentile --percentile 0.50 --seeds 101

# TargetedSweep: calibrated to ReCom 50th percentile
redist state --state WI --search targeted \
  --target-ec 0.0866 \        # from redist-ensemble calibration
  --seeds 200

# The full workflow:
redist ensemble --state WI --steps 5000 --chains 4 \
  --output wi_calibration.json   # Step 1: calibrate

redist state --state WI --search targeted \
  --calibration wi_calibration.json \
  --percentile 0.50 \
  --seeds 200                    # Step 2: targeted plan
```

YAML config:

```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: percentile       # or: targeted
  percentile: 0.50         # target the 50th percentile within bisection space
  seeds: 101
```

---

## New research questions this opens

### For the G series papers:
- **G.2 revision**: Do partisan outcomes track the bisection percentile? If WI at ConvergenceSweep is 4D/4R (proportional) and at MedianSweep is still 4D/4R, this confirms that compactness percentile doesn't drive partisan outcomes.
- **G.3 revision**: What is the compactness-partisan correlation WITHIN the bisection seed space (not just the ReCom ensemble)?

### New paper: H-series?
**H.0 · PercentileSweep: Statutory Choice of Legal Posture in Algorithmic Redistricting**
- Empirical: run ConvergenceSweep, MedianSweep, and TargetedSweep(50th) for all 6 key states
- Show: partisan outcomes are largely INSENSITIVE to percentile choice (the Rodden sorting effect dominates, not the cut level)
- Legal: map percentile choice to legal doctrine (extremum → compactness doctrine; 50th → representativeness doctrine; targeted → proportionality doctrine)
- Recommendation: DIA should specify TargetedSweep(50th) to avoid the "over-optimizing" critique while remaining fully deterministic

### Connection to B.17 (Parameter Sensitivity):
B.17 shows partisanship is insensitive to parameter tuning. PercentileSweep suggests the same about the SEARCH percentile — the key determinant of partisan outcomes is STRUCTURE (bisection tree), not which plan in the EC distribution you pick. This should be tested.

---

## Bisection-Level Ensemble Integration

### The key insight

PercentileSweep (above) sorts T independent bisection plans by edge cut and picks rank ⌊p·T⌋. But this operates at the level of the **full k-way plan**. A more powerful approach applies the percentile choice at **each node of the bisection tree** — every bisection step is a k=2 problem, and at each step we can run a local T-step ReCom ensemble and pick the cut at the target percentile.

```
BisectionEnsemble(p, T):
  at each bisection node (region of ~n/level tracts):
    1. Run T steps of 2-way ReCom on the region subgraph
    2. Collect T plans with their edge cuts EC_1, ..., EC_T
    3. Pick plan at rank floor(p * T)
    4. Recurse on each half
```

### Integration with the compositor layers

BisectionEnsemble is a **search strategy** that plugs into the existing three-layer compositor:

- **Structure layer**: ApportionRegions / GeoSection / standard-bisect — determines WHICH nodes exist in the bisection tree and their population targets.
- **Search layer**: BisectionEnsemble(p, T) — at each node, runs a local 2-way ReCom ensemble and picks the cut at percentile p. This replaces the single METIS call per node.
- **Weights layer**: edge weights (county, geographic, uniform) are still applied to the subgraph at each level before running the local ensemble.

### CLI

```bash
redist state --state NC --structure prime-factor \
  --search bisection-ensemble \
  --percentile 0.50 \
  --ensemble-steps 500
```

### YAML config

```yaml
algorithm:
  structure: prime-factor
  search: bisection-ensemble
  percentile: 0.50
  ensemble_steps: 500
```

### Key advantages over full k-way ReCom

1. **Tractable for large k**: Each local ensemble is k=2, so even TX (k=38) decomposes into a sequence of 2-way problems. No TX bipartition failures — at each bisection node we are splitting a subregion into 2 parts, not searching for a balanced k=38 partition directly.
2. **Structure preserved**: The bisection tree from ApportionRegions or GeoSection is preserved. The structure layer determines WHICH populations are grouped together; the search layer determines HOW each bisection cut is chosen.
3. **Consistent percentile**: The percentile choice is applied at every level of the tree, not just the top level. A 50th-percentile choice produces a plan that is at the 50th percentile of the local ReCom distribution at every bisection step.
4. **Parallelisable**: Bisection nodes at the same level are independent — each local ensemble can run on a separate thread. The tree structure provides natural parallelism.
5. **No calibration required**: Unlike TargetedSweep, BisectionEnsemble does not require a separate calibration run against a global ReCom ensemble. The percentile is applied locally at each node.

### Relationship to PercentileSweep and TargetedSweep

| Strategy | Percentile applied at | Calibration needed | TX handling |
|----------|----------------------|--------------------|-------------|
| PercentileSweep | Full k-way plan level | No | METIS (failures possible) |
| TargetedSweep | Full k-way plan level | Yes (global ReCom) | METIS (failures possible) |
| BisectionEnsemble | Each bisection node (k=2) | No | Local 2-way ReCom (no failures) |

BisectionEnsemble is the strongest integration of the two specs: it combines the legal-posture knob from PercentileSweep with the sound statistical foundations of the ReCom ensemble crate.

---

## Implementation plan

### Phase 1 — PercentileSweep (1-2 days, builds on B.16)
- [ ] Add `--search percentile --percentile P --seeds T` to CLI args
- [ ] In runner.rs: run T seeds via SHA-256 chain, collect EC values, sort, pick rank ⌊P·T⌋
- [ ] Add `SeedCompositor::Percentile { p: f64, seeds: usize }` variant

### Phase 1.5 — BisectionEnsemble (1 week, requires redist-ensemble)
- [ ] Add `--search bisection-ensemble --percentile P --ensemble-steps T` to CLI args
- [ ] In runner.rs: at each bisection node, call `redist-ensemble` for 2-way ReCom with T steps
- [ ] Collect T cut records per node, sort by edge cut, return plan at rank ⌊P·T⌋
- [ ] Add `SeedCompositor::BisectionEnsemble { p: f64, steps: usize }` variant
- [ ] Parallel execution: use Rayon to run independent bisection nodes concurrently
- [ ] Integration tests: NC prime-factor + bisection-ensemble at p=0.0, 0.5, 1.0

### Phase 2 — TargetedSweep (1 week, requires redist-ensemble)
- [ ] Add calibration step: read `ensemble.json` output to get EC_target(p)
- [ ] Add `SeedCompositor::Targeted { ec_target: f64, seeds: usize }` variant
- [ ] CLI: `redist state --search targeted --calibration ensemble.json --percentile 0.50`

### Phase 3 — Research (ongoing)
- [ ] Run PercentileSweep for all 50 states at p=0.0, 0.25, 0.50, 0.75 — compare partisan outcomes
- [ ] Run BisectionEnsemble for all 50 states at p=0.0, 0.25, 0.50, 0.75 — compare partisan outcomes
- [ ] Write H.0 paper: confirm partisan insensitivity to percentile choice
- [ ] Update G.2/G.3 with bisection-space partisan/compactness correlation

---

## Key insight: the percentile IS the legal choice

The compactness percentile is not a technical parameter — it is a **legal/policy decision** about what "fairness" means:

| Percentile | Legal doctrine | Argument |
|-----------|---------------|----------|
| 0th (minimum) | Compactness maximisation (*Karcher v. Daggett*) | Most compact plan; no neutral alternative is more compact |
| 50th (median) | Representativeness (*Rucho* dissent) | The plan a neutral party would "most likely" draw |
| Calibrated to enacted | Status-quo preservation | Closest compact map to what currently exists |

PercentileSweep makes this choice **explicit, transparent, and auditable** — whichever percentile the statute specifies, the output is deterministic given the census data and the target percentile. This may be the strongest legal framing of all: the legislature chose the legal posture; the algorithm executes it without further discretion.
