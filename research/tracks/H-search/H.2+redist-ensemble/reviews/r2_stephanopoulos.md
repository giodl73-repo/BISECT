# Review — H.2: redist-ensemble
**Reviewer**: Nicholas Stephanopoulos (Election Law, Constitutional Dimensions of Partisan Gerrymandering)
**Round**: 2
**Score**: 4 / 5

---

## Response to Round 1 Concerns

### R09 (MODERATE): Legal framing — AEA replication vs. court admissibility — Partially addressed

My principal concern from Round 1 was that the paper conflates two distinct contributions: (a) the AEA replication improvement (genuine, defensible), and (b) the throughput improvement as a court-admissibility improvement (not well supported). The revised paper makes a meaningful improvement in Section 7.1 (conclusion), where it now correctly frames the TX/CA cold-start handling as a "throughput advantage" rather than a structural correctness fix. The conclusion no longer asserts that pair reselection enables "TX and CA runs that Python GerryChain cannot complete."

However, Section 1.1 and Section 7.2 retain framing that conflates the two contributions. Section 1.1 still states that faster computation enables diagnostics that "would give courts and litigants justified confidence in the ensemble's stationarity." Section 7.2 continues to use "closes the final methodological gap" language that implies the paper completes the court-admissibility case. The distinction I called for — AEA replication benefit (defensible) vs. throughput benefit for legal purposes (not directly relevant) — should be made explicit in both sections.

The most defensible framing would be: "removing the Python GerryChain dependency satisfies the AEA replication standard; the throughput improvement enables more thorough practitioner diagnostics but does not alter the court-admissibility determination, which turns on methodological acceptance rather than run length." This framing appears nowhere in the current revision.

### R15 (MINOR): State-court case law citations — Not addressed

Section 1.1 still lacks explicit citations for League of Women Voters v. Pennsylvania (2018), Harper v. Hall (2022/2023), and Harkenrider v. Hochul (2022). These were requested in Round 1 to support the claim that state courts have accepted ensemble evidence. The paper relies on the G.4 working paper citation for this claim, which is an indirect citation. For a paper with legal framing, direct citations to these cases would strengthen the credibility of the acceptance claim and allow readers to verify the claim independently.

This is a Minor issue (R15) but given the paper's legal framing, it carries more weight than a typical minor citation. Adding three case citations to Section 1.1 takes ten minutes and is strongly recommended before final submission.

---

## What Was Addressed

The most important change from my Round 1 perspective is the Section 7.1 revision. The original paper listed pair reselection as a correctness requirement that enables "TX and CA runs that Python GerryChain cannot complete from cold start" — a claim that was both technically incorrect (GerryChain with pair reselection handles TX) and legally problematic (it could be used to argue the methodology is GerryChain-incompatible). The revised language correctly frames this as a throughput advantage.

The SHA-256 seed specification fix (Liang's R05) is also legally relevant: a deterministic, fully specified seed derivation scheme is harder to challenge on reproducibility grounds than an underspecified one. The paper now has a complete specification that could be audited by an independent expert.

---

## Section 7.2: "Closes the Final Methodological Gap"

This phrase in Section 7.2 continues to concern me. The claim that \re{} "closes the final methodological gap in the G-track research series" implies that the G-track series, once \re{} is completed, will be methodologically complete for court purposes. But the G-track replication comparison (Section 7.3) has not yet been performed. Until \re{} has been empirically compared against GerryChain's ensemble distributions for the six G-track states, the claim that it "replicates GerryChain's stationary distribution" is unverified.

The revision plan correctly identified this in R09: "the legal framing should not be stated in the present tense until replication is confirmed." The revised text has not changed this language. I recommend softening "closes" to "is designed to close" or "will close, pending Phase 2 empirical replication."

---

## Remaining Priority Issues

1. **Section 1.1**: Distinguish AEA replication benefit from legal-confidence benefit. The phrase "courts and litigants" should be replaced with "researchers and practitioners" for the convergence-diagnostics claim.
2. **Section 7.2**: Soften "closes the final methodological gap" to acknowledge that G-track replication is pending.
3. **R15 citations**: Add League of Women Voters, Harper, and Harkenrider citations to Section 1.1.

---

## Recommendation

**Minor revision** (score unchanged from Round 1). The TX framing correction is the most legally significant change and it is done correctly. The remaining items are all editorial: one framing adjustment in Section 1.1, one hedge in Section 7.2, and three case citations. None require new analysis. The paper is close to an acceptable state for publication; these final edits are straightforward.
