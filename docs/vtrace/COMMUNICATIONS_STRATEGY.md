# Communications Strategy

## Scope

This strategy controls how maintainers describe the BISECT VTRACE baseline,
research corpus, public artifacts, and release-readiness posture.

Status: `bounded_communications_control`.

This is a messaging and claim-boundary artifact. It does not publish a release,
select a public evidence bundle, close any DCR, certify legal readiness, validate
non-author usability, or upgrade the current
`internal_engineering_baseline_only` posture.

Source controls:

- `MISSION.md`
- `CONOPS.md`
- `REQUIREMENTS.md`
- `READINESS_DECISION.md`
- `BASELINE_HANDOFF.md`
- `RELEASE_GATE_REGISTER.md`
- `ARTIFACT_PUBLICATION_POLICY.md`
- `PAPER_EVIDENCE_INVENTORY.md`

## Communication objectives

| Objective | Controlled by |
|---|---|
| Explain what BISECT does in plain language without turning research results into official conclusions. | REQ-005, REQ-024, REQ-025 |
| Give maintainers a consistent way to point readers from README/docs/papers to VTRACE evidence and non-claims. | REQ-001, REQ-026, REQ-032 |
| Separate public education, research communication, operator handoff, legal framing, and release readiness. | REQ-010, REQ-011, REQ-026, REQ-027 |
| Prevent generated artifacts, dashboards, papers, or local replay outputs from being described as release evidence before the relevant gate passes. | DCR-004, DCR-007, `ARTIFACT_PUBLICATION_POLICY.md` |
| Keep communications reviewable by DATUM, SCALE, COMMONS, VAULT, BOUNDARY, and WARD roles when claims cross their domains. | REQ-035, `RELEASE_GATE_REGISTER.md` |

## Audiences and approved posture

| Audience | Approved message | Required caveat |
|---|---|---|
| Maintainers and future agents | BISECT has an accepted internal VTRACE engineering baseline and may continue through scoped waves/pulses. | This is not public-release readiness; future work must preserve DCR gates and claim boundaries. |
| Researchers | Research claims can be cited only with paper/evidence pointers, review state, data coverage, and uncertainty or gap status. | Paper PDFs, dashboards, and headline metrics are not independently release-final unless the cited evidence gate says so. |
| Special masters, state staff, and operators | The CLI and docs can help generate inspectable algorithmic baselines and reports for supported workflows. | Run completion is not legal, statutory, official, certified, or court-ready status. |
| Civic advocates and public reviewers | Public-facing docs may explain the algorithm, dashboards, and evidence chain in plain language. | Public artifacts must state limitations, non-claims, and privacy/custody status before publication. |
| Election officials and auditors | Package and count workflows can support arithmetic, lineage, reconciliation, and replay inspection where source data supports it. | The software does not certify elections or replace official canvass/certification authority. |
| Legal or court-facing readers | Legal docs may frame candidate uses, boundaries, and required review gates. | No legal/court-ready, filing-ready, approved, statutory, or certified language is allowed without DCR-006 L2 review. |

## Message classes

| Class | Allowed wording | Blocked wording |
|---|---|---|
| Internal baseline | "accepted internal VTRACE engineering baseline"; "controlled continuation"; "DCR-gated release residuals" | "release-ready"; "production certified"; "public launch complete" |
| Research finding | "empirical research claim tied to cited data/review state"; "current paper/dashboard claim"; "requires rerun or review before release-final citation" | "proven final number"; "official result"; "externally certified finding" |
| Algorithm explanation | "deterministic algorithmic baseline"; "reviewable from inputs, configs, manifests, and reports"; "procedural fairness claim" | "legally fair map"; "court-approved map"; "community fairness proven" |
| Public artifact | "inspectable artifact with stated limitations and custody posture"; "candidate evidence bundle" | "accepted public evidence package" unless DCR-004 L2 passes for that bundle |
| Reproducibility | "local smoke"; "candidate replay"; "data-dirty replay"; "clean replay packet ready" | "clean reproducible"; "release-subset reproducible"; "full-scale reproducible" unless DCR-007 L2 passes |
| Usability | "documented workflow"; "operator packet"; "internal dry run" | "externally validated"; "non-author ready"; "public operator validated" unless DCR-003 L2 passes |
| Legal framing | "legal-boundary discussion"; "model statute draft"; "requires jurisdiction-specific review" | "legal-ready"; "court-ready"; "filing-ready"; "approved"; "certified" unless DCR-006 L2 passes |

