# Algorithmic Congressional Redistricting via Edge-Weighted Recursive Bisection

This paper combines and extends the work from Papers 1 and 2, presenting a unified treatment of recursive bisection for congressional redistricting with edge-weighted enhancements.

## Abstract

Congressional redistricting in the United States faces persistent challenges from gerrymandering and public distrust. While the Huntington-Hill method resolved seat apportionment disputes through mathematical objectivity, boundary drawing remains subject to partisan manipulation. We present an algorithmic approach using edge-weighted recursive bisection—a graph partitioning method that directly minimizes district perimeter by incorporating actual boundary lengths as edge weights. Applied to all 435 congressional districts across 50 states using 2020 Census data, our approach achieves a mean Polsby-Popper compactness score of 0.367—a 56% improvement over unweighted baseline recursive bisection (0.235) and 20% better than enacted 2020 congressional districts (0.305).

## Key Contributions

1. **Complete algorithmic framework**: End-to-end system from census data to final districts
2. **Edge-weighted enhancement**: Novel application of boundary length weighting via METIS
3. **National-scale validation**: Comprehensive 50-state comparison showing 37 of 50 states exceed enacted district compactness
4. **Baseline comparison**: Rigorous analysis against enacted 2020 districts using official Census boundaries
5. **Open-source implementation**: Reproducible pipeline with documented methodology

## Key Results

**National Compactness (Mean Polsby-Popper):**
- Enacted 2020 districts: 0.305
- Baseline recursive bisection: 0.235 (-23% vs enacted)
- Edge-weighted recursive bisection: 0.367 (+20% vs enacted, +56% vs baseline)

**State-Level Performance:**
- 37 of 50 states exceed enacted district compactness
- 43 of 50 states improve over baseline with edge-weighted mode
- Top improvements over enacted:
  - Illinois: +174.8% (0.406 vs 0.148 enacted)
  - Louisiana: +104.0% (0.294 vs 0.144 enacted)
  - New Hampshire: +101.7% (0.321 vs 0.159 enacted)
  - Texas: +85.7% (0.350 vs 0.189 enacted)
  - Massachusetts: +83.6% (0.398 vs 0.217 enacted)

## Structure

The paper is organized into 8 sections:

1. **Introduction**: Problem motivation, algorithmic approach, key findings
2. **Background**: Related work, Huntington-Hill analogy, METIS graph partitioning
3. **Methodology**: Baseline and edge-weighted recursive bisection algorithms
4. **Implementation**: Data sources, software architecture, computational performance
5. **Results**: National and state-level compactness comparison across three methods
6. **Analysis**: Political characteristics, partisan neutrality, demographic representation
7. **Discussion**: Algorithmic transparency, when algorithms excel vs human expertise
8. **Conclusion**: Implications for redistricting reform, future directions

## Compilation

Run the compile script:

```bash
# Linux/Mac
./compile.sh

# Windows
compile.bat
```

This will generate `recursive_bisection_with_edge_weighted_cuts.pdf`.

## Figures

The paper includes four main figures from the three-way comparison analysis:

1. **National comparison bar chart** (`figures/national_comparison_bar.png`): Shows mean Polsby-Popper scores for baseline (0.235), enacted (0.305), and edge-weighted (0.367) with percentage improvements annotated
2. **State scatter plot** (`figures/state_scatter.png`): Edge-weighted vs enacted compactness by state, with 37 of 50 states above diagonal
3. **Improvement distribution histograms** (`figures/improvement_distribution.png`): Distribution of compactness gains for edge-weighted vs baseline (left) and edge-weighted vs enacted (right)
4. **Top/bottom states comparison** (`figures/top_bottom_states.png`): Grouped bar charts showing the 10 best and 10 worst performing states relative to enacted districts

## Data Sources

- **Population**: 2020 Census PL 94-171 redistricting data (84,414 census tracts nationwide)
- **Geographic boundaries**: Census TIGER/Line shapefiles (2020 vintage)
- **Enacted districts**: Census TIGER/Line 118th Congressional Districts (2023)
- **Political data**: MIT Election Data + Science Lab 2020 presidential results at tract level

## Relationship to Papers 1 and 2

**Paper 1** (Introducing Recursive Bisection to Redistricting):
- Establishes baseline recursive bisection algorithm
- Shows that algorithmic redistricting can satisfy constitutional requirements
- Demonstrates partisan neutrality (zero political input)
- Result: 0.235 mean PP score, 23% below enacted districts

**Paper 2** (Edge-Weighted Recursive Bisection for Compact Congressional Redistricting):
- Introduces boundary length edge weights for compactness optimization
- Proves that perimeter minimization directly optimizes Polsby-Popper metric
- Achieves 56% improvement over baseline, 20% over enacted
- Result: 0.367 mean PP score, exceeds enacted districts

**Paper 3** (This paper):
- Integrates both approaches into unified presentation
- Shows natural progression from baseline to edge-weighted enhancement
- Provides comprehensive three-way comparison (baseline vs edge-weighted vs enacted)
- Positions edge-weighted mode as viable path toward algorithmic redistricting adoption

## Status

**COMPLETE** - Full 50-state analysis completed with comprehensive three-way comparison. Paper integrates best content from Papers 1 and 2 into cohesive presentation suitable for publication or policy analysis.

## Citation

```bibtex
@article{dellalibera2026recursive,
  title={Algorithmic Congressional Redistricting via Edge-Weighted Recursive Bisection},
  author={Della-Libera, Giovanni},
  journal={Working Paper},
  year={2026},
  month={January}
}
```

## Future Work

- Multi-objective optimization incorporating communities of interest
- Block-level data (11M blocks vs 85K tracts)
- Longitudinal analysis across multiple census cycles
- Legal evaluation for Voting Rights Act compliance
- Alternative partitioning schemes (k-way METIS, region-growing)
