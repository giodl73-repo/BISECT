# Review: G.1 — GerryChain Congressional Comparison
**Reviewer**: Nicholas Stephanopoulos (Election law, redistricting doctrine)
**Round**: 1
**Score**: 3/4

## Summary

G.1 provides the empirical backbone of the G-series legal argument. The central finding — that the AR plan falls near the partisan median of published ensemble distributions in 5/6 states — is exactly the kind of evidence that has been used in redistricting litigation. The legal argument section is well-structured and the "70th percentile threshold" proposal is novel, if somewhat under-theorized. I have concerns about how the paper characterises legal uses of ensemble evidence and about the Georgia result's implications.

## Strengths

The legal argument in Section 8 is structured correctly. The paper's "primary argument + corroboration" structure matches how expert evidence should be organised: the constitutional argument does not depend on the ensemble result, but the ensemble result makes the constitutional argument more defensible. This is sophisticated legal framing.

The verifiability argument is strong: ensemble plans selected from a distribution are vulnerable to a "why this one?" challenge. The AR plan eliminates that challenge entirely by having a unique, derivable answer. This is the paper's most important legal contribution.

The treatment of the Rucho framework (Section 8.3) correctly notes that the AR plan's legal claim does not require Rucho to be overruled — it operates via a different doctrinal path. This is the kind of careful legal reasoning that would survive adverse judicial scrutiny.

## Weaknesses

**The proposed "70th percentile threshold" (Section 8.4) risks being read as a legal standard when it is not.** The paper states that plans with compactness percentile $p \in [60, 85]$ and partisan percentile $q \in [30, 70]$ are "legally defensible as non-gerrymandered." This will be cited by lawyers on both sides. The paper should include a prominent disclaimer that this is a practical heuristic derived from the author's reading of the literature, not a constitutional standard, and that courts have not adopted it. Georgia at $q = 38$ is described as "within the [30, 70] range, though at the lower end" — this is an understatement of the political significance of the result.

**The paper does not engage with the VRA implications of the minority-majority district results.** The AR plan produces 2 MM districts for NC (71st percentile), 1 for WI (58th percentile), 5 for GA (62nd percentile), and 3 for PA (59th percentile). For NC, the 71st percentile means the AR plan concentrates minority populations more than 71% of random valid plans — this is the "packing" concern under VRA Section 2. For litigation purposes, a plan that packs minority voters more than average could be challenged under VRA even if it is not a partisan outlier. The paper needs a section addressing whether the MM percentile results create VRA exposure.

**The paper's characterisation of state vs. federal redistricting law needs updating.** After Allen v. Milligan (2023), Section 2 VRA claims are in active litigation across multiple states. The paper's legal section does not engage with Allen at all, which is a significant gap for a paper explicitly aimed at "statutory implications" and litigation use.

**The "exactly one AR plan per state" claim (repeated from G.0) is potentially falsifiable.** If METIS uses a different random number generator implementation or a different internal tie-breaking rule, a different plan could result. The paper should specify the exact binary (version number, compilation flags) and input data version that produces the specific plan, so the uniqueness claim is precisely scoped.

## Minor Issues

- The claim that the AR plan's "challenge must challenge the algorithm itself, not the selection among plans" is correct but the paper should acknowledge that algorithmic challenges are actually quite common in redistricting litigation — see the North Carolina cases challenging computer-drawn maps on various grounds.
- The summary table uses "\est" for all four direct-comparison states but these four have different levels of evidence quality (Herschlag 2020 is stronger than DeFord 2021 even among the directly-compared states).

## Recommendation

Accept with moderate revisions. Add a VRA section for the MM district results. Clarify the 70th percentile threshold as a heuristic. Engage with Allen v. Milligan.
