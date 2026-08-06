# NRS 2000–2010 National Comparison

Both historical Census cycles now have independently verified, wall-to-wall
national reference baselines. This comparison is an aggregate completion and
proof-coverage audit, not a district-shape stability analysis.

| Measure | Census 2000 | Census 2010 |
|---|---:|---:|
| Frozen profile | NRS v0.3 | NRS v0.2 |
| States verified | 50 | 50 |
| Districts | 435 | 435 |
| Recursive nodes | 385 | 385 |
| Blocks | 8,199,908 | 11,071,790 |
| Population | 280,849,847 | 308,143,815 |
| Recorded State seconds | 2,667.114305 | 3,199.537400 |
| Population tolerance | 385/385 | 385/385 |
| Arithmetic-floor proof | 2/385 | 7/385 |
| Boundary proof | 0/385 | 0/385 |
| Canonical proof | 0/385 | 0/385 |

The profiles differ only because cycle-specific national failures required
versioned deterministic refinements: California motivated v0.2 in 2010, and
Hawaii's bridge structure motivated v0.3 in 2000. NRS v0.3 preserves the v0.2
seed stream and evaluates v0.1 and v0.2 first, but it has not yet been rerun
against the 2010 inventory. Therefore this report does not claim byte-identical
cross-cycle algorithm profiles or tree/cut stability.

The next comparison work is node- and district-level tree/cut stability and a
three-decade audit including the already verified 2020 baseline.
