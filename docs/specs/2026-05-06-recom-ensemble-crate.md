# Spec: `redist-ensemble` — Rust ReCom Feasibility Sampler

**Status**: Proposed  
**Date**: 2026-05-06  
**Motivation**: GerryChain (Python) runs ~20 ReCom steps/second for NC (k=14, 2672 tracts). A Rust implementation should achieve ~50,000 steps/second — a 2,500× speedup — enabling 10,000-step ensembles for all 50 states in under 10 seconds total.

---

## What is ReCom?

**ReCom** (Recombination) is a Markov Chain Monte Carlo proposal for exploring the space of valid redistricting plans, introduced by DeFord, Duchin & Solomon (2021).

**One step:**
1. Find all pairs of adjacent districts (share at least one edge between tracts)
2. Pick a random adjacent pair (d_i, d_j)
3. Form the "region" = all tracts in d_i ∪ d_j (a connected subgraph)
4. Sample a **random spanning tree** of the region subgraph
5. Remove one spanning tree edge, creating two connected components
6. Check population balance: each component within ε of ideal
7. If balanced → accept new assignment. If not → reject and try again (or resample pair)

**Key property**: ReCom produces a Markov chain that mixes over all contiguous, population-balanced plans — it's a sampler, not an optimizer.

**Use cases**:
- Audit enacted maps: does the enacted plan fall in the middle of the distribution, or is it an outlier?
- Characterise the feasible space: what is the range of partisan outcomes, PP scores, minority representation possible under fair constraints?
- Compute ensemble diagnostics (R-hat, ESS) to certify chain convergence

---

## Proposed architecture

### New crate: `redist-ensemble`

```
redist/crates/redist-ensemble/
  src/
    lib.rs         — public API
    recom.rs       — ReCom proposal: spanning tree sampling + partition update
    spanning.rs    — Wilson's algorithm for uniform random spanning trees
    chain.rs       — Markov chain runner (serial and parallel)
    diagnostics.rs — R-hat, ESS, Hamming autocorrelation (from redist-analysis)
    updaters.rs    — Metrics computed per step: cut_edges, pop_balance, partisan
  Cargo.toml
```

### Key data structures

```rust
/// One full Markov chain run.
pub struct RecomChain {
    graph: CsrGraph,            // from redist-core
    assignment: Vec<u32>,       // tract index → district id (1-based)
    k: u32,
    pop_tolerance: f64,         // ε: max deviation from ideal (e.g. 0.01 = 1%)
    rng: SmallRng,
    steps_taken: u64,
}

/// Per-step metrics stored for diagnostics.
pub struct StepRecord {
    step: u64,
    cut_edges: usize,           // raw edge cut count
    cut_fraction: f32,          // cut_edges / total_edges
    pop_deviation: f32,         // max |pop_i - ideal| / ideal
    accepted: bool,             // was the proposal accepted?
}

/// Full ensemble result (all chains).
pub struct EnsembleResult {
    state: String,
    k: u32,
    n_steps: u64,
    n_chains: usize,
    chains: Vec<Vec<StepRecord>>,
    chain_seeds: Vec<u64>,      // Per-chain seed; chain i uses SHA-256("ENSEMBLE_CHAIN_" || i || "_" || base_seed)
    r_hat: Option<f64>,         // Gelman-Rubin (continuous stats only; see footnote), requires n_chains >= 2
    ess: Option<f64>,           // Effective sample size
    hamming_autocorr: Vec<f64>, // Lag-1 to lag-20 autocorrelation
}
```

### Wilson's algorithm for random spanning trees

The key primitive is sampling a **uniformly random spanning tree** (UST) of a connected graph. Wilson's algorithm (1996) runs a loop-erased random walk:

```rust
fn random_spanning_tree(graph: &SubGraph, rng: &mut SmallRng) -> Vec<(u32, u32)> {
    // Wilson's algorithm: O(cover time) expected; O(|V| log |V|) expected for planar graphs
    // (Wilson 1996, Theorem 1.1). For redistricting graphs (planar, ~n/k nodes per region),
    // this is O((n/k) log(n/k)) per bisection step.
    let n = graph.n_vertices();
    let mut in_tree = vec![false; n];
    let mut parent = vec![u32::MAX; n];
    
    // Start from a random root
    let root = rng.gen_range(0..n) as u32;
    in_tree[root as usize] = true;
    
    for start in 0..n as u32 {
        if in_tree[start as usize] { continue; }
        // Loop-erased random walk from `start`
        let mut path = vec![start];
        let mut cur = start;
        loop {
            let neighbors = graph.neighbors(cur);
            cur = neighbors[rng.gen_range(0..neighbors.len())];
            // Erase any loop
            if let Some(pos) = path.iter().position(|&v| v == cur) {
                path.truncate(pos + 1);
            } else {
                path.push(cur);
                if in_tree[cur as usize] { break; }
            }
        }
        // Add path to tree
        for i in 0..path.len()-1 {
            parent[path[i] as usize] = path[i+1];
            in_tree[path[i] as usize] = true;
        }
    }
    
    // Convert parent array to edge list
    (0..n as u32)
        .filter(|&v| parent[v as usize] != u32::MAX)
        .map(|v| (v, parent[v as usize]))
        .collect()
}
```

