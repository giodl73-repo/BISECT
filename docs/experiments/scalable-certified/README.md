# Rhode Island Scalable Certification Frontier

This directory records the first State-scale discovery and compact proof-model
packages.

## Discovery

Thirty-two deterministic METIS seeds were screened with exact post-validation:
13 produced connected population-floor candidates and 19 were rejected. Seed 4
was then fully refined and exactly rescored:

- populations: 548,689 / 548,690;
- scaled deviation: 1;
- weighted cut: 43,047,238; and
- population proof status: verified UNSAT below 1.

## Compact Proof Models

All three decision models compile from the same instance/discovery identities:

| Stage | Variables | Constraints | Size |
|---|---:|---:|---:|
| Population | 1,228,520 | 1,468,963 | 171 MB |
| Boundary | 1,228,520 | 1,468,964 | 172 MB |
| Boundary, right pop. 548,689 | 1,228,520 | 1,468,963 | 172 MB |
| Boundary, right pop. 548,690 | 1,228,520 | 1,468,963 | 172 MB |
| Canonical | 1,264,557 | 1,577,074 | 176 MB |

The files remain under ignored `data/` custody. Their hashes and model metadata
are committed in `ri-model-frontier.json`.

## Probe Result

RoundingSat proves the population lower bound and VeriPB accepts the proof.
The 64,132,468 exact-population branches reached `TIMELIMIT` after 600 seconds
and are superseded. The current 43,047,238 branches are compiled and unsearched.
No boundary or canonical conclusion is claimed.

## Solver Frontier

`solver-frontier.json` records the unrestricted root-free comparison:

- RoundingSat with and without LP;
- Exact 2.2.x;
- verified adder OPB-to-CNF translation;
- Kissat 4.0.4; and
- CaDiCaL 3.0.0.

Every solver returned timeout or unknown. Solver substitution without additional
mathematical decomposition is therefore closed as an active path.

## Regional Decomposition

`regional-decomposition-frontier.json` records two proof-safe splits:

- five exhaustive county-population branches; and
- a pure-tract central/complement split inside Providence County.

True variable elimination reduces the central models to 56,932 and 23,669
variables respectively. Both still reach `TIMELIMIT`, but the branch contracts
are exact and reusable for the next decomposition level.

## Claim Boundary

The State-scale compiler works. No Rhode Island proof has yet been generated.
