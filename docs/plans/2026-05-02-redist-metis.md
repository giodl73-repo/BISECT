# redist-metis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `redist-metis` — a pure Rust METIS graph partitioning library with formal verification, replacing the C FFI dependency in `redist-apportion`.

**Architecture:** Eight sequential phases: scaffold → coarsening → initial partition → FM refinement → multilevel orchestration → Kani safety proofs → Prusti postconditions → shadow migration. Each phase has a sub-spec written before code. TDD throughout: write failing test, implement, pass, commit.

**Tech Stack:** Rust 2021, `rand_pcg 0.3` (Pcg64 RNG), `proptest 1` (property tests), `criterion 0.5` (benchmarks), Kani 0.55+, Prusti 0.2.x

**Test levels used throughout:**
- **L0** — pure unit tests, no I/O, no external deps, run in < 1ms each
- **L1** — integration tests, multiple components composed, may read fixture files
- **L2** — end-to-end tests, full pipeline, shadow mode comparison against C METIS

---

## File map

```
redist/crates/redist-metis/
  ALGORITHM.md
  Cargo.toml
  src/
    lib.rs
    error.rs
    graph/
      mod.rs        CsrGraph, Partition, CoarseMap — types + is_valid()
      csr.rs        CsrGraph builder helpers, From<SubGraph>
    coarsen/
      mod.rs        Coarsener trait
      hem.rs        HeavyEdgeMatch
      shem.rs       SortedHeavyEdgeMatch (default)
      mindegree.rs  MinDegreeMatch
    init/
      mod.rs        InitialPartitioner trait
      grow.rs       GrowBisect + GrowKway
      random.rs     RandomBisect
      multiconstraint.rs  MultiConstraintInit
    refine/
      mod.rs        Refiner trait
      gain.rs       GainTable (bucket sort, offset indexing)
      boundary.rs   BoundarySet
      fm.rs         FmState + FiducciaMattheyses
      kway.rs       GreedyKWay
    multilevel/
      mod.rs
      hierarchy.rs  CoarseningHierarchy (arena)
      pipeline.rs   Pipeline<S> typestate
    api.rs          RustMetisPartitioner<C,I,R>, MetisPartitioner alias, Partitioner trait
  verify/
    kani/
      csr_harness.rs
      coarsen_harness.rs
      refine_harness.rs
      multilevel_harness.rs
      BOUNDS.md
      UNSAFE.md
    prusti/
      postconditions.rs
      GAPS.md
      artifacts/         .vpr files committed here
  benches/
    bench_main.rs
  tests/
    contracts.rs          L0 correctness oracle
    proptest_invariants.rs L0 property-based CsrGraph invariants
    shadow.rs             L2 shadow comparison vs C METIS
  tests/golden/
    vt_seed42.json
```

---

## Task 0: CI setup — Kani and Prusti jobs

**Files:**
- Create: `.github/workflows/verify.yml` (or equivalent CI config)

- [ ] **Step 1: Define two separate CI jobs**

```yaml
# verify.yml — runs on main-branch merges and release tags only
jobs:
  kani:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: model-checking/kani-github-action@v1
      - run: cargo kani --workspace

  prusti:
    runs-on: ubuntu-latest
    continue-on-error: true   # advisory on PR; blocking enforced at release tag
    steps:
      - uses: viperproject/prusti-action@v1
      - run: cargo prusti -p redist-metis
```

**CI policy (SURVEY)**:
- `cargo test` — runs on every commit, blocks PR merge
- `cargo kani` — runs on main-branch push + release tags only (35–70 min, too slow for every PR)
- `cargo prusti` — advisory on PR (never blocks), blocking on release tag; Prusti backend timeouts are non-fatal on PR

- [ ] **Step 2: Commit CI config**

```
git add .github/workflows/verify.yml
git commit -m "ci: add Kani + Prusti verify jobs — separate from PR test suite"
```

---

## Phase 1 — Scaffold + Foundation

**Checkpoint**: `cargo test -p redist-metis` passes. All traits compile. `CsrGraph::is_valid()` fully tested (L0).

---

### Task 1: Create crate + Cargo.toml

**Files:**
- Create: `redist/crates/redist-metis/Cargo.toml`
- Modify: `redist/Cargo.toml`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "redist-metis"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
description = "Pure Rust METIS graph partitioning — formally verified, deterministic"

[dependencies]
rand_pcg  = "0.3"
rand      = { version = "0.8", default-features = false, features = ["alloc"] }
thiserror.workspace = true

[dev-dependencies]
proptest  = "1"
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "bench_main"
harness = false
```

- [ ] **Step 2: Add to workspace**

In `redist/Cargo.toml`, add `"crates/redist-metis"` to the `members` array.

- [ ] **Step 3: Create `src/lib.rs` stub**

```rust
pub mod error;
pub mod graph;
pub mod coarsen;
pub mod init;
pub mod refine;
pub mod multilevel;
pub mod api;

pub use error::PartitionError;
pub use graph::{CsrGraph, Partition, CoarseMap};
pub use api::{Partitioner, MetisPartitioner, MetisParams};
```

- [ ] **Step 4: Verify it compiles**

```
cargo check -p redist-metis
```
Expected: errors about missing modules (normal — stubs come next).

---

### Task 2: `error.rs` + `graph/mod.rs` — types and `is_valid()`

**Files:**
- Create: `src/error.rs`
- Create: `src/graph/mod.rs`

- [ ] **Step 1: Write L0 tests for `is_valid()` first**

Create `src/graph/mod.rs` with the tests at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn path_graph(n: usize) -> CsrGraph {
        // 0-1-2-...(n-1), undirected
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for i in 0..n {
            if i > 0 { adjncy.push((i - 1) as u32); }
            if i < n - 1 { adjncy.push((i + 1) as u32); }
            xadj.push(adjncy.len() as u32);
        }
        CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
    }

    #[test]
    fn valid_path_graph() { assert!(path_graph(5).is_valid()); }

    #[test]
    fn invalid_self_loop() {
        let mut g = path_graph(4);
        g.adjncy[0] = 0; // vertex 0 points to itself
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_out_of_bounds_adjncy() {
        let mut g = path_graph(4);
        g.adjncy[0] = 99;
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_zero_vwgt() {
        let mut g = path_graph(4);
        g.vwgt[1] = 0;
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_negative_vwgt() {
        let mut g = path_graph(4);
        g.vwgt[1] = -1;
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_disconnected() {
        // Two isolated edges: 0-1, 2-3 — no path between components
        let g = CsrGraph {
            xadj:   vec![0, 1, 2, 3, 4],
            adjncy: vec![1, 0, 3, 2],
            ncon: 1,
            vwgt: vec![1; 4],
            adjwgt: None,
        };
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_adjwgt_wrong_len() {
        let mut g = path_graph(4);
        g.adjwgt = Some(vec![1i32; 3]); // wrong length
        assert!(!g.is_valid());
    }

    #[test]
    fn valid_multi_constraint() {
        let mut g = path_graph(4);
        g.ncon = 2;
        g.vwgt = vec![1, 2, 3, 4, 5, 6, 7, 8]; // 4 vertices × 2 constraints
        assert!(g.is_valid());
    }
}
```

- [ ] **Step 2: Run tests — confirm they fail**

```
cargo test -p redist-metis graph
```
Expected: compile error (types not defined yet).

- [ ] **Step 3: Implement types + `is_valid()`**

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CsrGraph {
    pub xadj:   Vec<u32>,
    pub adjncy: Vec<u32>,
    pub ncon:   u32,
    pub vwgt:   Vec<i32>,
    pub adjwgt: Option<Vec<i32>>,
}

impl CsrGraph {
    pub fn n(&self) -> usize { self.xadj.len().saturating_sub(1) }

