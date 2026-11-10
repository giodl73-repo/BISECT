# GitHub adjacency release acquisition

Native adjacency tools now offer **Find matching release files**, **Download
matching files**, **Cancel release request**, **Clear release selection** and an
explicit acquisition-receipt download. Repository/tag defaults are checked against
the native manifest. The selected Census state/year and tract scope determine
matching binary, GEOID and centroid names. Discovery lists all asset pages within
a 2,000-asset bound; it does not infer completeness from the release's first page.
Downloading replaces source inputs only after every requested file validates.
Inspection/preparation and plan Run remain explicit.

The browser uses the [GitHub release asset API](https://docs.github.com/en/rest/releases/assets?apiVersion=2022-11-28):
metadata provides asset IDs/lengths and optional published digests, and the binary
request uses `application/octet-stream`. Asset endpoints are constructed under the
selected repository; metadata URLs must match that repository/ID. Metadata redirects
refuse, and binary responses may end only at the expected API endpoint or recognized
GitHub asset hosts. Every observed body is hashed. Supplied SHA-256 digests and
published lengths must match; absent published digests are disclosed, not invented.

## Credentials and imported-project boundaries

An optional private-release token is kept in the live selection's private memory
only. The field clears after discovery. Requests omit browser cookies/referrers and
use `cache: no-store`. Authorization goes to the GitHub API; real browser redirect
checks verify it is removed before contacting the asset host. Tokens and signed
redirect URLs appear in neither project sources nor receipt exports. No token is
read automatically from the native CLI or imported project. No Cloudflare secret
or generic relay is introduced.

A private WeakMap binds one live discovery to its downloads; copied/saved metadata,
retired and concurrently active selections cannot grant download authority. Clear,
completion, cancellation and project/settings changes retire that selection. Open
also clears an unsubmitted token. There is no claim of secure heap erasure or
protection from privileged same-origin scripts/browser extensions. A user who
chooses to provide a token trusts this running application with its in-memory use.

Native sources retain the existing 64 MiB combined/64 MiB binary/16 MiB companion
bounds; metadata pages are capped at 2 MiB and requests have bounded deadlines.
Cancellation and changed source scope preserve previous project inputs. No native
completion marker, folder permission, automatic execution or installation is
inferred from successful download. The receipt is a local observation, not source
or publisher authentication, a legal certificate, or a stored execution grant.

Pickle, request JSON and display bodies are not downloaded into this input path.
Custom county companions are also excluded: native Census inputs use canonical
GEOID county prefixes. This corrects the prior grouped import's acceptance of a
county companion that native tract preparation would reject. The local importer
now leaves that file unread. Original source bytes and old packages remain retained.
Native gh's full wildcard artifact collection, completion markers and filesystem/
force/authentication lifecycle remain separate unresolved contracts.

## Packaged and live evidence — 2026-10-26

`target/adjacency-release/package-1793028220992456400/evidence.json` passes 14
stages across the unchanged default and C-enabled engine profiles: offline creation,
asset/offline closure, existing local import/native-host preparation, actual browser
local import, release-adapter boundaries and release-browser journeys. All source
and package identities are checked before/after. Both shells pass pagination,
digest/length failures, cancellation preserving sources, no implicit execution,
credential-free exports and inert Open. The private-header redirect test uses
actual browser network interception (CDP); the earlier Playwright-only redirect
mock did not intercept the follow-up host and failed. No CSP or production guard
was weakened to make that harness pass.

`target/adjacency-release/live-1793028309453/evidence.json` exercises the actual
private `data-inputs-v1` release through existing native gh authentication. The
credential remains inside gh. The adapter's observed binary/GEOID bytes match the
actual native CLI `fetch --release` output exactly and match published SHA-256.
The native CLI test uses fresh owned data/output roots and preserves its opaque
pickle artifact and completion marker without executing pickle.

The real RI 2020 release has 250 units /population 1,097,379. Its v2 binary lacks
physical geometry attributes. Both engines preserve the native default-preparation
refusal. A separate explicit supplement comes from the hash-checked retained RI
graph, matching every GEOID and per-unit population before ordering areas,
perimeters and WGS84 centroids. It is not a display-geometry substitution or fresh
Census derivation. Both WASM/native hosts match complete successful preparation
and population-build receipts. Requested/accepted seed 42 yields two connected
districts (129 and 121 units), populations 548,703 and 548,676, and independently
computed maximum deviation 0.0024604079356357286 percent. Both builds use
`metis-core-rust`; the C-enabled shell does not imply C-backend execution here.

The authenticated-native transport remains separate from browser acceptance.
With explicit user authorization, an ephemeral Chromium context used the existing
gh login in memory against the real private release. Metadata succeeded (HTTP 200).
The original candidate's forwarded API-version header caused an asset-host OPTIONS
405. Binary requests now omit that header; metadata requests retain it. The final
candidate reaches the actual asset GET with no Authorization header forwarded,
but Chromium refuses the response with `MissingAllowOriginHeader`.

The sanitized final failure record is
`target/adjacency-release/private-browser-1793028279756/evidence.json`.
The earlier 405 and intermediate GET failures are retained in
`private-browser-1793028053747` and `private-browser-1793028259634`.
No CORS bypass, mocked success, persistent browser profile, raw exception, token,
or signed redirect URL was saved. HTTP caching was disabled and the context closed.
A private browser acquisition success is **not supported by this provider path**;
use native gh download followed by grouped local import. A future server-side
bridge requires its own credential and access-control contract. The existing
public source relay was not widened. Hosted-origin verification also remains open.

All 14 final package stages pass with unchanged engine binaries. The first v4
validation run completed its checks but its source freeze failed because the unit
regression was edited during the run; it is not accepted. The v5 run repeated all
checks with stable source identities and is the accepted package record above.
Other states/releases, full native gh lifecycle, host/API and acceptance-v1 remain
open. No public deployment is accepted by this document.

## Retained scope

The independently verified content-addressed snapshot is
`target/adjacency-release/snapshot-scope/snapshot.json`, SHA-256
`30cd99a44625963aa053cca6b5a3a2c5f9cb6caa67b7520289af5a0bbf80285e`:
1,569 mappings /526 unique identities /540,370,022 logical bytes. It retains the
final package, real native provider proof and sanitized browser failures separately.
This post-freeze pointer does not change the frozen contract or its failed provider
gate. No portable secret-bearing project was generated by the failed live check.
