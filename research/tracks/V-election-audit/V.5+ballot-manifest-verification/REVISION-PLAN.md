# Revision Plan

Round 1 simulated review is complete.

## P1 Items

1. [x] Define manifest rows as accounting controls and separate ballots, ballot
   cards, CVRs, contests, selections, voters, and batches.
2. [x] Add a batch lifecycle/status model covering late mail, provisional,
   duplicated-ballot, central-count, vote-center, and recount-driven changes.
3. [x] Add exact fixture file-path and transcript traceability for
   `mail-batch-added` and `missing-batch`.
4. [x] Define the canonical path/source-ref contract for normalized batch
   manifests.

## P2 Items

- [x] Add a compact manifest-to-summary-to-jurisdiction diagram.
- [x] Add a privacy table for small batches and rare batch metadata.
- [x] Point forward to V.6 for CVR-to-summary reconciliation and V.7 for RLAs.
