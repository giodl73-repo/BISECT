# Apportionment Library

Core Python library for congressional redistricting using graph-based algorithms.

## Overview

This library provides reusable components for:
1. **Data Loading**: Census tract geometries, adjacency graphs, demographic data
2. **Graph Partitioning**: Recursive bisection with METIS integration
3. **Visualization**: Map generation for districts and analysis

## Module Structure

```
src/apportionment/
├── __init__.py          # Package initialization
├── data/                # Data loading and graph construction
│   ├── __init__.py
│   ├── adjacency.py     # Build adjacency graphs from geometries
│   ├── census.py        # Load census tract data
│   └── io.py            # File I/O utilities (parquet, pickle)
├── partition/           # Graph partitioning algorithms
│   ├── __init__.py
│   ├── metis_wrapper.py     # Python wrapper for METIS library
│   ├── metis_executable.py  # Fallback: call METIS as subprocess
│   └── recursive_bisection.py  # Main redistricting algorithm
└── visualization/       # Map creation
    ├── __init__.py
    └── maps.py          # District map generation
```

## data/ Module

### adjacency.py

**Purpose**: Build adjacency graphs from tract geometries

**Key Function**:
```python
def build_adjacency_graph(tracts_gdf):
    """
    Create NetworkX graph with queen contiguity (shared edge or corner).

    Args:
        tracts_gdf: GeoDataFrame with tract geometries

    Returns:
        NetworkX Graph with nodes for each tract and edges for adjacencies
    """
```

**Algorithm**: Uses geopandas spatial join to find touching tracts

**Special Handling**:
- Water-based adjacency for coastal areas
- County-aware for cross-county connections

**Used By**: `scripts/data/geography/build_adjacency_graphs.py`

### census.py

**Purpose**: Load and process census tract data

**Key Functions**:
```python
def load_tracts(state_code, year='2020'):
    """Load tract geometries and population."""

def load_places(state_code, year='2020'):
    """Load city points for labeling."""

def standardize_geoid(df):
    """Ensure GEOID is 11-digit zero-padded string."""
```

**CRITICAL Pattern**:
```python
# Always ensure GEOIDs are properly formatted
df['GEOID'] = df['GEOID'].astype(str).str.zfill(11)
```

**Why**: Census tract GEOIDs have leading zeros (e.g., California starts with 06) that get lost when stored as integers.

### io.py

**Purpose**: File I/O utilities for various formats

**Key Functions**:
```python
def load_parquet(file_path):
    """Load parquet file with error handling."""

def save_parquet(df, file_path):
    """Save DataFrame to parquet with compression."""

def load_pickle(file_path):
    """Load pickle file (graphs, assignments)."""

def save_pickle(obj, file_path):
    """Save Python object to pickle."""
```

**Format Guidelines**:
- **Parquet**: Large tabular data (tracts, demographics, elections)
  - Fast columnar storage
  - Preserves dtypes (no GEOID confusion)
  - 5-10x smaller than CSV
- **Pickle**: Python objects (NetworkX graphs, dict assignments)
  - Not portable to other languages
  - Fast serialization/deserialization
- **CSV**: Human inspection, Excel compatibility, small files

## partition/ Module

### recursive_bisection.py

**Purpose**: Main redistricting algorithm using recursive binary splits

**Key Function**:
```python
def recursive_bisection(
    graph: nx.Graph,
    num_districts: int,
    ideal_pop: int,
    tolerance: float = 0.01,
    compactness_weight: float = 0.0
) -> dict:
    """
    Partition graph into districts using recursive bisection.

    Algorithm:
    1. Start with entire state as single partition
    2. Recursively split into 2 parts using METIS
    3. Continue until reaching target number of districts

    Args:
        graph: NetworkX graph with 'population' node attribute
        num_districts: Target number of districts (2, 3, 4, ..., 435)
        ideal_pop: Target population per district
        tolerance: Population deviation allowed (default 1%)
        compactness_weight: Weight for compactness vs population balance

    Returns:
        dict: {tract_geoid: district_number}
    """
```

