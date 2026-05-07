# Replication Materials Repository — Plan

**Artifact Type**: PP3.3 (Optional - Future Work)
**Goal**: Open-source repository enabling full reproduction of all 10 papers
**Estimated Effort**: 2-3 weeks
**Status**: Not started

---

## Objective

Create a comprehensive, well-documented repository that enables:
- **Full replication**: Anyone can reproduce all 435 districts × 3 census years
- **Method comparison**: Researchers can test alternative algorithms
- **Policy adoption**: State commissions can use code for production redistricting
- **Educational use**: Students can learn algorithmic redistricting through examples

**Impact**: Strengthens reproducibility, increases citations, enables real-world deployment

---

## Repository Structure

```
redistricting-recursive-bisection/
├── README.md                       # Quick start guide
├── CITATION.md                     # How to cite papers
├── LICENSE                         # MIT or BSD license
├── requirements.txt                # Python dependencies
├── environment.yml                 # Conda environment
├── Makefile                        # Automated workflows
│
├── data/                           # Raw census data
│   ├── 2000/
│   │   ├── redistricting/          # Tract populations
│   │   ├── tiger/tracts/           # Tract shapefiles
│   │   └── demographics/           # Minority demographics
│   ├── 2010/ (same structure)
│   └── 2020/ (same structure)
│
├── outputs/                        # Generated districts
│   ├── v1/
│   │   ├── 2000/
│   │   │   ├── california/
│   │   │   │   ├── districts.csv
│   │   │   │   ├── map.png
│   │   │   │   └── metrics.json
│   │   │   └── ...50 states
│   │   ├── 2010/ (same)
│   │   └── 2020/ (same)
│   └── README.md                   # Output format documentation
│
├── src/
│   ├── partition/
│   │   ├── recursive_bisection.py  # Core algorithm
│   │   ├── edge_weighted.py        # Edge-weighting variants
│   │   ├── metis_wrapper.py        # METIS interface
│   │   └── vra_compliance.py       # VRA-aware methods
│   ├── data/
│   │   ├── adjacency.py            # Graph construction
│   │   ├── census.py               # Census data loading
│   │   └── geometries.py           # Shapefile processing
│   ├── analysis/
│   │   ├── compactness.py          # Polsby-Popper, Reock, etc.
│   │   ├── demographics.py         # Minority concentration
│   │   └── political.py            # Partisan analysis (post-hoc)
│   └── visualization/
│       ├── maps.py                 # District map generation
│       └── dashboards.py           # HTML dashboard
│
├── scripts/
│   ├── download_data.py            # Fetch census data
│   ├── preprocess.py               # Build adjacency graphs
│   ├── run_redistricting.py        # Main pipeline
│   ├── compare_methods.py          # Recursive vs n-way vs adaptive
│   └── generate_paper_figures.py   # Reproduce all paper figures
│
├── notebooks/
│   ├── 01_quickstart.ipynb         # 5-minute demo (Vermont)
│   ├── 02_single_state.ipynb       # Full state walkthrough
│   ├── 03_vra_compliance.ipynb     # VRA methods tutorial
│   ├── 04_edge_weighting.ipynb     # Parameter tuning guide
│   └── 05_comparison.ipynb         # Method comparison
│
├── tests/
│   ├── unit/                       # Unit tests
│   ├── integration/                # Integration tests
│   └── e2e/                        # End-to-end tests
│
├── docs/
│   ├── installation.md             # Setup instructions
│   ├── quickstart.md               # 5-minute tutorial
│   ├── api/                        # API documentation
│   ├── tutorials/                  # Step-by-step guides
│   │   ├── basic_usage.md
│   │   ├── vra_compliance.md
│   │   ├── edge_weighting.md
│   │   └── custom_methods.md
│   ├── papers/                     # Per-paper replication
│   │   ├── recursive-bisection.md
│   │   ├── edge-weighted.md
│   │   └── ... (10 papers)
│   └── faq.md                      # Common questions
│
└── examples/
    ├── alabama_2020.py             # VRA compliance example
    ├── california_2020.py          # Large state example
    ├── vermont_2020.py             # Small state example
    └── custom_algorithm.py         # Extending the framework
```

---

## Core Components

### 1. Data Pipeline

