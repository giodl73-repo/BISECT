# Restoring retained child-proof evidence

The current local archive set contains 2,525 files for 33 verified child scopes,
totaling 1,070,597,221 restored bytes and 595,775,980 archive bytes across seven
ZIPs. Two small shards were merged without changing any of the 2,439 prior
files; five ZIPs remain byte-identical. The new shard adds 86 files for the
standalone and before-Run State target actions. Both the merged
shard and the new shard were restored into empty roots and checked byte for
byte, and the complete seven-shard set was verified.
See the [parity plan](wasm-desktop-parity-plan.md)
for the individual proof scopes and earlier full-set restore results.
Obtain the current lock from the trusted source checkout. Publishing the asset
and checking an actual clean Git checkout remain pending.

Restore it from a directory containing the seven named ZIPs:

```powershell
python scripts/web/parity_evidence_archive.py import --root . --archive C:/path/to/current-child-evidence-shards --lock docs/specs/wasm-parity-child-evidence.lock.json
python scripts/web/wasm_parity_ledger.py
```

`verify` checks the archive without restoring it. `ensure` supports build
preflight: an existing proof directory continues to the mandatory ledger checks;
an absent directory requires the explicit local
`BISECT_PARITY_CHILD_EVIDENCE_ARCHIVE` environment variable or `--archive`.
The tool performs no network requests and reports zero executed files. Existing
proof directories are preserved; use a fresh worktree to restore another capsule.

The trusted lock binds the archive and manifest SHA-256, encoded length, file
count and decoded byte total. Import checks ZIP structure and bounded central
directory records before allocating the ZIP index. It then verifies every object
before writing a staging directory and checks every restored file again before
publishing the new proof namespace. Paths stay under the declared namespace;
duplicates, case aliases, traversal, reserved Windows names, linked parents,
special ZIP entries and encryption are refused. Encoded size is bounded at
512 MiB, decoded files at 256 MiB total and 128 MiB each, count at 4096 and
central-directory/manifest data at 4 MiB. A Windows sharing or access error
during the final rename is retried briefly against the same staging directory.
If the destination appears or the staging directory changes, publication stops.
File IO failures retain staging; failure receipts are saved where storage permits.
A diagnostic-write failure after
successful publication is reported separately from restored proof bytes.
The repository directories must remain under caller control
during import; this is not an adversarial local-filesystem transaction guarantee.

A ledger `build.wasm` reference may point to raw WebAssembly or to a single
XZ stream. For XZ, the outer `sha256` identifies the stored file and
`encoding` records `codec: "xz"`, `decoded_sha256`, and
`decoded_byte_length`. The ledger checks both identities, the decoded WASM
header, and the same metadata for every declared additional profile. It
limits the encoded size, decoded size and decoder memory to 64 MiB each,
rejects trailing bytes, and never loads the module for execution. This
representation leaves the archive's existing size limits in place.

Small fixtures still regenerate through the source-owned native/WASM/browser
gates. Restoring historical evidence supplies the declared bytes required by the
ledger; those bytes establish neither a fresh compiler run nor source publisher
authentication, complete parent acceptance or execution authority for imported
projects. The complete amended builders and original WP-01–WP-10 remain required.

Maintainer export reads only the independently declared transfer destinations
inside the selected repository. It hashes source bytes again during ZIP writing,
uses content-addressed objects and refuses existing archive/lock outputs:

```powershell
python scripts/web/parity_evidence_archive.py export --root . --transfer C:/path/to/reviewed-transfer.json --archive C:/path/to/current-child-evidence.zip --lock C:/path/to/new-lock.json
python scripts/web/test_parity_evidence_archive.py
```

The sidecar lock identifies this exported capsule's exact bytes. ZIP compression
output can vary by toolchain; retain the declared capsule rather than assuming a
later re-export has the same archive hash. A new scope or evidence change requires
a newly reviewed capsule and lock.

For larger evidence collections, a trusted set lock can bind two to eight ZIP
shards. Each shard retains the single-archive limits above. A set is limited to
16,384 restored files and 2 GiB of restored bytes. Restore checks every shard
and the combined destination tree before staging any files. It refuses
cross-shard collisions, case aliases and file/directory conflicts. One final
rename publishes the complete set; a bad shard leaves the destination absent.
The set receipt binds the trusted set lock hash and records zero executed files.

Pass the shard directory as `--archive`, or use that directory as the value of
`BISECT_PARITY_CHILD_EVIDENCE_ARCHIVE` for `ensure`. To compose a reviewed set:

```powershell
python scripts/web/parity_evidence_archive.py compose-set --archive C:/path/to/shards --lock C:/path/to/set.lock --part base.zip=C:/path/to/base.lock --part map.zip=C:/path/to/map.lock
python scripts/web/parity_evidence_archive.py import --root . --archive C:/path/to/shards --lock C:/path/to/set.lock
```

Read the ZIP names from the trusted lock rather than reusing a list from a
previous release. To merge only selected small shards, pass their names as
repeated `--merge-shard` arguments to `repack_parity_evidence.py`. It uses the
already restored files to build the replacement shard, hard-links unchanged
ZIPs into a new directory on the same volume, and verifies that every path,
digest, and byte length in the new set matches the original. Keep the original
set and lock until the new set is verified. These are local validation
artifacts; the hosted workbench does not serve them to visitors.
