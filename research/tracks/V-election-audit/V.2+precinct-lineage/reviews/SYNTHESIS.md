# V.2 Precinct Lineage: Review Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Scores

| Role | Score |
|---|---:|
| CANVASS | 3 / 4 |
| TALLY | 3 / 4 |
| BENCHMARK | 2 / 4 |
| LEDGER | 2 / 4 |
| VAULT | 3 / 4 |

Average: 2.6 / 4. Minimum: 2 / 4.

## P1 Blocking Items

### P1.1 Add reproducible fixture and harness commands

V.2 should show how to regenerate and verify the `precinct-split-lineage`
fixture, the positive L2 multi-election harness, and the negative L2 cases.

### P1.2 Define cycle ids and unit-universe hashes

LEDGER and TALLY need stable language for cycle id scope, reporting-unit id
scope, and the unit-universe hash used by district aggregation transcripts.

### P1.3 Show the split/merge lineage visually

CANVASS needs a visual ledger showing prior units, current units, authority, and
explanation. This paper's core claim is "do not label-match"; the article
should make the replacement visible.

### P1.4 Clarify privacy/legal boundaries for lineage evidence

VAULT and CANVASS agree that reporting-unit lineage is not voter movement, not
legal approval of a precinct change, and not a license to publish sensitive
support files.

## P2 Important Improvements

- Add a compact bad-lineage expected-failure excerpt.
- Add a "RCOUNT enables / does not prove" table.
- Mark rename and boundary-adjustment lineage kinds as future work.

## Recommended Next Action

Revise V.2 before starting V.3. The paper has the right structure, but it needs
the executable harness evidence and one clearer lineage visual to become a
ready track anchor.
