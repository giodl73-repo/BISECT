# PyO3 API Surface

The Python-facing API is intentionally minimal. Rust does computation; Python does I/O and orchestration.

## `redist_py.Graph`

```python
class Graph:
    @staticmethod
    def from_dict(data: dict) -> "Graph":
        """
        Build from a Python dict with keys:
          - 'adjacency': list of (u, v) int pairs
          - 'vertex_weights': 1D numpy array of int (population per tract)
          - 'n_vertices': int
        Raises ValueError if vertex_weights is 2D (multi-constraint not supported).
        """

    def n_vertices(self) -> int: ...
    def n_edges(self) -> int: ...
    def to_metis_file(self, path: str, edge_weights: dict[tuple, float] | None = None) -> None:
        """Write METIS .graph format file."""
```

## `redist_py.Partition`

```python
class Partition:
    @staticmethod
    def from_dict(assignments: dict[int, int]) -> "Partition":
        """tract_index -> district_id"""

    def to_dict(self) -> dict[int, int]: ...
    def population_balance(self, vertex_weights: np.ndarray, n_districts: int) -> float:
        """Return max absolute deviation from ideal as a fraction (0.005 = 0.5%)."""
    def assert_balanced(self, vertex_weights: np.ndarray, n_districts: int, tolerance: float = 0.005) -> None:
        """Raise ValueError if any district exceeds tolerance."""
```

## `redist_py.build_vra_edge_weights`

```python
def build_vra_edge_weights(
    graph: Graph,
    minority_fracs: np.ndarray,   # per-tract minority fraction
    threshold: float = 0.40,
) -> dict[tuple[int, int], float]:
    """
    Adaptive boost formula: max(3.0, 10.0 * (1.0 - 0.7 * f_minority))
    where f_minority = fraction of tracts with minority_frac >= threshold.
    Returns edge_weights dict for edges that get boosted (all others are 1.0).
    """
```

## `redist_py.build_adjacency_graph` (Phase 2)

```python
def build_adjacency_graph(
    geoid_list: list[str],
    geometries_wkb: list[bytes],   # WKB-encoded polygons
    populations: np.ndarray,
    min_boundary_length: float = 10.0,   # meters
) -> dict:
    """
    Returns dict compatible with Graph.from_dict() plus:
      - 'boundary_lengths': dict[(u,v) -> float] shared boundary in meters
      - 'geoids': list[str]
    """
```

## `redist_py.analyze_mm_districts` (Phase 4)

```python
def analyze_mm_districts(
    assignments: dict[int, int],
    minority_pops: np.ndarray,
    total_pops: np.ndarray,
    threshold: float = 0.50,
) -> dict:
    """
    Returns:
      - 'mm_count': int
      - 'mm_districts': list[int]
      - 'districts': list[dict] with keys: district_id, pct_minority, is_mm
    Written atomically with final_assignments — no vra_mode flag needed.
    """
```
