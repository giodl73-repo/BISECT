# Revision Plan

Round 1 simulated review is complete.

## P1 Items

1. Add a threat model table separating tamper evidence from malware resistance,
   secrecy, certification, source completeness, parser correctness, and
   end-to-end voting.
2. Define the source-index and canonical-hash contract more explicitly:
   package-relative paths, raw source bytes, normalized canonical JSON, and
   domain-separated prefixes.
3. Add fixture-to-expected-result traceability for `summary-basic`,
   `mail-batch-added`, `tampered-source`, and `missing-source-hash`.
4. Clarify batch-hash and source-reference boundaries for TALLY/CANVASS readers.

## P2 Items

- Add a compact hash layer diagram.
- Add a source-index row sketch.
- Add one note about detached signatures and custody attestations as future
  work.
