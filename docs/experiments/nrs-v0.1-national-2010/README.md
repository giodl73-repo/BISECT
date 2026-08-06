# NRS v0.1 National 2010 Result

## Outcome

The governed NRS v0.1 2010 batch finished with **49 verified States and one
retained candidate failure**. It therefore did not earn a complete national
conformance claim.

- Census cycle: 2010
- States evaluated: 50
- Blocks covered by the inventory: 11,071,790
- House districts: 435
- Verified State packages: 49
- Failed candidates: 1 (California)
- Sum of recorded State elapsed times: 6,297.529970 seconds
- Ledger SHA-256: `183eb7f80523f5cb4e91e9763f3c3fdda628f84efd08bb443f4fb7411a4fea2d`
- Standard-profile canonical SHA-256: `475fede6fc35d5d2bb6534a088a77232ed3e7fa64899847d7baaeb029f1f6969`
- Legal-profile canonical SHA-256: `742b460e8150bcedb94276a98e467a6528d7f3af6fb2429d9ff8716e3a202bae`
- `bisect.exe` SHA-256: `f2f19787ab6899ae495fa648fe8166011b2842d046e96e9869c1b8cbabaed135`

The aggregate verifier passed every successful package and accepted the
retained failure record as the sole nonconforming result. Michigan was the
largest recorded runtime at 3,146.108986 seconds; Texas took 418.408319
seconds. Tennessee was recovered from a completed staging package, so its
3.333715-second ledger duration records recovery verification rather than its
original construction time.

## California witness

California stopped at recursive node `00110`:

- Engine seed: `1361835940`
- Achieved scaled deviation: `14,735`
- Allowed scaled deviation: `6,991`
- Discovery SHA-256: `fef4364c584e9608118bcb9ea00203a2b4249195328033ce19f288227ecdb2f8`
- Failure-witness SHA-256: `4e973e4a4eef0297d830a19405a92a50aae862d6a19976c7066991c7f4d2c903`

This means the single canonical v0.1 candidate missed the profile tolerance.
It does **not** prove that no feasible partition exists.

## Seed experiments

Sixteen adjacent node seeds (`1361835941` through `1361835956`) all produced
the identical `14,735` deviation. A separately derived full-State candidate
seed (`1571129042`) also reached the same node and deviation. Seed retry was
therefore rejected as the amendment mechanism.

## Versioned response

NRS v0.2 is a narrow deterministic refinement. It always evaluates the v0.1
minimum-GEOID-rooted DFS candidate first. Only when that candidate misses the
unchanged 0.5 percent population tolerance does it evaluate a fixed set of 16
canonical DFS roots, apply the same connectivity-preserving population repair,
and select by `(population deviation, weighted cut, moved population,
canonical assignment)`.

On the retained California node, v0.2 reduced scaled deviation from `14,735`
to `207`. A complete 53-district California v0.2 package subsequently passed
independent population, tree, hash, assignment-coverage, and connectivity
verification. This does not alter the recorded v0.1 result or turn it into a
pass.

## Claim boundary

These are candidate reference-baseline results. They do not establish global
optimality, legal validity, VRA compliance, partisan fairness, or official
adoption.
