# Review — C.1: Spatial Resolution and Algorithmic Redistricting (MAUP)
**Reviewer**: Richard Pildes (Election law, constitutional law, VRA)
**Round**: 1
**Score**: 2/4
**Verdict**: Major Revision

## Summary

C.1's core empirical finding — that resolution choice within the county-to-block-group range has limited impact on aggregate metrics — is useful for practitioners who need to justify their choice of spatial unit. However, the paper does not engage with the legal question of whether resolution choice is a legally significant decision in redistricting proceedings. Courts may care about boundary placement precision (relevant for VRA minority VAP calculations) even when seat counts are stable. The paper's framing as a pure methods paper misses the opportunity to provide legal guidance.

## Strengths
- The empirical finding is credible and addresses a real practitioner concern.
- The 50-state extension provides national breadth.
- The RSI metric is simple and interpretable.

## Concerns
- **VRA boundary precision omitted**: The paper tests PP and seat counts but not minority VAP across resolutions. For VRA analysis, whether a near-threshold district's minority VAP is above or below 50% depends on which tracts/block-groups are included at district boundaries. A stability finding for aggregate seat counts does not imply stability for minority VAP in near-threshold districts. The paper should test minority VAP stability across resolutions for the five focus states.
- **Legal implications not addressed**: The paper treats resolution choice as a purely technical decision. But in litigation, a hostile expert could argue that the plaintiff's experts used one resolution and the defendant's used another to manipulate outcomes. The paper should recommend that resolution be specified and fixed before analysis begins.
- **Single run limitation**: See Karypis's review. All results are seed 42 only.

## Required Changes (P1/P2)
- **P1**: Add a minority VAP stability analysis for the five focus states across resolutions. Test whether near-threshold minority districts (30-55% minority VAP) change their VRA classification across resolutions.
- **P2**: Add a legal guidance section: resolution choice should be specified in the redistricting statute and fixed before analysis begins; practitioners should not choose resolution after seeing partisan outcomes.
