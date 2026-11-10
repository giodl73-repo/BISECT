# Successful full-chamber retained-graph matrix

Status: CLI, both actual WASM profiles and bounded GUI journey pass. Broader
WP-08, full source/map/report/recovery/host/national and release acceptance remain open.
Owner: coding agent; milestone acceptance: BISECT maintainer.

The authoritative declaration is the `full-chamber-standard-core-42-v1` follow-up
campaign in [the existing matrix](wasm-real-data-matrix.json). Its original three
apportion-regions rows and expected failures are unchanged. The follow-up fixes
standard bisection, geographic weights, single search, metis-core, requested seed
42 and 0.5% tolerance, before the new CLI/WASM/browser acceptance runs. An
exploratory native-host baseline is retained separately and identified as such.
No failed case was changed into a success by relaxing tolerance or district count.

| State | Units / vintage | Population | Requested districts | Maximum deviation | Accepted attempt / seed |
|---|---|---|---|---|---|
| NC | 2,672 tracts / 2020 | 10,439,388 | 14 | 0.1861986546% | 0 / 42 |
| CA | 9,129 tracts / 2020 | 39,538,223 | 52 | 0.2900560301% | 4 / 46 |

These are observed heuristic results, not optimality or legal-compliance findings.
The C-enabled build ran **metis-core** for these rows; this is not a full-chamber
C-FFI method claim. Seed 42 is the original request; CA's successful retry uses 46.

## Compared behavior

The native binary transcription is lossless: selected prepared decoding matches
the retained graph's complete ordered units, adjacency, populations, edge weights,
areas, exterior perimeters and centroids. Actual `bisect build` CLI runs then
match accepted native-host assignments exactly through the original unit order.
All requested districts are present. Native index summaries show one successful
state and no failed states per independent case.

Both actual WASM profiles match the complete fixed native-host result. Independent
checks recompute population sums, complete assignments, every requested nonempty
district, graph connectivity and the 0.5% population bound with integer arithmetic.
Original raw-source and retained graph identities are checked. This does not prove
a fresh raw-to-graph derivation or authenticate the Census source independently.

The default browser journey uses current source modules and the pinned default
engine. The C-enabled journey uses `dist/wasm-label-build-application-validated/toolkit`
and its exact staged C-enabled engine. Each runs both full-chamber workloads,
saves the exact accepted result, opens it without executing, and reproduces the
same output after another explicit Run. The native RPLAN/RCTX and report-manifest
exports are part of the exact result. Polygon maps, downstream analyses/practitioner
reports, acquisition and OS-folder behavior are outside this journey.

## Retained evidence

- Actual CLI: `target/full-chamber-matrix/native-1792822863039/evidence.json`.
- Default WASM: `target/full-chamber-matrix/wasm-1792822942239/evidence.json`.
- C-enabled WASM: `target/full-chamber-matrix/wasm-1792823060660/evidence.json`.
- Default browser: `target/full-chamber-matrix/wasm-1792822942239/browser-1792823114761/evidence.json`.
- C-enabled staged browser: `target/full-chamber-matrix/wasm-1792823060660/browser-1792823197394/evidence.json`.

The declaration records each evidence file's digest and test-source identity.
Each run retains its original declaration snapshot; subsequent evidence
registration does not rewrite requests or expected outcomes. Scripts are
`test_full_chamber_matrix_native.mjs`, `test_full_chamber_matrix_wasm.mjs` and
`test_full_chamber_matrix_browser.mjs`, with the follow-up recipe in
`fixtures/full-chamber-matrix-case.mjs`. These are retained-data integration tests;
missing prerequisites cannot be counted as successful runs.

## Remaining acceptance

Freeze and verify the new evidence in immutable storage and apply scoped roles
review before support registration. Add map/report outcomes, fresh source
derivation, additional vintages/resolutions/methods, recovery and actual host/API
coverage as their own rows. National mode needs explicit participating states,
full chamber counts, seed/backend policies, acquisition/Run/output/recovery and
native outcomes. Two independent successful states do not establish national
capacity or close the whole compatibility matrix.

## Follow-up checkpoint, 2026-10-24

The original successful CLI/profile/GUI cases are now included in the verified
Build snapshot `d91c574a06608c9429e96e32336321c159eef73f1126056ff6ccc415e868a1bb`.
They remain bounded evidence, not full WP-08 acceptance.

The three existing matrix runners now preflight missing inputs before any native,
WASM or browser execution. `target/full-chamber-matrix/prerequisite-tests-1792825773202/evidence.json`
records missing-graph/request and evidence-capacity refusals as typed not-run
outcomes. The capacity guard describes validation-host evidence storage, not a
browser product minimum. Original passing runner-source pins remain historical;
current guard sources and evidence are registered separately in the matrix.

On the identical accepted NC/CA plans and complete retained display-topology
geometry, sequential dissolve matches a native work-budget refusal. Spatial
balanced completes NC but still matches CA's native refusal. Neither required
candidate was rewritten into a passing case. The original geometry keeps its
0.0002-degree shared-boundary-conservative simplification; no additional
simplification or district/unit reduction was made.

The existing batch dissolve option completes both states' district maps,
compactness maps and reports. Complete replies match native on both actual
profiles, with no partition execution. Missing demographic/election/urban data
remain unavailable/partial. Evidence:

- Sequential candidate: `target/full-chamber-matrix/map-report-1792824341758/evidence.json` (required target failed).
- Spatial-balanced candidate: `target/full-chamber-matrix/map-report-1792824522881/evidence.json` (required two-state target failed).
- Batch/default: `target/full-chamber-matrix/map-report-1792824673295/evidence.json` (passed).
- Batch/C-enabled: `target/full-chamber-matrix/map-report-1792824878824/evidence.json` (passed).

The first actual staged GUI/archive check retained a clock mismatch. Its strict
request comparison exposed only canonical ISO milliseconds (`.000Z`) versus the
older no-millisecond native reference. Subsequent tests retain both originals
and recompute an exact native reference for the GUI's canonical timestamp; no
report field or upstream hash is normalized. A separate test Blob inspection was
corrected after the existing CSP refused a fetch of the preview Blob URL; product
CSP was not loosened.

Observed ENOSPC attempts remain interrupted failures: map/archive validation and
the first national native fixture preparation could not finish their evidence
writes. No accepted package or original source was deleted. Verified lossless
compression of owned evidence and inactive build-cache files recovered space;
validation now performs a capacity preflight. PP-42 records the remaining
OS-wide capacity race. GUI/map support registration and fresh evidence freeze
remain separate from fresh raw derivation, broader recovery/host and other
method/vintage/resolution coverage. The current Core and C-enabled packages
both passed the maintained GUI check and an independent readback of the native
workflow result, archive, maps and ZIP. The comparison used the same retained
accepted plans and prepared display geometry, with no partition execution.
