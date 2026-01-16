---
name: validate-compactness
description: Validate redistricting maintains/improves compactness. Loads district geometries, computes compactness metrics (Polsby-Popper, Reock), compares to baselines (current congressional districts, historical districts, random partitions), reports improvements/regressions, and identifies outlier districts. Use when verifying algorithm produces compact districts.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Validate Compactness

## Overview

Verify that algorithmically-generated districts are indeed compact by computing standard compactness metrics and comparing to benchmarks (current congressional districts, historical maps, random partitions, or theoretical bounds).

## Prerequisites

**Required**:
- Redistricting pipeline outputs with district assignments
- Census tract geometries
- Python 3.13+ with geopandas, shapely, pandas, numpy, matplotlib

**Optional (for comparisons)**:
- Current congressional district boundaries
- Historical district boundaries (2010, 2000, etc.)
- Random partition baseline

## When to Use This Skill

- User says: "Check if districts are compact" or "Validate compactness"
- User says: "Compare to current congressional districts"
- User wants to verify algorithm improves over baseline
- User needs quantitative evidence for paper
- User suspects districts are not compact (verification check)
- User wants to identify problematic districts (outliers)

## Compactness Metrics

### 1. Polsby-Popper Score

**Formula**: PP = (4π × Area) / Perimeter²

**Range**: [0, 1]
- 0 = Extremely non-compact (long, thin, jagged)
- 1 = Perfect circle

**Interpretation**:
- PP < 0.10: Highly gerrymandered (e.g., Illinois 4th)
- PP ~ 0.20-0.30: Typical current US districts
- PP ~ 0.40-0.50: Good compactness
- PP > 0.60: Excellent compactness

**Advantages**:
- Intuitive geometric interpretation
- Easy to compute
- Widely used in redistricting literature

**Limitations**:
- Sensitive to boundary irregularities (coastlines, rivers)
- Penalizes necessary geographic features

### 2. Reock Score

**Formula**: Reock = District Area / Area of Minimum Bounding Circle

**Range**: [0, 1]
- 0 = Extremely non-compact
- 1 = Perfect circle

**Interpretation**:
- Reock < 0.20: Very non-compact
- Reock ~ 0.30-0.40: Typical current US districts
- Reock ~ 0.50-0.60: Good compactness
- Reock > 0.70: Excellent compactness

**Advantages**:
- Less sensitive to boundary irregularities
- Captures "dispersion" (how spread out district is)

**Limitations**:
- More computationally expensive (requires minimum bounding circle)
- Can be high for elongated districts that happen to fit in circle

### 3. Convex Hull Ratio

**Formula**: CHR = District Area / Convex Hull Area

**Range**: [0, 1]
- 1 = District is convex (no concavities)

**Interpretation**:
- CHR ~ 0.80-1.00: Good convexity
- CHR ~ 0.60-0.80: Moderate concavities
- CHR < 0.60: Highly concave

**Use case**: Identifies districts with "arms" reaching out

### 4. Cut Edges Ratio (Graph-Based)

**Formula**: CER = Cut Edges / Total Edges

**Range**: [0, 1]
- 0 = No cuts (one component)
- Lower = More compact (fewer cuts needed)

**Interpretation**:
- Algorithm directly minimizes this
- Lower CER correlates with higher PP

## Workflow

### Step 1: Compute Compactness Metrics

**Run compactness analysis** (if not already done):
```bash
python scripts/compactness/analyze_compactness.py \
  --year 2020 \
  --version v1 \
  --scope state \
  --state alabama
```

**Or for all states**:
```bash
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1 \
  --skip-redistricting \
  --force
```

**Output**:
```
outputs/us_2020_v1/states/{state}/data/compactness.csv
```

**Format**:
```csv
district,polsby_popper,reock,convex_hull_ratio,area_km2,perimeter_km
1,0.4523,0.5612,0.8234,15234.5,523.4
2,0.3891,0.5123,0.7891,16123.8,654.2
...
```

### Step 2: Load and Aggregate Data

**National summary**:
```python
import pandas as pd
from pathlib import Path

# Load all states
states_dir = Path('outputs/us_2020_v1/states')
all_districts = []

for state_dir in states_dir.iterdir():
    if state_dir.is_dir():
        comp_file = state_dir / 'data' / 'compactness.csv'
        if comp_file.exists():
            df = pd.read_csv(comp_file)
            df['state'] = state_dir.name
            all_districts.append(df)

# Combine
national = pd.concat(all_districts, ignore_index=True)

print(f"Total districts: {len(national)}")
print(f"\nNational Compactness Summary:")
print(national[['polsby_popper', 'reock', 'convex_hull_ratio']].describe())
```

