# redist-metis: Mathematical Specification

**Document type**: Court-facing algorithm specification
**Version**: 1.0 — redist-metis v1
**Date**: 2026-05-02
**Audience**: Redistricting experts, court-appointed special masters, and technical
reviewers who can read mathematics but are not expected to read Rust source code.

---

## Table of Contents

1. Problem Statement
2. Definitions
3. Multilevel Paradigm
4. Phase I: Coarsening
5. Phase II: Initial Partition
6. Phase III: FM Refinement
7. Projection
8. Output Invariants
9. Determinism
10. Limitations

---

## 1. Problem Statement

Given a weighted undirected graph G = (V, E, w_v, w_e) where:

- V is a finite set of n vertices (census tracts or geographic units),
- E is a set of m undirected edges (shared boundaries between units),
- w_v : V → Z⁺ assigns a positive integer population weight to each vertex, and
- w_e : E → Z⁺ assigns a positive integer adjacency weight to each edge (optional;
  defaults to 1 for all edges when not specified),

the k-way graph partitioning problem asks for a surjective assignment function

    P : V → {0, 1, …, k−1}

that satisfies:

**Balance constraint**: For every part i in {0, …, k−1}, the total population of
that part deviates from the equal-population target by no more than ε:

    | Σ_{v: P(v)=i} w_v(v)  −  (Σ_v w_v(v)) / k |  ≤  ε

where ε = ⌈(Σ_v w_v(v)) × 0.005⌉ (ceiling of 0.5% of total population, computed
using integer arithmetic).

**Objective**: Subject to the balance constraint, minimise the total weight of edges
whose two endpoints are assigned to different parts:

    minimise  Σ_{(u,v)∈E, P(u)≠P(v)} w_e(u,v)

Minimising this edge cut corresponds to minimising the length of district boundaries
shared between parts, which promotes geographic compactness.

This problem is NP-hard in general. The multilevel approach (Sections 3–7) finds a
high-quality heuristic solution in O((n + m) log n) time.

---

## 2. Definitions

**CSR representation (Compressed Sparse Row).**
The graph G is stored as three arrays:

- `xadj[0..n]` (length n+1): `xadj[v]` is the index in `adjncy` where vertex v's
  neighbor list begins; `xadj[v+1]` is where it ends. `xadj[0] = 0` always.
- `adjncy[0..m']` (length 2m for undirected graphs): the concatenated adjacency
  lists. The neighbors of vertex v are `adjncy[xadj[v]..xadj[v+1]]`.
- `vwgt[0..n×ncon]`: vertex weights. For ncon=1 (single population constraint),
  `vwgt[v]` is the population of unit v. Weights must be strictly positive integers.
- `adjwgt[0..m']` (optional): edge weights parallel to `adjncy`. When absent, all
  edge weights are treated as 1.
- `ncon`: number of weight constraints per vertex (v1 supports ncon ≥ 1 in the graph
  representation; FM refinement enforces balance only for ncon=1 — see Section 10).

The predicate `is_valid(G)` holds iff:
  (a) `xadj` has length n+1 and `xadj[0] = 0`,
  (b) all vertex weights are strictly positive,
  (c) no self-loops (adjncy[j] ≠ v for any neighbor j of v),
  (d) all neighbor indices are in-bounds (adjncy[j] < n),
  (e) if adjwgt is present, its length equals adjncy's length, and
  (f) the graph is connected (BFS from vertex 0 reaches all n vertices).

**Edge cut.**
Given a partition P, the edge cut is:

    cut(P, G) = Σ_{(u,v)∈E, P(u)≠P(v)} w_e(u,v)

Each undirected edge (u,v) is counted exactly once. In CSR storage both directions
appear; the implementation divides the sum by 2.

**Population balance.**
For a k-way partition P of G, the balance deviation of part i is:

    dev(i) = | pop(i) − pop_target |

where  pop(i) = Σ_{v: P(v)=i} w_v(v)  and  pop_target = (Σ_v w_v(v)) / k
(integer division).

The balance tolerance ε is:

    ε = ⌈ (Σ_v w_v(v)) × 5 / 1000 ⌉  =  (total_pop × 5 + 999) / 1000

