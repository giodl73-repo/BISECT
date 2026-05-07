# B.7 — The Solution Space of Minimum-Edge-Cut Redistricting: Seed Sensitivity and Partisan Variance

**Paper Type**: Algorithm Theory + Empirical Study  
**Status**: Planned (stub created 2026-05-01)  
**Target Venue**: Journal of Computational Geometry / Political Analysis / ACM SODA proceedings  
**Format**: 20-25 pages + appendix  
**Target Audience**: Algorithm theorists, election-law specialists, federal-statute coalition partners, opposing counsel

---

## Motivation

The Districting Integrity Act publishes a single fixed random seed (§ 107(c)(1)) that all states use as input to METIS. This is the statute's reproducibility guarantee: anyone who runs the reference implementation with the published seed and TIGER inputs gets the same map.

But METIS is a heuristic. Graph minimum bisection is NP-hard; METIS finds a *local* optimum via multilevel refinement, not the global minimum. The seed controls which local optimum is reached. A well-prepared challenger will argue:

> "The 'random' seed is not random — it was chosen (by the Director of Census, or whoever drafted § 107(c)) after the partisan consequences of different seeds were known. A different seed produces a different map with materially different partisan outcomes. The statute's procedural neutrality claim depends on the partisan consequences of seed choice being negligible — but that claim is unproven."

This paper proves or disproves that claim empirically, and provides the theoretical grounding for why the claim is or is not defensible.

---

## Research Questions

**RQ1 — Is METIS actually finding the minimum edge cut?**  
What is the approximation ratio of METIS bisection in practice on U.S. state census graphs? How does the achieved edge cut compare to theoretical lower bounds? Are there states where the heuristic's gap is large?

**RQ2 — How many near-minimum-cut plans exist?**  
For a given state, how large is the ε-ball of plans within ε% of the minimum edge cut? Is the solution space highly degenerate (many equivalent plans) or concentrated (one dominant minimum)?

**RQ3 — What is the partisan variance across seeds?**  
For each state, run N seeds (proposed: N = 1000). Measure the distribution of:
- `proportionality_gap_pp` (the new metric — Task 1 of current session)
- Efficiency gap
- Mean-median difference
- Seat allocation (Dem seats / Rep seats)

**RQ4 — Is the seed's partisan effect state-dependent?**  
Hypothesis: states with clear geographic sorting (MA, AL) have low seed variance — the Dem/Rep geography is so dominant that all near-minimum-cut plans look alike. Swing states (WI, PA, AZ) may have higher variance because there are multiple ways to cut through politically mixed zones.

**RQ5 — Is the federal statute's "one seed" design defensible?**  
If the seed's partisan effect is ≤ 2pp proportionality gap across all states, the seed is procedurally irrelevant and the statute's claim holds. If variance is higher in some states, the statute needs a seed-selection procedure that provably does not advantage either party (e.g., seed derived from census release SHA-256 — not chosen by a human post-knowing-the-consequences).

---

## The Theoretical Frame: Not "The Minimum" But "A Minimum-Class Member"

The critical reframe: METIS does not find "the" minimum-edge-cut partition. It finds a representative member of the *minimum-class* — the set of plans within some approximation ratio of the true minimum.

For procedural fairness, the right claim is not "this is the unique geometric minimum" but "this plan was drawn by a procedure that selects from the minimum-class without partisan input." The seed is the tie-breaking mechanism within the minimum-class. The fairness argument holds if and only if:

1. The minimum-class is large enough that many plans satisfy it (otherwise one plan is effectively "the" answer, and no tie-breaking is needed).
2. The seed is selected without knowledge of partisan consequences (either randomly before knowing the data, or derived deterministically from the census data SHA-256).
3. The partisan variance within the minimum-class is bounded (otherwise seed choice is an implicit partisan choice).

All three conditions need empirical grounding. This paper provides it.

---

## Proposed Methodology

### Phase 1: Single-state deep dive (Vermont, Pennsylvania, Texas)
- Vermont: 1 district, geographic minimum is clear. Seed variance should be ~0.
- Pennsylvania: 17 competitive districts. Expected to show nontrivial seed variance.
- Texas: 38 districts, mixed geography. Expected to show moderate variance in D/R seat allocation.

For each: run 1000 seeds, record `proportionality_gap_pp`, efficiency gap, seat allocation. Plot distribution.