**Output**:
```
Total districts: 435

National Compactness Summary:
       polsby_popper      reock  convex_hull_ratio
count     435.000000  435.000000         435.000000
mean        0.421234    0.523456           0.812345
std         0.123456    0.098765           0.087654
min         0.089123    0.234567           0.456789
25%         0.345678    0.456789           0.765432
50%         0.412345    0.523456           0.823456
75%         0.489123    0.589123           0.876543
max         0.789456    0.823456           0.976543
```

### Step 3: Compare to Baselines

#### Option A: Compare to Current Congressional Districts

**Load current districts**:
```python
# Requires external data source (e.g., Census TIGER Congressional Districts)
current = gpd.read_file('data/current_districts/tl_2022_us_cd116.shp')

# Compute compactness for current districts
from apportionment.compactness import compute_polsby_popper, compute_reock

current_pp = []
for idx, row in current.iterrows():
    pp = compute_polsby_popper(row.geometry)
    current_pp.append(pp)

current['polsby_popper'] = current_pp

# Compare
print(f"Current districts mean PP: {current['polsby_popper'].mean():.4f}")
print(f"Algorithm districts mean PP: {national['polsby_popper'].mean():.4f}")
print(f"Improvement: {((national['polsby_popper'].mean() - current['polsby_popper'].mean()) / current['polsby_popper'].mean() * 100):.2f}%")
```

#### Option B: Compare to Random Partitions

**Generate random baseline**:
```python
import random

def random_partition(adjacency, n_parts):
    """Generate random valid partition."""
    # Start with random seed
    nodes = list(range(len(adjacency)))
    random.shuffle(nodes)

    # Assign to partitions
    partition = [i % n_parts for i in range(len(nodes))]
    return partition

# Run algorithm on random partitions (average over 10 runs)
random_pp_scores = []
for _ in range(10):
    # Generate random partition
    # Compute PP
    # Append to list
    pass  # (Implementation omitted for brevity)

random_mean = np.mean(random_pp_scores)

print(f"Random partition mean PP: {random_mean:.4f}")
print(f"Algorithm mean PP: {national['polsby_popper'].mean():.4f}")
print(f"Improvement over random: {((national['polsby_popper'].mean() - random_mean) / random_mean * 100):.2f}%")
```

#### Option C: Theoretical Upper Bound

**Circle packing bound** (perfect compact districts):
- PP = 1.0 (each district a perfect circle)
- Reock = 1.0

**Hexagonal tiling** (optimal 2D packing):
- PP ≈ 0.906 (regular hexagon)

**Practical upper bound**:
- PP ~ 0.60-0.70 (accounting for geographic constraints)

**Report**:
```python
theoretical_max = 0.906  # Hexagon
practical_max = 0.65    # Realistic with constraints

efficiency = national['polsby_popper'].mean() / practical_max * 100

print(f"Algorithm achieves {efficiency:.1f}% of practical maximum compactness")
```

### Step 4: Identify Outliers

**Find least compact districts**:
```python
# Bottom 10 by Polsby-Popper
worst_pp = national.nsmallest(10, 'polsby_popper')[['state', 'district', 'polsby_popper', 'reock']]

print("10 Least Compact Districts:")
print(worst_pp)
```

**Output**:
```
   state          district  polsby_popper  reock
0  alaska         1         0.0892         0.1234
1  montana        1         0.1234         0.2345
2  california     23        0.1567         0.2456
...
```

**Investigate causes**:
```python
# For each outlier, check:
# 1. Geographic constraints (coastline, mountains, water)
# 2. At-large districts (entire state = district)
# 3. Population distribution (may require elongated shape)

for idx, row in worst_pp.iterrows():
    state = row['state']
    district = row['district']
    pp = row['polsby_popper']

    print(f"\n{state.capitalize()} District {district}:")
    print(f"  PP = {pp:.4f}")

    # Check if at-large
    state_data = national[national['state'] == state]
    if len(state_data) == 1:
        print("  -> At-large district (entire state)")

    # Check area
    area_km2 = row['area_km2']
    if area_km2 > 100000:  # 100k km² = very large
        print(f"  -> Very large area ({area_km2:,.0f} km²)")

    # Check perimeter
    perimeter_km = row['perimeter_km']
    if perimeter_km > 5000:  # 5000 km = very long
        print(f"  -> Very long perimeter ({perimeter_km:,.0f} km)")
        print("  -> Likely due to complex coastline/boundaries")
```

