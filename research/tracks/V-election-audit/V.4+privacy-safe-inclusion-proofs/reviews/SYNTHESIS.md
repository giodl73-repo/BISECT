# V.4 Privacy-Safe Inclusion Proofs: Review Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Scores

| Role | Score |
|---|---:|
| VAULT | 2 / 4 |
| COMMONS | 3 / 4 |
| BENCHMARK | 3 / 4 |
| CANVASS | 3 / 4 |
| TALLY | 2 / 4 |

Average: 2.6 / 4. Minimum: 2 / 4.

## P1 Blocking Items

### P1.1 Add a receipt/coercion threat model table

The paper should distinguish direct choice receipts, linkability receipts,
small-cell disclosure, deterministic token issuance, and ordinary inclusion
checking.

### P1.2 Define the public proof contract

The public proof contract should name allowed fields, forbidden fields,
package-local identifiers, token-hash caveats, and the fact that
`proof_privacy_gate` is a necessary baseline rather than a full protocol proof.

### P1.3 Add fixture expected-result traceability

The positive and negative fixtures should point to the proof NDJSON files,
transcripts, `proof_privacy_gate`, and the exact negative error.

### P1.4 Clarify voter-facing and canvass semantics

The paper must not imply "your vote was counted" or "your choices counted."
It should say that a package contains an anonymized accepted or counted token,
subject to legal status decisions and certification outside this proof.

## P2 Important Improvements

- Add a compact proof-surface diagram.
- Mention salts, nonces, key custody, and cryptographic protocols as future
  work.
- Point forward to V.5 and V.6 for ballot manifest and CVR reconciliation.

## Recommended Next Action

Revise V.4 before starting V.5. The fixture pair is a strong start, but the
paper needs a stronger privacy and accounting contract before it can be marked
ready.
