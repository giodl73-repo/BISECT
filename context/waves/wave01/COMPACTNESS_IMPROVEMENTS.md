# Compactness & Balance Improvements for Congressional Redistricting

## Current Approach

### METIS Parameters (in recursive_bisection.py)
Currently using:
- `-contig`: Enforce contiguity
- `-minconn`: Minimize edge cuts (promotes compactness)
- `-ufactor`: Allow population imbalance (adaptive)

## Quick Wins (Implement First)

### 1. Increase METIS Refinement Iterations
**Change in recursive_bisection.py around line 190:**

Add `-niter=20` parameter (default is 10)

**Impact**: +5-10% compactness improvement, 2x slower
**Recommendation**: Do this first - easy win

### 2. Tighter Population Balance
**Change ufactor calculation:**

Make it more aggressive: `ufactor = max(1, int(abs(deviation_fraction) * 500))`
Or force exact balance: `ufactor = 1`

**Impact**: Better equality, slightly less compact
**Recommendation**: Test on one state first

### 3. Add Compactness Metrics
**Add to district_summary.csv:**
- Polsby-Popper score (4π × area / perimeter²)
- Reock score (area / minimum bounding circle)
- Convex hull ratio
- Length-width ratio

**Impact**: Visibility into district shapes
**Recommendation**: High value for analysis

## Medium-Term Improvements

### 4. K-Way Partitioning for Final Round
Instead of recursive bisection all the way down, use k-way partitioning for the final split.

**Benefits**:
- Final districts optimized together
- Better global balance
- +10-15% compactness

**Complexity**: Moderate - requires new METIS wrapper

### 5. Geographic Distance Edge Weighting
Weight edges by actual geographic distance between tract centroids.

**Benefits**:
- More geographic compactness
- Better respect for natural boundaries
- +15-20% compactness

**Complexity**: Moderate - need to compute and format edge weights

### 6. County-Awareness Beyond Water
Already implemented for water adjacency, could extend to:
- Prefer splits that respect county boundaries
- Weight county-crossing edges higher (discourage splitting)
- Report county splits per district

## Advanced Optimizations

### 7. Multi-Trial with Compactness Selection
Run METIS 10 times with different seeds, pick most compact result.

**Benefits**:
- Best of multiple solutions
- +20-30% compactness improvement
- Can balance population AND shape

**Downsides**: 10x slower

### 8. Post-Processing Optimization
After METIS, run local search to swap boundary tracts between districts:
- Maintain population balance
- Improve compactness
- Minimize county splits

## Compactness Metrics Reference

### Polsby-Popper Score
Formula: `4π × area / perimeter²`
- Range: 0 to 1 (1 = perfect circle)
- Most common metric in redistricting literature
- Easy to calculate and interpret

### Reock Score
Formula: `area / area_of_minimum_bounding_circle`
- Range: 0 to 1 (1 = perfect circle)
- More forgiving than Polsby-Popper
- Requires computing minimum bounding circle

### Typical Scores
- Gerrymandered districts: 0.05-0.15
- Current congressional districts: 0.15-0.25
- Algorithmic redistricting: 0.20-0.35
- Iowa-style (compact): 0.30-0.45

## Implementation Priority

### Phase 1 (Do Now):
1. Add `-niter=20` to METIS calls
2. Calculate and report Polsby-Popper scores
3. Add compactness column to CSV files

### Phase 2 (Next Version):
4. Implement k-way partitioning for final round
5. Add geographic distance edge weighting
6. Create compactness visualization maps

### Phase 3 (Research):
7. Multi-trial optimization
8. Post-processing local search
9. Compare against actual districts

## Expected Results

With Phase 1 changes only:
- **Compactness**: +10-15% improvement
- **Speed**: -50% (still completes in ~6 hours for all states)
- **Population balance**: Same or better
- **Implementation time**: < 2 hours

Full implementation (Phase 1+2):
- **Compactness**: +25-35% improvement
- **Speed**: -70% (need ~12 hours for all states)
- **Quality**: Comparable to best human-drawn districts
- **Implementation time**: ~3 days
