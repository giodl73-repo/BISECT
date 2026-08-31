# Ferris Web Docs Shadow

## Frame

BISECT already owns a buildable Vue/Vite application under `web/docs`; the
missing behavior is that no current pull-request workflow installs or builds
it. The existing broad Rust, Python, browser, and formal workflows remain
authoritative and unchanged.

This slice declares `web/docs/**` in the checked-in
`ferris.owner-validation-domains/v1` contract, passes the pull-request base,
head, and tested revisions to Ferris, then runs `npm ci` and `npm run build`
only when Ferris
selects the opaque `web-docs-build` entrypoint. Ferris does not infer npm
semantics, execute the build, or become a required check in this shadow.

The thesis is falsified if the hosted lane cannot:

1. derive a deterministic Ferris plan from the actual base/head path set;
2. select the declared owner entrypoint for web-only and deleted-web inputs
   without crossing a path-segment prefix boundary;
3. compose Cargo and web ownership while retaining fallback for undeclared
   paths;
4. reject overlapping owner-domain declarations;
5. build the application from the committed lockfile; and
6. retain a receipt binding owner result, contract, revisions, and plan identity.

## Audit

- `web/docs/package.json` declares the owner build entrypoint as
  `npm run build`.
- `web/docs/package-lock.json` permits reproducible `npm ci` installation.
- `.github/workflows/ci.yml`, `test_pipeline.yml`, `shared-kernels.yml`, and
  `verify.yml` do not run `npm ci` or `npm run build`.
- Ferris commit `1f2a9d1d7047d676f5fe36832e646535d7d43c60` accepts the
  checked-in path-prefix contract, derives committed changes from the exact
  revisions, and binds the resulting plan without learning the command behind
  `web-docs-build`.

## Comparison

The existing BISECT workflows keep commands in the owning job rather than in a
central orchestration service. This slice reuses that ownership pattern and
adapts the repository's existing path-trigger mechanism. It avoids encoding
Vue, Vite, npm, or GitHub Actions semantics in Ferris.

The current production comparison is deliberately asymmetric:

- BISECT owns path-to-entrypoint semantics and the build result.
- Ferris owns deterministic selection of opaque owner IDs, Cargo package
  evidence, and visible fallback.
- GitHub owns scheduling and artifact transport.

## Role review

### BENCHMARK

Accepted. The lane checks positive web-only, mixed Cargo/web, deleted-web,
unknown fallback, and invalid-overlap controls. Removing the contract mapping,
fallback, or owner build makes the job fail. The build uses the committed
lockfile rather than an unbounded dependency install.

### TRENCH

Accepted with retained shadow status. The structural failure was a web-only
change receiving extensive unrelated evidence but no web build. Actual changed
paths now drive selection, and the declared owner check remains unavoidable for
`web/docs/**`; current required workflows remain untouched.

### DATUM

Accepted with a bounded claim. One green run proves only that the pinned Ferris
revision selects the declared opaque entrypoint for this path set, preserves
the tested controls, and that this source revision builds. It does not prove
job-minute savings, safe removal of existing checks, or a general polyglot
execution API.

### Revision-binding migration closure

- **BENCHMARK:** accepted. The actual plan requires a native revision binding,
  non-empty change set, independent owner-gate agreement, and the retained
  positive, negative, and mutation controls.
- **TRENCH:** accepted for hosted proof. PP-20 remains mitigated until the
  hosted Linux lane retains the native binding and completes the owner build.
- **DATUM:** accepted with the same bounded claim. No savings, CI-equivalence,
  required-check replacement, or workflow-removal claim is made.

## Slice and deletion gate

The evidence slice is `.github/workflows/ferris-web-docs-shadow.yml`.

- Primary input: actual base, head, and tested revisions.
- Manual dispatch compares the selected head with an explicit base revision,
  defaulting to `origin/main`, rather than only the final commit.
- Contract: `.ferris/owner-validation-domains.json`.
- Controls: web-only, mixed Cargo/web, deleted web, undeclared fallback,
  path-prefix boundary, and invalid overlap.
- Owner result: `npm ci` plus `npm run build`.
- Proof: retained plans with native revision bindings and
  `bisect.owner-validation-receipt/v1`.

No existing workflow is removed. Trigger narrowing or removal of overlapping
Rust/Python jobs requires a separate replay and required-check migration.

## Prior hosted evidence (superseded)

Run `33318111484` against Ferris
`73ee870fd7e7689637a93bfb835fcbf8d1ccda4e` passed the owner build and
retained:

- Ferris plan
  `validation-plan:f2aa57418a85a96f332d9a0f69e08e304b9a439b28d3f4551fbc4afdafd25d66`;
- input disposition `full_workspace_fallback`;
- `fallback.required_by_inputs: true`; and
- owner status `passed` for `npm run build`.

The first receipt correctly bound the tested pull-request merge revision but
called it `source_revision`, which could be mistaken for the pull-request head.
The follow-up made `tested_revision`, `head_revision`, and the pinned
`ferris_revision` separate evidence fields.

The native revision-binding migration deletes BISECT's duplicate name-status
parser. A path-scoped `git diff --quiet` remains only as an independent
`web/docs` selection oracle. The Node adapter uses the same explicit 32 MiB
subprocess-output bound as Ferris so valid revision-derived plans are not
silently constrained by Node's smaller default. Hosted acceptance now requires
a green run against Ferris `1f2a9d1d7047d676f5fe36832e646535d7d43c60` with
the v1 receipt, native revision binding, and all six scenario records retained.