### Phase 2: 50-state seed sweep (N = 100 seeds per state)
- Run `redist state` for all 50 states × 100 seeds
- For each state: compute mean, SD, IQR of `proportionality_gap_pp` across seeds
- Identify states with high variance (the "seed-sensitive" states) vs. low variance

### Phase 3: Theoretical lower bound analysis
- For each state, compute a graph-theoretic lower bound on the minimum edge cut (Cheeger constant / Laplacian spectral gap)
- Compare METIS achieved cut to the lower bound
- Report the approximation ratio distribution across states

### Phase 4: Statute implications
- Propose a seed-selection procedure that is provably non-partisan: `SHA-256(census_release_identifier || "redistricting_seed_v1")` — derived from the census data itself, not chosen by a human
- Analyze whether this changes the statute's § 107(c)(1) design

---

## Expected Findings (Hypotheses)

- **H1**: For states with ≤ 3 districts (VT, AK, WY, DE, etc.), seed variance on proportionality is ≈ 0pp. The geography dominates.
- **H2**: For states with 10-20 competitive districts (PA, WI, AZ, NC), seed variance on proportionality is 5-15pp. These are the statute's vulnerable cases.
- **H3**: For deep-red/deep-blue states (MA, AL, MS), seed variance is low (2-5pp) because geographic sorting is so dominant that all near-optimal plans produce the same partisan outcome.
- **H4**: The mean proportionality gap across seeds is close to the single-seed result already computed (Task 1). The single-seed plan is a typical draw, not an outlier.

If H4 holds, the federal statute's reproducibility argument is strengthened: our single published plan is representative of what any neutral seed would produce.

---

## Connection to Existing Work

- **B.1** (recursive bisection): B.7 studies the solution space that B.1 samples from.
- **B.6** (computational complexity): B.6 establishes NP-hardness; B.7 measures the practical consequences of heuristic approximation.
- **C.7** (uncertainty quantification): C.7 studies compactness variance across seeds; B.7 focuses on *partisan* variance specifically, which is the legally salient question.
- **E.4** (partisan similarity): E.4 changes the edge-weighting; B.7 studies variance within the standard (non-partisan-input) weighting.
- **FAIRNESS_DOCTRINE.md**: The ensemble methodology in FAIRNESS_DOCTRINE requires understanding whether a single METIS run is representative. B.7 provides that grounding.
- **MODEL_FEDERAL_STATUTE.md § 107(c)(1)**: The statute's seed-publication design may need to be updated to a content-derived seed if B.7 finds high variance in competitive states.

---

## Data Requirements

- `redist state` run for all 50 states × 1000 seeds (Phase 1 deep dive) + 50 × 100 (Phase 2 sweep)
- `proportionality.json` output from each run (Task 1 infrastructure already built)
- Adjacency graphs for all 50 states (already built, in `outputs/data/`)
- `presidential_by_tract.csv` for 48 states (AK + HI missing; note as limitation)

Estimated compute: ~50 × 100 = 5,000 state runs. At ~30s/state average → ~42 hours wall-clock single-threaded. Parallelizable to ~4 hours on an 8-core machine.

---

## Statute Design Implication (If High Variance Found)

If Phase 2 finds seed variance > 10pp in competitive states, the proposed fix for § 107(c)(1) is:

> The random seed shall be derived as `SHA-256(decennial_census_release_identifier || "DIA_SEED_V1")`, where `decennial_census_release_identifier` is the identifier published under § 107(c)(4). No human shall choose the seed value; it is determined by the census data itself.

This makes the seed:
1. Deterministic given public data (still reproducible)
2. Not chosen by any person post-knowing-the-partisan-consequences (closes the cherry-picking attack)
3. Uniform across all states (same property as the current design)

---

## See Also

- `docs/legal/MODEL_FEDERAL_STATUTE.md` — § 107(c)(1) seed publication
- `docs/legal/STATUTE_REVIEW_NOTES.md` — open tensions (§ 3.3 panel composition)
- `docs/legal/PARTISAN_OPTIONS.md` — proportionality as the cross-cutting metric
- `research/C.7+uncertainty-quantification/` — compactness uncertainty (companion paper)
- `research/E.4+partisan-similarity-districts/` — partisan edge-weighting (contrast case)
- `redist/crates/redist-analysis/src/proportionality.rs` — the metric this paper measures
