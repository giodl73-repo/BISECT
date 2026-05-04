# redist-metis: Pure Rust METIS Port
**Date:** 2026-05-02
**Status:** Design approved — pending implementation plan
**Replaces:** `metis = { version = "0.2", features = ["vendored"] }` (C FFI)
**New crate:** `redist/crates/redist-metis/`

## Why this exists

The project currently depends on `metis-rs`, a thin Rust wrapper around the C METIS
library (Karypis et al., 1995). The C library is unverified: it carries no formal
safety guarantees, has platform-specific floating-point behavior, and cannot be
audited end-to-end for legal purposes. Every call into METIS crosses an `unsafe`
FFI boundary — Rust's safety guarantees stop there.

`redist-metis` is a faithful pure-Rust port of the same algorithm. Nothing about
the algorithm changes. The multilevel paradigm, the FM refinement, the population
balance constraint, the contiguity enforcement — all identical to C METIS. What
changes is how it is built: pure Rust, formally verified, bit-reproducible across
all platforms.

In a deposition:
> *"We run the same METIS algorithm the redistricting community has used since 1995.
> We ported it to Rust to obtain formal safety guarantees. For all inputs satisfying
> the documented preconditions — which are checked at graph construction and
> machine-verified by the Kani harness — the population balance constraint is a
> formally proven postcondition on the partition function, not merely a tested one."*

## Scope

### In scope

1. **Full METIS public API** — recursive bisection (`part_recursive`), k-way
   partitioning (`part_kway`), and multi-constraint partitioning. All three paths
   used or planned in the pipeline.

2. **Deterministic-within-Rust output** — bit-reproducible across all platforms
   and compiler versions. Does not match C METIS numerically (not a goal). A
   stronger property than C METIS provides today.

3. **Kani safety proofs** — one harness per module proving absence of buffer
   overflows, integer overflow, index-out-of-bounds, and unreachable panics.

4. **Three Prusti postconditions** on the public partition output:
   - Every vertex is assigned (full coverage)
   - All part IDs are in `[0, k)` (no phantom districts)
   - Population balance ≤ ε (one-person-one-vote, machine-verified)

5. **Shadow mode migration** — `MetisPartitioner` (C) and `RustMetisPartitioner`
   run in parallel during validation. C dep dropped after shadow mode passes.

6. **`ALGORITHM.md`** — mathematical specification readable by a court expert
   without opening a code editor.

### Out of scope

- Matching C METIS output numerically (not achievable without unsafe hacks)
- ParMETIS (distributed memory parallel) — not used in this pipeline
- Fortran wrappers — not used
- GPU variants
- Full Prusti functional verification of FM internals (too stateful for today's
  tools; the three postconditions on *output* are sufficient)

## Architecture

### Workspace integration

```
redist-core          (pure Rust, unchanged)
    ↑
redist-metis         (NEW — pure Rust METIS port, no FFI)
    ↑
redist-apportion     (drops C metis dep, gains redist-metis)
    ↑
redist-cli           (unchanged)
```

`redist-apportion/Cargo.toml`: replace `metis = { version = "0.2", features = ["vendored"] }`
with `redist-metis = { path = "../redist-metis" }`.

Workspace `Cargo.toml`: remove the `metis` workspace dependency.

### Crate layout

