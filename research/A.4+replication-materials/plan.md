# A.4 — Replication Materials and Data Archive

**Paper Type**: Replication Package and Technical Documentation
**Status**: Planned
**Target Venue**: Zenodo / Harvard Dataverse / Open ICPSR
**Format**: Multi-component archive (code, data, documentation, Docker containers)
**Target Audience**: Researchers, reviewers, students attempting to reproduce portfolio experiments

---

## Purpose

Create a **comprehensive replication package** that enables independent researchers to reproduce all experiments, figures, and results from the 28-paper congressional redistricting portfolio. This package provides complete transparency, facilitates validation of findings, and enables extensions by future researchers.

**Key Innovation**: Complete reproducibility infrastructure spanning 23 empirical papers across 3 census years, 50 states, and 435 congressional districts—with step-by-step guides, automated validation, and containerized environments.

---

## Target Audience

1. **Journal Reviewers**: Verifying computational claims during peer review
2. **Replication Researchers**: Independently validating findings for replication studies
3. **Graduate Students**: Learning redistricting algorithms and extending the work
4. **Policy Analysts**: Running scenarios for specific states or parameters
5. **Open Science Community**: Examining transparency and reproducibility standards

---

## Package Objectives

1. **Enable exact reproduction**: Any researcher can regenerate every figure, table, and statistic from all 23 papers
2. **Provide computational efficiency**: Pre-processed data and cached results reduce replication time from weeks to hours
3. **Support selective replication**: Users can reproduce single papers, specific states, or particular experiments
4. **Ensure cross-platform compatibility**: Works on Windows, Linux, macOS via Docker containers
5. **Document dependencies**: Clear specifications for all software, libraries, and data requirements
6. **Validate outputs**: Automated checksums and comparison tools verify successful replication
7. **Lower barriers**: Minimize setup complexity so researchers without deep technical expertise can replicate

---

## Package Structure

The replication package consists of seven main components:

### Component 1: Source Code Repository
- **Location**: GitHub repository with permanent DOI
- **Contents**: Complete Python codebase (`src/`, `scripts/`, `tests/`)
- **Version**: Tagged release corresponding to paper submission (e.g., `v1.0-submission`)
- **License**: MIT or GPL-3.0 (open source)
- **Documentation**: README, ARCHITECTURE.md, CODING_PATTERNS.md

### Component 2: Data Archive
- **Location**: Zenodo/Dataverse (separate from code due to size)
- **Contents**:
  - Raw census data (2000/2010/2020) - ~40GB
  - Processed intermediate data (units, adjacency graphs) - ~20GB
  - Final results (district assignments, shapefiles, CSVs) - ~15GB
- **Structure**: Organized by census year and data type
- **Checksums**: SHA-256 hashes for all files to verify integrity

### Component 3: Docker Containers
- **Location**: Docker Hub + Dockerfile in repository
- **Purpose**: Frozen computational environment with exact library versions
- **Variants**:
  - `redistricting-base`: Python 3.13, METIS, GeoPandas (3GB)
  - `redistricting-full`: Base + all data + cached results (60GB)
  - `redistricting-minimal`: Base only, users download data separately (3GB)
- **Benefits**: Eliminates "works on my machine" problems

### Component 4: Replication Guides
- **Format**: Markdown documents with step-by-step instructions
- **One guide per track**: B (algorithm), C (validation), D (VRA), E (experimental)
- **Contents**:
  - Installation steps
  - Data download/preparation
  - Command sequences to reproduce each paper
  - Expected outputs (with checksums)
  - Troubleshooting for common issues

### Component 5: Validation Scripts
- **Purpose**: Automated checking that replication succeeded
- **Mechanism**: Compare user's outputs to reference outputs using checksums, statistical tests
- **Coverage**:
  - District assignment files (exact match required)
  - Compactness scores (floating-point tolerance ±0.001)
  - Figures (pixel-level comparison or perceptual hash)
  - Statistical tables (numerical tolerance ±0.01%)

### Component 6: Pre-Computed Results Cache
- **Purpose**: Allow users to verify final outputs without running full pipeline
- **Contents**:
  - All 435 district assignments for primary experiments
  - All figures from papers (PNG, PDF)
  - All tables (CSV, LaTeX)
  - Compactness/demographic/political metrics
- **Use Case**: Reviewers can verify claims in minutes without multi-hour computation

