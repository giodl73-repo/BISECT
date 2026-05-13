# V.4 Round 2 Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Status

Ready.

## Resolved P1 Items

- Added a coercion and receipt threat-model table covering direct choice
  receipts, public identifiers, ballot-style/timestamp linkage, small-cell
  disclosure, and deterministic token linkage.
- Expanded the public proof contract with allowed fields, forbidden fields,
  package-local identifier scope, token-hash caveats, and the limit of
  `proof_privacy_gate`.
- Added fixture-to-transcript traceability for `privacy-inclusion-sketch` and
  `choice-bearing-proof`, including expected transcript status, check ids, and
  the negative error.
- Clarified accepted-token and counted-token semantics, voter-facing language,
  canvass boundaries, and the relationship to future ballot manifest and CVR
  papers.

## Remaining Risk

V.4 is intentionally not a deployable cryptographic voter-verification protocol.
It does not specify token issuance, salts, nonces, key custody, signatures,
mixing, or formal coercion resistance. Those are correctly named as future
protocol work.
