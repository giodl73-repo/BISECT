# Fixed-DFS root county-response diagnostic

Date: 2026-09-10. Development States only; this is a follow-up measurement to
the [county readiness audit](../county-preservation-readiness-2020/README.md),
not completion of the full-plan W1 bakeoff in the
[development roadmap](../../specs/2026-09-09-districting-rule-development-roadmap.md).

## Result

| State | Oriented population-minimum candidates | Distinct physical cuts before boundary scoring | Winner changed at tested weights |
|---|---:|---:|---|
| RI | 4 | 2 | No |
| IA | 4 | 2 | No |
| NC | 2 | 1 | No |
| TX | 2 | 1 | No |

All four roots reproduce the archived census's unit count and three existing
candidate counters. Tested additive alpha values: 0, 0.5, 1, 2, 4, 8.
Score = raw cut + alpha × within-county cut, computed using exact rational
arithmetic (not the official engine's floating-point/scaling implementation).
Cross-county edge costs are unchanged. Bridges and retained zero-population
blocks are included as present in the source graph.

NC and TX have no physical choice remaining at this selection stage. In RI and
IA the raw-boundary winner also has strictly lower within-county cut cost;
therefore no nonnegative additive alpha can reverse these two-candidate
rankings under this fixed-candidate score. This is a local algebraic result,
not a claim that complete plans or other search procedures are weight-insensitive.

The diagnostic mirrors the current selector's mark-on-push DFS construction,
sorted neighbors, subtree populations and population-first filter. Every proper
subtree excludes root zero; distinct subtree roots identify distinct physical
cuts, with complementary orientations counted once. It reports winning sets
and does not emulate METIS-dependent orientation/tie-breaking or downstream
repair. Counter agreement is a cross-check, not a full assignment replay.

## Implication for the next experiment

Retain the planned county-weight control arm, but include the separately labeled
expanded-search arm: additional predetermined roots or candidate trees, with
the same population policy and county-score schedule. Measure candidate diversity
and final county fragments separately. These results justify testing search
restriction; they do not justify silently relaxing population requirements or
claiming fewer county splits. Freeze executable manifests before full-plan runs.

## Reproduce

Inputs are read from M; the baseline corpus and production selector are not
modified. The JSON records SHA-256 hashes for all four contexts, the source
selector, this diagnostic, and the archived census. An existing output is refused.

```powershell
python -m unittest discover -s scripts/research -p test_dfs_county_response.py
python scripts/research/analyze_dfs_county_response.py --contexts M:\DATA_VAULT\projects\apportionment\derived-data\census\2020\certified --out target/dfs-root-county-response-new.json
```

Compare the generated file with [analysis.json](analysis.json). Five synthetic
tests cover complementary orientations, physical diversity, a county-weight
winner change including a tie, disconnected input, and noninteger weights.
This is descriptive root-only evidence: no final county-fragment, geometry,
partisan-neutrality, legal-compliance, or global-optimality claim.
