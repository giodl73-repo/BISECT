# Certified Recursive Bisection

## Split Schemas

- Instance: `certified-recursive-bisection-split-instance-v1`
- Certificate: `certified-recursive-bisection-split-certificate-v1`
- Proof: `certified-recursive-bisection-split-proof-v1`
- Tree: `certified-recursive-bisection-tree-v1`
- Model: `certified-standard-bisect-split-v1`

## Canonical Seat Schedule

Every node uses the existing `standard-bisect` rule:

```text
k_left  = floor(k_parent / 2)
k_right = k_parent - k_left
```

California's 52-seat schedule therefore begins `26/26`, then `13/13`, then
`6/7`.

## Population Objective

For parent population `P`, left-child population `P_left`, and seat counts
`k_parent`, `k_left`, define:

```text
left_deviation = abs(k_parent * P_left - k_left * P)
```

The right deviation is defined analogously. The exact objective minimizes:

1. maximum left/right scaled deviation;
2. total left/right scaled deviation;
3. weighted boundary cut; and
4. canonical assignment.

This generalizes the equal-population E0 objective to odd recursive splits
without changing the enacted bisection tree.

For every binary split, the left and right deviations are equal, so total
deviation is exactly twice maximum deviation. Both fields are retained for
schema continuity with the E0 certificate; the second does not independently
break ties.

## Orientation

- Equal-seat splits remove label symmetry by requiring canonical unit 0 in the
  left child.
- Unequal-seat splits are already oriented: left receives `floor(k/2)` seats
  and right receives `ceil(k/2)` seats. Unit 0 is not forced left because that
  could exclude the correct ratio-oriented cut.

Assignments use `0` for the left child and `1` for the right child. Both
children must be nonempty and connected. Objective evaluation alone does not
establish connectivity; exact solvers and verifiers must apply the connectivity
predicate as a feasibility condition before comparing objectives.

## Identity

The instance binds:

- the existing `BisectionTree` node path (empty root, then binary child path);
- parent certificate ID for non-root nodes;
- recomputable canonical unit-universe hash;
- canonical unit IDs and populations;
- weighted adjacency;
- parent and child seat counts; and
- orientation rule.

Certificate identity is canonical JSON SHA-256 over the instance identity,
result, and proof summary.

## Bounded Exact Oracle

`bisect-ilp::solve_certified_split_bounded` supports at most 24 units.

- Equal-seat splits enumerate the symmetry-reduced space with unit 0 fixed
  left: `2^(n-1)-1` candidates.
- Unequal-seat splits enumerate both seat-oriented assignments, excluding only
  empty children: `2^n-2` candidates.

Connectivity is checked before objective evaluation. The oracle emits either
the unique canonical optimum or exact infeasibility, plus an ordered transcript
commitment over every candidate.

Feasible and tie counts refer to the enumerated canonical space. For equal-seat
splits, excluded label-swap twins are not counted. The transcript domain
directly includes the instance hash before candidate records.

`bisect-ilp::verify_certified_split_bounded` recomputes the instance,
certificate and proof identities, exhaustive search, objective, tie count,
canonical assignment, and transcript commitment without trusting the submitted
result.

## Recursive Tree

`bisect-ilp::solve_certified_bisection_tree_bounded` recursively applies the
exact split oracle. The tree artifact contains:

- split instance, certificate, and proof for every non-leaf node;
- nodes in the exact BFS order emitted by `bisect_core::BisectionTree`;
- parent-certificate IDs on every child node;
- recomputed child unit universes, populations, and induced edges;
- one-seat leaves in lexicographic binary-path order; and
- one canonical district index per leaf.

`bisect-ilp::verify_certified_bisection_tree_bounded` verifies every split,
reconstructs both child contexts from the certified parent assignment, checks
the canonical seat schedule, rejects missing or duplicate paths, and requires
the leaves to partition the root unit universe exactly once.

The split feasibility rule also requires each child to contain at least as many
units as seats assigned to that child. This prevents a certified local cut from
creating structurally empty final districts.

## CLI Package

`bisect exact --method certified-recursive` emits:

- `certified-bisection-tree.json`;
- `certified-tree-package-manifest.json`;
- `exact.rplan`;
- `exact.rctx`; and
- `audit-certificate.json`.

The manifest records the tree ID, root unit-universe hash, district/split/leaf
counts, declared audit tolerance, and SHA-256 of every emitted sibling
artifact.

`docs/examples/certified-recursive/` commits one valid path-8/four-district
package and five hostile trees covering tree-ID tampering, false split
optimality, missing leaves, noncanonical node order, and leaf-universe
substitution.

The standalone package verifier re-derives the final assignment from verified
leaves, compares it to RPLAN, verifies the RPLAN audit certificate against
RCTX, and recomputes every package-manifest file hash.

## Claim Boundary

The current implementation exactly solves and verifies a complete bounded
recursive tree. It does not yet provide the second independent implementation,
prove that local greedy cuts always admit downstream completion, provide a
production proof backend, or prove real block-scale cuts.

The proof-backend prototype and its discovery/certification boundary are
specified in [`certified-proof-backend.md`](certified-proof-backend.md).
