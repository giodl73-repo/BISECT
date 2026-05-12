# T.17 Flow-Based Construction

![T.17 flow construction visual](assets/t17-flow-construction.svg)

## Mental Model

Flow-based construction treats assignment as movement from supply to capacity.
Units provide population supply. District bins or seeds provide demand-side
capacity. Eligible arcs say which assignments the declared graph/profile allows,
and costs make some assignments preferable to others.

The current T.17 implementation is a deterministic feasibility-oriented
baseline. It is deliberately narrower than the later U.16/U.17 optimization
machinery. The central promise is that valid, infeasible, and invalid outcomes
are distinguished in the artifact trail.

## How BISECT Uses It

T.17 gives BISECT a constrained-assignment view of construction. Depending on
the configured profile, district bins or seeded targets receive unit supply
through eligible arcs. The method is useful when the question is not "where is
the next cut?" but "can this population supply be assigned to these district
capacities under the declared graph/profile?"

So the BISECT role is:

```text
unit supply + district capacity -> choose eligible assignments or emit witness
```

This also makes T.17 a bridge toward the later optimization papers. It prepares
the audit vocabulary for capacity bounds, costs, assignment extraction, and
infeasibility evidence before the heavier branch-and-cut or branch-and-price
machinery arrives.

## Picture 0: Eligible Arcs To Status Evidence

The opening figure shows the flow decision in countable form. Unit supplies sit
on the left. District bins sit on the right with visible lower and upper
capacity bounds. Only eligible arcs may carry assignment flow, and a selected
arc is acceptable only if the running bin load remains inside the declared
bound.

The important failure is explicit: sending `u3 = 50` to bin B after `u2 = 60`
creates `B = 110`, which exceeds the upper bound of `105`. T.17 must either
find a different feasible extraction or emit `infeasible`/`invalid` evidence.
For BISECT, that status fork is the point of the algorithm: a package should
say whether it contains a valid plan, a model infeasibility witness, or a
blocked invalid extraction.

## Algorithm Shape

```text
adjacency + populations
  -> source-side unit supply
  -> eligible arcs and assignment costs
  -> demand-side capacity targets
  -> extracted assignment or infeasibility witness
  -> RPLAN/RCTX/certificate package
```

## Picture 1: Flow Network

![T.17 flow network](assets/t17-flow-network.svg)

The flow view separates the construction into layers. A source provides unit
supply. Units connect through eligible arcs to district bins or seeded targets.
Each target has lower/upper capacity bounds. A successful run selects enough
arcs to assign all units while satisfying the declared capacity profile.

## Picture 2: Infeasibility Witness

![T.17 infeasibility witness](assets/t17-infeasibility-witness.svg)

When the staged model cannot satisfy capacity, the package should not pretend a
bad assignment is a plan. It records `infeasible` with a witness summary. When
an extracted assignment fails validation after construction, the status is
`invalid` and final export is blocked. This status taxonomy is one of the main
things T.17 adds to the construction family.

## Step-By-Step Mechanics

1. Create source-side supply from units and their populations.
2. Create demand-side capacity targets for districts or construction bins.
3. Add eligible arcs according to the declared graph/profile.
4. Attach costs from the staged constructor configuration.
5. Enforce capacity bounds while extracting assignments from selected flow.
6. Mark the result `valid` when assignment and validation checks pass.
7. Mark the result `infeasible` when the declared model cannot satisfy capacity,
   or `invalid` when extraction fails validation.

## What The Certificate Needs To Explain

The certificate needs to bind the status, capacity profile, selected seeds or
targets, edge cut, population deviation, parameter hash, and any infeasibility
witness. The fixed-point package then carries the plan, context, certificate,
and manifest only when the artifact behavior is appropriate for the status.

## Inputs

- Unit adjacency graph
- Unit populations
- Target district count
- Balance tolerance
- Arc eligibility and cost profile for the staged constructor

## Outputs

- District assignment when valid
- Flow summary with status, seeds/targets, edge cut, population deviation, and
  parameter hash
- Optional infeasibility witness
- RPLAN plan, RCTX context, audit certificate, and manifest in package runs

## When To Use It

Use flow construction when you want an assignment baseline that makes
capacity/cost status and infeasibility behavior explicit. It is also the right
mental bridge toward later branch-and-cut and branch-and-price work because it
already frames construction as a constrained assignment problem.

## Claim Boundary

Flow construction establishes benchmark-tier packaging and deterministic
capacity lineage. It is not yet a mature min-cost-flow solver and does not
prove legal sufficiency, optimality, or real-data quality. `Infeasible` means
infeasible for the declared staged model and profile, not necessarily infeasible
for every possible districting method.

## Tiny Example

Imagine four unit supplies trying to fill two district bins with capacity
between 95 and 105. If every eligible arc into the second bin comes from units
that overfill it, the model may have no valid assignment. T.17 should surface
that as an infeasibility witness rather than silently emitting an invalid plan.

## Worked Capacity Ledger

| Unit | Supply | Eligible bins | Selected bin | Running bin load |
|---|---:|---|---|---:|
| u0 | 48 | A, B | A | A = 48 |
| u1 | 52 | A | A | A = 100 |
| u2 | 60 | B | B | B = 60 |
| u3 | 50 | B | B | B = 110, invalid |

If bin B has an upper bound of 105, the last assignment cannot be accepted as a
valid plan. The correct artifact is either a different feasible extraction or
an infeasibility/invalid status with a witness.

## Witness Reading Checklist

- Capacity bounds should be visible as lower and upper numbers, not only a
  tolerance phrase.
- Eligible arcs should explain why some apparently reasonable assignments were
  unavailable.
- The status should distinguish model infeasibility from post-extraction
  validation failure.

## References In This Repo

- Crate: `bisect-flow`
- Paper: `docs/papers/T.17+flow-based-construction.pdf`
- Golden package: `docs/examples/rplan-golden-packages/T.17+flow-construction/`
- Benchmark package: `docs/examples/rplan-benchmark-packages/T.17+flow-path100-benchmark/`