using integer ceiling division. This equals 0.5% of total population, rounded up.
The constraint is  dev(i) ≤ ε  for all parts i.

**Coarse graph.**
Given G and a surjective mapping cmap : V_fine → V_coarse (where |V_coarse| < |V_fine|),
the coarse graph G_c = contract(G, cmap) is constructed by:
- Each super-vertex c in V_coarse has weight equal to the sum of weights of all fine
  vertices v with cmap(v) = c.
- Two super-vertices c1 ≠ c2 are adjacent in G_c iff there exists an edge (u,v) in G
  with cmap(u) = c1 and cmap(v) = c2; the coarse edge weight equals the sum of
  all such fine edge weights.

**Matching.**
A matching M of G is a set of edges such that no vertex appears in more than one
edge of M. A maximal matching covers all vertices that can be matched. Every matching
induces a surjective comap: matched pairs (u,v) ∈ M share one coarse vertex; unmatched
vertices map to their own singleton coarse vertex.

**CoarseMap surjectivity.**
The function cmap : V_fine → V_coarse produced by each coarsening step is surjective:
every coarse vertex is the image of at least one fine vertex. This is guaranteed by
construction — each coarse vertex ID is assigned exactly once during the matching sweep.

---

## 3. Multilevel Paradigm

The multilevel graph partitioning paradigm solves the NP-hard partitioning problem
in three phases, exploiting the observation that a good partition of a small
approximation of G is a good starting point for refining a partition of G.

**Informal description.**

Phase I (Coarsening) constructs a sequence of progressively smaller graphs:

    G_0 = G,  G_1,  G_2,  …,  G_d

where each G_{i+1} is obtained from G_i by collapsing matched vertex pairs. The
sequence terminates when |V(G_d)| falls below a problem-size threshold (by default,
max(20k, 40) vertices). At each level, the graph retains the essential community
structure of G because heavily-weighted edges — which correspond to strongly-coupled
geographic units — are preferentially collapsed together.

Phase II (Initial partition) partitions the coarsest graph G_d directly. Because G_d
has far fewer vertices than G, even a simple heuristic (BFS expansion from k random
seeds) produces a partition whose quality is not much worse than optimal on G_d.

Phase III (Refinement and projection) uncoarsens the hierarchy: starting from G_d,
the partition is projected to G_{d-1} and then refined locally using Fiduccia-
Mattheyses (FM) boundary swaps. This projection-and-refine step repeats until the
partition on G_0 (the original graph) is obtained.

**Why it works.**

The key insight is that coarsening is a hierarchy of approximations, not a sequence of
independent problems. The projection step (Section 7) guarantees that a valid partition
of G_{i+1} maps to a valid partition of G_i: every vertex in G_i inherits the part
assignment of its coarse representative. The FM refinement at each level then makes
local improvements that are inexpensive because the refinement operates on the
(smaller) coarse graph, and the improvements at coarser levels propagate to finer
levels through projection.

---

## 4. Phase I: Coarsening

Two coarsening algorithms are implemented. Both operate in O(n + m) time and produce
a strictly smaller graph.

### 4.1 Heavy-Edge Matching (HEM)

HEM builds a maximal matching by visiting vertices in a random permutation and matching
each unmatched vertex with its heaviest unmatched neighbor.

```
Algorithm HEM(G, seed):
  rng ← Pcg64(seed)
  order ← Fisher-Yates shuffle of [0, 1, …, n−1] using rng
  matched[v] ← false  for all v
  cmap[v] ← UNSET     for all v
  next_id ← 0

  for v in order:
    if matched[v]: continue
    best_j ← argmax_{j ∈ neighbors(v), not matched[adjncy[j]]}  adjwgt[j]
              (ties broken by position; adjwgt[j]=1 if no weights)
    if best_j exists:
      u ← adjncy[best_j]
      cmap[v] ← next_id;  cmap[u] ← next_id
      matched[v] ← true;  matched[u] ← true
    else:
      cmap[v] ← next_id          // singleton super-vertex
      matched[v] ← true
    next_id ← next_id + 1

  return contract(G, cmap)
```

