# 2010 Census Data Status

## ✅ What Exists

### TIGER Tract Shapefiles
**Location:** `data/2010/tiger/tracts/tl_2010_{fips}_tract10/`

| State | FIPS | Directory | Status |
|-------|------|-----------|--------|
| Alabama | 01 | `tl_2010_01_tract10/` | ✓ EXISTS |
| Georgia | 13 | `tl_2010_13_tract10/` | ✓ EXISTS |
| Louisiana | 22 | `tl_2010_22_tract10/` | ✓ EXISTS |
| Mississippi | 28 | `tl_2010_28_tract10/` | ✓ EXISTS |
| South Carolina | 45 | `tl_2010_45_tract10/` | ✓ EXISTS |

Each directory contains:
- `.shp` (shapefile with geometries)
- `.dbf` (attribute data)
- `.prj` (projection info)
- `.shx` (shape index)

### Demographics Data
**Location:** `data/2010/demographics/{state}_demographics_2010.csv`

| State | File | Size | Status |
|-------|------|------|--------|
| Alabama | `alabama_demographics_2010.csv` | 70 KB | ✓ EXISTS |
| Georgia | `georgia_demographics_2010.csv` | 121 KB | ✓ EXISTS |
| Louisiana | `louisiana_demographics_2010.csv` | 69 KB | ✓ EXISTS |
| Mississippi | `mississippi_demographics_2010.csv` | 40 KB | ✓ EXISTS |
| South Carolina | `south_carolina_demographics_2010.csv` | 67 KB | ✓ EXISTS |

### Redistricting Data (PL Files)
**Location:** `data/2010/redistricting/{state}2010.pl/`

| State | Directory | Status |
|-------|-----------|--------|
| Alabama | `al2010.pl/` | ✓ EXISTS |
| Georgia | `ga2010.pl/` | ✓ EXISTS |
| Louisiana | `la2010.pl/` | ✓ EXISTS |
| Mississippi | `ms2010.pl/` | ✓ EXISTS |
| South Carolina | `sc2010.pl/` | ✓ EXISTS |

### Tract Relationship Files (2010→2020)
**Location:** `data/tract_relationships/{state}_2010_2020.csv`

| State | File | Relationships | Status |
|-------|------|---------------|--------|
| Alabama | `alabama_2010_2020.csv` | 1,967 | ✓ READY |
| Georgia | `georgia_2010_2020.csv` | 3,719 | ✓ READY |
| Louisiana | `louisiana_2010_2020.csv` | 2,069 | ✓ READY |
| Mississippi | `mississippi_2010_2020.csv` | 1,276 | ✓ READY |
| South Carolina | `south_carolina_2010_2020.csv` | 1,889 | ✓ READY |

## ❌ What's Missing

### Processed Outputs Directory
**Location:** `outputs/data/2010/` - **DOES NOT EXIST**

Need to create:
```
outputs/data/2010/
├── units/              # Processed tract data with demographics
├── adjacency/          # Adjacency matrices for graph partitioning
└── [other processed outputs]
```

**Status:** ✓ CREATED (empty directories)

### Adjacency Matrices
**Location:** `outputs/data/2010/adjacency/{state}_adjacency.npz`

| State | File | Status |
|-------|------|--------|
| Alabama | `alabama_adjacency.npz` | ❌ MISSING |
| Georgia | `georgia_adjacency.npz` | ❌ MISSING |
| Louisiana | `louisiana_adjacency.npz` | ❌ MISSING |
| Mississippi | `mississippi_adjacency.npz` | ❌ MISSING |
| South Carolina | `south_carolina_adjacency.npz` | ❌ MISSING |

**Action Required:** Run adjacency builder
```bash
python scripts/data/build_adjacency.py --year 2010 --states AL GA LA MS SC
```

## ⚠️ Data Structure Issue

### Problem
Experiment scripts expect:
- Tracts: `data/2010/tiger/tracts/{state}/` (state names like "alabama")
- Demographics: `data/2010/demographics/{state}/tract_demographics.csv`

But we actually have:
- Tracts: `data/2010/tiger/tracts/tl_2010_{fips}_tract10/` (FIPS codes)
- Demographics: `data/2010/demographics/{state}_demographics_2010.csv`

### Solutions

#### Option 1: Update Scripts (RECOMMENDED)
Modify `research/gerry-temporal-stability/scripts/run_2010_redistricting.py` to use FIPS-based paths:

```python
# Map state name to FIPS code
STATE_TO_FIPS = {
    'alabama': '01',
    'georgia': '13',
    'louisiana': '22',
    'mississippi': '28',
    'south_carolina': '45'
}

def load_2010_census_data(state: str):
    fips = STATE_TO_FIPS[state]

    # Load tract geometries using FIPS
    tracts_dir = Path(f'data/2010/tiger/tracts/tl_2010_{fips}_tract10')
    tracts = gpd.read_file(tracts_dir)

    # Load demographics using state name
    demographics_file = Path(f'data/2010/demographics/{state}_demographics_2010.csv')
    demographics = pd.read_csv(demographics_file)

    # Merge and return
    data = tracts.merge(demographics, on='GEOID', how='inner')
    return data
```

#### Option 2: Create Symbolic Links
```bash
cd data/2010/tiger/tracts
ln -s tl_2010_01_tract10 alabama
ln -s tl_2010_13_tract10 georgia
ln -s tl_2010_22_tract10 louisiana
ln -s tl_2010_28_tract10 mississippi
ln -s tl_2010_45_tract10 south_carolina
```

#### Option 3: Reorganize Demographics
```bash
cd data/2010/demographics
mkdir -p alabama georgia louisiana mississippi south_carolina
mv alabama_demographics_2010.csv alabama/tract_demographics.csv
mv georgia_demographics_2010.csv georgia/tract_demographics.csv
# etc.
```

## 📋 Next Steps

### Immediate (to run experiments)
1. **Build adjacency matrices** (required for partitioning)
   ```bash
   python scripts/data/build_adjacency.py --year 2010 --states AL GA LA MS SC
   ```
   Estimated time: 5-10 minutes

2. **Fix script paths** (choose Option 1 above)
   - Update `run_2010_redistricting.py` to use FIPS-based paths
   - Test with one state (Alabama) first

3. **Run 2010 redistricting**
   ```bash
   cd research/gerry-temporal-stability
   python scripts/run_2010_redistricting.py
   ```
   Estimated time: 30-60 minutes for all 5 states

### Verification Commands

```bash
# Check tract data exists and is readable
python -c "import geopandas as gpd; print(len(gpd.read_file('data/2010/tiger/tracts/tl_2010_01_tract10')))"

# Check demographics exists and has right columns
python -c "import pandas as pd; df = pd.read_csv('data/2010/demographics/alabama_demographics_2010.csv'); print(df.columns.tolist())"

# Check adjacency after building
python -c "from scipy import sparse; import numpy as np; adj = sparse.load_npz('outputs/data/2010/adjacency/alabama_adjacency.npz'); print(f'Shape: {adj.shape}, Edges: {adj.nnz}')"
```

## Summary

✅ **Raw data:** COMPLETE (tracts, demographics, PL files, relationship files)
✅ **Tract relationships:** COMPLETE (downloaded & processed)
❌ **Adjacency matrices:** MISSING (need to build)
⚠️ **Script paths:** Need updating to match actual data structure

**Readiness:** 70% - Data exists, just needs processing and path fixes
