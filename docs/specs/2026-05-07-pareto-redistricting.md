# Spec: Multi-Objective Pareto Redistricting — NSGA-II Genetic Algorithm for Pareto-Optimal Plan Frontier

**Status**: Accepted (R2 avg 3.75/4 — ready for implementation)
**Reviewed R1**: MERIDIAN 3/4, BENCHMARK 3/4, SURVEY 3/4, COVENANT 3/4 → avg 3.0/4
**Reviewed R2**: MERIDIAN 4/4, BENCHMARK 4/4, SURVEY 3/4, COVENANT 4/4 → avg 3.75/4
**Date**: 2026-05-07
**Related paper**: B.26
**Depends on**: `bisect-core` (adjacency, population), `bisect-cli` (Flip chain mutation), `bisect-metis` (METIS plan generation)
**Architecture**: Standalone command (`bisect pareto`) — not a `SeedCompositor` variant
**Citation**: Deb et al. (2002) — "A Fast and Elitist Multiobjective Genetic Algorithm: NSGA-II"

---

## 1. Motivation and positioning

All current redistricting optimisation uses **weighted-sum objectives**: minimise w1×EC + w2×partisan_deviation + w3×VRA_deficit. This has two problems:

1. The weight choice is **opaque** — practitioners tune weights until they get a plan they like
2. It produces ONE plan, hiding the trade-off structure

**Pareto-front redistricting** is honest: it produces a frontier of plans where no plan dominates another on ALL objectives. Practitioners then choose with full transparency — "I chose this plan because it trades X points of compactness for Y points of VRA improvement."

**Legal value**: If the enacted plan is **Pareto-dominated** (another plan is strictly better on ALL criteria simultaneously), that is direct evidence of gerrymandering — the legislature chose a worse plan by their own stated criteria.

### What Pareto redistricting is and is not

| Property | NSGA-II (bisect pareto) | ConvergenceSweep | BisectionEnsemble |
|---------|------------------------|-----------------|------------------|
| Output | N Pareto-optimal plans | single plan | single plan |
| Objective | multi-objective frontier | single weighted sum | single weighted sum |
| Weight choice | none — full frontier | required | required |
| Legal use | dominance test on enacted plan | certification | optimisation |
| Distribution | not calibrated | deterministic | approximate |
| Per-state runtime | minutes–hours (N_pop=100, N_gen=200) | seconds | seconds |
| `SeedCompositor` variant | no — standalone | yes | yes |

NSGA-II is the right tool when the question is "what are the trade-offs between compactness, partisan fairness, and VRA compliance?" Weighted-sum methods are the right tool when the question is "find the best single plan given my priorities." The enacted-plan dominance test is a distinct legal application: it requires no weight choices and produces a binary answer.

---

## 2. Three objectives

Every plan is evaluated on three objectives simultaneously. All three are minimised.

### 2.1 EC (edge cuts)

Total boundary length — the number of edges in the tract adjacency graph that cross district boundaries. Lower is more compact.

```
EC(plan, adj) = |{(u, v) ∈ adj : plan[u] ≠ plan[v]}|
```

### 2.2 D_seats (partisan distance)

Distance from proportional representation using 2020 presidential precinct returns. Lower means closer to proportional.

```
proportional_seats = k × (total_D_votes / total_votes)
D_seats_won = |{d : majority(d) = D}|
D_seats(plan) = |D_seats_won − proportional_seats|
```

The distance framing is neutral — it does not favour D or R outcomes, only representational fidelity. See §8 (Open questions) for alternative framings.

### 2.3 VRA_deficit

Measures the degree to which minority-opportunity districts are underserved. Lower means better VRA compliance.

```
VRA_deficit(plan) = Σ_{d : minority_VAP(d) < 0.50} max(0, 0.50 − minority_VAP(d))
```

Only districts that fall below the 50% minority VAP threshold contribute. Districts already above 50% contribute 0. If no district falls below 50%, VRA_deficit = 0.