The random visit order breaks ties that would otherwise introduce systematic bias
in the matching. The fixed seed (0x1234_5678_9ABC_DEF0 for the HEM struct; the
caller-supplied seed in the full pipeline) ensures determinism (Section 9).

**Termination proof.** Each iteration of the outer loop marks at least one vertex
as matched (the vertex v itself). The loop visits each vertex at most once (the
`if matched[v]: continue` guard). Therefore the loop executes at most n iterations
and terminates. The output graph has at most ⌈n/2⌉ super-vertices (since each
coarse vertex absorbs at least one fine vertex), so |V(G_c)| ≤ ⌈n/2⌉ < n for n ≥ 2.
The graph strictly shrinks at every level.

The MAX_LEVELS = 50 cap in the CoarseningHierarchy builder provides an absolute
bound: coarsening cannot proceed past 50 levels regardless. For n vertices, after
at most ⌈log₂ n⌉ levels the graph would reach a single vertex; 50 levels is far
beyond any realistic census graph (n ≤ ~8 million block-level units).

**Complexity.** O(n + m): the shuffle is O(n), and the matching sweep visits each
edge at most twice.

### 4.2 Sorted Heavy-Edge Matching (SHEM)

SHEM improves upon HEM by processing vertices in decreasing order of their maximum
incident edge weight, so the heaviest edges in the graph are matched first.

```
Algorithm SHEM(G):
  for each v: max_w[v] ← max_{j ∈ neighbors(v)} adjwgt[j]  (or 1 if unweighted)

  // Bucket sort: O(n + max_weight) — NOT a comparison sort
  B ← array of empty lists, indexed 0 .. max(max_w)
  for each v: B[max_w[v]].append(v)

  matched[v] ← false  for all v
  cmap[v] ← UNSET     for all v
  next_id ← 0

  for bucket in B, iterated from highest index to lowest:
    for v in bucket:
      if matched[v]: continue
      best_j ← argmax_{j ∈ neighbors(v), not matched[adjncy[j]]}  adjwgt[j]
      if best_j exists:
        u ← adjncy[best_j]
        cmap[v] ← next_id;  cmap[u] ← next_id
        matched[v] ← true;  matched[u] ← true
      else:
        cmap[v] ← next_id
        matched[v] ← true
      next_id ← next_id + 1

  return contract(G, cmap)
```

**Quality theorem.** SHEM preserves heavy edges: an edge (u,v) with weight w is
matched (collapsed) before any edge with weight w' < w is considered, because v and
u both appear in bucket w or higher and are visited before lighter-weighted vertices
reach the matching step. This produces coarse graphs that better represent the
connectivity structure of G, leading to better final partitions.

SHEM is the default coarsener in the MetisPartitioner pipeline.

**Complexity.** O(n + m + W) where W = max edge weight. For integer weights bounded
by any polynomial in n, this is O(n + m).

### 4.3 Coarse graph construction

Both algorithms use the same `build_coarse_graph` routine:

- Super-vertex weights are accumulated in 64-bit integers to prevent overflow when
  many fine vertices (each with large population) are collapsed into one super-vertex.
- For each pair of super-vertices (c1, c2), parallel edges (arising from multiple
  fine edges between the same two super-vertices) are merged by summing their weights.
  This keeps the coarse graph free of multi-edges.
- Edge weight semantics are preserved: an unweighted input produces an unweighted
  output; a weighted input produces a weighted output with accumulated weights.

**Stopping condition.** Coarsening stops when `should_stop(G_current)` returns true.
The default threshold is |V(G_current)| ≤ max(coarsen_to × k, 40), where coarsen_to
is a parameter (default 20). This ensures the initial partitioner receives a graph
small enough to partition quickly while still large enough to provide a reasonable
initial solution.

---

## 5. Phase II: Initial Partition

### 5.1 GrowBisect (k = 2)

For the bisection case (k = 2), the initial partition is produced by simultaneous
BFS expansion from two randomly-selected seed vertices.

