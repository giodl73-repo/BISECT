# Explicit private release download bridge

The release controls offer an unchecked **Use this site’s download bridge (sends
the optional token to this site)** option. Selecting it changes only binary asset
transport. Metadata discovery still talks to GitHub directly. Download, prepare
and Run remain separate explicit actions. The bridge works with this site's Worker;
a purely static host retains direct transport and grouped local import.

The token travels over the selected site's HTTPS connection, then only to the
GitHub API. The hosting operator is trusted with it. It is not a browser-only secret
in this mode. The implementation has no credential database, shared server token,
credential logging, cookie forwarding, receipt field or saved-project field.
Site scripts, extensions, a compromised host and platform logging are outside those
guarantees. Operators must keep request headers/bodies out of logs and use narrowly
scoped, short-lived read credentials. Local integration tests use loopback HTTP.

`POST /api/adjacency-release` requires the exact Origin, the explicit v1 protocol
header, JSON content type and a body of at most 2 KiB. The body contains only
repository, tag, state, year and asset ID; tokens belong only in Authorization.
The Worker accepts only the configured REDIST/data-inputs-v1 repository/tag,
re-discovers all bounded release pages and checks the asset against the selected
native-input filenames. A URL, filename or saved receipt cannot grant a download.

The server follows at most three asset redirects manually to the recognized HTTPS
GitHub asset hosts. It constructs headers afresh, dropping Authorization after the
API hop. Signed URLs never return to the browser. Provider cookies, redirects and
CORS grants are not forwarded. Responses use no-store and an explicit bridge
marker. Published size bounds the stream; oversized/truncated bodies fail, and
disconnect, cancellation and deadline propagate upstream. The client independently
checks original length and published SHA-256 before atomically replacing inputs.
The public Census/LODES/EIA relay's allowlist and credential policy are unchanged.

## Verification

The dual-profile package record is
`target/adjacency-release/package-1793028682152983900/evidence.json`: 18 passing
stages, including both existing browser/import journeys, bridge boundary tests and
public-relay regressions. Engine bytes retain the previously verified identities.
Additional streaming fault checks passed at
`target/adjacency-release/bridge-streams-1793028879761/evidence.json`.

The actual private GitHub provider was exercised in ephemeral, cache-disabled
Chromium with the explicitly authorized existing gh login:

- Default: `target/adjacency-release/private-browser-1793028820234/evidence.json`.
- C-enabled shell: `target/adjacency-release/private-browser-1793028830172/evidence.json`.

Both exact sealed packages downloaded the original RI 2020 binary and GEOIDs,
verified published digests, exported a project and receipt without the credential
or signed URL, and successfully prepared the graph with explicitly selected
matching geometry/centroids. This uses the Node-hosted production Worker handler
and actual GitHub transport, not mocked provider responses or a CORS bypass.
Source identities were checked before/after. Each context was closed and the UI
credential cleared. No secure heap-erasure claim is made.

The original direct-browser CORS failure remains reproducible and retained in
the [direct acquisition contract](wasm-adjacency-release-acquisition.md). The bridge
does not establish publisher authenticity, missing physical attributes, fresh
Census derivation, native disk completion markers, all releases/states or C-backend
partition execution. Full parent/acceptance-v1 remains open.

Cloudflare configuration routes the endpoint to the Worker, but this change has
not been deployed. Intended-host browser success, runtime limits, operational rate
controls and platform log settings require separate verification before hosted
private-download support is accepted. The maintainer owns that release decision.

## Retained scope

`target/adjacency-release/bridge-snapshot-scope/snapshot.json`, SHA-256
`ce2414da0bf9b0b29fbb5feffc5b5f67e3ee249bdaaa5201b24d63cff73179db`,
fully verifies 1,549 mappings /514 identities /513,411,461 logical bytes. It retains
both real browser proofs, the staged packages, stream fault evidence and the
pre-closure contract/review. This post-freeze pointer does not accept hosted support.