```
crates/redist-metis/
  ALGORITHM.md           mathematical spec (court/academic audience)
  Cargo.toml
  src/
    lib.rs               public re-exports
    graph/
      mod.rs
      csr.rs             CsrGraph — the single graph representation
    coarsen/
      mod.rs             Coarsener trait
      hem.rs             HeavyEdgeMatch
      shem.rs            SortedHeavyEdgeMatch (default)
      mindegree.rs       MinDegreeMatch
    init/
      mod.rs             InitialPartitioner trait
      grow.rs            GrowBisect (default for k=2)
      random.rs          RandomBisect
      multiconstraint.rs MultiConstraintInit
    refine/
      mod.rs             Refiner trait
      fm.rs              FiducciaMattheyses (default)
      gain.rs            GainTable — bucket sort, O(1) max lookup
      boundary.rs        BoundarySet
      kway.rs            GreedyKWay (k-way extension of FM)
    multilevel/
      mod.rs
      hierarchy.rs       CoarseningHierarchy — arena owns all levels
      pipeline.rs        Typestate pipeline (NeedsPartition → NeedsRefinement → Complete)
    api.rs               Partitioner trait + RustMetisPartitioner + MetisPartitioner alias
    error.rs             PartitionError
  verify/
    kani/                one harness per module
    prusti/              postcondition annotations
  benches/               criterion benchmarks vs C METIS quality baseline
  tests/
    contracts.rs         correctness oracle — known-optimal graphs + golden RNG value
    proptest.rs          property-based tests — random CsrGraph → is_valid() after coarsening
    shadow.rs            parallel-run quality comparison harness (used during migration)
  tests/golden/
    vt_seed42.json       golden partition vector for RNG determinism pin
```

## Core types

### `CsrGraph`

The single graph representation throughout the crate. Replaces `SubGraph` in
`redist-apportion`. Same memory layout as C METIS's internal representation.

```rust
pub struct CsrGraph {
    pub xadj:   Vec<u32>,         // xadj[i]..xadj[i+1] = neighbor range for vertex i
    pub adjncy: Vec<u32>,         // neighbor IDs, flat
    pub ncon:   u32,              // number of weight constraints (1 = scalar, default)
    pub vwgt:   Vec<i32>,         // ncon × n flat; vwgt[i*ncon + c] = weight of vertex i
                                  //   for constraint c. For ncon=1: vwgt[i] = population.
    pub adjwgt: Option<Vec<i32>>, // edge weights (boundary length); None = unweighted
}
```

Invariant (checked by `is_valid()`):
- `xadj.len() == n + 1`
- `xadj[0] == 0`, `xadj` is non-decreasing
- all `adjncy[j] < n`
- **no self-loops**: `adjncy[j] != i` for all j in `xadj[i]..xadj[i+1]` (PP-01)
- **connected**: the graph has exactly one connected component (PP-05);
  checked via BFS from vertex 0 — if any vertex is unreachable, `is_valid()` returns false
- `ncon >= 1`
- `vwgt.len() == n * ncon as usize`
- all `vwgt[i] > 0`
- if `adjwgt` is `Some`, `adjwgt.len() == adjncy.len()`

**Multi-constraint**: when `ncon > 1`, each vertex carries `ncon` weights (e.g., total
population + voting-age population). `MultiConstraintInit` reads all `ncon` weight
vectors; single-constraint phases read only `vwgt[i*ncon + 0]`. The flat layout
matches C METIS's internal representation exactly — no conversion needed.

**Coarsened vertex weight overflow (PP-02)**: during coarsening, matched vertices
merge weights: `coarse_vwgt[c] = fine_vwgt[v1] + fine_vwgt[v2]`. Accumulation
uses `i64` checked arithmetic. If the result exceeds `i32::MAX`, the coarsener
returns `Err(PartitionError::WeightOverflow)`. The Kani harness for
`coarsen/shem.rs` verifies no silent overflow for all inputs within the stated
bounds. For census-realistic inputs (max tract population ~40k, max 9129 tracts
for California), the maximum possible coarsened weight is ~40k × 9129 ≈ 365M —
well within `i32::MAX` (2.1B). The check is a safety net for adversarial inputs.

### `Partition` and `CoarseMap`

```rust
pub struct Partition { pub assignment: Vec<u32>, pub k: u32 }
pub struct CoarseMap  { pub cmap: Vec<u32> }   // cmap[fine_v] = coarse_v
```

**`CoarseMap` formal contract**:
- `cmap.len() == fine_graph.n()` — every fine vertex has a mapping
- `cmap[v] < coarse_graph.n()` for all v — all targets are valid coarse vertices
- Surjective: every coarse vertex `c` is the target of at least one fine vertex
  (no orphan coarse vertices; guaranteed by the matching algorithm)
- Not injective: multiple fine vertices may map to the same coarse vertex
  (this is the point — matched pairs collapse to one)