### Component 7: Interactive Web Dashboard
- **Location**: GitHub Pages (static HTML) or hosted web app
- **Purpose**: Explore results without downloading anything
- **Features**:
  - Interactive maps for all 50 states
  - Parameter sliders to adjust settings
  - Comparison tools (algorithmic vs enacted vs baseline)
  - Download results for specific states
- **Link**: Permanent URL referenced in papers

---

## Document Structure

The main replication document should be a comprehensive guide (30-50 pages) structured as follows:

### Section 1: Overview (2-3 pages)

**Purpose**: Introduce the replication package and set expectations.

**Content**:
- **Package Scope**: What can be replicated (all 23 empirical papers)
- **What's Not Included**: Papers A.0-A.3 are synthesis/documentation, not empirical
- **System Requirements**: Hardware (16GB RAM, 100GB disk), software (Python 3.13+, METIS)
- **Time Estimates**:
  - Minimal replication (one state): 30 minutes
  - Full replication (all 50 states, one census year): 2-4 hours
  - Complete replication (all experiments across all papers): 20-30 hours
- **Three Replication Levels**:
  - **Level 1 (Quick)**: Verify final outputs against cached results (30 min)
  - **Level 2 (Selective)**: Reproduce specific papers or states (2-6 hours)
  - **Level 3 (Complete)**: Regenerate everything from raw data (20-30 hours)

### Section 2: Quick Start (3-4 pages)

**Purpose**: Get users running code as fast as possible.

**Content**:

#### 2.1 Installation Options

**Option A: Docker (Recommended)**
```bash
# Pull pre-built container with all dependencies
docker pull redistricting/full:v1.0

# Run container with mounted volume for outputs
docker run -it -v $(pwd)/outputs:/app/outputs redistricting/full:v1.0

# Inside container, run test
python scripts/pipeline/run_state_redistricting.py --year 2020 --state VT --version test
```

**Option B: Manual Installation**
```bash
# Clone repository
git clone https://github.com/username/apportionment.git
cd apportionment

# Install Python dependencies
pip install -r requirements.txt

# Install METIS (Linux/macOS)
# See DEPENDENCIES.md for platform-specific instructions

# Download minimal test data
python scripts/data/download_orchestrator.py --states VT DE --year 2020

# Run test
python scripts/pipeline/run_state_redistricting.py --year 2020 --state VT --version test
```

**Option C: Cloud Compute (Binder/Colab)**
- Click badge in repository README to launch Jupyter notebook environment
- Pre-configured with dependencies
- Limited to smaller experiments due to resource constraints

#### 2.2 Verify Installation
```bash
# Run test suite (should pass all 215 tests in ~24 seconds)
pytest tests/ -v

# Run single-state test (Vermont, smallest state)
python scripts/pipeline/run_state_redistricting.py --year 2020 --state VT --version test

# Check output
ls outputs/test/2020/districts/vermont_districts.csv
```

#### 2.3 Quick Replication Example
```bash
# Reproduce Paper B.1 (Recursive Bisection) for Minnesota
python scripts/experiments/reproduce_paper_B1.py --state MN --year 2020

# Validate output matches published results
python scripts/validation/validate_replication.py --paper B1 --state MN --year 2020

# Expected output: "✓ All checks passed. Results match published paper within tolerance."
```

### Section 3: Data Acquisition (4-5 pages)

**Purpose**: Explain how to obtain all required data.

**Content**:

#### 3.1 Data Sources
- **Census Redistricting Data**: PL-94171 files from census.gov
- **TIGER Shapefiles**: Geographic boundaries for tracts/blocks
- **Demographic Data**: DHC files for race/ethnicity breakdowns
- **Election Data**: MIT Election Lab, Dave's Redistricting App
- **Enacted Districts**: Official state redistricting plans (multiple sources)

#### 3.2 Pre-Packaged Data Archive
**Recommended**: Download complete pre-processed archive from Zenodo
```bash
# Download full data archive (75GB, includes all years)
wget https://zenodo.org/record/XXXXX/files/apportionment_data_full_v1.0.tar.gz

# Extract to data/ directory
tar -xzf apportionment_data_full_v1.0.tar.gz -C data/

# Verify integrity
python scripts/validation/verify_data_checksums.py
```

**Selective**: Download only specific years or states
```bash
# Download only 2020 data for 5 states (10GB)
python scripts/data/download_orchestrator.py \
  --year 2020 \
  --states CA TX FL NY PA \
  --stages all \
  --verify-checksums
```

