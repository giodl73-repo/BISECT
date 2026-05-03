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
> We ported it to Rust to obtain formal safety guarantees. The population balance
> constraint is not merely tested — it is a machine-verified postcondition on the
> partition function."*

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
    contracts.rs         output quality matches C METIS on known graphs
    shadow.rs            parallel-run comparison harness (used during migration)
```

## Core types

### `CsrGraph`

The single graph representation throughout the crate. Replaces `SubGraph` in
`redist-apportion`. Same memory layout as C METIS's internal representation.

```rust
pub struct CsrGraph {
    pub xadj:   Vec<u32>,         // xadj[i]..xadj[i+1] = neighbor range for vertex i
    pub adjncy: Vec<u32>,         // neighbor IDs, flat
    pub vwgt:   Vec<i32>,         // vertex weights (population)
    pub adjwgt: Option<Vec<i32>>, // edge weights (boundary length); None = unweighted
}
```

Invariant (checked by `is_valid()`):
- `xadj.len() == n + 1`
- `xadj[0] == 0`, `xadj` is non-decreasing
- all `adjncy[j] < n`
- all `vwgt[i] > 0`
- if `adjwgt` is `Some`, `adjwgt.len() == adjncy.len()`

### `Partition` and `CoarseMap`

```rust
pub struct Partition { pub assignment: Vec<u32>, pub k: u32 }
pub struct CoarseMap  { pub cmap: Vec<u32> }   // cmap[fine_v] = coarse_v
```

### `CoarseningHierarchy`

Arena owns all coarsened graphs. No pointer chains. Indices are stable.

```rust
pub struct CoarseningHierarchy {
    levels: Vec<CsrGraph>,  // [0] = original … [n] = coarsest
    cmaps:  Vec<CoarseMap>, // cmap[i] maps level[i+1] → level[i]
}
```

### Typestate pipeline

Illegal phase transitions are compile errors.

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
    fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap);
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
    fn split_weighted(&self, g: &CsrGraph, fracs: &[f32], seed: Option<u64>)
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

Concrete implementations per trait:

| Trait | Implementations |
|-------|----------------|
| `Coarsener` | `HeavyEdgeMatch`, `SortedHeavyEdgeMatch` *(default)*, `MinDegreeMatch` |
| `InitialPartitioner` | `GrowBisect` *(default)*, `RandomBisect`, `MultiConstraintInit` |
| `Refiner` | `FiducciaMattheyses` *(default)*, `GreedyKWay` |

## Algorithm phases

### Phase 1 — Coarsening

Repeatedly collapse the graph by matching vertices until `should_stop` returns
true (default: ≤ 20×k vertices). Each call adds one level to the hierarchy.

`SortedHeavyEdgeMatch` (default): sort vertices by max incident edge weight, then
greedily match each unmatched vertex with its heaviest-edge unmatched neighbor.
O(m log m).

### Phase 2 — Initial partition

Applied once at the coarsest level (tens to hundreds of vertices).

`GrowBisect`: seed two regions, expand greedily by best cut gain. For k > 2:
recursive bisection until k parts reached, or direct k-way seeding.

`MultiConstraintInit`: handles multiple simultaneous weight objectives — required
for multi-constraint partitioning.

### Phase 3 — FM refinement

`FmState` owns `assignment`, `GainTable` (bucket sort, O(1) max), and
`BoundarySet`. Inner loop: pick highest-gain boundary vertex, move it, update
neighbor gains. Rollback to best checkpoint after budget exhausted.

`GreedyKWay` extends FM to k parts: each boundary vertex may move to any
adjacent part, not just the complement.

### Phase 4 — Projection

`project_up`: expand coarse partition to finer level via `CoarseMap`:
`fine[v] = coarse[cmap[v]]`. Then `refine`. Repeat from coarsest to level 0.

## Verification

### Kani — safety floor

One harness per module. Proves (for all valid inputs up to bounded size):
- No buffer overflows on `xadj`/`adjncy` access
- No integer overflow in gain arithmetic or weight accumulation
- No index out of bounds on partition assignment or coarse map lookup
- No reachable panics (`unwrap`, `expect`, array index)

Runs in CI on every commit. Bounded to graphs of ≤ 16–64 vertices per harness
(sufficient to cover all code paths).

### Prusti — three legal postconditions

Applied to `Pipeline<Complete>::into_partition` and `Partitioner::split`:

```
#[ensures(result.assignment.len() == graph.n())]
#[ensures(forall |i: usize| i < result.assignment.len() ==> result.assignment[i] < result.k)]
#[ensures(population_balance(&result, graph) <= params.epsilon)]
```

| Postcondition | Legal meaning |
|---------------|---------------|
| Full coverage | No census tract omitted from any district |
| Valid part IDs | No phantom or out-of-range districts |
| Population balance ≤ ε | One-person-one-vote compliance, machine-verified |

Contiguity verified via a separate Kani reachability harness on the output graph.

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
2. **Validation gate**: shadow mode must pass 50-state × 3-year run with Rust
   quality ≥ C METIS quality (cut ≤ C cut, balance ≤ 0.5%). Gate is automated.
3. **Cutover**: remove `cfg` flag, `MetisPartitioner` becomes the Rust type alias.
4. **C dep removal**: remove `metis` from `redist-apportion/Cargo.toml` and
   workspace `Cargo.toml`. No C compiler required to build the project.

## Open questions

None. All design decisions resolved during brainstorming session 2026-05-02.

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
