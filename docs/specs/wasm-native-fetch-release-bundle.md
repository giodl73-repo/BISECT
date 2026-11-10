# Native Fetch release ZIP

The adjacency release controls offer an unchecked **Export complete native Fetch
ZIP** option. Find performs complete bounded discovery for the selected native
`<state>_adjacency_<year>*` wildcard. Download exports all matching original assets,
including opaque pickle/request/display files, without replacing engine inputs,
decoding those files, preparing a graph or executing an operation.

This is separate from the default engine-input import, which still excludes
pickle/request/display/custom Census county sources. The explicit site-bridge
checkbox retains its disclosure that the optional token is sent to this site.
The Worker re-discovers the configured release with the explicit bundle purpose,
checks asset membership, follows recognized HTTPS asset redirects manually and
omits Authorization after the API hop. Genuine CDN JSON assets may be retained as
opaque bytes; API metadata returned instead of an asset still refuses.

The ZIP reproduces native Fetch's `manifest.local_outputs_dir`-relative
`V3/data/<year>/adjacency/` layout and `<state>_adjacency_<year>.done` containing
exactly `done`. The receipt is separate at the ZIP root. Every original file is
hashed, published digests are checked when present, and all files validate before
any ZIP is offered. No partial bundle is exported. Cancellation and revision checks also guard each package hash and the final ZIP offer. Changing settings, Clear,
cancel or Open retires the live selection and credential as before.

Native Fetch's presence predicate requires both the primary `.pkl` and marker.
A binary-only release can still reproduce native marker creation, but the receipt
records `primary_asset_present=false`. An invalid pickle can satisfy native file
presence: that predicate is not graph validation, publisher authenticity or
execution evidence. Exported marker bytes confer no browser filesystem authority.
The ZIP does not inspect an existing target, install, overwrite, skip or force files.
Actual selected-folder review/application remains a separate lifecycle step.

Discovery retains the 2,000-asset listing bound. Bundle limits are 256 matching
files /64 MiB total, including empty opaque assets. Filenames must be portable
ASCII basenames of at most 180 characters. Unsafe paths/characters, trailing dots,
case-fold collisions and an asset colliding with the generated marker refuse
before body download. These explicit safe-package constraints do not claim every
filename that an unrestricted native gh download might accept.

## Verification

`target/adjacency-release-bundle/package-1793032264011081700/evidence.json` passes
16 stages across unchanged default/C-enabled engines: offline/assets, actual native
CLI layout/marker comparison with controlled gh transport, input-client and bridge
regressions, full real-browser release journeys and prior election-source browser
regressions. Independent Python ZIP reading verifies CRCs and exact native output
files. Synthetic opaque files, empty bodies, unsafe names, altered bytes and digest
failure are exercised. The browser verifies that bundle export leaves project
sources unchanged and sends no engine request.

Additional name/presence checks are retained in
`target/adjacency-release-bundle/names-1793032031560/evidence.json`: case aliases,
combined byte limits and wrong-state inventory refuse; pickle-only bundles remain
opaque and binary-only bundles do not report the primary pickle present.

With the user-authorized existing gh login, both sealed browser profiles also
downloaded the actual private RI 2020 release through the locally hosted production
bridge. Each ZIP equals a fresh actual native CLI gh Fetch run, including the
original pickle, binary, GEOIDs and marker:

- Default: `target/adjacency-release/private-browser-1793032583676/evidence.json`.
- C-enabled: `target/adjacency-release/private-browser-1793032615282/evidence.json`.

Project sources remain unchanged and no engine operation runs. Credentials and
signed URLs are absent from exports; contexts close and cached HTTP is disabled.
The first default attempt hit `ERR_CONNECTION_CLOSED` in preflight before any
authenticated GET; `private-browser-1793031799815` retains the sanitized failure.
The C profile succeeded and a later explicit default retry succeeded. No production
auto-retry or mocked success was inserted.

Wrangler 4.153.0 dry-run bundles the production Worker and binds 739 staged assets:
`target/adjacency-release-bundle/worker-dry-run-final-1793032340/evidence.json`.
The CLI output is retained as a labeled normalized transcription. The Worker
bundle is 111.76 KiB /17.62 KiB gzip. This is a local build check, not deployment
or Cloudflare runtime/origin/log/rate-policy acceptance. Native skip/force/target
installation, other releases/scales, source authenticity and full parent /
acceptance-v1 remain open.


## Cancellation regression and observation windows

Targeted real-browser fault injection pauses the first package hash after all
asset bodies have already validated, then clicks Cancel. The original prototype
offers one ZIP and fails the test
(`target/adjacency-release-bundle/cancel-browser-1793032709424/evidence.json`).
The final v2 must offer zero ZIPs and run zero engine operations; its passing record is
`target/adjacency-release-bundle/cancel-browser-1793032714958/evidence.json`. The prototype is retained as a failed
boundary, not accepted by the final package's passing stages.

The first v2 private-provider observations used the older harness's 20-second
phase budget, shorter than multiple legitimate 30-second product requests. Those
terminal observation failures and the exact old harness are retained under the
source-before tree. The final explicit checks use a 120-second observation budget;
production per-request deadlines and cancellation behavior are unchanged.

## Retained scope

`target/adjacency-release-bundle/snapshot-scope/snapshot.json`, SHA-256
`86181cdacefbdb514a4c5d0f1ba08f9372dd7ad0b6840c05cdac9d36da72a791`,
fully verifies 1,610 mappings /539 identities /504,316,471 logical bytes. It retains
final v2 packages, both fresh actual private/native comparisons, cancellation
red/green, name/presence cases, Worker dry-run and pre-closure contract/review.
Historical observation records are retained as diagnostics with their exact old
harness/prototype helper, rather than treated as current-source acceptance.
This post-freeze pointer leaves broader lifecycle/host and acceptance-v1 open.
