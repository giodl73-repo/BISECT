# V.3 Tamper-Evident Hashing: Review Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Scores

| Role | Score |
|---|---:|
| VAULT | 3 / 4 |
| LEDGER | 2 / 4 |
| BENCHMARK | 3 / 4 |
| TALLY | 3 / 4 |
| CANVASS | 3 / 4 |

Average: 2.8 / 4. Minimum: 2 / 4.

## P1 Blocking Items

### P1.1 Add a threat model table

The paper should distinguish byte tampering, source omission, parser
substitution, package truncation, hash collision, source completeness, malware
resistance, ballot secrecy, and legal certification.

### P1.2 Define source-index and canonical-hash contracts

LEDGER needs package-relative source paths, raw byte hashes, normalized
canonical JSON hashes, domain-separated prefixes, algorithm/version language,
and volatile metadata exclusions.

### P1.3 Add fixture expected-result traceability

BENCHMARK wants a compact table for positive and negative fixtures plus a second
negative excerpt for `missing-source-hash`.

### P1.4 Clarify source custody and batch source refs

CANVASS and TALLY need clearer language that source refs bind declared evidence
but do not prove ballot interpretation, public-record sufficiency, or legal
certification.

## P2 Important Improvements

- Add a compact hash layer diagram.
- Mention detached signatures and custody attestations as future work.
- Say V.5/V.6 handle ballot manifest and CVR reconciliation in depth.

## Recommended Next Action

Revise V.3 before starting V.4. The paper has the correct security posture, but
it needs a more explicit format contract to become a ready anchor.
