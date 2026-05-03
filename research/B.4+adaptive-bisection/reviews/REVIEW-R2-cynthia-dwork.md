# Review (Round 2): Edge-Weighting Makes Method Selection Irrelevant

**Reviewer**: Cynthia Dwork (Harvard University)
**Expertise**: Algorithmic fairness, optimization theory, computational complexity, differential privacy
**Round**: 2 (Recheck after major revision)
**Date**: 2026-05-02

## Overview of Revision

In Round 1, I raised three major concerns: (M1) the paper lacked connections to the algorithmic fairness literature, (M2) fairness properties were not formalized, and (M3) no empirical evidence existed for a fairness phase transition beyond α = 5. The revision also addresses my key question — "Can we formalize when edge-weighting dominates algorithmic choice?" — through the new Theorem 1 (Method Equivalence Conditions) with explicit conditions C1–C3 and a closed-form α_crit.

---

## Response to Major Issues

### M1: Fairness Theory Background — ADDRESSED

The new sections 02.4_fairness_background.tex and 07.4_fairness_guarantees.tex are a welcome addition. The connection to individual fairness, algorithmic determinism as a fairness property, and the gaming-resistance framing are all appropriately positioned. The citation to Dwork et al. (2012) is correct in context.

I am satisfied with the structural additions. My remaining concern on this front is that the connection between the formal Theorem 1 conditions (C1–C3 as graph-structural properties) and the fairness formalization (Property 1: Algorithmic Determinism) is stated but not explicitly derived. The revision adds both components but treats them as parallel developments rather than showing that (C3) uniqueness implies Property 1. A single bridging sentence or corollary in Section 7.4 that cites Theorem 1 Part 2 (Partition Uniqueness) would close this gap cleanly.

### M2: Fairness Properties Formalized — ADDRESSED

The formalization of three properties (Algorithmic Determinism, Gaming Resistance, Transparency-Fairness Equivalence) in Section 7.4 addresses my request. Property 1 now reads as a formal statement rather than an informal claim. The verification mechanism discussion is appropriately brief.

One nuance that the revision does not address: Property 2 (Gaming Resistance) is formalized as "no adversary can manipulate outcomes by choosing favorable tree structure or algorithmic approach." This is true for α ≥ α_crit, but the revision does not specify what class of adversary is considered. Is this a computationally bounded adversary? An adversary who can choose any algorithm satisfying balance constraints? For a fairness publication, the adversary model should be explicit. I recommend a single sentence clarifying that the guarantee holds against any algorithm satisfying conditions C1–C3 (not against, e.g., an adversary who can perturb the input graph).

### M3: Phase Transition Empirically Validated — ADDRESSED

The α ablation study (430-row dataset, 10 α values) provides the empirical validation of the phase transition I requested. Zero method variance is confirmed at α ≥ 50. I note, as Hendrickson also observes, that the empirical threshold (α ∈ [20,50]) is higher than the theoretical prediction (α_crit ∈ [3,5] from the original paper, or [11,38] from the Corollary bounds). For a fairness audience, this discrepancy matters: practitioners using α = 5 are operating in a regime where the theory guarantees near-zero but possibly nonzero variance, while zero variance is only confirmed empirically at α = 50.

For redistricting practice, this means the fairness guarantee at α = 5 is a near-guarantee (Obj ≤ Obj* + O(1/α)) rather than an exact guarantee (Obj = Obj*). This should be stated explicitly in Section 7.4 under the discussion of Property 1.

---

## Score: 3.5/4

**Assessment**: Accept with minor revisions. The addition of formal fairness theory is appropriate and the α ablation validates the key phase transition claim. Two minor clarifications are needed: (1) an explicit adversary model in the Gaming Resistance property, and (2) a clarification that α = 5 provides a near-guarantee rather than an exact fairness guarantee. Neither requires additional experiments or major rewriting.

---

## Remaining Minor Issues

**m1 (new)**: The fairness analysis in Section 7.4 should explicitly note that α = 5 achieves empirical equivalence on the original five states but that the zero-variance threshold is α ≈ 50. At α = 5, the formal guarantee is Obj ≤ Obj*(1 + C1/α), which for typical instances is indistinguishable from zero, but this is distinct from exact method equivalence. For a legal context, the distinction matters.

**m2 (new)**: Property 2 (Gaming Resistance) needs an explicit adversary model. Suggested addition: "against any algorithm satisfying the balance constraint and the structural conditions (C1)–(C3)."

**m3 (carried from R1)**: The fairness metrics beyond VRA compliance (representation gap, opportunity parity, demographic parity) are listed in Section 02.4 but it is unclear whether they are computed in the experiments. If they are not, they should be listed as future work rather than as contributions of the current paper.

**m4 (new)**: The NP-hardness remark (Remark rem:nphard) is correct and is a valuable clarification, particularly for a fairness-oriented reader who might otherwise infer that the paper achieves polynomial-time optimal solutions. No changes needed here — I am noting this for the record as a positive addition.

---

## Summary

The revision substantially addresses my Round 1 concerns. The formal conditions C1–C3 directly answer my key question about when edge-weighting dominates algorithmic choice, and the addition of fairness-specific sections adds context appropriate for FAT*/FAccT audiences. The remaining issues are minor and can be addressed in copy-editing.

## Recommendation

**Accept with minor revisions** (non-blocking).

## Conflicts of Interest

None. I work on algorithmic fairness theory but not specifically on redistricting applications.