**VRA as objective, not constraint**: plans may be VRA-non-compliant but are ranked lower on this objective. This lets the frontier show the compactness cost of VRA compliance. Practitioners can filter the output to only Pareto-optimal VRA-compliant plans (VRA_deficit = 0).

---

## 3. Algorithm: NSGA-II for redistricting

NSGA-II (Deb et al. 2002) is a genetic algorithm that maintains population diversity by replacing weighted aggregation with non-dominated sorting and crowding distance. We adapt it to redistricting plans as chromosomes.

### 3.1 High-level structure

```
NSGA-II(adj, pop, k, N_pop=100, N_gen=200, base_seed):

  // Initialise population: N_pop METIS plans with distinct seeds
  population = [run_all_splits(adj, pop, k, init_seed(i, base_seed)) for i in 0..N_pop]

  for gen in 0..N_gen:
    // Evaluate objectives for all plans
    objectives = [evaluate(p, adj, pop, k) for p in population]   // (EC, D_seats, VRA_deficit)

    // Non-dominated sort: assign Pareto rank to each plan
    fronts = fast_non_dominated_sort(objectives)

    // Crowding distance: maintain diversity within each front
    for front in fronts:
      crowding_distance(front, objectives)

    // Selection: binary tournament based on (rank, crowding_distance)
    parents = tournament_select(population, fronts)

    // Crossover: merge two adjacent district pairs from parents
    offspring = [crossover(parents[2*i], parents[2*i+1], adj, pop, k,
                           cross_seed(gen, i, base_seed))
                 for i in 0..N_pop/2]

    // Mutation: single boundary-tract flip
    offspring = [mutate(p, adj, pop, balance_tolerance, mut_seed(gen, i, base_seed))
                 for i, p in enumerate(offspring)]

    // Combine parent + offspring population, truncate to N_pop
    combined = population + offspring
    objectives_combined = [evaluate(p, adj, pop, k) for p in combined]
    fronts_combined = fast_non_dominated_sort(objectives_combined)
    population = select_top_N(combined, N_pop, fronts_combined)

  // Return Pareto front (front rank 0)
  final_objectives = [evaluate(p, adj, pop, k) for p in population]
  final_fronts = fast_non_dominated_sort(final_objectives)
  front_0 = [i for i, rank in fronts if rank == 0]
  return ParetoResult {
    frontier: [population[i] for i in front_0],
    all_objectives: [final_objectives[i] for i in front_0],
  }
```

### 3.2 Non-dominated sort: `fast_non_dominated_sort`

Plan A **dominates** plan B if A is strictly better on at least one objective and no worse on any objective:

```
dominates(A, B) = (∀ obj: A[obj] ≤ B[obj]) ∧ (∃ obj: A[obj] < B[obj])
```

The fast non-dominated sort (Deb et al. 2002, Algorithm 1) assigns each plan a Pareto rank:

```
fast_non_dominated_sort(objectives) -> Vec<usize>:   // rank for each plan
  n = len(objectives)
  domination_count = [0] × n     // how many plans dominate plan i
  // dominates_set[p] = set of plans that plan p dominates (p is better on all objectives)
  dominates_set[i] = []

  for p in 0..n:
    for q in 0..n:
      if dominates(objectives[p], objectives[q]):
        dominates_set[p].push(q)
      elif dominates(objectives[q], objectives[p]):
        domination_count[p] += 1

  fronts = [[]]
  for p in 0..n:
    if domination_count[p] == 0:
      fronts[0].push(p)         // rank 0: Pareto front

  i = 0
  while fronts[i] is not empty:
    next_front = []
    for p in fronts[i]:
      for q in dominates_set[p]:
        domination_count[q] -= 1
        if domination_count[q] == 0:
          next_front.push(q)    // rank i+1
    fronts.push(next_front)
    i += 1

  return ranks   // rank[i] = front index for plan i
```

O(N²) per generation. For N_pop=100, this is 10,000 comparisons — fast enough.

### 3.3 Crowding distance

Within each Pareto front, crowding distance measures how isolated a plan is in objective space. Plans with higher crowding distance are preferred during selection to maintain diversity.