**Algorithm Details**:
```
1. Start: All tracts in partition 0
2. Round 1: Split partition 0 → [partition 0, partition 1]
3. Round 2: Split partition 0 → [partition 0, partition 2]
            Split partition 1 → [partition 1, partition 3]
4. Round N: Continue until N partitions = num_districts
5. Renumber: Map partitions to district numbers 1..N
```

**Constraints**:
- Population deviation: ±1% of ideal (configurable)
- Contiguity: All districts must be connected (enforced by graph)
- Compactness: Minimized by METIS (via edge cuts)

**Output Format**:
```python
{
    '06001400100': 1,  # Tract in district 1
    '06001400200': 1,  # Tract in district 1
    '06001400300': 2,  # Tract in district 2
    ...
}
```

### metis_wrapper.py

**Purpose**: Python interface to METIS library via ctypes

**Key Function**:
```python
def partition_graph(graph, num_parts=2, **options):
    """
    Partition graph into k parts using METIS.

    Uses METIS_PartGraphKway for k-way partitioning.

    Args:
        graph: NetworkX graph
        num_parts: Number of partitions (2 for bisection)
        options: METIS options (ubvec, tpwgts, etc.)

    Returns:
        list: Partition assignment for each node [0, 1, 0, 1, ...]
    """
```

**METIS Integration**:
- **Primary**: Uses METIS shared library via ctypes (fast)
- **Fallback**: Calls METIS executable if library not found
- **Platform**: Works on Windows, Linux, macOS

**Graph Format Conversion**:
```python
# NetworkX → METIS CSR format
xadj = [0, 2, 5, 8, ...]  # Adjacency index
adjncy = [1, 3, 0, 2, 4, ...]  # Neighbor list
vwgt = [1000, 1500, 2000, ...]  # Node weights (population)
```

**Why METIS?**
- State-of-the-art graph partitioning
- Fast: O(|E| log k) for k-way partitioning
- Minimizes edge cuts (improves compactness)
- Handles weighted graphs (population constraints)

### metis_executable.py

**Purpose**: Fallback METIS interface via subprocess

**When Used**: If METIS shared library not available

**Approach**:
1. Write graph to METIS format file
2. Call `gpmetis` executable
3. Read partition assignments from output file
4. Clean up temporary files

**Trade-offs**:
- Slower (file I/O overhead)
- More portable (only requires executable in PATH)
- Easier debugging (can inspect intermediate files)

## visualization/ Module

### maps.py

**Purpose**: Generate district maps with proper styling

**Key Functions**:
```python
def create_district_map(
    tracts_gdf: gpd.GeoDataFrame,
    assignments: dict,
    output_file: str,
    title: str = None,
    dpi: int = 150,
    show_cities: bool = False
):
    """
    Create district map with standard styling.

    Styling:
    - Thin white tract boundaries (0.1pt)
    - Thick black district boundaries (1.5pt)
    - Rainbow colormap for districts
    - Optional city labels
    """
```

**Standard Map Pattern** (used everywhere):
```python
import matplotlib.pyplot as plt
import geopandas as gpd

# 1. Plot tracts with thin white boundaries
tracts_gdf.plot(
    ax=ax,
    column='district',
    edgecolor='white',
    linewidth=0.1,
    alpha=0.8,
    cmap='tab20'
)

# 2. Add thick black district boundaries as overlay
districts_dissolved = tracts_gdf.dissolve(by='district', as_index=False)
districts_dissolved.boundary.plot(
    ax=ax,
    edgecolor='black',
    linewidth=1.5,
    zorder=10  # Ensure on top
)

# 3. Save with configured DPI
plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
```

**Why This Pattern?**
- Thin white boundaries: Show tract-level detail without cluttering
- Thick black boundaries: Make district outlines prominent
- `dissolve()`: Merges adjacent tracts into district polygons
- `boundary.plot()`: Draws only the outline (not filled)
- `zorder=10`: Ensures boundaries render on top of tracts

**DPI Guidelines**:
- 72: Low quality (testing only)
- 100: Development
- **150**: Production default (good balance)
- 200: High quality
- 300: Print quality (very slow!)

## Usage Examples

### Load Data and Build Graph

```python
from apportionment.data import census, adjacency, io

# Load tract data
tracts_gdf = census.load_tracts('CA', year='2020')

# Ensure GEOID is properly formatted
tracts_gdf = census.standardize_geoid(tracts_gdf)

# Build adjacency graph
graph = adjacency.build_adjacency_graph(tracts_gdf)

# Save for later use
io.save_pickle(graph, 'ca_adjacency_2020.pkl')
```