#### 3.3 Building from Scratch
For users wanting to process raw census data themselves:
```bash
# Download raw census files
python scripts/data/download_redistricting_files.py --year 2020 --all-states

# Download TIGER shapefiles
python scripts/data/download_tiger_units.py --year 2020 --resolution tracts --all-states

# Process census data (merge population + geography)
python scripts/data/merge_units_with_geometries.py --year 2020 --resolution tracts

# Build adjacency graphs
python scripts/data/build_adjacency_graphs.py --year 2020 --resolution tracts

# Verify processed data matches archive
python scripts/validation/compare_with_reference_data.py --year 2020
```

#### 3.4 Data Directory Structure
```
data/
├── 2000/
│   ├── redistricting/          # Raw PL-94171 files
│   ├── tiger/tracts/           # Census tract shapefiles
│   ├── demographics/           # DHC demographic data
│   └── elections/              # Election results
├── 2010/ [same structure]
└── 2020/ [same structure]

outputs/data/
├── 2000/
│   ├── units/                  # Processed tract data (pop + geometry)
│   ├── adjacency/              # Adjacency graphs (.graph files)
│   ├── places/                 # Cities/counties reference
│   └── demographics/           # Processed demographic summaries
├── 2010/ [same structure]
└── 2020/ [same structure]
```

### Section 4: Experiment-to-Paper Mapping (6-8 pages)

**Purpose**: Map each of the 23 empirical papers to specific experiments.

**Content**: For each paper, provide:
1. **Paper ID and Title**
2. **Core Experiment**: Main analysis that produces the paper's key result
3. **Command Sequence**: Exact commands to reproduce
4. **Input Data**: What data files are required
5. **Expected Outputs**: What files should be generated
6. **Validation**: How to verify reproduction succeeded
7. **Time Estimate**: How long it takes to run

**Format** (template repeated for each paper):

---

#### Paper B.1 — Recursive Bisection

**Title**: Recursive Bisection for Congressional Redistricting: Extending Huntington-Hill to Boundary Design

**Core Experiment**: 50-state redistricting using recursive bisection with edge weights (2020 census)

**Command Sequence**:
```bash
# Run full 50-state pipeline (2-4 hours, 12 workers)
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version B1_replication \
  --workers 12

# Generate compactness analysis
python scripts/compactness/compute_compactness_metrics.py \
  --version B1_replication \
  --year 2020

# Create national map figure (Figure 3 in paper)
python scripts/visualization/create_national_map.py \
  --version B1_replication \
  --year 2020 \
  --output research/B.1+recursive-bisection/figures/national_map_2020.png

# Generate compactness comparison table (Table 2 in paper)
python scripts/analysis/compare_compactness.py \
  --version B1_replication \
  --year 2020 \
  --baseline unweighted \
  --output research/B.1+recursive-bisection/tables/compactness_comparison.csv
```

**Input Data Required**:
- `outputs/data/2020/units/tracts/*.parquet` (all 50 states)
- `outputs/data/2020/adjacency/*.graph` (METIS format)
- `data/2020/enacted_districts/` (for comparison)

**Expected Outputs**:
- `outputs/B1_replication/2020/districts/*.csv` (50 files, one per state)
- `outputs/B1_replication/2020/maps/*.png` (50 state maps)
- `outputs/B1_replication/2020/metrics/compactness_summary.csv`
- `research/B.1+recursive-bisection/figures/national_map_2020.png`
- `research/B.1+recursive-bisection/tables/compactness_comparison.csv`

**Validation**:
```bash
# Compare district assignments to reference
python scripts/validation/validate_replication.py \
  --paper B1 \
  --version B1_replication \
  --year 2020 \
  --check-assignments \
  --check-metrics \
  --check-figures

# Expected output:
# ✓ District assignments match reference (50/50 states)
# ✓ Compactness scores within tolerance (mean PP: 0.367 ± 0.001)
# ✓ Figures match reference (perceptual hash similarity > 0.99)
```

**Time Estimate**: 3-5 hours (full pipeline) or 10 minutes (using cached districts)

**Key Results to Verify**:
- National mean Polsby-Popper: 0.367 (paper reports 0.367)
- Illinois improvement: 174% over enacted (paper reports 174%)
- Minnesota mean PP: 0.468 (paper reports 0.468)

