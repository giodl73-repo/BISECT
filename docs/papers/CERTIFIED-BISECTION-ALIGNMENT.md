# Certified Recursive Bisection Paper Alignment

## Flagship Paper

| Paper | Role | Status |
|---|---|---|
| U.21 | Certified recursive bisection methods, certificates, OPB proof architecture, and RI frontier | Draft source and PDF complete |

## Required Portfolio Updates

| Paper | Required change | Status |
|---|---|---|
| A.0 | Add certified BISECT to the synthesis architecture and distinguish heuristic national evidence from the exact North Star | Source updated |
| A.5 | Replace the old “foundations only” gap list with current bounded/tree/RI evidence | Source and PDF updated |
| B.02 | Align federal-law adoption stages with per-cut certificates and external proof gates | Source updated |
| B.1 | Replace blanket exact-infeasibility language; add certified extension and comparison agenda | Source and PDF updated; absent historical histogram disclosed |
| U.6 | Distinguish general ILP from recursive three-stage proof certification | Source and PDF updated |
| U.13 | Update exact-vs-heuristic boundary with split/tree/proof-request evidence | Source and PDF updated |
| U.16 | Place branch-and-cut in the discovery role, not the final proof role | Source and PDF updated |
| U.17 | Explain branch-and-price as another discovery/lower-bound engine feeding U.21 certification | Source and PDF updated |
| U.20 | Distinguish plan audit certificates from optimization proofs and link tree-to-RPLAN verification | Source and PDF updated |

## Comparison Needed Before “Comes Out On Top”

The current evidence supports:

> Certified BISECT has the strongest verifiability and execution-finality
> architecture in the project.

It does not yet support:

> Certified BISECT produces the best national maps or fastest national runs.

The first same-instance package is now complete for the path-8 root. Vendored
METIS seed 42 matches the certified assignment and objective exactly. On that
fixture certification adds proof strength, not a better cut. A deliberate
connected suboptimal control is correctly identified as worse.

A publication-grade comparison package must use identical instances and report:

1. METIS versus certified per-cut objective agreement;
2. population and weighted-cut differences;
3. downstream final-plan compactness and split metrics;
4. discovery runtime;
5. proof generation time;
6. proof size and independent verification time;
7. SAT counterexamples, timeouts, and failures; and
8. State-selection rules fixed before results are inspected.

## Recommended Publication Sequence

1. Publish U.21 as the methods and claim-boundary paper.
2. Update U.13/U.16/U.17/U.20 as the certification spine.
3. Update B.1 as the construction paper.
4. Update A.0/A.5/B.02 only after technical wording is stable.
5. Expand the comparison package from path-8 to precommitted real/synthetic
   benchmark instances.
6. Publish a separate empirical paper only if the results support a defensible
   superiority claim.
