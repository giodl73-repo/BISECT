# Track V -- Vote Counting, Certification, And Public Verification

Track V studies how election totals become public facts.

The track is built around RCOUNT, the proposed reproducible election-count and
canvass-audit substrate. RCOUNT is to election counts what RPLAN is to district
plans: a neutral package format and verifier path that lets third parties
recompute public claims from lower-level evidence.

## Papers

| Code | Working title | Status |
|------|---------------|--------|
| V.0 | RCOUNT Overview: Reproducible Election Count Packages | planned |
| V.1 | Canvass Arithmetic: From Unofficial Returns To Certified Totals | planned |
| V.2 | Precinct Lineage Across Elections | planned |
| V.3 | Tamper-Evident Precinct And Batch Hashing | planned |
| V.4 | Privacy-Safe Voter Inclusion Proofs | planned |
| V.5 | Ballot Manifest Verification | planned |
| V.6 | CVR-To-Summary Reconciliation | planned |
| V.7 | Replayable Risk-Limiting Audits | planned |
| V.8 | District Vote Aggregation With RPLAN | planned |
| V.9 | Count-System Interoperability: Vendor Exports To RCOUNT | planned |
| V.10 | Certification Evidence Matrix | planned |
| V.11 | Performance And Parallel Verification In Rust | planned |

## Track Contract

Each paper should separate:

- what election officials must certify under law;
- what current systems and public exports actually provide;
- what RCOUNT can verify mechanically;
- what remains a human, statutory, judicial, recount, or audit-board decision.

The voter-facing rule is strict: prove inclusion, not candidate choices.
