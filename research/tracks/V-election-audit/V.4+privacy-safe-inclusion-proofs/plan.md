# V.4 Plan: Privacy-Safe Voter Inclusion Proofs

## Question

How can a public count package support inclusion-style verification without
creating vote-choice receipts or exposing small-cell voter facts?

## Claims

1. Inclusion proofs must prove presence of an accepted or counted token, not a
   candidate choice.
2. Any proof that carries candidate selections is a receipt risk and should fail
   the baseline privacy gate.
3. Token proofs must avoid direct voter identity, ballot style, and timestamp
   linkage in public records.
4. V.4 is a proof sketch, not a full cryptographic voting protocol.

## Evidence

- `privacy-inclusion-sketch` positive fixture.
- `choice-bearing-proof` negative fixture.
- `proof_privacy_gate` verifier transcript.
