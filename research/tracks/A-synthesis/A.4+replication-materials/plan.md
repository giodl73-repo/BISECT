# A.4 — Replication Materials and Data Archive

**Paper Type**: Replication Package and Technical Documentation
**Status**: Planned
**Target Venue**: Zenodo / Harvard Dataverse / Open ICPSR
**Format**: Multi-component archive (code, data, documentation, Docker containers)
**Target Audience**: Researchers, reviewers, students attempting to reproduce portfolio experiments

---

## Purpose

Create a **comprehensive replication package** that enables independent researchers to reproduce all experiments, figures, and results from the 28-paper congressional redistricting portfolio. This package provides complete transparency, facilitates validation of findings, and enables extensions by future researchers.

**Key Innovation**: Complete reproducibility infrastructure spanning empirical papers across tracks B, C, D, E, F, G, and H — covering 3 census years, 50 states, and 435 congressional districts — with step-by-step guides, automated validation, and containerized environments.

---

## Target Audience

1. **Journal Reviewers**: Verifying computational claims during peer review
2. **Replication Researchers**: Independently validating findings for replication studies
3. **Graduate Students**: Learning redistricting algorithms and extending the work
4. **Policy Analysts**: Running scenarios for specific states or parameters
5. **Open Science Community**: Examining transparency and reproducibility standards

---

## Package Objectives

1. **Enable exact reproduction**: Any researcher can regenerate every figure, table, and statistic from all empirical papers across tracks B–H
2. **Provide computational efficiency**: Pre-processed data and cached results reduce replication time from weeks to hours
3. **Support selective replication**: Users can reproduce single papers, specific states, or particular experiments
4. **Ensure cross-platform compatibility**: Works on Windows, Linux, macOS via Docker containers or statically linked binary
5. **Document dependencies**: Clear specifications for all software, libraries, and data requirements
6. **Validate outputs**: Automated checksums and comparison tools verify successful replication
7. **Lower barriers**: Minimize setup complexity so researchers without deep technical expertise can replicate

---

## Package Structure

The replication package consists of seven main components:

### Component 1: Source Code Repository
- **Location**: GitHub repository with permanent DOI
- **Contents**: Complete Rust workspace (`redist/crates/`) plus Python support scripts (`scripts/`)
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
  - `redistricting-base`: redist binary, Python 3.13, GeoPandas (1GB)
  - `redistricting-full`: Base + all data + cached results (60GB)
  - `redistricting-minimal`: Base only, users download data separately (1GB)
- **Benefits**: Eliminates "works on my machine" problems

### Component 4: Replication Guides
- **Format**: Markdown documents with step-by-step instructions
- **One guide per track**: B (algorithm), C (validation), D (VRA), E (experimental), F, G, H
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
  - District assignment files (exact match required via `redist label-verify`)
  - Compactness scores (floating-point tolerance +/-0.001)
  - Figures (pixel-level comparison or perceptual hash)
  - Statistical tables (numerical tolerance +/-0.01%)

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
- **Package Scope**: What can be replicated (all empirical papers across tracks B–H)
- **What's Not Included**: Papers A.0-A.3 are synthesis/documentation, not empirical
- **System Requirements**: Hardware (16GB RAM, 100GB disk), software (redist binary, Python 3.13+)
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

# Inside container, run test (Vermont -- small state, fast)
redist state --state VT --year 2020 --version test
```

**Option B: Pre-built Binary**
```bash
# Download statically linked redist binary from GitHub Releases
# (Linux/macOS/Windows builds available -- no Rust toolchain required)

# Download minimal test data
redist fetch --year 2020 --workers 8 --states VT DE

# Run test
redist state --state VT --year 2020 --version test
```

**Option C: Build from Source**
```bash
# Clone repository
git clone https://github.com/username/apportionment.git
cd apportionment

# Build with Cargo (requires Rust toolchain)
cargo build --release -p redist-cli

# Or build portable pure-Rust binary (no C compiler required)
cargo build --release --no-default-features -p redist-cli

# Download minimal test data
redist fetch --year 2020 --workers 8 --states VT DE

# Run test
redist state --state VT --year 2020 --version test
```

**Option D: Cloud Compute (Binder/Colab)**
- Click badge in repository README to launch Jupyter notebook environment
- Pre-configured with dependencies
- Limited to smaller experiments due to resource constraints

#### 2.2 Verify Installation
```bash
# Run test suite
cargo test -p redist-cli --lib -- --test-threads=1   # Rust unit tests
pytest tests/unit/ -v                                 # Python support script tests

