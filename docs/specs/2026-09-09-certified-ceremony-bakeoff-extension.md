# Certified-Ceremony Bakeoff Extension

**Protocol ID:** `certified-ceremony-bakeoff-extension-v1`  
**Effective:** prospective runs frozen on or after 2026-09-09  
**Depends on:** `certified-bisection-ceremony-v1`

## Purpose

This extension adds procedural verifiability to BISECT bakeoffs. It does not
rewrite the frozen Wisconsin, national structure-family, or NRS/CD118 bakeoff
protocols and it does not retroactively award ceremony evidence to their
outputs. Future comparisons must report map outcomes and process evidence as
separate result families.

The motivating question is not only which method returns which map, but also:

> After the rule is frozen, how much discretion remains hidden inside map
> selection, and what can an observer verify before later cuts are disclosed?

## Precommitment

Before any compared output is inspected, publish and externally timestamp:

1. the parent bakeoff protocol and this extension;
2. the common input manifest and algorithm-arm definitions;
3. the certified rule profile, canonical district-count tree, review interval,
   witness policy, and challenge policy;
4. the State and seed schedule, timeout and failure treatment, and exclusions;
5. all map-quality, runtime, proof, and ceremony estimands; and
6. the rule for reporting unsupported, partial, halted, and completed arms.

An arm may not be dropped because it lacks certification or halts during the
ceremony. Missing evidence is a result.

## Two non-combinable scoreboards

### Map and computational outcomes

Retain the parent protocol's validity, population, boundary, subdivision,
geometry, assignment-overlap, runtime, timeout, and downstream outcome fields.
Intentional public-review delays are not included in solver or verifier runtime.

### Procedural evidence outcomes

For every State-arm cell report:

| Field | Meaning |
|---|---|
| `procedure_class` | `replay-only`, `audit-package`, `certified-tree`, or `certified-ceremony` |
| `internal_cut_count` | Required value `k - 1` for a completed binary district tree |
| `certified_cut_count` | Cuts with accepted model-scoped optimality evidence |
| `proof_coverage` | `certified_cut_count / internal_cut_count` |
| `released_round_count` | Accepted public depth rounds |
| `prefix_verification` | Whether the disclosed prefix verifies without unreleased descendants |
| `final_tree_binding` | Whether a completed transcript binds the submitted final tree |
| `witness_policy` | Required receipt count and external validation policy |
| `review_seconds` | Frozen minimum delay between genesis and rounds |
| `status` | `unsupported`, `open`, `halted`, or `completed` |
| `halt_reason` | Mandatory when status is `halted` |
| `transcript_bytes` | Size of the completed or last accepted transcript |
| `verification_seconds` | Independent prefix/final verification time |
| `tamper_suite` | Pass/fail for the frozen negative corpus |

The four procedure classes are evidence levels, not claims that the underlying
map is fair or legally valid. A method can produce attractive descriptive
metrics and weak process evidence, or strong process evidence and unattractive
metrics.

## Release schedule

The root cut is released only after one complete review interval following
genesis. Later releases occur by canonical tree depth. All eligible cuts at one
depth are released together and ordered by binary path. States may proceed in
parallel, but no State may release descendants before its prior depth has
completed the review interval.

Depth batching preserves a meaningful challenge window without imposing
`k - 1` serial calendar delays. The transcript still contains all `k - 1`
individual cut decisions.

## Independent verification

The verifier must:

1. verify the parent bakeoff package under its unchanged protocol;
2. verify every released ceremony prefix without access to future descendants;
3. verify certificate, parent-child derivation, path order, timestamps, receipt
   uniqueness, hash-chain, status, and immutable-version rules;
4. require the complete certified tree for final binding and leaf coverage;
5. run frozen transcript, certificate, timing, witness, and child-substitution
   attacks; and
6. retain all failures, timeouts, and halted transcripts.

External witness signatures and publication times must be checked by the named
transparency-log or timestamp adapters. BISECT receipt-string uniqueness alone
does not satisfy that production requirement.

## Reporting and decision rule

Report a vector of results; do not collapse map quality, compute cost, proof
coverage, and ceremony completion into one weighted score. Pareto dominance may
be reported only for fields frozen before execution. A method cannot be called
the overall winner merely because it completes a ceremony, and an uncertified
method cannot be described as equivalent in verifiability because it reproduces
the same assignment.

The extension passes when all scheduled cells are retained, parent-protocol
results regenerate, procedural fields validate, and claimed ceremony arms pass
the negative corpus. Deterministic outputs regenerate exactly; verifier timing
is compared only under a separately frozen environment and tolerance. It may establish
comparative execution finality and disclosure properties. It cannot establish
partisan neutrality, VRA compliance, constitutional validity, public
legitimacy, or a generally best map.

## Initial evidence posture

The ceremony schema, prefix verifier, immutable CLI transitions, review-window
checks, halt behavior, and bounded hostile tests are implemented. No historical
bakeoff has yet been rerun as an externally witnessed ceremony. Accordingly,
current papers may describe the protocol, implementation, and bounded fixture,
but must label externally witnessed comparative ceremony results as pending.

The bounded executable fixture at
`docs/examples/certified-ceremony-bakeoff/` demonstrates the analysis contract.
Its timestamps and receipts are test values, not externally witnessed evidence.
