# Track R — Adversarial Robustness Panel Review
**Date**: 2026-05-09 | **Round**: 1
**Papers**: R.0–R.4 (5 papers)
**Reviewers**: Karypis, Liang, Duchin, Stephanopoulos, cryptographer (Rogaway)

## Paper Scores (Round 1)

| Paper | Title | Score | Verdict |
|-------|-------|-------|---------|
| R.0 | Gaming Taxonomy | 3.2/4 | Conditional Accept |
| R.1 | Parameter Gaming | 3.6/4 | Conditional Accept (strong) |
| R.2 | Input Data Manipulation | 3.0/4 | Conditional Accept |
| R.3 | Geographic Gaming | 3.2/4 | Conditional Accept |
| R.4 | Audit Chain Defense | 3.8/4 | Conditional Accept (strong) |
| **Track Mean** | | **3.4/4** | |

## Module Score: 8.4/10

## Track-Level Strengths
- R.4's Daubert analysis (all four criteria satisfied) and model DIA statutory language are immediate legal deliverables. This is the most practitioner-ready paper in the track.
- R.1's Gaming Impossibility Theorem (Theorem 3.1 in R.0 cross-reference) is the track's strongest theoretical contribution: it formally bounds the maximum manipulation under DIA pre-registration constraints.
- R.3's synthetic poisoned-adjacency experiment (50 strategically removed edges = 0.15 seat shift) is methodologically clean and the first empirical test of adjacency manipulation in the redistricting literature.

## P1 Items by Paper
- **R.2**: TLS-only (no certificate pinning) limitation is correctly disclosed, but the paper should quantify the attack surface: how many intermediaries could theoretically perform a MITM attack on a state's Census Bureau download? The answer (essentially zero, given PKI infrastructure) strengthens the claim.
- **R.0**: The Gaming Impossibility Theorem claims all vectors are bounded ≤ 0.5 seats combined, but this uses additive combination of bounds. If the vectors are correlated (e.g., resolution gaming and parameter gaming both move in the same direction in a specific state), the true bound could be higher. Address the correlation assumption.

## P2 Items
- R.3: Extend the poisoned-adjacency experiment to TX and PA to confirm state-level generalizability. (Karypis)
- R.2: Add a formal security definition for "audit chain integrity" (analogous to Merkle tree security definitions). (Rogaway)
- R.4: Note that the SHA-256 manifest approach is similar to software bill of materials (SBOM) requirements in Executive Order 14028 — this is a useful federal policy analogue. (Liang)

## Next Action
R.2 MITM quantification (P1) is a one-paragraph addition. R.0 vector correlation assumption (P1) is a footnote. Both are short prose fixes. The track is the program's most technically rigorous new contribution.
