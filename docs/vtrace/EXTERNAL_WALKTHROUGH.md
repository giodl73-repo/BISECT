# External User Walkthrough

## Scope

This is the DCR-003 walkthrough record for non-author workflows. It separates
internal role simulation from true external-user evidence.

## L1 role-simulation path

Persona: special master or state staff user who receives a label and needs to
verify evidence before review.

Steps:

1. Read `README.md` for project purpose and claim posture.
2. Follow `docs/quickstart/quickstart-special-master.md` or
   `docs/quickstart/quickstart-state-staff.md`.
3. Use the current label-pipeline command family:

```bash
bisect build <label> --year 2020 --workers 8
bisect label-analyze <label> --year 2020 --types all
bisect label-report <label> --year 2020 --format html json
bisect label-verify <label> --year 2020
```

Expected understanding:

- `label-verify` checks the evidence chain; it is not legal certification.
- Generated reports are evidence packages, not court-ready filings.
- Missing data/config/artifacts are blockers, not soft passes.
- Release or public claims must cite DCR-backed fixtures, smoke evidence, and
  the public evidence package contract.

## Findings

| Finding | Disposition |
|---|---|
| Quickstarts now use lowercase label-pipeline commands and point to evidence-package caveats. | L1 pass by inspection. |
| No real non-author operator has completed the walkthrough in this session. | L2 remains open. |
| Legal/certification misunderstanding risk is addressed by `docs/vtrace/EVIDENCE_PACKAGE_CONTRACT.md` and `docs/legal/COURT_PACKAGING_BOUNDARY.md`. | L1 pass with L2 public review pending. |

## Current disposition

DCR-003 is satisfied only at L1 role-simulation level. L2 closure requires a
named real non-author reviewer or operator to follow one path end to end or to a
documented blocker.
