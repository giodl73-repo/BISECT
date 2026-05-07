> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: VRASection: Geographic Alignment Score for Minority Opportunity District Bisection
# Round 2

**Reviewer**: Moon Duchin (Rutgers University, MGGG Redistricting Lab)
**Expertise**: Redistricting mathematics, VRA compliance, Shaw v. Reno analysis, GerryChain, geometric probability
**Round**: 2
**Date**: 2026-05-02

## Summary of Revisions

The authors have substantially revised the two issues I flagged as highest priority.
Section 5.2 (now titled "The Shaw v. Reno Line and the Cooper v. Harris Counterfactual")
replaces the original $w_\text{vra} < 0.5$ predominance argument with a proper
counterfactual framework grounded in *Cooper v. Harris* (2017).
The Alabama Black Belt paragraph now reports the 2:5 sub-region as approximately 52%
Black VAP, making it a majority-minority subgraph before further recursion — directly
addressing the *Allen v. Milligan* 2-district requirement.
A new remark acknowledges the decoupled optimization bias (Karypis Round 1, Issue 1)
and explains why it is conservative.

This is a meaningful revision. The legal analysis is now substantively correct where it
previously was not.

## Score: 3.5/4

**My score**: 3.5/4 — The Shaw counterfactual replacement is the right move, the 3/6
state ratio-change finding is well-framed, and the Black Belt majority-minority subgraph
result is a genuine advance. Residual issues in empirical completeness and the CVAP
question prevent a 4/4.

## What Changed and Whether It Works

### P1-I (Shaw/Miller Counterfactual): Addressed — Convincingly

The rewritten Section 5.2 correctly identifies the *Cooper v. Harris* framework: the
operative question is whether specific boundary choices are explicable on non-racial
grounds, not whether an objective function weight exceeds 50%. Proposition 1
(VRASection Counterfactual Availability) is a genuine contribution: in 3 of 6 states
VRASection and GeoSection agree exactly, and in the 3 where they diverge the deviation
is geographically explicable. The three geographic explanations — Black Belt northern
boundary (AL), Charlotte–Raleigh corridor (NC), Sea Island coast (SC) — are the right
level of analysis for expert testimony.

The expert-testimony formulation at the end of §5.2 is particularly strong: it frames
the boundary choice as "nearly as compact as the GeoSection winner (4.3% EC premium)
with a geographically real tie-breaking signal." This is exactly the argument that
would survive a *Hunt v. Cromartie* challenge — the racial explanation is available,
but so is an equally complete non-racial (geographic) explanation.

My only remaining concern here is that Proposition 1's "at most 50% of tested states"
framing could be read as a purely descriptive result. The legal force of the argument
depends on whether the 3 states where VRASection agrees with GeoSection are the states
where minority population is *least* geographically concentrated. If MS, GA, and LA
happen to have dispersed minority populations (which they do for different reasons),
then "no change in 3 states" is explained by graceful degradation, not just
coincidence. The paper's alignment score data for the no-change states (where
*A* is presumably low at the winning ratio) would strengthen this.

### P1-II (Alabama MM Districts): Partially Addressed — Not Complete

The claim that the 2:5 sub-region is ~52% Black VAP is a genuine improvement over the
Round 1 projection of "35–40%." The footnote acknowledges that full MM district
counting requires the complete pipeline run, which is honest and appropriate. The
argument that a 52% majority-minority subgraph "will produce at least one
majority-Black congressional district" in the recursive GeoSection is plausible — but
it is a plausibility argument, not a verified result.

*Allen v. Milligan* required Alabama to draw **two** Black-opportunity districts.
The paper's 52% figure for the 2-district southern sub-region supports one MM district
there. Whether the 5-district northern sub-region (Jefferson County + Piedmont) also
produces one MM district is not demonstrated. The claim that VRASection "responds
directly to *Allen v. Milligan*'s requirement" in the conclusion still outruns the
empirical verification.

For a revision at this stage, the 52% subgraph result is acceptable evidence of
progress. The "pending full run" caveat is honest. I am raising my score because the
argument is now directionally correct, but a 4/4 requires the full end-to-end count.

### Decoupled Optimization (Karypis Issue, acknowledged here): Well-Handled

The new remark acknowledging the bias — that alignment is evaluated on the
compactness-optimal seed, not the alignment-optimal seed — and explaining that this
makes the algorithm *conservative* (less likely to select minority-aligned ratios than
a joint-optimal version would) is exactly the right framing. It converts a potential
weakness into a legal design feature: the algorithm undersells its own minority
alignment, which is the legally safe direction.

## Remaining Issues

### Issue A: Alignment Scores for No-Change States Not Reported
**Severity**: Medium
The paper argues that MS, GA, LA show no ratio change because alignment does not alter
the outcome. The counterfactual argument in §5.2 is strengthened if the paper reports
the alignment score *A* at the winning ratio for these three states, showing *A* is
low (dispersed minority population → graceful degradation). Currently, *A* = 0.42 for
Alabama is the only alignment score in the comparison table. Table 2 should have an *A*
column for all six states.

### Issue B: CVAP vs. VAP Still Not Addressed
**Severity**: Low-medium
The CVAP limitation I raised in Round 1 is not addressed in this revision. For Alabama
and Mississippi this is a minor issue, but the paper now discusses NC (with a larger
non-citizen urban population in Charlotte) and SC (coastal). A one-sentence
acknowledgment in the limitations section would be sufficient.

### Issue C: The Callais Section Remains Thin
**Severity**: Low
Section 5.5 is unchanged from Round 1. The integration between VRASection's
first-level alignment and the Callais evidence standard remains at three paragraphs.
I accept the authors' apparent decision to defer the full integration to the companion
Callais paper, but the current section should make this deferral explicit rather than
leaving it as an apparent incompleteness.

## Minor Notes

- The MVAP_frac clarification in §3.1 (concentration vs. minority percentage) is well
  done. The distinction between "52% of the state's minority population is on this
  side" and "this side is 52% minority" is now clear.
- The "50% borderline" row in Table 3 remains problematic. The paper has correctly
  explained that *w* < 0.5 is not a legal bright line, but Table 3 still lists
  $w_\text{vra} = 0.50$ as "Borderline." This is internally inconsistent with the
  revised §5.2. Replace "Borderline" with "Within proposed safety margin (narrow)" or
  similar.
- The NC 1:13 result and its caterpillar implications (my Round 1 minor note, Rodden's
  major issue) are not resolved. §5.2's geographic explanation (Charlotte–Raleigh urban
  corridor) is plausible but does not engage with the electoral effectiveness question.

## Recommendation

Accept with minor revisions. The central legal analysis revision is substantive and
correct. Add alignment scores for the no-change states to complete the counterfactual
argument, fix the "Borderline" cell in Table 3, and add the CVAP footnote. The Alabama
52% subgraph result is a good faith response to the MM district demand; the full
end-to-end count remains as a known gap for the final version.
