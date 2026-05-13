# V.19 Plan

## Thesis

RCOUNT cannot fully cover modern audit practice without ranked-choice audit
support. RAIRE and AWAIRE are the target families for IRV/RCV outcome audits.

## Atlas

- `docs/algorithm-atlas/v19-raire-awaire-rcv.md`

## Implementation Tasks

- [x] Add ranked-choice audit-run surface with reported IRV elimination order
  and ranked sample choices.
- [x] Add tiny boundary fixtures for RAIRE and AWAIRE method identity.
- [x] Add malformed ranked-ballot negative fixture.
- [ ] Add ranked-choice contest and ranked CVR fixture support.
- [ ] Add tiny IRV tabulation fixture with elimination order.
- [ ] Add RAIRE assertion-generation fixture.
- [ ] Add assertion-level risk replay transcript.
- [ ] Defer real jurisdiction adapter until ranked CVR semantics are stable.

## Claim Boundary

RAIRE/AWAIRE audit an RCV outcome under a reported rule and ballot record.
RCOUNT now preserves ranked-choice audit evidence and reports both methods as
explicit boundaries, but it does not yet generate or replay RAIRE/AWAIRE
assertions.
