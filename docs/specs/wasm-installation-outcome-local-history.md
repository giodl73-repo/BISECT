# Local history for installation outcomes

The browser now keeps one bounded local observation of the most recently started
native-output installation/recovery attempt. **Download last stored outcome** and
**Forget stored outcome** appear beside project status, including after a reload
or opening another project. This is separate from the current project's session
outcome and live folder authority. Nothing resumes automatically.

Before installation begins, the UI attempts to commit an `in-progress` observation
with no final receipt. It then records the completed/cancelled/interrupted outcome
when available. Closing the page during a body write leaves the progress marker;
on reload the UI explicitly reports unknown completed writes. It does not infer
failure, success, absence of effects or rollback from that marker.

## Storage and lifetime

A dedicated IndexedDB database, `bisect-installation-outcome-v1`, stores one JSON
string using `bisect-local-installation-outcome-v1`. The record includes an attempt
UUID and the existing passive `bisect-browser-installation-outcome-v1` wrapper.
Limits are one MiB UTF-8, 50,000 JSON values and the existing strict JSON parser's
bounds. Unknown top-level/wrapper fields, invalid schema/status/hash/UUID values,
hostile JSON and an in-progress marker with a receipt refuse. Nested receipts stay
verbatim JSON data, not independently authenticated execution evidence.

Initial saves replace the previous stored observation. A final save only updates
its matching attempt ID in a read/write transaction: an older tab cannot overwrite
a newer attempt. Forget likewise refuses to delete a valid newer record; explicitly
forgetting malformed stored data is permitted. The startup read cannot replace a
newer in-session save. IndexedDB connections close after transactions and on a
version change. Open/transaction timeouts report storage failure.

The browser may clear/evict storage; private profiles, quota, blocked storage,
closing before the initial transaction commits and origin changes can prevent
retention. No persistence permission or cross-device sync is claimed. Storage
failure leaves the existing installation workflow available and visibly asks users
to download the session outcome. The UI never claims that unsaved data was stored.
The single-record design is not a complete multi-attempt audit history. Two active
tabs may replace the last-started observation; the older terminal save is refused.
Download a portable backup for custody beyond browser storage.

## Authority boundary

Stored history is unverified, passive metadata. It has no folder handle, installer
review or execution grant and is never consumed as an installation/recovery request.
It cannot make imported projects fresh. Automatic startup reads only this bounded
record; it starts no worker execution, picker, partition or target write. Opening
another project still clears the original session outcome and all prior live review
authority; the separate stored observation remains downloadable. Forget deletes
only the local history record, never output files or native interruption markers.

The saved wrapper preserves original installer receipts and error text; paths may
be present in a deliberately downloaded local file. There is no upload service.
Actual current folder bytes must be reviewed before any recovery, even when the
stored observation says completed. No legal or election certification is implied.

## Validation — 2026-10-24

`target/install-outcome/validation-1792864144904949300/evidence.json` passes nine
checks: strict storage schema/refusal tests, and asset closure, offline closure,
single recovery and batch recovery on both staged engine profiles. Each browser
journey regenerates its synthetic fixture and uses genuine OPFS with an injected
picker. Real Windows selected-folder permission grants remain outside this tier.

Both profiles pass exact stored-record download after real reload, no automatic
execution/picker/review authority, atomic stale-tab final-save/Forget refusal,
oversized and unavailable/quota failures, and Forget followed by reload. The batch
journey reloads during an actual paused body write, retaining the progress marker
and native `manifest.tmp`; the page reports unknown completed writes and starts
nothing. The single journey completes a real installation with browser outcome
storage unavailable and preserves the exact receipt for session download. Existing
exact file/checkpoint/recovery, inert Open and 390-pixel layout assertions pass.

Packages: `dist/wasm-install-outcome-default-durable-preview` and
`dist/wasm-install-outcome-c-sidecar-durable-preview`. The staging record is
`target/install-outcome/stage-1792864084533756100/evidence.json`. Existing accepted
packages and engine bytes remain unchanged; no Rust or installer semantics change.
This does not repeat the national campaign or accept broader native parent, host,
raw-source acquisition or release parity. Nothing is deployed.


## Scoped evidence closure

The [five-role implementation review](../../signals/roles/check/wasm-installation-outcome-local-history-roles-check-2026-10-24.md)
records three locally closed P2 findings and 12 bounded/carried P3 conditions.
These are one coding agent's installed lenses, not independent maintainers.
The independently readable scope snapshot is
`target/install-outcome-durable/snapshot-scope/snapshot.json`, SHA-256
`3c8a1964224a271094237e4c214df73b034538d58c9f28a06149fc38ec77d156`.
All 1,508 mappings /490 content identities /482,709,610 logical bytes passed full
readback verification. Reused bytes link only digest-verified immutable blobs;
this scope snapshot has no inherited-blob dependency. It includes the two tested
shells, exact browser records/downloads, source baseline and producing scripts.
Earlier historical evidence remains separately retained. This closure and support
registration are post-freeze documentation; the frozen contract retains its
pre-registration bytes. Run/State/States parents stay partial and acceptance-v1
remains false. No deployment follows from this registration.
