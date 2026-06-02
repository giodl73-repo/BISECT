# R1 - BOUNDARY Review

### F-01 - WARN: admissibility terms remain in the court-submission spec
File: `docs/superpowers/specs/2026-04-30-court-submission-reports.md:25`, `docs/superpowers/specs/2026-04-30-court-submission-reports.md:30`
Finding: The remediation correctly stops claiming a court-ready filing package, but the spec still names a "Daubert-readiness self-assessment" and says missing race-of-candidate data makes evidence "not Daubert-defensible."
Consequence: These phrases can still be read as legal-admissibility posture rather than software packaging posture, especially by external legal readers.
Fix: In a follow-up communications pass, route these phrases through DCR-006 explicitly or rephrase them as "legal-review support checklist" / "admissibility-support input gap" with a citation to `docs/legal/COURT_PACKAGING_BOUNDARY.md`.

### F-02 - NOTE: core court-ready claim boundary is now correct
File: `docs/superpowers/plans/2026-04-30-court-submission-reports.md:8`, `docs/superpowers/specs/2026-04-30-court-submission-reports.md:19`
Finding: The previous affirmative court-ready goal language is now bounded as legal-review draft/package-check language and points to the legal boundary and communications strategy.
Consequence: The highest-risk public/legal claim no longer outruns the accepted VTRACE posture.
Fix: Preserve this pattern for future legal-facing docs.

## Summary

No BLOCK findings. One legal-language WARN remains around admissibility terminology.