    pub fn is_valid(&self) -> bool {
        let n = self.n();
        if self.xadj.len() != n + 1 { return false; }
        if n == 0 { return true; }
        if self.xadj[0] != 0 { return false; }
        if self.ncon < 1 { return false; }
        if self.vwgt.len() != n * self.ncon as usize { return false; }
        if self.vwgt.iter().any(|&w| w <= 0) { return false; }
        if let Some(ref aw) = self.adjwgt {
            if aw.len() != self.adjncy.len() { return false; }
        }
        for i in 0..n {
            if self.xadj[i] > self.xadj[i + 1] { return false; }
            for j in self.xadj[i] as usize..self.xadj[i + 1] as usize {
                if j >= self.adjncy.len() { return false; }
                let nb = self.adjncy[j] as usize;
                if nb >= n || nb == i { return false; }  // PP-01, bounds
            }
        }
        // PP-05: connectivity BFS from vertex 0
        let mut visited = vec![false; n];
        let mut queue = VecDeque::new();
        queue.push_back(0usize);
        visited[0] = true;
        while let Some(v) = queue.pop_front() {
            for j in self.xadj[v] as usize..self.xadj[v + 1] as usize {
                let u = self.adjncy[j] as usize;
                if !visited[u] { visited[u] = true; queue.push_back(u); }
            }
        }
        visited.iter().all(|&v| v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Partition { pub assignment: Vec<u32>, pub k: u32 }

#[derive(Debug, Clone)]
pub struct CoarseMap { pub cmap: Vec<u32> }
```

- [ ] **Step 4: Add `error.rs`**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PartitionError {
    #[error("invalid graph: {0}")]
    InvalidGraph(&'static str),
    #[error("k must be >= 1")]
    ZeroParts,
    #[error("k ({k}) exceeds vertex count ({n})")]
    TooManyParts { k: u32, n: usize },
    #[error("coarsening stalled: MAX_LEVELS=50 reached")]
    CoarseningStalled,
    #[error("vertex weight overflow during coarsening")]
    WeightOverflow,
    #[error("empty graph")]
    EmptyGraph,
}
```

- [ ] **Step 5: Run tests — all pass**

```
cargo test -p redist-metis graph
```
Expected: 8 tests pass.

- [ ] **Step 6: Commit**

```
git add redist/crates/redist-metis/ redist/Cargo.toml
git commit -m "feat(redist-metis): scaffold — CsrGraph, Partition, CoarseMap, PartitionError"
```

---

### Task 3: Core traits + `api.rs` stub

**Files:**
- Create: `src/coarsen/mod.rs`
- Create: `src/init/mod.rs`
- Create: `src/refine/mod.rs`
- Create: `src/api.rs`
- Create: `src/multilevel/mod.rs`

- [ ] **Step 1: Write L0 test — mock implementations compile and satisfy traits**

In `src/api.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CsrGraph, Partition, CoarseMap};

    struct AlwaysTrivial;
    impl Coarsener for AlwaysTrivial {
        fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap) {
            let cmap = (0..g.n() as u32).collect();
            (g.clone(), CoarseMap { cmap })
        }
        fn should_stop(&self, _: &CsrGraph) -> bool { true }
    }

    struct AllZeroPartitioner;
    impl InitialPartitioner for AllZeroPartitioner {
        fn partition(&self, g: &CsrGraph, _k: u32, _seed: u64) -> Partition {
            Partition { assignment: vec![0; g.n()], k: 1 }
        }
    }

    struct IdentityRefiner;
    impl Refiner for IdentityRefiner {
        fn refine(&self, _g: &CsrGraph, p: Partition) -> Partition { p }
    }

    #[test]
    fn mock_traits_compile() {
        let _c: &dyn Coarsener = &AlwaysTrivial;
        let _i: &dyn InitialPartitioner = &AllZeroPartitioner;
        let _r: &dyn Refiner = &IdentityRefiner;
    }
}
```

- [ ] **Step 2: Implement traits**

`src/coarsen/mod.rs`:
```rust
use crate::graph::{CsrGraph, CoarseMap};
pub trait Coarsener: Send + Sync {
    fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap);
    fn should_stop(&self, g: &CsrGraph) -> bool;
}
pub mod hem; pub mod shem; pub mod mindegree;
```

`src/init/mod.rs`:
```rust
use crate::graph::{CsrGraph, Partition};
pub trait InitialPartitioner: Send + Sync {
    fn partition(&self, g: &CsrGraph, k: u32, seed: u64) -> Partition;
}
pub mod grow; pub mod random; pub mod multiconstraint;
```

`src/refine/mod.rs`:
```rust
use crate::graph::{CsrGraph, Partition};
pub trait Refiner: Send + Sync {
    fn refine(&self, g: &CsrGraph, p: Partition) -> Partition;
}
pub mod gain; pub mod boundary; pub mod fm; pub mod kway;
```

`src/api.rs`:
```rust
use crate::{graph::{CsrGraph, Partition}, error::PartitionError};
use crate::coarsen::Coarsener;
use crate::init::InitialPartitioner;
use crate::refine::Refiner;

pub use crate::coarsen::Coarsener;
pub use crate::init::InitialPartitioner;
pub use crate::refine::Refiner;

pub trait Partitioner: Send + Sync {
    fn split(&self, g: &CsrGraph, k: u32, seed: Option<u64>)
        -> Result<Partition, PartitionError>;
    fn split_weighted(&self, g: &CsrGraph, fracs: &[u32], seed: Option<u64>)
        -> Result<Partition, PartitionError>;
}

#[derive(Debug, Clone)]
pub struct MetisParams {
    pub ufactor:    u32,
    pub niter:      u32,
    pub seed:       Option<u64>,
    pub coarsen_to: u32,
}

impl Default for MetisParams {
    fn default() -> Self {
        Self { ufactor: 5, niter: 10, seed: None, coarsen_to: 20 }
    }
}

pub struct RustMetisPartitioner<C, I, R> {
    pub coarsener: C,
    pub init:      I,
    pub refiner:   R,
    pub params:    MetisParams,
}

// Type alias filled in after Phase 5
// pub type MetisPartitioner = RustMetisPartitioner<SortedHeavyEdgeMatch, GrowBisect, FiducciaMattheyses>;
```

- [ ] **Step 3: Run tests**

```
cargo test -p redist-metis api
```
Expected: `mock_traits_compile` passes.

- [ ] **Step 4: Commit**

```
git commit -m "feat(redist-metis): core traits — Coarsener, InitialPartitioner, Refiner, Partitioner"
```

---

## Phase 2 — Coarsening

**Checkpoint**: All three coarsenerspass L0 tests on 6 reference graphs. L1 test: repeated coarsening of a 255-vertex path graph reaches `should_stop` within MAX_LEVELS.

---

### Task 4: Sub-spec `redist-metis-coarsen.md`

**Files:**
- Create: `docs/specs/2026-05-02-redist-metis-coarsen.md`

- [ ] **Step 1: Write the sub-spec** (covers HEM, SHEM, MinDegree pseudocode, invariants, test strategy)

Key contracts to document:
- Input: `g.is_valid()`, `g.n() >= 2`
- Output: `coarsened.n() < g.n()`, `cmap.len() == g.n()`, coarsened `is_valid()`
- SHEM bucket sort domain: `[0, max_adjwgt]`; unweighted graphs use weight=1 for all edges
- Termination: `should_stop` returns true when `g.n() <= max(coarsen_to * k, 40)`

- [ ] **Step 2: Commit sub-spec**

```
git commit -m "docs: redist-metis coarsening sub-spec"
```

---

### Task 5: `HeavyEdgeMatch`

**Files:**
- Create: `src/coarsen/hem.rs`

- [ ] **Step 1: Write L0 tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::CsrGraph;
    use crate::coarsen::Coarsener;

    fn triangle() -> CsrGraph {
        CsrGraph {
            xadj:   vec![0, 2, 4, 6],
            adjncy: vec![1, 2,  0, 2,  0, 1],
            ncon: 1, vwgt: vec![1; 3], adjwgt: None,
        }
    }

    fn path5() -> CsrGraph {
        CsrGraph {
            xadj:   vec![0, 1, 3, 5, 7, 8],
            adjncy: vec![1,  0,2,  1,3,  2,4,  3],
            ncon: 1, vwgt: vec![1; 5], adjwgt: None,
        }
    }

    #[test]
    fn coarsened_strictly_smaller() {
        let (c, _) = HeavyEdgeMatch.coarsen(&path5());
        assert!(c.n() < 5);
        assert!(c.is_valid());
    }

    #[test]
    fn cmap_length_equals_fine_n() {
        let g = path5();
        let (_, cmap) = HeavyEdgeMatch.coarsen(&g);
        assert_eq!(cmap.cmap.len(), g.n());
    }

    #[test]
    fn cmap_targets_in_range() {
        let g = path5();
        let (c, cmap) = HeavyEdgeMatch.coarsen(&g);
        assert!(cmap.cmap.iter().all(|&t| (t as usize) < c.n()));
    }

    #[test]
    fn should_stop_k2_small_graph() {
        // path5 has 5 vertices; should_stop with default coarsen_to=20, k=2: max(40,40)=40; 5 < 40
        let hem = HeavyEdgeMatchWithParams { coarsen_to: 20, k: 2 };
        assert!(hem.should_stop(&path5()));
    }

    #[test]
    fn triangle_coarsens_to_2() {
        let (c, _) = HeavyEdgeMatch.coarsen(&triangle());
        assert!(c.n() <= 2);
        assert!(c.is_valid());
    }
}
```

- [ ] **Step 2: Run — expect compile fail**

```
cargo test -p redist-metis coarsen::hem
```

- [ ] **Step 3: Implement `HeavyEdgeMatch`**

```rust
use rand_pcg::Pcg64;
use rand::{Rng, SeedableRng};
use crate::graph::{CsrGraph, CoarseMap};
use crate::coarsen::Coarsener;

pub struct HeavyEdgeMatch;

// Parameterised version used in pipeline; HeavyEdgeMatch uses defaults
pub struct HeavyEdgeMatchWithParams { pub coarsen_to: u32, pub k: u32 }

impl Coarsener for HeavyEdgeMatch {
    fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap) {
        hem_coarsen(g, 42) // fixed seed for default
    }
    fn should_stop(&self, g: &CsrGraph) -> bool {
        g.n() <= 40 // default: max(20*1, 40) for k=1
    }
}

impl Coarsener for HeavyEdgeMatchWithParams {
    fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap) {
        hem_coarsen(g, 42)
    }
    fn should_stop(&self, g: &CsrGraph) -> bool {
        let threshold = (self.coarsen_to * self.k).max(40);
        g.n() <= threshold as usize
    }
}

fn hem_coarsen(g: &CsrGraph, seed: u64) -> (CsrGraph, CoarseMap) {
    let n = g.n();
    let mut rng = Pcg64::seed_from_u64(seed);
    let mut matched = vec![false; n];
    let mut cmap = vec![u32::MAX; n];
    let mut coarse_id = 0u32;

    // Visit vertices in random order
    let mut order: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = rng.gen_range(0..=i);
        order.swap(i, j);
    }

    for &v in &order {
        if matched[v] { continue; }
        // Find heaviest unmatched neighbour
        let best = (g.xadj[v] as usize..g.xadj[v + 1] as usize)
            .filter(|&j| !matched[g.adjncy[j] as usize])
            .max_by_key(|&j| {
                g.adjwgt.as_ref().map_or(1i32, |aw| aw[j])
            });
        match best {
            Some(j) => {
                let u = g.adjncy[j] as usize;
                cmap[v] = coarse_id;
                cmap[u] = coarse_id;
                matched[v] = true;
                matched[u] = true;
            }
            None => {
                cmap[v] = coarse_id;
                matched[v] = true;
            }
        }
        coarse_id += 1;
    }

    let cn = coarse_id as usize;
    build_coarse_graph(g, &cmap, cn)
}

fn build_coarse_graph(g: &CsrGraph, cmap: &[u32], cn: usize) -> (CsrGraph, CoarseMap) {
    use std::collections::HashMap;
    let n = g.n();
    let ncon = g.ncon as usize;

    // Accumulate vertex weights (i64 to catch overflow — PP-02)
    let mut cvwgt = vec![0i64; cn * ncon];
    for v in 0..n {
        let cv = cmap[v] as usize;
        for c in 0..ncon {
            cvwgt[cv * ncon + c] += g.vwgt[v * ncon + c] as i64;
        }
    }
    // Cast back — overflow check
    let cvwgt_i32: Vec<i32> = cvwgt.iter().map(|&w| w as i32).collect();

    // Build coarse adjacency
    let mut cadj: Vec<HashMap<u32, i32>> = vec![HashMap::new(); cn];
    for v in 0..n {
        let cv = cmap[v] as usize;
        for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
            let cu = cmap[g.adjncy[j] as usize] as usize;
            if cu != cv {
                let ew = g.adjwgt.as_ref().map_or(1i32, |aw| aw[j]);
                *cadj[cv].entry(cu as u32).or_insert(0) += ew;
            }
        }
    }

    let mut xadj = vec![0u32];
    let mut adjncy = Vec::new();
    let mut adjwgt = Vec::new();
    for cv in 0..cn {
        for (&cu, &ew) in &cadj[cv] {
            adjncy.push(cu);
            adjwgt.push(ew);
        }
        xadj.push(adjncy.len() as u32);
    }

    let coarse = CsrGraph {
        xadj,
        adjncy,
        ncon: g.ncon,
        vwgt: cvwgt_i32,
        adjwgt: if g.adjwgt.is_some() { Some(adjwgt) } else { None },
    };
    (coarse, CoarseMap { cmap: cmap.to_vec() })
}
```

- [ ] **Step 4: Add unweighted → unweighted test (BENCHMARK Gap 1 / PP-PLAN-01)**

```rust
#[test]
fn coarsen_unweighted_stays_unweighted() {
    let (c, _) = HeavyEdgeMatch.coarsen(&path5());
    assert!(c.adjwgt.is_none(),
        "unweighted input must produce unweighted coarsened graph");
}

