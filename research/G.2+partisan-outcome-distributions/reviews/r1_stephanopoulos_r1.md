# Review: G.2 — Partisan Outcome Distributions
**Reviewer**: Nicholas Stephanopoulos (Election law, redistricting doctrine)
**Round**: 1
**Score**: 3/4

## Summary

G.2 is the paper most directly relevant to redistricting litigation, since partisan bias claims are the dominant legal theory in modern gerrymandering cases. The proportionality corridor framework is a useful contribution that addresses one of the central debates in the post-Rucho world. The paper's finding that AR falls inside the corridor for competitive states is significant. My concerns are primarily about legal framing and the paper's engagement with existing fairness metrics.

## Strengths

The proportionality corridor (within one seat of strict proportionality) is a defensible operationalization of a fair-outcome benchmark. It is more defensible than raw proportionality (which courts have uniformly rejected as a constitutional requirement) and less arbitrary than, say, the partisan symmetry metric.

The finding that AR falls inside the corridor for NC, WI, and PA — three of the most litigated states in recent redistricting history — is politically significant. For Pennsylvania, where LWV v. PA established a state constitutional right to non-gerrymandered maps, a demonstration that AR falls inside the proportionality corridor would be directly relevant.

The paper correctly distinguishes proportionality from corridor inclusion: "falling inside the proportionality corridor does not mean the plan is proportional by design." This is the right disclaimer for legal purposes.

## Weaknesses

**The paper does not engage with the existing partisan fairness metrics used in redistricting litigation.** The efficiency gap (Stephanopoulos and McGhee 2015), the mean-median gap, and the partisan bias metric are all standard measures used by expert witnesses. The paper should compute these for the AR plans and compare them to the corridor result. A plan that falls inside the proportionality corridor might still have a large efficiency gap if the winning margins are lopsided. Conversely, a plan outside the corridor in a sorted state might have a small efficiency gap. The corridor analysis and the fairness metric literature need to be connected.

**The Georgia result (5D vs. corridor of 6–7D) is the critical case and the paper's treatment is inadequate for litigation purposes.** The paper explains the Georgia deviation through geographic sorting but does not address whether there exists any compact, contiguous plan that achieves 6D Democratic seats. If compact plans with 6D Democratic seats exist in the ensemble (as implied by the 28% corridor fraction), then the AR plan's failure to achieve this is a specific design choice, not a geographic inevitability. A potential challenger would make exactly this argument: "Your algorithm produces 5D; 28% of random plans produce 6D or 7D; your algorithm's design choice costs Democrats one seat." The paper should address this directly.

**The proportionality corridor fractions (28%–61% across states) are not placed in comparative context.** Are these corridor fractions higher or lower than what enacted plans typically achieve? If most enacted plans fall outside the corridor in sorted states (which is likely, since enacted plans are often politically motivated), then AR's performance outside the corridor for Georgia is not unusual — but the paper does not establish this benchmark.

**The abstract states AR is "not systematically biased" based on 5/6 states.** But whether a result is "systematic" depends on whether the deviation is correlated with direction: in all studied states, the deviation when it occurs is in the Republican direction. One state in six at the 38th percentile of Democratic seats might be within statistical noise, or it might be the beginning of a systematic pattern for sorted states. The paper does not have enough sorted states in the sample to distinguish these possibilities.

## Minor Issues

- The paper should note that in Minnesota (Section 4.3), the AR plan produces 4D/4R in an $8$-seat state with $v_D = 0.531$, which means Democrats get 50% of seats with 53.1% of the vote — a small Republican over-representation. This is inside the corridor (since proportional would be 4.25 seats) but still below proportionality.
- The "algorithm choice" section (Section 5) should note that the B.0 bakeoff is unpublished and readers cannot verify the claims made about it.

## Recommendation

Accept with moderate revisions. Add engagement with standard partisan fairness metrics, and address the question of whether ensemble plans that achieve 6D for Georgia are compact.
