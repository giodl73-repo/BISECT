# V.21 SOBA And Observable Ballot-Level Audits

## Mental Model

SOBA is about making ballot-level audits publicly observable while preserving
ballot secrecy. It focuses on commitments, ballot identifiers, CVR linkage, and
observer-verifiable evidence rather than only the stopping math.

For RCOUNT, SOBA is the privacy and observability counterpart to ballot-level
comparison audits.

## How RCOUNT Uses It

```text
ballot commitments -> CVR linkage -> sampled ballot observations -> public audit evidence
```

RCOUNT should use this family when a package includes cryptographic commitments,
public ballot-level records, or privacy-preserving linkage claims.

## Step-By-Step Mechanics

1. Publish or ingest committed ballot/CVR records.
2. Preserve the mapping commitment without exposing voter-identifying order.
3. Replay public sample selection.
4. Verify that sampled ballot identifiers open correctly.
5. Compare public CVR claims to human observations.
6. Bind the audit transcript to the commitments and source hashes.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `soba-observable-ballot-audit-v1` |
| `commitment_scheme` | hash/commitment construction |
| `ballot_commitment` | public commitment for sampled ballot |
| `opening_status` | opened, missing, malformed |
| `privacy_boundary` | what remains hidden |
| `cvr_claim` | committed electronic interpretation |
| `human_observation` | audit-board interpretation |
| `comparison_status` | match, discrepancy, unresolved |

## Fixtures

- Commitment/opening toy fixture backed by an anonymized inclusion proof.
- Missing-opening negative fixture.
- CVR mismatch fixture that feeds V.14 comparison math.
- Privacy-boundary fixture showing what RCOUNT must not expose.

## Current Implementation

RCOUNT can preserve a SOBA-style observable-ballot boundary run:

- `soba-observable-ballot-audit-v1` uses
  `ObservableBallotLinkage` assertions;
- `AuditSampleStep.sample_unit_id` references a public anonymized
  `InclusionProof.proof_id`;
- the verifier requires every sampled opening to exist;
- linked proofs must remain privacy-safe, with no choice-bearing proof payload
  and no voter id;
- non-SOBA methods are rejected if they carry observable-ballot linkage
  assertions.

`rcount-audit` reports this method as a replay boundary: commitment/opening
linkage is recorded, but comparison-risk replay remains outside this slice.

## Claim Boundary

SOBA-style evidence improves public observability and linkage, but it still
depends on the physical audit trail and ballot-custody process. RCOUNT should
verify commitments and transcripts, not infer voter identity or publish secret
ballot order.

## References

- SOBA paper: <https://arxiv.org/abs/1105.5803>
- SOBA USENIX page: <https://www.usenix.org/conference/evtwote-11/soba-secrecy-preserving-observable-ballot-level-audit>
