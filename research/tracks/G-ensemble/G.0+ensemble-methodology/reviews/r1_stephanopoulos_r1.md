# Review: G.0 — Ensemble Methodology
**Reviewer**: Nicholas Stephanopoulos (Election law, redistricting doctrine)
**Round**: 1
**Score**: 3/4

## Summary

G.0 provides the legal and methodological scaffolding for the G-series. The central legal argument — that a deterministic algorithmic plan with verifiable provenance is legally stronger than any ensemble plan selected from a distribution — is novel and potentially important. The paper's framework is coherent and the two-goal distinction maps onto distinct legal uses of redistricting evidence. I have concerns about the legal claims' grounding and about how the paper characterises judicial treatment of ensemble evidence.

## Strengths

The distinction between ensemble evidence as detection versus selection (Section 6.1) is legally precise. Courts have been more comfortable using ensemble evidence to identify outliers than to prescribe specific remedial maps. The paper correctly identifies this asymmetry and uses it to motivate the AR plan's role in the G-track framework.

The three-part argument structure — (1) AR is constitutionally required; (2) AR is not a partisan outlier; (3) AR converges reproducibly — is exactly what a court would need to evaluate. The paper correctly orders these claims with constitutional grounding first and ensemble corroboration second.

The Rucho discussion (Section 6.2) is accurate in its characterisation of the majority opinion. The suggestion that the AR plan operates "at the Article I §2 level rather than the Equal Protection or First Amendment level" is a clever doctrinal move that avoids the federal justiciability problem.

## Weaknesses

**The paper's account of how courts have actually used ensemble evidence is slightly off.** The paper states ensemble evidence "appeared in League of Women Voters of Pennsylvania v. Commonwealth of Pennsylvania (2018)" — this is correct but misleading. The Pennsylvania Supreme Court's decision was under the state constitution, not the federal constitution, and the ensemble evidence was one of several factors, not the primary basis. The paper implies courts have begun "relying on this logic" more broadly than the case law supports. After Rucho, federal courts cannot use ensemble evidence at all for partisan claims. The paper should more carefully distinguish state court uses (where ensemble evidence has been more successful, as in Harper v. Hall in NC and LWV v. PA) from federal court limitations.

**The "one AR plan per state" claim requires more precision.** The paper states "there is exactly one AR plan per state" given census data and seat count. This is true for a fixed METIS seed, but METIS is not guaranteed to find a global minimum — it finds a local minimum. The ConvergenceSweep process searches over seeds to find the best local minimum found, but this does not guarantee uniqueness or global optimality. A challenge to the AR plan could attack this — "if you had used a different seed sequence, you would have gotten a different plan." The paper needs to either (a) demonstrate that the CS plan is globally optimal or (b) acknowledge that it is the best local optimum found by the 600-seed sweep, and argue that this is sufficient for the legal purpose.

**Section 6.3 would benefit from engagement with Vera v. Richards and Shaw v. Reno** on compactness as a legal requirement. The paper invokes compactness as a corroborative factor, but courts have treated compactness as a constitutional minimum (Shaw) rather than as an optimality criterion. The paper could be stronger by acknowledging that the AR plan's compactness is above the constitutional minimum while noting it is not at the extreme.

## Minor Issues

- The statement that "No particular sampled plan has a privileged status" in an ensemble is true for the ensemble itself, but courts have sometimes required specific criteria for selecting from a remedial ensemble (e.g., "the most compact plan from the ensemble"). The paper should acknowledge that courts can impose selection criteria on ensemble plans, which partially addresses the indeterminacy problem.
- The footnote marker framework (\est) is introduced but not explained in G.0 — it should be defined here since G.0 is the framework paper.

## Recommendation

Accept with minor-to-moderate revisions. The legal framing is strong but needs more accurate characterisation of the post-Rucho landscape and more precision about what "exactly one AR plan" means under scrutiny.