```
crowding_distance(front, objectives):
  n = len(front)
  distances = [0.0] × n

  for each objective m:
    sorted = sort(front by objectives[·][m])
    distances[sorted[0]] = +∞     // boundary plans: infinite distance
    distances[sorted[n-1]] = +∞
    obj_range = objectives[sorted[n-1]][m] − objectives[sorted[0]][m]
    if obj_range == 0: continue   // all plans identical on this objective

    for i in 1..n-1:
      distances[sorted[i]] += (objectives[sorted[i+1]][m] − objectives[sorted[i-1]][m])
                               / obj_range

  return distances
```

### 3.4 Tournament selection

Binary tournament: draw two plans at random; prefer the plan with lower Pareto rank; break ties by higher crowding distance.

```
tournament_select(population, fronts, crowding) -> parents:
  parents = []
  for _ in 0..N_pop:
    a, b = random_pair(population)
    if rank[a] < rank[b]:    parents.push(a)
    elif rank[b] < rank[a]:  parents.push(b)
    elif crowd[a] > crowd[b]: parents.push(a)
    else:                     parents.push(b)
  return parents
```

### 3.5 `select_top_N`

After combining parent + offspring (2×N_pop plans), truncate to N_pop:

```
select_top_N(combined, N_pop, fronts_combined) -> Vec<plan>:
  selected = []
  for front in fronts_combined (ascending rank):
    if len(selected) + len(front) ≤ N_pop:
      selected += front        // entire front fits
    else:
      // partial front: fill by crowding distance (descending)
      remaining = N_pop − len(selected)
      distances = crowding_distance(front, objectives_combined)
      sorted_front = sort(front by distances desc)
      selected += sorted_front[0..remaining]
      break
  return selected
```

---

## 4. Crossover operator

ReCom-style crossover: merge two adjacent districts from parent A, then regrow using a spanning tree proposal seeded from the crossover seed. This is simpler than full ReCom — no Metropolis-Hastings ratio is needed since NSGA-II has its own selection pressure.

```rust
fn crossover(parent_a: &[u32], parent_b: &[u32], adj: &[Vec<usize>], pop: &[i64],
             k: usize, seed: u64) -> Vec<u32> {
    // 1. Pick a random adjacent district pair (d1, d2) from parent_a
    // 2. Merge districts d1 and d2 into one region
    // 3. Sample a spanning tree of the merged region (Wilson's algorithm, seed-derived)
    // 4. Find a balanced cut edge in the spanning tree
    //    (balance criterion: each component within pop_tolerance of target_pop)
    // 5. Assign the two new districts; keep all other districts from parent_a unchanged
    // 6. If no balanced cut exists: retry up to 5 times with different spanning trees
    // 7. If all retries fail: return parent_a unchanged
}
```

The fallback to parent_a (unchanged) is intentional: NSGA-II tolerates a fraction of no-op crossovers without degenerating, provided the mutation operator is active. If the crossover validity rate falls below 20%, Phase 2 will investigate improved proposals (see §8).

---

## 5. Mutation operator

Single boundary-tract flip, reusing the Flip chain logic from `bisect-cli`:

```rust
fn mutate(plan: &[u32], adj: &[Vec<usize>], pop: &[i64],
          balance_tolerance: f64, seed: u64) -> Vec<u32> {
    // 1. Collect all boundary tracts:
    //    a tract t is a boundary tract if ∃ neighbour u with plan[u] ≠ plan[t]
    // 2. Pick a random boundary tract t
    // 3. Collect candidate target districts: all districts adjacent to t that differ from plan[t]
    // 4. Pick a random target district d_new
    // 5. Tentatively assign plan[t] = d_new
    // 6. Check validity:
    //    a. Population balance: |pop(d_new_updated) − target_pop| ≤ balance_tolerance × target_pop
    //                          |pop(d_old_updated) − target_pop| ≤ balance_tolerance × target_pop
    //    b. Contiguity: d_old (now minus tract t) is still connected (BFS check)
    // 7. If valid: return updated plan
    // 8. If invalid: return plan unchanged
}
```

