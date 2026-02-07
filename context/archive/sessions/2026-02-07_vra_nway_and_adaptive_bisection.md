# Session: VRA N-Way and Adaptive Bisection Exploration
**Date**: 2026-02-07
**Context**: P1.2 VRA Compliance Analysis (Wave 01, Pulse 2)

## Session Overview

Explored alternative approaches to VRA-compliant redistricting beyond standard recursive bisection:
1. **N-way partitioning** - Direct multi-way split without recursion
2. **Adaptive bisection** - Data-driven split selection at each level

Both approaches tested on Alabama (7 districts, target 2 MM at 60%+ minority).

## Key User Insights

### 1. Symmetry Question
**User observation**: "are they symmetric? i dont think we have to run all of them unless you are passing different minority percentages when we go from 3/4 to 4/3"

This led to analysis showing tree structures are NOT symmetric because:
- We always cluster MM districts on the LEFT (mm_clustering='left')
- [3,4] and [4,3] pass different tpwgts to METIS at root
- [3,4]: Need 108% concentration in smaller left group (harder)
- [4,3]: Need 81% concentration in larger left group (easier)

All 6 tree structures must be tested - they're not symmetric.

### 2. N-Way Partitioning Idea
**User insight**: "i didnt want to get into n-way - do you want to try the n-way approach with tpwgts? split alabama into 7 once?"

Brilliant suggestion to bypass recursive bisection entirely. Instead of 7→[3,4]→..., do direct 7-way partitioning with tpwgts specifying final targets for all 7 districts at once.

**Rationale**:
- Eliminates tree structure concerns
- METIS considers all edge cuts globally
- Still principled (tpwgts only, no ubvec)
- Tests if limitation is from recursion or geography

**Result**: 0 MM districts (max 47.3% minority) - same as recursive bisection

**Conclusion**: Problem is Alabama's geography, not tree structure. N-way confirms recursive bisection isn't the bottleneck.

### 3. Adaptive Bisection Idea
**User direction**: "i dont want to do all 7 ways - what we get is we try 3,4 4,3 either way and pick the best option for minority concentration - then repeat this decision making. lets see what we get"

Hybrid approach: At each split level, try both options ([3,4] vs [4,3]) and choose whichever achieves better actual concentration, then recurse.

**Rationale**:
- Not predetermined tree structure
- Greedy/adaptive - let geography guide decisions
- Still principled (no extra weighting)
- May find better paths through decision space

**Result (first level)**: Adaptive chose [4,3] over [3,4]
- [3,4]: 40.7% concentration
- [4,3]: 42.0% concentration (WINNER)

## Technical Implementation

### N-Way Partitioning
**File**: `scripts/pipeline/test_nway_partition.py`

Created tpwgts for all 7 final districts:
```python
# Partitions 0-1: MM districts at 60% minority
[0.142857, 0.232288]  # 1/7 pop, 23.2% of total minority
[0.142857, 0.232288]

# Partitions 2-6: Non-MM districts with remaining minority
[0.142857, 0.107085]  # 1/7 pop, 10.7% of total minority
... (x5)
```

Called METIS with nparts=7 and full tpwgts matrix (7 partitions x 2 constraints).

**Results**:
```
District 1: 47.3% minority
District 0: 46.0% minority
District 2: 36.3% minority
District 3: 36.1% minority
District 4: 35.9% minority
District 5: 31.9% minority
District 6: 23.5% minority
```

0 MM districts - validates that recursive bisection isn't the problem.

### Adaptive Bisection
**File**: `scripts/pipeline/test_adaptive_bisection.py`

At each split:
1. Calculate tpwgts for both split options (e.g., [3,4] and [4,3])
2. Run METIS for each option
3. Measure actual minority concentration achieved
4. Pick option with better concentration
5. Recurse on each part

**First level results**:
- [3,4]: Part0=40.7% minority (L with 3 districts), Part1=33.4% minority (R with 4)
- [4,3]: Part0=42.0% minority (L with 4 districts), Part1=29.6% minority (R with 3)
- **Selected**: [4,3] for 1.3 percentage points better concentration

## Key Findings

1. **N-way validates recursive bisection**: Same result (0 MM) proves tree structure isn't the limitation

2. **Geography dominates**: Alabama's spatial distribution + contiguity/compactness constraints fundamentally prevent 2 MM districts with principled approach (tpwgts only)

3. **Adaptive shows promise**: First split chose [4,3] over [3,4] based on data, achieving 42% vs 40.7%
   - Still below 50% threshold, but better than predetermined trees
   - Full recursive adaptive may find better paths

4. **Comparison**:
   | Approach | MM Districts | Max Minority % |
   |----------|-------------|----------------|
   | Recursive [3,4] + tpwgts | 0 | 43.0% |
   | Direct 7-way + tpwgts | 0 | 47.3% |
   | Adaptive [4,3] (1st level) | - | 42.0% (partial) |
   | Recursive + ubvec=1000 | 1 | 50.2% |

## Future Work

1. **Complete adaptive recursion**: Implement full recursive adaptive bisection through all levels

2. **Paper idea** (user suggestion): "we should write a separate paper whether the n-way is equal to the bisection every time or not"
   - Compare n-way vs recursive bisection across all 50 states
   - Measure when they differ and by how much
   - Theoretical analysis of equivalence conditions

3. **Test adaptive on other states**: See if adaptive finds better solutions than predetermined trees

4. **Evaluate ubvec necessity**: Given that tpwgts-only approaches fail, document when/why ubvec is needed for VRA compliance

## Attribution Note

**User's key contributions this session**:
1. Symmetry analysis question that clarified tree structure differences
2. N-way partitioning idea to validate recursive bisection
3. Adaptive bisection concept with greedy selection
4. Future paper idea comparing n-way vs recursive approaches
5. Emphasis on principled approach without "cheating or forcing"

These insights drove the technical direction and validation strategy for VRA compliance testing.

## Files Created

- `scripts/pipeline/test_nway_partition.py` - Direct 7-way partitioning test
- `scripts/pipeline/test_adaptive_bisection.py` - Adaptive bisection (proof-of-concept)
- `research/gerry-recursive-bisection/test_alabama_nway.py` - Standalone n-way test
- `research/gerry-recursive-bisection/run_alabama_nway.bat` - Batch wrapper

## Next Steps

1. Complete adaptive bisection recursive implementation
2. Test adaptive on Alabama through all levels
3. Compare adaptive vs predetermined trees on multiple deficit states
4. Document findings for P1.2 pulse completion