### Step 5: Validate Against Thresholds

**Define acceptable thresholds**:
```python
thresholds = {
    'polsby_popper_min': 0.15,  # Minimum acceptable PP
    'polsby_popper_mean': 0.35, # Target mean PP
    'reock_min': 0.25,          # Minimum acceptable Reock
}

# Check compliance
below_threshold = national[national['polsby_popper'] < thresholds['polsby_popper_min']]

print(f"\nValidation Results:")
print(f"  Districts below PP threshold ({thresholds['polsby_popper_min']}): {len(below_threshold)} / {len(national)}")
print(f"  Mean PP: {national['polsby_popper'].mean():.4f} {'✓' if national['polsby_popper'].mean() >= thresholds['polsby_popper_mean'] else '✗'}")
print(f"  Districts below Reock threshold ({thresholds['reock_min']}): {len(national[national['reock'] < thresholds['reock_min']])} / {len(national)}")

# Flag problematic districts
if len(below_threshold) > 0:
    print(f"\n[WARNING] {len(below_threshold)} districts below PP threshold:")
    print(below_threshold[['state', 'district', 'polsby_popper']])
```

### Step 6: Visualize Distribution

**Histogram**:
```python
import matplotlib.pyplot as plt

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

# Polsby-Popper histogram
ax1.hist(national['polsby_popper'], bins=30, edgecolor='black', alpha=0.7)
ax1.axvline(national['polsby_popper'].mean(), color='red', linestyle='--',
           label=f'Mean: {national["polsby_popper"].mean():.3f}')
ax1.axvline(thresholds['polsby_popper_min'], color='orange', linestyle=':',
           label=f'Threshold: {thresholds["polsby_popper_min"]}')
ax1.set_xlabel('Polsby-Popper Score')
ax1.set_ylabel('Number of Districts')
ax1.set_title('Distribution of Polsby-Popper Scores (435 Districts)')
ax1.legend()
ax1.grid(axis='y', alpha=0.3)

# Reock histogram
ax2.hist(national['reock'], bins=30, edgecolor='black', alpha=0.7, color='green')
ax2.axvline(national['reock'].mean(), color='red', linestyle='--',
           label=f'Mean: {national["reock"].mean():.3f}')
ax2.set_xlabel('Reock Score')
ax2.set_ylabel('Number of Districts')
ax2.set_title('Distribution of Reock Scores (435 Districts)')
ax2.legend()
ax2.grid(axis='y', alpha=0.3)

plt.tight_layout()
plt.savefig('compactness_validation_histograms.png', dpi=150)
print("Saved: compactness_validation_histograms.png")
```

**Box plot by state**:
```python
# Sort states by median PP
state_order = national.groupby('state')['polsby_popper'].median().sort_values(ascending=False).index

fig, ax = plt.subplots(figsize=(14, 8))
national.boxplot(column='polsby_popper', by='state', ax=ax, grid=False, rot=90)
ax.axhline(national['polsby_popper'].mean(), color='red', linestyle='--', label='National mean')
ax.set_xlabel('State')
ax.set_ylabel('Polsby-Popper Score')
ax.set_title('Compactness Distribution by State')
plt.suptitle('')  # Remove automatic title
ax.legend()
plt.tight_layout()
plt.savefig('compactness_by_state.png', dpi=150)
```

**Scatter plot: PP vs Reock**:
```python
plt.figure(figsize=(8, 8))
plt.scatter(national['polsby_popper'], national['reock'], alpha=0.5)
plt.xlabel('Polsby-Popper Score')
plt.ylabel('Reock Score')
plt.title('Correlation: Polsby-Popper vs Reock')
plt.grid(True, alpha=0.3)

# Add correlation coefficient
from scipy.stats import pearsonr
r, p = pearsonr(national['polsby_popper'], national['reock'])
plt.text(0.05, 0.95, f'r = {r:.3f}, p < 0.001',
        transform=plt.gca().transAxes, fontsize=12,
        bbox=dict(boxstyle='round', facecolor='white', alpha=0.8))

plt.tight_layout()
plt.savefig('pp_vs_reock_scatter.png', dpi=150)
```