```
Algorithm GrowBisect(G, k=2, seed):
  rng ← Pcg64(seed)
  assignment[v] ← UNSET  for all v

  seed_a ← rng.uniform(0, n)
  seed_b ← rng.uniform(0, n), resampled until seed_b ≠ seed_a

  assignment[seed_a] ← 0;  queue_0 ← [seed_a]
  assignment[seed_b] ← 1;  queue_1 ← [seed_b]
  unassigned ← n − 2

  while unassigned > 0:
    for part in {0, 1}:
      v ← dequeue(queue_part)
      for each neighbor u of v:
        if assignment[u] = UNSET:
          assignment[u] ← part
          enqueue(queue_part, u)
          unassigned ← unassigned − 1

  // Safety fallback: any remaining UNSET vertex (only on disconnected input)
  // is assigned to part 0.
  return Partition(assignment, k=2)
```

The alternating BFS ensures roughly equal growth: each part expands one vertex at a
time per round, producing a roughly balanced bisection without any explicit balance
enforcement. FM refinement (Phase III) corrects any residual imbalance.

**For k > 2**: GrowBisect delegates to GrowKway (below). The name GrowBisect reflects
its primary use case; the implementation handles arbitrary k transparently.

### 5.2 GrowKway (k ≥ 2)

GrowKway generalises to k parts by selecting k distinct seed vertices at random and
then expanding all k BFS frontiers in round-robin order.

```
Algorithm GrowKway(G, k, seed):
  rng ← Pcg64(seed)
  assignment[v] ← UNSET  for all v

  seeds ← k distinct vertices chosen uniformly at random
          (with fallback if k > n: vertices are reused)

  for part in 0 .. k:
    assignment[seeds[part]] ← part
    queue[part] ← [seeds[part]]

  unassigned ← n − (number of distinct seeds assigned)
  round_part ← 0

  while unassigned > 0:
    part ← round_part mod k
    round_part ← round_part + 1
    v ← dequeue(queue[part])  (if non-empty)
    for each neighbor u of v:
      if assignment[u] = UNSET:
        assignment[u] ← part
        enqueue(queue[part], u)
        unassigned ← unassigned − 1

  return Partition(assignment, k)
```

**Important design note — NOT recursive bisection.** The initial partitioner is a
direct k-way BFS, not recursive bisection. This is an intentional departure from the
original METIS C library, which uses recursive bisection internally for the initial
coarsest-level partition. GrowKway produces a k-way partition in a single pass,
avoiding the error accumulation that can occur when recursive bisection is applied
to a heavily coarsened graph.

**Multi-constraint generalisation.** The CsrGraph struct supports ncon > 1
(multiple weight vectors per vertex, e.g. total population plus voting-age population).
In v1, the initial partitioner and FM refinement enforce the balance constraint for
ncon = 1 only. When ncon > 1, the additional constraints are stored and passed through
the pipeline but are not actively balanced. See Section 10 (Limitations).

---

## 6. Phase III: FM Refinement

The Fiduccia-Mattheyses (FM) algorithm performs local boundary improvement. It
iterates over boundary vertices in decreasing order of move gain, applying each move
if it does not violate the balance constraint, then rolls back to the best partition
seen during the pass.

### 6.1 Gain definition

For a vertex v currently assigned to part p, the gain of moving v to part q is:

    gain(v, p→q) = (weight of edges from v to vertices in q)
                 − (weight of edges from v to vertices in p \ {v})

Equivalently:  gain(v) = Σ_{(v,u)∈E, P(u)≠P(v)} w_e(v,u)  −  Σ_{(v,u)∈E, P(u)=P(v)} w_e(v,u)

A positive gain means moving v reduces the edge cut; a negative gain increases it.
For k = 2 the destination part is uniquely determined (the other part). For k > 2,
the destination is the part q ≠ p that maximises the sum of edge weights from v to
vertices already in q.

### 6.2 Bucket-sort gain structure

The GainTable maintains a set of (vertex, gain) pairs supporting O(1) max-gain
lookup and O(1) insert/remove/update, using a bucket array indexed by gain value.

    Buckets indexed 0 .. 2×max_gain,
    where bucket_index(g) = g + max_gain  (offset to make all indices non-negative).
    max_gain = max_edge_weight × max_degree  (an upper bound on any vertex gain).

