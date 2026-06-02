# Artifact Publication Policy

## Scope

This policy resolves DREQ-001 for the internal VTRACE baseline by stating when
reports, dashboards, paper PDFs, maps, packages, evidence bundles, and related
generated artifacts may be committed or published.

This policy is a control artifact. It does not promote any generated artifact,
select a release bundle, close DCR-004 at L2, or upgrade the current
`internal_engineering_baseline_only` posture.

## Default rule

Generated artifacts are local-only by default. An artifact may be committed or
published only when the table below allows it and the required custody, claim,
and review records exist in the same change or in a cited prior gate.

## Artifact classes

| Class | Examples | Default disposition | Commit / publication rule |
|---|---|---|---|
| Source documentation | `README.md`, `docs/**/*.md`, VTRACE ledgers, quickstarts, legal boundary docs | tracked | May be committed after normal review. Public-facing claims must cite evidence posture and non-claims. |
| Research source | `research/**/*.tex`, bibliography/source material | tracked when already part of the research corpus | May be committed when it is source text or reproducible paper input. Do not treat compiled outputs as independently validated evidence unless reviewed. |
| Committed public paper PDFs | `docs/papers/**/*.pdf` | tracked exception | May be committed only for paper corpus updates with source pointer, claim boundary, and review status. Large unrelated PDFs remain out of scope. |
| Generated maps and figures | `*.png`, `*.jpg`, geospatial exports, presentation figures | ignored/local by default | Commit only when a paper, guide, or evidence package explicitly cites the figure and the change records custody/claim review. |
| Run and analysis outputs | `runs/`, `analysis/`, `reports/`, `outputs/` | ignored/local by default | Do not commit directly. Promote only through an evidence-package or release gate with manifest, hashes, limitations, and review disposition. |
| Dashboards and static sites | generated HTML/JS/CSS dashboard outputs, site bundles | ignored/local unless a named public site artifact is selected | Publish only through a selected bundle with source inputs, generation command, hashes, limitations, and VAULT/public-claim review. |
| Package-family evidence bundles | RPLAN/RCOUNT/RCTX/RHIST audit packages, examples, certificates | tracked only when intentionally fixture/example evidence | Commit fixtures/examples only when they are synthetic or custody-cleared and include verifier path, hash shape, and negative coverage where required. |
| Public evidence packages | `BISECT-EVIDENCE-PACKAGE-v1` bundles | not committed by default | Publish only after DCR-004 L2 review against `EVIDENCE_PACKAGE_CONTRACT.md` and release-gate disposition. |
| Raw or restricted source data | Census downloads, election inputs, local caches, protected or licensed data | ignored/local | Do not commit. Use source pointers, manifests, hashes, or acquisition instructions instead. |
| Local environment files | Cargo patches, temp files, logs, machine-specific manifests | ignored/local | Do not commit unless converted into a documented source artifact with review. |

## Required promotion record

Before any local/generated artifact becomes tracked or public, the promoting
change must record:

1. Artifact path or bundle root.
2. Source inputs and acquisition/custody status.
3. Generation command or reproduction procedure.
4. SHA-256 hash or manifest pointer.
5. Claim status: internal-review, public-review, release-candidate, or
   withdrawn.
6. Limitations and non-claims.
7. Review lane and decision: VAULT for custody, DATUM/SCALE for quantitative
   claims, COMMONS for public/operator wording, and any domain lane required by
   the release gate register.

## Stop rules

Stop and update the relevant DCR, trace, review, and custody records before:

- Publishing generated reports, dashboards, maps, packages, or bundles.
- Adding new binary artifacts outside the already governed docs/paper corpus.
- Treating smoke outputs, candidate replay outputs, or local dashboard outputs as
  public evidence.
- Replacing or superseding a previously published evidence package.
- Including raw/source data in a tracked artifact.

## Relationship to release gates

This policy supplies the DREQ-001 artifact-publication rule. It does not replace
`RELEASE_GATE_REGISTER.md`.

If an artifact is part of a public release, legal/court package, clean replay, or
external-user claim, the relevant DCR and review gate must pass before the
artifact can be described with that stronger claim.