### ReCom step

```rust
impl RecomChain {
    pub fn step(&mut self) -> bool {
        // 1. Find all adjacent district pairs
        let pairs = self.find_adjacent_pairs();
        if pairs.is_empty() { return false; }
        
        // 2. Pick random pair
        let (d_i, d_j) = pairs[self.rng.gen_range(0..pairs.len())];
        
        // 3. Extract region = all tracts in d_i ∪ d_j
        let region: Vec<u32> = self.assignment.iter().enumerate()
            .filter(|(_, &d)| d == d_i || d == d_j)
            .map(|(i, _)| i as u32)
            .collect();
        
        // 4. Build subgraph of the region
        let sub = SubGraph::induced(&self.graph, &region);
        
        // 5–6. Attempt up to 10 full spanning tree resamples.
        //    For each tree, check ALL edges for balance and accept the first balanced cut.
        //    If no balanced cut exists in this tree, resample the ENTIRE spanning tree.
        //    If all 10 tree resamples fail, reject the proposal and pick a new district pair.
        for _ in 0..10 {
            let tree = random_spanning_tree(&sub, &mut self.rng);
            
            // Enumerate all tree edges, find all balanced cuts
            let mut balanced_cuts: Vec<(u32, u32)> = Vec::new();
            for &cut_edge in &tree {
                let (comp_a, comp_b) = split_on_edge(&sub, &tree, cut_edge);
                let pop_a: i64 = comp_a.iter().map(|&v| self.graph.pop(v) as i64).sum();
                let pop_b: i64 = comp_b.iter().map(|&v| self.graph.pop(v) as i64).sum();
                let ideal = (pop_a + pop_b) as f64 / 2.0;
                if (pop_a as f64 - ideal).abs() / ideal <= self.pop_tolerance {
                    balanced_cuts.push(cut_edge);
                }
            }
            
            if !balanced_cuts.is_empty() {
                // Pick uniformly among balanced cuts (matches GerryChain's stationary distribution)
                let cut_edge = balanced_cuts[self.rng.gen_range(0..balanced_cuts.len())];
                let (comp_a, comp_b) = split_on_edge(&sub, &tree, cut_edge);
                for &v in &comp_a { self.assignment[v as usize] = d_i; }
                for &v in &comp_b { self.assignment[v as usize] = d_j; }
                return true;
            }
            // No balanced cut in this tree — resample the ENTIRE spanning tree
        }
        false // Proposal rejected after 10 full tree resamples
    }
}
```

---

## CLI surface

```bash
# Run 10,000-step ensemble for NC, 4 chains
redist ensemble --state NC --year 2020 --steps 10000 --chains 4 \
  --output nc_ensemble.json

# Run all 6 key states
redist ensemble --states NC WI GA PA TX CA --steps 10000 --chains 4 \
  --output-dir research/G.1+gerrychain-congressional-comparison/data/

# Compute diagnostics on an existing ensemble
redist ensemble-diagnostics --input nc_ensemble.json

# Audit an enacted plan: compute its percentile in the ensemble
redist ensemble --state NC --year 2020 --steps 10000 --chains 4 \
  --compare-plan runs/official_2020/2020/north_carolina/final_assignments.json \
  --output nc_ensemble.json
```

The `--compare-plan` flag loads a specific plan (by path to a `final_assignments.json`) and computes its edge-cut fraction and percentile position within the ensemble. This is the correct workflow for auditing enacted maps. The default (no `--compare-plan`) audits the `redist build` plan for the same state/year/version.

Output JSON format:
```json
{
  "state": "NC", "k": 14, "n_steps": 10000, "n_chains": 4,
  "base_seed": 12345678,
  "chain_seeds": [3735928559, 14983423, 9182736450, 7654321098],
  "pooled_cut_fraction": { "mean": 0.0970, "std": 0.0059, "p5": 0.0875, "p95": 0.1065 },
  "plan_cut_fraction": 0.0973,
  "plan_percentile": 49.8,
  "r_hat": 1.003,
  "ess": 4821,
  "hamming_autocorr": [0.12, 0.04, 0.02, 0.01, 0.01, 0.00, 0.00, 0.00, 0.00, 0.00,
                        0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00]
}
```

**Seed specification**: `--seed <u64>` sets the base seed (if omitted, drawn from OS entropy and printed to stderr). Chain i uses seed `SHA-256("ENSEMBLE_CHAIN_" || i || "_" || base_seed)`, truncated to 64 bits. All `chain_seeds` are recorded in the JSON output for full reproducibility.

---

## 5.1 The Compactness Extremum and Ensemble Position