# Run single-state test (Vermont, smallest state -- fastest verification)
redist state --state VT --year 2020 --version test

# Verify SHA chain on outputs
redist label-verify test --year 2020

# Check output
ls runs/test/2020/vermont/
```

#### 2.3 Quick Replication Example
```bash
# Reproduce Paper B.1 (Recursive Bisection) -- one-state smoke test
redist build b1_replication --year 2020 --workers 4

# Run full analysis
redist label-analyze b1_replication --year 2020 --types all

# Verify SHA chain matches reference
redist label-verify b1_replication --year 2020

# Expected output: all states pass hash verification
```

### Section 3: Data Acquisition (4-5 pages)

**Purpose**: Explain how to obtain all required data.

**Content**:

#### 3.1 Data Sources
- **Census Redistricting Data**: PL-94171 files from census.gov
- **TIGER Shapefiles**: Geographic boundaries for tracts/blocks
- **Demographic Data**: DHC files for race/ethnicity breakdowns
- **Election Data**: MIT Election Lab, Dave's Redistricting App (scripts/elections/)
- **Enacted Districts**: Official state redistricting plans (multiple sources)

#### 3.2 Fetching Data via the redist CLI
**Recommended**: Use the built-in fetch command to download and prepare census data:
```bash
# Download all 2020 data (all 50 states, ~55GB)
redist fetch --year 2020 --workers 8

# Download only specific states for selective replication
redist fetch --year 2020 --workers 8 --states CA TX FL NY PA

# Download all three census years
redist fetch --year 2020 --workers 8
redist fetch --year 2010 --workers 8
redist fetch --year 2000 --workers 8
```

**Pre-Packaged Archive**: Alternatively, download a complete pre-processed archive from Zenodo:
```bash
# Download full data archive (75GB, includes all years)
wget https://zenodo.org/record/XXXXX/files/apportionment_data_full_v1.0.tar.gz

# Extract to data/ directory
tar -xzf apportionment_data_full_v1.0.tar.gz -C data/

# Verify integrity (SHA-256 checksums)
python scripts/validation/verify_data_checksums.py
```

**Selective**: Download only specific years via Zenodo:
```bash
# Download only 2020 data for 5 states (10GB)
# See Zenodo record for per-state archive files
```

#### 3.3 Election Data (Python Scripts)
Election results data requires the Python support scripts (scripts/elections/ remains Python):
```bash
# Download election data overlays
python scripts/elections/download_election_data.py --year 2020
```

#### 3.4 Data Directory Structure
```
data/
+-- 2000/
|   +-- redistricting/          # Raw PL-94171 files
|   +-- tiger/tracts/           # Census tract shapefiles
|   +-- demographics/           # DHC demographic data
|   +-- elections/              # Election results
+-- 2010/ [same structure]
+-- 2020/ [same structure]

runs/                           # Pipeline outputs (gitignored)
analysis/                       # Analysis outputs (gitignored)
reports/                        # Report outputs (gitignored)
```

### Section 4: Experiment-to-Paper Mapping (6-8 pages)

**Purpose**: Map each empirical paper (tracks B–H) to specific experiments.

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
# Build config: configs/b1_replication.yml
# (structure: standard-bisect, weights: geographic, search: convergence)

# Run full 50-state build (2-4 hours, 8 workers)
redist build b1_replication --year 2020 --workers 8

# Run full analysis suite
redist label-analyze b1_replication --year 2020 --types all

# Generate HTML + JSON report
redist label-report b1_replication --year 2020 --format html json

# Verify SHA chain
redist label-verify b1_replication --year 2020
```

**Input Data Required**:
- `data/2020/` (census PL-94171 + TIGER, fetched via `redist fetch`)
- `configs/b1_replication.yml`

**Expected Outputs**:
- `runs/b1_replication/2020/` (district assignments, one file per state)
- `analysis/b1_replication/2020/` (compactness, demographic, political metrics)
- `reports/b1_replication/2020/` (HTML report, JSON data)

**Validation**:
```bash
# Verify SHA chain matches reference hashes
redist label-verify b1_replication --year 2020

# Expected output:
# [OK] vermont          SHA chain valid
# [OK] delaware         SHA chain valid
# ... (50/50 states pass)
```

**Time Estimate**: 3-5 hours (full pipeline) or 10 minutes (using cached districts)

**Key Results to Verify**:
- National mean Polsby-Popper: 0.367 (paper reports 0.367)
- Illinois improvement: 174% over enacted (paper reports 174%)
- Minnesota mean PP: 0.468 (paper reports 0.468)