### Run Redistricting

```python
from apportionment.partition import recursive_bisection
from apportionment.data import io

# Load adjacency graph
graph = io.load_pickle('ca_adjacency_2020.pkl')

# Calculate ideal population
total_pop = sum(nx.get_node_attributes(graph, 'population').values())
num_districts = 52  # California has 52 districts
ideal_pop = total_pop / num_districts

# Run recursive bisection
assignments = recursive_bisection(
    graph=graph,
    num_districts=num_districts,
    ideal_pop=ideal_pop,
    tolerance=0.01,  # ±1% population deviation
    compactness_weight=0.0
)

# Save assignments
io.save_pickle(assignments, 'ca_assignments.pkl')
```

### Create District Map

```python
from apportionment.visualization import maps
from apportionment.data import io, census

# Load data
tracts_gdf = census.load_tracts('CA', year='2020')
assignments = io.load_pickle('ca_assignments.pkl')

# Add district assignments to GeoDataFrame
tracts_gdf['district'] = tracts_gdf['GEOID'].map(assignments)

# Create map
maps.create_district_map(
    tracts_gdf=tracts_gdf,
    assignments=assignments,
    output_file='ca_districts.png',
    title='California - 52 Congressional Districts',
    dpi=150,
    show_cities=True
)
```

## Key Design Decisions

### Why Census Tracts?

**Choice**: Use census tracts as atomic redistricting units

**Alternatives Considered**:
- Blocks: Too granular (7M+ nationwide), computationally expensive
- Counties: Too coarse-grained, can't achieve population balance
- Precincts: Not standardized, boundaries change frequently

**Trade-offs**:
- ✅ ~84K tracts nationwide (manageable)
- ✅ Stable boundaries (decade-to-decade)
- ✅ Rich demographic data available
- ✅ Align with other Census products
- ❌ Still quite granular (slow for large states)
- ❌ Some tracts cross county lines

### Why NetworkX Graphs?

**Choice**: Represent tract adjacency as NetworkX graphs

**Alternatives Considered**:
- Adjacency matrix: Dense, memory-intensive
- Adjacency list: Lower-level, more manual
- Spatial index: Fast queries but no graph algorithms

**Trade-offs**:
- ✅ Rich graph algorithm library (connected components, etc.)
- ✅ Easy integration with METIS
- ✅ Intuitive API
- ✅ Serializable (pickle)
- ❌ Not as fast as pure C++ implementations
- ❌ Memory overhead for large graphs

### Why Recursive Bisection?

**Choice**: Binary splits instead of direct k-way partitioning

**Alternatives Considered**:
- Direct k-way: METIS can partition into k districts at once
- Hierarchical: Build district hierarchy (not just binary)
- Iterative refinement: Start with seed districts, refine boundaries

**Trade-offs**:
- ✅ Simple, interpretable algorithm
- ✅ Creates hierarchical structure (useful for analysis)
- ✅ Each split independent (easier to debug)
- ✅ Better population balance (focus on one split at a time)
- ❌ May not be globally optimal
- ❌ More splits required (log₂ k instead of 1)

### Why Parquet?

**Choice**: Use Parquet for large tabular data

**Alternatives Considered**:
- CSV: Human-readable, universal
- HDF5: Scientific computing standard
- Feather: Arrow format, very fast
- SQLite: Relational database

**Trade-offs**:
- ✅ Columnar: Fast filtering, aggregation
- ✅ Compressed: 5-10x smaller than CSV
- ✅ Typed: Preserves dtypes (no GEOID confusion)
- ✅ Pandas-native: Direct read/write
- ✅ Widely supported: Spark, Dask, Arrow
- ❌ Not human-readable
- ❌ Requires library (pyarrow or fastparquet)

## Performance Characteristics

### Recursive Bisection

**Time Complexity**: O(N log k)
- N = number of tracts
- k = number of districts
- Each METIS bisection: O(N) for 2-way partition
- Number of bisections: O(k) for k districts

