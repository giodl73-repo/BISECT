# Academic Paper: Algorithmic Redistricting via Recursive Bisection

Professional 8-page paper on mathematical approach to congressional redistricting.

## Paper Structure

### Title
**"Algorithmic Redistricting: Applying Huntington-Hill Principles to District Boundary Design"**

### Sections (Target: 8 pages)

1. **Introduction** (1 page)
   - Current redistricting challenges (gerrymandering, partisanship)
   - Need for objective, mathematical approaches

2. **Historical Context: Huntington-Hill Method** (1 page)
   - How Huntington-Hill solved apportionment for 70+ years
   - Mathematical fairness principles
   - Why it's politically accepted

3. **Methodology: Recursive Bisection** (2 pages)
   - Graph-based representation of census tracts
   - METIS graph partitioning algorithm
   - Recursive bisection process (odd/even handling)
   - Technical implementation details

4. **Results: 2020 Census Analysis** (2.5 pages)
   - 50-state implementation statistics
   - Population deviation analysis
   - Compactness metrics (Polsby-Popper, Reock)
   - Example: 8-district state visualization (3 rounds)

5. **Political and Demographic Analysis** (1 page)
   - Partisan lean distribution
   - Competitive vs safe districts
   - Demographic diversity metrics

6. **Discussion and Conclusions** (0.5 pages)
   - Advantages of algorithmic approach
   - Limitations and future work
   - Policy implications

## Directory Structure

```
paper/
├── README.md                    # This file
├── analysis/                    # Analysis scripts
│   ├── compute_population_stats.py       # Population deviation analysis
│   ├── compute_compactness_stats.py      # Compactness scores
│   ├── compute_political_stats.py        # Political analysis
│   ├── compute_demographic_stats.py      # Demographic analysis
│   ├── select_example_state.py           # Find good 8-district state
│   └── generate_all_statistics.py        # Master script
├── figures/                     # Generated figures
│   ├── round_progression_*.png          # 8-district state rounds
│   ├── population_deviation_hist.png    # National histogram
│   ├── compactness_distribution.png     # Compactness scores
│   ├── political_lean_pie.png           # Partisan distribution
│   └── demographic_diversity.png        # Diversity metrics
├── data/                        # Extracted statistics
│   ├── population_stats.csv
│   ├── compactness_stats.csv
│   ├── political_stats.csv
│   └── demographic_stats.csv
├── sections/                    # LaTeX sections
│   ├── 01_introduction.tex
│   ├── 02_huntington_hill.tex
│   ├── 03_methodology.tex
│   ├── 04_results.tex
│   ├── 05_analysis.tex
│   └── 06_discussion.tex
├── paper.tex                    # Main LaTeX file
├── references.bib               # Bibliography
├── build_paper.py               # Assemble and compile paper
└── paper.pdf                    # Final output
```

## Data Sources

All data from: `outputs/us_2020_v1/`

### National Statistics
- `us_district_summary.csv` - All 435 districts
- `us_rounds_hierarchy.csv` - Bisection hierarchy

### State-Level Data
- `states/{state}/district_summary.csv` - District metrics
- `states/{state}/political_analysis/district_political_2020.csv`
- `states/{state}/demographic_analysis/district_demographics.csv`
- `states/{state}/maps/round_*.png` - Round visualizations
- `states/{state}/rounds_hierarchy.csv` - State bisection tree

## Key Statistics to Extract

### Population Deviation
- Mean absolute deviation across all 435 districts
- Maximum deviation
- Standard deviation
- Histogram distribution
- Compare to constitutional ±1% requirement

### Compactness
- Polsby-Popper scores (mean, median, distribution)
- Reock scores (mean, median, distribution)
- Compare to actual congressional districts
- State-by-state comparison

### Political Analysis
- Distribution across 5 categories:
  - Strong D (margin > 15%)
  - Lean D (margin 5-15%)
  - Toss-up (margin -5% to 5%)
  - Lean R (margin -15% to -5%)
  - Strong R (margin < -15%)
- Competitive district count (toss-ups)
- Safe district count (Strong D/R)

### Demographic Analysis
- Majority-minority districts (non-white > 50%)
- Diversity index distribution
- Gender balance
- Racial/ethnic composition

## Example State Selection

**Criteria for 8-district state**:
- Exactly 8 districts (3 bisection rounds: 1→2→4→8)
- Interesting geography (not just rectangles)
- Clear round progression in visualizations
- Moderate compactness scores

**Candidates**:
- **Minnesota** (8 districts) - Great Lakes, interesting geography
- **Colorado** (8 districts) - Mountains, varied terrain
- **Indiana** (9 districts) - Close, but 9 not 8
- **Louisiana** (6 districts) - Too few
- **South Carolina** (7 districts) - Too few

**Likely choice**: Minnesota or Colorado (verify which has better visual rounds)

## Analysis Scripts

### 1. compute_population_stats.py

