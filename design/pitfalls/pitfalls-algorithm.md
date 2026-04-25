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