`pop_max()` scans from the highest bucket downward to find the first non-empty bucket
and returns its last element. Insert, remove, and update are O(1) with O(max_gain)
table size.

### 6.3 FM algorithm

```
Algorithm FM(G, P, niter):
  state ← FmState(G, P)     // initialise gains for all boundary vertices
  best ← checkpoint(state)  // (assignment, cut)

  for pass = 1 .. niter:
    locked[v] ← false  for all v
    improved ← fm_pass(state, locked, best)
    state ← restore(state, best)   // reset to best known partition
    if not improved: break

  return state.assignment

Algorithm fm_pass(state, locked, best):
  total_pop ← Σ_v w_v(v)
  target    ← total_pop / k         // integer division
  ε         ← (total_pop × 5 + 999) / 1000   // ceiling of 0.5%
  start_cut ← best.cut

  loop:
    (v, gain) ← gain_table.pop_max()   // highest-gain unlocked boundary vertex
    if no such vertex: break
    locked[v] ← true

    from_part ← assignment[v]
    to_part   ← best destination for v (part with most edge weight from v, for k>2;
                 or the single other part for k=2)

    new_from_pop ← part_pop[from_part] − w_v(v)
    new_to_pop   ← part_pop[to_part]   + w_v(v)

    if new_from_pop < target − ε  or  new_to_pop > target + ε:
      continue   // skip: move would violate balance

    // Apply move
    assignment[v] ← to_part
    part_pop[from_part] ← new_from_pop
    part_pop[to_part]   ← new_to_pop

    // Update edge cut incrementally: O(degree(v))
    for each neighbor u of v:
      if assignment[u] = from_part: cut += w_e(v,u)   // edge newly crosses
      if assignment[u] = to_part:   cut -= w_e(v,u)   // edge no longer crosses

    // Update gains for unlocked neighbors of v
    for each neighbor u of v:
      if locked[u]: continue
      new_gain ← compute_gain(G, assignment, u)
      if u is on the boundary (has a neighbor in a different part):
        gain_table.insert_or_update(u, new_gain)
      else:
        gain_table.remove(u)  // u is now interior; no move gain

    if current_cut < best.cut:
      best ← checkpoint(state)   // save new best

  return (best.cut < start_cut)   // true iff this pass improved the solution
```

**Locked-vertex invariant.** Once a vertex is popped from the gain table and marked
`locked`, it is never re-inserted or moved again in the same pass. This is the
fundamental FM invariant: each vertex moves at most once per pass. It guarantees
that the loop terminates in at most |boundary| ≤ n iterations.

**Rollback to best checkpoint.** After each pass, the state is restored to the best
assignment seen during the pass (not the end-of-pass state). This allows FM to
"look ahead" through temporarily worse assignments to reach a better optimum.

**Convergence argument.** Each pass terminates in O(n) moves (at most n vertices,
each moved at most once). The cut is non-increasing from one best-checkpoint to the
next because a new checkpoint is saved only when the current cut strictly improves.
If no improvement is found during a pass, FM terminates early. In the worst case,
FM runs for `niter` passes (default 10); in practice, convergence occurs in 2–4
passes on typical census graphs.

### 6.4 k-way extension

For k > 2, FM selects the best destination part using the `best_destination` function:
the part q ≠ from_part that maximises Σ_{(v,u)∈E, P(u)=q} w_e(v,u) (sum of edge
weights from v to vertices already in q). The balance check and gain update are
identical to the k=2 case.

GreedyKWay (a simpler k-way refinement that performs greedy boundary swaps without
the gain table) is also implemented in `refine/kway.rs` but is not the default.

---

## 7. Projection

After refining the partition at level i+1 (a coarser graph), it must be mapped back
down to level i (the finer graph). The projection uses the CoarseMap stored during
coarsening.

**Definition.** Let cmap_i : V_i → V_{i+1} be the surjective mapping produced when
constructing G_{i+1} from G_i. Given an assignment A_{i+1} : V_{i+1} → {0,…,k−1},
the projected assignment A_i : V_i → {0,…,k−1} is:

    A_i[v] = A_{i+1}[cmap_i[v]]    for all v ∈ V_i

