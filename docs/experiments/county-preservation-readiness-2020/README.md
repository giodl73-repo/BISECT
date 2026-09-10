# County preservation readiness: first diagnostic

Status: diagnostic complete; county sweep not run.

This is W0 of the [rule-development roadmap](../../specs/2026-09-09-districting-rule-development-roadmap.md).
It summarizes existing governed 2020 DFS census rows, without generating maps.

| Measurement | Nodes |
|---|---:|
| Total recursive cuts | 385 |
| Multiple minimum-population-deviation candidates before boundary scoring | 287 |
| Boundary scoring reduced the candidate count | 71 |
| Multiple physical partitions after boundary scoring | 0 |

The 287 count includes possible label-orientation duplicates. It is not a
count of distinct physical alternatives. The prior census does not expose
physical partition diversity before boundary scoring. All 385 observed winners
being unique therefore does not establish that county weights cannot matter.
No county reduction or chosen alpha is established here.

Next: instrument physical alternatives before boundary scoring, preserving
the existing candidate order and assignments. Then run the declared weight
response experiment under a separately frozen manifest and budget.

Profile inventory: NRS v0.3 uses geographic shared-boundary weights and the
canonical DFS/fallback construction. The official proposal uses
apportion-regions, county weights with additive alpha 2 (multiplier 3), and
convergence search. These are different experimental configurations. T.3's
historical sweep must be verified separately before transporting its claims.

Run from the repository root with an unused output path:

```powershell
py scripts/research/test_county_preservation_readiness.py
py scripts/research/analyze_county_preservation_readiness.py --out target/county-readiness-replay.json
```

Compare the regenerated JSON byte-for-byte with `analysis.json`. The analyzer
uses standard Python, requires no M: data, refuses to overwrite output, and
binds its own source and all inspected inputs by SHA-256. Those bindings
identify the inspected files; they do not independently replay the underlying
national experiment. Six tests cover valid data, missing/duplicate nodes,
invalid counts, non-preserving rows, and changed State schedules.