The mutation makes exactly one attempt. If the selected flip is invalid, the plan is returned unchanged. This preserves the invariant that all plans in the population are always valid.

---

## 6. Seeding specification

All stochastic operations are derived from `base_seed` via SHA-256 with domain-separated prefixes. This ensures full reproducibility given only `base_seed`.

```
init_seed(i: u32, base_seed: u64) -> u64:
  SHA-256("PARETO_INIT_"              // 12 bytes
          || i.to_le_bytes()          // 4 bytes (u32)
          || "_"                      // 1 byte
          || base_seed.to_le_bytes()) // 8 bytes (u64)
  → first 8 bytes as little-endian u64  // total input: 25 bytes

cross_seed(gen: u32, i: u32, base_seed: u64) -> u64:
  SHA-256("PARETO_CROSS_"             // 13 bytes
          || gen.to_le_bytes()        // 4 bytes (u32)
          || "_"                      // 1 byte
          || i.to_le_bytes()          // 4 bytes (u32)
          || "_"                      // 1 byte
          || base_seed.to_le_bytes()) // 8 bytes (u64)
  → first 8 bytes as little-endian u64  // total input: 31 bytes

mut_seed(gen: u32, i: u32, base_seed: u64) -> u64:
  SHA-256("PARETO_MUT_"               // 11 bytes
          || gen.to_le_bytes()        // 4 bytes (u32)
          || "_"                      // 1 byte
          || i.to_le_bytes()          // 4 bytes (u32)
          || "_"                      // 1 byte
          || base_seed.to_le_bytes()) // 8 bytes (u64)
  → first 8 bytes as little-endian u64  // total input: 29 bytes
```

The prefix strings are version-locked. Any change to the proposal algorithm must change the prefix to prevent silent seed compatibility across versions.

**Seed separation**: the three prefixes `PARETO_INIT_`, `PARETO_CROSS_`, `PARETO_MUT_` differ in length (12, 13, 11 bytes) and content, so `init_seed(i) ≠ cross_seed(gen, i) ≠ mut_seed(gen, i)` for all i, gen — no seed collisions between initialisation, crossover, and mutation operations.

---

## 7. CLI interface

NSGA-II Pareto redistricting is a top-level `bisect pareto` subcommand, not a variant of `bisect ensemble` or `bisect state`:

```bash
# Run NSGA-II for NC congressional (k=14)
bisect pareto \
  --state NC --year 2020 \
  --population 100 \
  --generations 200 \
  --base-seed 42 \
  --output nc_pareto_2020.ndjson

# Enacted plan dominance test
bisect pareto \
  --state NC --year 2020 \
  --population 100 \
  --generations 200 \
  --base-seed 42 \
  --enacted-plan configs/enacted_nc.yml \
  --output nc_pareto_2020.ndjson

# SMC-project alternative (faster, uses existing SMC ensemble)
bisect pareto \
  --state NC --year 2020 \
  --method smc-project \
  --smc-input nc_smc_2020.ndjson \
  --output nc_pareto_smc_2020.ndjson
```

### 7.1 Parameter defaults

| Flag | Default | Notes |
|------|---------|-------|
| `--population` | 100 | NSGA-II population size N_pop |
| `--generations` | 200 | Number of NSGA-II generations N_gen |
| `--base-seed` | 42 | Seed for all stochastic operations |
| `--balance-tolerance` | 0.005 | ±0.5% population balance (congressional standard) |
| `--method` | `nsga2` | `nsga2` or `smc-project` |
| `--enacted-plan` | none | Path to YAML plan config for dominance test |
| `--output` | stdout | NDJSON output file path |

### 7.2 Parameter scaling

| State size | N_pop | N_gen | Expected runtime (sequential) | Pareto front size |
|-----------|-------|-------|-------------------------------|------------------|
| Small (k≤5, e.g. VT) | 50 | 100 | <5 min | 3–8 |
| Medium (k=8–14, e.g. WI, NC) | 100 | 200 | 15–60 min | 10–30 |
| Large (k=30+, e.g. TX, CA) | 200 | 500 | 4–12 hours | 20–60 |

