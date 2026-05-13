# V.3 Plan: Tamper-Evident Precinct And Batch Hashing

## Question

How can RCOUNT make public count packages tamper-evident without pretending
that hashes alone certify elections, protect ballot secrecy, or prove tabulator
correctness?

## Claims

1. Source hashes bind raw public inputs to normalized package records.
2. Package/content hashes bind the normalized RCOUNT claim set.
3. Batch manifests add a smaller accounting surface for late mail,
   provisional, election-day, and central-count evidence.
4. Source hash failures are distinct from arithmetic failures and must be
   reported separately.

## Evidence

- `summary-basic` package hash metadata.
- `mail-batch-added` batch manifest fixture.
- `tampered-source` negative fixture.
- `missing-source-hash` negative fixture.