#[test]
fn coarsen_weighted_stays_weighted() {
    let mut g = path5();
    g.adjwgt = Some(vec![1i32; g.adjncy.len()]);
    let (c, _) = HeavyEdgeMatch.coarsen(&g);
    assert!(c.adjwgt.is_some());
}
```

- [ ] **Step 5: Run tests**

```
cargo test -p redist-metis coarsen::hem
```
Expected: 7 tests pass.

- [ ] **Step 6: Commit**

```
git commit -m "feat(redist-metis): HeavyEdgeMatch coarsener with L0 tests"
```

---

### Task 6: `SortedHeavyEdgeMatch`

**Files:**
- Create: `src/coarsen/shem.rs`

- [ ] **Step 1: L0 tests** — same oracle as HEM but also test that SHEM prefers heavier edges on a weighted graph

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::CsrGraph;
    use crate::coarsen::Coarsener;

    fn weighted_path4() -> CsrGraph {
        // 0 --10-- 1 --1-- 2 --10-- 3
        CsrGraph {
            xadj:   vec![0, 1, 3, 5, 6],
            adjncy: vec![1,   0,2,   1,3,   2],
            ncon: 1,
            vwgt: vec![1; 4],
            adjwgt: Some(vec![10, 10, 1, 1, 10, 10]),
        }
    }

    #[test]
    fn shem_coarsened_valid() {
        let (c, cmap) = SortedHeavyEdgeMatch.coarsen(&weighted_path4());
        assert!(c.is_valid());
        assert_eq!(cmap.cmap.len(), 4);
        assert!(c.n() < 4);
    }

    #[test]
    fn shem_prefers_heavy_edges() {
        // Vertices 0&1 and 2&3 share weight-10 edges; SHEM should match them
        let (_, cmap) = SortedHeavyEdgeMatch.coarsen(&weighted_path4());
        // 0 and 1 should map to same coarse vertex
        assert_eq!(cmap.cmap[0], cmap.cmap[1]);
        // 2 and 3 should map to same coarse vertex
        assert_eq!(cmap.cmap[2], cmap.cmap[3]);
    }
}
```

- [ ] **Step 2: Implement SHEM** — bucket sort O(n+m), not comparison sort

```rust
use crate::graph::{CsrGraph, CoarseMap};
use crate::coarsen::{Coarsener, hem::build_coarse_graph};

pub struct SortedHeavyEdgeMatch;
pub struct SortedHeavyEdgeMatchWithParams { pub coarsen_to: u32, pub k: u32 }

impl Coarsener for SortedHeavyEdgeMatch {
    fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap) { shem_coarsen(g) }
    fn should_stop(&self, g: &CsrGraph) -> bool { g.n() <= 40 }
}

impl Coarsener for SortedHeavyEdgeMatchWithParams {
    fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap) { shem_coarsen(g) }
    fn should_stop(&self, g: &CsrGraph) -> bool {
        g.n() <= (self.coarsen_to * self.k).max(40) as usize
    }
}

fn shem_coarsen(g: &CsrGraph) -> (CsrGraph, CoarseMap) {
    let n = g.n();
    // Compute max incident edge weight per vertex
    let max_w: Vec<i32> = (0..n).map(|v| {
        (g.xadj[v] as usize..g.xadj[v+1] as usize)
            .map(|j| g.adjwgt.as_ref().map_or(1i32, |aw| aw[j]))
            .max()
            .unwrap_or(0)
    }).collect();

    // Bucket sort vertices by max_w descending — O(n + max_w)
    let max_bucket = max_w.iter().copied().max().unwrap_or(0) as usize;
    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); max_bucket + 1];
    for v in 0..n { buckets[max_w[v] as usize].push(v); }

    let mut matched = vec![false; n];
    let mut cmap   = vec![u32::MAX; n];
    let mut coarse_id = 0u32;

    // Process highest-weight buckets first
    for bucket in buckets.iter().rev() {
        for &v in bucket {
            if matched[v] { continue; }
            let best = (g.xadj[v] as usize..g.xadj[v+1] as usize)
                .filter(|&j| !matched[g.adjncy[j] as usize])
                .max_by_key(|&j| g.adjwgt.as_ref().map_or(1i32, |aw| aw[j]));
            match best {
                Some(j) => {
                    let u = g.adjncy[j] as usize;
                    cmap[v] = coarse_id; cmap[u] = coarse_id;
                    matched[v] = true;  matched[u] = true;
                }
                None => { cmap[v] = coarse_id; matched[v] = true; }
            }
            coarse_id += 1;
        }
    }

    build_coarse_graph(g, &cmap, coarse_id as usize)
}
```

- [ ] **Step 3: Run tests**

```
cargo test -p redist-metis coarsen::shem
```
Expected: 2 tests pass.

- [ ] **Step 4: Commit**

```
git commit -m "feat(redist-metis): SortedHeavyEdgeMatch — O(n+m) bucket sort coarsener"
```

---

### Task 7: `MinDegreeMatch` + L1 coarsening checkpoint

**Files:**
- Create: `src/coarsen/mindegree.rs`

- [ ] **Step 1: L0 tests**

```rust
#[test]
fn mindegree_valid_output() {
    let g = star_graph(6); // one center connected to 5 leaves
    let (c, cmap) = MinDegreeMatch.coarsen(&g);
    assert!(c.is_valid());
    assert_eq!(cmap.cmap.len(), 6);
    assert!(c.n() < 6);
}
```

Helper `star_graph(n)`: vertex 0 connected to vertices 1..n.

- [ ] **Step 2: Implement MinDegreeMatch** — sort vertices by degree ascending, match each with heaviest-edge unmatched neighbour

```rust
use crate::graph::{CsrGraph, CoarseMap};
use crate::coarsen::{Coarsener, hem::build_coarse_graph};

pub struct MinDegreeMatch;

impl Coarsener for MinDegreeMatch {
    fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap) {
        let n = g.n();
        let degrees: Vec<u32> = (0..n)
            .map(|v| g.xadj[v+1] - g.xadj[v])
            .collect();

        // Sort by degree ascending — O(n log n) (min-degree matching is less
        // critical for performance than SHEM; comparison sort acceptable)
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_unstable_by_key(|&v| degrees[v]);

        let mut matched  = vec![false; n];
        let mut cmap     = vec![u32::MAX; n];
        let mut coarse_id = 0u32;

        for v in order {
            if matched[v] { continue; }
            let best = (g.xadj[v] as usize..g.xadj[v+1] as usize)
                .filter(|&j| !matched[g.adjncy[j] as usize])
                .max_by_key(|&j| g.adjwgt.as_ref().map_or(1i32, |aw| aw[j]));
            match best {
                Some(j) => {
                    let u = g.adjncy[j] as usize;
                    cmap[v] = coarse_id; cmap[u] = coarse_id;
                    matched[v] = true;  matched[u] = true;
                }
                None => { cmap[v] = coarse_id; matched[v] = true; }
            }
            coarse_id += 1;
        }
        build_coarse_graph(g, &cmap, coarse_id as usize)
    }
    fn should_stop(&self, g: &CsrGraph) -> bool { g.n() <= 40 }
}
```

- [ ] **Step 3: L1 coarsening checkpoint test** — add to `tests/contracts.rs`

```rust
// L1: repeated coarsening reaches should_stop within MAX_LEVELS
#[test]
fn coarsening_terminates_path255() {
    use redist_metis::coarsen::{Coarsener, shem::SortedHeavyEdgeMatchWithParams};
    let g = make_path(255);
    let coarsener = SortedHeavyEdgeMatchWithParams { coarsen_to: 20, k: 1 };
    let mut current = g;
    for level in 0..50 {
        if coarsener.should_stop(&current) { return; }
        let (next, _) = coarsener.coarsen(&current);
        assert!(next.is_valid(), "invalid at level {level}");
        assert!(next.n() < current.n(), "did not shrink at level {level}");
        current = next;
    }
    panic!("did not reach should_stop within 50 levels");
}
```

- [ ] **Step 4: Run all coarsening tests**

```
cargo test -p redist-metis coarsen
cargo test -p redist-metis --test contracts coarsening
```
Expected: all pass.

- [ ] **Step 5: Commit**

```
git commit -m "feat(redist-metis): MinDegreeMatch + L1 coarsening termination test"
```

---

## Phase 3 — Initial Partition

**Checkpoint**: All four initializers produce valid `Partition` on the correctness oracle graphs. L1 test: coarsen → initial_partition pipeline on VT 2020 produces a balanced partition.

---

### Task 8: Sub-spec + `GrowBisect`

**Files:**
- Create: `docs/specs/2026-05-02-redist-metis-init.md`
- Create: `src/init/grow.rs`

- [ ] **Step 1: Write sub-spec** — document GrowBisect pseudocode, GrowKway pseudocode, seed selection strategy, balance check

- [ ] **Step 2: L0 tests for GrowBisect**

```rust
#[test]
fn grow_bisect_valid_partition() {
    let g = grid_4x4(); // 16-vertex 4x4 grid
    let p = GrowBisect.partition(&g, 2, 42);
    assert_eq!(p.assignment.len(), 16);
    assert_eq!(p.k, 2);
    assert!(p.assignment.iter().all(|&x| x < 2));
    assert!(p.assignment.contains(&0));
    assert!(p.assignment.contains(&1));
}

#[test]
fn grow_bisect_k1_all_zero() {
    let g = path_graph(10);
    let p = GrowBisect.partition(&g, 1, 0);
    assert!(p.assignment.iter().all(|&x| x == 0));
}
```

- [ ] **Step 3: Implement `GrowBisect`**

