---
skill: simulate-contract
topic: U.21-certified-recursive-bisection
date: 2026-09-09
gate_result: PASS
---

# U.21 specification-to-evidence contract

Contract sources: `plan.md`, the certified-recursive-bisection specification,
the sequential-ceremony specification, and the prospective ceremony bakeoff
extension. Actual outputs: Rust tests, the eight-instance comparison package,
the path-8 ceremony package, the RI frontier, and published national packages.

| Contract element | Required behavior | Actual evidence | Gate |
|---|---|---|---|
| Fixed recursive schedule | Solver cannot alter floor/ceiling seat tree | Model and verifier bind seat counts and breadth-first paths | PASS |
| Unique model-scoped cut | Population, boundary, then canonical tie-break | Bounded oracle and proof-request pipeline implement the ordering | PASS, bounded proof |
| Parent-child binding | Child inputs reconstructed, not trusted | Hostile child-substitution tests reject mismatches | PASS |
| Complete plan binding | Exactly `k-1` cuts and one-seat leaf coverage | Tree verifier and path-8 completed transcript | PASS |
| Immutable staged release | Genesis, timed depth rounds, halt finality | CLI lifecycle and negative tests | PASS |
| Prefix verification | No future descendants required | Open-prefix test and fixture verifier | PASS |
| External publication | Named witnesses validate clocks/signatures | Only opaque receipt uniqueness is implemented | NOT CLAIMED / OPEN |
| State-scale exact objective | Independent boundary and canonical UNSAT proofs | RI population lower bound only; boundary timed out | NOT CLAIMED / OPEN |
| Bounded comparison | Retain every frozen graph-seed row | Independent verifier passes 8 graphs and 40 rows | PASS |
| Procedural bakeoff | Separate process scoreboard; unsupported arms retained | Certified arm 3/3; METIS replay arm retained as unsupported | PASS, synthetic |
| National operations | Complete assignments and tolerance verification | Clean-worktree replay regenerates all 150 State-cycle assignments and matches every governed hash | PASS, full replay |
| National bakeoff regeneration | Rebuild Tier 1--2 from bound source assignments | Restored vault-backed assignments and TIGER inputs pass both national verifiers | PASS, external corpus required |
| No composite winner | Do not blend map and process scores | Manuscript and protocol keep two scoreboards | PASS |
| Claim boundary | No fairness, VRA, legal, witness, or superiority theorem | Explicitly disclaimed in abstract, claims, and conclusion | PASS |

The paper delivers its narrowed methods contract. National clean-worktree
replay and Tier 1--2 regeneration now pass; the manuscript states that the
large source corpus remains outside Git. The generic
`GateTokenSchema` fields in the skill template are not part of this paper's
declared redistricting contract and are therefore outside this component gate.
