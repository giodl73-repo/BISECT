# Public Evidence Package Contract

Contract ID: BISECT-EVIDENCE-PACKAGE-v1

Status: DCR-004 closed at L1 for the internal contract/checklist. L2 public
release use requires VAULT and public-claim review against a concrete bundle.

## Required package layout

```text
evidence-package/
  README.md
  MANIFEST.json
  HASHES.sha256
  config/
  runs/
  analysis/
  reports/
  review/
  limitations/
```

## Required manifest fields

`MANIFEST.json` must include:

- `contract_id`: `BISECT-EVIDENCE-PACKAGE-v1`
- `label`
- `year`
- `scope`: state list, chamber, and any release-subset label
- `created_at`
- `bisect_version`
- `git_commit`
- `working_tree_status`: clean, dirty, or unknown
- `build_features`
- `metis_engine`
- `command_lines`
- `config_path`
- `config_sha256`
- `source_data`: source family, year, custody status, and hash pointers where available
- `artifacts`: paths and SHA-256 values for run, analysis, report, and verification files
- `verification_status`: pass, fail, partial, blocked, or deferred
- `claim_status`: research, internal-review, public-review, release-candidate, or withdrawn
- `limitations`
- `non_claims`
- `supersedes`: prior package IDs or null

## Required artifacts

- Config files used to create the run.
- Run/build manifests and assignment hash pointers.
- Analysis outputs and analysis manifest where generated.
- HTML/JSON reports where generated.
- Label verification output.
- Review notes identifying validation level, role lanes, and open risks.
- Limitation and non-claim statement.

## L1 package review checklist

Before an internal evidence package may claim conformance to this contract, a
reviewer must record:

- The package root contains `README.md`, `MANIFEST.json`, `HASHES.sha256`,
  `config/`, `runs/`, `analysis/`, `reports/`, `review/`, and `limitations/`,
  or records an explicit `blocked`, `partial`, or `not_applicable` reason for
  each missing item.
- `MANIFEST.json` includes every required field listed above.
- Every artifact named in `MANIFEST.json` has a matching SHA-256 entry in
  `HASHES.sha256`, or the manifest records why the artifact is deferred.
- `verification_status` and `claim_status` use the controlled vocabulary in this
  contract.
- `limitations` and `non_claims` are present before any public-facing use.
- The package does not include ignored/local-only source data unless a custody
  review explicitly promotes it.

This checklist is sufficient for DCR-004 L1 internal contract closure. It is not
L2 public bundle evidence until applied to a concrete release package.

## Optional artifacts

- Maps, dashboards, paper-mode reproduction bundle, package-family audits, and
  external interoperability fixtures may be included only with custody and claim
  disposition.

## Compatibility and change control

Breaking changes to required layout, required fields, hash semantics,
verification vocabulary, or claim-status vocabulary require a DCR or explicit
interface-change disposition. Additive optional fields are compatible if they do
not change the meaning of required fields.

## Immutability and supersession

Published packages are immutable by default. If an evidence package is replaced,
the replacement must retain the previous `MANIFEST.json` hash, name the
superseded package, explain the reason for replacement, and preserve a
replacement notice in `review/`.

## Non-claims

An evidence package does not by itself establish legal compliance, court
admissibility, official election certification, statutory enactment, or
jurisdiction-specific plan adoption. Those decisions remain with counsel,
experts, courts, commissions, officials, or other authorized bodies.