**Download script** (`scripts/download_data.py`):
```bash
python scripts/download_data.py --year 2020 --states CA TX NY
# Downloads census tracts, demographics, shapefiles
```

**Preprocessing** (`scripts/preprocess.py`):
```bash
python scripts/preprocess.py --year 2020 --state california
# Builds adjacency graph, merges geometries, validates data
```

### 2. Redistricting Pipeline

**Single state**:
```bash
python scripts/run_redistricting.py \
  --state california \
  --year 2020 \
  --method recursive-bisection \
  --edge-weight 5.0 \
  --minority-threshold 0.40
```

**Full 50-state run**:
```bash
python scripts/run_redistricting.py \
  --all-states \
  --year 2020 \
  --method recursive-bisection \
  --workers 4
# Parallel execution, ~1 hour runtime
```

**Method comparison**:
```bash
python scripts/compare_methods.py \
  --state alabama \
  --year 2020 \
  --methods recursive,nway,adaptive \
  --output comparison.json
```

### 3. Analysis Tools

**Compactness metrics**:
```python
from src.analysis import compactness
pp_score = compactness.polsby_popper(district_geometry)
reock = compactness.reock(district_geometry)
```

**VRA compliance check**:
```python
from src.analysis import demographics
mm_districts = demographics.count_majority_minority(
    districts,
    threshold=0.50
)
```

**Visualization**:
```python
from src.visualization import maps
maps.create_district_map(
    state="california",
    districts=results,
    output="ca_2020.png"
)
```

### 4. Jupyter Notebooks

**Quickstart** (01_quickstart.ipynb):
- 5-minute demo using Vermont (1 district)
- Load data → build graph → partition → visualize
- No census download required (uses cached data)

**Full walkthrough** (02_single_state.ipynb):
- Complete pipeline for California
- Census data download
- Graph construction
- Multiple algorithm variants
- Compactness analysis
- VRA compliance checking

**VRA tutorial** (03_vra_compliance.ipynb):
- Demonstrates edge-weighted partitioning
- Parameter tuning (weight factors, thresholds)
- Alabama case study (0 MM → 2 MM districts)
- Pareto frontier analysis

---

## Documentation Requirements

### README.md (Top-Level)

**Sections**:
1. **One-paragraph summary**: What is this repository?
2. **Quick start**: 5 commands to run Vermont example
3. **Installation**: Dependencies, conda environment, METIS setup
4. **Usage examples**: Basic redistricting commands
5. **Paper replication**: How to reproduce each of 10 papers
6. **Citation**: BibTeX entries for all papers
7. **License**: MIT/BSD
8. **Contact**: Author email, issue tracker

### Per-Paper Replication Guides

Each paper gets a dedicated `docs/papers/{paper-name}.md`:
- **Objective**: What does this paper demonstrate?
- **Data required**: Which census years and states?
- **Commands**: Exact command-line invocations to reproduce
- **Expected output**: What files/figures should be generated?
- **Runtime**: How long does replication take?
- **Figures**: How to regenerate each figure/table

**Example** (`docs/papers/edge-weighted-bisection.md`):
```markdown
# Replicating: Edge-Weighted Recursive Bisection

## Objective
Reproduce 56% compactness improvement findings from paper.

## Data Required
- 2020 census tracts (50 states)
- TIGER/Line shapefiles

## Commands
```bash
# Run unweighted baseline
python scripts/run_redistricting.py --year 2020 --method recursive --edge-weight 1.0

# Run edge-weighted variant
python scripts/run_redistricting.py --year 2020 --method recursive --edge-weight 5.0

# Generate comparison figures
python scripts/generate_paper_figures.py --paper edge-weighted-bisection
```

## Expected Output
- `outputs/v1/2020/{state}/districts.csv` for all 50 states
- `figures/edge-weighted-bisection/` containing all paper figures
- Runtime: ~1 hour on 4-core machine

## Key Figures
- Figure 3: Compactness improvement by state (generated from outputs)
- Figure 5: Alabama case study map (requires shapefile rendering)
```

---

## Installation & Dependencies

### Python Requirements
```
# requirements.txt
numpy>=1.24.0
scipy>=1.10.0
pandas>=2.0.0
geopandas>=0.13.0
shapely>=2.0.0
matplotlib>=3.7.0
pymetis>=2023.1
networkx>=3.1
tqdm>=4.65.0
```

