---
journal: District Studies
volume: 1
title: "Geography Is Destiny"
status: provisional-public-preview
updated: 2026-05-09
---

# Tasks

## To Reach Provisional

| ID | Task | Gate | Status |
|----|------|------|--------|
| DS1-001 | Add B.12 METIS `niter`, `ncuts`, and `numbering` to source chain | MERIDIAN | implementation defaults found; paper-run vector still open |
| DS1-002 | Determine whether B.12 Table 1 values are deterministic or averaged across seeds | SCALE | open |
| DS1-003 | Identify C(G) estimator as Lorenz-analytical or METIS-empirical | MERIDIAN/SCALE | open |
| DS1-004 | Write ESS-safe G.1 summary language for NC and compactness percentiles | SCALE | reviewed for audition; exact percentile still blocked |
| DS1-005 | Close L.1 efficiency-gap sign-convention discrepancy in review record | SCALE/BENCHMARK | done-code; output provenance open |
| DS1-006 | Decide whether measurement slot runs in Vol. 1 or is deferred to Vol. 2 | DATUM | done; result-bearing slot deferred |
| DS1-007 | Choose state case study: B.12 GA/WI/NV vs. B.11/G.1 Texas | PRECINCT/DATUM | done; B.12 three-state frame selected |
| DS1-008 | Draft editorial after DS1-001 through DS1-007 | COMMONS | draft-note complete; revise after lineup |
| DS1-009 | Draft issue guardrail note | BOUNDARY/SCALE/DATUM | draft-note complete |
| DS1-010 | Create source-request list for promotion gates | DATUM | complete |
| DS1-011 | Run role-panel review on draft package | BOUNDARY/COVENANT/MERIDIAN/SCALE/DATUM | complete; provisional blocked |
| DS1-012 | Convert draft notes into article-summary drafts | DATUM/COMMONS | complete; not locked |
| DS1-013 | Run board review on article-summary package | BOUNDARY/SCALE/DATUM | complete; recommend scope-down path |
| DS1-014 | Decide source-chain path vs. scope-down path | BOUNDARY/DATUM | complete; scope-down path selected |
| DS1-015 | Expand summaries with examples and source notes | COMMONS/DATUM | complete |
| DS1-016 | Re-run review on expanded scope-down summaries | BOUNDARY/SCALE/DATUM | complete; audition-ready summaries |
| DS1-017 | Build one-file audition packet for human review | DATUM/COMMONS | complete |
| DS1-018 | Human editor approval of scope-down strategy | BOUNDARY/DATUM/COMMONS | complete; provisional preview approved |
| DS1-019 | Expand article summaries into provisional public drafts | COMMONS/DATUM | complete |
| DS1-020 | Run BOUNDARY/SCALE/DATUM review on provisional drafts | BOUNDARY/SCALE/DATUM | complete; copyedit next |
| DS1-021 | Copyedit provisional drafts for public voice and consistency | COMMONS/BOUNDARY | complete |
| DS1-022 | Convert source notes into consistent reference form | DATUM | complete; source-basis sections normalized |
| DS1-023 | Run final risky-claim scan on provisional issue | BOUNDARY/SCALE | complete; accepted hits are boundary statements |
| DS1-024 | Final lock review on provisional packet | BOUNDARY/SCALE/DATUM/COMMONS | complete; hold at provisional public preview |
| DS1-025 | Convert source-basis sections into final reference format | DATUM | complete; provisional references added |
| DS1-026 | Final copyedit of preview packet and articles | COMMONS | complete |
| DS1-027 | Final citation-format pass before lock | DATUM | complete for preview; final publication citation pass remains before lock |
| DS1-028 | Re-run risky-claim scan after copyedit | BOUNDARY/SCALE | complete; accepted hits are boundaries |
| DS1-029 | Decide whether to lock preview or keep it provisional | BOUNDARY/DATUM/COMMONS | complete; keep provisional public preview |
| DS1-030 | Final publication packaging if lock is later requested | DATUM/COMMONS | open |

## Current Recommendation

Proceed with four core pieces first:

1. Editorial: The Shape of the Vote.
2. B.12 summary with explicit reproducibility caveat.
3. G.1 NC/ensemble summary using ESS-safe language.
4. Correction note explaining what the issue does not show.

The result-bearing measurement slot is deferred by
`decisions/measurement-slot.md` until L.1/C.5 numeric outputs are regenerated
or traced to a post-`81a57bbb` run. The code-level sign issue itself is closed.

The G.1 summary can now move to review from
`drafts/g1-ensemble-median-case.md`.

The article summaries passed audition review in
`reviews/article-summary-review.md`. The scope-down path is now recorded in
`decisions/scope-down-path.md`: keep Vol. 1 diagnostic, avoid exact
quantitative claims, and use later issues for source-chain-heavy measurement
claims.

B.12 remains the lead formal source, but
`reviews/b12-reproducibility-and-scope-audit.md` keeps DS1-001 through DS1-003
open until the missing parameter, variance, and C(G) provenance fields are
closed.

Draft notes and article-summary drafts now exist for the working lineup:

- `drafts/editorial-the-shape-of-the-vote.md`
- `drafts/b12-geography-constrains-proportionality.md`
- `drafts/g1-ensemble-median-case.md`
- `drafts/three-state-types.md`
- `drafts/what-this-issue-does-not-show.md`
- `articles/the-shape-of-the-vote.md`
- `articles/geography-constrains-proportionality.md`
- `articles/the-ensemble-median-case.md`
- `articles/three-state-types.md`
- `articles/what-this-issue-does-not-show.md`
