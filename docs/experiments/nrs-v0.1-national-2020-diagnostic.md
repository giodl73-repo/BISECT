# NRS v0.1 2020 national diagnostic

## Status

This is a retained diagnostic record, not a conformance claim. The run exercised
the pre-revision reference engine whose profile named source commit
`c247e3032c7f5045d118ec3bdfebe747d226e4ee`. The national batch harness was
introduced at `138ad803`. The run used the fixed batch timestamp
`2026-08-05T00:00:00Z` and completed all 50 State rows.

The final `verify-nrs-batch` sweep passed with 38 verified packages and 12
failed rows. Ten failures were genuine single-candidate population-tolerance
failures. Two were harness failures. The diagnostic ledger had SHA-256
`c40e42f1d0d9ae1b4f72aa590fd2b1d8a6a12626ab5e09ce86ab4fd72ccbc455`.
The sum of recorded State elapsed times was 8,437.277 seconds (2.34 hours),
excluding recovered acceptance packages that did not retain an elapsed time.

## Population-tolerance failures

Each witness means only that the one canonical candidate missed the profile
tolerance. It does not prove that no feasible connected partition exists.

| State | Node | Achieved scaled deviation | Allowed | State seconds | Failure witness SHA-256 |
|---|---:|---:|---:|---:|---|
| CA | `10000` | 202,774 | 7,604 | 1,657.761 | `2f31ac7d14080dd1be3b00b32e2598d3da5a512ff9abce444361d6f7cbc8f731` |
| CO | `0` | 371,414 | 28,869 | 334.013 | `3eb28bbc94f37c781e3a128a8223ad6e1b9fee23c7fa633b65225bb5a3260257` |
| FL | `0101` | 9,646 | 7,693 | 509.820 | `9b8cc2498d0b2ecd10f4a445b2dea686aab0397bd877c3cf161f4b03f6a770e1` |
| GA | `000` | 34,193 | 7,652 | 402.882 | `03ea53d0043dae68ac29559c141ef9f166ef24af68a1c6a287d1e80498a9d14a` |
| KY | `01` | 48,207 | 7,510 | 175.104 | `e00628633952735ee3321fe13432239d947468681523b4c2545365451ad685a0` |
| LA | `10` | 36,759 | 7,763 | 208.073 | `992f332ca3deb5bb67db9aaaf9a142705dc255739e63049c60bb96c6cede8a36` |
| MA | `1` | 94,468 | 31,245 | 165.745 | `e54df84fb19c5801bc33a4d7c5ea56e988db9265db056a1bb969fd361a563468` |
| NC | `11` | 317,516 | 29,827 | 428.861 | `b830cc7989cc48726f0e25857a1251d78ff2684e21e2fcacf64753cfb6ad66c0` |
| NY | `1100` | 95,375 | 7,770 | 789.280 | `6fe097077197026e904c287a1c56e23680d2ccc3a023457860ccf738d2d115fe` |
| TX | `01111` | 69,210 | 7,670 | 593.661 | `01613371f24b48b52c9dc8c2efefc13c1cc3a1ecba7bc7355af152e308a720c8` |

The failures occurred at root-adjacent, middle, and deep recursive nodes. Raw
METIS candidates that were already connected bypassed the deterministic DFS
tree-cut initialization, leaving articulation-safe block moves to repair poor
population balance. That behavior was therefore a general engine defect, not a
State-specific data anomaly.

## Harness failures

- Minnesota encountered Windows sharing violation `os error 32`. No algorithm
  failure witness was produced.
- New Jersey retained a partial canonical `package/` after the first worker was
  interrupted between artifact writes and ledger checkpointing. The old resume
  path rejected the existing directory instead of rebuilding it.

The replacement harness builds in `package.in-progress`, independently verifies
there, and atomically promotes only complete packages or explicit algorithm
failure witnesses. It also recovers staged terminal artifacts, rebuilds legacy
partial canonical directories, automatically retries harness-only rows, and
uses bounded retry for Windows sharing violations. That change is committed at
`9ec2a729`.

## Performance signal

Illinois verified, but took 1,730.421 seconds. Its two-district node `011`
alone consumed about 20.2 minutes before passing. California node `0` took
about 13 minutes before passing, and California later failed at node `10000`.
These are retained before/after regression cases; recursion size alone did not
explain the long tail.

## Corrective revisions and next gate

Commit `8f6239e5` removes the connected-candidate bypass. Every NRS recursive
split now selects the same minimum-GEOID-rooted, ascending-neighbor DFS tree
edge cut, ordered first by population deviation and then by weighted boundary
cut, population moved from raw METIS labels, and minimum GEOID. The profile and
verifier now call this operation `candidate-initialization` and state that it
applies to every recursive split.

A targeted rerun rejected that change as sufficient: Massachusetts node `1`
and Kentucky node `01` reproduced exactly the prior achieved deviations
(94,468 and 48,207). Their method records proved that the new initialization
ran, but the single-block population repair converged to the same articulation
local minima. The targeted worker was stopped after those two conclusive
failures rather than spending time on the remaining rejected-profile States.

Commit `094160f8` replaces the single-block population repair with
connectivity-preserving connected-subtree moves. At each repair step it builds
ascending-neighbor DFS trees from 16 evenly spaced canonical heavy-child
roots, considers improving subtrees whose transfer keeps both children
connected, and orders equal-deviation candidates by boundary-cut change,
moved population, GEOID, root, and subtree root. A fixture specifically proves
escape from an articulation local minimum that no boundary-block move can
cross.

No national success claim follows from this diagnostic or from the corrective
commits. The revised profile must regenerate all seeds, pass the ten failed
States and the Illinois runtime regression, complete all 50 State packages, and
pass `verify-nrs-batch --require-complete` before national baseline completion
is recorded.

Before that rerun, the batch custody model was also strengthened. Ledger v2
binds both the canonical standard-profile hash and the exact executable hash;
State manifests and retained failure witnesses bind the executable too. Resume
now rejects packages, failures, and seed directories from a different profile
or executable, preventing a single ledger from silently mixing algorithm
revisions.

## Subtree-repair targeted debug run

The custody-bound ledger-v2 debug run (ledger SHA-256
`95f3e70acc60622281d33f7596084e19653c3df76a9755a4c9533aa237fd62c1`)
verified the first five historical tolerance failures with no retained failure
witnesses:

| State | Wall seconds | Nodes | Arithmetic floors attained | Maximum scaled deviation |
|---|---:|---:|---:|---:|
| Colorado | 111.785 | 7 | 7 | 2 |
| Georgia | 247.615 | 13 | 11 | 6 |
| Kentucky | 102.478 | 5 | 5 | 1 |
| Louisiana | 100.557 | 5 | 5 | 3 |
| Massachusetts | 104.673 | 8 | 5 | 4 |

This includes exact passage through the formerly failing CO `0`, GA `000`, KY
`01`, LA `10`, and MA `1` nodes. The same run then passed North Carolina's
formerly failing `11` node, but its final 30,743-block `111` node exceeded 20
CPU-minutes in an unoptimized debug executable. The worker was deliberately
stopped before a terminal artifact was emitted. That interruption is a runtime
diagnostic, not an algorithm failure. Production national certification must
use optimized release executables in a fresh ledger because ledger v2 forbids
mixing executable hashes.