Pareto front size grows with k because the objective space becomes larger. For small k (e.g. VT k=1), the Pareto front degenerates to a single plan. All timing estimates are approximate and hardware-dependent; report with CPU model, core count, and RAM.

---

## 8. New crate: `bisect-pareto`

NSGA-II is complex enough to warrant its own crate, separate from `bisect-ensemble` and `bisect-smc`.

```
BISECT/crates/bisect-pareto/
  src/
    lib.rs              // pub use
    algorithm.rs        // run_nsga2() top-level, NSGA-II main loop
    dominance.rs        // fast_non_dominated_sort(), crowding_distance()
    crossover.rs        // ReCom-style crossover operator
    mutation.rs         // single flip mutation
    objectives.rs       // EC, D_seats, VRA_deficit evaluation
    seeds.rs            // init_seed(), cross_seed(), mut_seed() SHA-256 derivation
    output.rs           // ParetoResult, NDJSON serialisation
    smc_project.rs      // SMC-to-Pareto projection (--method smc-project)
  tests/
    L0_unit.rs          // dominance, crowding distance, seed derivation
    L1_integration.rs   // small synthetic graphs (4-node path, 9-node grid)
    L2_real.rs          // #[ignore] NC, WI real data
  Cargo.toml
```

### 8.1 Core types

```rust
/// Objective values for a single redistricting plan.
pub struct Objectives {
    pub ec: u64,            // edge cuts (lower = more compact)
    pub d_seats: f64,       // |D_seats_won − proportional_seats| (lower = more proportional)
    pub vra_deficit: f64,   // sum of shortfalls below 50% minority VAP (lower = better VRA)
}

/// A single Pareto-optimal plan with its objective values.
pub struct ParetoEntry {
    pub plan: Vec<u32>,              // assignment[tract] = district (1-based)
    pub objectives: Objectives,
    pub dominated: bool,             // always false in the returned frontier
    pub generation_found: u32,       // generation in which this plan entered front 0
}

/// Full result of an NSGA-II run.
pub struct ParetoResult {
    /// Plans on the Pareto front (front rank 0 only)
    pub frontier: Vec<ParetoEntry>,
    /// Algorithm parameters (for audit)
    pub n_population: usize,
    pub n_generations: usize,
    pub base_seed: u64,
    pub runtime_secs: f64,
}
```

### 8.2 Output format: NDJSON

Each line is one Pareto-optimal plan on the frontier:
```json
{"plan": [1,1,2,...], "ec": 3124, "d_seats": 7, "vra_deficit": 0.0, "dominated": false, "generation_found": 142}
```

If `--enacted-plan` is specified, the enacted plan is evaluated and appended as a separate record:
```json
{"type": "enacted", "plan": [...], "ec": 3891, "d_seats": 5, "vra_deficit": 0.12,
 "is_dominated": true, "dominating_plans": 3}
```

Final line is a metadata record:
```json
{
  "type": "metadata",
  "n_population": 100,
  "n_generations": 200,
  "pareto_front_size": 23,
  "base_seed": 42,
  "runtime_secs": 847.3,
  "pareto_version": "1.0",
  "file_sha256": "a3f4..."
}
```

**Line-ending encoding**: NDJSON lines are terminated with `\n` (LF, 0x0A) regardless of host platform. Implementations on Windows must normalise `\r\n` to `\n` before computing the SHA-256. This ensures cross-platform reproducibility.

The `file_sha256` field is the SHA-256 of all bytes in the file preceding the final metadata line (all plan records and the enacted record if present).

---

## 9. Audit chain

The metadata record provides full reproducibility. A verifier with access to the census data and `base_seed` can reconstruct the entire run:

