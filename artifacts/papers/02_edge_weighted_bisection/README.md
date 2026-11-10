# Edge-Weighted Recursive Bisection for Congressional Redistricting

This paper presents an enhancement to baseline recursive bisection that uses actual boundary lengths as edge weights to minimize total district perimeter and dramatically improve compactness.

## Key Contributions

- Edge-weighted graph partitioning using METIS with boundary length minimization
- **National-scale validation**: All 435 congressional districts across 50 states
- **Exceeds enacted districts**: +20% mean compactness over 2020 enacted plans (0.367 vs 0.305)
- **Dramatic improvement over normal mode**: +56% mean compactness (0.367 vs 0.235)
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

**National Scale (All 50 States, 435 Districts):**
- Normal mode: 0.235 mean Polsby-Popper
- Edge-weighted mode: 0.367 mean Polsby-Popper (+56%)
- Enacted 2020 districts: 0.305 mean Polsby-Popper
- **43 of 50 states** improved over normal mode
- **37 of 50 states** exceed enacted district compactness

**Top Improvements Over Enacted:**
- Illinois: +174% (0.406 vs 0.148 enacted)
- Louisiana: +104%
- New Hampshire: +102%
- Texas: +86%
- Massachusetts: +84%

**Largest Improvements Over Normal:**
- Hawaii: +177%
- Iowa: +164%
- Pennsylvania: +164%
- Indiana: +152%
- Kentucky: +142%

## Status

**COMPLETE** - Full 50-state analysis completed with comprehensive baseline comparison
