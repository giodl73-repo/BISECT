# Data Dictionary

**Last Updated**: January 17, 2026

This document explains all data fields in the redistricting pipeline outputs. Use this guide to understand CSVs, interpret results, and analyze redistricting quality.

## Table of Contents

- [District Summary](#district-summary)
- [District Cities](#district-cities)
- [Political Analysis](#political-analysis)
- [Demographic Analysis](#demographic-analysis)
- [Compactness Analysis](#compactness-analysis)
- [Rounds Hierarchy](#rounds-hierarchy)
- [Input Data Fields](#input-data-fields)

## District Summary

**File**: `outputs/us_{year}_{version}/states/{state}/district_summary.csv`

This is the primary output file containing geographic and population metrics for each congressional district.

### Fields

#### `district`
**Type**: Integer (1 to N)

**Description**: District number assigned during redistricting. Numbers are arbitrary and assigned sequentially as the recursive bisection algorithm creates districts.

**Example Values**:
- `1`, `2`, `3`, ..., `52` (for California with 52 districts)
- `1` (for Wyoming with 1 at-large district)

**Important**: District numbers are NOT related to existing congressional district numbers. They're newly created by the algorithm.

---

#### `population`
**Type**: Integer

**Description**: Total population living in the district, from Census data.

**Example Values**:
- 761,169 (typical district in 2020)
- 710,767 (typical district in 2010)
- 646,952 (typical district in 2000)

**Target**: Equal population across districts, within ±0.5% tolerance.

**Why it matters**: The Supreme Court requires congressional districts to have nearly equal populations (one person, one vote principle).

**Calculation**: Sum of all census tract populations within the district.

---

#### `deviation_pct`
**Type**: Float (percentage)

**Description**: How far this district's population deviates from the ideal (target) population for the state.

**Formula**: `(actual_population - target_population) / target_population × 100`

**Example Values**:
- `0.23%` (well within tolerance)
- `-0.48%` (slightly under target)
- `0.01%` (nearly perfect)

**Target**: ±0.5% maximum

**Why it matters**: Large deviations may violate constitutional requirements for equal representation.

**Interpretation**:
- Positive = district is overpopulated
- Negative = district is underpopulated
- Zero = exactly on target (rare)

---

#### `area_sq_km`
**Type**: Float

**Description**: Total land area of the district in square kilometers.

**Example Values**:
- `4,523.45` (urban district)
- `234,567.89` (rural district like Alaska's at-large district)
- `15.23` (dense urban district)

**Data Source**: Sum of `ALAND` fields from Census TIGER/Line shapefiles, converted from square meters to square kilometers.

**Why it matters**: Area affects compactness metrics and helps identify urban vs. rural districts.

**Note**: Excludes water area (lakes, rivers, coastline).

---

#### `perimeter_km`
**Type**: Float

**Description**: Total length of the district's boundary in kilometers.

**Example Values**:
- `412.3` (relatively compact district)
- `1,234.5` (irregular, high-perimeter district)

**Calculation**: Length of the district's exterior polygon boundary (shapely geometry length).

**Why it matters**: Used in compactness metrics. Higher perimeter for same area = less compact = potentially gerrymandered shape.

---

#### `polsby_popper`
**Type**: Float (0.0 to 1.0)

**Description**: Polsby-Popper compactness score measuring how close a district's shape is to a perfect circle.

**Formula**: `4π × Area / Perimeter²`

**Example Values**:
- `1.0` = Perfect circle (theoretical maximum)
- `0.67` = Fairly compact (e.g., Wyoming at-large)
- `0.34` = Moderately compact (typical urban district)
- `0.14` = Highly irregular (potential gerrymander, e.g., Maryland's 3rd District)

**Interpretation**:
| Score | Compactness Level |
|-------|-------------------|
| 0.80 - 1.00 | Very Compact |
| 0.50 - 0.80 | Compact |
| 0.30 - 0.50 | Moderately Compact |
| 0.15 - 0.30 | Irregular |
| 0.00 - 0.15 | Highly Irregular |

**Why it matters**: Lower scores may indicate gerrymandering or complex geographic boundaries (coastlines, rivers).

**Limitations**: Penalizes districts with natural geographic features (peninsulas, archipelagos).

---

#### `reock`
**Type**: Float (0.0 to 1.0)

**Description**: Reock compactness score comparing district area to the area of its minimum bounding circle.

**Formula**: `District Area / Minimum Bounding Circle Area`

**Example Values**:
- `1.0` = District fills its bounding circle perfectly
- `0.61` = Fairly compact
- `0.32` = Somewhat irregular
- `0.15` = Very irregular

**Interpretation**:
| Score | Compactness Level |
|-------|-------------------|
| 0.70 - 1.00 | Very Compact |
| 0.50 - 0.70 | Compact |
| 0.30 - 0.50 | Moderately Compact |
| 0.10 - 0.30 | Irregular |
| 0.00 - 0.10 | Highly Irregular |

**Why it matters**: Alternative compactness measure, less sensitive to boundary complexity than Polsby-Popper.

**Comparison**: Reock scores are typically higher than Polsby-Popper scores for the same district.

---

#### `num_tracts`
**Type**: Integer

**Description**: Number of census tracts included in this district.

**Example Values**:
- `142` (urban district with many small tracts)
- `15` (rural district with larger tracts)

**Why it matters**:
- More tracts = finer geographic granularity
- Fewer tracts = less flexibility in creating balanced, compact districts
- Very low counts may indicate data quality issues

**Typical Range**: 50-200 tracts per district (varies by state and urban/rural character)

---

#### `num_cities`
**Type**: Integer

**Description**: Number of named cities (population > 50,000) with centroids inside this district.

**Example Values**:
- `8` (major urban district with multiple cities)
- `0` (rural district with no large cities)
- `1` (district centered on one city)

**Data Source**: Census TIGER/Line Places shapefiles, filtered by population threshold.

**Why it matters**: Helps identify and label districts for human readability (e.g., "District 12 includes San Francisco and Oakland").

**Note**: A city may be split across multiple districts if its population exceeds one district's capacity.

---

## District Cities

**File**: `outputs/us_{year}_{version}/states/{state}/district_cities.csv`

Lists which major cities are located in each district, for labeling and interpretation.

### Fields

#### `district`
**Type**: Integer

**Description**: District number (matches district_summary.csv).

**See**: [district](#district) in District Summary section.

---

#### `city_name`
**Type**: String

**Description**: Official name of the city or place from Census data.

**Example Values**:
- `"Los Angeles"`
- `"New York"`
- `"San Francisco"`

**Data Source**: Census TIGER/Line Places shapefile `NAME` field.

---

#### `city_population`
**Type**: Integer

**Description**: City population from Census data.

**Example Values**:
- `3,898,747` (Los Angeles, 2020)
- `8,336,817` (New York City, 2020)
- `873,965` (San Francisco, 2020)

**Note**: This is the city's total population, not the portion in this district (cities may be split).

---

#### `city_geoid`
**Type**: String (7 digits)

**Description**: Census GEOID for the place (city).

**Format**: `SSCCCPP` where SS = state FIPS, PPP = place code

**Example Values**:
- `0644000` (Los Angeles: CA=06, place=44000)
- `3651000` (New York: NY=36, place=51000)

---

## Political Analysis

**File**: `outputs/us_{year}_{version}/states/{state}/political/district_political.csv`

**Availability**: 2020 only (no precinct-level data for 2010/2000)

Presidential election results aggregated to district level, showing political lean.

### Fields

#### `district`
**Type**: Integer

**Description**: District number.

---

#### `dem_votes`
**Type**: Integer

**Description**: Total Democratic votes in this district (estimated from tract-level data).

**Example Values**:
- `345,123`
- `187,456`

**Data Source**: MIT Election Lab county results, geocoded to tracts, aggregated to districts.

**Important**: This is an estimate based on spatial disaggregation. Not actual district-level election results (which don't exist for these new districts).

---

#### `rep_votes`
**Type**: Integer

**Description**: Total Republican votes in this district (estimated).

**Example Values**:
- `234,567`
- `298,765`

---

#### `total_votes`
**Type**: Integer

**Description**: Total votes cast (Democratic + Republican + third party).

**Example Values**:
- `589,012`
- `598,234`

**Note**: Includes third-party votes, so `dem_votes + rep_votes < total_votes`.

---

#### `dem_pct`
**Type**: Float (percentage)

**Description**: Democratic vote percentage.

**Formula**: `dem_votes / total_votes × 100`

**Example Values**:
- `58.6%` (strong Democratic)
- `48.1%` (competitive)
- `35.2%` (strong Republican)

---

#### `rep_pct`
**Type**: Float (percentage)

**Description**: Republican vote percentage.

**Formula**: `rep_votes / total_votes × 100`

**Example Values**:
- `39.8%` (moderate Republican)
- `49.9%` (competitive)
- `62.3%` (strong Republican)

---

#### `margin`
**Type**: Float (percentage)

**Description**: Democratic margin (positive = Dem win, negative = Rep win).

**Formula**: `dem_pct - rep_pct`

**Example Values**:
- `+18.8%` (Democratic by 18.8 points)
- `-1.8%` (Republican by 1.8 points)
- `+0.2%` (very competitive)

**Interpretation**:
| Margin | Competitiveness |
|--------|-----------------|
| ±0-5% | Highly Competitive |
| ±5-10% | Competitive |
| ±10-20% | Leans Party |
| ±20%+ | Safe Party |

---

#### `winner`
**Type**: String ('D', 'R', 'TIE')

**Description**: Which party won this district.

**Example Values**:
- `D` (Democratic winner)
- `R` (Republican winner)
- `TIE` (exact tie, very rare)

---

## Demographic Analysis

**File**: `outputs/us_{year}_{version}/states/{state}/demographic/district_demographics.csv`

Race/ethnicity and sex demographics for each district, from Census DHC data.

### Fields

#### `district`
**Type**: Integer

**Description**: District number.

---

#### `total_pop`
**Type**: Integer

**Description**: Total district population (same as in district_summary.csv).

---

#### `male`
**Type**: Integer

**Description**: Male population in district.

**Example Values**:
- `378,234` (49.7% male)

**Data Source**: Census DHC API variable `P12_002N`.

---

#### `female`
**Type**: Integer

**Description**: Female population in district.

**Example Values**:
- `382,935` (50.3% female)

**Data Source**: Census DHC API variable `P12_026N`.

**Note**: `male + female = total_pop` (approximately; census may have rounding differences).

---

#### `white_nh`
**Type**: Integer

**Description**: White alone, Non-Hispanic population.

**Example Values**:
- `234,567` (30.8% of district)

**Data Source**: Census DHC API variable `P3_003N`.

**Important**: "Non-Hispanic" means excluding Hispanic/Latino individuals who identify as white.

---

#### `black_nh`
**Type**: Integer

**Description**: Black or African American alone, Non-Hispanic population.

**Example Values**:
- `123,456` (16.2% of district)

**Data Source**: Census DHC API variable `P3_004N`.

---

#### `asian_nh`
**Type**: Integer

**Description**: Asian alone, Non-Hispanic population.

**Example Values**:
- `178,901` (23.5% of district)

**Data Source**: Census DHC API variable `P3_006N`.

---

#### `hispanic`
**Type**: Integer

**Description**: Hispanic or Latino population (any race).

**Example Values**:
- `198,765` (26.1% of district)

**Data Source**: Census DHC API variable `P4_002N`.

**Important**: Hispanic is an ethnicity, not a race. Hispanic individuals may be of any race (white, Black, Asian, etc.).

**Note**: `white_nh + black_nh + asian_nh + hispanic + other_groups ≈ total_pop`

---

#### `white_nh_pct`, `black_nh_pct`, `asian_nh_pct`, `hispanic_pct`
**Type**: Float (percentage)

**Description**: Percentage of district population for each demographic group.

**Formula**: `(group_population / total_pop) × 100`

**Example Values**:
- `white_nh_pct: 30.8%`
- `black_nh_pct: 16.2%`
- `asian_nh_pct: 23.5%`
- `hispanic_pct: 26.1%`

---

## Compactness Analysis

**File**: `outputs/us_{year}_{version}/states/{state}/compactness/district_compactness.csv`

Detailed compactness metrics and comparisons to baseline districts.

### Fields

#### `district`
**Type**: Integer

**Description**: District number.

---

#### `polsby_popper`
**Type**: Float (0.0 to 1.0)

**Description**: Polsby-Popper compactness score.

**See**: [polsby_popper](#polsby_popper) in District Summary section for full explanation.

---

#### `reock`
**Type**: Float (0.0 to 1.0)

**Description**: Reock compactness score.

**See**: [reock](#reock) in District Summary section for full explanation.

---

#### `convex_hull_ratio`
**Type**: Float (0.0 to 1.0)

**Description**: Ratio of district area to its convex hull area.

**Formula**: `District Area / Convex Hull Area`

**Example Values**:
- `1.0` = District is already convex (no indentations)
- `0.85` = Fairly convex
- `0.60` = Moderate indentations
- `0.30` = Highly indented (archipelago, complex coastline)

**Why it matters**: Measures how "filled in" the district shape is. Lower values indicate tentacles, holes, or complex boundaries.

---

#### `baseline_polsby_popper`
**Type**: Float (0.0 to 1.0) or null

**Description**: Polsby-Popper score for the existing congressional district (baseline comparison).

**Data Source**: Current congressional district boundaries (if available).

**Use**: Compare algorithm-generated districts to real-world districts to assess compactness improvement.

**Example**:
- Algorithm: `0.45`
- Baseline: `0.23`
- **Improvement**: Algorithm created more compact districts

---

#### `compactness_improvement_pct`
**Type**: Float (percentage) or null

**Description**: How much more compact the algorithm's district is compared to baseline.

**Formula**: `((algorithm_score - baseline_score) / baseline_score) × 100`

**Example Values**:
- `+95.7%` (algorithm nearly doubled compactness)
- `+12.3%` (modest improvement)
- `-5.2%` (algorithm slightly worse than baseline)

**Interpretation**:
- Positive = Algorithm more compact than current districts
- Negative = Algorithm less compact (rare, may indicate baseline was already good)

---

## Rounds Hierarchy

**File**: `outputs/us_{year}_{version}/states/{state}/rounds_hierarchy.csv`

Tracks the recursive bisection process, showing how districts were split round-by-round.

### Fields

#### `round`
**Type**: Integer (0 to log₂(N))

**Description**: Bisection round number. Round 0 is the initial state, then each round splits regions in half.

**Example Values**:
- `0` (entire state, before any splits)
- `1` (state split into 2 regions)
- `2` (each of 2 regions split into 2, creating 4 regions)
- `6` (final round for a 52-district state, creating 52 districts)

**Formula**: For N districts, there are `ceil(log₂(N))` rounds.

---

#### `district_ids`
**Type**: String (range notation)

**Description**: Which districts were created or active in this round.

**Format**:
- Single: `"1"` (one district)
- Range: `"2-27"` (districts 2 through 27)
- List: `"1, 3, 5"` (non-contiguous districts)

**Example Values**:
- Round 0: `"1"` (entire state)
- Round 1: `"2-27"` and `"28-52"` (state split in two)
- Round 6: `"1", "2", "3", ...` (individual districts)

---

#### `population`
**Type**: Integer

**Description**: Total population in this region during this round.

**Example Values**:
- Round 0: `39,538,223` (entire California)
- Round 1: `20,517,500` (half of California)
- Round 6: `761,169` (one final district)

**Target**: Each region in a round should have similar population (balanced splits).

---

#### `parent_district`
**Type**: String or empty

**Description**: Which region from the previous round was split to create this region.

**Example Values**:
- `""` (empty for round 0, no parent)
- `"1"` (this region came from splitting region 1)
- `"2-27"` (this region came from splitting the 2-27 region)

**Use**: Reconstruct the full bisection tree. Shows which districts share a common parent split.

---

#### `num_tracts`
**Type**: Integer

**Description**: Number of census tracts in this region during this round.

**Why it matters**: Shows how the geographic area was subdivided. Helps debug if splits are unbalanced.

---

## Input Data Fields

For completeness, here are key fields from input data sources:

### Census Tract Data (TIGER/Line Shapefiles)

#### `GEOID20` / `GEOID10` / `GEOID`
**Type**: String (11 digits)

**Description**: Census Geographic Identifier for the tract.

**Format**: `SSCCCTTTTTT`
- `SS` = State FIPS (e.g., 06 = California)
- `CCC` = County FIPS (e.g., 037 = Los Angeles County)
- `TTTTTT` = Tract code (e.g., 980000 = Tract 9800.00)

**Example**: `06037980000` = California, LA County, Tract 9800.00

**Critical**: Always handle as string with leading zeros preserved.

---

#### `ALAND20` / `ALAND10`
**Type**: Integer (square meters)

**Description**: Land area of the tract in square meters.

**Example**: `4523450` (4.52 km²)

---

#### `AWATER20` / `AWATER10`
**Type**: Integer (square meters)

**Description**: Water area of the tract (lakes, rivers, coastline).

**Example**: `125000` (0.125 km²)

---

#### `geometry`
**Type**: Shapely Polygon or MultiPolygon

**Description**: Geographic boundary of the tract.

**Use**: Used for spatial operations (adjacency detection, area calculation, map rendering).

---

### Census Population Data (PL 94-171)

#### `P0010001` / `P1_001N`
**Type**: Integer

**Description**: Total population in the tract.

**Example**: `5,234` (tract population)

**Critical**: This is the key field for redistricting. Must join to tract geometries.

---

## Data Quality Notes

### Population Targets

**2020 Census**: Target district population ≈ 761,169 (331,449,281 / 435)
**2010 Census**: Target district population ≈ 710,767 (308,745,538 / 435)
**2000 Census**: Target district population ≈ 646,952 (281,421,906 / 435)

**Tolerance**: ±0.5% (roughly ±3,800 people in 2020)

### Missing Data

**Political data**: Only available for 2020. Not available for 2010 or 2000.

**Demographic data**: Available for all years (2020, 2010, 2000) but field names vary by year.

**Places (cities)**: Filtered to population > 50,000 for labeling. Smaller places not included.

### Data Accuracy

**Tract-level election estimates**: Modeled data based on county-level results. Not actual precinct results. Use for analysis, not official reporting.

**Compactness scores**: Sensitive to coordinate system and projection. All calculations use consistent projection (NAD83, EPSG:4269).

**City assignments**: Based on city centroid location. A city may span multiple districts, but is only labeled in the district containing its centroid.

## Using This Data

### For Researchers

**Compactness analysis**: Compare `polsby_popper` and `reock` scores across states and years. Look for patterns in urban vs. rural districts.

**Political fairness**: Analyze `margin` distribution. Check for efficiency gaps (wasted votes). Compare seat share to vote share.

**Demographic representation**: Check if demographic percentages in districts match statewide demographics. Identify majority-minority districts.

### For Journalists

**Story angles**:
- "Algorithm creates 45% more compact districts than current maps"
- "Nonpartisan redistricting eliminates 12 uncompetitive districts"
- "New maps increase Hispanic representation by 3 seats"

**Data to cite**: `polsby_popper`, `margin`, demographic percentages.

**Visualizations**: Use state maps from `maps/` directory, district-by-district comparisons.

### For Policymakers

**Compliance checks**: Verify all districts meet population equality (±0.5%), are contiguous, respect Voting Rights Act requirements.

**Baseline comparisons**: Compare algorithm compactness to existing districts. Quantify gerrymandering reduction.

**Public engagement**: Use maps and district summaries to explain methodology, show fairness metrics.

## Questions?

**Can't find a field?** Check if it's in a different output file (summary vs. political vs. demographic).

**Unexpected values?** Verify you're using the correct year's data (2020 vs. 2010 vs. 2000).

**Need more detail?** See [DATA_FORMATS.md](../context/DATA_FORMATS.md) for technical specifications and [CONTRIBUTING.md](CONTRIBUTING.md) for developer information.
