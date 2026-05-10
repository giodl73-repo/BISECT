---
journal: The Districting Review
volume: 2
title: "The Map That Fixes Itself"
status: provisional-public-preview
arc-position: 6 of 10 -- shows the output
reader-promise: "When you run the algorithm on public Census data, what outputs does it produce, how reproducible are they, and which headline numbers survive source-chain review?"
target-reader: Commissioners and staff who want to see the actual output before committing
excluded-claims: No adoption mechanics. No statistical evidence framing (that's The Evidence Standard). No universal court result.
terminal-connection: Showing concrete output builds practitioner intuition before any adoption vote or legal mandate is considered.
---

> Audition status: all map, partisan, compactness, VRA, and runtime numbers are candidates until source-chain review confirms inputs, commands, and versions.

## Reader Promise (expanded)

Abstract arguments about algorithmic neutrality are necessary but not sufficient. Commissioners want to see the map. This volume auditions the concrete-output story: North Carolina, the national plan, ensemble position, VRA mode, and cross-census stability. Each number needs a source-chain note that names the run, data version, command, and caveat.

## Slots (6-8 candidates, 2-3x pool)

| Slot | Role | Claim class | Candidate papers |
|------|------|-------------|-----------------|
| Editorial | "Let me show you what it makes" -- the demonstration posture | interpretive | -- |
| Formula | The ApportionRegions result -- prime-factor bisection, NC 7D/7R | empirical | T.4 (ApportionRegions) |
| National picture | National map outcome candidate -- compactness and partisan outcome together | empirical | T.4 section national, A.0 finding 4 |
| Ensemble position | Plans at a compactness percentile candidate -- more compact than most valid plans | empirical | G.1 (GerryChain comparison) |
| Reproducibility | Any party can verify -- bisect label-verify, SHA-256, 20-minute replication | operational | A.4 (replication package), R.4 |
| VRA | Majority-minority district counts under VRA mode vs. enacted maps -- scope and legal caveats required | empirical/legal | D.1, A.0 finding 2 |
| Stability | The same algorithm applied to 2000/2010/2020 produces consistent results | empirical | C.2 (cross-census validation), T.8 |

## Review lenses

- COMMONS: The NC result is the front door -- state the reproducible output in the editorial opening
- SCALE: G.1 ensemble percentiles are preliminary (ESS approx 70 at 1K steps) -- use the ESS-corrected language from S.1
- DATUM: G.1 percentile claims must use corrected S.1 framing, not raw percentile