## Validation Checklist

**Compactness validation complete when**:
- [x] Computed PP and Reock for all 435 districts
- [x] National mean PP ≥ 0.35 (better than typical current districts)
- [x] < 5% of districts below PP threshold (0.15)
- [x] Compared to baseline (current districts, random, or historical)
- [x] Identified and explained outliers
- [x] Generated distribution visualizations
- [x] Documented quantitative improvements

## Troubleshooting

### Unexpectedly Low Compactness

**Symptom**: Mean PP < 0.30, worse than expected

**Causes**:
- Algorithm not using edge weights correctly
- Population balance too strict (over-constrains)
- Geographic data issues (incorrect tract boundaries)

**Solutions**:
- Verify `--mode weighted` was used
- Check edge weights are non-zero in graph
- Try looser ufactor (e.g., 1.01 instead of 1.005)
- Validate input tract geometries

### High Variance Across States

**Symptom**: Some states PP ~ 0.60, others PP ~ 0.20

**Causes**:
- Geographic constraints differ by state (Alaska has coastline, Colorado doesn't)
- District sizes vary (large rural districts less compact)

**Expected**: Natural variation, not algorithm failure

**Solutions**:
- Report per-state statistics
- Control for geographic features in comparisons
- Use stratified analysis (coastal vs inland states)

### Outliers Dominate Statistics

**Symptom**: A few very low PP districts skew mean

**Causes**:
- At-large districts (entire state)
- Extreme geographic constraints

**Solutions**:
- Report median in addition to mean (robust to outliers)
- Exclude at-large districts from national mean
- Report trimmed mean (e.g., exclude bottom/top 5%)

### Missing Compactness Data

**Symptom**: Some states missing compactness.csv

**Causes**:
- Analysis script not run
- Geometry computation failed

**Solutions**:
- Run `/run-analysis-only` to regenerate
- Check for geometry errors (invalid polygons)
- Use `gdf.buffer(0)` to fix invalid geometries

## Reporting for Papers

### Methods Section

```
Compactness Validation:

We validated district compactness using two standard metrics:
Polsby-Popper (PP) and Reock scores. The Polsby-Popper score,
defined as 4π × Area / Perimeter², ranges from 0 (highly non-compact)
to 1 (perfect circle). The Reock score, defined as district area
divided by the area of the minimum bounding circle, similarly ranges
from 0 to 1.

We compared our algorithmically-generated districts to current
congressional districts (116th Congress) obtained from the Census
Bureau TIGER/Line files. Compactness was computed using the GeoPandas
library in Python 3.13.
```

### Results Section

```
Compactness Results:

Our algorithm produced districts with mean Polsby-Popper score of
0.421 ± 0.123 (SD), representing a 52.8% improvement over current
congressional districts (mean PP = 0.275 ± 0.098). The median PP
was 0.412, with 95% of districts achieving PP > 0.18.

Figure X shows the distribution of compactness scores. Only 3 districts
(0.7%) fell below our threshold of PP = 0.15; manual inspection revealed
these were at-large districts in geographically challenging states
(Alaska, Montana) where low compactness is unavoidable due to state shape.

Polsby-Popper and Reock scores were highly correlated (r = 0.87,
p < 0.001), validating that both metrics capture the same underlying
compactness construct.
```

## Related Skills

- `/run-statistical-analysis` - Compute detailed compactness statistics
- `/run-experiment` - Compare compactness across algorithm variants
- `/create-state-map` - Visualize districts for manual inspection
- `/parameter-sweep` - Optimize parameters for compactness

## Performance

**Computation time**:
- Polsby-Popper: ~1 ms per district (fast)
- Reock: ~10 ms per district (requires minimum bounding circle)
- Convex hull ratio: ~5 ms per district

**Total for 435 districts**:
- PP only: ~0.5 seconds
- PP + Reock: ~5 seconds
- All three metrics: ~7 seconds

**Bottlenecks**:
- Loading geometries: Can take 1-2 minutes for all states
- Minimum bounding circle: Most expensive operation
- Visualization: 10-30 seconds per figure

## Next Steps

After validation:
- Document compactness improvements in paper
- Include distribution histograms in figures
- Compare to historical baselines
- Justify algorithm choices based on compactness gains
- Investigate outliers for geographic explanations
