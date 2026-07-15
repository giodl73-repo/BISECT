---
pulse: 03
title: Batch operational tree generation
status: in_progress
depends_on: 02
wave: nationwide-2020-operational-certification
validation_level: L2 nationwide recursive execution
---

# Pulse 03 - Batch Operational Tree Generation

Generate deterministic operational recursive trees for every multi-district
State and reuse the completed one-district packages.

## Deliverables

- [x] resumable State/tree batch runner;
- [x] node-specific seed screening and retry policy;
- [ ] 50 operational State packages;
- [ ] 435 one-seat leaves;
- [ ] per-node arithmetic population proof status;
- [x] failure and retry ledger.

## Progress

The ledger contains 39 verified multi-district State packages and five open
failures (Arizona, California, Florida, New York,
and Texas). Every multi-district State has now
been attempted; no untouched State remains. Virginia is the first
package completed under deterministic two-phase seed screening: METIS-only
screens are ranked before population refinement, each screen has a recorded
180-second operational timeout, completed nodes and screens are reusable, and
timeouts remain distinct from infeasibility.

Virginia produces 11 connected one-seat leaves. Every recursive node reaches
its ratio-arithmetic population floor. At node `10`, eight screens timed out;
seed 14 subsequently reached the arithmetic floor, demonstrating why bounded
screening and retry evidence are both required.

Kansas then completed four connected leaves with arithmetic-floor deviation
zero at all three recursive nodes. The run exposed and repaired a scratch-space
scaling defect: screening now retains only the discovery required for ranking
and deterministic resume rather than duplicating full split instances.

Iowa completed four connected leaves with all three recursive nodes at their
arithmetic floors and no screen timeouts.

Tennessee completed nine connected leaves after bounded root and five-seat-node
screen timeouts. All eight recursive nodes reached their arithmetic floors,
including the resumed final three- and two-seat nodes.

Oklahoma's original three-seat frontier was recovered by the 32-seed
extension. Seed 28 reached floor 1 at the former frontier, and seed 21 reached
floor 1 at its final two-seat child. The completed package produces five
connected leaves and proves floor 1 at all four recursive nodes; its combined
original and extended screening evidence records ten timeouts.

Alabama completed seven connected leaves with all six recursive nodes at their
arithmetic population floors. Root seed 13 reached floor 3 after four bounded
screen timeouts; one three-seat child screen also timed out, while its second
ranked refinement reached floor zero. The independently verified State report
and manifest preserve the complete screening evidence.

Minnesota completed eight connected leaves with all seven recursive nodes at
their arithmetic population floors and no bounded screen timeouts. The final
two-seat pair required seven ranked population refinements before seed 7
reached floor zero, while the independently verified report preserves every
incumbent rather than reporting only the successful seed.

Wisconsin completed eight connected leaves with all seven recursive nodes at
their arithmetic population floors and no bounded screen timeouts. Each root
and four-seat split reached its floor on the first ranked population
refinement; the final two-seat node reached floor zero on its second ranked
refinement.

Indiana completed nine connected leaves with all eight recursive nodes at
their arithmetic population floors. The expensive nine-seat root was retained
across a bounded wrapper restart. Its five-seat child recorded nine screen
timeouts before seed 2 reached floor 1, demonstrating both completed-node and
completed-screen resume under a scaling-heavy State run.

Georgia completed 14 connected leaves with all 13 recursive nodes at their
arithmetic population floors. Its complete seven-seat left half was retained
across a bounded wrapper restart; the right-half population candidate and all
16 screens were then reused before the remaining subtrees completed.

North Carolina's timeout-limited right seven-seat frontier was recovered by
the 32-seed extension. Seed 18 reached floor 2 at the former frontier, and the
five remaining nodes reached floors 1, 1, 0, 1, and 1 using seeds 26, 6, 23,
23, and 31. The completed package produces 14 connected leaves, proves all 13
arithmetic population floors, and preserves 26 timeout markers across the
original and extended run.

Missouri completed eight connected leaves with all seven recursive nodes at
their arithmetic population floors. The root reached floor 4, its two
four-seat children reached floors 0 and 2, and their four two-seat descendants
reached floors 0, 0, 0, and 1. The independently verified report and manifest
preserve all deterministic seed screens and ranked refinements.

Michigan completed 13 connected leaves with all 12 recursive nodes at their
arithmetic population floors. Its 13-seat root recorded 12 bounded screen
timeouts but reached floor 2 from one of four completed candidates. The
six-seat parent survived a wrapper boundary, and its first three-seat child
reused one timeout marker before recording eight additional timeouts and
reaching floor 1. The independently verified report preserves the complete
resume and seed-refinement evidence.

