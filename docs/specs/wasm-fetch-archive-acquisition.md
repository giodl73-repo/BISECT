# School-district and national EIA archive acquisition

Date: 2026-10-23. Status: scoped local package and both-build/native/browser checks pass.
Complete Fetch/FLETCH, zone weighting,
WP-05/WP-08 and release acceptance remain open.

[Scoped implementation review](../../signals/roles/check/wasm-fetch-archive-acquisition-roles-check-2026-10-23.md)
and the [immutable local snapshot](../../target/fetch-archive-acquisition/package-snapshots/wasm-fetch-archive-acquisition-validated-v2/snapshot.json)
bind this scope to `dist/wasm-fetch-archive-acquisition-validated-v2`. Its offline
identity is `64a5943b10d1543b370b3545705eddd25e51c5d16a3209e8739029dca344216b`.
The C-enabled engine remains
`56d1898c7743dbed3634e342112d061b9400e37d3d040e76def74ad4bf13704b`;
the actual default build remains
`60a69fd0ad32af37c3853ba423a6af5487d27c5f80937dceac323a80966b24cd`.
All 651 package hashes/sizes, 271 toolkit files and 1,109 dependency resolutions
passed. These counts establish asset closure, not capability completion.

The workbench now offers school-district ZIPs and national EIA-861 ZIPs from a
fresh native Fetch plan. Users can acquire the pinned public URL or import a
local ZIP for that selection. Acquisition retains the original ZIP and receipt,
then selects **Prepare native Fetch ZIP extraction package**. Run is separate:
it checks the retained bytes, inspects the archive and prepares an explicit ZIP
download containing native destination paths and the native `done` marker.
It installs no files and performs no partition or spatial join.

Existing TIGER/PL94 paired state workflows and ACS/LODES preparation remain
separate. The added archive package operation can also inspect a retained
TIGER/PL94 ZIP through an explicit saved selection; that does not replace their
existing Census preparation workflow or expand their advertised capacity.

## Native authority and observable outcomes

The authority is `fetch_plan.rs::build_fetch_list`, native
`fetch.rs::download_items`/`extract_zip_file`, and FLETCH raw acquisition/cache.
School URLs are state/FIPS scoped; EIA is one `US` item per selected vintage.
The current native vintages are 2000/2010/2020. A planned URL is not evidence
that a provider currently serves that vintage.

The native ZIP branch extracts every member under the planned target's parent
and writes its marker afterward. It does not require the planned target to
exist. The package preserves that distinction: its summary reports target
presence separately from extraction/marker completion. It never fabricates a
missing `service_territories.shp`. Incremental skip requires both target and
marker in the supplied native selection observations. Force rebuilds the
package; it grants no filesystem overwrite authority. Marker-only observations
do not skip. Imported observations and markers are not installed-byte evidence.

Native Run still warns and falls back to geographic weights for the unimplemented
school/EIA zone spatial join. Explicit prepared zone observations are a separate
path. These downloads do not make the native spatial join available in WASM.
Other Fetch release adapters, shared no-op types and unused native flags retain
the [source reconciliation](wasm-fetch-source-reconciliation.md) boundaries.

## Network, local files, cache and project contract

- Public requests use exact pinned Census school/EIA URLs, no browser credentials,
  no redirects, bounded bodies, cancellation and the existing retry/timeout policy.
  The relay adds these same pins with the same limits; ACS keys stay direct-only.
  Updating repository relay code does not update a hosted Worker automatically.
- Local ZIP import reads only the explicitly selected file. Its receipt records
  `transport: local-file`, a null HTTP status and no network attempts. The public
  URL remains a source claim. Local import performs no provider/cache request,
  key use, refresh or retry. Original selected filename and bytes are retained.
  This is an additive receipt transport; previous receipt meanings are unchanged.
- Raw retained ZIPs use the existing single-copy `fetchArchiveSource` reference
  for explicit FLETCH staging and indexed browser-cache storage. Reuse runs the
  existing checked cache reader, then inspects the ZIP again. Acquisition does
  not silently store or publish a cache object.
- Projects store inert source bytes, receipt and native selection/settings.
  Open never fetches, extracts, partitions, restores a folder grant or installs.
  Imported results cannot enable download until explicit Run. Changes retire
  fresh exports; asynchronous download checks its original revision/result again.
