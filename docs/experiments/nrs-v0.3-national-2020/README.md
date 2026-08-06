# NRS v0.3 National 2020 Result

## Outcome

The governed NRS v0.3 Census 2020 batch and a separate
`verify-nrs-batch --require-complete` invocation both passed with **50 verified
States and zero failures**.

- Census cycle: 2020
- States: 50
- Census blocks: 8,126,956
- Population: 330,759,736
- House districts: 435
- Recursive split nodes: 385
- Recorded State elapsed time: 3,165.878260 seconds
- Ledger SHA-256: `d589d02a45e4c4a99817ecd6810d8836937ff45633200d76f96c4809c9aa0ca8`
- Standard-profile canonical SHA-256: `ca0f5649bce9631f34933d5d87cfd989843481172f48796c039a68293812fea9`
- Legal-profile canonical SHA-256: `ae372b369aa3532eaa69d6a082cdd47c716b05bca932b4ffcf9e05242e855e40`
- `bisect.exe` SHA-256: `2bcf6b13f17f237db6f755943ea1ccdac0d2e0267395c616892c6e46ce66e90e`

All 435 district assignments have complete, nonduplicated block coverage. All
districts and recursive children passed independent connectivity verification,
and all 385 recursive nodes satisfy the frozen population tolerance.

## Profile compatibility

NRS v0.3 preserves the predecessor seed stream and runs the v0.1 and v0.2
candidate stages unchanged before making the bridge-aware v0.3 fallback
available. The published 2020 inventory remains byte-for-byte unchanged and
hash-bound; the operational verifier accepts its missing explicit year only
under the exact legacy schema `certified-national-2020-inventory-v1`.

## Proof coverage

- Population tolerance: 385/385 nodes verified
- Population arithmetic floor: 4/385 nodes proved
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