**Paper-Specific Notes**: This is the foundation paper — all other papers build on this baseline.

---

#### Paper B.2 — Edge-Weighted Bisection

**Title**: Edge-Weighted Graph Partitioning for Compact Congressional Districts

**Core Experiment**: Comparison of edge-weighted vs unweighted partitioning across 50 states

**Command Sequence**:
```bash
# Build config: configs/b2_unweighted.yml (weights: unweighted)
redist build b2_unweighted --year 2020 --workers 8

# Build config: configs/b2_edgeweighted.yml (weights: geographic)
redist build b2_edgeweighted --year 2020 --workers 8

# Analyze both runs
redist label-analyze b2_unweighted   --year 2020 --types all
redist label-analyze b2_edgeweighted --year 2020 --types all

# Verify both
redist label-verify b2_unweighted   --year 2020
redist label-verify b2_edgeweighted --year 2020
```

**Input Data Required**: Same as B.1

**Expected Outputs**:
- `runs/b2_unweighted/2020/` and `runs/b2_edgeweighted/2020/`
- `analysis/b2_*/2020/` (comparative metrics)

**Validation**:
```bash
redist label-verify b2_unweighted   --year 2020
redist label-verify b2_edgeweighted --year 2020
```

**Time Estimate**: 6-8 hours (both runs) or 15 minutes (using cached results)

**Key Results to Verify**:
- Edge-weighted mean PP: 0.367
- Unweighted mean PP: 0.235
- Improvement: 56% (paper reports 56%)

---

**[Continue this pattern for all empirical papers: B.3–T.10, C.1–C.5, D.0–D.3, E.1–E.5, F, G, H series]**

---

### Section 5: Full Replication Workflows (4-5 pages)

**Purpose**: Provide end-to-end workflows for reproducing entire tracks.

#### 5.1 Track B: Algorithm Design (Complete Replication)

**Goal**: Reproduce all papers in Track B showing algorithm design decisions

**Workflow**:
```bash
# 1. Fetch data for all three census years
redist fetch --year 2020 --workers 8
redist fetch --year 2010 --workers 8
redist fetch --year 2000 --workers 8

# 2. Run baseline experiment (unweighted)
redist build b_unweighted_baseline --year 2020 --workers 8

# 3. Run core algorithm experiments (each has a config in configs/)
redist build b1_replication --year 2020 --workers 8
redist build b2_replication --year 2020 --workers 8
# ... (one build per paper)

# 4. Analyze all Track B labels
redist label-analyze b1_replication --year 2020 --types all
redist label-analyze b2_replication --year 2020 --types all
# ...

# 5. Verify all Track B labels
redist label-verify b1_replication --year 2020
redist label-verify b2_replication --year 2020
# ...

# 6. Generate reports
redist label-report b1_replication --year 2020 --format html json
# ...
```

**Time Estimate**: 12-16 hours (parallelized) or 40-60 hours (sequential)

**Outputs**: Complete replication of all Track B papers

#### 5.2 Track C: Validation (Complete Replication)

**Goal**: Reproduce all papers showing multi-faceted validation

**Workflow**:
```bash
# 1. Run cross-census builds (2000, 2010, 2020)
redist build c_validation_2020 --year 2020 --workers 8
redist build c_validation_2010 --year 2010 --workers 8
redist build c_validation_2000 --year 2000 --workers 8

# 2. Analyze all census years
redist label-analyze c_validation_2020 --year 2020 --types all
redist label-analyze c_validation_2010 --year 2010 --types all
redist label-analyze c_validation_2000 --year 2000 --types all

# 3. Verify all builds
redist label-verify c_validation_2020 --year 2020
redist label-verify c_validation_2010 --year 2010
redist label-verify c_validation_2000 --year 2000

# 4. Run paper-specific experiments (MAUP, partisan, temporal)
redist build c_maup_tracts     --year 2020 --workers 8
redist build c_maup_blocks     --year 2020 --workers 8
redist label-analyze c_maup_tracts --year 2020 --types all
redist label-analyze c_maup_blocks --year 2020 --types all
```

**Time Estimate**: 20-30 hours (multiple census years, multiple resolutions)

**Outputs**: Complete replication of all Track C papers

#### 5.3 Track D: VRA Compliance (Complete Replication)

**Goal**: Reproduce all papers on Voting Rights Act compliance

