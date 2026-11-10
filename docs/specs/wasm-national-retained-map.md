# Full national retained-plan display map

Date: 2026-10-24. Status: fresh native/C-enabled stream comparison and both actual
default/C-enabled staged map GUIs passed; immutable evidence/scoped support
closure remains pending. Owner: coding agent; maintainer acceptance is
separate. WP-07/WP-08.

This follows `national-50-2020-tract-standard-core-42-v1`: all 50 states, 435
districts and 84,208 units, with the same accepted standard-bisection plans.
The declaration at `target/national-matrix/map-1792829919513/acceptance-contract.json`
was written before rendering. It binds every accepted plan and retained geometry
source, 72 DPI, district display, explicit batch dissolve and stop-on-failure.
No state is omitted or replaced, and no new partition is executed.

Geometry retains all features from the existing display-topology sources: NAD83
degrees / EPSG:4269, 0.0002-degree shared-boundary-conservative simplification.
It contains 11,252,139 input positions. The final dissolved map has 1,101,055
positions. This is a display contract, not fresh Census authenticity, unsimplified
geographic measurement or a legal-compliance finding. Graph validity/population
checks remain bound to the separate generation campaign.

The fresh native instance-local protocol reference is recorded in
`target/national-matrix/map-engine-1792829799214781500/evidence.json`: original
build command/log, source digests and owned executable. This is a shared native
kernel reference, not CLI filesystem orchestration. All 52 native replies match
actual C-enabled WASM at `target/national-matrix/map-1792829919513/evidence.json`,
including the complete SVG and PNG. The 1,152-by-720 PNG and source-preserving
archive were inspected; the archive reopens exactly.

The map input plans are byte-identical across both actual metis-core generation
profiles, recorded in `cross-profile-plan-identities.json` beside that evidence.
The original retained source package is 98,790,616 bytes. Reference data is shared
read-only between browser tests, rather than copied or regenerated to create a
different national workload.

Actual C-enabled staged browser evidence:
`target/national-matrix/map-1792829919513/browser-1792830241708/evidence.json`.
It uses `dist/wasm-national-queue-preview/toolkit`, renders all 50 states, displays
and decodes the image, downloads exact native PNG/SVG, saves identical sources
and result, and reopens with exports disabled and no execution. Exactly one map
operation and zero partition operations dispatch. Source/runtime identities and
the final served modules are retained. The actual default-profile staged map
test also passes at
`target/national-matrix/map-1792829919513/browser-1792830429589/evidence.json`,
using `dist/wasm-national-queue-default-preview/toolkit`. It matches the same full
native result and exact sources/exports, without changing the accepted plans.

Reproduction entry points: `test_national_matrix_map.mjs` for the complete native/
WASM stream and archive comparison, and `test_national_matrix_map_browser.mjs`
for isolated loopback actual staged UI/worker tests. Both use owned fresh evidence
directories and preserve failures. Large integration prerequisites and sufficient
evidence storage are required; missing prerequisites yield not-run.

Neither a passing map nor the separate queue/recovery tier closes fresh source
derivation/acquisition, other years/unit kinds/methods, real selected-folder host
permissions, immutable evidence review/support registration or full desktop parity.
Historical packages and public hosting remain unchanged.


## Scoped evidence closure — 2026-10-24

The [seven-role implementation review](../../signals/roles/check/wasm-national-browser-capacity-roles-check-2026-10-24.md)
approves this local preview with conditions. One coding agent applied the repository
role perspectives; this is not independent maintainer or release acceptance.
Both registered build profiles and all 29 final-package gate outcomes pass.
The immutable local snapshot is
`target/national-matrix/package-snapshots/wasm-national-queue-preview-v4/snapshot.json`,
SHA-256 `257707f9d3fb4e9533cbd0fd502d6d7b78d68b7eab2db555a4c1fbd9c74648c0`.
Full verification covers 9,586 mappings and 3,201 content identities, including
historical failed attempts and the exact tested default/C packages. Earlier failed
freeze attempts remain retained; the completed snapshot passed full readback.

This post-freeze registration changes documentation and support records only.
The Run, Map and TUI scoped child records are implemented with partial evidence;
parent reconciliation, acceptance-v1, real host/API coverage, fresh acquisition,
other vintages/resolutions/methods and deployment remain open. Tested runtime and
package bytes remain unchanged. The map reference is a shared native renderer
sequence, not native CLI directory discovery; national generation uses actual CLI
Build references for configured jobs, not full Run filesystem equivalence.
