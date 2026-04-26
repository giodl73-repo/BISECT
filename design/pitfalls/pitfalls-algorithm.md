# Algorithm Pitfalls (AP-01..AP-05)

Structural vulnerabilities in the core redistricting algorithm. Each describes a class of failure — not a specific bug, but the pattern that makes a whole category of bugs possible.

---

## AP-01: Multi-objective optimization trades constitutional constraints

**Pattern:** When an optimizer has two objectives, it can trade one for the other. If population balance and minority concentration are both METIS objectives, METIS will sacrifice population balance to improve minority concentration whenever the geometry makes that trade locally beneficial. No parameter tuning prevents this — it is inherent to multi-objective optimization.

**Domain:** Any system that asks METIS (or any optimizer) to satisfy multiple hard constraints simultaneously. If one constraint is constitutional and the other is policy-driven, the constitutional constraint must not be in the optimizer's objective function.

**Structural solution:** Encode policy signals (minority clustering) in the graph structure (edge weights), not in the optimization objective (vertex weights). The optimizer sees a single objective — minimize weighted edge cuts — and population balance is enforced by ufactor alone.

**Status:** SOLVED
**Proved by:** All 44 multi-district V4 states pass ±0.5% population balance
**Test:** `tests/integration/test_vra_pipeline_balance.py`

---

## AP-02: Flag-as-mode conflation creates invariant drift

**Pattern:** A boolean flag used to mean two different things will drift when one meaning changes but code depending on the other meaning doesn't. The flag starts as a unified signal; refactoring splits its meanings; code written for the original meaning breaks silently.

**Domain:** Any system where a single flag controls both "is this a special run?" and "use this special algorithm mode?". When the algorithm changes (e.g., from multi-constraint to edge-weighting), the "special run" meaning persists but the "algorithm mode" meaning becomes wrong.

**Structural solution:** Separate the two concerns into separate signals. `vra_mode=True` means "this is a VRA run" (for logging, analysis, branching). `multi_constraint` is a derived property (`vra_target_tree is not None`), not a copy of `vra_mode`. Each flag has exactly one meaning.

**Status:** SOLVED
**Proved by:** recursive_bisection.py uses `multi_constraint=(vra_target_tree is not None)` in all 3 METIS call sites
**Test:** `tests/unit/test_vra_edge_weighting.py::TestVRACodePath`

---

## AP-03: Weighting saturation — signal becomes noise at high density

**Pattern:** A weighting signal applied to a large fraction of edges loses its discriminative power. If 63% of edges are boosted, the boost is nearly uniform — the algorithm sees a graph where almost all edges are expensive, and reverts to arbitrary cuts based on remaining topology. The signal that was supposed to guide clustering instead disrupts balance without creating the intended structure.

**Domain:** Any graph-weighting approach where the signal condition (e.g., "both endpoints are minority") applies to a large fraction of the graph. High-diversity regions have many minority tracts, making most edges minority-minority.

**Structural solution:** Adaptive scaling that tapers the boost as signal density rises. When nearly all edges carry the signal, the boost is reduced to maintain discriminative contrast. Floor ensures some signal always exists.

**Status:** SOLVED
**Proved by:** California (63% minority tracts) and Texas (52%) pass ±0.5% balance
**Test:** `tests/unit/test_vra_edge_weighting.py::TestAdaptiveBoostScaling`, `tests/integration/test_vra_pipeline_balance.py::test_california_vra_population_balance`

---

## AP-04: Indexing assumptions outlast their context

**Pattern:** Code written for data structure A (2D array) uses an indexing pattern (`array[i, 0]`) that fails silently when the data structure changes to B (1D array) but the condition that was supposed to gate the indexing (`if vra_mode:`) still evaluates to True. The gate and the assumption were coupled when the code was written; they decouple during refactoring.

**Domain:** Any system where a flag controls both (a) a branching condition and (b) an assumption about data layout. When the data layout changes, the flag no longer tracks that change.

**Structural solution:** Derive indexing from the data itself (`ndim`), not from a flag. `array.ndim == 2` is always true when the array is 2D, regardless of what flags say.

**Status:** SOLVED
**Proved by:** All population calculations in recursive_bisection.py use `vertex_weights.ndim`
**Test:** `tests/unit/test_vra_edge_weighting.py::TestVRACodePath::test_vertex_weights_remain_1d`

---

## AP-05: Degenerate input to an algorithm designed for N≥2

**Pattern:** An algorithm designed for a specific domain (bisection requires at least 2 districts) receives input outside that domain (1 district) and either crashes, produces degenerate output, or silently succeeds with wrong results. Callers assume the algorithm handles all cases.

**Domain:** Any recursive or iterative algorithm with a minimum meaningful input size. Single-district states in redistricting have no bisection to perform; any bisection algorithm applied to them is operating outside its designed range.

**Structural solution:** Explicit domain check at the entry point, before any algorithm logic runs. Return early with a documented no-op when input is outside the domain.

**Status:** SOLVED
**Proved by:** VRA block begins with `if num_districts == 1: return to edge-weighted`
**Test:** `tests/integration/test_vra_pipeline_balance.py::TestVRACodePathIntegrity`

