# Y.4 Biological Network Analogues For Spatial Districting

Goal: translate biological network formation analogies into measurable,
auditable graph features for redistricting research.

## Analogy Boundary

The analogies in this track are design metaphors for graph weighting. They do
not prove legal fairness, voter behavior, or community identity.

Valid use:

```text
biological system -> measurable graph principle -> BISECT fixture test
```

Invalid use:

```text
biological system -> civic legitimacy claim
```

## Candidate Analogues

| System | Network Principle | BISECT Feature |
|---|---|---|
| Slime mold | Reinforce efficient paths under cost pressure. | Flow/mass-weighted local cohesion. |
| Spider web | Coverage plus redundancy under material limits. | Cycle support and bridge-likeness. |
| Roots | Branch toward resource density, prune weak paths. | Population-mass reinforcement with sparse-edge decay. |
| Veins | Preserve flow and redundancy through hierarchy. | Multi-scale module and factor tree support. |
| Rivers | Drainage hierarchy and low-resistance paths. | Separator and corridor detection. |
| Transit networks | Connect dense centers with resilient links. | Population-density mass plus edge betweenness diagnostics. |

## Translation Rule

Each analogue must produce:

1. a graph feature;
2. a weighting formula or diagnostic;
3. a synthetic fixture where expected behavior is obvious;
4. a non-claim statement limiting interpretation.

Example:

```text
spider web redundancy
-> short alternate path around edge
-> cycle_support(e)
-> mesh fixture keeps high-support edges intact
-> no claim that web behavior defines community
```

## First Experiment Set

1. Slime-mold fixture: dense source nodes with low-cost redundant paths.
2. Web fixture: local mesh with one sparse connector.
3. Root fixture: population mass gradient across an adjacency grid.
4. River fixture: separator corridor where cuts should concentrate.

## Acceptance Criteria

- Every biological analogy maps to an auditable graph feature.
- No analogy bypasses BISECT's mode controls.
- Results are compared to existing B/T/U construction methods.
- Public wording stays in research-hypothesis language.
