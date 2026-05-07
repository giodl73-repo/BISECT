# Track H — Ensemble Search Strategies

**Module**: track-H
**Theme**: Given the ensemble MCMC landscape from Track G, which search strategies best serve the statutory objectives of redistricting — legal posture selection, local bisection-node exploration, computational scale, and geographic resolution?
**Papers**: 4
**Author**: Giovanni Della Libera
**Created**: 2026-05-07

---

## Tracks

### Track search-strategies

**Theme**: What search strategies, built on Track G's ensemble foundations, enable statutory-grade redistricting — choosing a legal posture, exploring bisection nodes locally, running at Rust speed, and handling geographic resolution?

**Chain**: `H.0+percentile-sweep` → `H.1+bisection-ensemble` → `H.2+redist-ensemble` → `H.3+resolution-aware`

**Arc**: H.0 introduces PercentileSweep — the statutory choice of legal posture — selecting among plans by their partisan percentile position in the ensemble rather than picking the global optimum; accepted at 3.4/4. H.1 presents BisectionEnsemble — local ReCom exploration at each bisection node — combining the bisection tree structure of Track B with the local ensemble search of Track G; accepted at 3.0/4. H.2 introduces the `redist-ensemble` Rust implementation achieving 2500× speed over the Python GerryChain baseline; accepted at 3.2/4. H.3 addresses resolution-aware redistricting — geographic granularity as a first-class parameter — quantifying how census resolution choices affect plan quality and ensemble mixing; accepted at 3.8/4.

---

## Papers

| Paper | Tracks | Primary Number | Status | Venue |
|-------|--------|----------------|--------|-------|
| H.0+percentile-sweep | search-strategies | statutory legal posture selection via percentile; 3.4/4 accepted | ready | Pol. Analysis |
| H.1+bisection-ensemble | search-strategies | local ReCom at each bisection node; 3.0/4 accepted | ready | ICML |
| H.2+redist-ensemble | search-strategies | Rust ReCom at 2500× Python speed; 3.2/4 accepted | ready | USENIX |
| H.3+resolution-aware | search-strategies | geographic granularity as first-class parameter; 3.8/4 accepted | ready | GIS |

---

## Module Arc

Track H is the most applied track in the portfolio — it translates the ensemble science of Track G into deployable statutory tools. All 4 papers are accepted (avg 3.35/4), making this the most review-mature track in the program after G.14. The four papers address orthogonal aspects of the search problem: legal posture (H.0), structural integration with bisection (H.1), computational scale (H.2), and data resolution (H.3). Together they form the `--search` flag ecosystem in the `redist` binary. Track H depends on G.6 (short-burst), G.9 (forest-ReCom), G.10 (merge-split) for its algorithmic foundations, and on B.16 (convergence sweep) for the baseline search strategy it supersedes. The primary cross-track dependency for the board: H.1 (bisection-ensemble) must cite and extend B.11 (ApportionRegions) — the bisection tree structure it explores locally.
