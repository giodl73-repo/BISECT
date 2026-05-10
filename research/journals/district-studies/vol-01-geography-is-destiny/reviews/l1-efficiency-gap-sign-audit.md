---
journal: District Studies
volume: 1
title: "Geography Is Destiny"
status: audit-note
updated: 2026-05-09
---

# L.1 Efficiency-Gap Sign Audit

## Result

The code-level sign-convention blocker recorded in the May 8 board review is
closed in the repository history. The publication-use blocker is narrower:
any public article that cites L.1 or C.5 numeric outputs must either trace those
outputs to a run after the fix or regenerate the outputs before lock.

## Evidence

Board review record:

- `research/tracks/board-reviews/BOARD-REVIEW-2026-05-08.md` records a P1
  mismatch: L.1 defines positive efficiency gap as Republican advantage, while
  the then-current implementation computed the opposite sign.
- `research/tracks/L-partisan-fairness/PANEL-REVIEW-BATCH1.md` recommends
  changing the implementation to `(wasted_d - wasted_r) / total`, matching the
  paper convention.

Repository history:

- Commit `81a57bbb4419cbccc6e23f534d039bb0a538554e` is titled
  `fix: EG sign inversion + partisan bias field consistency in partisan.rs`.
- The commit changes `compute_efficiency_gap()` from
  `(wasted_r - wasted_d) / total_votes` to
  `(wasted_d - wasted_r) / total_votes`.
- The commit message records that `95/95 tests pass`.

Current implementation:

- `crates/bisect-analysis/src/partisan.rs` defines efficiency gap as
  `(Wasted_D - Wasted_R) / Total_votes`.
- Positive efficiency-gap values are labeled Republican-favoring.
- Negative efficiency-gap values are labeled Democratic-favoring.
- Partisan bias now uses the same `S(0.50) - 0.50` convention as the
  seats-votes curve field.

## Publication Rule

For District Studies Vol. 1:

- L.1 may be used for metric definition and convention explanation.
- C.5 or L.1 numeric results may not be locked until their provenance is tied to
  a post-`81a57bbb` run or regenerated under the current implementation.
- If the measurement slot is included before regeneration, it must be framed as
  a method note rather than a result-bearing article.

## Recommended Line

"The implementation now follows the paper convention: positive efficiency-gap
values indicate a Republican-favoring plan, and negative values indicate a
Democratic-favoring plan. Issue-level numeric claims should be regenerated or
traced to post-fix outputs before publication."
