# Research Program Architecture - 10-Paper Portfolio

## Figure 1: Research Program Dependency Graph

```mermaid
graph TB
    %% Foundation Papers
    P1["Paper 1: Recursive Bisection<br/>Foundation Method<br/>→ Baseline algorithm"]
    P2["Paper 2: Edge-Weighted Bisection<br/>Key Innovation<br/>→ Compactness via edge cuts"]

    %% Empirical Papers
    P3["Paper 3: VRA Compliance<br/>+69 MM Districts<br/>→ Exceeds enacted maps"]
    P4["Paper 4: Threshold Analysis<br/>42% Demographic Threshold<br/>→ Geographic feasibility"]
    P5["Paper 5: Cross-Census Validation<br/>80% Tract Retention<br/>→ Temporal stability"]

    %% Technical Papers
    P6["Paper 6: Multi-Constraint Conflicts<br/>VRA-Compactness Tradeoffs<br/>→ Pareto frontiers"]
    P7["Paper 7: Adaptive Parameter Selection<br/>Robustness Analysis<br/>→ Parameter sensitivity"]

    %% Comparison Papers
    P8["Paper 8: N-Way vs Recursive<br/>Method Equivalence<br/>→ Algorithmic convergence"]
    P9["Paper 9: Temporal Stability<br/>Hierarchical Advantage<br/>→ 14pt retention gain"]
    P10["Paper 10: Compactness Tradeoffs<br/>VRA-Geometry Balance<br/>→ Policy guidance"]

    %% Synthesis
    P11["Paper 11: Synthesis Metapaper<br/>Science/Nature Target<br/>→ Paradigm shift narrative"]

    %% Dependencies
    P1 --> P2
    P1 --> P3
    P1 --> P8
    P2 --> P3
    P2 --> P4
    P2 --> P5
    P2 --> P6
    P2 --> P7
    P2 --> P10
    P3 --> P4
    P3 --> P6
    P3 --> P10
    P4 --> P6
    P5 --> P9
    P6 --> P10
    P8 --> P9

    %% All papers feed synthesis
    P1 --> P11
    P2 --> P11
    P3 --> P11
    P4 --> P11
    P5 --> P11
    P6 --> P11
    P7 --> P11
    P8 --> P11
    P9 --> P11
    P10 --> P11

    %% Styling
    classDef foundation fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    classDef empirical fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px
    classDef technical fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef comparison fill:#f3e5f5,stroke:#6a1b9a,stroke-width:2px
    classDef synthesis fill:#ffebee,stroke:#c62828,stroke-width:3px

    class P1,P2 foundation
    class P3,P4,P5 empirical
    class P6,P7 technical
    class P8,P9,P10 comparison
    class P11 synthesis
```

## Paper Categories

### Foundation (Blue)
- **P1: Recursive Bisection** - Establishes baseline method
- **P2: Edge-Weighted Bisection** - Core innovation (compactness optimization)

### Empirical (Green)
- **P3: VRA Compliance** - 101% increase in MM districts
- **P4: Threshold Analysis** - 42% feasibility threshold discovery
- **P5: Cross-Census Validation** - Temporal consistency across decades

### Technical (Orange)
- **P6: Multi-Constraint Conflicts** - VRA-compactness Pareto frontiers
- **P7: Adaptive Parameter Selection** - Robustness and sensitivity analysis

### Comparison (Purple)
- **P8: N-Way vs Recursive** - Method equivalence validation
- **P9: Temporal Stability** - 14pt hierarchical advantage
- **P10: Compactness Tradeoffs** - Policy-relevant tradeoff analysis

### Synthesis (Red)
- **P11: Metapaper** - Science/Nature submission synthesizing all findings

## Key Findings by Paper

| Paper | Core Finding | Metric |
|-------|-------------|--------|
| P1 | Baseline feasibility | 435 districts, 100% contiguous |
| P2 | Compactness improvement | 56% vs unweighted baseline |
| P3 | VRA surplus | +69 MM districts (+101%) |
| P4 | Feasibility threshold | 42% demographic threshold |
| P5 | Temporal stability | 80% tract retention |
| P6 | Constraint conflicts | Pareto frontiers quantified |
| P7 | Parameter robustness | Adaptive selection validated |
| P8 | Method equivalence | N-way ≈ recursive convergence |
| P9 | Hierarchical advantage | 14pt retention over n-way |
| P10 | Policy tradeoffs | VRA-compactness guidance |
| P11 | Paradigm shift | Algorithmic objectivity at scale |

## Research Program Breadth

**Venues**: APSR (political science), KDD (algorithms), JOP (policy), PLDI (compilers), Science (interdisciplinary)
**Methods**: Graph partitioning, computational geometry, statistical analysis, legal analysis
**Contributions**: Technical (algorithms), Empirical (VRA findings), Theoretical (impossibility defense), Policy (actionable guidance)
**Scale**: 50 states × 3 census years = 1,305 districts analyzed

