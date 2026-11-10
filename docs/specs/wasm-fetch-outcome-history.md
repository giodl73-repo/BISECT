# Passive local Fetch attempt history

This extends reviewed native Fetch folder application with one bounded latest
local observation. It is an operational aid, not proof of source authenticity,
folder contents, successful recovery, certification or authority to write.

Before Apply issues filesystem effects, the UI attempts to persist an
`in-progress` record with no final receipt. After completion, cancellation or
failure it attempts to save the exact terminal installer receipt. These are two
checkpoints, not per-file durable logging. Closing the page between them leaves
`in-progress`: completed writes are unknown even if some files or a marker exist.
Filesystem writes and IndexedDB cannot be committed in a single transaction.

On reload, the observation is passive. No directory handle, review grant,
downloaded bytes, token, Force selection or continuation is restored. The user
selects the actual folder, acquires source files and reviews it again. Native
presence skip still applies; Force requires an explicit choice. Refresh, download
and Forget operate only on this observation. Open retains passive history while
dropping the current folder authority through the existing invalidation path.

## Storage and concurrency contract

`bisect-fetch-installation-outcome-v1` is a separate IndexedDB database from
Run/State/States history. Each has an `outcomes` store and one `latest` entry.
The new wrapper schema is `bisect-local-fetch-installation-outcome-v1`, containing
an attempt ID and `bisect-fetch-installation-history-v1` outcome. A terminal raw
receipt uses `bisect-native-fetch-install-outcome-v1`. Session downloads preserve
the raw receipt format when one exists; stored downloads include the wrapper.

Text parsing is bounded to 1 MiB and 50,000 JSON values, rejects duplicate and
unsafe keys, and checks exact scope, fixed native paths, versioned schemas,
allowed fields and consistent effect flags. Saving validates JSON data before
cloning and serializing, avoiding getter or custom `toJSON` execution. Stored
values remain unverified and editable by the local user or same-origin code.
The record retains the folder's display name; downloaded backups include it.

A new explicit Apply replaces the latest observation. Terminal saves and Forget
compare the expected attempt ID inside the write transaction; an old tab cannot
replace or forget a newer valid record. The UI guards asynchronous reads with an
epoch so an earlier Refresh cannot replace a newer local state. Attempt IDs are
coordination identifiers, not authentication. This does not lock files against
another tab, native program or process.

Storage open and transaction deadlines are bounded. Storage refusal does not
cancel an already authorized Apply: the UI keeps a warning and session download.
Browser clearing, eviction, device loss and a failed initial checkpoint can lose
history. There is one latest observation, not an archive; keep downloaded backups.
Forget removes only local history, never native files.

## Validation scope

`test_fetch_install_outcome_store.mjs` checks hostile and oversized records,
effect/path/scope consistency, accessor and serialization hooks, fixed database
namespaces and explicit storage failures. The old outcome-store test still passes.

`test_fetch_install_outcome_browser.mjs` uses real Chrome IndexedDB and OPFS with
a controlled picker and an intercepted close callback. It closes the page after
the first actual file write, reloads the origin and verifies the unknown-effects
observation and absence of restored authority. Explicit re-review completes;
the terminal receipt survives reload. Real two-tab CAS save/Forget, namespace
isolation and storage-unavailable Apply/session-download cases are exercised.
The fixture requires retained original RI release assets and local Chrome.

This original tier proves the bounded page-close browser-storage path. It does not
prove external native-directory permissions, browser-process crash/power-loss
durability, cross-process locking, another browser/OS or a hosted deployment.
The process-termination follow-up below supplies separate checkpoint evidence. Full parent/acceptance-v1
remains open. No engine or Rust implementation changes are involved.

## Retained local evidence

Both default/C-enabled shells pass all 16 staged checks in
`target/fetch-outcome-history/package-1793038807337809800/evidence.json`; source
and tested package identities remain unchanged during the gates. The independent
scoped snapshot `target/fetch-outcome-history/snapshot-scope/snapshot.json`, SHA-256
`e3dd06b1fda9e3495d90bdca92ae919d7e352c29157e2fb9f993441cb220f0f1`, fully
verifies 1,756 mappings /507 identities /479,122,583 logical bytes. It retains the
pre-closure contract/review, exact packages, test records and source preimages.
This post-freeze evidence pointer does not change native/host acceptance limits.

## Abrupt process-restart follow-up — 2026-10-26

`test_fetch_install_outcome_crash.mjs` runs the unchanged sealed packages in a
fresh persistent Chrome profile on Windows. It force-terminates the entire owned
browser process tree with `taskkill /T /F` and awaits parent exit, then relaunches
the same profile and origin. Three checkpoints are distinct:

1. Initial Apply after the first real OPFS file close: in-progress history survives,
   receipt remains null, actual partial bytes remain and the marker is absent.
2. Forced replacement after old-marker removal and the first source close: history
   remains in progress and the old marker stays absent. Historical force=true does
   not select Force in the restarted UI.
3. After the verified marker and terminal receipt: the same receipt and every
   original source hash survive restart.

Each restart restores no picker choice, source bytes, review, live Force or automatic
continuation. Explicit selection/acquisition/review recovers the first two cases.
The profile contains only local fixtures, is checked within its owned absolute
boundary and is removed after the test. Sealed package identities and test source
identity are verified; no application or engine source changes are required.

The initial harness completed checkpoint assertions but hung in teardown because
CDP disconnect did not terminate Chrome. That assisted prototype is retained as
`checkpoint-assertions-passed-harness-cleanup-incomplete`, excluded from acceptance.
The final gate explicitly terminates its remaining owned process in cleanup.

These are observations at named crash checkpoints, not guarantees about all
possible crash timings, power loss, storage eviction or another host. The OS
remains running. External native-directory grants/revocation, filesystem locking,
wider browser/OS cases and full parent/host acceptance remain open. The
[roles follow-up](../../signals/roles/check/wasm-fetch-history-process-crash-roles-check-2026-10-26.md)
keeps those limits explicit.

Final passing process gates:

- Default: `target/fetch-outcome-history/crash-1793039827126/evidence.json`.
- C-enabled shell: `target/fetch-outcome-history/crash-1793039896691/evidence.json`.
- Assisted prototype, excluded: `target/fetch-outcome-history/crash-1793039642837/evidence.json`.

Independent scope: `target/fetch-outcome-history/snapshot-crash-scope/snapshot.json`,
SHA-256 `67ff7d5e35fd6ab7956b865352c7d7f44159e11e7d9566b788547e6ccb429746`.
All 1,758 mappings /509 identities /479,049,064 logical bytes verify. The scope
retains exact packages, final gates, assisted prototype and pre-closure documents;
this post-freeze pointer changes no acceptance boundary.
