# H.1 BisectionEnsemble — Revision Plan (Round 1)

**Paper**: H.1: BisectionEnsemble: Integrating Local Feasibility Sampling into the Redistricting Bisection Tree
**Review Round**: 1
**Date**: 2026-05-05
**Reviewers**: Karypis (graph partitioning), Rodden (political geography), Duchin (GerryChain/metric geometry), Stephanopoulos (election law), Liang (reproducibility)

---

## Aggregate Recommendations

| Reviewer | Recommendation |
|---|---|
| Karypis | Major Revision |
| Rodden | Minor Revision |
| Duchin | Major Revision |
| Stephanopoulos | Minor Revision |
| Liang | Major Revision (Reproducibility) |

**Consensus**: Major Revision required. Three of five reviewers flag substantive issues. The algorithm is considered sound; the theoretical, empirical, and legal sections each have specific gaps.

---

## Critical Fixes (Must Address)

### C-1: Theorem 1 Proof — Incomplete (Karypis)

**Problem**: The proof appeals to an "intermediate-value property of the discrete population function" without making the argument explicit. The IVP for spanning tree cuts needs to be spelled out: as you enumerate all $|V_H|-1$ edges of a spanning tree, the left-component population traverses from near-zero to near-total. The proof also conflates "at least one plan will be accepted within $O(|V_H|)$ steps" with "each step accepts with positive probability."

**Fix**: Rewrite Theorem 1 proof in three parts:
1. Show that any spanning tree of $H$ has a balanced cut edge (discrete IVP argument, explicit).
2. Show that this spanning tree is sampled with positive probability under the UST distribution (since UST assigns positive probability to every spanning tree of a connected graph).
3. Conclude that the chain accepts in finite expected steps.

Add a note on the distinction between "existence of acceptance" and "per-step acceptance probability."

---

### C-2: Wilson's Algorithm Complexity Caveat (Karypis)

**Problem**: Proposition 2 claims Wilson's algorithm runs in $O(|V_H|)$ expected time. This holds for well-connected graphs but not path-like census tract subgraphs (rural counties), where cover time can be $O(|V_H|^2)$.

**Fix**: Add a caveat: "Wilson's algorithm runs in $O(|V_H|)$ expected time for graphs with good expansion properties (e.g., constant spectral gap). For near-path-like subgraphs (rural regions), worst-case expected time is $O(|V_H|^2)$, and the bound should be interpreted as an average-case statement over typical redistricting subgraphs."

Alternatively, state the bound as $O(T \cdot \text{cover}(H) \cdot d)$ where $\text{cover}(H)$ is the cover time of $H$.

---

### C-3: "No Bipartition Failure" Claim — Overstated (Duchin, Karypis)

**Problem**: The abstract and introduction state bipartition failure is "structurally impossible" for BisectionEnsemble. Two issues:
1. Duchin: The correct GerryChain failure mode for large $k$ is not a $1:(k-1)$ balance issue (it is a 1:1 balance within the merged region, constrained by neighboring districts). The paper's causal diagnosis is partially wrong.
2. Karypis: For ApportionRegions nodes with $p'>2$, METIS is still used and could produce poor partitions; the "no failure" guarantee applies only to the local ReCom steps.

**Fix (two-part)**:
1. Correct the causal account of GerryChain's TX failure: the problem is not an inherently extreme balance ratio, but a low-probability spanning tree configuration caused by the geometric constraints of 36 surrounding districts. The paper's current description ($1:37$ balance) is the wrong framing.
2. Add a scoping sentence: "The bipartition failure guarantee applies to the local ReCom steps at each bisection node. The METIS calls at $p'>2$ ApportionRegions nodes are not covered by this guarantee."

---

### C-4: GerryChain Experiment Reproducibility (Liang, Duchin)

**Problem**: Table 1 acceptance rates are not reproducible as stated. GerryChain version, configuration (tolerance $\varepsilon$, ReCom variant), initial plan, random seed, and number of runs are unspecified. The "<1% acceptance" figure appears to be from a single run.

**Fix**: Add a reproducibility appendix or footnote specifying:
- GerryChain version (pip-installed tag or commit hash)
- Population balance tolerance $\varepsilon$
- ReCom variant used (reversible/non-reversible)
- Initial plan
- Number of independent runs
- Hardware and random seed

Consider running at least 10 independent GerryChain runs on TX and reporting the distribution of acceptance rates, not a single figure.

---

### C-5: All Empirical Results Are Single-Run Observations (Liang)

**Problem**: Tables 2, 3, and 4 present point estimates from what appear to be single BisectionEnsemble runs. For a stochastic method, this is insufficient. The WI one-seat shift at $p=0.5$ (Table 3) may be within the variance of a single run.