**Correctness invariant.** If A_{i+1} is a valid partition of G_{i+1} (every vertex
assigned, all IDs in {0,…,k−1}), then A_i is a valid partition of G_i. Proof:
- Every v ∈ V_i has A_i[v] defined because cmap_i is total (defined on all of V_i).
- A_i[v] = A_{i+1}[cmap_i[v]] ∈ {0,…,k−1} because A_{i+1} assigns valid IDs.

**Refine-then-project order.** The pipeline applies refinement before projection at
each level:

    for lev = depth−1 downto 0:
      A_{lev+1} ← FM_refine(G_{lev+1}, A_{lev+1})
      A_lev     ← project_up(lev, A_{lev+1})
    A_0 ← FM_refine(G_0, A_0)

This order means: (1) FM improves the partition at the coarser level (where moves are
cheaper because the graph is smaller); (2) the improved partition is projected to the
finer level; (3) FM then improves again at the finer level. The final FM call at
level 0 operates on the original graph G_0 and produces the output partition.

---

## 8. Output Invariants

The following three properties are the legally-cited correctness guarantees of
redist-metis v1. They are stated as theorems with proof sketches and are implemented
as Prusti postconditions on the `Partitioner::split()` entry point.

### Theorem 1: Full Coverage

**Statement.** For any valid input (G satisfying `is_valid(G)`, k ≥ 1, k ≤ n), if
`split(G, k, seed)` returns `Ok(P)`, then `P.assignment.len() = n`.

**Proof sketch.** Every vertex v ∈ V receives an assignment in GrowBisect or
GrowKway (Phase II) because those algorithms run BFS until all n vertices are
reachable (the graph is connected by `is_valid`) or fall through to the safe
fallback that assigns any UNSET vertex to part 0. FM refinement (Phase III) moves
vertices between parts but never adds or removes vertices from the assignment vector.
Projection copies the coarse assignment to a new vector of length |V_fine| = n_fine
at each level; the final projection produces a vector of length n = |V_0|.

### Theorem 2: Valid Part IDs

**Statement.** For any valid input, if `split(G, k, seed)` returns `Ok(P)`, then
`P.assignment[i] < k` for all i in 0..n.

**Proof sketch.** GrowBisect assigns values in {0, 1} for k=2, or values in {0,…,k−1}
for k>2 (GrowKway seeds each part i with value i). FM refinement only moves vertex v
to a destination part in {0,…,k−1} (the range is bounded by k in `best_destination`
and the k=2 complement formula). Projection copies values from the coarse assignment,
which by induction also satisfies this invariant.

### Theorem 3: Population Balance

**Statement.** For any valid input, if `split(G, k, seed)` returns `Ok(P)`, then for
all parts i in {0,…,k−1}:

    | pop(i) − pop_target |  ≤  ε

where:
- pop(i)      = Σ_{v: P(v)=i} w_v(v)
- pop_target  = (Σ_v w_v(v)) / k        (integer division)
- ε           = ⌈ (Σ_v w_v(v)) × 0.005 ⌉ = (total_pop × 5 + 999) / 1000

**Proof sketch.** FM refinement enforces the balance constraint explicitly: in
`fm_pass`, any move that would push `part_pop[from] < target − ε` or
`part_pop[to] > target + ε` is rejected with `continue` before being applied.
The initial partition (GrowBisect / GrowKway) produces a roughly balanced assignment,
which FM then refines while maintaining the ε-balance invariant. Because FM is called
as the final step at level 0, and no subsequent step alters assignments, the balance
constraint holds at output.

**Note on pop_target.** Integer division means pop_target = floor(total_pop / k).
For populations not divisible by k, this definition allows a maximum imbalance of
k−1 due to rounding alone, before adding ε. In practice, total census tract
populations are large (millions for a state) and k ≤ 53 (Alaska at-large through
California at 52 seats), making rounding error negligible relative to ε.

---

## 9. Determinism

