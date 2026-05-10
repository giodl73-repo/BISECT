# R2 Review — Moon Duchin
**Paper**: U.1 ConvergenceSweep: T=600 Statutory Seed Formula
**Round**: 2
**Score**: 3.5/4
**Verdict**: Accept with minor revisions

## Assessment of P1 Items from R1

### EC_norm definition inconsistency — RESOLVED

The revision adds a bifurcated definition of EC_norm directly in Section 2 (the notation subsection of the algorithm section) with clearly labelled paragraphs: "Recursive bisection context" and "Full k-way partition context." The recursive-bisection normalisation retains the level-indexed formula with denominator sqrt(min(i, k_l - i)); the flat k-way normalisation uses EC(Pi)/sqrt(k/2) for states where METIS is called as a direct k-way partitioner. Crucially, a Remark on cross-state comparability is added immediately after, stating that convergence tails tau are comparable across states only when the same normalisation is applied throughout the sweep, and that the B.7 dataset uses recursive-bisection normalisation for all states receiving a recursive-bisection call and flat k-way normalisation for Nebraska and New Mexico. The table legend is updated accordingly. This resolves the definitional gap completely. The comparability concern I raised is addressed directly and specifically.

### Theorem 1 exponent — RESOLVED

Theorem 1 is now correctly stated as O(n^{2(k-1)}), matching the argument given. The proof sketch is updated: the O(n^2) cuts-per-level argument is now attributed to the Euler formula bound on edge-cut sets with a cross-reference to the planar separator theorem rather than conflating s-t cuts with balanced bisection cuts. The proof sketch correctly states that a k-way recursive bisection tree has k-1 internal nodes, each contributing O(n^2) choices, giving O(n^2)^{k-1} = O(n^{2(k-1)}). A remark is added noting that the bound O(n^{2(k-1)}) is an existence result and astronomically large for US congressional graphs — the empirical B.7 evidence is more informative. The two errors I flagged (wrong exponent, Euler-formula conflation) are both corrected.

### Gumbel i.i.d. assumption — RESOLVED

The revision adds three substantive items: (1) an "Exchangeability assumption" paragraph explicitly stating that the 50 observed tails are treated as approximately exchangeable draws from an underlying distribution of congressional graph convergence tails, and that this is an empirical assumption — not a consequence of extreme-value theory — requiring future US congressional graphs to resemble the 50 current state structures; (2) a "Goodness-of-fit assessment" paragraph reporting a KS test against the fitted Gumbel(mu=200, sigma=150) distribution, yielding KS statistic D=0.11 with p≈0.52 (no significant departure at 5%); and (3) explicit statements in both the Gumbel subsection and Theorem 2 that the statutory recommendation rests primarily on the empirical 89-seed margin, with the Gumbel tail bound as supporting evidence only. The exchangeability language is precise — it says "approximately exchangeable" and notes the limitations correctly. The KS test result is reported with sufficient detail for independent verification. This addresses all three sub-items of my R1 concern.

## Remaining Concerns

### SHA-256 concatenation order (P2) — ADDRESSED

My P2 concern about byte-level concatenation order is resolved: Definition 2.2 and the statutory text in Section 5.4 now both specify the SHA-256 input as census_release_id followed by "DIA_SEED_V1" with byte concatenation, and the floor-of-big-endian interpretation is stated explicitly. This was a P2 item and is now handled.

### Null hypothesis for T=600 adequacy (P3) — NOT ADDRESSED

My P3 request for an explicit null hypothesis statement is not present in the revised sections. The introduction states "the probability of missing a better plan is below 10^{-3}" and the conclusion references the Gumbel model, but neither adds a formal null hypothesis in the form I requested (e.g., "the B.7 50-state dataset contains all worst-case graph structures; the 2030 census will not produce a state with tau > 600"). This is a P3 item and its absence does not block acceptance, but a sentence in the conclusion situating the Gumbel bound as evidence against the complement of the null would sharpen the paper for readers accustomed to frequentist framing.

## Recommendation

Accept with minor revisions. All three P1 items from my R1 review are resolved adequately. The EC_norm bifurcation is clean and the comparability remark is exactly what was needed. The Theorem 1 proof sketch is now internally consistent. The Gumbel exchangeability paragraph, KS test result, and explicit primacy of the empirical margin together address the statistical concern I raised. The one outstanding item (null hypothesis framing) is a P3 refinement that does not affect the paper's central claims. The mathematical errors that would have been embarrassing in a court citation are corrected; the paper is ready to support the contingency footnote removal in G.0/G.14/U.8.
