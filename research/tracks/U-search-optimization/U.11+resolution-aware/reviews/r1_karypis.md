# R1 Review — George Karypis
**Paper**: U.11 Resolution-Aware Redistricting: Geographic Granularity as a First-Class Parameter
**Score**: 2/4
**Verdict**: Major Revision

## Summary
The paper makes geographic resolution a first-class parameter throughout a redistricting pipeline, introducing GEOID prefix-matching functions for fine-to-coarse partition derivation and multi-scale Markov chain options. The central claim is that the complete fine-to-coarse mapping is derivable from existing GEOID data with no additional files. Empirical validation on Texas shows a 27% reduction in lag-100 autocorrelation for Option B multi-scale chains, though this figure requires stronger validation.

## Strengths
- The GEOID prefix-matching insight is clean and correct: the Census Bureau encoding genuinely encodes hierarchy, and the paper formalizes this well.
- The correctness proof for `derive_partition` (Theorem 1) is properly structured with existence, uniqueness, and completeness arguments.
- The manifest audit extension is practically valuable: recording `plan_resolution`, `n_units`, and `fine_to_coarse_formula` enables independent verification without pipeline code.
- Option B requiring no extra data files is a genuine engineering virtue that the paper correctly highlights.

## Concerns
- **GEOID prefix derivation correctness**: Definition 1 asserts that the prefix property holds as "an invariant of the Census Bureau FIPS encoding." This is stated without citation to a formal Census specification that guarantees this across all census years (2000, 2010, 2020). If any census year has nonstandard GEOID assignments (e.g., tract splits/merges causing GEOID reassignment), the prefix property may fail for cross-year use. The paper should either restrict the claim to within-year use or provide a citation to the official FIPS encoding specification.
- **County adjacency criterion**: Theorem 2 proves that the criterion "counties A and B are adjacent iff some tract in A is adjacent to some tract in B" correctly propagates geographic adjacency. The forward direction of the proof relies on the claim that "any county boundary segment is covered by a sequence of tract boundary segments." This is geometrically plausible but not formally proven — it depends on the fact that TIGER tracts tile their county exactly, without gaps or overlaps along county boundaries. This should be cited or proven.
- **O(|E_T|) efficiency claim**: The complexity analysis claims `build_county_coarsening` runs in $O(|E_T| + n_T + n_C)$, described as "linear in the tract graph size." However, the deduplication step is stated as $O(|E_T| \log |E_T|)$ via sort-and-unique, which is superlinear. The paper resolves this by noting that hash-set deduplication achieves $O(|E_T|)$ expected, but this is a probabilistic bound under the hash function. The $O(|E_T| \log |E_T|)$ worst case should be stated clearly in the complexity theorem, or the hash-set version should be elevated to the main theorem with appropriate caveats.
- **Option B as the only "data-free" option**: The paper repeatedly emphasizes that Option B requires no extra data. However, it does require that the tract adjacency file contains GEOIDs, which is not universally guaranteed by the current pipeline's file format. The paper should verify that the GEOID field is always present in `{state}_adjacency_{year}.pkl` files and state this as a precondition.

## Required Changes (P1/P2)
- **P1**: Restrict GEOID prefix correctness claim to within-year use, or cite the official Census FIPS specification guaranteeing year-invariant prefix structure.
- **P1**: In Theorem 2 forward direction, add a citation or brief argument that TIGER tracts tile their county without boundary gaps — the proof as written has a geometric gap.
- **P1**: Clarify the complexity of `build_county_coarsening`: state the worst-case $O(|E_T| \log |E_T|)$ and the expected $O(|E_T|)$ separately; do not conflate them in the theorem statement.
- **P2**: Confirm that the GEOID field is a guaranteed field in the adjacency file format and state it as a precondition in Section 2.
- **P2**: The 27% autocorrelation reduction from a single 2000-step run is footnoted as estimated. Before final submission, either provide multi-run validation or reframe this figure as a preliminary observation rather than a headline result.