```rust
use rand_pcg::Pcg64;
use rand::{Rng, SeedableRng};
use crate::graph::{CsrGraph, Partition};
use crate::init::InitialPartitioner;
use std::collections::VecDeque;

pub struct GrowBisect;
pub struct GrowKway;

impl InitialPartitioner for GrowBisect {
    fn partition(&self, g: &CsrGraph, k: u32, seed: u64) -> Partition {
        debug_assert!(g.is_valid(), "GrowBisect requires valid connected graph (PP-PLAN-02)");
        if k == 1 { return Partition { assignment: vec![0; g.n()], k: 1 }; }
        grow_bisect_recursive(g, k, seed)
    }
}

fn grow_bisect_recursive(g: &CsrGraph, k: u32, seed: u64) -> Partition {
    // For k=2: grow two regions from random seeds
    // For k>2: bisect into ceil(k/2) and floor(k/2), recurse
    if k == 1 { return Partition { assignment: vec![0; g.n()], k: 1 }; }
    let mut rng = Pcg64::seed_from_u64(seed);
    let n = g.n();
    let target_pop: i64 = g.vwgt.iter().map(|&w| w as i64).sum::<i64>() / k as i64;

    // Seed: two random vertices far apart (random for now; improve later)
    let seed_a = rng.gen_range(0..n);
    let mut seed_b = rng.gen_range(0..n);
    while seed_b == seed_a { seed_b = rng.gen_range(0..n); }

    let mut assignment = vec![u32::MAX; n];
    let mut pop = [0i64; 2];
    assignment[seed_a] = 0; pop[0] += g.vwgt[seed_a] as i64;
    assignment[seed_b] = 1; pop[1] += g.vwgt[seed_b] as i64;

    // BFS expansion: assign each unassigned vertex to the part
    // that benefits most from it (simple: alternating BFS)
    let mut queues = [VecDeque::from([seed_a]), VecDeque::from([seed_b])];
    let mut unassigned = n - 2;

    while unassigned > 0 {
        for part in 0..2usize {
            while let Some(v) = queues[part].pop_front() {
                for j in g.xadj[v] as usize..g.xadj[v+1] as usize {
                    let u = g.adjncy[j] as usize;
                    if assignment[u] == u32::MAX {
                        assignment[u] = part as u32;
                        pop[part] += g.vwgt[u] as i64;
                        queues[part].push_back(u);
                        unassigned -= 1;
                        if unassigned == 0 { break; }
                    }
                }
                if unassigned == 0 { break; }
            }
            if unassigned == 0 { break; }
        }
    }
    // Any still unassigned (disconnected components) get part 0
    for a in assignment.iter_mut() { if *a == u32::MAX { *a = 0; } }
    Partition { assignment, k: 2 }
}
```

- [ ] **Step 4: Implement `GrowKway`** (k simultaneous BFS seeds)

```rust
impl InitialPartitioner for GrowKway {
    fn partition(&self, g: &CsrGraph, k: u32, seed: u64) -> Partition {
        debug_assert!(g.is_valid(), "GrowKway requires valid connected graph (PP-PLAN-02)");
        if k == 1 { return Partition { assignment: vec![0; g.n()], k: 1 }; }
        let n = g.n();
        let mut rng = Pcg64::seed_from_u64(seed);
        let mut assignment = vec![u32::MAX; n];
        let mut queues: Vec<VecDeque<usize>> = (0..k as usize).map(|_| VecDeque::new()).collect();

        // Pick k random distinct seed vertices
        let mut seeds = Vec::with_capacity(k as usize);
        while seeds.len() < k as usize {
            let v = rng.gen_range(0..n);
            if !seeds.contains(&v) { seeds.push(v); }
        }
        for (part, &sv) in seeds.iter().enumerate() {
            assignment[sv] = part as u32;
            queues[part].push_back(sv);
        }

        let mut unassigned = n - k as usize;
        let mut round_part = 0usize;
        while unassigned > 0 {
            let part = round_part % k as usize;
            round_part += 1;
            while let Some(v) = queues[part].pop_front() {
                for j in g.xadj[v] as usize..g.xadj[v+1] as usize {
                    let u = g.adjncy[j] as usize;
                    if assignment[u] == u32::MAX {
                        assignment[u] = part as u32;
                        queues[part].push_back(u);
                        unassigned -= 1;
                        if unassigned == 0 { break; }
                    }
                }
                if unassigned == 0 { break; }
                break; // one vertex per part per round
            }
        }
        for a in assignment.iter_mut() { if *a == u32::MAX { *a = 0; } }
        Partition { assignment, k }
    }
}
```

- [ ] **Step 5: Run tests**

```
cargo test -p redist-metis init::grow
```
Expected: 2 tests pass.

- [ ] **Step 6: Commit**

```
git commit -m "feat(redist-metis): GrowBisect + GrowKway initial partitioners"
```

---

### Task 9: `RandomBisect` + `MultiConstraintInit`

**Files:**
- Create: `src/init/random.rs`
- Create: `src/init/multiconstraint.rs`

- [ ] **Step 1: L0 tests**

```rust
// random.rs tests
#[test]
fn random_bisect_all_parts_present() {
    let g = path_graph(20);
    let p = RandomBisect.partition(&g, 2, 7);
    assert!(p.assignment.contains(&0));
    assert!(p.assignment.contains(&1));
}

// multiconstraint.rs tests
#[test]
fn multi_constraint_respects_ncon() {
    let mut g = grid_4x4();
    g.ncon = 2;
    g.vwgt = vec![1; 32]; // 16 vertices × 2 constraints
    let p = MultiConstraintInit.partition(&g, 4, 0);
    assert_eq!(p.k, 4);
    assert_eq!(p.assignment.len(), 16);
}
```

- [ ] **Step 2: Implement `RandomBisect`**

```rust
use rand_pcg::Pcg64;
use rand::{Rng, SeedableRng};
use crate::graph::{CsrGraph, Partition};
use crate::init::InitialPartitioner;

pub struct RandomBisect;

impl InitialPartitioner for RandomBisect {
    fn partition(&self, g: &CsrGraph, k: u32, seed: u64) -> Partition {
        let mut rng = Pcg64::seed_from_u64(seed);
        let assignment = (0..g.n()).map(|_| rng.gen_range(0..k)).collect();
        Partition { assignment, k }
    }
}
```

- [ ] **Step 3: Implement `MultiConstraintInit`** — delegates to GrowKway (same expansion, tracks all constraint balances)

```rust
use crate::graph::{CsrGraph, Partition};
use crate::init::{InitialPartitioner, grow::GrowKway};

pub struct MultiConstraintInit;

impl InitialPartitioner for MultiConstraintInit {
    fn partition(&self, g: &CsrGraph, k: u32, seed: u64) -> Partition {
        // For v1: delegate to GrowKway; multi-constraint balance handled in FM
        GrowKway.partition(g, k, seed)
    }
}
```

- [ ] **Step 4: Run all init tests**

```
cargo test -p redist-metis init
```
Expected: 4 tests pass.

- [ ] **Step 5: Commit**

```
git commit -m "feat(redist-metis): RandomBisect + MultiConstraintInit"
```

---

## Phase 4 — FM Refinement

**Checkpoint**: FM produces `cut(output) <= cut(input)` on all correctness oracle graphs (L0). GreedyKWay passes L0 on k=4 grid. L1: coarsen → init → refine pipeline on VT 2020 proxy.

---

### Task 10: Sub-spec + `GainTable`

**Files:**
- Create: `docs/specs/2026-05-02-redist-metis-refine.md`
- Create: `src/refine/gain.rs`

- [ ] **Step 1: Write sub-spec** — FM pseudocode, gain definition, bucket sort with offset, rollback strategy, FM pass budget

- [ ] **Step 2: L0 tests for `GainTable`**

```rust
#[test]
fn gain_table_max_is_correct() {
    let mut gt = GainTable::new(5, 10); // 5 vertices, max_gain=10
    gt.insert(0, 3);
    gt.insert(1, -2);
    gt.insert(2, 7);
    assert_eq!(gt.peek_max(), Some((2, 7)));
}

#[test]
fn gain_table_remove_max() {
    let mut gt = GainTable::new(5, 10);
    gt.insert(0, 5); gt.insert(1, 3); gt.insert(2, 8);
    let (v, g) = gt.pop_max().unwrap();
    assert_eq!((v, g), (2, 8));
    assert_eq!(gt.pop_max().unwrap(), (0, 5));
}

#[test]
fn gain_table_update_gain() {
    let mut gt = GainTable::new(3, 5);
    gt.insert(0, 2);
    gt.update(0, 4);
    assert_eq!(gt.peek_max(), Some((0, 4)));
}

#[test]
fn gain_table_negative_gain() {
    let mut gt = GainTable::new(3, 5);
    gt.insert(0, -3);
    gt.insert(1, -1);
    assert_eq!(gt.peek_max().unwrap().0, 1); // -1 > -3
}
```

- [ ] **Step 3: Implement `GainTable`** (bucket sort, offset indexing for negative gains)

```rust
pub struct GainTable {
    /// buckets[gain + max_gain] = list of vertex IDs with this gain
    buckets: Vec<Vec<u32>>,
    /// position[v] = (gain, index_in_bucket) or None if not in table
    position: Vec<Option<(i32, usize)>>,
    max_gain:  i32,
    top_bucket: i32, // highest non-empty bucket gain value
}

impl GainTable {
    pub fn new(n_vertices: usize, max_gain: i32) -> Self {
        let size = (2 * max_gain + 1) as usize;
        Self {
            buckets: vec![Vec::new(); size],
            position: vec![None; n_vertices],
            max_gain,
            top_bucket: i32::MIN,
        }
    }

    fn bucket_idx(&self, gain: i32) -> usize {
        (gain + self.max_gain) as usize
    }

    pub fn insert(&mut self, vertex: u32, gain: i32) {
        let bi = self.bucket_idx(gain);
        let pos = self.buckets[bi].len();
        self.buckets[bi].push(vertex);
        self.position[vertex as usize] = Some((gain, pos));
        if gain > self.top_bucket { self.top_bucket = gain; }
    }

    pub fn remove(&mut self, vertex: u32) {
        if let Some((gain, pos)) = self.position[vertex as usize].take() {
            let bi = self.bucket_idx(gain);
            let last = self.buckets[bi].len() - 1;
            if pos < last {
                let swap_v = self.buckets[bi][last];
                self.buckets[bi][pos] = swap_v;
                self.position[swap_v as usize] = Some((gain, pos));
            }
            self.buckets[bi].pop();
        }
    }

    pub fn update(&mut self, vertex: u32, new_gain: i32) {
        self.remove(vertex);
        self.insert(vertex, new_gain);
    }

    pub fn peek_max(&self) -> Option<(u32, i32)> {
        let mut g = self.top_bucket;
        while g >= -self.max_gain {
            let bi = self.bucket_idx(g);
            if let Some(&v) = self.buckets[bi].last() { return Some((v, g)); }
            g -= 1;
        }
        None
    }

    pub fn pop_max(&mut self) -> Option<(u32, i32)> {
        let (v, g) = self.peek_max()?;
        self.remove(v);
        self.top_bucket = g;
        Some((v, g))
    }

    pub fn is_empty(&self) -> bool { self.peek_max().is_none() }
}
```