```python
# Load us_district_summary.csv
# Calculate:
# - ideal_pop = total_us_pop / 435
# - deviation = (district_pop - ideal_pop) / ideal_pop * 100
# - Statistics: mean, median, std, max, min
# - Generate histogram
# Output: data/population_stats.csv, figures/population_deviation_hist.png
```

### 2. compute_compactness_stats.py

```python
# Load us_district_summary.csv
# Extract Polsby-Popper and Reock scores
# Calculate statistics per score type
# Generate distribution plots
# Output: data/compactness_stats.csv, figures/compactness_distribution.png
```

### 3. compute_political_stats.py

```python
# Load all state political_analysis files
# Aggregate across 435 districts (48 states with data)
# Count districts per lean category
# Generate pie chart and bar chart
# Output: data/political_stats.csv, figures/political_lean_pie.png
```

### 4. compute_demographic_stats.py

```python
# Load all state demographic_analysis files
# Calculate:
# - Number of majority-minority districts
# - Diversity index distribution
# - Racial/ethnic composition statistics
# Output: data/demographic_stats.csv, figures/demographic_diversity.png
```

### 5. select_example_state.py

```python
# Find 8-district states (exactly)
# Load their round visualizations
# Check quality (clarity, geography)
# Recommend best example
# Output: Selected state name
```

### 6. generate_all_statistics.py

```python
# Master script that runs all analyses in order:
# 1. Population stats
# 2. Compactness stats
# 3. Political stats
# 4. Demographic stats
# 5. Select example state
# 6. Copy example state figures
# Output: All data/ and figures/ populated
```

## LaTeX Template

### Main Document (paper.tex)

```latex
\documentclass[twocolumn]{article}
\usepackage{graphicx}
\usepackage{amsmath}
\usepackage{booktabs}
\usepackage[margin=1in]{geometry}

\title{Algorithmic Redistricting: Applying Huntington-Hill Principles to District Boundary Design}
\author{Anonymous}
\date{January 2026}

\begin{document}
\maketitle

\begin{abstract}
Congressional redistricting in the United States faces persistent challenges...
\end{abstract}

\input{sections/01_introduction}
\input{sections/02_huntington_hill}
\input{sections/03_methodology}
\input{sections/04_results}
\input{sections/05_analysis}
\input{sections/06_discussion}

\bibliographystyle{plain}
\bibliography{references}

\end{document}
```

## Build Process

### build_paper.py

```python
# 1. Run generate_all_statistics.py
# 2. Verify all data/ and figures/ exist
# 3. Compile LaTeX: pdflatex paper.tex (twice for refs)
# 4. Verify paper.pdf generated
# 5. Report page count and file size
```

**Usage**:
```bash
# Generate all statistics and figures
python paper/analysis/generate_all_statistics.py

# Build PDF
python paper/build_paper.py

# View result
open paper/paper.pdf  # macOS
start paper/paper.pdf  # Windows
```

## Key Technical Details to Include

### METIS Invocation
```c
METIS_PartGraphRecursive(
    nvtxs,           // Number of vertices (tracts)
    ncon,            // Number of constraints (1: population)
    xadj, adjncy,    // Adjacency structure
    vwgt,            // Vertex weights (population)
    NULL,            // Vertex sizes (not used)
    NULL,            // Edge weights (not used)
    2,               // Number of parts (always 2 for bisection)
    tpwgts,          // Target partition weights
    ubvec,           // Imbalance tolerance (1.01 = 1%)
    options,         // METIS options
    &edgecut,        // Output: number of edges cut
    part             // Output: partition assignment
);
```

### Odd vs Even Division

**Even divisions** (8→2 groups of 4):
- Split into two equal groups
- Each group gets same number of districts
- Simple balanced bisection

**Odd divisions** (9→4+5):
- One group gets extra district
- Population targets adjusted: 4*ideal vs 5*ideal
- METIS balances population accordingly

### Recursive Bisection Example (8 districts)

```
Round 0: [8 districts]
         ↓
Round 1: [4 districts] | [4 districts]
         ↓                ↓
Round 2: [2|2]           [2|2]
         ↓                ↓
Round 3: [1|1|1|1]       [1|1|1|1]
```

## Timeline

1. **Analysis Scripts** (2-3 hours)
   - Write all 6 analysis scripts
   - Test on v1 2020 data
   - Generate all statistics and figures

2. **LaTeX Sections** (3-4 hours)
   - Write introduction and background
   - Describe methodology
   - Present results with figures/tables
   - Discussion and conclusions

3. **Assembly and Refinement** (1-2 hours)
   - Create build script
   - Compile and check page count
   - Adjust content to fit 8 pages
   - Final polish

**Total**: 6-9 hours for complete paper

## References to Include

1. Huntington-Hill method papers
2. Congressional apportionment history
3. Gerrymandering literature (academic)
4. METIS algorithm papers (Karypis & Kumar)
5. Graph partitioning for redistricting (Chen & Rodden, etc.)
6. Compactness measures (Polsby-Popper, Reock)
7. Supreme Court cases (Baker v. Carr, Shaw v. Reno)