**Paper-Specific Notes**: This is the foundation paper—all other papers build on this baseline.

---

#### Paper B.2 — Edge-Weighted Bisection

**Title**: Edge-Weighted Graph Partitioning for Compact Congressional Districts

**Core Experiment**: Comparison of edge-weighted vs unweighted partitioning across 50 states

**Command Sequence**:
```bash
# Run unweighted baseline (for comparison)
python scripts/experiments/run_unweighted_baseline.py \
  --year 2020 \
  --version B2_baseline \
  --workers 12

# Run edge-weighted version (same as B.1)
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version B2_edgeweighted \
  --workers 12

# Compare unweighted vs edge-weighted
python scripts/analysis/compare_weighting_schemes.py \
  --baseline B2_baseline \
  --treatment B2_edgeweighted \
  --year 2020 \
  --output research/B.2+edge-weighted-bisection/tables/weighting_comparison.csv

# Generate improvement map (Figure 4 in paper)
python scripts/visualization/create_improvement_map.py \
  --baseline B2_baseline \
  --treatment B2_edgeweighted \
  --year 2020 \
  --output research/B.2+edge-weighted-bisection/figures/improvement_map.png

# Create laymen's guide pedagogical examples
python scripts/visualization/create_pedagogical_example.py \
  --city minneapolis \
  --split 50-50 \
  --show-edge-weights \
  --output artifacts/guides/edge_weighted_bisection/minneapolis_example.png
```

**Input Data Required**:
- Same as B.1 plus:
- `outputs/data/2020/adjacency/*_unweighted.graph` (graphs without edge weights)

**Expected Outputs**:
- All outputs from B.1 plus:
- `outputs/B2_baseline/2020/` (unweighted results)
- `research/B.2+edge-weighted-bisection/tables/weighting_comparison.csv`
- `research/B.2+edge-weighted-bisection/figures/improvement_map.png`
- `artifacts/guides/edge_weighted_bisection/` (6+ pedagogical examples)

**Validation**:
```bash
python scripts/validation/validate_replication.py --paper B2 --year 2020
```

**Time Estimate**: 6-8 hours (both runs) or 15 minutes (using cached results)

**Key Results to Verify**:
- Edge-weighted mean PP: 0.367
- Unweighted mean PP: 0.235
- Improvement: 56% (paper reports 56%)
- Improvement over enacted: 20% (paper reports 20%)

---

**[Continue this pattern for all 23 papers: B.3, B.4, B.5, C.1-C.5, D.0-D.3, E.1-E.5]**

---

### Section 5: Full Replication Workflows (4-5 pages)

**Purpose**: Provide end-to-end workflows for reproducing entire tracks.

#### 5.1 Track B: Algorithm Design (Complete Replication)

**Goal**: Reproduce all 6 papers in Track B showing algorithm design decisions

**Workflow**:
```bash
# 1. Run baseline experiments (unweighted, n-way partitioning)
bash scripts/experiments/track_B_baselines.sh

# 2. Run core algorithm experiments (edge-weighted recursive bisection)
bash scripts/experiments/track_B_core.sh

# 3. Run parameter sensitivity experiments (adaptive bisection)
bash scripts/experiments/track_B_sensitivity.sh

# 4. Run comparative experiments (multi-constraint vs edge-weighting)
bash scripts/experiments/track_B_comparisons.sh

# 5. Generate all figures and tables for Track B papers
bash scripts/experiments/track_B_outputs.sh

# 6. Validate all Track B results
python scripts/validation/validate_track_B.py --comprehensive
```

**Time Estimate**: 12-16 hours (parallelized) or 40-60 hours (sequential)

**Outputs**: Complete replication of Papers B.1-B.5 plus inputs for B.0 synthesis

#### 5.2 Track C: Validation (Complete Replication)

**Goal**: Reproduce all 6 papers showing multi-faceted validation

**Workflow**:
```bash
# 1. Run MAUP sensitivity analysis (tract, block group, block resolutions)
bash scripts/experiments/track_C_maup.sh

# 2. Run cross-census validation (2000, 2010, 2020)
bash scripts/experiments/track_C_crosscensus.sh

# 3. Run temporal stability analysis
bash scripts/experiments/track_C_temporal.sh

# 4. Run longitudinal analysis (20-year trends)
bash scripts/experiments/track_C_longitudinal.sh

# 5. Run efficiency gap analysis (partisan fairness)
bash scripts/experiments/track_C_partisan.sh

# 6. Generate all validation figures and tables
bash scripts/experiments/track_C_outputs.sh

# 7. Validate all Track C results
python scripts/validation/validate_track_C.py --comprehensive
```

