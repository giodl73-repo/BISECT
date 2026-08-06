# Certified Versus METIS Multi-Instance Protocol

**Status:** precommitted before benchmark execution
**Protocol ID:** `certified-vs-metis-multi-instance-v1`
**Scope:** bounded synthetic root splits only

## Question

On a fixed, heterogeneous suite of small connected graphs, how often does the
vendored METIS implementation return the same feasible assignment, the same
three-part primary objective, or a different objective from the bounded exact
certified oracle?

This protocol measures agreement and disagreement. It does not designate
either outcome as success and must not be revised in response to the observed
results.

## Frozen Methods

- Exact method: `solve_certified_split_bounded`, exhaustive enumeration under
  the certified split model and its canonical tie rule.
- Heuristic method: `bisect_runner::bisection_runner::split_subgraph` using the
  vendored METIS backend selected by the normal build.
- METIS seeds: `1`, `7`, `42`, `2020`, and `314159` for every instance.
- METIS iterations: `10`.
- METIS imbalance multiplier: `1.10`.
- Target weights: `k_left / k_parent` and `k_right / k_parent`.
- Equal-seat orientation: complement the METIS labels when necessary so unit
  zero is in the left child. Unequal-seat labels are not complemented.
- All METIS errors, disconnected results, and insufficient-child-unit results
  remain in the report; no failed row may be discarded or retried.

## Frozen Instances

Unit IDs are `u00` through `uNN` in numeric order. Unless an override is
listed, every undirected edge has integer weight 1. Grid coordinates are
row-major. Each instance stays below the oracle's 24-unit limit.

| ID | Units | Parent seats | Graph | Populations in unit order | Edge-weight override |
|---|---:|---:|---|---|---|
| `path8-equal` | 8 | 4 | path | eight values of 100 | none |
| `cycle10-varied` | 10 | 4 | cycle | `80,120,90,110,95,105,85,115,100,100` | none |
| `ladder2x6-varied` | 12 | 4 | two 6-unit paths plus six same-column rungs | `90,110,95,105,85,115,120,80,100,100,108,92` | none |
| `grid3x4-weighted` | 12 | 4 | orthogonal 3-by-4 grid | twelve values of 100 | edges crossing columns 1 and 2 have weight 5 |
| `barbell12-bridge` | 12 | 4 | cliques on units 0--5 and 6--11 joined only by edge 5--6 | `95,105,100,100,90,110,110,90,100,100,105,95` | bridge edge has weight 3 |
| `tree13-unequal` | 13 | 5 | heap tree with edge `(i,(i-1)/2)` for each `i=1..12` | `70,130,90,110,80,120,100,95,105,85,115,75,125` | none |
| `grid4x4-unequal` | 16 | 5 | orthogonal 4-by-4 grid | `82,118,91,109,97,103,86,114,121,79,106,94,88,112,99,101` | none |
| `cycle20-equal` | 20 | 6 | cycle | `90,110` repeated ten times | every edge incident to a unit divisible by 5 has weight 2 |

For the weighted grid, "crossing columns 1 and 2" means the three horizontal
edges `(1,2)`, `(5,6)`, and `(9,10)` under zero-based row-major indexing. This
explicit edge list controls.

## Recorded Fields

For every instance, record the instance hash, unit/edge counts, seat split,
candidate and feasible exact-assignment counts, exact objective, exact
canonical assignment, number of exact primary-objective ties, exact elapsed
time, and exact proof commitment.

For every instance-seed pair, record the METIS status, elapsed time,
assignment when available, objective when evaluable, connectivity,
child-unit sufficiency, assignment agreement, full-primary-objective
agreement, population-objective agreement, and weighted-boundary difference.

The aggregate must report counts over all 40 precommitted METIS rows, including
errors and infeasible results in the denominator. It must also report results
by instance and by seed. Runtime values are descriptive observations from the
named machine, not stable conformance hashes or speed claims.

## Verification

An independent Python verifier must:

1. verify the package and generator hashes;
2. reconstruct every graph, population vector, seat split, and edge weight
   from this protocol;
3. recompute instance hashes through an implementation independent of the Rust
   serializer;
4. recompute each reported objective and connectivity result from assignments;
5. confirm that all eight instances and all 40 fixed instance-seed rows exist
   exactly once;
6. confirm aggregate and grouped counts from the rows; and
7. reject any conclusion that exceeds the claim boundary below.

The verifier does not independently solve the optimization problem. Exact
optimality is supported by the Rust exhaustive proof artifacts and bounded
oracle verifier; cross-language independent optimality remains a separate
gate.

## Interpretation Fixed Before Results

- Assignment agreement means METIS found the exact canonical tie-selected
  assignment.
- Objective agreement means METIS found any assignment with the same complete
  lexicographic primary objective; it need not select the same tied assignment.
- A lower METIS boundary value paired with worse population deviation is not a
  better certified objective.
- Disconnection or insufficient child units is a heuristic feasibility failure.
- This finite synthetic suite may reveal behavior and test the comparison
  machinery, but cannot estimate national agreement rates.

## Claim Boundary

The package may claim only bounded exact-versus-heuristic agreement and
disagreement on these eight precommitted synthetic instances and five fixed
seeds. It cannot establish State-scale proof feasibility, national runtime,
national map quality, compactness superiority, partisan fairness, community
representation, VRA compliance, legal validity, or official adoption.
