# Accepted evidence must survive an evidence-free source tree

Status: MITIGATED in the tested source candidate; complete release acceptance pending.

Verified child records that reference ignored target files can block a fresh
source tree before its normal fixture/runtime tests start. Preserve a declared,
content-addressed capsule and a source-owned hash lock for historical evidence;
keep small fixture regeneration and mandatory current ledger checks in the build.
The restored namespace is specific and every file remains data until a separate,
explicit source-owned operation uses it.

`parity_evidence_archive.py` bounds ZIP structure before index allocation,
validates the lock and manifest, checks each payload hash before staging, and
rehashes restored files before publication. Import refuses existing namespaces,
linked parents, path escapes, duplicates/case aliases and unsupported entries.
The 23-case suite exercises actual Windows junction refusal and simulated IO
publication/diagnostic failures. Preserve primary errors when failure receipts
cannot be written; report a successful publication independently from receipt IO.

The actual 327-file, 97,038,379-byte set round-trips through a 26,954,165-byte
capsule. An evidence-free 13,791-file source projection accepts the restored
bytes and validates all 313 records while the full-parent gate still refuses.
All 66 partial and 247 not-run parents remain unchanged. No archived file runs.

Keep the distinction between trusted-lock integrity and publisher authenticity,
between source projection and clean Git checkout, and between historical evidence
restoration and regenerated native/browser results. The capsule is unpublished;
actual clean checkout, full amended builds and original acceptance remain open.