- `From<SubGraph> for CsrGraph`: `impl From<&SubGraph> for CsrGraph` provides
  the one-line conversion at `PfrCompositor` call sites.

### `CoarseningHierarchy`

Arena owns all coarsened graphs. No pointer chains. Indices are stable.

```rust
pub struct CoarseningHierarchy {
    levels: Vec<CsrGraph>,  // [0] = original … [n] = coarsest
    cmaps:  Vec<CoarseMap>, // cmap[i] maps level[i+1] → level[i]
}
```

### Typestate pipeline

Illegal phase transitions are compile errors. The state markers carry evidence of
what invariant holds, not just a label:

```rust
pub struct NeedsPartition  { pub levels_built: usize }   // how many coarsening rounds completed
pub struct NeedsRefinement { pub k: u32, pub coarsest_n: usize } // partition params locked in
pub struct Complete;        // refined and projected — invariants verified
```

This means a `Pipeline<NeedsRefinement>` knows at compile time how many parts were
requested and how small the coarsest level is — useful for verification annotations
and for `assert!` in debug mode. The enhanced form adds no runtime overhead
(zero-sized types are optimized away by the compiler).

A future enhancement (P3, not required for v1) would carry formal invariant
witnesses using Prusti's `pure` functions. This is deferred.

```rust
pub struct NeedsPartition;   // coarsening complete
pub struct NeedsRefinement;  // initial partition assigned at coarsest level
pub struct Complete;         // refined and projected back to level 0

pub struct Pipeline<S> {
    hierarchy: CoarseningHierarchy,
    partition: Option<Partition>,
    _state: PhantomData<S>,
}

impl Pipeline<NeedsPartition>  { pub fn initial_partition(self, p: &InitParams)  -> Pipeline<NeedsRefinement> }
impl Pipeline<NeedsRefinement> { pub fn refine_and_project(self, p: &RefineParams) -> Pipeline<Complete>       }
impl Pipeline<Complete>        { pub fn into_partition(self) -> Partition                                       }
```

## Trait architecture

### Internal phase traits

```rust
pub trait Coarsener: Send + Sync {
    /// Collapse `g` by one level. Returns coarsened graph + the mapping.
    /// **Requires**: `g.is_valid()`, `g.n() >= 2`.
    /// **Guarantees**: returned graph has strictly fewer vertices than `g`;
    ///   `cmap.len() == g.n()`; all CoarseMap contracts hold.
    fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap);

    /// True when the graph is small enough to partition directly.
    /// **Contract**: must eventually return `true` as `g.n()` decreases —
    ///   i.e., `should_stop` returns `true` for all graphs with `n <= MIN_COARSE`
    ///   where `MIN_COARSE` is implementation-defined but ≥ 2×k.
    ///   The multilevel orchestrator enforces a hard cap of `MAX_LEVELS = 50`
    ///   coarsening rounds regardless of `should_stop`. If the cap is hit,
    ///   the orchestrator returns `Err(PartitionError::CoarseningStalled)`
    ///   — never silently proceeds with an under-coarsened graph (PP-04).
    fn should_stop(&self, g: &CsrGraph) -> bool;
}

pub trait InitialPartitioner: Send + Sync {
    fn partition(&self, g: &CsrGraph, k: u32, seed: u64) -> Partition;
}

pub trait Refiner: Send + Sync {
    fn refine(&self, g: &CsrGraph, p: Partition) -> Partition;
}
```

### Public API trait

Replaces `redist-apportion::split::Partitioner`. The graph type changes from
`&SubGraph` to `&CsrGraph` — `PfrCompositor` call sites each need a one-line
`CsrGraph::from(subgraph)` conversion. No logic changes.

