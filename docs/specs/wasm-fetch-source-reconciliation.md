# Fetch/FLETCH reconciliation and first real-data matrix

Date: 2026-10-23. Status: source reconciliation and a bounded real-data tier
executed; full WP-01/WP-05/WP-08 and release acceptance remain open.

Subsequent scoped delivery: [school/EIA archive acquisition](wasm-fetch-archive-acquisition.md)
now adds pinned acquisition, local import, raw cache/project handoff and explicit
native-layout extraction/export. The table and execution narrative below record
the initial tier at its checkpoint; its immutable snapshot preserves those older
source identities. The machine-readable source table now reflects the delivered
archive follow-up. Shared zone-join and broader acquisition obligations remain open.

[Scoped roles review](../../signals/roles/check/wasm-fetch-source-reconciliation-roles-check-2026-10-23.md)
and an [immutable local evidence snapshot](../../target/fetch-source-reconciliation/accepted-2026-10-23/snapshot.json)
retain the declared sources, requests, raw outcomes, actual served assets and
both engine identities. Raw national-data prerequisites stay external and hashed.

The [machine-readable source table](wasm-fetch-source-coverage.json) covers all
12 shared Fetch `DataType` declarations. Each row names native action, browser
action, downstream consumer, source references, classification, owner and next
test. Its source digests fail on drift. This is a planning/consumer reconciliation,
not a claim that every listed source can be downloaded or consumed in the browser.

| Selection | Initial-tier behavior / remaining gap at that checkpoint |
|---|---|
| TIGER and redistricting | Native FLETCH acquisition and ZIP extraction; browser pinned acquisition, retained ZIP decoding and paired state handoff exist. Full native extraction/completion layout equivalence remains open. |
| Adjacency | Native Fetch invokes a release adapter only with `--release`; browser local native-adjacency import exists. Release acquisition remains a distinct gap. |
| Enacted and geography | Shared Fetch declarations produce no items. Do not infer missing unrelated import/practitioner actions from that no-op. |
| Elections | Explicit selection warns to use the Python downloader; shared planner returns no items. Separate Python acquisition and local election consumers need their own contracts. |
| LODES WAC/OD and ACS housing | Bounded browser acquisition, cache, preparation and native derived-data ZIPs exist. Preserve WAC versus OD semantics, ACS estimates versus PL94 counts, API-key non-persistence and native header-only 404 outputs. Full lifecycle acceptance remains open. |
| School districts and EIA-861 | Native Fetch acquires ZIPs. Browser public acquisition currently rejects these families. Native Run's zone spatial join is also unimplemented and warns/falls back to geographic weights. Explicit prepared zone assignments are a separate supported path; downloading the archives does not implement a spatial join. EIA is national, not per-state. |
| Empty / `all` | Selects TIGER, PL94 and adjacency, plus separately requested extras. It does not automatically enable every declared family. |

Two native details need explicit lifecycle work: ZIP dispatch writes a completion
marker after extraction without validating the planner's target; PL94's planned
target names the ZIP while extraction produces members. Also `verify_downloads`
is parsed but has no read in the reviewed native Fetch dispatch. These are shared
behaviors/limitations to investigate, not reasons to invent browser effects.

FLETCH owns registry/cache/raw source acquisition; BISECT owns extraction,
derivation, marker semantics and actual consumers. A generic-ready handoff is
metadata, not acquisition, source authenticity or usable weighting evidence.

## Executed evidence

`test_fetch_source_coverage.mjs` compares all 12 type selections plus empty and
All-with-extras over 2000/2010/2020. Each selection includes RI/NC and a repeated
RI to exercise native item repetition versus FLETCH definition deduplication.
Both actual default and C-enabled WASM artifacts passed 42 exact native filesystem
planner comparisons and 42 native FLETCH report comparisons apiece. Tests also
check current public acquisition eligibility, warnings and single national EIA
items. No network request, extraction or partition occurs in these comparisons.

The [predeclared real-data manifest](wasm-real-data-matrix.json) pins the first
RI/NC/CA 2020 tract cases: original SHP/DBF/SHX/PRJ/PL94 byte identities,
retained compressed and decoded graph identities, historical derivation witness,
tracked request recipe, exact generated request hash and fixed expected native
outcome. The declaration refuses overwrite. Inputs remain an explicit retained
integration tier; they are not copied into the static site or silently replaced.
Its declaration-time `predeclared-not-run` status is immutable; later execution
records, rather than edits to that manifest, report outcomes.

| State / units / districts | Both-build native-adapter and WASM outcome | Actual packaged GUI |
|---|---|---|
| RI / 250 / 2 | Complete exact reply match; independent population/contiguity checks pass with seed 42. | Exact request/result, visible seed, Save/Open inertness and 390px reopen checked. |
| NC / 2,672 / 14 | Exact expected failure: `apportion-regions balance 36.1% exceeds 3% research limit`. | Same request and visible failure; accepted-run download stays disabled. |
| CA / 9,129 / 52 | Exact expected failure: `apportion-regions balance 43.9% exceeds 3% research limit`. | Same request and visible failure; accepted-run download stays disabled. |

Each build attempted three cases: one successful execution, two matching native
failures, zero unexpected failures. The C-enabled artifact ran these rows with
`metis-core`; availability of its C bridge is not C execution. Browser checks
serve the existing tested toolkit with each run's frozen engine and retain hashes
of actually served assets. This exercises those exact bytes; it does not create a
new release package or rerun the whole toolkit. No production module changed.

These cases compare a native host using shared adapters. They do not prove fresh
raw-to-graph derivation, independent native CLI orchestration, map/report follow-ups,
recovery/installation, successful NC/CA support, all methods/vintages/resolutions,
OS folder pickers, national capacity or full desktop parity. Each remains explicit
in the matrix. Source hashes and prior derivation witnesses do not establish
provider authenticity or a newly verified derivation.

## Run and acceptance boundaries

Portable source reconciliation, after building native references:

```powershell
cargo build -p bisect-cli --no-default-features --example fetch_plan_reference --example fletch_sources_reference --locked --offline
node scripts/web/test_fetch_source_coverage.mjs --wasm=path/to/engine.wasm
```

It is registered in the standard toolkit, laboratory and workspace preview
builders. The amended builder entry points were syntax checked; a complete new
builder/package run was not performed for this source/test/documentation slice.
Historical build/package records are preserved. The declared integration tier
is explicit, not a clean-checkout default build prerequisite:

```powershell
node scripts/web/test_real_data_matrix.mjs --wasm=path/to/engine.wasm
# Supply the printed run directory; Playwright package root must be configured.
node scripts/web/test_real_data_matrix_browser.mjs target/wasm-real-data-matrix/run-ID dist/wasm-workspace-continuation-validated/toolkit
```

Matrix runs freeze native/WASM bytes, retain exact requests/raw replies/errors,
check declared inputs and write per-row outcomes. Missing declared inputs produce
not-run and a nonzero incomplete outcome. Unexpected outcomes fail; they never
rewrite expected outcomes. Each browser run has its own profile/context and
server; it does not attach to the user's tabs. Open confers no execution authority.

Next: select and deliver the bounded missing acquisition lifecycle (school ZIP
and national EIA must retain separate scope/consumer contracts), expose precise
unsupported-source reasons, and finish the WP-07 practitioner action inventory.
Expand the matrix with successful medium/large workloads and fresh derivation,
then browser follow-ups/recovery and national rows. Preserve all existing native
failure witnesses. Full parent evidence remains partial.
