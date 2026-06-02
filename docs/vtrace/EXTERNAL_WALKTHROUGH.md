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

## L2 external-operator packet

This packet is the required script for the first real DCR-003 external-user
run. It is intentionally narrow: one non-author must follow one declared path
end to end or to a documented blocker. The reviewer may be a collaborator,
domain reviewer, civic-technology operator, counsel-facing analyst, or state
staff proxy, but they must not be the person who authored the walkthrough or
the code/docs being reviewed.

### Declared path

Before the run starts, fill these fields in the observation record:

| Field | Required value |
|---|---|
| Reviewer role | Special master, state staff, researcher, public reviewer, or other named non-author role. |
| Reviewer independence | State whether the reviewer authored any BISECT implementation or VTRACE docs under review. |
| Repo commit | `git rev-parse HEAD`. |
| Operating environment | OS, shell, Rust toolchain if commands are run, and whether census data is pre-provisioned. |
| Selected quickstart | One document path from `docs/quickstart/`. |
| Selected workflow | Read-only verification, label build/analyze/report/verify, import-label, or documented blocker path. |
| Data/config scope | Label, year, state(s), and whether data are real, fixture, or unavailable. |

### Operator tasks

The observer must not coach the reviewer except to clarify that blockers should
be recorded rather than worked around silently.

1. Start from `README.md` and ask the reviewer to state, in their own words,
   what BISECT claims and does not claim.
2. Ask the reviewer to open the selected quickstart and identify the first
   command they would run.
3. Ask the reviewer to locate the expected output or evidence artifact for that
   command.
4. If the environment has the required data/config, ask the reviewer to run the
   selected workflow. For the canonical label-pipeline path:

   ```bash
   bisect build <label> --year 2020 --workers 8
   bisect label-analyze <label> --year 2020 --types all
   bisect label-report <label> --year 2020 --format html json
   bisect label-verify <label> --year 2020
   ```

5. Ask the reviewer to classify each failure as missing prerequisite, unclear
   instruction, command failure, output-location confusion, legal/claim
   confusion, or accepted limitation.
6. Ask the reviewer to decide whether `label-verify` means legal certification.
   A correct walkthrough must answer no: it is evidence-chain verification, not
   court, statutory, or public-authority approval.
7. Ask the reviewer to locate the next handoff document:
   `docs/vtrace/EVIDENCE_PACKAGE_CONTRACT.md` for public evidence packaging and
   `docs/legal/COURT_PACKAGING_BOUNDARY.md` for legal/court packaging limits.

### Observation record template

Save the completed record outside generated outputs if it will be committed,
or under ignored `reports/vtrace/` if it contains raw console logs. Do not
include private personal data; identify reviewers by role or approved public
name only.

```text
DCR-003 external walkthrough record

Date:
Observer:
Reviewer role:
Reviewer independence:
Repo commit:
Environment:
Selected quickstart:
Selected workflow:
Data/config scope:

Task results:
1. README claim posture understood? pass / partial / fail
   Notes:
2. First command discoverable? pass / partial / fail
   Notes:
3. Expected outputs discoverable? pass / partial / fail
   Notes:
4. Workflow executed or blocked? pass / blocked / not-run
   Commands or blocker:
5. Failure modes understandable? pass / partial / fail
   Notes:
6. Legal/certification boundary understood? pass / partial / fail
   Notes:
7. Evidence/legal handoff docs found? pass / partial / fail
   Notes:

Friction items:
- Item:
  Class: doc fix / command fix / accepted limitation / environment blocker
  Disposition:

Reviewer summary:

Observer disposition:
pass_l2_candidate / blocked / needs_fixes
```

### L2 promotion rule

DCR-003 may move beyond `partial_l1` only after the completed observation
record is reviewed by COMMONS/operator-review and each friction item is either:

- fixed in docs or command behavior,
- recorded as an accepted limitation with a user-visible warning, or
- classified as an environment blocker outside BISECT's control.

The review must preserve the distinction between a successful user walkthrough
and legal/public-release readiness. Passing DCR-003 proves the selected
non-author workflow was understandable within its declared scope; it does not
certify all personas, all data environments, or any court/legal filing.

## Current disposition

DCR-003 is satisfied only at L1 role-simulation level and is now
`ready_for_external_run` because the operator packet and observation template are
defined. L2 closure requires a real non-author reviewer or operator to follow
one path end to end or to a documented blocker, with friction dispositioned by
COMMONS/operator-review.