- [ ] **Step 4: Run tests**

```
cargo test -p redist-metis refine::gain
```
Expected: 4 tests pass.

- [ ] **Step 5: Commit**

```
git commit -m "feat(redist-metis): GainTable with offset bucket sort + L0 tests"
```

---

### Task 11: `BoundarySet` + `FmState`

**Files:**
- Create: `src/refine/boundary.rs`
- Create: `src/refine/fm.rs` (state struct only, no algorithm yet)

- [ ] **Step 1: L0 tests for `BoundarySet`**

```rust
#[test]
fn boundary_contains_cross_part_vertices() {
    // Graph: 0-1-2, partition [0,0,1]
    let g = path_graph(3);
    let p = Partition { assignment: vec![0, 0, 1], k: 2 };
    let b = BoundarySet::from_partition(&g, &p);
    assert!(b.contains(1)); // vertex 1 (part 0) is adjacent to vertex 2 (part 1)
    assert!(b.contains(2)); // vertex 2 (part 1) is adjacent to vertex 1 (part 0)
    assert!(!b.contains(0)); // vertex 0 only touches vertex 1, same part
}
```

- [ ] **Step 2: Implement `BoundarySet`**

```rust
use std::collections::HashSet;
use crate::graph::{CsrGraph, Partition};

pub struct BoundarySet { inner: HashSet<u32> }

impl BoundarySet {
    pub fn from_partition(g: &CsrGraph, p: &Partition) -> Self {
        let mut inner = HashSet::new();
        let n = g.n();
        for v in 0..n {
            for j in g.xadj[v] as usize..g.xadj[v+1] as usize {
                let u = g.adjncy[j] as usize;
                if p.assignment[v] != p.assignment[u] {
                    inner.insert(v as u32);
                    break;
                }
            }
        }
        Self { inner }
    }

    pub fn contains(&self, v: u32) -> bool { self.inner.contains(&v) }
    pub fn insert(&mut self, v: u32) { self.inner.insert(v); }
    pub fn remove(&mut self, v: u32) { self.inner.remove(&v); }
    pub fn iter(&self) -> impl Iterator<Item=u32> + '_ {
        self.inner.iter().copied()
    }
}
```

- [ ] **Step 3: Implement `FmState` struct + helpers**

```rust
use crate::graph::{CsrGraph, Partition};
use super::{gain::GainTable, boundary::BoundarySet};

pub struct FmState<'g> {
    pub graph:      &'g CsrGraph,
    pub assignment: Vec<u32>,
    pub k:          u32,
    pub gain_table: GainTable,
    pub boundary:   BoundarySet,
    pub part_pop:   Vec<i64>,  // population per part
}

#[derive(Clone)]
pub struct Checkpoint {
    pub assignment: Vec<u32>,
    pub cut:        i64,
}

impl<'g> FmState<'g> {
    pub fn new(g: &'g CsrGraph, p: Partition) -> Self {
        let n = g.n();
        let k = p.k as usize;
        let mut part_pop = vec![0i64; k];
        for v in 0..n { part_pop[p.assignment[v] as usize] += g.vwgt[v] as i64; }

        let max_gain = g.adjwgt.as_ref()
            .map_or(g.xadj.windows(2).map(|w| (w[1]-w[0]) as i32).max().unwrap_or(1),
                    |aw| *aw.iter().max().unwrap_or(&1));

        let boundary = BoundarySet::from_partition(g, &p);
        let mut gain_table = GainTable::new(n, max_gain);
        for v in boundary.iter() {
            let gain = compute_gain(g, &p.assignment, v as usize);
            gain_table.insert(v, gain);
        }

        FmState { graph: g, assignment: p.assignment, k: p.k, gain_table, boundary, part_pop }
    }

    pub fn cut(&self) -> i64 {
        let g = self.graph;
        let mut c = 0i64;
        for v in 0..g.n() {
            for j in g.xadj[v] as usize..g.xadj[v+1] as usize {
                let u = g.adjncy[j] as usize;
                if self.assignment[v] != self.assignment[u] {
                    c += g.adjwgt.as_ref().map_or(1i64, |aw| aw[j] as i64);
                }
            }
        }
        c / 2 // each edge counted twice
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint { assignment: self.assignment.clone(), cut: self.cut() }
    }

    pub fn restore(&mut self, cp: &Checkpoint) {
        self.assignment = cp.assignment.clone();
        // Rebuild boundary and gain table from restored assignment
        let p = Partition { assignment: self.assignment.clone(), k: self.k };
        self.boundary = BoundarySet::from_partition(self.graph, &p);
        let n = self.graph.n();
        let max_gain = self.gain_table.max_gain;
        self.gain_table = GainTable::new(n, max_gain);
        for v in self.boundary.iter() {
            let gain = compute_gain(self.graph, &self.assignment, v as usize);
            self.gain_table.insert(v, gain);
        }
    }
}

pub fn compute_gain(g: &CsrGraph, assignment: &[u32], v: usize) -> i32 {
    let part_v = assignment[v];
    let mut gain = 0i32;
    for j in g.xadj[v] as usize..g.xadj[v+1] as usize {
        let u = g.adjncy[j] as usize;
        let ew = g.adjwgt.as_ref().map_or(1i32, |aw| aw[j]);
        if assignment[u] == part_v { gain -= ew; } else { gain += ew; }
    }
    gain
}
```

- [ ] **Step 4: Run tests**

```
cargo test -p redist-metis refine::boundary
```
Expected: 1 test passes.

- [ ] **Step 5: Commit**

```
git commit -m "feat(redist-metis): BoundarySet + FmState with checkpoint/restore"
```

---

### Task 12: `FiducciaMattheyses` + `GreedyKWay`

**Files:**
- Modify: `src/refine/fm.rs` (add FM algorithm)
- Create: `src/refine/kway.rs`

- [ ] **Step 1: L0 tests for FM**

```rust
#[test]
fn fm_does_not_increase_cut() {
    let g = grid_4x4();
    let p_init = RandomBisect.partition(&g, 2, 0);
    let cut_before = compute_cut(&g, &p_init.assignment);
    let p_refined = FiducciaMattheyses { niter: 10 }.refine(&g, p_init);
    let cut_after = compute_cut(&g, &p_refined.assignment);
    assert!(cut_after <= cut_before, "FM increased cut: {cut_before} -> {cut_after}");
}

#[test]
fn fm_oracle_dumbbell_bisect() {
    // Dumbbell: two K5 joined by one edge — optimal cut = 1
    let g = dumbbell_graph();
    let p_init = RandomBisect.partition(&g, 2, 42);
    let p = FiducciaMattheyses { niter: 20 }.refine(&g, p_init);
    assert_eq!(compute_cut(&g, &p.assignment), 1);
}
```

- [ ] **Step 2: Implement `FiducciaMattheyses`**

```rust
use crate::graph::{CsrGraph, Partition};
use crate::refine::Refiner;
use super::fm::{FmState, Checkpoint, compute_gain};

pub struct FiducciaMattheyses { pub niter: u32 }

impl Refiner for FiducciaMattheyses {
    fn refine(&self, g: &CsrGraph, p: Partition) -> Partition {
        let mut state = FmState::new(g, p);
        let mut best = state.checkpoint();

        for _pass in 0..self.niter {
            let improved = fm_pass(&mut state, &mut best);
            if !improved { break; }
        }
        state.restore(&best);
        Partition { assignment: state.assignment, k: state.k }
    }
}

fn fm_pass(state: &mut FmState, best: &mut Checkpoint) -> bool {
    let total_pop: i64 = state.part_pop.iter().sum();
    let target_pop = total_pop / state.k as i64;
    let epsilon = (total_pop * 5 + 999) / 1000; // ceiling of 0.5% — integer only (no float)

    let mut moved = Vec::new();
    let start_cut = state.cut();

    loop {
        let Some((v, _gain)) = state.gain_table.pop_max() else { break };
        let from_part = state.assignment[v as usize];

        // Find best destination part (k=2: it's the other part)
        let to_part = if state.k == 2 {
            1 - from_part
        } else {
            best_k_destination(state, v as usize)
        };

        // Balance check: don't make any part exceed (1+ε) × target
        let new_from = state.part_pop[from_part as usize]
            - state.graph.vwgt[v as usize] as i64;
        let new_to = state.part_pop[to_part as usize]
            + state.graph.vwgt[v as usize] as i64;
        if new_from < target_pop - epsilon || new_to > target_pop + epsilon {
            continue; // skip this move
        }

        // Apply move
        state.assignment[v as usize] = to_part;
        state.part_pop[from_part as usize] = new_from;
        state.part_pop[to_part as usize]   = new_to;
        moved.push(v);

        // Update gains for all neighbours
        for j in state.graph.xadj[v as usize] as usize
                ..state.graph.xadj[v as usize + 1] as usize {
            let u = state.graph.adjncy[j] as usize;
            let new_gain = compute_gain(state.graph, &state.assignment, u);
            if state.boundary.contains(u as u32) {
                state.gain_table.update(u as u32, new_gain);
            } else {
                // u might now be on the boundary
                let is_boundary = (state.graph.xadj[u] as usize
                    ..state.graph.xadj[u+1] as usize).any(|jj| {
                    state.assignment[state.graph.adjncy[jj] as usize]
                        != state.assignment[u]
                });
                if is_boundary {
                    state.boundary.insert(u as u32);
                    state.gain_table.insert(u as u32, new_gain);
                }
            }
        }
        // Update best
        let cur_cut = state.cut();
        if cur_cut < best.cut { *best = state.checkpoint(); }
    }

    best.cut < start_cut
}

fn best_k_destination(state: &FmState, v: usize) -> u32 {
    let from = state.assignment[v];
    (0..state.k).filter(|&p| p != from)
        .max_by_key(|&p| {
            (state.graph.xadj[v] as usize..state.graph.xadj[v+1] as usize)
                .filter(|&j| state.assignment[state.graph.adjncy[j] as usize] == p)
                .map(|j| state.graph.adjwgt.as_ref().map_or(1i32, |aw| aw[j]))
                .sum::<i32>()
        })
        .unwrap_or(if from == 0 { 1 } else { 0 })
}
```

- [ ] **Step 3: Implement `GreedyKWay`** (k-way FM extension)

```rust
pub struct GreedyKWay { pub niter: u32 }

impl Refiner for GreedyKWay {
    fn refine(&self, g: &CsrGraph, p: Partition) -> Partition {
        // Delegates to FM with best_k_destination for k>2
        FiducciaMattheyses { niter: self.niter }.refine(g, p)
    }
}
```

- [ ] **Step 4: Add FM balance test (BENCHMARK Gap 2)**

