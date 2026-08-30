# Ferris Web Docs Shadow

## Frame

BISECT already owns a buildable Vue/Vite application under `web/docs`; the
missing behavior is that no current pull-request workflow installs or builds
it. The existing broad Rust, Python, browser, and formal workflows remain
authoritative and unchanged.

This slice declares `web/docs/**` as an owner-native validation domain through
the workflow path filter, records Ferris' current conservative Cargo fallback,
then runs `npm ci` and `npm run build`. Ferris does not infer npm semantics,
execute the build, or become a required check in this shadow.

The thesis is falsified if the hosted lane cannot:

1. retain a deterministic Ferris fallback plan for a representative web input;
2. reject a Ferris result that silently narrows the non-Cargo path;
3. build the application from the committed lockfile; and
4. retain a receipt binding the owner result to the source revision and Ferris
   plan identity.

## Audit

- `web/docs/package.json` declares the owner build entrypoint as
  `npm run build`.
- `web/docs/package-lock.json` permits reproducible `npm ci` installation.
- `.github/workflows/ci.yml`, `test_pipeline.yml`, `shared-kernels.yml`, and
  `verify.yml` do not run `npm ci` or `npm run build`.
- Ferris `validation-plan` classifies `web/docs/package.json` as
  `full_workspace_fallback` because the path is outside all Cargo package
  anchors. That is safe but cannot name the missing owner build.

## Comparison

The existing BISECT workflows keep commands in the owning job rather than in a
central orchestration service. This slice reuses that ownership pattern and
adapts the repository's existing path-trigger mechanism. It avoids encoding
Vue, Vite, npm, or GitHub Actions semantics in Ferris.

The current production comparison is deliberately asymmetric:

- BISECT owns path-to-entrypoint semantics and the build result.
- Ferris owns the deterministic statement that Cargo evidence cannot narrow
  this input.
- GitHub owns scheduling and artifact transport.

## Role review

### BENCHMARK

Accepted. The lane asserts the exact Ferris failure posture before building;
removing either the fallback or the owner build makes the job fail. The build
uses the committed lockfile rather than an unbounded dependency install.

### TRENCH

Accepted with retained shadow status. The structural failure was a web-only
change receiving extensive unrelated evidence but no web build. The path filter
makes the owner check unavoidable for `web/docs/**`, while current required
workflows remain untouched until hosted evidence exists.

### DATUM

Accepted with a bounded claim. One green run proves only that the pinned Ferris
revision reports the expected fallback and that this source revision builds. It
does not prove job-minute savings, safe removal of existing checks, or a general
polyglot-domain API.

## Slice and deletion gate

The evidence slice is `.github/workflows/ferris-web-docs-shadow.yml`.

- Representative input: `web/docs/package.json`.
- Conservative result: Ferris full-workspace fallback is asserted.
- Owner result: `npm ci` plus `npm run build`.
- Proof: retained Ferris plan and `bisect.owner-validation-receipt/v0`.
- Mutation control: a non-fallback Ferris disposition fails before the owner
  build.

No existing workflow is removed. A future Ferris owner-domain contract is
justified only after this lane demonstrates that the owner mapping is stable and
that retained plan/receipt identities are useful in review. Trigger narrowing
or removal of overlapping Rust/Python jobs requires a separate replay and
required-check migration.