```rust
pub trait Partitioner: Send + Sync {
    fn split(&self, g: &CsrGraph, k: u32, seed: Option<u64>)
        -> Result<Partition, PartitionError>;
    /// Split into `fracs.len()` parts with proportional population targets.
    /// `fracs[i]` is a proportional integer weight — the target population
    /// fraction for part i is `fracs[i] / fracs.iter().sum()`. Integer weights
    /// eliminate float arithmetic in the partition path, preserving the
    /// determinism guarantee. The caller (PfrCompositor) converts rational
    /// prime-factor ratios to integer weights before calling.
    fn split_weighted(&self, g: &CsrGraph, fracs: &[u32], seed: Option<u64>)
        -> Result<Partition, PartitionError>;
}
```

### Composed implementation

```rust
pub struct RustMetisPartitioner<C, I, R> {
    coarsener: C,
    init:      I,
    refiner:   R,
    params:    MetisParams,  // ufactor, niter, seed, coarsen_to
}

// Standard METIS configuration — what redist-apportion uses
pub type MetisPartitioner = RustMetisPartitioner<
    SortedHeavyEdgeMatch,
    GrowBisect,
    FiducciaMattheyses,
>;
```

### Determinism and RNG

Determinism-within-Rust is the core legal reproducibility claim. All randomness
flows through a single RNG instance constructed from the user-supplied seed:

**Crate**: `rand_pcg::Pcg64` (from the `rand` ecosystem). PCG64 is:
- Deterministic: identical output for identical seed on all platforms
- Reproducible across compiler versions (pure integer arithmetic)
- Fast: ~2× faster than `rand::StdRng` for the access pattern used here
- No platform-specific behaviour (unlike `SmallRng`)

**Seed threading**: the seed is set once in `MetisParams`, passed into
`RustMetisPartitioner`, and threaded through all phases via explicit `&mut Rng`
parameters — no global RNG state, no thread-local state. This means:

```rust
pub struct MetisParams {
    pub ufactor:    u32,           // balance tolerance × 1000 (default 5 → 0.5%)
    pub niter:      u32,           // FM refinement iterations (default 10)
    pub seed:       Option<u64>,   // None = fixed default; Some(s) = Pcg64 seeded with s
    pub coarsen_to: u32,           // stop coarsening when n ≤ max(coarsen_to × k, 40) (default 20)
}
```

`seed = None` uses a fixed platform-independent constant (not `rand::thread_rng()`).
`seed = Some(0)` seeds Pcg64 with 0 — different from `None`. Using `Option<u64>`
eliminates the PP-03 ambiguity where `0` could mean either "default" or "seed zero."
The same `MetisParams` on the same graph always produces the same partition.

Concrete implementations per trait:

| Trait | Implementations |
|-------|----------------|
| `Coarsener` | `HeavyEdgeMatch`, `SortedHeavyEdgeMatch` *(default)*, `MinDegreeMatch` |
| `InitialPartitioner` | `GrowBisect` *(default)*, `RandomBisect`, `MultiConstraintInit` |
| `Refiner` | `FiducciaMattheyses` *(default)*, `GreedyKWay` |

## Algorithm phases

### Phase 1 — Coarsening

Repeatedly collapse the graph by matching vertices until `should_stop` returns
true (default: `n ≤ max(20×k, 40)`; the absolute floor of 40 prevents
over-coarsening on small states like Vermont). Each call adds one level to the
hierarchy.

`SortedHeavyEdgeMatch` (default): sort vertices into buckets by max incident edge
weight (bucket sort, O(n + m)), then greedily match each unmatched vertex with its
heaviest-edge unmatched neighbor. **O(n + m)**, not O(m log m) — the sort is a
bucket sort over the integer weight domain, not a comparison sort.

### Phase 2 — Initial partition

Applied once at the coarsest level (tens to hundreds of vertices).

**Two distinct algorithm entry points** (not a dispatch on k — they have
different internal paths throughout):

- **Recursive bisection** (`part_recursive` entry): `GrowBisect` seeds two
  regions and expands greedily. Recursively applied: the coarsest graph is
  bisected, each half coarsened and bisected again, until k single-district
  leaves are reached. Uses `GrowBisect` at every level of recursion.

- **Direct k-way** (`part_kway` entry): `GrowKway` seeds k regions simultaneously
  from k random vertices, expands in round-robin. Refinement uses `GreedyKWay`
  rather than repeated FM bisections. Faster for large k; lower quality than
  recursive bisection for k=2.

