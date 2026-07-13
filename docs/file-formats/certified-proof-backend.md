# Certified Split Proof Backend

## Separation Of Responsibilities

### Discovery

A branch-and-cut, branch-and-price, or other fast solver proposes:

- a connected canonical split assignment;
- its exact population/cut objective;
- solver and method identity; and
- the exact split-instance hash.

`certified-split-discovery-v1` is explicitly not an optimality proof.

`bisect exact --method certified-discovery` emits this record from deterministic
METIS. Equal-seat discovery uses METIS connectivity/minimum-connection options;
the candidate is rejected before emission if either child fails the certified
connectivity predicate. Zero-population METIS vertex weights are floored to one
for discovery only; the recorded objective is recomputed from true Census
population.

### Certification

The compiler emits `certified-split-proof-request-v5` decision problems:

1. population lower-bound decision;
2. boundary lower-bound decision at the accepted population bound; and
3. canonical predecessor decision at the accepted population/cut bounds.

Each decision is encoded as OPB over:

- `x_i`: unit `i` assigned to the right child;
- `y_e`: edge `e` cut;
- seat-count lower bounds;
- ratio-scaled population bounds;
- cut linearization and threshold;
- canonical equal-seat orientation;
- connectivity exclusions; and
- the lexicographic predecessor bound where applicable.

An UNSAT proof for all three decisions establishes the unique canonical split.
A SAT result is a counterexample and sends the discovery back to the solver.

## Proof Contract

- Requested proof format: VeriPB.
- Generator template: `roundingsat --proof-log={proof} {opb}`.
- Independent checker: VeriPB.
- The OPB SHA-256 and proof-request identity must be checked before proof
  verification.
- Solver incumbents, floating-point bounds, and optimality-gap metadata are not
  certificates.

## Prototype Boundary

The current bounded compiler generates static connectivity no-goods by
enumerating assignments. This validates the decision-query contract but is not
scalable. A pinned RoundingSat/VeriPB smoke proof is committed separately, while
the request package itself keeps `proof_status: not-generated`.

Production work must use a compact proof-loggable connectivity formulation or
proof-aware cut generation and must preserve the exact recursive split/tree
semantics.

## Compact Parent/Depth Encoding

`parent-depth-v3` replaces exponential disconnected-assignment no-goods.
For each child it introduces:

- exactly one assigned root;
- one incoming parent arc for every assigned non-root unit;
- parent arcs whose endpoints are assigned to the same child; and
- binary depth values with strict parent-to-child increase.

The depth inequalities prohibit cycles. Every assigned non-root unit follows a
finite parent chain to the unique root, which establishes connectivity.
Version 3 forces all depth bits for units outside a child to zero and uses
linear prefix variables to select the minimum-index assigned unit as each
child's unique root. These are symmetry reductions: they do not remove any
feasible connected partition.

For an instance with `n` units, `m` undirected edges, and
`b=ceil(log2(n))` depth bits, the encoding uses:

```text
n + m + 2n + 4m + 2nb
```

Boolean variables, plus polynomially many constraints.

The path-8 boundary decision compiles to 107 variables. The committed
RoundingSat/VeriPB smoke proof records the earlier parent-depth-v1 encoding;
current requests use the deterministic-root v3 encoding.

Compact requests are emitted as `proof-required-unclassified`: the compiler no
longer performs bounded exhaustive SAT/UNSAT classification. RoundingSat and
VeriPB determine the result, allowing instances above the 24-unit oracle limit
to be compiled without hidden enumeration.

## Cutset Connectivity Backend

`cutset-v1` keeps only assignment, boundary-edge, and deterministic-root prefix
variables in the initial model. A SAT assignment is independently decomposed
into connected components. For every disconnected component, the separator
adds a graph-checked constraint requiring either:

- the child root to lie inside that component; or
- at least one graph-boundary neighbor to join the child.

The compiler rejects cuts whose submitted outside-neighbor set does not exactly
match the instance graph. The iterative runner records every model, assignment,
cut set, and solver status. A final UNSAT claim still requires a proof generated
from the complete accumulated model and accepted by VeriPB.

For heuristic fixed-core branches, `cutset-reduced-v1` removes fixed assignment
variables entirely, folds fixed population and fixed-fixed boundary edges into
constants, and retains only disagreement units and incident cut edges. The
fixed cores must be nonempty and connected. This is useful for incumbent
discovery, but consensus-fixed cores are not globally proof-safe unless a
separate branch manifest proves complete coverage.

`connectivity-relaxation-outside-core-v1` is the exact complement of a
fixed-core branch: it keeps the unrestricted assignment/cut model and requires
at least one fixed label to change. The fixed-core branch plus this complement
cover every assignment. This decomposition changes proof organization only; it
does not promote consensus labels to global facts.

## Rhode Island Model Package

The connected Rhode Island discovery compiles into three local OPB files:

| Stage | Variables | Constraints | Bytes |
|---|---:|---:|---:|
| Population | 1,228,520 | 1,468,963 | 171,152,119 |
| Boundary | 1,228,520 | 1,468,964 | 172,119,842 |
| Canonical | 1,264,983 | 1,578,352 | 175,943,678 |

The canonical stage uses a linear-size prefix automaton rather than
power-of-two lexicographic coefficients. The files remain under ignored local
data custody; hashes and request identities are committed in the scalable
frontier report.

The current Rhode Island proof frontier closes the population stage at scaled
deviation 1. The boundary stage timed out and the canonical stage remains
blocked behind it.

Boundary proof-request v3 can fix exact right-child population. For Rhode
Island the two branches are 548,689 and 548,690; their union is exactly the
population-optimal feasible set. Both strengthened branch probes timed out.

The Rhode Island frontier contains 25,649 assignment variables and 66,161 cut
variables after applying 64 established island bridges. Static no-goods remain exponential and
are therefore prohibited as a claimed scalable solution.
