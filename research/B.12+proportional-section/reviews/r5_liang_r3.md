# Review: B.12 ProportionalSection — Round 3
## Reviewer: Percy Liang (Stanford University)
**Expertise**: Reproducibility, evaluation methodology, machine learning systems

---

### Summary

Round 3 makes meaningful progress on the Theorem 2 scope and the Nevada outlier, and the "Scope of Claims" addition is appropriate. The MAUP qualitative result is better than nothing. My Round 2 reproducibility concern (seed determinism for Table 1) remains unaddressed, which is my only remaining P1 concern.

### Strengths

1. **Theorem 2 scope remark is a correct and necessary addition.** The distinction between contiguous and non-contiguous partitions is technically important and the remark is well-placed.

2. **Nevada explanation is mechanistically satisfying.** Low-k granularity as the mechanism is specific and testable. The general principle (small k states are less reliable for B.12) is a useful take-away.

3. **"Scope of Claims" subsection makes the paper more honest.** Item (3) — the algorithm is explicitly partisan — is the most important disclaimer. Researchers and practitioners need to understand that B.12 is not a neutral algorithm.

4. **MAUP paragraph provides the qualitative answer.** "Qualitatively identical: WI improves at η=1.10, NC worsens at all η" at block-group resolution is the key claim. It would be stronger with quantitative values, but the qualitative result is sufficient for the paper's scope.

### Weaknesses

1. **Table 1 seed determinism still unresolved — my Round 2 P1.** The paper specifies "30 seeds per run; maximum deviation reported as the worst seed." This implies variance exists (otherwise why report the maximum?). If there is variance, the paper needs confidence intervals or at minimum the range (max − min) across seeds. If results are deterministic (zero variance), the paper should say "all 30 seeds produce identical outcomes" and use "maximum" only as a confirmation check. This is the single remaining item blocking my acceptance recommendation.

2. **MAUP quantitative values absent.** The paragraph says "qualitatively identical" but does not report σ at block-group resolution for WI or NC. For a reproducibility-focused review, "qualitatively identical" is not sufficient — I need to know whether σ_WI changes from 0.003 to 0.003 or from 0.003 to 0.031. The former supports the robustness claim; the latter would require a different framing.

3. **Repository or data URL still not provided.** The paper cites VEST/Fekrazad for vote data but does not provide a direct DOI or URL for the specific dataset used. For 2020 presidential data specifically, the VEST Harvard Dataverse entry should be cited.

### Questions for Authors

1. For Table 1: are the reported Δ values deterministic across all 30 seeds, or are they averages/maxima of a distribution? If there is variation, report it.

2. At block-group resolution, what are the σ values for WI and NC? Do they remain in the "free proportionality" category?

### Suggestions

- **P1**: Clarify Table 1 seed determinism explicitly — either "all 30 seeds produce identical outcomes" (deterministic) or report range/SD alongside the reported values.
- **P2**: Report σ values at block-group resolution for WI and NC.
- **P2**: Add Harvard Dataverse DOI for the specific VEST 2020 presidential precinct returns used.

### Verdict

[~] Accept with Minor Revisions (conditional on Table 1 seed determinism clarification)

**Rationale**: The Round 3 revisions address Theorem 2 scope, Nevada, Scope of Claims, and MAUP qualitative. My Round 2 P1 reproducibility concern (seed determinism) remains. This is a one-sentence fix if the results are deterministic, or a table revision if there is variance. I will not require another full round if the authors add the clarification at the journal submission stage.

**Score: 3.4 / 4.0** (up from 3.0)