`MultiConstraintInit`: multi-objective variant of `GrowKway` that seeds and
expands while tracking balance across all `ncon` weight vectors simultaneously.
Required for multi-constraint partitioning.

### Phase 3 — FM refinement

`FmState` owns `assignment`, `GainTable` (bucket sort, O(1) max), and
`BoundarySet`. Inner loop: pick highest-gain boundary vertex, move it, update
neighbor gains.

**FM pass budget and rollback**: one FM pass processes each boundary vertex at
most once (O(n) moves per pass). The best partition seen during the pass is
checkpointed. After the pass, the state rolls back to the best checkpoint if the
final state is worse. Passes repeat up to `niter` times (default 10) or until
a full pass produces zero improvement — whichever comes first. "Budget exhausted"
means `niter` passes completed or convergence detected.

`GreedyKWay` extends FM to k parts: each boundary vertex may move to any
adjacent part, not just the complement.

**GainTable — negative gains**: gain values can be negative (moving a vertex
increases cut). The bucket array uses *offset indexing*: gains ∈ `[-max_gain,
+max_gain]` where `max_gain = max edge weight × max vertex degree`. Bucket index
= `gain + max_gain` (always non-negative). `max_gain` is computed once at
`FmState` construction and stored. Overflow is impossible: `i32` gain arithmetic
with `max_gain ≤ 2^28` fits within `i32` range for all realistic census graphs.
This is verified by the Kani `refine/gain.rs` harness at the stated bounds.

**Bounds-check strategy in FM inner loop**: the FM inner loop accesses
`adjncy[xadj[v]..xadj[v+1]]` for every neighbor of every moved vertex. This is
the hot path. Strategy: use `get_unchecked` with a surrounding `debug_assert!`
in the inner loop body, inside a single `unsafe` block scoped to the loop.
Correctness of the unsafe access is guaranteed by the `CsrGraph::is_valid()`
precondition (checked at graph construction) plus the Kani `graph/csr.rs` harness
which proves no out-of-bounds access for all valid `CsrGraph` inputs up to the
stated bound. This is documented in the unsafe block's comment and in
`verify/kani/UNSAFE.md`.

### Phase 4 — Projection

`project_up`: expand coarse partition to finer level via `CoarseMap`:
`fine[v] = coarse[cmap[v]]`. Then `refine`. Repeat from coarsest to level 0.

## Verification

### Kani — safety floor

**Version**: Kani 0.55+ (bounded model checker, AWS). Pinned in `rust-toolchain.toml`.

One harness per module. Proves (for all valid inputs up to bounded size):
- No buffer overflows on `xadj`/`adjncy` access
- No integer overflow in gain arithmetic or weight accumulation
- No index out of bounds on partition assignment or coarse map lookup
- No reachable panics (`unwrap`, `expect`, array index)

Runs in CI on every commit.

**Bound sizes and justification**:

| Module | Bound | Justification |
|--------|-------|---------------|
| `graph/csr.rs` | n ≤ 64 | All indexing paths exercised at n=4; 64 covers edge cases |
| `coarsen/shem.rs` | n ≤ 128 | Matching bugs in star/chain topologies require ≥ 100 vertices to surface |
| `coarsen/hem.rs` | n ≤ 128 | Same as SHEM |
| `init/grow.rs` | n ≤ 64, k ≤ 8 | Seed selection and expansion paths fully covered |
| `refine/fm.rs` | n ≤ 64, k ≤ 4 | FM inner loop is path-independent of n above ~16; 64 ensures gain-range coverage |
| `refine/gain.rs` | gains ∈ [-128, 128] | Full gain range coverage; bucket indexing tested at extremes |
| `multilevel/hierarchy.rs` | levels ≤ 8 | Covers ≥ 5 coarsening rounds (typical for California-scale graphs) |

