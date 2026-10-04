Source: https://github.com/giodl73-repo/METIS-CORE
Pinned upstream revision: 78ae34090e043e79a206f2daffaa3889389b4790
License: MIT (see LICENSE).

The workspace patches the original Git dependency to this source copy so native
and WASM builds use the same reviewable fixes:

- Stop multilevel coarsening when a matching pass reduces the vertex count by
  at most 10%, retaining the valid hierarchy for initialization and refinement.
  Weighted hub graphs otherwise consume all 50 levels without reaching the
  coarse-size threshold.
- Respect requested contiguity in FM's label-propagation pre-balance pass. Moving
  an articulation vertex must not disconnect its source part or empty it.
- Bound parameterized SHEM/HEM aggregate vertex weights per constraint to
  ceil(1.5 * total constraint weight / coarse-size target). Existing oversized
  original vertices remain singletons. Both weighted and equal-edge matching
  enforce the same bound; it prevents population accumulating into huge
  indivisible coarse vertices.
- Grow one frontier vertex at a time from the least-populated initial region,
  rather than giving a hub's entire neighborhood to one region in one turn.
  Asymmetric target fractions guide this growth through `partition_with_targets`
  before refinement. Custom initializers retain their previous behavior through
  the trait's default implementation.
- Sample CSR vertex indices as `u32` in seed selection and HEM's Fisher-Yates
  shuffle. `usize` range sampling consumes different PCG output on wasm32 and
  64-bit native hosts, causing different assignments from identical seeds.

These changes do not establish balance guarantees or native/WASM parity.
The full engine validation gates remain open.

Current evidence: 184 partitioner tests, 264 pure-Rust runner tests (14 ignored),
16 release WASM smoke cases, and the RI/IA/NC Worker-adapter integration tests.
The allocated 2020 national release test completes all 50 states with contiguous
districts; 48 meet its 5% population tolerance. CA and TX still fail, and the
national validation command exits unsuccessfully. Seeded assignment results
change under these algorithm corrections; prior saved runs retain their original
module hashes and results.

`scripts/web/test_native_wasm_splits.mjs` compares every CA/TX tract assignment
and population/contiguity metrics from release native Rust and actual WASM for
the stated single-seed configuration. Both match after fixed-width sampling.
This evidence covers standard bisection only, not all structures/searches or
the native C METIS backend. Remaining CA/TX balance failures are also reproduced
by the native Rust split trace.


AreaSection integration adds per-constraint tolerance multipliers via
`MetisParams::with_constraint_tolerances`, propagated to FM refinement and trial
ranking. Joint frontier growth considers normalized population and secondary
weight targets. Multiconstraint LP uses interleaved weights correctly and cannot
improve population by worsening an area violation. The previous scalar LP and
post-refinement repair indexed/summed multi-constraint arrays as single weights;
they are bypassed for multiconstraint graphs. Scalar/default behavior remains
separate. AreaSection uses asymmetric population targets and equal secondary
(area) targets; non-equal secondary targets in the Rust runner remain unsupported
and explicit, pending the other engine ports. These are heuristic algorithms,
with independent physical-population/area feasibility checks before a root
candidate can be accepted. Current unit evidence: 187 partitioner tests and
270 pure-Rust runner tests (13 ignored), including area-swing and infeasible-atom
regressions. Native/WASM parity for AreaSection is recorded separately in
`target/native-wasm-area-parity.json`; this does not prove all-state quality.