**Workflow**:
```bash
# 1. Run VRA baseline (standard-bisect, geographic weights)
redist build d_vra_baseline --year 2020 --workers 8

# 2. Run VRA-aligned builds (weights: vra-aligned, structure: ratio-optimal-vra)
redist build d_vra_aligned  --year 2020 --workers 8

# 3. Analyze and verify
redist label-analyze d_vra_baseline --year 2020 --types all
redist label-analyze d_vra_aligned  --year 2020 --types all
redist label-verify  d_vra_baseline --year 2020
redist label-verify  d_vra_aligned  --year 2020

# 4. Run threshold sensitivity experiments
redist build d_vra_40pct --year 2020 --workers 8
redist build d_vra_45pct --year 2020 --workers 8
redist build d_vra_50pct --year 2020 --workers 8
```

**Time Estimate**: 10-15 hours

**Outputs**: Complete replication of all Track D papers

#### 5.4 Track E: Experimental Alternatives (Complete Replication)

**Goal**: Reproduce all papers exploring alternative systems

**Workflow**:
```bash
# 1. Run multi-member / n-way partition experiments
redist build e_nway_districts --year 2020 --workers 8

# 2. Run county-weight experiments (weights: county)
redist build e_county_weights --year 2020 --workers 8

# 3. Run proportional experiments (weights: proportional)
redist build e_proportional   --year 2020 --workers 8

# 4. Analyze all
redist label-analyze e_nway_districts --year 2020 --types all
redist label-analyze e_county_weights --year 2020 --types all
redist label-analyze e_proportional   --year 2020 --types all

# 5. Verify all
redist label-verify e_nway_districts --year 2020
redist label-verify e_county_weights --year 2020
redist label-verify e_proportional   --year 2020
```

**Time Estimate**: 15-20 hours

**Outputs**: Complete replication of all Track E papers

#### 5.5 Tracks F, G, H (Complete Replication)

Follow the same pattern as tracks above: one config per paper, `redist build` -> `redist label-analyze` -> `redist label-verify`. Refer to the per-track replication guides (Component 4 of the package) for paper-specific configs and expected outputs.

### Section 6: Validation and Verification (3-4 pages)

**Purpose**: Explain how to verify replication succeeded.

#### 6.1 Built-in Validation via SHA Chain

The `redist label-verify` command computes SHA-256 hashes over all district assignment files and compares them against the reference chain embedded in the label manifest. This is the primary validation mechanism.

```bash
# Verify single label
redist label-verify b1_replication --year 2020

# Expected output:
# [OK] alabama          SHA chain valid
# [OK] alaska           SHA chain valid
# ...
# [OK] wyoming          SHA chain valid
# PASSED: 50/50 states verified
```

#### 6.2 Automated Validation Scripts

Supplementary Python validation scripts check outputs beyond the SHA chain:

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
Paper: B.1 -- Recursive Bisection
Date: 2026-02-08

SHA Chain Verification: PASSED (50/50 states)
File Existence:         PASSED (156/156 files present)
Format Correctness:     PASSED (all files valid)
Numerical Tolerance:    PASSED (all metrics within +/-0.001)
Visual Similarity:      PASSED (all figures > 0.98 perceptual similarity)
Statistical Equiv:      PASSED (all distributions p > 0.05)

OVERALL: REPLICATION SUCCESSFUL
```

#### 6.3 Manual Verification Steps

For users wanting to manually verify key results:

1. **Check Key Metrics**: Compare summary statistics from analysis outputs
```bash
# Your replication
cat analysis/b1_replication/2020/metrics/compactness_summary.csv

# Reference values (from pre-computed cache)
cat reference_outputs/B1/2020/metrics/compactness_summary.csv

# Should match within +/-0.001
```

2. **Spot Check States**: Verify specific high-impact states
- California (52 districts): Largest state, most complex
- Illinois (17 districts): Biggest improvement claim (174%)
- Vermont (1 district): Smallest state, simplest case
- Texas (38 districts): Large population, diverse geography

#### 6.4 Troubleshooting Common Issues

**Problem**: SHA chain mismatch on district assignments
**Cause**: METIS randomness (tie-breaking in graph partitioning)
**Solution**: Configs use fixed seeds by default; verify `seed` field in configs/*.yml matches reference

**Problem**: Figure colors don't match exactly
**Cause**: Different matplotlib/backend versions
**Solution**: Use Docker container with frozen Python environment

**Problem**: `redist build` exits with error on a specific state
**Cause**: Missing data for that state/year
**Solution**: Run `redist fetch --year 2020 --states <state>` to re-download

**Problem**: Validation script reports missing files
**Cause**: Build did not complete successfully
**Solution**: Check `runs/<label>/<year>/<state>/error.log` for failures; re-run the failing state with `redist state --state <state> --year 2020 --version <label>`

### Section 7: Advanced Topics (3-4 pages)

#### 7.1 Running on HPC Clusters

For users with access to high-performance computing:

```bash
# The --workers flag maps directly to parallel METIS threads
redist build b1_replication --year 2020 --workers 32