Kani is a *bounded* model checker — it proves properties for all inputs up to the
stated bound, not for all possible inputs. The bounds above are chosen to cover all
distinct code paths (verified by inspecting LLVM bitcode coverage). The safety
properties (no UB, no overflow) are not input-size-dependent once all branches are
covered. This is documented in `verify/kani/BOUNDS.md` with coverage justification.

**`unsafe` accountability (PP-06)**: `verify/kani/UNSAFE.md` lists every `unsafe`
block in `src/` with its location, the invariant that justifies it, and the Kani
harness that covers it. A CI step counts `unsafe` occurrences via `grep -r "unsafe"
src/` and asserts the count matches `UNSAFE.md` entries. A new `unsafe` block
without a corresponding entry fails CI.

### Prusti — three legal postconditions

**Version**: Prusti 0.2.x (ETH Zurich, Viper backend). Pinned in `rust-toolchain.toml`
and `Cargo.toml` dev-dependency. Prusti supports a subset of Rust — verified
functions must avoid: trait objects (`dyn Trait`), closures, `async`, and complex
lifetime annotations. The public API functions subject to verification are written
to comply with this subset. Internal helper functions that Prusti cannot handle
fall back to Kani for safety properties only.

**Fallback policy**: if a function cannot be verified by Prusti (annotation failure
or unsupported feature), it is (a) flagged with `#[prusti_skip]`, (b) covered by
Kani safety harness instead, and (c) documented in `verify/prusti/GAPS.md` with
the reason. A CI step asserts that `GAPS.md` contains zero entries before release.

**Verification artifact archival (COVENANT)**: Prusti generates Viper intermediate
verification conditions (`.vpr` files) as part of its proof process. These are
committed to `verify/prusti/artifacts/` and are the legally-archivable proof
artifacts — not the CI pass/fail log, which expires. An opposing expert can
re-run Viper directly against the committed `.vpr` files to independently confirm
the postconditions without re-running the full Rust toolchain.

**Binary provenance (COVENANT)**: the released binary is built via a reproducible
build (`cargo build --locked` with a pinned `rust-toolchain.toml`). The
`rust-toolchain.toml` pins the exact Rust channel, version, and component set.
A SHA-256 of the compiled binary is published alongside each release tag. Any
party can reproduce the binary from the pinned source + toolchain and verify the
hash independently.

Applied to `Pipeline<Complete>::into_partition` and `Partitioner::split`:

```rust
#[ensures(result.assignment.len() == graph.n())]
#[ensures(forall(|i: usize| i < result.assignment.len()
    ==> result.assignment[i] < result.k))]
#[ensures(population_balance(&result, graph) <= params.epsilon)]
```

| Postcondition | Legal meaning |
|---------------|---------------|
| Full coverage | No census tract omitted from any district |
| Valid part IDs | No phantom or out-of-range districts |
| Population balance ≤ ε | One-person-one-vote compliance, machine-verified |

**Why output-only verification is sufficient for balance**: FM refinement may
produce intermediate states with temporary imbalance during the move sequence.
This is acceptable because: (a) the final postcondition is checked on the returned
`Partition`, not on intermediate states; (b) any violation that FM fails to correct
will be caught by the output postcondition and surface as a verification failure;
(c) the legal claim is about the *delivered* partition, not the internal path.

**Contiguity**: verified via a Kani BFS-witness harness on the output graph.
Each part `p` has a designated root vertex `r_p`; the harness proves that every
vertex in part `p` can reach `r_p` by traversing only edges within part `p`.
This is a bounded BFS (depth ≤ n) — the bound is the number of vertices in the
largest part, which is at most `n`.

## Performance

### Budget

`redist-metis` must stay within **20% of C METIS wall-clock time** on the
reference workload. The reference workload is:

| State | Year | k | Tract count | Mode |
|-------|------|---|-------------|------|
| Vermont | 2020 | 1 | 255 | bisection (trivial — used for smoke test) |
| Delaware | 2020 | 1 | 218 | bisection |
| California | 2020 | 53 | 9,129 | k-way |
| Texas | 2020 | 38 | 5,265 | k-way |

California k=53 is the bottleneck. A 20% wall-clock regression on this workload
blocks merge. The shadow-mode gate enforces this automatically.

