# Experimental Scripts - Paper #13: National Redistricting

Scripts for running the national redistricting experiment (no state boundaries).

## Workflow

### 1. Build National Adjacency Graph ✓ COMPLETE

```bash
python scripts/experimental/build_national_adjacency_graph.py --year 2020
```

**Output**: `outputs/experimental/national_adjacency_2020.pkl`
- 84,414 census tracts
- 266,466 spatial adjacency edges
- 4,544 cross-state edges

**Runtime**: ~6 seconds

---

### 2. Run National Redistricting ⏳ READY TO RUN

```bash
python scripts/experimental/run_national_redistricting.py --year 2020 --alpha 5.0
```

**Output**: `outputs/experimental/national_districts_2020.pkl`
- 435 congressional districts
- No state boundary constraints
- ±0.5% population balance

**Runtime**: 2-4 hours (METIS partitioning 84K tracts)

**Options**:
- `--n-districts 435` - Number of districts (default: 435)
- `--alpha 5.0` - Edge weight scaling for VRA compliance (default: 5.0)
- `--seed 42` - Random seed for reproducibility

**Run in background** (recommended):
```bash
nohup python scripts/experimental/run_national_redistricting.py --year 2020 --alpha 5.0 > outputs/experimental/redistricting.log 2>&1 &
```

Or on Windows:
```powershell
Start-Process python -ArgumentList "scripts/experimental/run_national_redistricting.py --year 2020" -NoNewWindow -RedirectStandardOutput outputs/experimental/redistricting.log
```

---

### 3. Compute Compactness Metrics (TODO)

```bash
python scripts/experimental/analyze_national_compactness.py --year 2020
```

Compare national vs state-based Polsby-Popper scores.

---

### 4. Identify Cross-State Districts (TODO)

```bash
python scripts/experimental/analyze_cross_state_districts.py --year 2020
```

Find which districts cross state lines and quantify "constitutional incompatibility".

---

### 5. Generate Visualizations (TODO)

```bash
python scripts/experimental/visualize_national_districts.py --year 2020
```

Create Figures 1-4 for paper:
- Figure 1: National map (435 districts)
- Figure 2: Regional zooms (4 panels)
- Figure 3: Compactness comparison (box plots)
- Figure 4: State boundary crossing analysis (bar chart)

---

## Data Files

### Input
- `data/2020/tiger/tracts/tl_2020_*_tract/` - Census tract shapefiles (all 50 states)
- `data/2020/demographics/*_demographics_2020.csv` - Population data by tract

### Output
- `outputs/experimental/national_adjacency_2020.pkl` - National graph (✓ exists)
- `outputs/experimental/national_districts_2020.pkl` - District assignments (⏳ pending)
- `outputs/experimental/compactness_analysis_2020.pkl` - Compactness metrics (TODO)
- `outputs/experimental/cross_state_analysis_2020.pkl` - Cross-state district data (TODO)

---

## Progress

- [x] Task 1: Build national adjacency graph
- [ ] Task 2: Run national redistricting **(2-4 hours - run this next)**
- [ ] Task 3: Compute compactness metrics
- [ ] Task 4: Identify cross-state districts
- [ ] Task 5-8: Generate figures

---

## Notes

- **Graph connectivity**: 644 unreachable nodes (island tracts in HI, AK) - expected behavior
- **Population**: ~331M total (2020 Census)
- **Target per district**: ~761,169 residents
- **Cross-state edges**: 4,544 tract pairs span state boundaries
- **Top border**: NJ <-> NY (167 cross-state tract adjacencies)

---

## Paper Context

This experiment establishes a **geometric upper bound** for congressional redistricting by removing the constitutional requirement that districts respect state boundaries (Article I, Section 2).

**Research questions**:
1. How much more compact could districts be without state constraints?
2. Do natural geographic regions emerge?
3. How many districts would cross state lines?
4. What is the geometric cost of federalism?

**Target venue**: Comparative Political Studies (federalism analysis)