# For SLURM: submit one job per state, then aggregate
# See scripts/hpc/submit_state_array.sh for a template SLURM array job
sbatch scripts/hpc/run_full_replication.slurm
```

**Benefits**: Complete replication in ~2 hours instead of 30 hours

#### 7.2 Modifying Experiments

Researchers wanting to extend the work:

**Example: Test different weight schemes**
```bash
# Create a custom config: configs/custom_county.yml
# (structure: standard-bisect, weights: county, search: convergence)

# Run custom experiment
redist build custom_county --year 2020 --workers 8

# Analyze and compare to baseline
redist label-analyze custom_county --year 2020 --types all
```

**Three-layer compositor** (structure / weights-override / search) controls all algorithm variants. See `docs/concepts/` for the full parameter space.

#### 7.3 Generating Paper-Ready Figures

All figures in papers can be regenerated via `redist label-report`:

```bash
# Generate HTML report with all figures for Paper B.1
redist label-report b1_replication --year 2020 --format html

# Reports are saved under reports/<label>/<year>/
# Figures are embedded in HTML and also available as separate PNG/PDF files
```

#### 7.4 Exporting Results for Analysis

Export data to other formats for external analysis:

```bash
# Export to R-friendly CSV format (from analysis/ outputs)
# District CSV files are plain CSV -- directly readable by R, Stata, etc.
cat analysis/b1_replication/2020/metrics/compactness_summary.csv

# Export shapefiles for GIS software (generated by label-analyze)
ls analysis/b1_replication/2020/shapefiles/
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

**Primary Computational Tool**:
- `redist` binary (Rust, statically linked) — handles all redistricting computation
  - Pre-built binaries available on GitHub Releases (no Rust toolchain required)
  - Alternatively: build from source with Cargo/Rust toolchain

**For Building from Source**:
- Cargo/Rust toolchain (stable, 2021 edition or later)
- C compiler (optional: only needed for `--metis-engine c-ffi` default build)
- `cargo build --no-default-features` builds the portable pure-Rust binary without C dependency

**Secondary Tools (Data Preparation Only)**:
- Python 3.13+ (for `scripts/elections/` and `scripts/dashboard/` only)
- GeoPandas, pandas, matplotlib (see scripts/requirements.txt)
- `redist fetch` handles all census data download (no separate Python download scripts needed)

**Optional but Recommended**:
- Docker 20.10+ (containerization — simplest cross-platform setup)
- Jupyter Lab 4.0+ (interactive exploration of analysis outputs)

#### 8.3 Operating System Compatibility

**Linux (Recommended)**:
- Ubuntu 24.04 LTS or Debian 12+
- Pre-built Linux binary available; or `cargo build --release`
- Best performance for parallel processing

**Windows**:
- Windows 10/11 (native)
- Pre-built Windows binary available
- No WSL2 required for the redist binary
- Note: console output is ASCII-only (no Unicode)

**macOS**:
- macOS 13+ (Ventura or later)
- Pre-built macOS binary available (x86-64 and ARM/Apple Silicon)
- ARM (M1/M2) fully supported via native Rust build

### Section 9: Archive Distribution (2 pages)

#### 9.1 Archive Components

The replication package is distributed as multiple archives to accommodate different user needs:

**Archive 1: Minimal (2 GB)**
- redist binary (pre-built for Linux/Windows/macOS)
- Source code and configs
- Documentation
- Test data (Vermont, Delaware only)
- Docker configurations
- Quick start scripts

**Archive 2: Complete Code + Processed Data (80 GB)**
- All source code and configs
- All processed data (ready for pipeline)
- Reference outputs (cached results)
- Validation scripts

**Archive 3: Raw Data (40 GB)**
- Original census files (PL-94171)
- Original TIGER shapefiles
- For users wanting to process from scratch via `redist fetch`

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