### Benchmark suite

**Tool**: Criterion (Rust). Location: `benches/`.

| Benchmark | Input | Metric | Regression threshold |
|-----------|-------|--------|----------------------|
| `bench_vt_bisect` | VT 2020 (255 tracts, k=1) | wall-clock, p50 | > 2× baseline |
| `bench_ca_kway` | CA 2020 (9129 tracts, k=53) | wall-clock, p50 | > 20% vs C METIS |
| `bench_tx_kway` | TX 2020 (5265 tracts, k=38) | wall-clock, p50 | > 20% vs C METIS |
| `bench_ca_memory` | CA 2020 | peak RSS | > 2× C METIS |
| `bench_coarsen_only` | CA 2020 | coarsening time | — (profiling aid) |
| `bench_fm_only` | CA 2020, post-init | refinement time | — (profiling aid) |

**C METIS baseline** is recorded in `benches/baseline.json` during the shadow mode
validation run and committed. Criterion compares against this file.

## Documentation structure

Three levels, one algorithm:

| Level | Location | Audience |
|-------|----------|----------|
| Mathematical spec | `crates/redist-metis/ALGORITHM.md` | Court experts, academics |
| Implementation spec | Rustdoc on every public item | Rust developers |
| Formal spec | Prusti `#[ensures]` annotations | Formal verifiers, CI |

**Rule**: if it is not in `ALGORITHM.md`, it is not part of the algorithm. If it
is not in Rustdoc, it is not part of the API. If it is not in a Prusti annotation,
it is not formally guaranteed.

### `ALGORITHM.md` outline

The mathematical spec covers the following sections, in this order:

1. **Problem statement** — graph G=(V,E,w_v,w_e), partition P: V→{0…k-1},
   balance constraint ‖P‖ ≤ ε, objective: minimize Σ w_e for cut edges.
2. **Definitions** — CSR representation, edge cut, balance, coarse graph,
   matching, CoarseMap surjectivity.
3. **Multilevel paradigm** — informal description of the three-phase approach
   and why it works (hierarchy of approximations, projection correctness).
4. **Phase I: Coarsening** — HEM and SHEM algorithms with pseudocode;
   termination proof (graph strictly shrinks each round, `MAX_LEVELS` cap);
   quality theorem (SHEM preserves heavy edges across levels).
5. **Phase II: Initial partition** — GrowBisect algorithm with pseudocode;
   k>2 recursive bisection strategy; multi-constraint generalisation.
6. **Phase III: FM refinement** — FM algorithm with pseudocode; gain definition;
   bucket-sort structure; rollback strategy; convergence argument (FM terminates
   in O(n) moves per pass); k-way extension.
7. **Projection** — correctness of `project_up`; invariant: a valid partition
   at level i+1 projects to a valid partition at level i.
8. **Output invariants** — the three Prusti postconditions stated as theorems,
   with proof sketches. These are the legally-cited properties.
9. **Determinism** — proof that `Pcg64` with fixed seed produces identical output
   across all IEEE 754 platforms; why integer-only arithmetic in all phases
   eliminates platform-specific float behaviour; scope of the claim (valid inputs
   only, as defined by `is_valid()`); `split_weighted` uses integer proportional
   weights (`u32`), not `f32` fracs, to maintain the integer-only guarantee.
10. **Limitations** — what the algorithm does NOT guarantee: global optimality
    (NP-hard), uniqueness of output, matching C METIS output numerically.

Sub-specs (one per phase, each written before implementation of that phase):

- `2026-05-02-redist-metis-graph.md` — CsrGraph + CoarseMap contracts
- `2026-05-02-redist-metis-coarsen.md` — HEM/SHEM/MinDegree
- `2026-05-02-redist-metis-init.md` — GrowBisect/Random/MultiConstraint
- `2026-05-02-redist-metis-refine.md` — FM + GainTable + BoundarySet
- `2026-05-02-redist-metis-multilevel.md` — hierarchy + typestate pipeline
- `2026-05-02-redist-metis-verify.md` — Kani harnesses + Prusti contracts
- `2026-05-02-redist-metis-migration.md` — shadow mode cutover + C dep removal