Ohio completed 15 connected leaves with all 14 recursive nodes at their
arithmetic population floors. Its 15-seat root recorded six bounded screen
timeouts before seed 14 reached floor 1; its seven-seat child recorded nine
timeouts before seed 3 reached floor 3. The remaining subtrees completed
without screen timeouts, while final two-seat node `110` required a second
ranked refinement to reach floor zero. The independently verified report and
manifest preserve all screening and refinement evidence.

New York reached the arithmetic population floors at its 26-seat root, first
13-seat half, six-seat child, and completed descendants through node `001`.
At two-seat node `0011`, all 32 completed screens were population-refined;
best seed 1 remained at deviation 2 against floor zero. This is an unresolved
extended local-search frontier, not proof of infeasibility.

Pennsylvania completed 17 connected leaves with all 16 recursive nodes at
their arithmetic population floors. Its 17-seat root reached floor 6 after a
long seed-10 refinement, and the nine-seat half reached floor 3 on ranked seed
16 after seed 8 fell short. Both results and the complete eight-seat half were
retained across a wrapper boundary; the resumed final subtrees then completed
and independently verified.

Illinois completed 17 connected leaves with all 16 recursive nodes at their
arithmetic population floors. Its 17-seat root recorded four bounded screen
timeouts before seed 6 reached floor 8; its nine-seat half recorded six
timeouts before seed 7 reached floor 4. The final five-seat branch also
recorded five timeouts, while ranked refinements still certified every
remaining node. The independently verified report preserves all evidence.

Florida's former right-half seven-seat frontier was recovered by the 32-seed
extension: seed 18 reached arithmetic floor zero at node `10`. The run then
advanced into the newly exposed subtree. At three-seat node `100`, all 32
completed screens were population-refined; best seed 20 remained at deviation
3 against arithmetic floor zero. This is an unresolved extended local-search
frontier, not proof of infeasibility.

California reached the arithmetic population floors at its 52-seat root,
first 26-seat half, first 13-seat child, and completed six-seat subtree. At
seven-seat node `001`, two screens timed out and all 14 completed screens were
population-refined; best seed 3 reached deviation 261,137 against floor 2.
This is an unresolved local-search frontier, not proof of infeasibility.

Texas reached the arithmetic population floors at its 38-seat root, complete
first 19-seat half, second 19-seat parent, complete nine-seat child, and the
second-half ten-seat parent. At five-seat node `110`, two screens timed out and
all 14 completed screens were population-refined; best seed 6 reached
deviation 442,993 against floor 2. This is an unresolved local-search
frontier, not proof of infeasibility.

Colorado was recovered from its original single-refinement floor miss using
the current deterministic two-phase retry policy. It now produces eight
connected leaves, and all seven recursive nodes reach their arithmetic
population floors with no screen timeouts. The root reached floor zero from
seed 2; the two four-seat parents reached floor 2 from seeds 15 and 6; and all
four two-seat descendants reached their respective floors 0, 1, 1, and 0.

Washington's original root memory-allocation crash was first narrowed to a
deviation-3/floor-2 frontier and then recovered by the 32-seed extension. Seed
27 reached floor 2 at five-seat node `1`; the remaining second-half nodes
reached floors 0, 1, and 1 using seeds 5, 7, and 21. The completed package
produces ten connected leaves, proves the arithmetic population floor at all
nine nodes, and records three screen timeouts from the original 16-seed pass.

Arizona's original articulation-thread stack overflow was also cleared by the
current bounded-screening workflow. Its nine-seat root, complete four-seat
half, second five-seat parent, and two-seat child reach their arithmetic
population floors. At three-seat node `11`, all 32 screens completed and were
population-refined; best seed 3 remained at deviation 9,047 against arithmetic
floor 1. This is an unresolved extended local-search frontier, not proof of
infeasibility.

Utah was recovered from its old single-policy child-node miss under the
current all-seed refinement workflow. It now produces four connected leaves;
the four-seat root and both two-seat children all reach arithmetic floor zero,
using seeds 7, 16, and 9 respectively, with no screen timeouts.

A versioned extended-frontier wrapper now raises the deterministic retry bound
from 16 to 32 seeds without changing the frozen base builder or invalidating
existing package hashes. Extended packages bind both wrapper and base-builder
hashes and record their seed-frontier maximum in the package manifest.

No untouched multi-district State remains. Pulse 03 now continues by revisiting
the five preserved local-search frontiers; national wall-to-wall verification
remains blocked until operational packages are complete.
