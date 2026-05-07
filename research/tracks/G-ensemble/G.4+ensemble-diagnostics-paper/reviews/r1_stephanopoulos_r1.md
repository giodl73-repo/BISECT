# Review: G.4 — Ensemble Diagnostics Paper
**Reviewer**: Nicholas Stephanopoulos (Election law, redistricting doctrine)
**Round**: 1
**Score**: 3/4

## Summary

G.4 provides the technical foundation for certifying redistricting ensembles as reliable evidence in litigation. The three-diagnostic framework (Rhat, ESS, Hamming) and the statutory minimum chain-length recommendations are directly useful for courts evaluating expert testimony. From a legal perspective, this paper's contribution to the G-series is establishing that ensemble evidence, when properly certified, meets an evidentiary standard — analogous to Daubert's requirement that expert methodology be reliable.

## Strengths

The statutory argument in Section 7 is the paper's most legally significant contribution. The four-condition framework ($\hat{R} < 1.1$, $\ess > 500$, $\hat{\rho}_1 < 0.95$, minimum chain length) is concrete enough to be used as an evidentiary standard in court. This is what the ensemble redistricting literature has lacked: a specific, defensible checklist for when an ensemble is reliable enough for use as expert evidence.

The ConvergenceSweep comparison in Section 6 correctly identifies the division of labor: CS for plan generation, certified ReCom for plan evaluation. This framing is legally useful because it addresses the common challenge that ensemble evidence is "generated to produce a desired outcome" — the CS plan is deterministic, the ensemble is the evaluation tool.

The recommendation to use 50,000 steps for California is practically important. Most published redistricting papers use 50,000–100,000 steps; the paper's validation that 50,000 steps is adequate for CA under the stated diagnostics provides retroactive certification for existing literature.

## Weaknesses

**The paper does not propose a specific process for courts to verify ensemble certification.** The four-condition framework is clear, but how would a court verify that an expert's ensemble meets the conditions? The paper should recommend: (a) that the full chain be deposited in a public repository; (b) that the Rhat and ESS be computed independently from the deposited chain, not just reported by the proponent; (c) a format for reporting these diagnostics in expert disclosures. Without a verification protocol, the evidentiary standard is aspirational rather than enforceable.

**The 10,000-step minimum for "states with $k \leq 20$" would not certify the Herschlag 2020 NC ensemble.** Herschlag 2020 used a single chain of 24,518 plans — but G.4 requires multi-chain Rhat ($m \geq 5$ parallel chains). A single chain of 24,518 cannot satisfy Condition 1 ($\hat{R} < 1.1$, which requires $m \geq 2$ chains). The paper should address how to certify single-chain ensembles in the literature: either the ESS and Hamming conditions are sufficient without Rhat, or the Herschlag ensemble is not certifiable under G.4 standards.

**The paper's Rucho discussion (implicit in the statutory section) does not address the Daubert framework.** Federal courts admit expert testimony under Daubert v. Merrell Dow (1993), which requires that methodology be: (1) testable and falsifiable; (2) peer-reviewed and published; (3) subject to known error rates; (4) generally accepted in the relevant scientific community. The G.4 framework satisfies (1) and (2) and arguably (4). But "known error rate" for an ensemble-based percentile claim requires specifying the probability that a non-gerrymandered plan would be flagged as an outlier under the diagnostic thresholds. The paper should address this.

## Minor Issues

- The threshold $\ess > 500$ for PP and $\ess > 300$ for D-seats (Condition 2) are inconsistent in their stringency: PP requires a larger ESS despite the fact that D-seats (integer-valued) has a coarser distribution and may require more precision. The justification for the lower D-seats threshold should be stated.
- The paper recommends "chain length $\geq$ state-specific minimum in Table 3" as Condition 4. But Table 3 gives two columns (Min. steps for PP, Min. steps for D-seats). Which binding condition applies? The paper should specify the maximum of the two (most conservative) as Condition 4.

## Recommendation

Accept with moderate revisions. Add a verification protocol for courts, address single-chain certification (the Herschlag ensemble case), and engage with the Daubert framework.