## Channel rules

| Channel | Allowed content | Review trigger |
|---|---|---|
| `README.md` | Plain-language overview, quickstart, research pointers, VTRACE status pointer, evidence caveats. | COMMONS plus DATUM/SCALE when headline numbers or public claims change. |
| `docs/quickstart/` | Operator workflows, prerequisites, commands, expected outputs, failure modes, and evidence locations. | DCR-003 if wording implies non-author validation or public operator readiness. |
| `docs/PAPERS.md` and `docs/papers/` | Research index, PDFs, paper status, quality reviews, and declared gaps. | DATUM/SCALE/PRECINCT when paper counts, evidence status, or quantitative claims change. |
| `research/journals/` | Public research staging, audition/seed status, editorial queue, and issue framing. | COMMONS plus DATUM/SCALE before claims are treated as locked public issues. |
| `docs/legal/` | Legal models, statutory drafts, and boundary memos. | BOUNDARY/WARD and DCR-006 before filing-ready or jurisdiction-specific language. |
| Generated dashboards/reports/maps | Local or candidate public verification artifacts with limitations and source pointers. | VAULT/DATUM/SCALE/COMMONS and DCR-004 before public evidence-package language. |
| Wave/pulse records | Internal execution control, parent IDs, validation commands, and closure evidence. | BENCHMARK/TRENCH plus affected role lanes before stronger public claims. |

## Required claim packet

Before adding or changing public-facing language about results, readiness,
release posture, legal framing, package compatibility, or reproducibility,
record or cite:

1. Claim text and message class.
2. Audience and channel.
3. Evidence pointer: command, paper, package, manifest, review row, or DCR.
4. Validation level: L0 inspection, L1 internal check, or L2 external/golden/clean evidence.
5. Limitations and non-claims.
6. Review lane and decision.

If any field is unknown, use gap/blocking language rather than success-shaped
language.

## Review routing

| Claim touches | Required review lane |
|---|---|
| Numeric result, dashboard metric, paper statistic, or empirical comparison | DATUM / SCALE |
| Plain-language public explanation or operator guidance | COMMONS |
| Legal, statutory, court, certification, or jurisdiction-specific language | BOUNDARY / WARD |
| Artifact publication, custody, generated outputs, or protected data | VAULT |
| Package schema, compatibility, canonicalization, or external-tool claims | LEDGER / package owners |
| Reproducibility, replay, clean checkout, or deterministic rerun claims | MERIDIAN / COVENANT |
| Wave/pulse completion or process compliance | BENCHMARK / TRENCH |

## Stop rules

Stop and update the relevant DCR, trace, and review records before using any
language that says or implies:

- Public release readiness.
- Legal, court-ready, filing-ready, official, approved, statutory, or certified
  status.
- Non-author usability validation or public operator readiness.
- Clean release-subset or full-scale reproducibility.
- Public evidence-package publication readiness.
- Universal compatibility or interoperability beyond named fixtures.
- Election certification or replacement of official canvass authority.

## Maintenance rule

When a communications change alters evidence posture, update
`STAGE_EXECUTION.md`, `TRACE.md`, `CODE_RIGOR.md`, and `REVIEW.md` in the same
change. If the change only rephrases text while preserving posture, cite this
strategy and the relevant source control in the commit or pulse record.