METIS minimises edge cuts by design. As a result, bisection plans produced by `redist build` are expected to sit at the **low end** of the ReCom cut-fraction distribution — i.e., more compact than most valid plans that ReCom samples.

Empirical results from the G-series:

| State | METIS plan percentile in ReCom ensemble |
|-------|-----------------------------------------|
| NC | ~50th (geographic convergence: both methods find similarly compact plans) |
| WI | ~0.2th |
| GA | ~0.1th |
| PA | ~0.2th |

**Interpretation**: The METIS plan is not a "typical" valid plan — it is a compactness optimum. The ReCom ensemble characterises the full feasible space, and the METIS plan sits at the compact extreme for most states. NC is the exception: geographic constraint (the Appalachians + coastal plain divide) dominates, so METIS and ReCom converge on similar cuts.

**Important distinction**: Sitting at the 0.1th percentile of the *cut-fraction* distribution does not imply partisan bias. It implies the METIS plan is more compact than 99.9% of valid plans. Whether partisan outcomes also sit at an extreme requires a separate analysis (D-seats percentile, efficiency gap) that is not provided by this crate.

**PercentileSweep as the solution**: The `2026-05-06-percentile-sweep.md` spec addresses this by enabling the algorithm to target any percentile of the ensemble — including the 50th — rather than always producing the compactness extremum. This provides the legislature or commission with an explicit, auditable choice of legal posture.

---

## Performance estimates (unvalidated)

> These estimates are extrapolated from GerryChain performance. Actual Rust benchmarks will be measured in Phase 2 using criterion.rs on identical hardware.

| State | k | Tracts | GerryChain 1K steps | Rust estimate 10K steps |
|-------|---|--------|--------------------|-----------------------|
| NC | 14 | 2,672 | ~47s | ~0.1s |
| WI | 8 | 1,542 | ~28s | ~0.05s |
| GA | 14 | 1,931 | ~30s | ~0.08s |
| PA | 17 | 3,236 | ~30s | ~0.1s |
| TX | 38 | 5,265 | ~90s | ~0.5s |
| CA | 52 | 8,057 | ~120s | ~0.8s |
| **All 6** | | | **~6 min/1K steps** | **~2s/10K steps** |

**Note on TX bipartition failures**: Bipartition failures for prime k (PA k=17, TX k=38) are a combinatorial property of the graph: some region subgraphs have no balanced spanning tree cut. Both GerryChain and the Rust implementation will encounter these; the Rust implementation handles them faster but cannot eliminate them. For TX k=38, pair reselection (try a different adjacent district pair) is the correct mitigation, not tree resampling.

---

## Implementation plan

### Phase 1 — Core (1 week)
- [ ] `spanning.rs`: Wilson's UST algorithm
- [ ] `recom.rs`: single ReCom step, balance check, retry logic
- [ ] `chain.rs`: serial chain runner, step records
- [ ] Unit tests: spanning tree is a valid tree; partition stays contiguous and balanced

### Phase 2 — CLI + Output (3 days)
- [ ] `redist ensemble` command in `redist-cli/src/args.rs`
- [ ] JSON output format
- [ ] Parallel chains (Rayon): each chain on its own thread

### Phase 3 — Diagnostics (3 days)
- [ ] `diagnostics.rs`: R-hat, ESS, Hamming autocorrelation
- [ ] `redist ensemble-diagnostics` command
- [ ] G.4 paper numbers become reproducible via `redist`

### Phase 4 — G-series paper update (1 day)
- [ ] Rerun G.1–G.4 data with Rust ensemble
- [ ] Update G.1 summary table with `n_steps=10000` per state
- [ ] Recompile all G PDFs

---

## Connection to existing work

- `redist-metis` already has `CsrGraph` — reuse directly
- `redist-analysis::ensemble_diagnostics` already has R-hat, ESS, Hamming — import these
- The `redist build` → `redist ensemble` pipeline: build the plan first, then characterise its position in the ensemble space
- `redist label-verify` + ensemble: the SHA chain certifies the plan; the ensemble certifies the plan's position in the feasible space

---

## Research significance

Adding Rust ReCom closes the last methodological gap in the portfolio:
- G.1 becomes **fully reproducible** via `redist` (no Python/GerryChain dependency)
- G.4 diagnostics become trivially fast (10K-step ensembles for R-hat convergence)
- The DIA statutory argument gains a new leg: the algorithm is not only deterministic and verifiable, but its position in the feasible space is also certifiable by the same tool that generated it

---

## Footnotes

**[1] R-hat for discrete statistics**: Standard Gelman-Rubin R-hat is defined for continuous parameters. For discrete redistricting statistics (cut edge count, seat counts), use rank-normalised R-hat (Vehtari et al. 2021) or treat the cut fraction as approximately continuous given its near-Gaussian distribution in large ensembles. R-hat computation in this crate is restricted to continuous statistics (`cut_fraction`, `pop_deviation`). For discrete statistics (seat counts, contiguity indicators), use TV distance between chain marginals or the rank-normalised variant.
