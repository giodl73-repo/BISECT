# NRS v0.3 National 2010 Result

## Outcome

The governed NRS v0.3 Census 2010 batch and a separate
`verify-nrs-batch --require-complete` invocation both passed with **50 verified
States and zero failures**.

- Census cycle: 2010
- States: 50
- Census blocks: 11,071,790
- Population: 308,143,815
- House districts: 435
- Recursive split nodes: 385
- Recorded State elapsed time: 3,801.288431 seconds
- Ledger SHA-256: `cd72864119273f3fa4e431906d8683a23386f7f48baab45202c3fd4e3870e61e`
- Standard-profile canonical SHA-256: `b06e48a925f2fe0deaea6fc3eb3331944fa8a1bcc35aab58fe6e64b399228a2d`
- Legal-profile canonical SHA-256: `742b460e8150bcedb94276a98e467a6528d7f3af6fb2429d9ff8716e3a202bae`
- `bisect.exe` SHA-256: `2bcf6b13f17f237db6f755943ea1ccdac0d2e0267395c616892c6e46ce66e90e`

All 435 district assignments have complete, nonduplicated block coverage. All
districts and recursive children passed independent connectivity verification,
and all 385 recursive nodes satisfy the frozen population tolerance.

## California amendment

NRS v0.3 preserves the v0.2 seed stream and runs the v0.1 and v0.2 candidate
stages unchanged. The bridge-aware v0.3 fallback is available only after a
v0.2 tolerance miss. This complete run therefore retains the governed staged
procedure that resolved the published California v0.1 tolerance witness while
keeping predecessor-compatible seed derivation.

## Proof coverage

- Population tolerance: 385/385 nodes verified
- Population arithmetic floor: 7/385 nodes proved
- Weighted-boundary optimum: 0/385 nodes proved (`not-run`)
- Canonical optimum: 0/385 nodes proved (`blocked-by-boundary`)

Tolerance conformance is not promoted to global objective optimality.

## Published artifacts

- `national-summary.json`: complete State and aggregate verification results
- `proof-coverage.json`: separate population, boundary, and canonical coverage
- `manifest.json`: transport hashes and publication claim boundary

## Claim boundary

This is a candidate reference geographic baseline. It does not establish
legal validity, VRA compliance, partisan fairness, global boundary or
canonical optimality, or official adoption.