```json
{
  "pareto_version": "1.0",
  "n_population": 100,
  "n_generations": 200,
  "base_seed": 42,
  "balance_tolerance": 0.005,
  "state": "NC",
  "year": "2020",
  "k": 14,
  "init_seed_formula":  "SHA-256('PARETO_INIT_'  || i:u32le || '_' || base_seed:u64le) → first 8 bytes as u64le",
  "cross_seed_formula": "SHA-256('PARETO_CROSS_' || gen:u32le || '_' || i:u32le || '_' || base_seed:u64le) → first 8 bytes as u64le",
  "mut_seed_formula":   "SHA-256('PARETO_MUT_'   || gen:u32le || '_' || i:u32le || '_' || base_seed:u64le) → first 8 bytes as u64le",
  "pareto_front_size": 23,
  "runtime_secs": 847.3,
  "file_sha256": "a3f4..."
}
```

**Verification protocol** (4 steps):
1. Verify `file_sha256` matches a fresh SHA-256 of the file body (all lines before the metadata line)
2. Re-derive `init_seed(i, base_seed)` for i in 0..N_pop and confirm the initialisation plans are reproducible
3. Confirm `dominated = false` for all plans in the frontier (no plan in the output is dominated by another plan in the output)
4. Confirm all frontier plans are valid: contiguous districts, population balance within `balance_tolerance`

The NSGA-II run itself (200 generations × crossover + mutation) is not fully replayed during verification — only the final frontier is checked for validity and non-dominance. Full replay is possible given the seed formulas and census data.

**Validity vs completeness distinction**: The 4-step verification protocol is a *validity check* — it confirms the returned plans are mutually non-dominating, have correct objective values, and the file is untampered. It does **not** guarantee the frontier is the true NSGA-II Pareto front from the stated population and generations. A manipulated implementation could return any set of mutually non-dominating plans.

*Completeness check* (expensive): to confirm the frontier is the actual NSGA-II output, an auditor must re-run the full algorithm with the documented seeds, population size, and generation count on the same census data. The seed formulas make this possible but not fast (hours of compute for large states). For litigation use, the completeness check can be performed by opposing counsel's expert with the documented parameters.

---

## 10. SMC-Pareto alternative

After running SMC (`bisect ensemble --method smc`), project the N_particles onto objective space. The Pareto front of the SMC sample approximates the true Pareto front without running NSGA-II.

```bash
bisect pareto \
  --method smc-project \
  --smc-input nc_smc_2020.ndjson \
  --output nc_pareto_smc_2020.ndjson
```

`smc_project.rs` reads the NDJSON file, evaluates objectives for each particle (weighted by SMC importance weight), runs `fast_non_dominated_sort`, and outputs the Pareto front using the same NDJSON format as NSGA-II.

**Comparison**: The SMC distribution may not cover all Pareto-optimal regions — SMC samples from the near-uniform distribution over valid plans, which may undersample extreme compactness or extreme VRA compliance. NSGA-II actively searches for Pareto-optimal diversity. Phase 2 will measure whether NSGA-II finds regions SMC misses (see §11, Open questions).

Both approaches are supported: `bisect pareto` (default) runs NSGA-II; `bisect pareto --method smc-project` runs the projection.

---

## 11. Enacted plan dominance test

```bash
bisect pareto \
  --state NC --year 2020 \
  --enacted-plan configs/enacted_nc.yml \
  --output nc_pareto_2020.ndjson
```

The enacted plan is evaluated on all three objectives and compared against the Pareto frontier:

- `is_enacted_dominated: true` if at least one frontier plan dominates the enacted plan (strictly better on at least one objective, no worse on any)
- `dominating_plans: N` — count of frontier plans that dominate the enacted plan

**Legal interpretation**: If `is_enacted_dominated: true`, the legislature could have enacted a plan that was simultaneously more compact, more proportional, and better for VRA than the chosen plan. This does not automatically constitute gerrymandering, but is evidence that the enacted plan was not Pareto-optimal by the legislature's own stated criteria.

A result of `is_enacted_dominated: false` means the enacted plan is on (or close to) the Pareto frontier — it makes genuine trade-offs that cannot be improved simultaneously on all objectives. This does not mean it is fair, only that it is not trivially dominated.