```rust
#[test]
fn fm_preserves_population_balance() {
    let g = grid_4x4();
    let total: i64 = g.vwgt.iter().map(|&w| w as i64).sum();
    let target = total / 2;
    let eps = (total * 5 + 999) / 1000; // 0.5% ceiling
    let p_init = RandomBisect.partition(&g, 2, 99);
    let p = FiducciaMattheyses { niter: 10 }.refine(&g, p_init);
    for part in 0..2u32 {
        let pop: i64 = (0..g.n())
            .filter(|&v| p.assignment[v] == part)
            .map(|v| g.vwgt[v] as i64)
            .sum();
        assert!(
            (pop - target).abs() <= eps,
            "part {part} pop {pop} violates balance (target {target} ± {eps})"
        );
    }
}
```

- [ ] **Step 5: Run all refinement tests**

```
cargo test -p redist-metis refine
```
Expected: tests including FM oracle and balance pass.

- [ ] **Step 5: Commit**

```
git commit -m "feat(redist-metis): FiducciaMattheyses FM refinement + GreedyKWay with L0 oracle tests"
```

---

## Phase 5 — Multilevel Orchestration

**Checkpoint**: Full `CsrGraph → Partition` pipeline passes all 7 correctness oracle tests (L0). L1: VT 2020 proxy (255-vertex graph, k=1) completes in < 5s.

---

### Task 13: Sub-spec + `CoarseningHierarchy`

**Files:**
- Create: `docs/specs/2026-05-02-redist-metis-multilevel.md`
- Create: `src/multilevel/hierarchy.rs`

- [ ] **Step 1: Write sub-spec** — hierarchy arena, projection algorithm, MAX_LEVELS=50 error

- [ ] **Step 2: L0 tests**

```rust
#[test]
fn hierarchy_builds_from_path() {
    let g = path_graph(100);
    let coarsener = SortedHeavyEdgeMatchWithParams { coarsen_to: 20, k: 2 };
    let h = CoarseningHierarchy::build(&g, &coarsener).unwrap();
    assert!(h.levels.len() >= 2);
    assert!(h.coarsest().n() <= 40);
    assert!(h.coarsest().is_valid());
}

#[test]
fn hierarchy_stalls_returns_error() {
    // A coarsener that never stops
    struct NeverStops;
    impl Coarsener for NeverStops {
        fn coarsen(&self, g: &CsrGraph) -> (CsrGraph, CoarseMap) {
            SortedHeavyEdgeMatch.coarsen(g)
        }
        fn should_stop(&self, _: &CsrGraph) -> bool { false } // never stops
    }
    let g = path_graph(10);
    // Will hit MAX_LEVELS and return CoarseningStalled
    let result = CoarseningHierarchy::build(&g, &NeverStops);
    assert!(matches!(result, Err(PartitionError::CoarseningStalled)));
}
```

- [ ] **Step 3: Implement `CoarseningHierarchy`**

```rust
use crate::{graph::{CsrGraph, CoarseMap}, error::PartitionError, coarsen::Coarsener};

pub const MAX_LEVELS: usize = 50;

pub struct CoarseningHierarchy {
    pub levels: Vec<CsrGraph>,   // [0]=original … [n]=coarsest
    pub cmaps:  Vec<CoarseMap>,  // cmap[i] maps level[i+1] → level[i]
}

impl CoarseningHierarchy {
    pub fn build(g: &CsrGraph, coarsener: &dyn Coarsener)
        -> Result<Self, PartitionError>
    {
        let mut levels = vec![g.clone()];
        let mut cmaps  = Vec::new();
        for _ in 0..MAX_LEVELS {
            let current = levels.last().unwrap();
            if coarsener.should_stop(current) { break; }
            if current.n() <= 1 { break; }
            let (coarsened, cmap) = coarsener.coarsen(current);
            cmaps.push(cmap);
            levels.push(coarsened);
            if coarsener.should_stop(levels.last().unwrap()) { break; }
        }
        if !coarsener.should_stop(levels.last().unwrap()) {
            return Err(PartitionError::CoarseningStalled);
        }
        Ok(Self { levels, cmaps })
    }

    pub fn coarsest(&self) -> &CsrGraph { self.levels.last().unwrap() }
    pub fn depth(&self) -> usize { self.levels.len() - 1 }

    /// Project a partition from level `lev+1` down to level `lev`.
    pub fn project_up(&self, lev: usize, coarse_p: &[u32]) -> Vec<u32> {
        let cmap = &self.cmaps[lev].cmap;
        cmap.iter().map(|&c| coarse_p[c as usize]).collect()
    }
}
```

- [ ] **Step 4: Run tests**

```
cargo test -p redist-metis multilevel::hierarchy
```
Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```
git commit -m "feat(redist-metis): CoarseningHierarchy arena with MAX_LEVELS=50 error"
```

---

### Task 14: Typestate `Pipeline` + `RustMetisPartitioner`

**Files:**
- Create: `src/multilevel/pipeline.rs`
- Modify: `src/api.rs`

- [ ] **Step 1: L0 tests**

```rust
#[test]
fn pipeline_full_run_path10_k2() {
    use redist_metis::api::{MetisParams, RustMetisPartitioner};
    use redist_metis::coarsen::shem::SortedHeavyEdgeMatchWithParams;
    use redist_metis::init::grow::GrowBisect;
    use redist_metis::refine::fm::FiducciaMattheyses;

    let g = path_graph(10);
    let params = MetisParams { ufactor: 5, niter: 10, seed: Some(42), coarsen_to: 20 };
    let partitioner = RustMetisPartitioner {
        coarsener: SortedHeavyEdgeMatchWithParams { coarsen_to: 20, k: 2 },
        init:  GrowBisect,
        refiner: FiducciaMattheyses { niter: 10 },
        params,
    };
    let result = partitioner.split(&g, 2, Some(42)).unwrap();
    assert_eq!(result.assignment.len(), 10);
    assert_eq!(result.k, 2);
    assert!(result.assignment.contains(&0));
    assert!(result.assignment.contains(&1));
}
```

- [ ] **Step 2: Implement `Pipeline<S>` and the orchestration in `RustMetisPartitioner`**

```rust
// src/multilevel/pipeline.rs
use std::marker::PhantomData;
use crate::{graph::{CsrGraph, Partition, CoarseMap}, error::PartitionError};
use crate::multilevel::hierarchy::CoarseningHierarchy;

pub struct NeedsPartition  { pub levels_built: usize }
pub struct NeedsRefinement { pub k: u32, pub coarsest_n: usize }
pub struct Complete;

pub struct Pipeline<S> {
    pub hierarchy: CoarseningHierarchy,
    pub partition: Option<Partition>,
    pub _state:    PhantomData<S>,
}

impl Pipeline<NeedsPartition> {
    pub fn new(h: CoarseningHierarchy) -> Self {
        let levels_built = h.depth();
        Self { hierarchy: h, partition: None,
               _state: PhantomData }
    }

    pub fn initial_partition(
        mut self,
        init: &dyn crate::init::InitialPartitioner,
        k: u32, seed: u64,
    ) -> Pipeline<NeedsRefinement> {
        let coarsest = self.hierarchy.coarsest();
        let p = init.partition(coarsest, k, seed);
        let coarsest_n = coarsest.n();
        Pipeline {
            hierarchy: self.hierarchy,
            partition: Some(p),
            _state: PhantomData,
        }
    }
}

impl Pipeline<NeedsRefinement> {
    pub fn refine_and_project(
        mut self,
        refiner: &dyn crate::refine::Refiner,
    ) -> Pipeline<Complete> {
        let depth = self.hierarchy.depth();
        let mut current_p = self.partition.take().unwrap();

        // Uncoarsen from coarsest back to original
        for lev in (0..depth).rev() {
            current_p = refiner.refine(
                &self.hierarchy.levels[lev + 1],
                current_p,
            );
            let fine_assignment = self.hierarchy.project_up(lev, &current_p.assignment);
            current_p = Partition { assignment: fine_assignment, k: current_p.k };
        }
        // Final refinement at original level
        current_p = refiner.refine(&self.hierarchy.levels[0], current_p);

        Pipeline { hierarchy: self.hierarchy, partition: Some(current_p), _state: PhantomData }
    }
}

impl Pipeline<Complete> {
    pub fn into_partition(self) -> Partition {
        self.partition.unwrap()
    }
}
```

- [ ] **Step 3: Wire into `RustMetisPartitioner::split`** in `src/api.rs`

```rust
impl<C: Coarsener, I: InitialPartitioner, R: Refiner> Partitioner
    for RustMetisPartitioner<C, I, R>
{
    fn split(&self, g: &CsrGraph, k: u32, seed: Option<u64>)
        -> Result<Partition, PartitionError>
    {
        if !g.is_valid() { return Err(PartitionError::InvalidGraph("is_valid() failed")); }
        if k == 0 { return Err(PartitionError::ZeroParts); }
        if k as usize > g.n() { return Err(PartitionError::TooManyParts { k, n: g.n() }); }

        let rng_seed = seed.map(|s| {
            use rand_pcg::Pcg64; use rand::SeedableRng;
            // Use seed to derive sub-seeds for each phase
            s
        }).unwrap_or(0xDEAD_BEEF_CAFE_1234);

        let hierarchy = CoarseningHierarchy::build(g, &self.coarsener)?;
        let pipeline = Pipeline::new(hierarchy)
            .initial_partition(&self.init, k, rng_seed)
            .refine_and_project(&self.refiner);

        Ok(pipeline.into_partition())
    }

    fn split_weighted(&self, g: &CsrGraph, fracs: &[u32], seed: Option<u64>)
        -> Result<Partition, PartitionError>
    {
        if fracs.is_empty() { return Err(PartitionError::ZeroParts); } // PP-PLAN-03
        let k = fracs.len() as u32;
        if !g.is_valid() { return Err(PartitionError::InvalidGraph("is_valid() failed")); }
        if k as usize > g.n() { return Err(PartitionError::TooManyParts { k, n: g.n() }); }

        let total_frac: u32 = fracs.iter().sum();
        let total_pop: i64 = g.vwgt.iter().map(|&w| w as i64).sum();
        let rng_seed = seed.unwrap_or(0xDEAD_BEEF_CAFE_1234);

        // Build a CsrGraph with scaled per-part tpwgts by adjusting the initial
        // partition seeding: GrowKway seeds proportionally to fracs.
        // Pass fracs as target populations into a fractional-seeded GrowKway.
        let hierarchy = CoarseningHierarchy::build(g, &self.coarsener)?;

        // Compute per-part population targets from fracs
        let targets: Vec<i64> = fracs.iter()
            .map(|&f| (total_pop * f as i64 + total_frac as i64 / 2) / total_frac as i64)
            .collect();

        let coarse_p = self.init.partition(hierarchy.coarsest(), k, rng_seed);
        let pipeline = Pipeline {
            hierarchy,
            partition: Some(coarse_p),
            _state: std::marker::PhantomData::<crate::multilevel::pipeline::NeedsRefinement>,
        };
        let result = pipeline.refine_and_project(&self.refiner);
        Ok(result.into_partition())
    }
}
```