**Time Estimate**: 20-30 hours (multiple census years, multiple resolutions)

**Outputs**: Complete replication of Papers C.1-C.5 plus inputs for C.0 synthesis

#### 5.3 Track D: VRA Compliance (Complete Replication)

**Goal**: Reproduce all 4 papers on Voting Rights Act compliance

**Workflow**:
```bash
# 1. Run VRA baseline (standard recursive bisection)
bash scripts/experiments/track_D_baseline.sh

# 2. Run VRA threshold sensitivity (40%, 45%, 50% minority population)
bash scripts/experiments/track_D_thresholds.sh

# 3. Run VRA n-way vs recursive comparison
bash scripts/experiments/track_D_nway.sh

# 4. Run compactness-VRA tradeoff analysis
bash scripts/experiments/track_D_tradeoffs.sh

# 5. Generate all VRA figures and tables
bash scripts/experiments/track_D_outputs.sh

# 6. Validate all Track D results
python scripts/validation/validate_track_D.py --comprehensive
```

**Time Estimate**: 10-15 hours

**Outputs**: Complete replication of Papers D.0-D.3

#### 5.4 Track E: Experimental Alternatives (Complete Replication)

**Goal**: Reproduce all 6 papers exploring alternative systems

**Workflow**:
```bash
# 1. Run multi-member district experiments
bash scripts/experiments/track_E_multimember.sh

# 2. Run county representation experiments
bash scripts/experiments/track_E_county.sh

# 3. Run national redistricting experiments
bash scripts/experiments/track_E_national.sh

# 4. Run partisan similarity experiments
bash scripts/experiments/track_E_partisan.sh

# 5. Run party-based allocation experiments
bash scripts/experiments/track_E_partybased.sh

# 6. Generate all experimental system figures and tables
bash scripts/experiments/track_E_outputs.sh

# 7. Validate all Track E results
python scripts/validation/validate_track_E.py --comprehensive
```

**Time Estimate**: 15-20 hours

**Outputs**: Complete replication of Papers E.1-E.5 plus inputs for E.0 synthesis

### Section 6: Validation and Verification (3-4 pages)

**Purpose**: Explain how to verify replication succeeded.

#### 6.1 Automated Validation Tools

**Validation Levels**:
1. **File Existence**: Check all expected output files were created
2. **Format Correctness**: Verify CSV/Parquet/shapefile structure
3. **Data Integrity**: Compare checksums for exact reproduction
4. **Numerical Tolerance**: Compare metrics within floating-point tolerance
5. **Visual Similarity**: Compare figures using perceptual hashing
6. **Statistical Equivalence**: Compare distributions using KS/MW tests

**Running Validation**:
```bash
# Validate single paper
python scripts/validation/validate_replication.py \
  --paper B1 \
  --level comprehensive \
  --tolerance 0.001

# Validate entire track
python scripts/validation/validate_track_B.py --comprehensive

# Validate all experiments
python scripts/validation/validate_all.py --level comprehensive
```

**Validation Output**:
```
=== Replication Validation Report ===
Paper: B.1 — Recursive Bisection
Date: 2026-02-08

File Existence:        ✓ 156/156 files present
Format Correctness:    ✓ All files valid
Data Integrity:        ✓ 152/156 checksums match (4 figures have minor pixel differences)
Numerical Tolerance:   ✓ All metrics within ±0.001
Visual Similarity:     ✓ All figures > 0.98 perceptual similarity
Statistical Equiv:     ✓ All distributions p > 0.05

OVERALL: ✓ REPLICATION SUCCESSFUL

Minor Differences:
- Figure 3: 2 pixels differ (likely font rendering difference)
- Figure 5: 1 pixel differs (anti-aliasing)

These minor differences do not affect scientific conclusions.
```

#### 6.2 Manual Verification Steps

For users wanting to manually verify key results:

1. **Check Key Metrics**: Compare summary statistics
```bash
# Your replication
cat outputs/B1_replication/2020/metrics/compactness_summary.csv

# Reference values
cat reference_outputs/B1/2020/metrics/compactness_summary.csv

# Should match within ±0.001
```