---

## 12. Test invariants

### L0 (unit, always run)

- **Dominance**: if A = (ec=100, d_seats=5, vra=0.0) and B = (ec=200, d_seats=5, vra=0.0), then `dominates(A, B)` is true and `dominates(B, A)` is false
- **Non-dominance**: if A = (ec=100, d_seats=6, vra=0.0) and B = (ec=200, d_seats=5, vra=0.0), then neither dominates the other
- **Non-dominated sort**: if A dominates B, then `rank[A] < rank[B]` — B is in a higher-numbered front than A
- **Crowding distance**: plans at the extreme of any objective (minimum or maximum value on that objective within their front) have infinite crowding distance
- **Seed separation**: `init_seed(i, s) ≠ init_seed(j, s)` for i ≠ j (distinct initialisation seeds)
- **Seed separation**: `cross_seed(g, i, s) ≠ mut_seed(g, i, s)` for all g, i, s (crossover and mutation seeds never collide)
- **Determinism**: same `base_seed` → identical sequence of `init_seed`, `cross_seed`, `mut_seed` values (exact equality)
- **Non-dominance in frontier**: all plans in the returned `ParetoResult.frontier` have `dominated = false`; no plan in the frontier is dominated by any other plan in the frontier
- `file_sha256` field in metadata is exactly 64 hexadecimal characters
- NDJSON lines are LF-terminated (`\n`, 0x0A); no `\r\n` sequences in output

### L1 (integration, synthetic graphs, always run)

- `run_nsga2(4-node path, pop=[100,100,100,100], k=2, N_pop=10, N_gen=5, base_seed=42)` → at least 1 plan in the Pareto frontier; all plans are valid (contiguous districts, population within balance_tolerance)
- **Determinism**: `run_nsga2(..., base_seed=42)` called twice produces identical `frontier` plans and identical objective values (exact equality)
- **Dominance test**: construct two plans A and B where A has lower EC, same D_seats, same VRA_deficit. After `fast_non_dominated_sort([A, B])`, rank[A] = 0 and rank[B] = 1
- **Crossover validity**: crossover on any two valid plans either returns a valid plan or returns parent_a unchanged; it never panics and never returns an invalid plan
- **Mutation validity**: mutation on a plan returns either a valid flipped plan or the original plan unchanged; it never panics and never returns an invalid plan
- **Mutation with no valid flips**: if all boundary-tract flips are invalid (due to balance or contiguity), mutation returns the original plan unchanged
- All plans in the Pareto frontier: `dominated = false`; no plan in the frontier is dominated by any other plan in the frontier (self-check)
- `generation_found` ≤ N_gen for all plans in the frontier

**SMC-project path (L1)**:
- Write a synthetic NDJSON file with 10 particle lines (using the `SmcResult` format from bisect-smc) where each particle has a pre-computed `plan`, `log_weight`, and `particle_idx`
- Call `smc_project::load_smc_and_evaluate(&synthetic_ndjson, adj, pop, k)`
- Assert: objective vectors are computed for all 10 particles; the returned Pareto front contains only non-dominated plans; the NDJSON output format is identical to the NSGA-II output format (same field names, same NDJSON structure)

### L2 (real data, `#[ignore]`)

- `run_nsga2(NC, 2020, k=14, N_pop=100, N_gen=50, base_seed=42)` → Pareto front size ≥ 5 (meaningful objective-space diversity)
- **Enacted plan dominance**: run against the NC enacted 2020 congressional map; report `is_enacted_dominated` (true or false); this is a regression test — the value is recorded, not asserted
- **VRA_deficit = 0 plans exist**: at least one plan in the NC Pareto frontier has VRA_deficit = 0 (confirms that VRA-compliant plans are reachable)

---

## 13. Implementation plan

### Phase 1 (this spec): `bisect-pareto` crate skeleton