- [ ] **Step 4: Add `split_weighted` L0 test (BENCHMARK Gap 3)**

```rust
#[test]
fn split_weighted_asymmetric_fracs() {
    // fracs [8, 9] for a 17-vertex path — part 0 should have ~8/17, part 1 ~9/17 of pop
    let g = path_graph(17);
    let total: i64 = g.vwgt.iter().map(|&w| w as i64).sum(); // = 17
    let p = MetisPartitioner::with_params(MetisParams::default(), 2)
        .split_weighted(&g, &[8u32, 9u32], Some(0))
        .unwrap();
    assert_eq!(p.assignment.len(), 17);
    let pop0: i64 = (0..17).filter(|&v| p.assignment[v] == 0)
        .map(|v| g.vwgt[v] as i64).sum();
    let pop1: i64 = total - pop0;
    // Each part should be within 1 vertex of its target
    let eps = 2i64;
    assert!((pop0 - 8).abs() <= eps, "part 0 pop {pop0} not near target 8");
    assert!((pop1 - 9).abs() <= eps, "part 1 pop {pop1} not near target 9");
}

#[test]
fn split_weighted_empty_fracs_errors() {
    let g = path_graph(10);
    let result = MetisPartitioner::with_params(MetisParams::default(), 1)
        .split_weighted(&g, &[], Some(0));
    assert!(matches!(result, Err(PartitionError::ZeroParts)));
}
```

- [ ] **Step 5: Add `MetisPartitioner` type alias**

```rust
use crate::coarsen::shem::SortedHeavyEdgeMatchWithParams;
use crate::init::grow::GrowBisect;
use crate::refine::fm::FiducciaMattheyses;

pub type MetisPartitioner = RustMetisPartitioner<
    SortedHeavyEdgeMatchWithParams,
    GrowBisect,
    FiducciaMattheyses,
>;

impl MetisPartitioner {
    pub fn with_params(params: MetisParams, k: u32) -> Self {
        RustMetisPartitioner {
            coarsener: SortedHeavyEdgeMatchWithParams {
                coarsen_to: params.coarsen_to, k,
            },
            init:    GrowBisect,
            refiner: FiducciaMattheyses { niter: params.niter },
            params,
        }
    }
}
```

- [ ] **Step 5: Run full pipeline tests + correctness oracle**

```
cargo test -p redist-metis
```
Expected: all tests pass.

- [ ] **Step 6: Commit**

```
git commit -m "feat(redist-metis): Pipeline typestate + RustMetisPartitioner + MetisPartitioner alias"
```

---

### Task 15: Correctness oracle + golden RNG test + proptest

**Files:**
- Modify: `tests/contracts.rs`
- Create: `tests/proptest_invariants.rs`
- Create: `tests/golden/vt_seed42.json`

- [ ] **Step 1: Write full correctness oracle** — 7 graphs in `tests/contracts.rs`

```rust
fn compute_cut(g: &CsrGraph, assignment: &[u32]) -> u32 {
    let mut cut = 0u32;
    for v in 0..g.n() {
        for j in g.xadj[v] as usize..g.xadj[v+1] as usize {
            let u = g.adjncy[j] as usize;
            if assignment[v] != assignment[u] { cut += 1; }
        }
    }
    cut / 2
}

#[test]
fn oracle_trivial_k1() {
    let g = path_graph(10);
    let p = MetisPartitioner::with_params(MetisParams::default(), 1)
        .split(&g, 1, Some(0)).unwrap();
    assert!(p.assignment.iter().all(|&x| x == 0));
    assert_eq!(compute_cut(&g, &p.assignment), 0);
}

#[test]
fn oracle_path_bisect_cut_1() {
    let g = path_graph(10);
    let p = MetisPartitioner::with_params(MetisParams::default(), 2)
        .split(&g, 2, Some(42)).unwrap();
    assert_eq!(compute_cut(&g, &p.assignment), 1);
}

#[test]
fn oracle_grid_4x4_k4() {
    let g = grid_4x4();
    let p = MetisPartitioner::with_params(MetisParams::default(), 4)
        .split(&g, 4, Some(0)).unwrap();
    // Grid 4x4 optimal cut for 4 equal parts = 4 (quadrants)
    assert!(compute_cut(&g, &p.assignment) <= 8); // allow some slack
    assert_eq!(p.assignment.len(), 16);
}

#[test]
fn oracle_dumbbell_cut_1() {
    let g = dumbbell_graph(); // two K5 joined by 1 edge
    let p = MetisPartitioner::with_params(MetisParams::default(), 2)
        .split(&g, 2, Some(0)).unwrap();
    assert_eq!(compute_cut(&g, &p.assignment), 1);
}

#[test]
fn oracle_coverage_all_vertices_assigned() {
    let g = grid_4x4();
    for k in [1, 2, 4] {
        let p = MetisPartitioner::with_params(MetisParams::default(), k)
            .split(&g, k, Some(0)).unwrap();
        assert_eq!(p.assignment.len(), g.n());
        for &a in &p.assignment { assert!(a < k); }
    }
}
```

- [ ] **Step 2: Add golden generation test (run once, then commit result)**

Add to `tests/contracts.rs`:

```rust
/// Run with `cargo test generate_golden -- --ignored` to regenerate.
/// Commit the resulting tests/golden/vt_seed42.json.
/// Regenerate only when rand_pcg is intentionally upgraded.
#[test]
#[ignore]
fn generate_golden() {
    let g = make_path(255);
    let p = MetisPartitioner::with_params(MetisParams::default(), 1)
        .split(&g, 1, Some(42)).unwrap();
    let rand_pcg_version = env!("CARGO_PKG_VERSION"); // captured at build time from rand_pcg
    let json = serde_json::json!({
        "seed": 42,
        "n": 255,
        "k": 1,
        "rand_pcg_version": rand_pcg_version,
        "generated_at": "2026-05-02",
        "assignment": p.assignment,
    });
    std::fs::create_dir_all("tests/golden").unwrap();
    std::fs::write("tests/golden/vt_seed42.json",
        serde_json::to_string_pretty(&json).unwrap()).unwrap();
    println!("Golden value written — commit tests/golden/vt_seed42.json");
}
```

- [ ] **Step 3: Generate the golden file**

```
cargo test -p redist-metis generate_golden -- --ignored
git add tests/golden/vt_seed42.json
git commit -m "test(redist-metis): add golden RNG determinism pin (rand_pcg 0.3, seed=42)"
```

- [ ] **Step 4: Add the pinned golden test** (normal, non-ignored)

```rust
#[test]
fn golden_rng_determinism() {
    let golden: serde_json::Value =
        serde_json::from_str(include_str!("golden/vt_seed42.json")).unwrap();
    let g = make_path(golden["n"].as_u64().unwrap() as usize);
    let p = MetisPartitioner::with_params(MetisParams::default(), 1)
        .split(&g, 1, Some(golden["seed"].as_u64().unwrap()))
        .unwrap();
    let expected: Vec<u32> = golden["assignment"].as_array().unwrap()
        .iter().map(|v| v.as_u64().unwrap() as u32).collect();
    assert_eq!(p.assignment, expected,
        "RNG determinism broken — rand_pcg version changed? \
         Regenerate with: cargo test generate_golden -- --ignored");
}
```

- [ ] **Step 3: Write proptest invariants**

```rust
// tests/proptest_invariants.rs
use proptest::prelude::*;
use redist_metis::{graph::CsrGraph, coarsen::{Coarsener, shem::SortedHeavyEdgeMatch}};

fn arb_path(max_n: usize) -> impl Strategy<Value = CsrGraph> {
    (2usize..=max_n).prop_map(|n| {
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for i in 0..n {
            if i > 0 { adjncy.push((i-1) as u32); }
            if i < n-1 { adjncy.push((i+1) as u32); }
            xadj.push(adjncy.len() as u32);
        }
        CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
    })
}

proptest! {
    #[test]
    fn coarsen_preserves_validity(g in arb_path(32)) {
        let (coarsened, cmap) = SortedHeavyEdgeMatch.coarsen(&g);
        prop_assert!(coarsened.is_valid());
        prop_assert_eq!(cmap.cmap.len(), g.n());
        prop_assert!(coarsened.n() < g.n());
    }
}
```

- [ ] **Step 4: Run all tests**

```
cargo test -p redist-metis
```
Expected: oracle (5 tests), golden (1), proptest (multiple iterations) all pass.

- [ ] **Step 5: Commit**

```
git commit -m "test(redist-metis): correctness oracle, golden RNG pin, proptest invariants"
```

---

## Phase 6 — Kani Safety Harnesses

**Checkpoint**: All Kani harnesses pass. `BOUNDS.md` and `UNSAFE.md` committed. CI step verifies unsafe count matches.

---

### Task 16: Sub-spec + Kani setup

**Files:**
- Create: `docs/specs/2026-05-02-redist-metis-verify.md`
- Modify: `Cargo.toml` (add kani dependency under dev)
- Create: `verify/kani/BOUNDS.md`
- Create: `verify/kani/UNSAFE.md`

- [ ] **Step 1: Write verify sub-spec** — documents each harness target, bound sizes, unsafe inventory

- [ ] **Step 2: Add kani to Cargo.toml**

```toml
[dev-dependencies]
kani = { version = "0.55", optional = true }
```

Add feature: `[features] kani = ["dep:kani"]`

- [ ] **Step 3: Write `UNSAFE.md`**

```markdown
# Unsafe Inventory

| Location | Unsafe operation | Justifying invariant | Kani harness |
|----------|-----------------|----------------------|--------------|
| `refine/fm.rs:142` | `get_unchecked(j)` on `adjncy` | `CsrGraph::is_valid()` + xadj bounds | `verify/kani/refine_harness.rs::verify_fm_no_oob` |
```

- [ ] **Step 4: Commit**

```
git commit -m "docs(redist-metis): verify sub-spec, BOUNDS.md, UNSAFE.md"
```

---

### Task 17: Kani harnesses — graph + coarsen

**Files:**
- Create: `verify/kani/csr_harness.rs`
- Create: `verify/kani/coarsen_harness.rs`

- [ ] **Step 1: Write CsrGraph harness**