**Fix**: Run at least 10 independent BisectionEnsemble runs per method per state (each run uses a different METIS initialization seed). Report:
- Table 2: Mean PP across runs ± standard deviation
- Table 3: Fraction of runs yielding each seat count (not a single point estimate)
- Table 4: Runtime distribution (mean ± SD across runs)

---

### C-6: Incorrect Data Citation for Partisan Returns (Liang)

**Problem**: Kuriwaki (2023) is cited for "2020 presidential two-party precinct returns interpolated to census tracts." Kuriwaki (2023) is the CCES cumulative survey file, not precinct returns.

**Fix**: Replace with the correct citation for 2020 precinct-level presidential returns. The Voting and Election Science Team (VEST) at Harvard Dataverse provides precinct-level 2020 general election returns. Specify the interpolation method (areal weighting, dasymetric mapping, or other).

---

### C-7: *Rucho* Citation Mischaracterizes the Holding (Stephanopoulos)

**Problem**: Section 5.1 cites *Rucho v. Common Cause* for the "political blindness requirement" for neutral algorithms. *Rucho* held the opposite: federal courts have no jurisdiction over partisan gerrymandering claims. It does not articulate a standard for neutral algorithms.

**Fix**: Replace the *Rucho* citation with state court cases that do articulate neutrality standards: *League of Women Voters v. Commonwealth* (Pa. 2018), *Common Cause v. Lewis* (N.C. 2019), and *Harper v. Hall* (N.C. 2022). Add a sentence acknowledging that post-*Rucho*, partisan gerrymandering claims are litigated in state courts under state constitutional provisions.

---

### C-8: Ergodicity Invoked as Established in Legal Section (Duchin)

**Problem**: Section 5.1 states "By the ergodicity of the ReCom chain, the distribution of accepted plans converges to the stationary distribution." Ergodicity of the local chain is not proved in this paper (acknowledged in Future Work). Using an unproved theorem as a legal argument is problematic.

**Fix**: Weaken the statement to: "To the extent the local chain mixes — a property whose formal proof is left to future work — the distribution of accepted plans approximates the stationary distribution over feasible bisections. Empirically, we observe [trace plot metric] suggesting reasonable mixing at $T=100$."

Consider adding a trace plot of edge-cut values over 100 steps for one node of one state as evidence of approximate mixing.

---

## Secondary Fixes (Should Address)

### S-1: METIS Seed Selection Protocol Unspecified (Stephanopoulos)

**Problem**: The paper lists the METIS seed as a required audit log disclosure but does not specify how the seed is selected. If the seed is chosen after inspecting results, the "pre-committed" argument is compromised.

**Fix**: Add: "The METIS seed for each node is drawn from the system CSPRNG before any plans are computed. The seed is logged before the first plan is generated."

---

### S-2: Multi-Seed METIS Baseline Missing (Rodden, Liang)

**Problem**: The natural comparison for BisectionEnsemble is not "single METIS call" but "100 independent METIS calls with different seeds, selecting the median." This baseline isolates the contribution of the ReCom chain from the contribution of sampling multiple solutions.

**Fix**: Add a fourth comparison method to Tables 2 and 3: MultiSeedMETIS($N=100$), which runs METIS 100 times with different seeds at each node and selects rank $\lfloor p \times 100 \rfloor$. If BisectionEnsemble outperforms MultiSeedMETIS, the ReCom chain is providing substantive benefit. If they are comparable, the contribution of ReCom is as a convenient single-chain alternative to multiple METIS seeds.

---

### S-3: Partisan Sensitivity Across $p$ Values — Reframe Legal Argument (Stephanopoulos)

**Problem**: The paper argues that NC's stable partisan outcome across $p$ values proves percentile selection is neutral. This argument is vulnerable to the WI one-seat shift counter-example.

**Fix**: Replace the retrospective NC stability argument with a prospective sensitivity analysis commitment: for any state where BisectionEnsemble is deployed in litigation, run the full sweep $p \in \{0.0, 0.25, 0.5, 0.75, 1.0\}$ and disclose the distribution of seat counts across $p$ values. A result that is stable across $p$ is evidence of neutrality; a result that varies with $p$ must be disclosed as such.

---

### S-4: Broader State Sample for Partisan Claims (Rodden)

**Problem**: Three states (NC, WI, TX) are insufficient to establish partisan neutrality as a general property.

**Fix**: Extend the empirical evaluation to at least 5-7 states, including at least one strongly Democratic-leaning and one strongly Republican-leaning state (to test whether the method has directional bias), and at least one additional prime-$k$ state (PA, $k=17$) to verify the tractability claim beyond TX.

