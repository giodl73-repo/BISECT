# Synthesis — prime-factor-redistricting
Round 1 | Avg score: 2.72/4.0

## Scores
| Reviewer | Score |
|--|--|
| R-50 (Jeffrey Ullman) | 2.5 |
| R-1 (Percy Liang) | 2.5 |
| R-37 (Nadia Polikarpova) | 2.8 |
| R-9 (Joseph Gonzalez) | 3.0 |
| R-6 (Matei Zaharia) | 2.8 |
| **Average** | **2.72** |

## What the reviewers agree on

**Genuine strength**: The NC/GA finding (same k=14=7×2 factorization, opposite partisan effects) is universally recognized as the paper's strongest empirical contribution. All five reviewers find the core idea — that geography determines the outcome, not the algorithm — compelling and the NC/GA pair is a clean demonstration of it.

**Reuse theorem**: All reviewers recognize the reuse property as novel and practically useful. The observation that k=34 and k=51 share the same 17-district partition is consistently praised.

**Core methodology gap (unanimous P1)**: Single-seed evaluation is flagged by every reviewer as insufficient. Ullman, Liang, Polikarpova, Gonzalez, and Zaharia all independently identify that seed sensitivity analysis is required to validate the empirical claims. The WI false floor problem is the concrete example showing this matters.

**Formal closure of binary fallback (Ullman + Polikarpova)**: Both formal methods reviewers flag that the binary fallback for prime k > 3 breaks the paper's formal claim that PFR is "pure prime factorization." The algorithm as described is not formally closed as a function from (n, G, seed) to partition.

## P1 Items (must fix for acceptance)

**P1-A** (all reviewers): Add seed sensitivity analysis. Run at minimum 10 seeds for all 50 states. Report distribution of D/R seat counts per state. This is required to determine whether the NC/GA headline finding and the 223D/209R national total are representative or seed-specific.

**P1-B** (Ullman, Polikarpova): Formally state and prove the Reuse Theorem, including: (a) a METIS determinism condition as a stated assumption or proved property, (b) a precise definition of "same partition," (c) a proof that top-level inputs are identical for two seat counts sharing the largest prime factor. Half a page suffices.

**P1-C** (Ullman, Polikarpova): Formally characterize the binary fallback. Either (a) define the fallback recursion tree uniquely and prove the resulting partition is well-defined, or (b) implement direct k-way METIS for all prime k, or (c) restrict formal claims to k with all prime factors ≤ 3. The current situation — where k=17 uses an underspecified sequence of binary splits — breaks formal closure.

**P1-D** (Liang): Report results at 0.5% balance tolerance for at least NC and GA. The 3% research-mode tolerance is a confound; if the headline finding disappears at statutory tolerance, the paper's claims are significantly weakened. If it survives, the paper is strengthened.

**P1-E** (Liang): Add at least one comparison to GerryChain or ReCom. MEC as a single baseline is insufficient for a journal claiming PFR is a principled alternative to existing methods.

## P2 Items (should fix)

**P2-A** (Ullman, Polikarpova, Zaharia): Specify region_hash construction (content-addressed vs. identifier-addressed). Discuss correctness implications for cross-census reuse.

**P2-B** (Ullman): Add citations to Baker v. Carr, Wesberry v. Sanders, and the political question doctrine. The constitutional section will not survive political science peer review without these.

**P2-C** (Gonzalez): Add a comparison of PFR vs. direct k-way METIS for states with k ≤ 10 to validate that hierarchical approximation does not significantly degrade partition quality (global edge cut).

**P2-D** (Zaharia, Liang): Report PfrCompositor cache hit rate across the 50-state sweep. This quantifies how often the reuse theorem provides practical speedup and whether it is empirically meaningful.

**P2-E** (Gonzalez): Report subgraph size distribution for the top-level partition (mean, std, min, max tract count per subgraph) to characterize load balance of the hierarchical approach.

**P2-F** (Liang): Provide a replication package (seed values, adjacency graph checksums, factorization trees for all 50 states).

## Path to acceptance

The paper is at **borderline accept** (2.72 average). The NC/GA finding is strong enough to carry the paper if the single-seed limitation is addressed. P1-A (seed sensitivity) is the most critical item: if results are stable across seeds, the paper becomes a clear accept. P1-B and P1-C (formal closure) are required for the paper to defend its "theorem" label but will not change the empirical conclusions.

Estimated impact of P1 items on score:
- P1-A resolved: +0.3 to +0.5 (all reviewers raise scores)
- P1-B/C resolved: +0.1 to +0.2 (formal reviewers satisfied)
- P1-D resolved favorably: +0.1 to +0.2
- P1-E resolved: +0.1 (Liang, Gonzalez satisfied)
Target revised score: **3.1–3.3 / 4.0**
