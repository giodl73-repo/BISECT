# Review: NestSection — George Karypis

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: Graph partitioning, multilevel algorithms, METIS, tree structures, sparse linear solvers
**Round**: 1
**Score**: 3/4 (accept with revisions)
**Recommendation**: Accept with Major Revisions

---

## Summary

NestSection proposes a hierarchical redistricting framework that forces congressional, state senate, and state house districts to share a common bisection tree prefix — the "compatible factorization spine" — determined by gcd(C, S, H). The paper includes a clean algorithm (CompatibleSpines), a scoring function (sigma), and an interesting empirical census of all 50 states. I am broadly positive about this work. The connection to METIS-based recursive bisection is the right framing, and the authors clearly understand the graph-partitioning substrate. However, several technical gaps need to be addressed before this is ready for publication.

---

## Strengths

1. **Clean algorithmic core.** The CompatibleSpines algorithm (Algorithm 1) is polynomial and correct. The proof of Proposition 1 (correctness) is sound. The observation that the trunk length equals the GCD prime factorization is the right abstraction.

2. **Bimodal empirical finding.** The complete absence of states in the 0 < sigma < 50 range is a genuine surprise worth documenting. The paper's structural explanation (gcd > m/2 requires gcd = m in current apportionments) is correct and worth formalizing.

3. **Oregon as the canonical example.** The gcd(6,30,60)=6 case is elegant and should be the anchor figure throughout.

4. **Integration with GeoSection.** Building on GeoSection (B.8) is sensible; the prior work handles the geographic balance problem at each recursion level.

---

## Major Concerns

### M1: The trunk is not geometrically canonical

The paper defines the trunk as the prime factorization of g in non-decreasing order. This is mathematically clean, but it conflates the combinatorial and geometric aspects of the spine construction. In METIS-based bisection, the order in which you apply factors matters geometrically: splitting into 2 then 3 is not the same as splitting into 3 then 2 on a planar census graph. The trunk $\tau = [2,3]$ for Oregon produces different geographic regions depending on which split comes first.

The paper needs to either (a) acknowledge that the trunk sequence defines a fixed recursion order that determines the geographic outcomes, or (b) prove that the spine is unique up to reordering of factors (which it is not, in general). The current claim that the trunk is the "longest common prefix that can be guaranteed" (Proposition 1, part 3) is about combinatorial sequences, not geographic regions. This is an important distinction.

**Required fix**: Add a remark clarifying that the canonical non-decreasing ordering is a convention, and that different orderings of the same prime factors produce different geographic spines. The paper should acknowledge this as a degree of freedom and suggest that the "best" ordering (by some geographic criterion, e.g., minimizing inter-trunk edge cut) is a separate optimization problem.

### M2: The NestSection algorithm (Algorithm 2) assumes exact divisibility at every trunk region

Line 6 sets $s_i = S/g$ and line 7 sets $h_i = H/g$ uniformly across all trunk regions. This assumes S and H divide evenly across the g trunk regions, which is true in the combinatorial sense but false in the population sense. After GeoSection partitions the state into g trunk regions by population weight, the trunk regions have approximately equal populations, but the integer seat allocations via ApportionRegions (line 5) may assign different seat counts to different trunk regions.

Specifically, $c_i$ (line 5) may vary across trunk regions by +/-1 seat. If $c_i$ varies, then the tail factorizations vary per region, and the spines are not identical across trunk regions. The paper does not address this. For Oregon with g=6, if one trunk region gets 1 congressional seat and another gets 1 also (all equal since 6/6=1), this is fine — but for Alabama with g=7, C=7, each trunk region gets exactly 1 congressional seat, which also works. But this coincidence needs to be stated explicitly: strict nesting (Mode 1) requires that C/g, S/g, H/g are all integers, which is equivalent to the definition of g = gcd(C,S,H). The paper conflates "the GCD makes these integers" with "all trunk regions get equal seat counts," which requires a bridge argument connecting population balance to seat allocation.

**Required fix**: Add a lemma or remark showing that for Mode 1 states, the integer seat allocations to trunk regions are identical across regions (or are within +/-1, and the tail factorization handles that). The current algorithm implicitly assumes $s_i = S/g$ is constant, but this needs justification given population variation across trunk regions.

### M3: No empirical complexity data

The paper claims in the Complexity subsection that NestSection adds "at most min(C,S,H) additional partitioning operations" over three independent GeoSection runs. This is true in terms of operation count but misleading in terms of wall time. The dominant cost in METIS-based bisection is the multilevel coarsening and uncoarsening passes, which scale with graph size at each level. The shared trunk means the first g partitioning operations work on the full census graph (all tracts), which is the most expensive phase. Three independent runs do not share this cost.

The claim needs either empirical timing data or a more careful asymptotic analysis. For a state like Oregon with 700,000 people in each trunk region and ~3,000 census tracts per region, the trunk-level GeoSection calls are not negligible.

**Required fix**: Either report actual timing benchmarks comparing NestSection vs. three independent GeoSection runs for at least one state, or add a more careful complexity analysis that accounts for graph-size reduction at each recursion level.

### M4: NestRefine boundary zone assignment is underspecified

The NestRefine procedure (described in §3.4) is invoked in Algorithm 2, line 10, but the paper only says boundary zones "must not exceed tau_pop." The actual assignment rule for census tracts that straddle a senate boundary is "assigned to the larger-overlap chamber." This rule is not formally defined. For a census tract that has equal population in two adjacent senate sub-districts, the tie-breaking rule affects the final district assignments. This is not a minor implementation detail — it is part of the algorithm specification.

**Required fix**: Formally define NestRefine as Algorithm 3, specifying the boundary-zone assignment rule, the measurement of tau_pop, and the tie-breaking convention.

---

## Minor Issues

- The claim that Algorithm 1 runs in O(log(max(C,S,H))) is correct for the GCD computation, but the prime factorization step is slightly more expensive (O(sqrt(max(C,S,H))) for trial division). For congressional seat counts up to 52, this is negligible in practice, but the complexity claim should be precise.

- Figure 1 (Oregon NestSection map) is a placeholder box. The paper would be significantly stronger with even a schematic diagram showing the 2-split, 3-split, 5-split, 2-split recursion tree, even without geographic data.

- The Mode 2 nesting (0 < sigma < 50) has zero empirical instances in current US apportionments. Defining a mode with no examples is a theoretical exercise. Either justify why Mode 2 is worth defining for future apportionments, or fold it into the discussion and remove it from the main mode taxonomy.

- Section 4.3 describes the NC nesting violation as "population fraction in boundary zones is determined empirically by the geographic census-tract distribution; the plan specification reserves tau_pop <= 0.04." This 4% figure appears without derivation or reference. How was 4% determined?

---

## Assessment

This is a solid first paper on an important algorithmic problem. The core technical contribution — the CompatibleSpines algorithm and the compatibility score — is clean and correct. The 50-state census is interesting and the bimodal finding is worth publishing. My major concerns are about gaps in the algorithm specification (M2, M4) and the absence of empirical validation (M3). These are fixable with moderate effort. The paper should not be published without at least one actual running example on census data, even for a small state like Vermont (C=1, trivially strict but a good sanity check for the pipeline) or Oregon.

**Score: 3/4 — Accept with major revisions.**