---

### S-5: Locality of Root-Level Ensemble Is Misleading (Duchin)

**Problem**: The paper claims "local feasibility sampling" as a key advantage. But at the root node, the local subgraph is $\approx n/2$ tracts — half the state. This is not meaningfully "local."

**Fix**: Qualify "local" relative to tree depth: "At depth $\ell$, each node manages $\approx n/2^\ell$ tracts. The root node ($\ell=0$) manages half the state, while leaf nodes manage $\approx n/k$ tracts. The locality advantage grows with depth."

---

### S-6: $m=0$ Edge Case in Definition 1 (Karypis)

**Problem**: The Select step in Definition 1 indexes into a set of $m$ accepted plans. If $m=0$, the index is undefined.

**Fix**: Add: "If $m=0$, return the METIS initialization $\pi_0$ as the fallback. By Theorem 1, $m=0$ has probability zero in the limit, but may occur for small $T$."

---

### S-7: Audit Log — Ensemble Distribution Preservation (Stephanopoulos)

**Problem**: The paper does not specify whether the JSONL audit log preserves the full ensemble of accepted plans or only the final selected plan. Opposing experts may need the ensemble distribution to independently verify the percentile selection.

**Fix**: Add to the Disclosure requirements: "The full sequence of accepted plans $\Pi$ (or their edge-cut values and population assignments) is optionally logged with the `--log-ensemble` flag. This enables independent verification of the percentile selection."

---

## Recommended Section-by-Section Changes

### Abstract
- Add "on TX, WI, and NC" after the compactness improvement claim.
- Replace "bipartition failure is structurally impossible" with "bipartition failure at the local ReCom level is structurally impossible."

### Section 1 (Introduction)
- Fix the bipartition failure causal account (C-3): the issue is not $1:(k-1)$ balance but geometric constraint from surrounding districts.
- Qualify "always 2-way" claim to "always 2-way at bisection nodes."

### Section 3.4 (Theorem 1)
- Rewrite proof per C-1.
- Add Wilson's algorithm caveat per C-2.
- Fix $m=0$ edge case per S-6.

### Section 4 (Results)
- Add multi-run statistics per C-5.
- Fix partisan data citation per C-6.
- Add MultiSeedMETIS baseline per S-2.
- Add additional states per S-4 (if feasible before revision deadline).

### Section 5 (Legal Properties)
- Fix *Rucho* citation per C-7.
- Weaken ergodicity claim per C-8.
- Add METIS seed protocol per S-1.
- Reframe percentile sensitivity argument per S-3.
- Add ensemble distribution logging note per S-7.

### Section 6 (Conclusion)
- Remove or qualify "bipartition failure is structurally impossible" — use the scoped language from C-3.

---

## Priority Order for Revision

| Priority | Item | Effort | Section |
|---|---|---|---|
| P0 | C-7: Fix *Rucho* citation | Low | §5 |
| P0 | C-6: Fix partisan data citation | Low | §4 |
| P0 | C-3: Scope "no failure" claim | Low | §1, §3, Abstract |
| P1 | C-1: Rewrite Theorem 1 proof | Medium | §3.4 |
| P1 | C-2: Wilson's algorithm caveat | Low | §3.4 |
| P1 | C-8: Weaken ergodicity claim | Low | §5 |
| P1 | S-1: METIS seed protocol | Low | §5 |
| P1 | S-6: $m=0$ edge case | Low | §3.2 |
| P2 | C-5: Multi-run statistics | High | §4 |
| P2 | C-4: GerryChain reproducibility | Medium | §4 |
| P2 | S-2: MultiSeedMETIS baseline | High | §4 |
| P3 | S-3: Reframe percentile sensitivity | Medium | §5 |
| P3 | S-5: Qualify "local" by depth | Low | §1, §3 |
| P3 | S-7: Audit log ensemble option | Low | §5 |
| P4 | S-4: Broader state sample | Very High | §4 |

---

## What Does NOT Require Revision

- The core algorithm (Definition 1, Algorithm 1) is accepted by all reviewers.
- The contiguity proof (Proposition 1) is accepted.
- The complexity bound direction ($O(Tn\log n)$) is accepted, pending the Wilson caveat.
- The compactness improvement direction (Tables 2) is plausible.
- The parallelism strategy (Rayon work-stealing) is accepted.
- The runtime measurements (Table 4) are plausible pending hardware disclosure.
- The disclosure requirements list in §5.3 is well-drafted.
- The population balance / *Wesberry* analysis in §5.2 is correct.
