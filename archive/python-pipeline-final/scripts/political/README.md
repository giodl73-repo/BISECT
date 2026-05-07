# Partisan Outcome Analysis (P1.1)

**Purpose**: Compute partisan metrics (efficiency gap, mean-median difference, partisan bias) for algorithmic vs enacted congressional districts.

**Status**: Infrastructure created, ready to execute

---

## Workflow

### Step 1: Download Election Data (4 hours)

```bash
# Download 2020 presidential election results by congressional district
python scripts/political/download_election_data.py --year 2020 --output outputs/data/2020/elections/
```

**Data Sources** (if automatic download fails):
1. **MIT Election Data Lab**: https://electionlab.mit.edu/data
   - Dataset: "U.S. President 1976–2020"
   - Download CSV with congressional district results

2. **Daily Kos Elections**: https://www.dailykos.com/stories/2021/2/5/2014924/
   - 2020 presidential results by congressional district
   - Copy data tables to CSV

3. **Dave's Redistricting App**: https://davesredistricting.org/
   - Export district-level election results
   - Use their API or manual download

**Expected Output**:
- `outputs/data/2020/elections/election_results_2020_by_district.json`
- `outputs/data/2020/elections/election_results_2020_by_district.csv`

---

### Step 2: Compute Enacted District Metrics (1 hour)

```bash
# Compute partisan metrics for enacted 2020 congressional districts
python scripts/political/compute_partisan_metrics.py \
    --year 2020 \
    --election-data outputs/data/2020/elections/election_results_2020_by_district.json \
    --output outputs/data/2020/partisan_metrics/
```

**Output**:
- `outputs/data/2020/partisan_metrics/partisan_metrics_2020_enacted.json`
- `outputs/data/2020/partisan_metrics/partisan_metrics_2020_enacted.csv`

**Metrics Computed**:
- **Efficiency Gap**: `(wasted_D - wasted_R) / total_votes`
- **Mean-Median Difference**: `median(dem_shares) - mean(dem_shares)`
- **Partisan Bias**: Expected seat advantage at 50% vote share

---

### Step 3: Map Election Data to Algorithmic Districts (1 day)

**Challenge**: Election data is by enacted districts, but we need it by census tracts to aggregate to algorithmic districts.

**Solution**: Disaggregate enacted district votes to census tracts, then reaggregate to algorithmic districts.

**Script to create**: `scripts/political/map_elections_to_tracts.py`

```python
# Pseudocode:

# For each state:
#   1. Load enacted district boundaries (TIGER/Line shapefiles)
#   2. Load census tract geometries
#   3. Overlay tracts onto enacted districts (spatial join)
#   4. Disaggregate district-level votes to tracts proportional to population
#   5. Save tract-level election estimates

# Example for Alabama:
# Tract 01001020100: 2,345 population in enacted District 1
# Enacted District 1: 100,000 dem_votes, 150,000 rep_votes (total 250,000)
# Tract gets: (2,345/district_pop) * 100,000 = dem_votes for that tract
```

**Files needed**:
- Enacted district boundaries: Download from Census TIGER/Line (118th Congressional Districts)
- Census tract populations: Already have in `outputs/data/2020/units/tracts/`

---

### Step 4: Compute Algorithmic District Metrics (4 hours)

Once election data is mapped to tracts:

```bash
# Aggregate tract votes to algorithmic districts
python scripts/political/aggregate_tract_votes.py \
    --year 2020 \
    --version v1 \
    --tract-votes outputs/data/2020/elections/tract_level_votes.csv \
    --output outputs/data/2020/partisan_metrics/

# Compute partisan metrics for algorithmic districts
python scripts/political/compute_partisan_metrics.py \
    --year 2020 \
    --election-data outputs/data/2020/partisan_metrics/algorithmic_district_votes.json \
    --output outputs/data/2020/partisan_metrics/
```

**Output**:
- `outputs/data/2020/partisan_metrics/partisan_metrics_2020_algorithmic.json`
- `outputs/data/2020/partisan_metrics/partisan_metrics_2020_algorithmic.csv`

---

### Step 5: Generate Comparison Tables (4 hours)

Create comparison tables for paper:

```bash
python scripts/political/compare_partisan_outcomes.py \
    --enacted outputs/data/2020/partisan_metrics/partisan_metrics_2020_enacted.csv \
    --algorithmic outputs/data/2020/partisan_metrics/partisan_metrics_2020_algorithmic.csv \
    --output research/gerry-edge-weighted-bisection/tables/
```

**Output Tables** (LaTeX format):
- `partisan_comparison_all_states.tex` — Full 50-state table
- `partisan_comparison_highlight.tex` — Gerrymandered states (IL, TX, LA, etc.)
- `partisan_summary.tex` — National-level summary

---

## Metrics Explained

### Efficiency Gap

**Formula**: `(wasted_D - wasted_R) / total_votes`

