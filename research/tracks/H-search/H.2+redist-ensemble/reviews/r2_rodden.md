# Review — H.2: redist-ensemble
**Reviewer**: Jonathan Rodden (Political Geography, Partisan Patterns in Redistricting)
**Round**: 2
**Score**: 3.5 / 5

---

## Response to Round 1 Concerns

### R07 (MAJOR): R-hat convergence semantics for redistricting — Not addressed

My principal concern from Round 1 was that the paper presents R-hat < 1.05 as indicating "convergence" for redistricting applications without acknowledging that this certifies convergence of marginal statistics, not of the underlying plan-space distribution. Section 6.1 and Section 1.1 remain unchanged on this point. The revision plan identified this as a required addition ("Add a paragraph acknowledging that R-hat diagnostics applied to summary statistics certify convergence of the marginal distribution of those statistics, not convergence of the full plan-space distribution"), but it does not appear in the revised paper.

The specific sentence in Section 1.1 that needs revision: "the computational cost discourages the thorough convergence diagnostics — multiple parallel chains, extended runs — that would give courts and litigants justified confidence in the ensemble's stationarity." The revision plan called for changing "courts and litigants" to "researchers and practitioners" and qualifying "stationarity" as referring to the marginal distributions of summary statistics. This edit has not been made.

This remains a required fix for the legal framing of the paper. A court-facing expert report that claims R-hat < 1.05 demonstrates "convergence" will be challenged by opposing counsel, correctly, on the grounds that marginal-statistic convergence does not imply plan-space exploration.

### R11 (MODERATE): GerryChain baseline hardware and reproducibility — Not addressed

Section 5.1 still describes the GerryChain measurements as "on a standard development workstation" without specifying CPU, Python version, NumPy version, or GerryChain configuration (sparse adjacency backend). The revision plan listed this as a Moderate issue. It remains unaddressed.

### R08 (MODERATE): Hamming proxy exactness claim — Not addressed

Section 6.3 still states the normalized cut fraction proxy is "exact in the sense that $\phi$ is the same summary statistic minimized by METIS." The revision plan called for replacing this with: "$\phi(\sigma)$ is used as a computationally efficient summary for tracking chain progress; it is not a collision-free plan identifier, but its correlation with plan-to-plan Hamming distance makes it a useful running diagnostic." This correction has not been made.

The word "exact" in this context is incorrect. Two plans can have identical $\phi$ while differing in boundary placement. Using $\phi$ as a proxy is defensible, but calling it exact is not.

---

## What Was Addressed

The TX framing corrections (Table 1 footnote and Section 7.1) are outside my primary domain, but they directly affect the political-science credibility of the paper's claims. The revised Table 1 footnote correctly distinguishes GerryChain-with and GerryChain-without pair reselection, and the conclusion accurately frames Rust's advantage as throughput rather than correctness. These changes improve the paper's credibility with political-science audiences who will recognize the TX bipartition failure mode as a known algorithmic property.

The conclusion's revised framing of the "real-time audit" contribution is also improved. The paper now avoids the overclaim that throughput improvement directly improves court admissibility — though Section 1.1 and Section 7.2 still contain language that blurs the AEA-replication benefit with the legal-confidence benefit (an issue also raised by Stephanopoulos).

---

## Remaining Priority Issues

1. **R07** (R-hat semantics): Section 6.1 needs a paragraph distinguishing marginal-statistic convergence from plan-space convergence. Section 1.1 needs the "courts and litigants" framing softened to "researchers and practitioners" with the convergence claim qualified. These are the most important remaining edits.
2. **R08** (Hamming proxy "exactness"): Remove the word "exact" and replace with the description of $\phi$ as a useful but non-collision-free proxy.
3. **R11** (Baseline hardware): Add Python version, NumPy version, GerryChain configuration to Section 5.1. This is a one-sentence addition.
4. **R17** (ESS formula): $\hat{\rho}_k^+$ is still used in the ESS formula without an inline definition. A parenthetical "(the rank-normalised autocorrelation; see Section~\ref{ssec:rhat})" would close this gap.

---

## Recommendation

**Minor revision** (score unchanged from Round 1). The TX framing correction is the right move and earns the paper's political-science credibility. However, the R-hat convergence qualification (R07) is the paper's most significant remaining correctness issue and must be addressed before the paper's legal framing is defensible. The Hamming proxy correction (R08) and baseline hardware specification (R11) are straightforward one-sentence fixes. None of these require new analysis — they are editorial corrections to existing text.
