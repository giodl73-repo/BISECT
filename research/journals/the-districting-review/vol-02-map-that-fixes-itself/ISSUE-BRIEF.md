---
journal: The Districting Review
volume: 2
title: "The Map That Fixes Itself"
status: seed
arc-position: 6 of 10 — shows the output
reader-promise: "When you run the algorithm on your state's Census data, here is exactly what you get — NC 7D/7R, 223D/209R nationally, plans at the compactness extremum of the ensemble, and a reproducible result any party can verify."
target-reader: Commissioners and staff who want to see the actual output before committing
excluded-claims: No adoption mechanics. No statistical evidence framing (that's The Evidence Standard).
terminal-connection: Showing the concrete output builds the intuition that the algorithm is a well-behaved tool, not a black box — which is required before the commissioner (or legislator) votes to mandate it.
---

## Reader Promise (expanded)

Abstract arguments about algorithmic neutrality are necessary but not sufficient. Commissioners want to see the map. This volume shows it: the 2020 algorithmic plan for North Carolina produces seven Democratic and seven Republican seats without accessing partisan data. The national plan produces 223 Democratic and 209 Republican seats — compared to 212D/222R enacted. The plans are at the compactness extremum of the GerryChain ensemble in four of five study states. And every result is reproducible from the public SHA-256 manifest in under 20 minutes.

## Slots (6-8 candidates, 2-3x pool)

| Slot | Role | Claim class | Candidate papers |
|------|------|-------------|-----------------|
| Editorial | "Let me show you what it makes" — the demonstration posture | interpretive | — |
| Formula | The ApportionRegions result — prime-factor bisection, NC 7D/7R | empirical | B.11 (ApportionRegions) |
| National picture | 223D/209R nationally — compactness and partisan outcome together | empirical | B.11 §national, A.0 §finding-4 |
| Ensemble position | Plans at 0.1-0.7th compactness percentile — more compact than 99%+ of valid plans | empirical | G.1 (GerryChain comparison) |
| Reproducibility | Any party can verify — bisect label-verify, SHA-256, 20-minute replication | operational | A.4 (replication package), R.4 |
| VRA | 137 majority-minority districts vs. 68 enacted — neutrality improves minority representation | empirical | D.1, A.0 §finding-2 |
| Stability | The same algorithm applied to 2000/2010/2020 produces consistent results | empirical | C.2 (cross-census validation), B.15 |

## Review lenses

- HERALD: The NC 7D/7R number is the front door — state it in the editorial opening
- LOKI: G.1 ensemble percentiles are preliminary (ESS≈70 at 1K steps) — use the ESS-corrected language from S.1
- CUSTODIAN: G.1 percentile claims must use corrected S.1 framing, not raw percentile
