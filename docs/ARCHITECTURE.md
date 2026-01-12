# System Architecture

This document explains the design of the redistricting system, key algorithms, and how components interact.

## Table of Contents
1. [Visual Diagrams](#visual-diagrams)
2. [System Overview](#system-overview)
3. [Data Flow](#data-flow)
4. [Algorithm: Recursive Bisection](#algorithm-recursive-bisection)
5. [Component Architecture](#component-architecture)
6. [Key Design Decisions](#key-design-decisions)
7. [Scalability](#scalability)

---

## Visual Diagrams

The following diagrams provide visual representations of the system architecture, pipeline flow, and data transformations. All diagrams are in Mermaid format and are version-controlled in `docs/diagrams/`.

### System Overview

High-level architecture showing components from data sources to final outputs:

```mermaid
graph TB
    subgraph "Data Sources"
        A1[Census TIGER/Line<br/>Tract Shapefiles]
        A2[Census PL94-171<br/>Population Data]
        A3[MIT Election Lab<br/>Presidential Results]
        A4[Census DHC<br/>Demographics]
    end

    subgraph "Data Processing"
        B1[Download Scripts<br/>scripts/data/]
        B2[Process to Parquet<br/>Tract-Level Aggregates]
        B3[Build Adjacency Graphs<br/>NetworkX Graph]
    end

    subgraph "Core Algorithm"
        C1[METIS Library<br/>Graph Partitioning]
        C2[Recursive Bisection<br/>src/apportionment/partition/]
        C3[District Assignment<br/>Tract → District Mapping]
    end

    subgraph "Analysis & Metrics"
        D1[Compactness<br/>Polsby-Popper, Reock]
        D2[Political Analysis<br/>Partisan Lean]
        D3[Demographics<br/>Race, Gender, Diversity]
        D4[City Assignment<br/>Major Cities per District]
    end

    subgraph "Visualization"
        E1[District Maps<br/>Individual & State-Level]
        E2[Round Progression<br/>Bisection History]
        E3[National Maps<br/>All 435 Districts]
        E4[Analysis Maps<br/>Political, Demographic]
    end

    subgraph "Output"
        F1[Web Dashboard<br/>Interactive Explorer]
        F2[CSV Exports<br/>District Summaries]
        F3[PNG Maps<br/>Publication Quality]
    end

    A1 --> B1
    A2 --> B1
    A3 --> B1
    A4 --> B1

    B1 --> B2
    B2 --> B3

    B3 --> C2
    C1 --> C2
    C2 --> C3

    C3 --> D1
    C3 --> D2
    C3 --> D3
    C3 --> D4

    C3 --> E1
    C3 --> E2
    C3 --> E3
    D1 --> E4
    D2 --> E4
    D3 --> E4

    E1 --> F1
    E2 --> F1
    E3 --> F1
    E4 --> F1
    D1 --> F2
    D2 --> F2
    D3 --> F2
    E1 --> F3
    E2 --> F3
    E3 --> F3
    E4 --> F3

    style C2 fill:#90EE90
    style F1 fill:#87CEEB
```

### Pipeline Flow

Complete pipeline execution showing parallel/sequential modes and all phases:

```mermaid
graph TD
    START[run_complete_redistricting.py] --> PARALLEL{Parallel or<br/>Sequential?}

    PARALLEL -->|Parallel| P1[Process 4-8 States<br/>Simultaneously]
    PARALLEL -->|Sequential| S1[Process States<br/>One at a Time]

    P1 --> STATE[Per-State Processing]
    S1 --> STATE

    subgraph "State Processing (run_state_redistricting.py)"
        STATE --> ST1[Load Tract Data<br/>& Adjacency Graph]
        ST1 --> ST2[Recursive Bisection<br/>METIS Partitioning]
        ST2 --> ST3[Create District Summary<br/>Population, Compactness]
        ST3 --> ST4[Assign Cities to Districts]
        ST4 --> ST5[Generate Maps<br/>Districts, Rounds, Individual]
    end

    ST5 --> CHECK{All 50 States<br/>Complete?}
    CHECK -->|No| STATE
    CHECK -->|Yes| AGG[Aggregation Phase]

    subgraph "US Aggregation"
        AGG --> AG1[Create US District Summary<br/>All 435 Districts]
        AG1 --> AG2[Create US Rounds Hierarchy<br/>National Bisection Tree]
        AG2 --> AG3[Create US National Maps<br/>Base, With Cities]
        AG3 --> AG4[Create US Round Progression<br/>Rounds 1-6]
    end

    AG4 --> ANALYSIS[Analysis Phase]

    subgraph "Political Analysis"
        ANALYSIS --> PA1[Analyze Districts<br/>Partisan Lean per State]
        PA1 --> PA2[Visualize Partisan Lean<br/>Maps with D/R Counts]
        PA2 --> PA3[Create National Political Map<br/>All 435 Districts]
    end

    PA3 --> DEMO[Demographic Phase]

    subgraph "Demographic Analysis"
        DEMO --> DM1[Analyze Demographics<br/>Race, Gender per State]
        DM1 --> DM2[Visualize Demographics<br/>Diversity, Majority Maps]
        DM2 --> DM3[Create National Demographic Map<br/>All 435 Districts]
    end

    DM3 --> COMPACT[Compactness Phase]

    subgraph "Compactness Visualization"
        COMPACT --> CP1[Visualize Compactness<br/>Polsby-Popper, Reock Maps]
    end

    CP1 --> FINISH[Pipeline Complete<br/>Generate Dashboard]

    style STATE fill:#FFE4B5
    style AGG fill:#E0BBE4
    style ANALYSIS fill:#B4E7CE
    style DEMO fill:#FFDAB9
    style COMPACT fill:#B0E0E6
    style FINISH fill:#90EE90
```

**For additional diagrams** (Script Dependencies, Data Flow), see [`docs/diagrams/`](diagrams/).

---

## System Overview

### What This System Does

Creates congressional districts for all 50 US states using:
- **Input**: Census tract geometries + population data
- **Algorithm**: Recursive bisection with METIS graph partitioning
- **Output**: District assignments, maps, metrics, analysis

### Why This Approach?

**Alternative**: Manual redistricting by legislatures
- **Problem**: Gerrymandering, bias, lack of transparency
- **Our Solution**: Algorithmic, reproducible, geography-aware

**Alternative**: Other algorithms (simulated annealing, genetic algorithms)
- **Problem**: Slow, complex, unpredictable
- **Our Solution**: Fast, simple, deterministic (given same inputs)

---

## Data Flow

### High-Level Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│ 1. DATA ACQUISITION                                         │
│                                                             │
│  Census TIGER/Line → Download tracts & places (by state)   │
│  Census API       → Download demographics (by state)       │
│  Election Data    → Download election results (by tract)   │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. DATA PROCESSING                                          │
│                                                             │
│  Tracts        → Build adjacency graphs (NetworkX)         │
│  Demographics  → Process to tract-level parquet files      │
│  Elections     → Geocode to tracts, aggregate              │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. REDISTRICTING (Per State)                                │
│                                                             │
│  Input: Tracts + Adjacency Graph + Target # Districts      │
│  Algorithm: Recursive Bisection with METIS                 │
│  Output: tract_idx → district assignments                  │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. ANALYSIS & VISUALIZATION                                 │
│                                                             │
│  Political     → Partisan lean, competitive districts      │
│  Demographic   → Gender, race/ethnicity, diversity         │
│  Compactness   → Polsby-Popper, Reock metrics              │
│  Maps          → District maps, city labels, analysis maps │
└─────────────────────────────────────────────────────────────┘
```

### Detailed Data Flow

```
data/
├── raw/                          # Downloaded from Census
│   ├── {state}_tracts_2020.parquet    (Geometries + population)
│   ├── {state}_places_2020.parquet    (City points)
│   ├── elections/
│   │   └── {state}_precinct_2020.csv  (Precinct-level results)
│   └── demographics/
│       └── {state}_demographics_2020.csv (Tract-level demographics)
│
├── adjacency/                    # Generated from tracts
│   └── {state}_adjacency_2020.pkl     (NetworkX Graph)
│
└── processed/                    # Aggregated/processed
    ├── elections/
    │   └── 2020_president_tract.parquet    (All states, by tract)
    └── demographics/
        └── 2020_demographics_tract.parquet (All states, by tract)

                    ↓ PROCESSING ↓

outputs/us_2020_v1/
├── states/
│   └── california/              # Per-state results
│       ├── final_assignments.pkl         (tract_idx → district)
│       ├── district_summary.csv          (Metrics per district)
│       ├── district_cities.csv           (Major cities)
│       ├── rounds_hierarchy.csv          (Bisection tree)
│       ├── maps/
│       │   ├── california_52_districts.png
│       │   ├── districts/        (Individual district PNGs)
│       │   └── round_*.png       (Bisection visualization)
│       ├── political_analysis/
│       │   ├── district_political_2020.csv
│       │   └── maps/
│       └── demographic_analysis/
│           ├── district_demographics.csv
│           └── maps/
│               ├── gender_balance.png
│               ├── majority_race.png
│               └── diversity_index.png
│
├── us_rounds_hierarchy.csv      # National aggregate
├── us_district_summary.csv
└── us_national_map.png          # All 435 districts
```

---

## Algorithm: Recursive Bisection

### Core Concept

**Problem**: Partition N tracts into K districts with equal population.

**Solution**: Divide and conquer
1. Split all tracts into 2 groups (using METIS)
2. Recursively split each group until you have K districts

### Visual Walkthrough

The following diagram shows how recursive bisection works step-by-step, from geographic input through graph representation, the cut operation, and resulting partitions:

```mermaid
graph TB
    subgraph "Step 1: Geographic Input"
        A1["<b>State with Census Tracts</b><br/>────────────────────<br/>┌───┬───┬───┐<br/>│ A │ B │ C │ Pop: 120K<br/>├───┼───┼───┤<br/>│ D │ E │ F │ Pop: 130K<br/>├───┼───┼───┤<br/>│ G │ H │ I │ Pop: 125K<br/>└───┴───┴───┘<br/>Total: 375K → 2 Districts<br/>Target: ~187.5K per district"]
    end

    subgraph "Step 2: Graph Representation"
        B1["<b>Adjacency Graph</b><br/>────────────────────<br/>Vertices (Census Tracts):<br/>• A (pop: 40K)<br/>• B (pop: 45K)<br/>• C (pop: 35K)<br/>• D (pop: 42K)<br/>• E (pop: 48K)<br/>• F (pop: 40K)<br/>• G (pop: 38K)<br/>• H (pop: 47K)<br/>• I (pop: 40K)<br/><br/>Edges (Adjacencies):<br/>A↔B, B↔C, D↔E, E↔F<br/>G↔H, H↔I, A↔D, B↔E<br/>C↔F, D↔G, E↔H, F↔I"]
    end

    subgraph "Step 3: METIS Partitioning"
        C1["<b>Graph with Edge Weights</b><br/>────────────────────<br/>      A(40)───B(45)───C(35)<br/>       │       │       │<br/>      D(42)───E(48)───F(40)<br/>       │       │       │<br/>      G(38)───H(47)───I(40)<br/><br/>Find minimum edge cut that:<br/>✓ Balances population (~50%/50%)<br/>✓ Minimizes edges crossed<br/>✓ Keeps regions contiguous"]
    end

    subgraph "Step 4: The Cut"
        D1["<b>Partition Cut Found</b><br/>────────────────────<br/>      A(40)───B(45) ║ C(35)<br/>       │       │     ║  │<br/>      D(42)───E(48) ║ F(40)<br/>       │       │     ║  │<br/>      G(38)───H(47) ║ I(40)<br/>━━━━━━━━━━━━━━━━━━━<br/>Cut edges: B↔C, E↔F, H↔I<br/>Edge cut weight: 3"]
    end

    subgraph "Step 5: Two Balanced Partitions"
        E1["<b>Region 0 (Left)</b><br/>────────────────────<br/>┌───┬───┐<br/>│ A │ B │<br/>├───┼───┤<br/>│ D │ E │<br/>├───┼───┤<br/>│ G │ H │<br/>└───┴───┘<br/>Population: 187K<br/>Target: 1 district<br/>✓ Balanced!"]
        E2["<b>Region 1 (Right)</b><br/>────────────────────<br/>┌───┐<br/>│ C │<br/>├───┤<br/>│ F │<br/>├───┤<br/>│ I │<br/>└───┘<br/>Population: 188K<br/>Target: 1 district<br/>✓ Balanced!"]
    end

    subgraph "Step 6: Recurse if Needed"
        F1["<b>Check Each Region</b><br/>────────────────────<br/>Region 0: 1 district → DONE ✓<br/>Region 1: 1 district → DONE ✓<br/><br/>If target_districts > 1:<br/>  Split again recursively<br/><br/>Round 1: 1 → 2 regions<br/>Round 2: 2 → 4 regions<br/>Round 3: 4 → 8 regions<br/>Continue until each region<br/>has target_districts = 1"]
    end

    A1 ==> B1
    B1 ==> C1
    C1 ==> D1
    D1 ==> E1
    D1 ==> E2
    E1 ==> F1
    E2 ==> F1

    style A1 fill:#E8F5E9
    style B1 fill:#FFF3E0
    style C1 fill:#E3F2FD
    style D1 fill:#FFEBEE
    style E1 fill:#F3E5F5
    style E2 fill:#F3E5F5
    style F1 fill:#E0F2F1
```

**Key Steps Explained**:
1. **Geographic Input**: Start with census tracts and their populations
2. **Graph Representation**: Convert spatial data into a graph where vertices are tracts and edges connect adjacent tracts
3. **METIS Partitioning**: Use METIS to find the optimal cut that balances population while minimizing edge cuts
4. **The Cut**: Partition the graph by removing edges (shown with ║ symbols)
5. **Balanced Partitions**: Result is two contiguous regions with nearly equal population
6. **Recurse**: If a region needs more than 1 district, split it again recursively

### Why Recursive Bisection?

**Alternative**: Direct K-way partitioning
- **Problem**: METIS struggles with large K, slow, poor balance
- **Our Solution**: Binary splits are fast and well-balanced

**Advantages**:
- Hierarchical (creates natural nested structure)
- Fast (log₂K levels of recursion)
- Good population balance at each level
- Easy to visualize (binary tree)

### Pseudocode

```python
def recursive_bisection(tracts, adjacency_graph, target_districts):
    """
    Recursively partition tracts into districts.

    Args:
        tracts: List of tract data (with population)
        adjacency_graph: NetworkX graph of tract connectivity
        target_districts: Number of districts to create

    Returns:
        assignments: dict mapping tract_idx -> district_id
    """
    if target_districts == 1:
        # Base case: assign all tracts to district 1
        return {idx: 1 for idx in range(len(tracts))}

    # Split into two groups
    split_point = target_districts // 2

    # Use METIS to partition graph into 2 balanced parts
    partition = metis_bisect(adjacency_graph, tracts_population)

    # Separate into two subgraphs
    group_0_tracts = [i for i in range(len(tracts)) if partition[i] == 0]
    group_1_tracts = [i for i in range(len(tracts)) if partition[i] == 1]

    group_0_graph = adjacency_graph.subgraph(group_0_tracts)
    group_1_graph = adjacency_graph.subgraph(group_1_tracts)

    # Recursively partition each group
    group_0_assignments = recursive_bisection(
        group_0_tracts, group_0_graph, split_point
    )

    group_1_assignments = recursive_bisection(
        group_1_tracts, group_1_graph, target_districts - split_point
    )

    # Offset group_1 district IDs
    for tract_idx, district_id in group_1_assignments.items():
        group_1_assignments[tract_idx] = district_id + split_point

    # Combine assignments
    return {**group_0_assignments, **group_1_assignments}
```

### METIS Integration

**METIS** (Multi-constraint Graph Partitioning): Fast graph partitioning library
- Input: Weighted graph + target partition sizes
- Output: Partition assignment for each node
- Objective: Minimize edge cuts (keeps geographically compact)

**Our Usage**:
```python
import pymetis

# Convert NetworkX graph to METIS format
adjacency_list = [list(adj_graph.neighbors(i)) for i in nodes]

# Partition with population weights
parts = pymetis.part_graph(
    nparts=2,                    # Binary split
    adjacency=adjacency_list,
    vweights=population_weights, # Balance population
    niter=100                    # Quality (more iterations = better)
)
```

**Why niter=100?**: Trade-off between speed and quality
- niter=10: Fast, lower quality
- niter=100: Good balance (our default)
- niter=1000: Slow, diminishing returns

### Handling Odd Numbers of Districts

**Problem**: 52 districts → 26, 26 ✓
**Problem**: 53 districts → 26, 27? or 27, 26?

**Solution**: Split as evenly as possible
```python
split_point = target_districts // 2
group_0_size = split_point
group_1_size = target_districts - split_point

# Example: 53 districts
# split_point = 53 // 2 = 26
# group_0 = 26, group_1 = 27
```

### Bisection Tree Example

**California: 52 districts**

```
                    52 districts
                   /            \
              26                 26
             /  \               /  \
           13   13            13    13
          / \   / \          / \   / \
         7  6  7  6         7  6  7  6
        ...
```

- Level 0: 1 region (all tracts)
- Level 1: 2 regions
- Level 2: 4 regions
- Level 3: 8 regions
- Level 4: 16 regions
- Level 5: 32 regions
- Level 6: 52 districts ✓

Total rounds: ⌈log₂(52)⌉ = 6 levels

---

## Component Architecture

### Directory Structure

```
redistricting/
├── src/apportionment/          # Core library (algorithmic)
│   ├── data/                   # Data loading & validation
│   ├── redistricting/          # METIS wrapper, bisection algorithm
│   ├── compactness/            # Metric calculations
│   └── visualization/          # Map utilities
│
├── scripts/                    # Executable scripts (orchestration)
│   ├── pipeline/               # Main workflow orchestration
│   ├── political/              # Political & demographic analysis
│   ├── data/                   # Data acquisition
│   ├── validation/             # Testing & validation
│   └── debug/                  # Debugging utilities
│
├── data/                       # All data files
│   ├── raw/                    # Downloaded, unmodified
│   ├── processed/              # Cleaned, aggregated
│   └── adjacency/              # Generated graphs
│
└── outputs/                    # Results
    └── us_{year}_{version}/
        ├── states/             # Per-state results
        └── *.csv, *.png        # National aggregates
```

### Separation: Library vs Scripts

**src/apportionment/** (Library)
- Pure functions, no I/O side effects
- Reusable components
- Unit testable
- No progress bars, no print statements

**scripts/** (Executable)
- Orchestration, I/O, progress reporting
- CLI argument parsing
- Calls library functions
- Manages file paths, subprocess communication

### Example: Two-Layer Design

**Library (src/apportionment/redistricting/bisection.py)**:
```python
def redistribute_recursive(graph, populations, num_districts, niter=100):
    """
    Pure algorithm - no file I/O, no progress bars.

    Returns: assignments dictionary
    """
    # ... algorithm ...
    return assignments
```

**Script (scripts/pipeline/run_state_redistricting.py)**:
```python
def main():
    # Parse arguments
    args = parser.parse_args()

    # Load data
    tracts = gpd.read_parquet(f'data/raw/{state}_tracts_2020.parquet')
    graph = load_adjacency_graph(state)

    # Progress reporting
    with tqdm(total=num_districts) as pbar:
        # Call library function
        assignments = redistribute_recursive(
            graph, populations, num_districts
        )
        pbar.update(num_districts)

    # Save results
    save_assignments(assignments, output_dir)
```

---

## Key Design Decisions

### 1. Census Tracts as Base Unit

**Decision**: Use census tracts (not blocks, not precincts)

**Why tracts?**
- ✅ Stable boundaries across decades
- ✅ ~4,000 people each (good granularity)
- ✅ Available nationwide with consistent data
- ✅ TIGER/Line provides geometries + population
- ✅ Smaller than counties, larger than blocks

**Alternatives considered**:
- Census blocks: Too small (~100 people), too many (7M+ nationwide)
- Counties: Too large, inflexible
- Precincts: Vary by state, change frequently, no standard geometries

### 2. METIS for Graph Partitioning

**Decision**: Use METIS gpmetis algorithm

**Why METIS?**
- ✅ Fast: Handles 10,000+ node graphs in seconds
- ✅ Quality: Produces compact, balanced partitions
- ✅ Proven: Industry standard (used in HPC, circuit design)
- ✅ Available: Open source, Python bindings (pymetis)

**Alternatives considered**:
- Simulated annealing: Too slow, unpredictable
- Genetic algorithms: Complex, hard to tune
- Greedy algorithms: Poor quality, local optima

### 3. Recursive Bisection over K-way

**Decision**: Binary splits, recursive

**Why binary?**
- ✅ Fast: log₂K rounds instead of direct K-way
- ✅ Balanced: Easy to enforce population balance
- ✅ Hierarchical: Creates natural district groupings
- ✅ Visualizable: Binary tree structure

**Trade-off**:
- ❌ May create slightly less compact districts than optimal K-way
- ✅ But much faster and more stable

### 4. Parquet over CSV

**Decision**: Use Parquet for tabular data

**Why Parquet?**
- ✅ Fast: Columnar storage, efficient reads
- ✅ Compressed: 5-10x smaller than CSV
- ✅ Typed: Preserves dtypes (no string/int confusion)
- ✅ Pandas-native: `df.to_parquet()` / `pd.read_parquet()`

**Use CSV only for**:
- Human-readable outputs (summaries, reports)
- Excel compatibility

### 5. Progress Bar Protocol

**Decision**: Parent-child STATUS message protocol

**Why this design?**
- ✅ Centralized: Parent manages all progress bars
- ✅ Clean: Children emit simple text messages
- ✅ Flexible: Works with any subprocess depth
- ✅ Non-intrusive: Children detect via environment variable

**Alternative**: Each script manages own progress bar
- ❌ Cluttered output
- ❌ Overlapping progress bars
- ❌ No coordination

### 6. Skip Logic Everywhere

**Decision**: All scripts check for existing outputs

**Why?**
- ✅ Resumable: Can restart after failures
- ✅ Efficient: Skip expensive re-computation
- ✅ Debugging: Can re-run single failing step

**Pattern**:
```python
if not force and output_file.exists():
    return 0  # Skip
```

---

## Scalability

### Current Scale

- **States**: 50 (all US states)
- **Tracts**: ~84,000 nationwide (~100 to 9,000 per state)
- **Districts**: 435 total
- **Processing time**:
  - Sequential: ~8-12 hours (all 50 states)
  - Parallel (4 workers): ~2-3 hours

### Bottlenecks

1. **METIS graph partitioning**: O(N log N) per bisection
   - Not a bottleneck (each state processes in 5-30 minutes)

2. **Map rendering**: O(N²) for complex geometries
   - Bottleneck for large states (CA, TX)
   - Solution: Configurable DPI, skip logic

3. **I/O**: Reading/writing large parquet files
   - Minor bottleneck
   - Solution: Parquet compression

### Parallelization Strategy

**State-level parallelism**: Process multiple states simultaneously
- ✅ Natural unit: Each state is independent
- ✅ Load balancing: Larger states take longer, but run in parallel
- ✅ Implementation: `multiprocessing.Pool` with 4-8 workers

**Why not tract-level parallelism?**
- Recursive bisection is inherently serial (must split, then recurse)
- Graph partitioning (METIS) already parallelizes internally

### Future Scalability

**If processing 100 states**:
- Current architecture handles this fine
- Just add more parallel workers

**If tracts increase 10x** (e.g., using census blocks):
- METIS would still handle it (tested to 100K+ nodes)
- Map rendering would be slower → need lower DPI or simplified geometries

**If adding more analysis types**:
- Current pipeline easily extended
- Add new scripts in `scripts/political/` or new directory
- Integrate into `run_complete_redistricting.py`

---

## System Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                      REDISTRICTING SYSTEM                        │
└──────────────────────────────────────────────────────────────────┘

┌─────────────────────┐
│  EXTERNAL DATA      │
│  - Census API       │──┐
│  - TIGER/Line       │  │
│  - Election Data    │  │
└─────────────────────┘  │
                         │
        ┌────────────────▼───────────────────┐
        │  DATA ACQUISITION (scripts/data/)  │
        │  - Download tracts, places         │
        │  - Download demographics           │
        │  - Download election results       │
        └────────────────┬───────────────────┘
                         │
        ┌────────────────▼───────────────────┐
        │  PREPROCESSING                     │
        │  - Build adjacency graphs          │
        │  - Process demographics            │
        │  - Geocode election data           │
        └────────────────┬───────────────────┘
                         │
        ┌────────────────▼───────────────────┐
        │  CORE ALGORITHM (src/redistricting)│
        │  - Recursive bisection             │
        │  - METIS graph partitioning        │
        │  - Population balancing            │
        └────────────────┬───────────────────┘
                         │
        ┌────────────────▼───────────────────┐
        │  ANALYSIS (scripts/political/)     │
        │  - Compactness metrics             │
        │  - Political analysis              │
        │  - Demographic analysis            │
        └────────────────┬───────────────────┘
                         │
        ┌────────────────▼───────────────────┐
        │  VISUALIZATION                     │
        │  - District maps                   │
        │  - Bisection round maps            │
        │  - Analysis maps (lean, demo)      │
        │  - National aggregate map          │
        └────────────────┬───────────────────┘
                         │
        ┌────────────────▼───────────────────┐
        │  OUTPUTS                           │
        │  - District assignments            │
        │  - Metrics & statistics            │
        │  - Maps (150+ per run)             │
        │  - CSV reports                     │
        └────────────────────────────────────┘
```

---

## Web Dashboard

### Architecture

The dashboard provides an interactive interface to explore all redistricting outputs:

```
┌─────────────────────────────────────────────────────┐
│ Dashboard (Single-Page Application)                │
│                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
│  │   Sidebar    │  │   Top Nav    │  │ Content  │ │
│  │              │  │              │  │          │ │
│  │ 50 States    │  │ 7 Dimensions │  │  Maps    │ │
│  │ + Districts  │  │              │  │  Tables  │ │
│  │              │  │ Year/Version │  │  Links   │ │
│  └──────────────┘  └──────────────┘  └──────────┘ │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
             Dynamic Path Resolution
          (us_2020_v1, us_2030_v1, etc.)
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│ Output Structure                                    │
│                                                     │
│  outputs/                                           │
│    ├── index.html (deployed dashboard)             │
│    ├── us_2020_v1/                                 │
│    │   └── states/                                 │
│    │       ├── alabama/                            │
│    │       │   ├── maps/                           │
│    │       │   │   ├── districts/                  │
│    │       │   │   └── rounds/                     │
│    │       │   ├── political_analysis/             │
│    │       │   └── demographic_analysis/           │
│    │       └── ... (49 more states)                │
│    └── us_2030_v1/ (future)                        │
└─────────────────────────────────────────────────────┘
```

### Design Decisions

**Single HTML File**:
- **Why**: Zero dependencies, works offline, easy to deploy
- **Trade-off**: No server-side processing, all logic in JavaScript
- **Benefit**: Can be hosted anywhere (GitHub Pages, S3, local file)

**Hash-Based Navigation**:
- **Why**: Preserve state without page reloads
- **Implementation**: URL hash like `#us_2020_v1` switches output directory
- **Benefit**: Bookmarkable, shareable, browser history works

**Source vs Deployed**:
- **Source**: `web/dashboard.html` - Master template, version controlled
- **Deployed**: `outputs/index.html` - Copied by `web/deploy_dashboard.py`
- **Benefit**: Outputs dir can be gitignored, source is tracked

**Grid Layout with CSS Grid**:
- **Why**: Responsive, clean, modern
- **Rounds**: Max 2 per row (better for vertical comparison)
- **Districts**: Auto-fit grid (adapts to screen size)
- **Benefit**: Works on desktop and mobile

### Navigation Structure

**Three-Level Hierarchy**:
1. **Year/Version**: Top-level selector (us_2020_v1, us_2030_v1)
2. **State**: Sidebar (Alabama → Wyoming)
3. **Dimension**: Top tabs (Overview, Districts, Rounds, Political, Demographics, Compactness, Urban)

**Dynamic Content Generation**:
- JavaScript generates HTML from templates
- Image paths computed from `getBasePath()`
- Graceful degradation with fallback SVG placeholders

### Integration with Pipeline

**Deployment Workflow**:
```bash
# 1. Generate outputs
python scripts/pipeline/run_complete_redistricting.py --workers 4

# 2. Deploy dashboard
python web/deploy_dashboard.py

# 3. Open in browser
open outputs/index.html
```

**Automatic Path Resolution**:
- Dashboard reads year/version from URL hash
- Constructs paths: `{year}_{version}/states/{state}/...`
- No configuration needed, works for any output structure

---

## Summary

**Key Architectural Principles**:

1. **Separation of Concerns**: Library (algorithm) vs Scripts (orchestration)
2. **Data Flow**: Raw → Processed → Analyzed → Visualized
3. **Scalability**: State-level parallelism, efficient algorithms
4. **Reproducibility**: Deterministic algorithm, version-controlled
5. **Extensibility**: Easy to add new analysis types, visualization styles
6. **Maintainability**: Clear patterns, documented conventions

**Core Algorithm**: Recursive bisection with METIS
- Fast: O(N log K) time complexity
- Quality: Produces compact, balanced districts
- Scalable: Handles all 50 states efficiently

**Design Philosophy**:
- Simple over complex
- Fast over perfect
- Reproducible over novel
- Practical over academic