**Tertiary**: GitHub Releases (binary + code only)
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
```bash
# Replication Note: This config reproduces the main experiment from Paper B.1
# Label: b1_replication
# Config: configs/b1_replication.yml
# Expected output: runs/b1_replication/2020/  analysis/b1_replication/2020/
# Runtime: ~3-5 hours on 8 cores
# Validation: redist label-verify b1_replication --year 2020
```

### Reproducibility Principles
1. **Deterministic**: Fixed seeds in all configs (seed field in configs/*.yml)
2. **Versioned**: Pin binary version; configs are version-controlled in configs/
3. **Containerized**: Provide Docker images with frozen environments
4. **Documented**: Every config has usage instructions; `redist --help` for CLI reference
5. **Validated**: SHA chain via `redist label-verify`; supplementary Python validators
6. **Transparent**: All data sources and processing steps documented

---

## Success Criteria

This replication package succeeds if:

1. An independent researcher can reproduce any paper's main result in <=4 hours
2. `redist label-verify` confirms SHA chain matches for >95% of outputs
3. Docker containers eliminate "works on my machine" problems
4. Documentation is clear enough that a graduate student can follow it
5. Selective replication (single paper) requires <50GB disk and <8GB RAM
6. All data sources are permanently archived with DOIs
7. Troubleshooting guide covers >90% of actual user problems
8. Reviewers can verify key claims in <1 hour using cached results

---

## Target Metrics

- **Setup Time**: <30 minutes (Docker or pre-built binary) or <2 hours (build from source)
- **Minimal Replication**: <1 hour (verify cached results via `redist label-verify`)
- **Single Paper Replication**: 2-6 hours (reproduce one paper)
- **Full Replication**: 20-30 hours (reproduce all empirical papers across tracks B-H)
- **HPC Replication**: 2-4 hours (with 50+ cores)
- **Validation Time**: <15 minutes per paper
- **User Support**: Respond to GitHub issues within 48 hours

---

## Dependencies

**This package depends on**:
- All empirical papers (tracks B–H): Provides experiments to replicate
- redist Rust binary and source: `redist/crates/`
- Python support scripts: `scripts/elections/`, `scripts/dashboard/`
- Data infrastructure: Census API via `redist fetch`, TIGER downloads
- Testing framework: Rust unit tests + Python unit tests

**Papers that depend on this**:
- **All Track B-H papers**: Should reference this as supplementary material for replication
- **A.0 (Synthesis)**: Will highlight replication package as evidence of transparency
- **A.3 (Visualization)**: Links to web dashboard component

---

## Next Steps for Implementation

1. **Create validation scripts** (scripts/validation/)
   - validate_replication.py (single paper validator)
   - validate_track_[B|C|D|E|F|G|H].py (track validators)
   - validate_all.py (comprehensive validation)
   - verify_data_checksums.py (data integrity checker)

2. **Create per-paper configs** (configs/)
   - One YAML config per empirical paper (e.g., configs/b1_replication.yml)
   - Fixed seeds, documented structure/weights/search choices
   - Aligned with three-layer compositor parameters

3. **Build Docker containers** (docker/)
   - Dockerfile.base (redist binary + Python support scripts)
   - Dockerfile.full (with data)
   - docker-compose.yml (multi-container setup)

4. **Write comprehensive documentation**
   - REPLICATION_GUIDE.md (main guide, 30-50 pages)
   - QUICK_START.md (5-page quick start)
   - TROUBLESHOOTING.md (common issues)
   - VIDEO_TUTORIALS.md (links to walkthrough videos)

5. **Generate reference outputs**
   - Run all experiments with fixed seeds
   - Compute SHA chains via `redist label-verify` and store manifests
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
- Create per-paper configs
- Generate reference outputs and SHA manifests
- Create validation scripts

**Phase 2 (Docker Containers)**: 1 week
- Build Docker images with redist binary + Python support scripts
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
- The statically linked `redist` binary eliminates most platform-specific issues without requiring Docker
- Docker containers provide an additional layer of environment isolation for Python support scripts
- `redist label-verify` provides built-in SHA chain validation, reducing the burden on external tools
- Web dashboard provides exploration without requiring local installation
- Permanent archival with DOIs ensures long-term availability

**Gold Standard Elements**:
- Complete source code with version control
- All data permanently archived
- Statically linked binary + containerized computational environment
- Built-in SHA chain validation (`redist label-verify`) + supplementary automated tools
- Step-by-step documentation
- Multiple replication levels (quick/selective/complete)
- Troubleshooting guide
- External user testing
- Permanent DOIs
- Interactive web dashboard

This package should serve as a model for future computational social science research.
