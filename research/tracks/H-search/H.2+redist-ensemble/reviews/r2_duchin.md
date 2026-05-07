# Review — H.2: redist-ensemble
**Reviewer**: Moon Duchin (Geometric Group Theory, Mathematical Foundations of Redistricting)
**Round**: 2
**Score**: 3.5 / 5

---

## Response to Round 1 Concerns

### R01 (CRITICAL): Texas framing — Substantially addressed

The Table 1 footnote is now: "\gc{} \emph{without} pair reselection fails on TX ($k=38$) and CA ($k=52$) from cold start due to bipartition infeasibility; \gc{} \emph{with} pair reselection and \re{} both handle these cases via pair reselection." It then adds: "GerryChain's bipartition failures on TX $k=38$ are combinatorial, not language-specific. Both GerryChain-with-pair-reselection and \re{} succeed. The advantage of \re{} is throughput, not correctness."

This is exactly the correction I requested. The framing is now accurate and will withstand scrutiny from a hostile expert. The distinction between GerryChain-with and GerryChain-without pair reselection is now explicit.

The Section 7.1 conclusion has also been revised: "The TX and CA cold-start challenge is combinatorial, not language-specific: \gc{} with pair reselection also handles these states. \re{}'s advantage is that Rust's throughput reduces the wall time for the pair-reselection warm-up phase from seconds (Python) to microseconds." This is the correct framing.

**One remaining issue with Section 5.4 (Texas and California subsection)**: The first paragraph of Section 5.4 still reads: "\re{}'s pair-reselection mechanism enables it to traverse the low-feasibility region rapidly." This sentence, standing alone, retains ambiguity about whether pair reselection is a Rust-specific capability. The revised language in Section 4.5 and the Table 1 footnote corrects this at the implementation and results levels, but the Section 5.4 prose still needs a clause noting that GerryChain with pair reselection would achieve the same traversal, albeit more slowly. This is a minor consistency fix.

### R13 (MODERATE): CA per-step cost — Fully addressed

The revised Section 5.4 now correctly identifies PA ($m \approx 379$) as having the largest average merged region, not California ($m \approx 310$). The formula $O((n/k)\log(n/k))$ is applied correctly. This correction is accurate and well-executed.

### R02 (CRITICAL): Pair reselection and stationarity — Adequately addressed for Round 2

The new stationarity paragraph in Section 4.5 acknowledges the issue, provides the informal detailed-balance argument, and correctly defers formal proof to future work. For the purposes of this paper's legal framing, the key requirement was to acknowledge that stationarity under pair reselection is a conjecture pending verification — the revision does this. The hedge "a formal ergodicity analysis is deferred to future work" is appropriate.

I concur with Karypis's observation that the detailed-balance argument as stated is not tight (the compound 10-tree protocol is not a single-step Metropolis rejection). The language "preserves detailed balance if the pair-selection distribution is symmetric" should be softened to "is consistent with preservation of detailed balance under the following informal argument." This is a one-sentence fix.

---

## Remaining Minor Concerns

### Section 5.4 consistency (noted above)

The opening of the Texas/California subsection still implies pair reselection is a \re{}-specific solution. One sentence should be added to clarify that GerryChain with pair reselection also traverses the low-feasibility region; \re{}'s advantage is speed, not capability.

### Hamming autocorrelation definition (R14 — not addressed)

Section 6.3 still defines $\ham(k)$ using $d_H(\sigma_t, \sigma_0)$ (distance from the starting plan) rather than plan-to-plan Hamming distance $d_H(\sigma_t, \sigma_{t+k})$. I flagged this in Round 1 as a nonstandard definition. The revision plan listed it as Minor (R14), and it remains uncorrected. For mathematical precision, the paper should clarify whether the autocorrelation is computed relative to a fixed reference plan or as a lag-$k$ plan-to-plan correlation.

### Cold-start definition still missing

Section 5.4 still does not define what constitutes a "cold start" for the TX experiments — i.e., what the initial partition is for $k=38$ Texas. This was flagged as a minor issue in Round 1. Generating a valid random starting plan for $k=38$ is non-trivial; the paper should state whether it uses a seeded random plan, a grid-based partition, or some other initialization.

---

## What the Revision Gets Right

The Table 1 footnote and Section 7.1 correction together constitute the most important change in this revision. The paper's credibility in a litigation context depends on not overstating what pair reselection achieves, and the revision accomplishes this. The per-step cost correction (PA > CA) is accurate and demonstrates that the authors have checked their own formula.

---

## Recommendation

**Minor revision** (upgraded from Major Revision). The critical TX framing issue is resolved. The stationarity issue is adequately hedged. Two sentence-level fixes are needed before acceptance: (1) the Section 5.4 opening sentence should be brought into consistency with the corrected Table 1 footnote; (2) the stationarity argument should be softened from "preserves detailed balance" to "is consistent with preservation of detailed balance." The Hamming autocorrelation definition and cold-start omission are minor issues that can be addressed in a final editing pass.