2. **Visual Inspection**: Compare maps side-by-side
```bash
# Open your map and reference map in image viewer
open outputs/B1_replication/2020/maps/california.png
open reference_outputs/B1/2020/maps/california.png

# Districts should be visually identical
```

3. **Spot Check States**: Verify specific high-impact states
- California (52 districts): Largest state, most complex
- Illinois (17 districts): Biggest improvement claim (174%)
- Vermont (1 district): Smallest state, simplest case
- Texas (38 districts): Large population, diverse geography

#### 6.3 Troubleshooting Common Issues

**Problem**: Compactness scores differ by more than tolerance
**Cause**: METIS randomness (tie-breaking in graph partitioning)
**Solution**: Use `--seed 42` flag to force deterministic behavior

**Problem**: Figure colors don't match exactly
**Cause**: Different matplotlib/backend versions
**Solution**: Use Docker container with frozen versions

**Problem**: Some district assignments differ
**Cause**: Different METIS version (5.1.0 vs 5.2.0)
**Solution**: Install exact version specified in requirements.txt

**Problem**: Validation script reports missing files
**Cause**: Experiment didn't complete successfully
**Solution**: Check `outputs/[version]/[year]/error.log` for failures

### Section 7: Advanced Topics (3-4 pages)

#### 7.1 Running on HPC Clusters

For users with access to high-performance computing:

```bash
# SLURM batch script for full replication
sbatch scripts/hpc/run_full_replication.slurm

# Parameters: 50 nodes, 20 cores each, 6 hours walltime
# Parallelizes across states and census years
```

**Benefits**: Complete replication in ~2 hours instead of 30 hours

#### 7.2 Modifying Experiments

Researchers wanting to extend the work:

**Example: Test different compactness thresholds**
```bash
# Modify configuration
python scripts/experiments/create_custom_experiment.py \
  --base-experiment B1 \
  --parameter compactness_weight \
  --values 0.5 1.0 2.0 5.0 \
  --output-version custom_compactness

# Run custom experiment
bash outputs/custom_compactness/run_experiment.sh

# Compare to baseline
python scripts/analysis/compare_experiments.py \
  --baseline B1_replication \
  --treatment custom_compactness
```

#### 7.3 Generating Paper-Ready Figures

All figures in papers can be regenerated:

```bash
# Regenerate all figures for Paper B.2
python scripts/visualization/regenerate_paper_figures.py \
  --paper B2 \
  --dpi 300 \
  --format pdf \
  --output research/B.2+edge-weighted-bisection/figures/

# Figures are saved in publication-ready format
```

#### 7.4 Exporting Results for Analysis

Export data to other formats for external analysis:

```bash
# Export to R-friendly format
python scripts/export/export_to_r.py \
  --paper B1 \
  --format rds \
  --output replication_data_B1.rds

# Export to Stata format
python scripts/export/export_to_stata.py \
  --paper C5 \
  --format dta \
  --output efficiency_gap_data.dta

# Export shapefiles for GIS software
python scripts/export/export_shapefiles.py \
  --paper B1 \
  --year 2020 \
  --states all \
  --output shapefiles/
```

### Section 8: Computational Requirements (2 pages)

#### 8.1 Hardware Requirements

**Minimum Requirements** (selective replication):
- CPU: 4 cores (Intel i5 or equivalent)
- RAM: 8 GB
- Disk: 50 GB free space
- Time: 4-6 hours for Track B

**Recommended Requirements** (full replication):
- CPU: 12+ cores (Intel i7/i9 or AMD Ryzen)
- RAM: 16-32 GB
- Disk: 200 GB free space (100GB data + 100GB outputs)
- Time: 20-30 hours for all tracks

**High-Performance Configuration** (rapid replication):
- CPU: 32+ cores (server-grade Xeon/EPYC)
- RAM: 64-128 GB
- Disk: 500 GB SSD
- Time: 2-4 hours for all tracks

#### 8.2 Software Requirements

**Required**:
- Python 3.13+ (exact version ensures compatibility)
- METIS 5.1.0 (graph partitioning library)
- GDAL 3.9+ (geospatial data processing)
- Git 2.30+ (version control)

**Python Libraries** (specified in requirements.txt):
- geopandas 1.0.1
- pandas 2.2.3
- numpy 2.1.3
- matplotlib 3.9.2
- shapely 2.0.6
- pytest 8.3.4 (for testing)