- Limits are 64 MiB compressed, 128 MiB expanded, 10,000 members and the existing
  portable path/depth rules. Generic Fetch expansion uses a named decoder layout;
  native-input and output-verification layouts retain their earlier 64 MiB bounds.
  CRC, local/central header, hidden/overlapping bytes, alias/collision, traversal,
  reserved-name, link/special-file and compression checks run before export.
  Empty/fileless archives are unsupported.
  Export uses portable package paths and a 128 MiB ZIP-source limit, including
  the marker. Existing project/export limits still apply and failures remain visible.

The browser ZIP boundary is stricter than native `extract_zip_file`, which joins
archive paths into filesystem destinations. Hostile archives are rejected in
the browser; this tier compares valid native extraction and claims no native
hostile-archive hardening. Empty directory effects and OS-specific filesystem
behavior are outside the exported-file comparison. No archive member is executed.

## Evidence and integration tiers

`test_fetch_archive_acquisition.mjs` uses a tracked generated-body recipe for six
cases: both source families in all three vintages. Each invokes the actual native
downloader/FLETCH/cache/extractor/marker through an owned loopback provider.
Only transport URL differs from the native public plan; that override and both
original URLs are recorded. Native filesystem output bytes equal the browser
package bytes. These are native function comparisons, not full CLI orchestration.

The cases cover raw-byte/project retention, exact u64 staging metadata, national
scope, source-hash tampering, marker-only/skip/force behavior, offline decoder
handoff, retry, 404, cancellation, local import without HTTP claims, relay pinning
and hostile ZIP paths/CRC. These generated member bodies test extraction; they
are not valid GIS datasets or authenticated EIA workbooks.
Native reference comparisons here are six successful download/extraction cases;
retry, 404, cancellation and imported observation guards are browser-host cases.
They do not establish native error/delay/force orchestration equivalence.

Actual browser tests use the workbench, archive/engine workers and indexed cache:
explicit acquisition then Run, exact native-matched download, source-preserving
Save/Open, edit freshness, explicit raw staging/storage, offline reuse without a
provider request, local file import, 404, cancellation and a 390px layout check.
Default and C-enabled engines run this path without executing the optional C
partitioner. Shared worker/decoder changes also retain existing project, workspace,
configuration, State, composed Verify, continuation and optional-C regressions.

Explicit live observation on 2026-10-23 downloaded the RI 2020 school ZIP:
298,372 bytes, SHA-256
`1afb6271c21fe37ff7b40e83e4b4e53ed6040c2319b7189c15386672dd97923b`.
Its seven members expand to 536,957 bytes and include the planned SHP. The EIA
request failed at the provider/network boundary in this environment. This records
one observation, not universal EIA unavailability. Node observation does not test
browser CORS or deployed relay behavior and establishes no source authenticity.

`test_fetch_archive_real.mjs` is a separate explicit integration tier using that
retained actual school ZIP, with a case declaration and exact native extraction
comparison. The same browser journey can consume its output. Ordinary builders
do not require that untracked retained provider data or access live providers.

```powershell
cargo build -p bisect-cli --no-default-features --example fetch_archive_reference --locked --offline
node scripts/web/test_fetch_archive_acquisition.mjs
# Supply its printed root; configure BISECT_PLAYWRIGHT_PACKAGES first.
node scripts/web/test_fetch_archive_acquisition_browser.mjs target/fetch-archive-acquisition/RUN
# Explicit integration/provider observations, never implicit build prerequisites:
node scripts/web/inspect_public_fetch_archives.mjs --run
node scripts/web/test_fetch_archive_real.mjs target/fetch-archive-live/RUN
```

The registered build gates retain actual native/WASM identities and raw outcomes.
Both actual builds passed the extended legacy gates. The final C-enabled package
passed its new archive journey, selected workspace/reset/continuation/recovery,
paired acquisition, TUI/composed Verify and optional-C regressions. Default-engine
GUI evidence uses the current source tree; it is not a separately staged default
package claim. Both engines also have retained actual-school-archive comparisons
and GUI checks, including the final C-enabled package.

The first package attempt's legacy TUI Verify browser check timed out while
waiting for import status. A standalone rerun passed. Its test now waits for the
specific imported project/readiness and emits failure diagnostics; the complete
fresh v2 package run passed. The first failed candidate and log remain retained;
the cause of that initial timeout was not independently established. No failure
was converted into a pass by changing recorded hashes or expected verification.

Broader source
types, streaming/disk acquisition recovery, actual EIA-provider/workbook coverage,
shared zone joins, fresh graph derivation, successful larger-state workloads,
practitioner inventory and full release acceptance remain open.
