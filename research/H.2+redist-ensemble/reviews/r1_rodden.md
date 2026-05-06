# Review — H.2: redist-ensemble
**Reviewer**: Jonathan Rodden (Political Geography, Partisan Patterns in Redistricting)
**Round**: 1
**Score**: 3.5 / 5

---

## Summary

This paper presents a Rust reimplementation of the GerryChain ReCom Markov chain, targeting a large throughput improvement that would make ensemble analysis practical as an in-pipeline step rather than an overnight computation. The political-science framing and legal motivation are handled competently. My principal concerns are: (1) the translation of MCMC convergence diagnostics from classical statistics to the redistricting setting deserves more scrutiny; and (2) the 2,300× speedup claim, while hedged appropriately with dagger notation, is presented with a confidence that the underlying estimation does not yet support.

---

## On the Redistricting Relevance of R-hat and ESS

Section 6 implements R-hat and ESS for redistricting ensembles, following the G.4 working paper. These are standard Bayesian convergence diagnostics, and applying them to redistricting chains is reasonable as a practical matter. However, the paper's framing that R-hat < 1.05 and ESS ≥ 400 constitute "convergence" for redistricting applications needs qualification.

Classical R-hat and ESS are validated for use with chains whose stationary distribution is a target posterior with smooth density. Redistricting chains have a discrete combinatorial state space (the set of valid partitions), which can be astronomically large but also highly structured. The ReCom chain is not reversible in the usual sense, its mixing time depends heavily on the starting plan and the geometry of the plan space, and the scalar summary statistics (seat counts, compactness scores) on which R-hat is computed may converge faster than the underlying plan distribution. In other words, low R-hat on Democratic seat share does not certify that the chain has explored the full space of plans — it certifies only that the marginal distribution of seat share has stabilized across chains.

This distinction matters enormously in a legal context. A court might accept "R-hat = 1.02 for Democratic seat share" as evidence of convergence, but a sophisticated opposing expert would correctly point out that the chain may be stuck in a region of plan space that produces similar seat-share distributions without actually exploring the full feasible set. The paper should explicitly acknowledge this limitation.

The rank-normalization step (Section 6.1, citing Vehtari et al. 2021) is appropriate and represents current best practice for R-hat. Its application to discrete redistricting statistics is a genuine methodological contribution worth highlighting more prominently.

---

## On the Hamming Autocorrelation Diagnostic

The Hamming autocorrelation diagnostic (Section 6.3) is the most redistricting-specific of the three diagnostics and the most interesting. Using the plan-to-reference-plan Hamming distance as a distribution-free mixing diagnostic is well-motivated. The paper states that $\hat{\rho}_1 \approx 0.87$--$0.93$ for ReCom, citing this as typical, but does not provide any evidence for this range from the paper's own implementation. Are these figures from GerryChain runs? From the G.4 working paper? From the literature?

More substantively, the use of the normalized cut fraction $\phi(\sigma)$ as a "computationally efficient proxy for plan identity" when computing Ham(1) is imprecise. The paper asserts that this proxy is "exact in the sense that $\phi$ is the same summary statistic minimized by METIS." This claim is not the right notion of exactness. Two plans can have identical $\phi$ but differ in which edges cross boundaries (one is a permutation of the other's district labels, or they differ in boundary placement but achieve the same cut fraction). Ham(1) computed via $\phi$ is not the same as Ham(1) computed from the full plan assignment. The paper should clarify whether it means something weaker — e.g., that $\phi$ is a sufficient statistic for the purposes of tracking chain progress — or correct the claim.

---

## Honesty of the Speedup Claim

This is the key issue and the paper handles it better than average but not perfectly. The abstract states "roughly 50,000 steps per second — an estimated 2,300× speedup." The word "estimated" is in the text of the abstract, and the dagger notation appears throughout Table 1 with a clear footnote. Section 5.1 is explicit that the Rust figures require criterion.rs validation in Phase 2. The conclusion repeats that "all throughput figures in this paper are complexity-analysis estimates."

This level of hedging is appropriate and I commend it. My concern is with the rest of the abstract and introduction, which present the speedup as if it were already achieved: "enabling ensemble analysis as an integrated real-time audit step" (abstract, line 98-99), "puts ensemble analysis in the same computational tier as compactness scoring" (conclusion). These statements are predictions, not demonstrated results. A reader who skims the abstract and conclusion might not retain the caveat from line 95.

I recommend adding a parenthetical note in the conclusion summary (Section 7.1) that explicitly restates the estimated nature of the throughput improvement, so the caveated state of the claim survives aggressive skimming.

---

## On the Political Science Framing

The opening framing around Rucho v. Common Cause is accurate and appropriately understated. The paper does not overclaim what ensemble analysis proves legally — it correctly frames percentile placement as "evidence of partisan intent" rather than as proof. The reference to "courts and litigants" (Section 1.1) is slightly imprecise; in state-court proceedings, the audience is state courts applying state constitutional law, not federal courts applying federal equal-protection doctrine. This is a minor framing issue.

The claim that a plan at the 99th percentile of Republican seat share is "strong evidence of partisan intent" is the kind of statement that opposing counsel will contest. The paper might note that courts have not universally accepted a specific percentile threshold as legally probative.

---

## Minor Points

- Section 5.2 uses NC as the worked example throughout (m ≈ 191 for the merged region). It would be useful to also show the computation for CA ($m \approx 2 \times 8057 / 52 \approx 310$), where the cover-time scaling is more aggressive and the $13\times$ overhead assumption is more questionable.
- The GerryChain baseline measurements (Table 1) appear to be from the author's own runs of GerryChain, not from published benchmarks. If so, the hardware specification should be stated to allow replication.
- The ESS formula in Section 6.2 uses the multi-chain bulk ESS from Vehtari et al. (2021). The formula as written defines $\hat{\rho}_k^+$ without giving the rank-normalization step that defines it. A forward reference to the R-hat subsection or a brief inline definition would help readers who are not familiar with the Vehtari et al. notation.

---

## Recommendation

Minor revision. The paper is well-organized and the hedging on throughput estimates is commendable. The principal required fix is a more careful treatment of what R-hat convergence means for redistricting chains (it is convergence of marginal statistics, not of the full plan distribution), and a clarification of the Hamming proxy claim. The final-section language about "real-time audit" should be softened to reflect the estimated status of the throughput.