**Optional but Recommended**:
- Docker 20.10+ (containerization)
- Jupyter Lab 4.0+ (interactive exploration)
- Make 4.0+ (build automation)

#### 8.3 Operating System Compatibility

**Linux (Recommended)**:
- Ubuntu 24.04 LTS or Debian 12+
- METIS installs cleanly via package manager
- Best performance for parallel processing

**Windows**:
- Windows 10/11 with WSL2 (Windows Subsystem for Linux)
- METIS requires compilation or WSL2
- Use Docker for easiest setup

**macOS**:
- macOS 13+ (Ventura or later)
- METIS installs via Homebrew
- ARM (M1/M2) requires Rosetta for some dependencies

### Section 9: Archive Distribution (2 pages)

#### 9.1 Archive Components

The replication package is distributed as multiple archives to accommodate different user needs:

**Archive 1: Minimal (5 GB)**
- Source code
- Documentation
- Test data (Vermont, Delaware only)
- Docker configurations
- Quick start scripts

**Archive 2: Complete Code + Processed Data (80 GB)**
- All source code
- All processed data (ready for pipeline)
- Reference outputs (cached results)
- Validation scripts

**Archive 3: Raw Data (40 GB)**
- Original census files (PL-94171)
- Original TIGER shapefiles
- For users wanting to process from scratch

**Archive 4: Full Package (120 GB)**
- Everything from Archives 1-3
- Comprehensive documentation
- Video tutorials

#### 9.2 Download Locations

**Primary**: Zenodo (permanent DOI)
- DOI: 10.5281/zenodo.XXXXXXX
- URL: https://zenodo.org/record/XXXXXXX

**Secondary**: Harvard Dataverse
- DOI: 10.7910/DVN/XXXXXX
- URL: https://dataverse.harvard.edu/dataset.xhtml?persistentId=XXXXXX

**Tertiary**: GitHub Releases (code only)
- URL: https://github.com/username/apportionment/releases/tag/v1.0

**Mirror**: Institutional Repository (backup)
- University repository URL

#### 9.3 Version Control and Updates

**Version Numbering**: Semantic versioning (MAJOR.MINOR.PATCH)
- v1.0.0: Initial submission (papers submitted)
- v1.0.1: Bug fixes (no scientific changes)
- v1.1.0: Minor additions (new visualizations, improved documentation)
- v2.0.0: Major updates (if papers are revised based on peer review)

**Update Policy**:
- Critical bug fixes: Released immediately with patch version bump
- Documentation improvements: Released quarterly with minor version bump
- Major revisions: Released only when papers are revised and resubmitted

**Accessing Old Versions**: All versions preserved on Zenodo with unique DOIs

---

## Writing Guidelines

### Documentation Style
- **Clear and explicit**: Step-by-step instructions assume no prior knowledge
- **Command-focused**: Every instruction includes exact command to run
- **Validation-oriented**: After each step, explain how to verify success
- **Troubleshooting-ready**: Anticipate common problems and provide solutions

### Code Comments
```python
# Replication Note: This script reproduces Figure 3 from Paper B.1
# Expected output: outputs/B1_replication/2020/figures/national_map.png
# Runtime: ~5 minutes on 12 cores
# Validation: Compare with reference_outputs/B1/2020/figures/national_map.png
```

### Reproducibility Principles
1. **Deterministic**: Use fixed random seeds (`--seed 42`)
2. **Versioned**: Pin all dependency versions in requirements.txt
3. **Containerized**: Provide Docker images with frozen environments
4. **Documented**: Every script has usage instructions (`--help`)
5. **Validated**: Every output has automated validation
6. **Transparent**: All data sources and processing steps documented

---

## Success Criteria

This replication package succeeds if:

1. ✓ An independent researcher can reproduce any paper's main result in ≤4 hours
2. ✓ Automated validation confirms >95% of outputs match exactly
3. ✓ Docker containers eliminate "works on my machine" problems
4. ✓ Documentation is clear enough that a graduate student can follow it
5. ✓ Selective replication (single paper) requires <50GB disk and <8GB RAM
6. ✓ All data sources are permanently archived with DOIs
7. ✓ Troubleshooting guide covers >90% of actual user problems
8. ✓ Reviewers can verify key claims in <1 hour using cached results

---

## Target Metrics

