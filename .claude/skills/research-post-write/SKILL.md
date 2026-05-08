---
name: research-post-write
description: "Post-writing validation pipeline for bisect research papers. Reads written sections, runs consistency check (values match across sections/tables/abstract), contract check (paper delivers what spec promised), and referee simulation (3 hostile reviewers from redistricting/algorithms/law). Produces a pre-panel-review checklist. Adapted from RMM research-post-write for bisect redistricting research structure."
allowed-tools: [Read, Write, Glob, Grep]
param_set: lean
---

You are running /research-post-write for: {{topic}}

Run the full post-writing validation pipeline for a bisect research paper. Reads the written
LaTeX sections, checks consistency and contract, simulates hostile peer review, and produces
a pre-panel-review checklist.

---

## PHASE 1 -- READ THE PAPER

Find and read all section files:
```
Glob: research/tracks/**/{{topic}}*/sections/*.tex
```
Also read:
- `docs/specs/[matching-spec].md` — the accepted spec (the contract)
- `research/tracks/**/{{topic}}*/references.bib`

Extract:
- **Algorithm**: what algorithm does the paper describe?
- **Key claims**: top 3 quantitative/empirical claims in the paper
- **Series code**: B.XX / G.XX / H.XX
- **Target audience**: algorithms / legal / practitioner

Print:
```
Paper: {{topic}}
Sections found: [list section files]
Spec found: [yes/no + path]
Series: [B.XX / G.XX etc]
Key claims:
  1. [claim + supporting evidence]
  2. [claim + supporting evidence]
  3. [claim + supporting evidence]
```

---

## PHASE 2 -- CONSISTENCY CHECK

Extract every quantitative value mentioned in the paper into a registry.
Check that the same value appears consistently across:
- Abstract
- Introduction
- Results tables
- Conclusion

| Q-ID | Quantity | Abstract | §Intro | Table | §Conclusion | Consistent? |
|------|----------|---------|--------|-------|-------------|-------------|
| Q-01 | NC PP improvement | +11% | +11% | +11.2% | +11% | WARN (rounding) |
| Q-02 | Runtime O(n²) | stated | not stated | — | stated | PASS |

Also check:
- Dagger notation (single-run results) — is it consistently applied?
- Algorithm complexity claims — same formula in §Algorithm and §Comparison?
- CLI flag names — do they match actual implementation flags?

Print:
```
CONSISTENCY: [PASS / N warnings / N failures]
P1 (reject): [list]
P2 (revision): [list]
P3 (minor): [list]
```

---

## PHASE 3 -- CONTRACT CHECK

The spec is the contract. For each major promise in the spec, check whether the paper delivers it.

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| NC Reock > CVD-Geographic | §4 Table 1 | Yes, Table 1 row NC | ✓ |
| Runtime O(n×n_orientations) | §3 Proposition 2 | Yes | ✓ |
| Fair-division legal argument | §5 | Yes, 2 pages | ✓ |
| L0 test for Reock clamping | §3 Remark | Mentioned but not shown | ✗ |

Print:
```
CONTRACT: [PASS / PARTIAL / FAIL]
Promises kept: [N/M]
Gaps: [list items spec promised that paper doesn't deliver]
```

---

## PHASE 4 -- REFEREE SIMULATION

Select 3 referees appropriate for redistricting algorithm papers:

**R1 — Algorithms Reviewer** (SODA/FOCS archetype):
Focus: correctness of complexity claims, completeness of proofs, comparison to prior art.
Hostile to: informal proofs, missing baselines, asymptotic claims without constants.

**R2 — Political Science Reviewer** (APSR/JOP archetype):
Focus: partisan neutrality, empirical validity, over-claiming about legal implications.
Hostile to: single-run empirical results, claims about court applicability, missing partisan analysis.

**R3 — Legal/Practitioner Reviewer** (Public Administration / Law Review archetype):
Focus: operationalizability, legal accuracy, court citation accuracy.
Hostile to: theoretical claims not grounded in actual case law, implementation gaps.

For each referee produce:
```
REFEREE [N] — [archetype]
Recommendation: Accept / Major Revision / Minor Revision / Reject

SUMMARY: [reaction]

MAJOR CONCERNS:
[I-NN] [specific issue]

MINOR CONCERNS:
[list]
```

Issue IDs continuous across all three referees (I-01, I-02, ...).

---

## PHASE 5 -- ABSTRACT CHECK

Read the abstract. Evaluate:
- Does it state the primary quantitative result?
- Is the algorithm described in one sentence?
- Is the legal/practitioner value stated?
- Word count (target: 150-200 words for B-series papers)

Print:
```
ABSTRACT: [word count] words
Primary result stated: [YES/NO]
Algorithm named: [YES/NO]
Value proposition: [YES/NO]
```

---

## PHASE 6 -- PRE-PANEL CHECKLIST

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: {{topic}}
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   [PASS / N issues]
  Contract:      [PASS / N gaps]
  Referee sim:   [likely decision]
  Abstract:      [word count] words

P1 blockers (fix before panel review):
[I-NN] [description] → [fix]

P2 items (should fix):
[I-NN] [description] → [fix]

P3 items (optional):
[list]

PRE-PANEL CHECKLIST:
□ All P1 consistency failures resolved
□ All spec contract promises delivered
□ Single-run results marked with dagger notation
□ Algorithm complexity claim in §Algorithm matches §Comparison
□ CLI flags match actual bisect binary flags
□ Court citations verified (no invented cases)
□ Abstract states primary quantitative result
□ Referee P1 blockers addressed

VERDICT: [READY FOR PANEL / FIXES REQUIRED]
Fixes required: [N]
Next: run panel review using the 5-role panel (Karypis/Rodden/Duchin/Stephanopoulos/Liang)
═══════════════════════════════════════════════════════
```

Write a brief artifact summary to: `research/tracks/[track]/[topic]/POST-WRITE-CHECK.md`