## Migration path

1. **Shadow mode**: `cfg(feature = "shadow-metis")` in `redist-apportion` runs
   both C `MetisPartitioner` and `RustMetisPartitioner` on every split, logs
   diffs in cut quality and balance. No behavioral change for users. The C dep
   is retained only during this phase.
2. **Correctness oracle** (before shadow mode): verify the port against a small
   set of graphs with known-optimal partitions. These are input-size-independent
   correctness checks, distinct from the quality comparison in shadow mode:

   | Graph | k | Known optimal cut | Notes |
   |-------|---|-------------------|-------|
   | Path graph P_10 | 2 | 1 | Only one optimal bisection |
   | Complete bipartite K_{4,4} | 2 | 16 | All bisections equal |
   | Grid 4×4 | 4 | 4 | Each quadrant = 1 part |
   | Petersen graph | 5 | 5 | Known partition |
   | Any graph | 1 | 0 | Trivial: all vertices in part 0 |
   | Dumbbell (two K_5 joined by one edge) | 2 | 1 | Unique minimum cut through the bridge |
   | Weighted path: alternating heavy/light edges | 2 | lightest edge weight | Tests weight-sensitive bisection |

   These live in `tests/contracts.rs` and run as unit tests on every commit.

   **RNG golden-value test (BENCHMARK)**: one additional test pins the output of a
   California-scale bisection (255-vertex VT proxy graph, seed `Some(42)`) against
   a committed golden partition vector. If `rand_pcg` changes its output sequence
   across versions, this test fails loudly — protecting the determinism claim.
   The golden vector is generated once and committed to `tests/golden/vt_seed42.json`.

   **Property-based tests (BENCHMARK)**: `tests/proptest.rs` uses `proptest` to
   generate random valid `CsrGraph` instances (n ≤ 32, valid invariants), apply
   coarsening, and assert `is_valid()` holds on the output. Catches any coarsening
   bug that produces an invalid graph at arbitrary size beyond Kani bounds.

3. **Validation gate**: shadow mode must pass 50-state × 3-year run with Rust
   quality ≥ C METIS quality (avg cut ≤ C avg cut per state-year, balance ≤ 0.5%).
   Gate is automated. Performance within 20% budget (see Performance section).
   **Note (COVENANT)**: the legal correctness claim rests on the correctness oracle
   and the Prusti postconditions — not on the C METIS comparison. Shadow mode
   validates quality parity, not correctness. C METIS is itself an unverified binary
   and cannot serve as a legal correctness baseline.
4. **Cutover**: remove `cfg` flag, `MetisPartitioner` becomes the Rust type alias.
5. **C dep removal**: remove `metis` from `redist-apportion/Cargo.toml` and
   workspace `Cargo.toml`. No C compiler required to build the project.

## Open questions

None. All design decisions resolved.
- Brainstorming session: 2026-05-02
- Panel review (Lattner, Amin, Polikarpova, Solar-Lezama, Berger) P1–P3: addressed 2026-05-02
- Role review (MERIDIAN, COVENANT, BENCHMARK, TRENCH, DATUM): addressed 2026-05-02

## Effort estimate

| Phase | Scope | Estimate |
|-------|-------|----------|
| Crate scaffold + CsrGraph + traits | Setup | 1–2 days |
| Coarsening (HEM, SHEM, MinDegree) | Phase 1 | 3–4 days |
| Initial partition (Grow, Random, MultiConstraint) | Phase 2 | 2–3 days |
| FM refinement (GainTable, BoundarySet, FM, KWay) | Phase 3 | 5–7 days |
| Multilevel orchestration + typestate | Phase 4 | 2–3 days |
| Kani harnesses | Verification | 2–3 days |
| Prusti postconditions | Verification | 1–2 days |
| Shadow mode + validation + migration | Integration | 2–3 days |
| ALGORITHM.md + sub-specs | Documentation | 2–3 days |
| **Total** | | **~6–10 weeks** |
