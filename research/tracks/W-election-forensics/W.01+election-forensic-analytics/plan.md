# W.01 Plan

## Thesis

RCOUNT packages provide clean, hash-bound inputs for forensic analytics. Those
analytics can highlight anomalies for investigation, but they must remain
separate from certification evidence.

## Atlas

- `docs/algorithm-atlas/w01-election-forensic-analytics.md`

## Implementation Tasks

- [ ] Define forensic report schema with source package hash.
- [ ] Add no-anomaly synthetic fixture.
- [ ] Add injected outlier fixture.
- [ ] Add scanner/batch-effect fixture.
- [ ] Add false-positive boundary fixture and report language.
- [ ] Decide whether implementation belongs in `rcount-audit` as reports or a
  later `rcount-forensics` crate.

## Claim Boundary

Forensic analytics generate investigation queues, not proof of fraud and not
certification pass/fail results.