**Space Complexity**: O(N + E)
- N = number of nodes (tracts)
- E = number of edges (adjacencies)
- Graph storage: ~200 MB for California (73K tracts)

**Typical Runtimes** (single state):
- Small states (<20 districts): 1-5 minutes
- Medium states (20-40 districts): 5-15 minutes
- Large states (>40 districts): 15-60 minutes

**Bottlenecks**:
1. METIS partitioning (~70% of time)
2. Graph construction from geometries (~20%)
3. File I/O (~10%)

### Adjacency Graph Construction

**Time Complexity**: O(N²) worst case, O(N log N) typical
- Uses geopandas spatial index (R-tree)
- Each tract only checks nearby tracts

**Typical Runtimes**:
- Small states: 10-30 seconds
- Large states: 60-180 seconds

**Memory**: ~2-5x tract GeoDataFrame size during construction

## Integration with Scripts

This library is used by scripts in `scripts/`:

**Pipeline Scripts** (`scripts/pipeline/`):
- `process_single_state.py`: Calls `recursive_bisection()`
- `visualize_districts.py`: Calls `maps.create_district_map()`

**Data Scripts** (`scripts/data/`):
- `build_adjacency_graphs.py`: Calls `adjacency.build_adjacency_graph()`
- All download scripts: Use `io.save_parquet()`

**Analysis Scripts** (`scripts/political/`):
- All analysis scripts: Use `io.load_parquet()` for data
- Visualization scripts: Use `maps.create_district_map()` pattern

## Testing

**Unit Tests**: (TODO - not yet implemented)
```python
# tests/test_adjacency.py
def test_build_adjacency_graph():
    # Test that graph is connected
    # Test that adjacencies are symmetric
    # Test queen contiguity

# tests/test_bisection.py
def test_recursive_bisection():
    # Test population balance
    # Test contiguity
    # Test number of districts
```

**Integration Tests**: Run full pipeline on small test case
```bash
# Process Vermont (1 district, simple)
python scripts/pipeline/run_state_redistricting.py --state VT --year 2020
```

## Dependencies

**Core**:
- `networkx`: Graph data structure and algorithms
- `geopandas`: Geospatial data processing
- `pandas`: Tabular data manipulation
- `pyarrow` or `fastparquet`: Parquet file I/O

**Visualization**:
- `matplotlib`: Static map generation
- `contextily`: Basemap tiles (optional)

**METIS**:
- `metis` Python package (optional, for ctypes wrapper)
- `gpmetis` executable (required, for fallback)

See `docs/DEPENDENCIES.md` for installation instructions.

## Common Patterns

### GEOID Handling (CRITICAL)

**Always use this pattern before merging DataFrames**:
```python
# Convert both dataframes to string with zero-padding
df1['GEOID'] = df1['GEOID'].astype(str).str.zfill(11)
df2['GEOID'] = df2['GEOID'].astype(str).str.zfill(11)

# Now safe to merge
result = df1.merge(df2, on='GEOID')
```

**Why**: GEOIDs are 11-digit identifiers with leading zeros. States like California (06...), Alabama (01...), etc. lose their leading zeros when stored as integers.

### Graph Node Attributes

**Standard pattern for tract graphs**:
```python
# Add node attributes
for node in graph.nodes():
    graph.nodes[node]['GEOID'] = tract_geoid
    graph.nodes[node]['population'] = tract_pop
    graph.nodes[node]['geometry'] = tract_geom  # Optional

# Retrieve node attributes
populations = nx.get_node_attributes(graph, 'population')
total_pop = sum(populations.values())
```

### Error Handling

**File I/O**:
```python
from pathlib import Path

def load_data(file_path):
    """Load data with robust error handling."""
    file_path = Path(file_path)

    if not file_path.exists():
        raise FileNotFoundError(f"File not found: {file_path}")

    try:
        return pd.read_parquet(file_path)
    except Exception as e:
        raise IOError(f"Failed to read {file_path}: {e}")
```

## See Also

- `../../context/CODING_PATTERNS.md` - Detailed coding conventions
- `../../context/ARCHITECTURE.md` - System design and algorithm details
- `scripts/pipeline/README.md` - Pipeline orchestration
- `scripts/data/README.md` - Data acquisition
- `scripts/political/README.md` - Political/demographic analysis
