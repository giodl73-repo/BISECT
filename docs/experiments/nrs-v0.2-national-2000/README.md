# NRS v0.2 National 2000 Result

## Outcome

The governed NRS v0.2 Census 2000 batch finished with **49 verified States
and one retained candidate failure**. It therefore did not earn a complete
national conformance claim.

- Census cycle: 2000
- States evaluated: 50
- Blocks covered by the inventory: 8,199,908
- House districts: 435
- Verified State packages: 49
- Failed candidates: 1 (Hawaii)
- Sum of recorded State elapsed times: 2,721.200417 seconds
- Ledger SHA-256: `2b73c53464a25db01254e050c70bc33775a2d7adcd56961e1d48cd3df4406cc9`
- Standard-profile canonical SHA-256: `f73297cd39ef5e91d21a1fa171bcf0f246df50630e577572e10b9b281a07e0e4`
- Legal-profile canonical SHA-256: `b610346fce43f78eceacdc73334a0d88ea98d3c8b98a29f3f62ca5ff477b33d4`
- `bisect.exe` SHA-256: `fbb57879d68d9d30e763c1907abadbe45245350a9b068652eddd71b30299a3db`

The aggregate verifier passed every successful package and accepted the
retained failure record as the sole nonconforming result.

## Hawaii witness

Hawaii stopped at the root split:

- Engine seed: `548725438`
- Achieved scaled deviation: `802,861`
- Allowed scaled deviation: `6,058`
- Discovery SHA-256: `65549818b5cdf7a6ed4ea00ed2cc6c66aed50c92eae0aa6bf9d11ac6ba683418`
- Failure-witness SHA-256: `af98ad5f49a6fc45a3110d507f4c16bee9701b00dd950a7b87db636f443e7473`

This means the deterministic v0.2 candidate set missed the profile tolerance.
It does **not** prove that no feasible partition exists.

## Diagnosis and versioned response

The Census 2000 Hawaii graph has 16 land-connected components linked by
versioned zero-weight bridge edges. The largest-by-unit-count bridge anchor is
not the largest-by-population island, and the v0.2 full-graph DFS fallback does
not expose a population-balanced connected subtree.

NRS v0.3 is a narrow deterministic refinement. It preserves the v0.2 seed
stream and evaluates the v0.1 primary candidate first. Only after that
candidate misses tolerance does it remove bridge edges for candidate
discovery, enumerate canonical DFS and BFS tree cuts within each land
component, place the other land components together on either side, and
select the first globally minimum-deviation candidate in the frozen
enumeration order that leaves both labels connected in the full graph. The
population tolerance is unchanged.

A direct production-path Hawaii pilot using v0.3 and the retained v0.2 seed
returned scaled deviation `41`, inside the unchanged `6,058` bound. That pilot
motivates, but does not replace, the governed v0.3 national batch.

## Claim boundary

These are candidate reference-baseline results. They do not establish global
optimality, legal validity, VRA compliance, partisan fairness, or official
adoption.
