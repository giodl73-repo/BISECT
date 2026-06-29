# Court and Legal Packaging Boundary

BISECT produces algorithmic evidence and review checklists. It does not create
legal authority, official certification, enacted maps, court-ready filings, or
jurisdiction-specific legal conclusions by running `label-report` or
`label-verify`.

## Package classes

| Class | What BISECT supplies | What remains outside BISECT |
|---|---|---|
| Generated evidence package | Runs, assignments, manifests, hashes, reports, verification output, limitations, and non-claims. | Legal strategy, admissibility, expert opinion, official adoption, and jurisdictional approval. |
| Legal review package | Evidence package plus issue checklist, source/custody notes, and review questions for counsel/experts. | Counsel/expert analysis, state-specific doctrine, court rules, filing format, service, and authentication. |
| Court-ready filing package | Not produced by BISECT alone. | Authorized legal team, court/commission rules, declarations, exhibits, certification, and jurisdiction-specific review. |

## Required legal gates

A legal review package must separately evaluate:

- Federal constitutional population requirements.
- State constitutional and statutory redistricting rules.
- Chamber rules and district count/apportionment authority.
- Voting Rights Act and related racial-vote-dilution analysis.
- Contiguity, compactness, subdivision, community, nesting, and incumbency rules
  where applicable.
- Source-data admissibility, custody, and expert foundation.
- Court/commission filing rules, format, timing, and authority.

## Handoff checklist

Before anyone describes a BISECT output as filing material, the handoff record
must identify:

- Evidence package ID and hash manifest.
- BISECT version, git commit, build features, METIS engine, and command lines.
- Data source and custody status.
- Verification status and any failed, partial, blocked, or deferred checks.
- Known limitations, uncertainty, and non-claims.
- Reviewing counsel/expert/official and jurisdiction-specific authority.

## L1 boundary review checklist

Before an internal legal-review package may pass the DCR-006 L1 boundary gate,
reviewers must confirm:

- The package is named as a generated evidence package or legal review package,
  not as a court-ready filing package.
- Any references to `label-report`, `label-verify`, dashboards, maps, or
  package-family audits describe evidence and checks, not legal certification.
- The jurisdiction, chamber, and governing authority are either named for review
  or explicitly marked as not yet selected.
- Federal, state, chamber, VRA, population, contiguity, subdivision, nesting,
  source-custody, and filing-rule gates are listed as separate human/legal
  review items.
- Counsel, expert, court, commission, or official authority remains responsible
  for legal conclusions and filing decisions.
- Public-facing text avoids "court-ready", "filing-ready", "certified",
  "approved", or equivalent language unless the external legal authority review
  is named and attached.

This checklist closes the internal boundary definition at L1. It does not close
the L2 legal-boundary gate for a specific jurisdiction or filing.

## Public-claim rule

`label-report`, `label-verify`, and evidence-package generation can support a
legal review, but they do not certify legality or filing readiness. Any
court-ready or filing-ready claim requires jurisdiction-specific human/legal
authority review outside the software.