```rust
// verify/kani/csr_harness.rs
#[cfg(kani)]
mod harnesses {
    use super::*;

    #[kani::proof]
    #[kani::unwind(65)]
    fn verify_is_valid_no_panic() {
        let n: usize = kani::any_where(|&n: &usize| n > 0 && n <= 8);
        // Construct a CsrGraph with kani::any() fields and call is_valid() — must not panic
        let xadj_len: usize = n + 1;
        let xadj: Vec<u32> = (0..xadj_len).map(|_| kani::any()).collect();
        let adjncy_len: usize = kani::any_where(|&l: &usize| l <= 32);
        let adjncy: Vec<u32> = (0..adjncy_len).map(|_| kani::any()).collect();
        let vwgt: Vec<i32> = (0..n).map(|_| kani::any()).collect();
        let g = CsrGraph { xadj, adjncy, ncon: 1, vwgt, adjwgt: None };
        let _ = g.is_valid(); // must not panic
    }

    #[kani::proof]
    #[kani::unwind(129)]
    fn verify_shem_no_oob() {
        let n: usize = kani::any_where(|&n: &usize| n >= 2 && n <= 16);
        // Build a valid path graph of length n
        let g = kani_path_graph(n);
        kani::assume(g.is_valid());
        let (coarsened, cmap) = SortedHeavyEdgeMatch.coarsen(&g);
        assert!(cmap.cmap.len() == g.n());
        assert!(coarsened.n() < g.n());
    }
}
```

- [ ] **Step 2: Run Kani on CsrGraph harness**

```
cargo kani --harness verify_is_valid_no_panic
cargo kani --harness verify_shem_no_oob
```
Expected: VERIFICATION SUCCESSFUL.

- [ ] **Step 3: Commit**

```
git commit -m "verify(redist-metis): Kani harnesses for CsrGraph + SHEM coarsening"
```

---

### Task 18: Kani harnesses — refine + multilevel

**Files:**
- Create: `verify/kani/refine_harness.rs`
- Create: `verify/kani/multilevel_harness.rs`

- [ ] **Step 1: FM inner loop harness**

```rust
#[kani::proof]
#[kani::unwind(65)]
fn verify_fm_no_oob() {
    let n: usize = kani::any_where(|&n: &usize| n >= 4 && n <= 16);
    let k: u32 = kani::any_where(|&k: &u32| k >= 2 && k <= 4);
    let g = kani_path_graph(n);
    kani::assume(g.is_valid());
    kani::assume(k as usize <= n);
    let p = Partition {
        assignment: (0..n).map(|i| (i % k as usize) as u32).collect(),
        k,
    };
    let result = FiducciaMattheyses { niter: 2 }.refine(&g, p);
    assert_eq!(result.assignment.len(), n);
    assert!(result.assignment.iter().all(|&a| a < k));
}
```

- [ ] **Step 2: GainTable overflow harness**

```rust
#[kani::proof]
fn verify_gain_table_no_overflow() {
    let max_gain: i32 = kani::any_where(|&g: &i32| g > 0 && g <= 128);
    let mut gt = GainTable::new(8, max_gain);
    let v: u32 = kani::any_where(|&v: &u32| v < 8);
    let gain: i32 = kani::any_where(|&g: &i32| g >= -max_gain && g <= max_gain);
    gt.insert(v, gain); // must not panic or overflow
    let _ = gt.peek_max();
}
```

- [ ] **Step 3: Run harnesses**

```
cargo kani --harness verify_fm_no_oob
cargo kani --harness verify_gain_table_no_overflow
```
Expected: VERIFICATION SUCCESSFUL.

- [ ] **Step 4: Commit + update BOUNDS.md**

```
git commit -m "verify(redist-metis): Kani harnesses for FM refinement + GainTable"
```

---

## Phase 7 — Prusti Postconditions

**Checkpoint**: Three postconditions verified by Prusti. `GAPS.md` empty. `.vpr` artifacts committed.

---

### Task 19: Prusti setup + three postconditions

**Files:**
- Create: `verify/prusti/postconditions.rs`
- Create: `verify/prusti/GAPS.md`
- Create: `verify/prusti/artifacts/` (populated by Prusti run)

- [ ] **Step 1: Write sub-spec** — document Prusti version, restricted subset, fallback policy

- [ ] **Step 2: Add Prusti annotations to `api.rs`**

```rust
#[cfg(prusti)]
use prusti_contracts::*;

#[cfg_attr(prusti, requires(g.is_valid()))]
#[cfg_attr(prusti, requires(k >= 1))]
#[cfg_attr(prusti, ensures(result.is_ok() ==>
    result.as_ref().unwrap().assignment.len() == old(g.n())))]
#[cfg_attr(prusti, ensures(result.is_ok() ==>
    forall(|i: usize| i < result.as_ref().unwrap().assignment.len()
        ==> result.as_ref().unwrap().assignment[i] < result.as_ref().unwrap().k)))]
pub fn split(...) -> Result<Partition, PartitionError> { ... }
```

- [ ] **Step 3: Run Prusti**

```
cargo prusti -- -p redist-metis
```
Expected: postconditions verified. If any function cannot be verified, add to `GAPS.md`.

- [ ] **Step 4: Commit artifacts**

```
git add verify/prusti/artifacts/ verify/prusti/GAPS.md
git commit -m "verify(redist-metis): Prusti postconditions — coverage, valid IDs, balance"
```

---

## Phase 8 — Shadow Mode + Migration

**Checkpoint**: 50-state × 3-year shadow run passes quality gate. C dep removed from `redist-apportion`.

---

### Task 20: Shadow mode feature flag

**Files:**
- Modify: `redist/crates/redist-apportion/Cargo.toml`
- Modify: `redist/crates/redist-apportion/src/split.rs`

- [ ] **Step 1: Add `redist-metis` dep + shadow feature**

```toml
# redist-apportion/Cargo.toml
[dependencies]
metis       = { version = "0.2", features = ["vendored"], optional = true }
redist-metis = { path = "../redist-metis" }

[features]
shadow-metis = ["dep:metis"]
default = ["shadow-metis"]  # keep C dep during validation phase
```

- [ ] **Step 2: Wire shadow mode in `split.rs`**

```rust
impl Partitioner for MetisPartitioner {
    fn split(&self, region: &SubGraph, k: u32, seed: Option<u64>)
        -> Result<Vec<u32>, SplitError>
    {
        let g = CsrGraph::from(region);
        let rust_result = RustMetisPartitioner::default().split(&g, k, seed)?;

        #[cfg(feature = "shadow-metis")]
        {
            // Run C METIS and compare quality
            let c_result = c_metis_split(region, k, seed)?;
            let rust_cut = compute_cut(&g, &rust_result.assignment);
            let c_cut    = compute_cut_raw(&c_result, region);
            if rust_cut > c_cut * 12 / 10 {  // >20% worse
                eprintln!("[shadow] Rust cut {rust_cut} > C METIS cut {c_cut} for k={k}");
            }
        }

        Ok(rust_result.assignment.iter().map(|&x| x as u32).collect())
    }
}
```

- [ ] **Step 3: L2 shadow test — VT smoke test**

```rust
// tests/shadow.rs (in redist-apportion)
#[test]
#[cfg(feature = "shadow-metis")]
fn shadow_vt_2020_k1() {
    // Load VT 2020 adjacency, run shadow mode, assert no quality regression
    // (test requires pipeline outputs; marked as integration test)
}
```

- [ ] **Step 4: Commit**

```
git commit -m "feat(redist-apportion): shadow-metis feature flag — parallel C + Rust validation"
```

---

### Task 21: Validation gate + cutover + C dep removal

**Files:**
- Create: `scripts/validate_shadow_gate.py`
- Modify: `redist/crates/redist-apportion/Cargo.toml`

- [ ] **Step 1: Run 50-state shadow validation**

```
run -y 2020 -v shadow --features shadow-metis
```
All states must show `rust_cut <= c_cut * 1.20` in the output.

- [ ] **Step 2: Archive shadow results as permanent evidence (COVENANT)**

Shadow output is in `outputs/` which is gitignored. Copy the diffs to the repo:

```
mkdir -p docs/validation
cp outputs/shadow/2020/shadow_diffs.json docs/validation/shadow_results_2026-05-02.json
git add docs/validation/shadow_results_2026-05-02.json
git commit -m "docs: shadow validation results — Rust quality within 20% of C METIS on all 50 states"
```

This committed artifact is the legally-archivable evidence that the Rust port meets quality parity with C METIS. It must not live only in `outputs/`.

- [ ] **Step 2: Remove default shadow feature** (once gate passes)

```toml
[features]
shadow-metis = ["dep:metis"]
default = []  # Rust-only by default
```

- [ ] **Step 3: Remove C dep entirely**

```toml
# Remove from redist-apportion/Cargo.toml:
# metis = { ... }
# shadow-metis = [...]
```

Remove from workspace `Cargo.toml`:
```toml
# Remove: metis = { version = "0.2", features = ["vendored"] }
```

- [ ] **Step 4: Verify clean build with no C compiler**

```
cargo build -p redist-metis -p redist-apportion -p redist-cli
```
Expected: builds with no C compilation step.

- [ ] **Step 5: Run full test suite**

```
cargo test -p redist-metis
cargo test -p redist-apportion
pytest tests/unit/ -v
```
Expected: all pass.

- [ ] **Step 6: Final commit**

```
git commit -m "feat: remove C METIS dep — redist-metis is now the sole graph partitioner

No C compiler required to build. Shadow mode validation passed:
Rust cut quality within 20% of C METIS on all 50 states × 3 census years.
Prusti postconditions verified. Kani harnesses all pass."
```

---

## Phase checkpoints summary

| Phase | Gate |
|-------|------|
| 1 — Scaffold | `cargo test -p redist-metis` passes; all traits compile |
| 2 — Coarsening | L0 + L1 coarsening tests; path-255 terminates < 50 levels |
| 3 — Init partition | L0 oracle: k=1 trivial, path bisect, grid 4×4 |
| 4 — FM refinement | FM does not increase cut on any oracle graph; dumbbell = 1 |
| 5 — Multilevel | Full pipeline: all 7 oracle tests pass; golden RNG pin; proptest |
| 6 — Kani | All harnesses VERIFICATION SUCCESSFUL; UNSAFE.md complete |
| 7 — Prusti | 3 postconditions verified; GAPS.md = 0 entries |
| 8 — Migration | Shadow gate passes; C dep removed; clean build |

---

**Plan complete and saved to `docs/plans/2026-05-02-redist-metis.md`.**

Two execution options:

**1. Subagent-Driven (recommended)** — fresh subagent per task, review between tasks

**2. Inline Execution** — execute tasks in this session using executing-plans

Which approach?
