# Neutral Algorithm-Family Bakeoff Protocol

**Protocol ID:** `neutral-algorithm-family-bakeoff-v1`

**Frozen:** 2026-08-09, before the governed comparison run

**Stage:** Wisconsin proof slice; national expansion is a separate decision

## Question

Holding the 2020 tract universe, adjacency graph, population source, edge-weight
signal, outer search mode, and requested initial seed fixed, how do four implemented
BISECT structure families differ in feasibility, realized assignment, and
graph-native boundary cost in Wisconsin's eight-district congressional problem?

This is a software-and-mechanics comparison. It is not a test of partisan
fairness, Voting Rights Act compliance, legal adequacy, or superiority for map
adoption.

## Frozen proof slice

| Field | Frozen value |
|---|---|
| State / chamber | Wisconsin / congressional |
| Census year | 2020 |
| Atomic units | Census tracts in the repository's Wisconsin 2020 adjacency package |
| Districts | 8 |
| Population source | total population |
| Balance tolerance | native default, recorded from each native manifest |
| Partition preset | `edge-weighted` |
| Weight override | `geographic` |
| Outer search | `single` |
| Requested initial seed | `0` |
| Structures | `standard-bisect`, `ratio-optimal`, `ratio-optimal-area`, `prime-factor` |
| Runner | repository `target/release/bisect.exe`, SHA-256 bound in the package |

The four structures are intentionally not described as one-factor treatment
levels. `ratio-optimal-area` also activates its documented area constraint, and
the ratio- and prime-factor families may perform structure-specific internal
seed search. These behaviors are part of the implemented families being tested.

## Deterministic retry rule

The CLI may retry successive seeds when the requested seed fails its population
balance check. The requested seed and the final accepted seed must both be
reported. A retry is not silently relabeled as a seed-0 result. The native retry
behavior is held fixed rather than reimplemented by the wrapper.

## Outcomes

For every structure, preserve either a successful native package or a structured
failure record. A successful row reports:

1. requested and final seed;
2. native audit result and population-balance flag;
3. tract count and realized district count;
4. native weighted edge cut;
5. SHA-256 of the canonical final assignment;
6. pairwise assignment agreement after maximum-overlap district-label matching.

The aggregate package also reports whether all successful rows bind the same
adjacency hash, tract count, district count, binary hash, population source,
weight parameters, and balance tolerance. Wall-clock timings may be retained as
diagnostics but are not comparative estimands because the proof slice does not
control machine load or warm-cache effects.

## Validation and regeneration

The governed package must bind:

- this protocol and the runner/verifier source hashes;
- the exact command for each structure;
- the BISECT binary hash;
- every native manifest and final-assignment hash;
- deterministic derived JSON/CSV/README output hashes.

Verification must independently check hashes and invariants, rerun the four
commands in a temporary directory, normalize assignments by tract index and
district labels, and require byte-identical deterministic derived outputs.
Timestamp-bearing native artifacts are hash-bound in the published package but
are compared semantically during regeneration.
Native weighted edge cuts are normalized to six decimal places in deterministic
derived outputs because parallel floating-point reduction can vary below
one-billionth even when canonical assignments are byte-identical.

## Decision rule

The Wisconsin proof slice passes when all four invocations finish successfully,
all native audits and population-balance checks pass, all common-input invariants
hold, and exact normalized regeneration passes. Any failed structure remains in
the result as a failure; it is not dropped or replaced after inspection.

National expansion is authorized only after the proof slice passes and its
results are documented. Expansion requires a new frozen state schedule and may
not reuse this proof slice to claim national performance.

## Claim boundary

The slice can establish that the named implementations ran reproducibly on one
common Wisconsin tract input and can describe their resulting graph-native
assignments. It cannot establish a generally best algorithm, national ranking,
causal effect of structure alone, geometric compactness, electoral fairness,
VRA compliance, or legal fitness.