**Claim.** For the same input (G, k) and seed, `split(G, k, Some(seed))` produces
identical output on all conforming platforms and across all builds of redist-metis.

**Basis of the claim.**

1. **RNG.** All random choices use `Pcg64` from the `rand_pcg` crate, seeded with a
   caller-supplied 64-bit seed (default 0xDEAD_BEEF_CAFE_1234 when no seed is given).
   Pcg64 is a well-specified, platform-independent PRNG with defined output on all
   IEEE 754 platforms. Its output depends solely on the seed and the number of calls.

2. **Integer arithmetic only.** All computations in every phase — coarsening weight
   accumulation, bucket sorting, BFS ordering, FM gain updates, population balance
   checks — use 32-bit or 64-bit signed integers exclusively. No floating-point
   arithmetic appears in any algorithmic phase. There is therefore no platform-specific
   floating-point rounding or NaN behaviour to affect results.

3. **Deterministic data structures.** The gain table uses a flat bucket array with
   deterministic pop/insert behaviour (LIFO within each bucket). No hash maps or other
   nondeterministic structures are used in the hot path.

**Scope of the claim.** Determinism holds for valid inputs only (`is_valid(G)` must
hold). For invalid inputs, `split` returns `Err(PartitionError::InvalidGraph)` and no
partition is produced.

Determinism is verified by a golden test: `tests/golden/vt_seed42.json` contains the
expected output for Vermont's 2020 census tract graph with seed 42. This test is part
of the continuous integration suite and will fail if any code change alters the output.

---

## 10. Limitations

The following properties are NOT guaranteed by redist-metis v1. They are documented
here explicitly to avoid misinterpretation in expert testimony or court filings.

**1. Global optimality.** The minimum edge cut partition is NP-hard to compute in
general. The multilevel FM heuristic finds high-quality solutions in polynomial time
but does not guarantee the globally optimal partition. Different random seeds may
produce different valid partitions with different edge-cut values.

**2. Uniqueness of output.** Different seeds produce different (but each valid)
partitions. There is no canonical "correct" partition for a given (G, k); any
partition satisfying the balance constraint is a valid redistricting plan. The
choice of seed affects which valid plan is produced, not whether the output is valid.

**3. Numerical match to the C METIS library.** redist-metis is an independent Rust
implementation of the multilevel FM algorithm. It does not reproduce the exact vertex
numbering, part assignments, or edge-cut values of the METIS 5.x C library for the
same input. This is by design: the Rust implementation uses GrowKway (not recursive
bisection), SHEM (not the METIS RANDOM_SHEM variant), and a different RNG. The
output is expected to differ numerically from C METIS while satisfying the same
mathematical guarantees.

**4. Multi-constraint population balance.** The CsrGraph representation supports
`ncon > 1` (multiple population weight vectors per vertex, e.g., total population
and voting-age population). In v1, the FM refinement enforces the ε-balance constraint
only for the first weight dimension (ncon constraint 0). Additional weight constraints
(ncon ≥ 2) are stored in `vwgt` and passed through coarsening (where they are
accumulated correctly), but the FM balance gate does not enforce them. Full
multi-constraint FM is tracked as a future enhancement.

**5. Proportional (weighted) part sizes.** The `split_weighted(G, fracs, seed)` entry
point accepts a vector of fractional targets (e.g., [8, 9] for a 8:9 population
split). In v1, these fractions are not passed to the FM refinement loop: the current
implementation delegates to equal-weight `split(G, k, seed)` where k = len(fracs),
and FM balances to equal targets. Callers requiring tight proportional balance should
scale vertex weights before calling this function. Full proportional tpwgts support
is tracked as a future enhancement.

**6. Contiguity guarantee.** The algorithm does not explicitly enforce geographic
contiguity of districts. Connected components are promoted by the graph structure
(the input graph contains only adjacency edges between physically contiguous tracts),
and the BFS-based initial partitioner tends to produce contiguous parts. However,
contiguity is not verified by the algorithm and is not a formal postcondition of v1.
Contiguity checking is the responsibility of the caller (redist-core validates
contiguity as a separate post-processing step).

---

*End of specification.*
