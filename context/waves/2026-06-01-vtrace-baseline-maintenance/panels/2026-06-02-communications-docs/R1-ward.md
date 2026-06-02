# R1 - WARD Review

### F-01 - WARN: jurisdiction-specific legal formatting remains out of scope but should stay visible
File: `docs/superpowers/specs/2026-04-30-court-submission-reports.md:56`, `docs/superpowers/specs/2026-04-30-court-submission-reports.md:57`
Finding: The spec says jurisdiction-specific court formatting is out of scope, which is correct. The new communications audit does not call out this residual jurisdiction-specific risk directly.
Consequence: Future implementers could over-generalize a generic legal-review PDF template into a jurisdiction-ready deliverable.
Fix: In a future spec cleanup, add a short cross-reference from the out-of-scope item to `docs/vtrace/COMMUNICATIONS_STRATEGY.md` and DCR-006.

### F-02 - NOTE: strategy separates state/legal audiences from operator audiences
File: `docs/vtrace/COMMUNICATIONS_STRATEGY.md:42`, `docs/vtrace/COMMUNICATIONS_STRATEGY.md:45`
Finding: The strategy correctly distinguishes operator use from legal/court-facing readers and requires DCR-006 L2 for court-ready, filing-ready, approved, statutory, or certified language.
Consequence: State-specific law and federal/legal-readiness gates remain separate.
Fix: Preserve this audience split.

## Summary

No BLOCK findings. One WARD carry-forward should keep jurisdiction-specific residual risk visible in future PDF-report work.