**Wasted votes**:
- Losing party: All their votes (couldn't win)
- Winning party: Votes beyond 50%+1 threshold (surplus)

**Interpretation**:
- Positive = Pro-Democratic bias
- Negative = Pro-Republican bias
- |EG| > 0.07 historically considered significant

**Example**:
- District 1: Dem 60,000, Rep 40,000 → Dem wins, wasted: Dem 10,000 (surplus), Rep 40,000 (all)
- District 2: Dem 30,000, Rep 70,000 → Rep wins, wasted: Dem 30,000 (all), Rep 20,000 (surplus)
- Total wasted: Dem 40,000, Rep 60,000
- EG = (40,000 - 60,000) / 200,000 = -0.10 (Pro-Republican)

### Mean-Median Difference

**Formula**: `median(dem_shares) - mean(dem_shares)`

**Interpretation**:
- Negative = Pro-Republican bias (median district more Republican than average)
- Positive = Pro-Democratic bias (median district more Democratic than average)

**Example** (3 districts):
- District 1: 65% Dem (safe D)
- District 2: 48% Dem (narrow R)
- District 3: 47% Dem (narrow R)
- Mean = 53.3%, Median = 48%
- MMD = 48% - 53.3% = -5.3% (Pro-Republican)

This shows Democrats concentrated in few safe districts while Republicans win many close districts.

### Partisan Bias

**Formula**: Fit seats-votes curve, evaluate at 50% vote share

**Interpretation**:
- Bias = Expected seat share at 50% statewide vote - 0.5
- Positive = Party would win >50% seats at 50% votes
- Negative = Party would win <50% seats at 50% votes

---

## Expected Timeline

| Step | Effort | Status |
|------|--------|--------|
| 1. Download election data | 4 hours | ⏳ Ready to start |
| 2. Compute enacted metrics | 1 hour | ⏳ Script ready |
| 3. Map elections to tracts | 1 day | ⚠️ Need to implement |
| 4. Compute algorithmic metrics | 4 hours | ⏳ Script ready |
| 5. Generate comparison tables | 4 hours | ⏳ Need to implement |
| **TOTAL** | **2-3 days** | |

---

## Next Steps

**Immediate** (today):
1. Run Step 1: `python scripts/political/download_election_data.py`
2. Verify data looks correct
3. Run Step 2: `python scripts/political/compute_partisan_metrics.py`
4. Review enacted district metrics

**Tomorrow**:
5. Implement Step 3: `map_elections_to_tracts.py`
6. Download enacted district boundaries (TIGER/Line 118th Congress)
7. Perform spatial join (tracts → enacted districts)
8. Disaggregate votes to tract level

**Day 3**:
9. Aggregate tract votes to algorithmic districts
10. Compute algorithmic metrics
11. Generate comparison tables
12. Write results subsection for paper

---

## Data Files Structure

```
outputs/data/2020/
├── elections/
│   ├── mit_election_lab_2020.csv              # Raw download
│   ├── election_results_2020_by_district.json # Parsed enacted
│   ├── election_results_2020_by_district.csv  # Parsed enacted
│   ├── tract_level_votes.csv                  # Disaggregated to tracts
│   └── enacted_district_boundaries/           # TIGER/Line shapefiles
│       ├── tl_2022_us_cd118.shp
│       └── ...
├── partisan_metrics/
│   ├── partisan_metrics_2020_enacted.json      # Enacted metrics
│   ├── partisan_metrics_2020_enacted.csv       # Enacted metrics
│   ├── partisan_metrics_2020_algorithmic.json  # Algorithmic metrics
│   └── partisan_metrics_2020_algorithmic.csv   # Algorithmic metrics
└── ...
```

---

## Troubleshooting

**Q: Election data download fails?**
A: Download manually from MIT Election Lab or Daily Kos, save as CSV, run with `--election-data` flag.

**Q: Missing enacted district boundaries?**
A: Download from Census TIGER/Line:
```bash
wget https://www2.census.gov/geo/tiger/TIGER2022/CD/tl_2022_us_cd118.zip
unzip tl_2022_us_cd118.zip -d outputs/data/2020/elections/enacted_district_boundaries/
```

**Q: How to validate partisan metrics?**
A: Compare to published analyses:
- FiveThirtyEight redistricting tracker
- Princeton Gerrymandering Project
- Brennan Center for Justice reports

Expected rough ranges:
- Efficiency gaps: typically -0.15 to +0.15
- Mean-median: typically -0.10 to +0.10
- Highly gerrymandered states (IL, MD, NC pre-2020): |EG| > 0.10

---

## References

- **Chen & Rodden (2013)**: "Unintentional Gerrymandering" — Geographic sorting effects
- **Stephanopoulos & McGhee (2015)**: "Partisan Gerrymandering and the Efficiency Gap" — EG metric definition
- **Rodden (2019)**: "Why Cities Lose" — Geography is destiny problem
- **MIT Election Data Lab**: https://electionlab.mit.edu/data
- **Daily Kos Elections**: Historical district-level results
