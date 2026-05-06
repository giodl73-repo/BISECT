# Review — H.2: redist-ensemble
**Reviewer**: George Karypis (Graph Partitioning, High-Performance Computing)
**Round**: 1
**Score**: 3 / 5

---

## Summary

The paper proposes a Rust implementation of the ReCom Markov chain for redistricting ensemble analysis, targeting a 2,300× speedup over GerryChain. The core algorithmic claims are partially correct but contain a significant complexity misstatement, the balance-check enumeration claim requires closer scrutiny for statistical correctness, and the throughput estimates, while clearly labeled as estimates, rest on a back-of-envelope calculation whose assumptions are underspecified.

---

## Algorithmic Correctness: Wilson's Complexity

The paper correctly identifies Wilson's algorithm as the right tool here — it is the standard method for generating uniform random spanning trees and the natural primitive for ReCom. The complexity statement in the abstract, however, is problematic.

The abstract states (line 91): "Wilson's algorithm achieves $O(|V|\log|V|)$ expected time on planar graphs." This formulation conflates two distinct statements. Wilson's algorithm runs in expected time equal to the *cover time* $\tau_{\text{cover}}(H)$ of a simple random walk on the input graph $H$. The $O(m \log m)$ bound on cover time for planar graphs is a separate result due to Aldous (1991), cited correctly later in Section 1.2. In the abstract, the connection to cover time is omitted and the bound is stated as if it were Wilson's original theorem, which is not accurate.

Section 3.2 is better: the theorem box correctly states $O(\tau_{\text{cover}}(H))$ and then cites Aldous for the planar specialization. However, Section 1.2 conflates these again: "Wilson's algorithm runs in $O(\tau_{\text{cover}})$ expected time... For planar graphs... this is $O(|V|\log|V|)$." This is close to correct — the planar graph cover-time bound is $\Theta(n \log n)$ in general (Aldous proved the upper bound; the matching lower bound comes from specific constructions). But the paper asserts this bound for *census-tract adjacency subgraphs* without justifying that these subgraphs are actually planar. Census-tract adjacency graphs derived from TIGER shapefiles are typically planar (tracts do not cross), but they sometimes contain small non-planar subgraphs near tri-point boundaries and state-line artifacts. The paper should either (a) verify that the subgraphs used in practice are planar, or (b) state that the $O(m \log m)$ bound is an approximation and that actual cover times may be higher.

The complexity computation in Section 5.2 is the paper's weakest passage. The estimate that Wilson requires "approximately $191 \times \log(191) \approx 191 \times 7.5 \approx 1{,}433$ random walk steps" for NC's merged region of $m \approx 191$ vertices treats the cover time as exactly $m \ln m$. This is the asymptotic leading term, not a tight bound for $m = 191$. At small $m$, constants matter significantly. Additionally, the per-walk-step cost of "approximately 8 instructions" is stated without justification — loop-erased random walk requires a path buffer, loop detection (hash-set lookup with potential cache misses), and re-indexing, none of which are $O(1)$ with small constants on realistic hardware. The $13\times$ overhead factor applied afterward is entirely ad hoc.

---

## Statistical Correctness: Balance-Cut Enumeration

The paper's claim in Section 3.2 that enumerating all balanced cuts is "necessary to replicate GerryChain's stationary distribution" is correct in principle but requires more careful discussion. The ReCom stationary distribution (DeFord et al. 2021) places weight on each spanning tree proportional to the number of balanced bipartitions it admits, normalized by the total number of spanning trees times the number of balanced cuts. Full enumeration is indeed the correct implementation.

What the paper does not address is whether the pair-selection step also needs to be uniform. Algorithm 2 (line 82) selects $(i,j)$ "uniformly from all adjacent district pairs" at the outer step, and then again at line 79 in the reselection branch — but the reselection branch is triggered after failure, meaning the effective pair distribution is no longer uniform over adjacent pairs. Pairs with low balanced-bipartition feasibility are systematically abandoned more often, tilting the Markov kernel away from the stationary distribution. The paper acknowledges this informally ("pair reselection triggers frequently during the first 50--100 steps") but does not analyze whether this bias washes out at stationarity or constitutes a permanent deviation from GerryChain's distribution.

This is a correctness concern, not just a performance question. If the pair-reselection mechanism induces a different stationary distribution than full GerryChain, the paper's claim that redist-ensemble is a faithful Rust port is false. The paper should either provide a formal argument that the stationary distribution is preserved under pair reselection, or acknowledge that pair reselection produces a related-but-distinct chain.

---

## Throughput Estimates

The 50,000 steps/sec estimate is labeled clearly as an estimate throughout (dagger notation, "Phase 2" disclaimer), which is appropriate. The speedup ratio of 2,300× follows from dividing 50,000 by the measured GerryChain rate of 21 steps/sec for NC. This arithmetic is sound conditional on the estimate. My concern is that the estimate's uncertainty bounds are never stated. The paper says 50,000 steps/sec is "conservative" based on a $13\times$ overhead factor, but provides no lower-bound estimate. A sensitivity analysis — what is the speedup if actual throughput is 5,000 steps/sec? 20,000? — would give readers a clearer picture of what the Phase 2 benchmarks need to confirm.

The GerryChain baseline measurements (Table 1) are stated as direct empirical measurements, which is appropriate. The NC figure (21 steps/sec for 2,672 tracts, $k=14$) is consistent with publicly known GerryChain performance, lending credibility to the baseline.

---

## Minor Issues

- The "13× overhead" framing in Section 5.2 compares MCMC to METIS multilevel contraction, which is a structurally very different workload. The comparison provides rhetorical support for the conservatism claim, but the two workloads have different cache footprints, different instruction mix profiles, and different memory-access patterns. A stronger justification would compare against other MCMC-in-Rust benchmarks.
- The stack-based small-buffer optimization for subgraphs with $m \leq 512$ (Section 4.2) is cited without a measurement. For NC's typical merged-region size of $m \approx 191$, this optimization applies most of the time. Its omission from the throughput estimate is unexplained.

---

## Recommendation

Major revision. The algorithm description in the abstract and introduction should be corrected to distinguish Wilson's cover-time bound from Aldous's planar graph specialization. The planarity of census-tract subgraphs should be stated or verified. Most importantly, the statistical correctness of pair reselection relative to GerryChain's stationary distribution must be addressed explicitly.
