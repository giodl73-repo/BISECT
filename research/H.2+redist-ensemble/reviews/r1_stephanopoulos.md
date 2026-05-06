# Review — H.2: redist-ensemble
**Reviewer**: Nicholas Stephanopoulos (Election Law, Constitutional Dimensions of Partisan Gerrymandering)
**Round**: 1
**Score**: 4 / 5

---

## Summary

This paper presents a high-performance Rust implementation of the ReCom Markov chain and argues, in part, that the throughput improvement enables better convergence diagnostics and therefore stronger G-track replicability for court-admissibility purposes. The legal framing is generally accurate. My primary concern is with what I will call the "G-track replicability" argument — the paper implies that faster computation improves legal admissibility, but the causal mechanism for this claim is not well specified and may not hold.

---

## The G-Track Replicability Argument

The paper makes the following claim in the conclusion (Section 7.2):

> "redist-ensemble closes the final methodological gap in the G-track research series. [...] redist-ensemble makes those results reproducible from the redist Rust binary alone, satisfying the AEA replication standard for the full portfolio."

And in Section 1.1:

> "The computational cost discourages the thorough convergence diagnostics — multiple parallel chains, extended runs — that would give courts and litigants justified confidence in the ensemble's stationarity."

These claims have two separable components. The first is a software-dependency claim: removing the Python GerryChain dependency satisfies the AEA replication standard. The second is a legal-confidence claim: faster computation enables better convergence diagnostics, which gives courts justified confidence.

The software-dependency claim is accurate and defensible. The AEA's replication standard (Data and Code Availability Policy, 2019) requires that results be reproducible from the provided code without manual intervention or external proprietary dependencies. If the G-track results previously depended on a Python GerryChain installation, and redist-ensemble now provides the equivalent computation from the redist binary, this is a genuine replication improvement. I accept this argument as stated.

The legal-confidence claim is more problematic. The paper argues that because faster computation enables longer runs and more parallel chains, courts will have greater confidence in the ensemble's stationarity. But court admissibility of ensemble evidence does not depend primarily on run length or chain count — it depends on the court's willingness to accept ensemble methodology at all, which is a doctrinal question. Post-Rucho, state courts have accepted ensemble evidence in Pennsylvania (League of Women Voters), North Carolina (Harper I and Harper II), and New York (Harkenrider). None of those decisions turned on whether the experts ran 10,000 vs. 50,000 steps, or on whether they used 4 chains vs. 8 chains. What mattered was the expert's methodological credibility, the statistical gap between the enacted plan and the ensemble distribution, and the political context.

Put directly: making ensemble computations 2,300× faster does not change the legal admissibility argument in any court that has already accepted ensemble evidence. And in courts that have not yet accepted ensemble evidence (which is most courts, because redistricting litigation is rare), the throughput improvement is irrelevant — the debate will be about whether ensemble methodology is valid at all, not about run length.

This is not a fatal criticism of the paper, which is primarily a software and algorithms contribution. But the paper repeatedly invokes the legal context as motivation, and the legal framing should be accurate. I recommend the authors revise the legal framing to distinguish two separate contributions: (a) the AEA replication improvement (software dependency removal), which is genuine and legally relevant; and (b) the throughput improvement for ensemble analysis, which is valuable for practitioners but does not straightforwardly improve legal admissibility.

---

## On the Rucho Framing

The paper's opening framing — that Rucho v. Common Cause (2019) shifted partisan gerrymandering claims from federal to state courts — is accurate. The paper correctly identifies that state-court litigation now drives the demand for ensemble analysis.

The subsequent statement that a plan at the 99th percentile of Republican seat share "is characterized as strong evidence of partisan intent" is defensible as a description of expert practice, but it slightly overstates the legal consensus. Courts have treated ensemble percentile rankings as one piece of evidence among several; no court has adopted a specific percentile threshold as legally probative. Allen v. Milligan (2023), cited in the G.4 working paper context, addresses VRA Section 2 rather than partisan gerrymandering and is not directly relevant to the ensemble-as-evidence framework. The paper does not cite Allen v. Milligan, which is appropriate.

---

## On What the Diagnostics Establish Legally

Section 6 introduces R-hat, ESS, and Hamming autocorrelation as convergence diagnostics. The paper's conclusion states that these diagnostics "enable immediate convergence assessment without a separate analysis pass." This is useful for practitioners. But the paper implies (through the "courts and litigants" framing in Section 1.1) that these diagnostics serve a legal function — giving courts "justified confidence in the ensemble's stationarity."

This overstates what courts are likely to do with MCMC convergence diagnostics. Redistricting litigation does not feature dueling MCMC statisticians debating R-hat values in court. Expert reports cite ensemble percentile rankings; opposing experts challenge the methodology or the starting conditions; courts make judgment calls. Providing R-hat values in an expert report is a methodological hygiene improvement but not a legally decisive contribution.

I am not criticizing the diagnostics themselves — R-hat and ESS are appropriate and their implementation is well-described. I am criticizing the repeated legal framing that treats convergence diagnostics as if they will be scrutinized by courts. The more accurate framing is that diagnostics are necessary for the paper's own academic credibility and for the AEA replication standard, not primarily for court audiences.

---

## On the Software Independence Contribution

The Section 7.2 claim that "G-track results previously depended on a Python GerryChain installation; redist-ensemble makes those results reproducible from the redist Rust binary alone" is the most legally and scientifically meaningful claim in the paper. A system that runs ensemble analysis, diagnostics, and report generation from a single binary, from a deterministic seed, with a SHA-256 hash chain, is genuinely harder to challenge on reproducibility grounds than a Python-dependent workflow. This argument should be elevated in the introduction.

The SHA-256 seed derivation scheme (Section 3.4) — $\text{seed}_i = \text{SHA256}(\text{"ENSEMBLE\_CHAIN\_"} \| i \| \text{"\_"} \| \text{base\_seed})$ — provides reproducibility across chain counts (adding an additional chain does not change the seeds for existing chains). This is a defensible design. The paper should note that this property — specifically, that the seed for chain $i$ does not depend on the total number of chains — is what enables adding chains to a run without invalidating previous ones.

---

## Minor Points

- The paper cites Rucho (2019) and implicitly the state-court cases through the G.4 working paper but does not provide explicit citations for Pennsylvania (LWV v. Pennsylvania), North Carolina (Harper v. Hall), or New York (Harkenrider v. Hochul). For a paper with legal framing, these citations would strengthen the credibility of the state-court acceptance claim.
- Section 7.3 describes "empirical G-track replication" as future work: running the six G-track states using redist-ensemble and comparing against G.1 results. This is precisely what is needed to establish that the Rust implementation produces "statistically indistinguishable output" from GerryChain. Until this comparison is done, the claim that redist-ensemble replicates GerryChain's stationary distribution is unverified. The paper appropriately defers this to Phase 2, but the legal framing should not be stated in the present tense until replication is confirmed.

---

## Recommendation

Minor revision. The paper is well-executed as a software and algorithms contribution. The legal framing should be tightened to distinguish the AEA replication improvement (genuine and legally relevant) from the throughput improvement (valuable for practitioners but not directly relevant to court admissibility). The G-track replication comparison needs to be completed before the stationary-distribution equivalence claim can be asserted as established.