1. **Week 1**: `Objectives`, `ParetoEntry`, `ParetoResult` types; `seeds.rs` L0 tests for all seed functions
2. **Week 2**: `dominance.rs` — `fast_non_dominated_sort`, `crowding_distance`; L0 dominance tests
3. **Week 3**: `objectives.rs` — EC, D_seats, VRA_deficit evaluation against NC census data
4. **Week 4**: `crossover.rs`, `mutation.rs`; L1 integration tests on 4-node and 9-node graphs
5. **Week 5**: `algorithm.rs` — full NSGA-II main loop; `output.rs` — NDJSON serialisation + `file_sha256`
6. **Week 6**: CLI wiring in `bisect pareto`; `smc_project.rs`; L2 NC enacted-plan dominance test

### Phase 2 (follow-on spec): SMC comparison + legal use

Once `bisect-pareto` is stable, a follow-on spec will define:
- Empirical comparison: does NSGA-II find Pareto regions that the SMC projection misses?
- Publication-quality dominance test: NC enacted plan vs NSGA-II frontier, N_pop=200, N_gen=500
- 50-state sweep: Pareto front size and enacted-plan dominance across all states

### Known risks

1. **Crossover validity rate**: if fewer than 20% of crossover attempts produce a valid plan (balanced + contiguous), NSGA-II degenerates to random mutation. Mitigation: measure validity rate in L2 test; if < 20%, switch to a targeted crossover (pick the adjacent district pair with the most balanced existing split).
2. **Pareto front collapse**: for small k (e.g. VT k=1), all valid plans may be non-dominated, producing a Pareto front of size N_pop with no diversity in objective space. Mitigation: detect degenerate case (front 0 = all plans) and warn; this is correct behaviour, not a bug.
3. **D_seats resolution**: D_seats is integer-valued; many plans may tie on D_seats. The Pareto ordering may then be dominated by EC and VRA_deficit. Mitigation: use vote margin (total D_votes - threshold) as a continuous proxy. Deferred to Phase 2.
4. **Memory**: 100 plans × 5200 NC tracts × 4 bytes = 2 MB. For N_pop=200, TX (5200+ tracts), 200 generations: still only ~80 MB. Memory is not a concern for NSGA-II at these scales.

---

## 14. Open questions (deferred)

1. **VRA as constraint vs objective**: current spec uses VRA_deficit as an objective (plans may violate VRA but are ranked lower). Should VRA-compliant plans be a hard filter with only the compliant Pareto front returned? Deferred — practitioners can filter the NDJSON output by `vra_deficit = 0`. A `--vra-constrained` flag is a Phase 2 addition.
2. **Crossover validity rate**: how often does the ReCom-style crossover produce a valid plan? If < 20%, NSGA-II degenerates to random mutation. Phase 2 will measure this empirically on NC and TX.
3. **Resolved (P1)**: D_seats integer resolution acknowledged and documented. With NC k=14, D_seats_won ∈ {0..14} produces only ~14 distinct values of D_seats = |D_seats_won - proportional_seats|. The Pareto front may degenerate to a 2D EC vs VRA_deficit frontier when many plans achieve the same D_seats value.

   **Phase 1 decision**: Use `d_seats: f64 = |D_seats_won as f64 - proportional_seats|` without modification. The discrete nature is disclosed in the NDJSON output via a metadata field `d_seats_discrete: true` and in the output documentation. When D_seats fails to differentiate plans, the effective Pareto front over EC and VRA_deficit is still valuable and legally defensible.

   **Phase 2**: Consider vote margin (sum of |district_vote_share - 0.5| × district_pop) as a continuous D_seats proxy to improve objective resolution.
4. **SMC-Pareto comparison**: does the SMC sample cover the Pareto frontier well, or does NSGA-II find regions SMC misses? Phase 2 empirical comparison on NC 2020 with N_particles=5000 vs N_pop=100, N_gen=200.
5. **Parallelism**: NSGA-II's objective evaluation is embarrassingly parallel (each plan evaluated independently). Rayon parallelism over the population is straightforward. Deferred to Phase 2 after the sequential implementation is validated.
