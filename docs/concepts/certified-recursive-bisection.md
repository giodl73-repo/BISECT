# Certified Recursive Bisection

## The proposition

BISECT should not ask the public to trust a black-box optimizer.

The governing body first fixes:

- Census blocks and population;
- adjacency and island-link rules;
- the recursive district-count schedule;
- population priority;
- geographic boundary weights;
- prohibited inputs; and
- the final tie-break.

The software then proves that every required cut follows those rules.

## What stays unchanged

This is still recursive bisection.

For a region requiring `k` districts:

```text
left seats  = floor(k / 2)
right seats = k - left seats
```

Population is divided in the same ratio. California with 52 districts begins
`26/26`, then `13/13`, then `6/7`. Certification cannot replace this with a
different statewide arrangement.

## What becomes stronger

The current METIS pipeline finds high-quality cuts quickly. It does not prove
that its selected cut is best.

The certified procedure asks three decision questions at each node:

1. **Population:** Can any connected permitted cut improve population balance?
2. **Boundary:** At the best population bound, can any cut reduce weighted
   boundary?
3. **Canonical tie:** At both bounds, does an earlier canonical assignment
   exist?

Three independently checked UNSAT proofs answer “no.” The selected cut is then
unique under the enacted rules.

## Why this is the Huntington--Hill analogy

Huntington--Hill did not end disagreement over every theory of representation.
Congress chose one rule, and arithmetic ended recurring discretion over its
execution.

Certified BISECT follows the same division:

| Enacted policy choice | Mathematical execution |
|---|---|
| Which units and population count | Hash and verify the exact instance |
| How islands are connected | Apply the published deterministic bridge rule |
| How district counts split | Enforce the canonical recursive tree |
| Which objectives have priority | Solve them lexicographically |
| How exact ties end | Select the canonical assignment |
| Whether the rule was followed | Check proof certificates independently |

Mathematics settles execution after enactment. It does not decide which values
Congress should enact.

## Evidence currently implemented

- bounded exact optimality and infeasibility certificates;
- deterministic proof transcripts;
- independent E0 verifier implementation;
- generalized odd/even split objectives;
- complete recursive certificate trees;
- tree-to-RPLAN package verification;
- positive and hostile certificate corpora;
- OPB decision compilation with SAT counterexample detection; and
- a connected Rhode Island RCTX containing 25,649 blocks and 66,161 edges,
  including 64 deterministic island bridges.

## Remaining frontier

The first State certificate still requires:

1. a scalable exact discovery solver;
2. compact proof-loggable connectivity constraints;
3. production RoundingSat proof generation;
4. production VeriPB checking; and
5. publication of every proof, model hash, and failure.

Until those gates close, nationwide METIS results remain heuristic benchmarks.

## What “comes out on top” can mean

Certified BISECT already has the strongest **claim posture** among the project’s
construction methods: it defines how a unique answer could be proved rather
than merely scored or audited.

It has not yet demonstrated superior nationwide runtime, compactness, partisan
outcomes, or community preservation. Those comparisons require a separate
evidence package. A paper may claim stronger verifiability now; it must treat
empirical map-quality superiority as an open hypothesis.

## Public claim boundary

The intended future statement is:

> Given the enacted input, recursive schedule, objective order, and tie-break,
> this is the unique BISECT plan, and independent proof checking confirms that
> no better permitted cut exists at any node.

That is not a declaration that the axioms are inevitable, the map is a VRA safe
harbor, or every measure of political fairness is optimized.