---

## AP-06: Implicit partition equality assumption in recursive bisection

**Pattern:** A bisection algorithm defaults to equal-weight partitioning (50/50) when splitting a node. This is correct when the node needs to produce two equal-sized children. It is wrong when the tree structure requires unequal children — for example, splitting 7 districts into [4, 3] requires 57%/43%, not 50%/50%. The algorithm produces a plausible-looking 2-way split every time, but only achieves population balance across the final districts when the target weights match the tree structure.

**Domain:** Any recursive bisection algorithm where the two halves are not always equal. Non-power-of-two district counts (3, 5, 6, 7, 9, ...) require unequal splits at some levels of the tree. The error is invisible until the final population balance check reveals systematic imbalance (12-75% deviation observed for k=7).

**Why it's hard to catch:** METIS accepts the call and produces a valid 2-way partition. The partition looks geometrically reasonable. Only the population balance assertion (applied after all bisection levels complete) reveals that the accumulated imbalance is unconstitutional. The connection between "missing target weights at depth 0" and "12% district imbalance" is not obvious.

**Structural solution:** Every call to a 2-way partitioner must explicitly pass the target partition weights derived from the bisection tree node's k_left/k_right ratio. The partitioner function signature should require target weights as a parameter (not optional), forcing callers to compute them explicitly. Store k_left and k_right in BisectionNode so callers always have the correct ratio.

**Status:** SOLVED in bisection_runner.rs
**Proved by:** Alabama VRA with k=7 passes ±0.5% population balance; tpwgts file written with correct ratios
**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_al_rust_population_balance`

## AP-07: Per-depth tolerance in recursive bisection causes compounding balance error

**Pattern:** A recursive bisection algorithm applies a fixed imbalance tolerance at each split level, assuming that the per-split tolerance equals the final district-level tolerance. In reality, balance errors accumulate across levels: a 5% tolerance per split across 7 rounds produces a worst-case cumulative error of ~35%. The deeper the bisection tree, the worse the final imbalance — even when each individual split appears to succeed.

**Domain:** Any recursive bisection algorithm where the depth of the tree varies with the number of output partitions. Non-power-of-two partition counts require unequal depths for different branches. Arises because the bisection literature focuses on power-of-two cases where each branch has the same depth, making the accumulation symmetric and bounded.

**Why it's hard to catch:** Each individual split passes its local balance check. The accumulated error only appears in the final constitutional validation. At small partition counts (k ≤ 10 for congressional districts), the accumulation is small enough that it never triggers the ±0.5% constitutional limit. At large state legislative counts (k ≥ 80), the accumulated error can exceed 20%.

**Structural solution:** Compute the allowed per-split tolerance at each bisection node from the final target tolerance and the number of districts the node produces: `node_ufactor = 1 + T / k_node`. This guarantees cumulative error ≤ T regardless of tree depth. The formula tightens tolerance at the root (k_node = k, very tight) and loosens it at leaves (k_node = 2, T/2). Pass the final balance tolerance, not a per-split ufactor, to the bisection function.

**Status:** SOLVED
**Proved by:** FL 120HD now passes 10% balance check after fix (was failing at 16.1% deviation)
**Test:** `redist-cli::bisection_runner::tests::test_per_node_ufactor_formula`, `test_run_all_splits_tight_balance_10pct`

---

## AP-08: Granularity floor constraint in geographic unit-based partitioning

**Pattern:** A partitioning algorithm is asked to balance populations to within T% of the ideal, but the geographic units (tracts, block groups) are too coarse: the average unit contains more than T% of the ideal district population. In this regime, no algorithm — regardless of parameters — can achieve the target balance, because a single unit swap changes the partition by more than the entire allowed tolerance.

**Domain:** Geographic redistricting at any resolution where the number of units per district is small (< 20). Arises when the number of districts is large relative to the total number of available geographic units. Congressional districts (10 per state, 178 tracts each) never hit this limit; state legislative chambers (98 districts, 18 tracts each) frequently do.

**Why it's hard to catch:** The algorithm runs to completion and produces a partition. The partition fails the final balance check, and the error appears to be a parameter problem (wrong ufactor, wrong tolerance). The actual cause — insufficient granularity — is not surfaced in the error message. Practitioners spend time tuning parameters instead of switching resolution.

**Structural solution:** Before attempting partitioning, compute `units_per_district = total_units / k` and `unit_size_as_pct = (total_pop / total_units) / (total_pop / k) * 100`. If `unit_size_as_pct > balance_tolerance`, emit an early error: "Cannot achieve {T}% balance with {U} units/district at tract resolution — use block_group or block." This converts a silent run-to-failure into an actionable early diagnostic.

**Status:** MITIGATED
**Proved by:** Block group resolution (54 units/district for WA 98HD) achieves 10% balance where tract (18/district) cannot. Validation script identifies states needing BG resolution.
**Test:** `scripts/pipeline/validate_state_legislative.py` detects granularity failures; `state_policy.json` documents low_density_note for MT/ND
