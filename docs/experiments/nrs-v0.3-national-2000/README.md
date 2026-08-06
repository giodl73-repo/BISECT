# NRS v0.3 National 2000 Result

## Outcome

The governed NRS v0.3 Census 2000 batch and a separate
`verify-nrs-batch --require-complete` invocation both passed with **50 verified
States and zero failures**.

- Census cycle: 2000
- States: 50
- Census blocks: 8,199,908
- Population: 280,849,847
- House districts: 435
- Recursive split nodes: 385
- Recorded State elapsed time: 2,667.114305 seconds
- Ledger SHA-256: `1b2518ba728585fece6accdfab427c3b6d550a00588396a62130de522a09b1e1`
- Standard-profile canonical SHA-256: `73215169d786ea3efcbb9f217b811436dd0b5ae451078185f642c8226c4979d9`
- Legal-profile canonical SHA-256: `b610346fce43f78eceacdc73334a0d88ea98d3c8b98a29f3f62ca5ff477b33d4`
- `bisect.exe` SHA-256: `2bcf6b13f17f237db6f755943ea1ccdac0d2e0267395c616892c6e46ce66e90e`

All 435 district assignments have complete, nonduplicated block coverage. All
districts and recursive children passed independent connectivity verification,
and all 385 recursive nodes satisfy the frozen population tolerance.

## Hawaii amendment

NRS v0.3 preserves the v0.2 seed stream and runs the v0.1 and v0.2 candidate
stages unchanged. Its bridge-aware land-component fallback activates only
after v0.2 still misses tolerance. On Hawaii's root, the retained v0.2 result
had scaled deviation `802,861` against an allowed `6,058`; v0.3 achieved `41`
and passed the full-graph connectivity gate.

## Proof coverage

- Population tolerance: 385/385 nodes verified
- Population arithmetic floor: 2/385 nodes proved
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
