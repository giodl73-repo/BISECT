---
journal: District Studies
volume: 1
title: "The Ensemble Median Case"
status: draft-note
source: G.1 GerryChain Congressional Comparison
updated: 2026-05-09
claim-class: empirical
review-gate: SCALE
---

# The Ensemble Median Case

## Draft Claim

G.1 can support a cautious version of the North Carolina claim:

> In the G.1 ensemble run, the Bisect plan lands near the middle of the
> compactness distribution for North Carolina. That result is informative
> because the algorithm is not tuned to match the ensemble; both methods are
> responding to the same geographic constraints. The estimate remains
> preliminary until multi-chain validation confirms the ensemble position.

This is safe for audition. It is not yet locked public copy.

## What To Say

- "In this ensemble run" rather than "the ensemble proves."
- "Near the middle of the compactness distribution" rather than "certified at
  the 50th percentile."
- "Consistent with the geography-constrained thesis" rather than "proves
  geography determines the result."
- "Preliminary point estimate" where a percentile is mentioned.
- "Effective sample size and multi-chain validation remain review gates."

## What Not To Say

- Do not present the 1,000-step chain as 1,000 independent draws.
- Do not cite raw percentile precision as if it were a court-ready p-value.
- Do not describe the result as a legal certificate.
- Do not infer partisan intent from compactness placement.
- Do not use G.1 to claim that all states behave like North Carolina.

## Source-Chain Notes

G.1 currently states:

- ReCom ensembles use 1,000 steps.
- The ensemble provides a preliminary distributional portrait.
- Multi-seed Phase 2 validation is required to confirm mixing.
- For G.1 1,000-step runs, sampling uncertainty is substantial.
- Using lag-1 autocorrelation estimates, NC-scale effective sample size is
  approximately 70 for compactness placement.
- All G.1 percentile figures are preliminary point estimates.

The draft article should quote or paraphrase those caveats before any
percentile number appears.

## Article Shape

1. Start with the reader problem: a compactness algorithm needs an external
   comparison class.
2. Explain what a ReCom ensemble is in one paragraph.
3. State the North Carolina result as a preliminary ensemble placement.
4. Explain why a middle-of-the-distribution result can still support the issue
   thesis: geography can make different valid methods converge.
5. Close with the review gate: stronger claims require multi-chain validation,
   ESS-corrected uncertainty, and source-chain review.

## Review Questions

- SCALE: Is the uncertainty phrasing strong enough for an ESS near 70?
- DATUM: Can every numeric value be traced to the current G.1 source or data?
- BOUNDARY: Does the copy avoid turning a statistical comparison into a legal
  conclusion?
- COMMONS: Does a nontechnical reader understand why "middle" is not a failure
  of the algorithm?
