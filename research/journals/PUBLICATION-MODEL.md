---
status: seed
updated: 2026-05-09
---

# Publication Model

The Bisect Research Press turns a large technical workbench into public-facing
issues. The core job is curation, not volume. A paper earns a slot only when it
is the best fit for that issue, has a clear claim class, and does not duplicate
an artifact already assigned to an ordinary volume.

## Issue Pipeline

| Stage | Output | Gate |
|-------|--------|------|
| seed | Issue brief | Theme, reader promise, excluded claims, candidate slots |
| audition | Audition pool | 2-3x candidates for every slot; no reuse unless marked reprint |
| provisional | Provisional lineup | Best-fit papers selected; weak claims demoted or archived |
| review | Review packet | Role panel, source chain, legal/statistical/reproducibility checks |
| locked | Public issue | Editorial, articles, notes, review log, assignment ledger |
| archived | Archive note | Correction, retirement reason, or reprint status |

## Promotion Rules

1. Every issue starts from its reader promise, not from the papers available.
2. Every slot gets 2-3x candidate papers before selection.
3. Every selected paper gets a claim class and a review lens.
4. Ordinary issues do not reuse papers already assigned elsewhere.
5. Legal and quantitative claims must be downgraded until source-chain review is complete.
6. Advocacy claims must be labeled as recommendations, not findings.

## Locked Issue Minimum

A locked issue needs:

- `index.md`
- `editorial.md`
- 3-6 selected articles or article summaries
- `AUDITION-POOL.md`
- `PROVISIONAL-LINEUP.md`
- `reviews/source-chain-audit.md`
- `reviews/role-panel-review.md`
- `FEEDBACK-LOG.md`

## Role Gates

Use `.roles/ROLE.md` as the board map. Minimum gates before a legal/public issue
locks:

- BOUNDARY: constitutional and VRA risk
- WARD: state-law and jurisdictional risk
- COVENANT: audit chain and admissibility
- CONTOUR: Census and geography data provenance
- MERIDIAN: algorithm correctness
- BENCHMARK: reproducibility and stale assertions
- SCALE: statistics, power, uncertainty, and multiple testing
- PRECINCT: partisan interpretation
- DATUM: publication quality
- COMMONS: civic readability and community impact
