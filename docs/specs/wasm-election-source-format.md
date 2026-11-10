# Explicit raw election source adaptation

Election analysis now offers an explicit column selector: canonical CSV or
Fekrazad 2020. The latter maps `tract_GEOID`, `G20PREDBID`, `G20PRERTRU` to the
native analyzer's `geoid,dem_votes,rep_votes`. It left-pads 10-digit numeric tract
IDs to 11 digits and preserves trimmed vote text, including fractional allocations
and scientific notation. It never rounds, aggregates duplicates, reallocates
precinct votes, converts geography vintages or silently infers a format.

The shared native source tools expose a gap: `download_election_data.py` obtains
the raw archive, while `process_election_data.py` writes a different Parquet schema
with Biden/Trump fields. The native political analyzer reads canonical CSV fields.
This browser adapter bridges that schema boundary explicitly; it does not claim
to implement Python/Parquet processing or all registry downloaders.

The original CSV remains byte-exact in `files.electionAnalysisSource`. The optional
record `files.electionCsvFormat={format:"fekrazad-2020"}` is validated on project
Open. Absent settings retain canonical behavior. Older readers reject the unknown format field rather than silently analyzing raw columns as canonical. Opening never converts, analyzes
or restores execution authority. Format edits retire fresh analysis exports.
Conversion happens only for explicit analysis or explicit source-package download.

Standalone election analysis, combined analysis and queued State follow-up recipes
consume the same conversion helper. Both primary/secondary election selections
must still agree if supplied together. The source package contains the original
and canonical bytes, independent SHA-256 identities, mapping rules and row count;
it grants neither source authenticity nor execution authority. The source-package
format is currently an export, not an automatically trusted project import.

Quoted commas/newlines and doubled quotes are supported. Empty/duplicate headers,
wrong column names, malformed quoting/row widths, invalid IDs, duplicate IDs after
padding, missing/nonfinite/negative votes, and more than 100,000 records refuse.
The existing 3 MiB source/project and combined standalone-analysis input limits
remain. Native numerical and coverage checks remain authoritative. Other parties
are excluded from the two-party canonical CSV. Reallocated fractional votes are
historical measurements, not exact certified integer ballot totals.

## Verification and limits

`target/election-source-format/package-1793030387676235200/evidence.json` passes
13 stages: the project security/handler suite and six checks per unchanged engine
profile, covering offline/assets, independent CSV/native comparisons, actual
browser Save/Open/Run/source download and prior release-client browser regression.

The independent Python standard-library CSV reader matches canonical bytes exactly
for the synthetic multiline/fractional/leading-zero fixture. Each profile matches
complete native/WASM standalone and combined-analysis results. The unrelated extra
source unit is excluded by the native consumer. A synthetic plan without native
engine tolerance still receives the exact accepted-follow-up refusal; provenance
was not fabricated to make that fixture succeed. Request construction and queued
follow-up retention of the mapping are tested separately.

Each actual browser proves raw-byte and mapping retention, explicit canonical
request text, exact maximum-u64 bootstrap seed, source-package output, inert Open,
result retirement after format edits, and 390px layout. The handler-only project
VM required minimal DOM append/after adapters and explicit stubs for separately
browser-tested controls; its parser/security/read validators remain real.

Live Dataverse metadata probe
`target/election-source-format/provider-probe-1793030449321263000/evidence.json`
returned HTTP 403. The actual native downloader --list-files invocation also exits 1 with HTTP403, recorded at `target/election-source-format/native-provider-list-1793030767768248400/evidence.json`; this observed provider failure is shared with native acquisition. No actual provider election file was acquired. All new vote
fixtures are synthetic, derived from source column declarations rather than an
authenticated publication. Registry-wide acquisition, ZIP-member selection,
guestbook/token handling, real dataset/vintage validation, Parquet compatibility,
large national source handling and full parent/acceptance-v1 remain open. This
change is packaged locally and has not been deployed.

## Retained scope

`target/election-source-format/snapshot-scope/snapshot.json`, SHA-256
`7576fe2c0bf9d0daee69da217e3dacffe2afa495f139560c6c4401257cd70269`,
fully verifies 1,555 mappings /508 identities /514,604,422 logical bytes. It retains
both staged profiles, independent/native/browser proof, direct and native provider
failures, source preimages and the pre-closure contract/review. This post-freeze
pointer leaves full source/native/host and acceptance-v1 gaps open.