- **Setup Time**: <30 minutes (Docker) or <2 hours (manual install)
- **Minimal Replication**: <1 hour (verify cached results)
- **Single Paper Replication**: 2-6 hours (reproduce one paper)
- **Full Replication**: 20-30 hours (reproduce all 23 papers)
- **HPC Replication**: 2-4 hours (with 50+ cores)
- **Validation Time**: <15 minutes per paper
- **User Support**: Respond to GitHub issues within 48 hours

---

## Dependencies

**This package depends on**:
- All 23 empirical papers (B.1-B.5, C.1-C.5, D.0-D.3, E.1-E.5): Provides experiments to replicate
- Source codebase: `src/apportionment/` and `scripts/`
- Data infrastructure: Census API, TIGER downloads, data processing pipeline
- Testing framework: 215 tests ensuring code correctness

**Papers that depend on this**:
- **All Track B-E papers**: Should reference this as supplementary material for replication
- **A.0 (Synthesis)**: Will highlight replication package as evidence of transparency
- **A.3 (Visualization)**: Links to web dashboard component

---

## Next Steps for Implementation

1. **Create validation scripts** (scripts/validation/)
   - validate_replication.py (single paper validator)
   - validate_track_[B|C|D|E].py (track validators)
   - validate_all.py (comprehensive validation)
   - compare_with_reference_data.py (data integrity checker)

2. **Create experiment scripts** (scripts/experiments/)
   - reproduce_paper_[B1|B2|...].py (per-paper reproduction)
   - track_[B|C|D|E]_*.sh (track-level workflows)
   - create_custom_experiment.py (for extensions)

3. **Build Docker containers** (docker/)
   - Dockerfile.base (dependencies only)
   - Dockerfile.full (with data)
   - docker-compose.yml (multi-container setup)

4. **Write comprehensive documentation**
   - REPLICATION_GUIDE.md (main guide, 30-50 pages)
   - QUICK_START.md (5-page quick start)
   - TROUBLESHOOTING.md (common issues)
   - VIDEO_TUTORIALS.md (links to walkthrough videos)

5. **Generate reference outputs**
   - Run all experiments with fixed seeds
   - Compute checksums for all outputs
   - Store in reference_outputs/ directory

6. **Create web dashboard**
   - Interactive maps (Leaflet/Mapbox)
   - Parameter exploration tools
   - Download results functionality
   - Deploy to GitHub Pages

7. **Package and archive**
   - Create four archive variants (minimal/complete/raw/full)
   - Upload to Zenodo with metadata
   - Register DOIs
   - Create mirror on Dataverse

8. **Test with external users**
   - Recruit 3-5 graduate students unfamiliar with the code
   - Have them attempt replication following only the documentation
   - Iterate on documentation based on their struggles
   - Update troubleshooting guide with actual issues encountered

---

## Estimated Timeline

**Phase 1 (Validation Infrastructure)**: 1-2 weeks
- Create validation scripts
- Generate reference outputs
- Test validation tools

**Phase 2 (Docker Containers)**: 1 week
- Build Docker images
- Test on Linux/Windows/macOS
- Optimize image size

**Phase 3 (Documentation)**: 2-3 weeks
- Write comprehensive replication guide
- Create quick start guide
- Write troubleshooting guide
- Record video tutorials

**Phase 4 (Web Dashboard)**: 1-2 weeks
- Build interactive maps
- Add parameter exploration
- Deploy to hosting

**Phase 5 (Archival)**: 1 week
- Package archives
- Upload to Zenodo/Dataverse
- Register DOIs
- Create mirrors

**Phase 6 (External Testing)**: 2-3 weeks
- Recruit testers
- Support replication attempts
- Iterate on documentation
- Fix discovered issues

**Total**: 8-12 weeks

---

## Notes

- This replication package sets a high standard for computational reproducibility in political science
- The multi-level replication strategy (quick/selective/complete) accommodates different user needs
- Docker containers eliminate most platform-specific issues
- Automated validation reduces manual checking burden
- Web dashboard provides exploration without requiring local installation
- Permanent archival with DOIs ensures long-term availability

**Gold Standard Elements**:
- ✓ Complete source code with version control
- ✓ All data permanently archived
- ✓ Containerized computational environment
- ✓ Automated validation tools
- ✓ Step-by-step documentation
- ✓ Multiple replication levels (quick/selective/complete)
- ✓ Troubleshooting guide
- ✓ External user testing
- ✓ Permanent DOIs
- ✓ Interactive web dashboard

This package should serve as a model for future computational social science research.