### System Dependencies
- **METIS**: Graph partitioning library (C library, Python bindings)
- **GDAL**: Geospatial data abstraction library
- **Proj**: Coordinate transformation library

### Conda Environment
```yaml
# environment.yml
name: redistricting
channels:
  - conda-forge
dependencies:
  - python=3.11
  - numpy
  - scipy
  - pandas
  - geopandas
  - shapely
  - matplotlib
  - metis
  - pymetis
  - networkx
  - tqdm
  - pytest
```

### Quick Setup
```bash
conda env create -f environment.yml
conda activate redistricting
make test  # Verify installation
```

---

## Testing Strategy

### Unit Tests (tests/unit/)
- Test individual functions (adjacency construction, compactness metrics)
- Mock METIS calls for speed
- ~135 tests, <5 seconds runtime

### Integration Tests (tests/integration/)
- Test full pipeline components together
- Use Vermont (smallest state) for speed
- ~24 tests, ~30 seconds runtime

### End-to-End Tests (tests/e2e/)
- Full redistricting runs on small states
- Verify output format, metrics, reproducibility
- ~10 tests, ~5 minutes runtime

### Continuous Integration
```yaml
# .github/workflows/tests.yml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: conda-incubator/setup-miniconda@v2
      - run: conda env create -f environment.yml
      - run: conda activate redistricting && pytest tests/
```

---

## Data Availability

### Option 1: Zenodo Archive (Recommended)
- Upload preprocessed data (adjacency graphs, shapefiles)
- ~40GB compressed
- Persistent DOI
- Free for academic use

### Option 2: GitHub LFS
- Store large files in Git LFS
- Download on-demand
- GitHub storage limits apply

### Option 3: Download Scripts
- Provide scripts to fetch from Census Bureau
- Users download directly (most flexible)
- Slower initial setup but always up-to-date

**Recommended approach**:
- GitHub repo + Zenodo archive for processed data
- Download scripts as fallback

---

## License Selection

### MIT License (Recommended)
- Permissive, widely adopted
- Allows commercial use
- Requires attribution
- Compatible with most academic work

### GPL v3
- Copyleft: derivatives must be open-source
- Ensures code remains free
- May deter commercial adoption

### BSD 3-Clause
- Similar to MIT
- Includes non-endorsement clause
- Good for academic projects

**Recommendation**: MIT for maximum adoption

---

## Deployment & Maintenance

### Pre-Release Checklist
- [ ] All tests pass (unit, integration, e2e)
- [ ] Documentation complete (README, tutorials, API)
- [ ] Example notebooks run without errors
- [ ] Per-paper replication guides verified
- [ ] License file included
- [ ] CITATION.md with all BibTeX entries
- [ ] Release notes written

### Release Process
1. Create GitHub release with semantic version (v1.0.0)
2. Upload to Zenodo (get DOI)
3. Announce on Twitter, mailing lists
4. Submit to Papers with Code
5. Post on r/datascience, r/政治

### Maintenance Plan
- **Bug fixes**: Respond within 1 week
- **Feature requests**: Triage monthly
- **Census updates**: Update when 2030 census released
- **METIS updates**: Test compatibility with new versions

---

## Impact Metrics

Track repository adoption through:
- **GitHub stars**: Measure community interest
- **Citations**: Papers citing code repository
- **Forks**: Independent development
- **Issues/PRs**: Community engagement
- **Downloads**: Zenodo download count
- **Real-world use**: State commissions using code

**Success criteria**:
- 100+ stars in first year
- 10+ citations within 2 years
- 1+ state commission adopting methods

---

## Next Actions

- [ ] Clean and document existing Python scripts
- [ ] Extract core algorithm into src/partition/
- [ ] Create quickstart notebook (Vermont example)
- [ ] Write comprehensive README
- [ ] Set up pytest test suite
- [ ] Prepare Zenodo data archive
- [ ] Create per-paper replication guides
- [ ] Get feedback from 2-3 potential users
- [ ] Public release on GitHub

---

**Created**: 2026-02-08
**Panel Reference**: REVIEW_PANEL.md Section V, PP3.3
**Related Artifacts**: gerry-portfolio-guide/ (documentation), gerry-synthesis-metapaper/ (citations)
