---
status: ACCEPTED
final_avg: 3.5/4
rounds: 2
last_updated: 2026-05-12
---

# T.1 GeoSection — Revision Plan

## Round 1 Summary (avg 3.1/4)

Five reviewers: Karypis (3.5), Rodden (3.0), Duchin (3.0), Polikarpova (3.0), Liang (3.0).

### P1 Items from Round 1

| ID | Reviewer | Issue | Status |
|----|----------|-------|--------|
| P1-I-K | Karypis | Report seed variance for normalised ratio score | RESOLVED (§4.6: CV < 2% for all 5 tested states) |
| P1-I-R | Rodden | Pennsylvania mechanistic explanation | RESOLVED (§5.1: tract placement analysis) |
| P1-II-R | Rodden | Confirmed caterpillar proportionality analysis | RESOLVED (§5.1: Table added) |
| P1-I-D | Duchin | Separate legal/geometric from partisan argument | RESOLVED (§5.1: explicit two-claim separation) |
| P1-II-D | Duchin | NC ReCom ensemble comparison | RESOLVED (§4.7: ensemble comparison added) |
| P1-I-P | Polikarpova | Fix/relabel Lemma 3.1 | RESOLVED (relabelled as Remark/Motivating Heuristic) |
| P1-I-L | Liang | Bound large-state exclusion effect | PARTIALLY RESOLVED (noted in limitations; full CA/TX/FL runs deferred) |
| P1-II-L | Liang | Variance on seat-count comparison | RESOLVED (§4.6: seed variance table added) |

## Round 2 Final Review Panel

Fresh panel: Karypis, Rodden, Duchin, Stephanopoulos, Liang.
Round 2 avg: 3.6/4 — ACCEPTED.

## 2026-05-12 Paper-Quality Check

Status: maintained accepted status after atlas-alignment review.
The pass corrected internal consistency issues found by the P1 claim-discipline
gate: the multi-district state count is 44 rather than 45, the head-to-head
seat-count arithmetic is 37 unchanged plus 7 one-seat changes, and unsupported
city-boundary examples were narrowed to ratio-scan evidence language.
The paper rebuilt with no undefined references/citations and zero BibTeX
warnings, and the PDF was recopied to `docs/papers/`.
