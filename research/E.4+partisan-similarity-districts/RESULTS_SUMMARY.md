# Paper 17: Partisan Similarity Districts - Results Summary

**Date**: 2026-02-08
**Status**: Experiments complete, surprising negative results

## Executive Summary

**Surprising finding**: Edge-weighting partisan vote similarity has **minimal effect** on creating safe seats in large states. Geographic sorting already maximizes partisan homogeneity naturally.

## Experimental Design

**Method**: Edge-weighted graph partitioning using partisan vote similarity
- Edge weight: w = α if |lean_i - lean_j| < τ, else w = 1
- α (scaling): 1, 5, 10, 25, 50, 100
- τ (threshold): 10, 15, 20 percentage points

**Data**:
- 5 states: CA (52), TX (38), NY (26), PA (17), FL (28) = 161 districts
- 2020 presidential election results (83K block groups)
- 18 configurations tested

## Key Findings

### 1. Baseline is Already Very Safe
- **77% safe seats** (>10pp lean) without any edge-weighting
- **54.7% super-safe** (>20pp lean)
- Geographic sorting dominates in large states
- Urban/rural partisan divides create natural clustering

### 2. Edge-Weighting Doesn't Help
**Baseline (α=1) vs Extreme (α=100)**:
- Mean |lean|: 27.1pp → 27.3pp (+0.2pp)
- Safe >10pp: 77.0% → 76.4% (-0.6%)
- **Super-safe >20pp: 54.7% → 53.4% (-1.2%)** ⚠️ DECREASES
- Polsby-Popper: 0.255 → 0.179 (-30%) ⚠️ Significant loss

**Interpretation**: METIS edge-weighting cannot improve on natural geographic clustering. In fact, extreme weighting slightly reduces safe seats while destroying compactness.

### 3. Compactness-Homogeneity Trade-off Confirmed
- Strong weighting (α=50, α=100) reduces PP by 25-30%
- Minimal gain in partisan homogeneity
- **Trade-off is not worth it** in these states

### 4. Tau Threshold Has Little Effect
Holding α=10 constant:
- tau=10pp: 51.6% super-safe, PP=0.197
- tau=15pp: 50.3% super-safe, PP=0.210
- tau=20pp: 51.6% super-safe, PP=0.241

Threshold choice doesn't significantly affect outcomes.

## Comparison to California-Only Results

Previous CA-only tests showed:
- Baseline (α=1): 82.7% safe, PP=0.234
- Strong (α=50): 82.7% safe (same!), PP=0.179 (-23%)

**5-state aggregate shows similar pattern**:
- Edge-weighting has negligible effect on safe seats
- Compactness suffers significantly

## Implications for Paper

### Research Questions Answered

**RQ1: Can algorithmic safe districts exceed gerrymanders?**
- **No** - Geographic sorting already maximizes homogeneity
- Edge-weighting cannot improve on natural clustering
- May actually reduce safe seats in extreme cases

**RQ2: What's the trade-off?**
- **Significant compactness loss** (25-30% reduction in PP)
- **Minimal homogeneity gain** (0-2% more super-safe seats)
- Trade-off heavily favors baseline (no weighting)

**RQ3: Comparison to VRA edge-weighting?**
- VRA weighting (racial demographics) may be more effective
- Partisan clustering already happens naturally
- Racial minority representation requires intentional clustering

**RQ4: Is this "gerrymandering"?**
- **Moot point** - method doesn't work well enough to matter
- Even if it worked, algorithmic transparency still applies

**RQ5: Voter preference?**
- Most voters already live in safe districts (77%)
- Geographic sorting reflects residential choices
- Policy implications: redistricting reform may have limited impact

## Paper Narrative Implications

This is actually a **more interesting story** than if edge-weighting had worked:

1. **Geographic sorting dominates** - Redistricting algorithms can't overcome fundamental residential patterns
2. **Reform limitations** - Even "fair" algorithms produce safe seats due to geography
3. **Gerrymandering necessity questioned** - If 77% of districts are naturally safe, why gerrymander?
4. **Compactness-homogeneity conflict** - Attempting to increase homogeneity destroys compactness for minimal gain

## Recommended Paper Framing

**Original framing**: "Algorithmic approach to creating safe seats"
**New framing**: "Why algorithmic partisan clustering fails: Geographic sorting dominates redistricting"

### New Abstract Focus
- Geographic sorting creates 77% safe seats naturally
- Edge-weighted algorithms cannot improve on this baseline
- Attempting to increase homogeneity destroys compactness (-30%)
- Implications for redistricting reform: geography constrains outcomes

### New Contribution
- **Negative result**: Demonstrates limits of algorithmic redistricting
- **Policy insight**: Geographic sorting, not gerrymandering, drives safe seats
- **Methodological lesson**: Edge-weighting effectiveness depends on baseline clustering

## Next Steps

1. **Reframe paper** - Lead with negative result, emphasize geographic sorting
2. **Add comparison** - Compare to enacted maps (do they deviate from geographic baseline?)
3. **Theory section** - Why geographic sorting creates optimal partisan clustering
4. **Discussion** - Implications for reform (can't overcome geography)
5. **Consider smaller states** - Test hypothesis in less polarized/sorted states

## Files Generated

- `analysis/aggregate_metrics.csv` - Full results across 18 configs
- `analysis/summary_table.csv` - Key metrics for paper
- `analysis/summary_table.tex` - LaTeX table
- `analysis/tradeoff_curve.png` - Compactness vs homogeneity
- `analysis/safe_seats_by_alpha.png` - Safe seats by α value
