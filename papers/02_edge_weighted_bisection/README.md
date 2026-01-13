# Edge-Weighted Recursive Bisection for Congressional Redistricting

This paper presents an enhancement to the baseline recursive bisection algorithm that uses actual boundary lengths as edge weights to minimize total district perimeter and improve compactness.

## Key Contributions

- Edge-weighted graph partitioning using METIS with boundary length minimization
- Significant compactness improvements: +52.8% Polsby-Popper score for Alabama test case
- 22.2% reduction in total district perimeter
- Maintains equal population and contiguity constraints

## Compilation

Run the compile script:

```bash
# Linux/Mac
./compile.sh

# Windows
compile.bat
```

This will generate `edge_weighted_bisection.pdf`.

## Results Summary

**Alabama Test Case (7 districts):**
- Normal mode: 0.218 Polsby-Popper score, 7,389 km perimeter
- Edge-weighted mode: 0.334 Polsby-Popper score, 5,751 km perimeter
- Improvement: +52.8% compactness, -22.2% perimeter (1,638 km saved)
- 75.9% of tracts reassigned (1,091/1,437)

## Status

**In Progress** - Initial implementation complete, full 50-state analysis pending
