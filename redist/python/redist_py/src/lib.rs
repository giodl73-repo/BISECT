use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use numpy::PyReadonlyArray1;
use std::collections::HashMap;
use redist_core::{
    Graph as CoreGraph, Partition as CorePartition, build_vra_edge_weights as core_vra,
    build_partisan_weights as core_partisan_weights,
    BisectionTree as CoreBisectionTree, max_depth_for_k, ufactor_for_depth,
    metis_format::{write_metis_graph as core_write_metis, parse_metis_partition as core_parse_metis},
};
use redist_data::{
    read_tiger_tracts, build_adjacency_graph, connect_island_components,
    serialize_adjacency, deserialize_adjacency, AdjacencyGraph,
};
use redist_analysis::{
    polsby_popper as rust_pp, reock as rust_reock, convex_hull_ratio as rust_chr,
    analyze_mm_districts as rust_vra_analysis,
};
use redist_analysis::compactness::all_metrics as rust_all_metrics;

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

#[pyclass]
struct Graph {
    inner: CoreGraph,
}

#[pymethods]
impl Graph {
    /// Build from a Python dict with keys:
    ///   'adjacency': list[list[int]]  (CSR format)
    ///   'vertex_weights': list[int] or 1D numpy int64 array
    ///   'n_vertices': int
    /// Raises ValueError if vertex_weights is 2D.
    #[staticmethod]
    fn from_csr(data: &Bound<'_, PyAny>) -> PyResult<Self> {
        // Extract adjacency (CSR: list of neighbor lists)
        let adj_obj = data.get_item("adjacency")?;
        let adjacency: Vec<Vec<usize>> = adj_obj.extract()?;

        // Extract vertex_weights — must be 1D
        let wt_obj = data.get_item("vertex_weights")?;

        // Try 1D numpy array first
        if let Ok(arr) = wt_obj.extract::<PyReadonlyArray1<i64>>() {
            let weights = arr.as_slice()
                .map_err(|e| PyValueError::new_err(e.to_string()))?
                .to_vec();
            let inner = CoreGraph::new(adjacency, weights)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            return Ok(Graph { inner });
        }

        // Try plain Python list/sequence of ints
        if let Ok(weights) = wt_obj.extract::<Vec<i64>>() {
            let inner = CoreGraph::new(adjacency, weights)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            return Ok(Graph { inner });
        }

        // Anything else (2D array, wrong type) — reject with clear message
        Err(PyValueError::new_err(
            "vertex_weights must be 1D (got 2D — multi-constraint mode not supported)"
        ))
    }

    fn n_vertices(&self) -> usize {
        self.inner.n_vertices()
    }

    fn n_edges(&self) -> usize {
        self.inner.n_edges()
    }
}

// ---------------------------------------------------------------------------
// Partition
// ---------------------------------------------------------------------------

#[pyclass]
struct Partition {
    inner: CorePartition,
}

#[pymethods]
impl Partition {
    #[staticmethod]
    fn from_dict(assignments: HashMap<usize, usize>) -> Self {
        Partition { inner: CorePartition::from_assignments(assignments) }
    }

    fn to_dict(&self) -> HashMap<usize, usize> {
        self.inner.to_assignments().clone()
    }

    fn population_balance(
        &self,
        vertex_weights: PyReadonlyArray1<i64>,
        n_districts: usize,
    ) -> PyResult<f64> {
        let weights = vertex_weights.as_slice()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(self.inner.population_balance(weights, n_districts))
    }

    #[pyo3(signature = (vertex_weights, n_districts, tolerance=0.005))]
    fn assert_balanced(
        &self,
        vertex_weights: PyReadonlyArray1<i64>,
        n_districts: usize,
        tolerance: f64,
    ) -> PyResult<()> {
        let weights = vertex_weights.as_slice()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        self.inner.assert_balanced(weights, n_districts, tolerance)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// VRA edge weights
// ---------------------------------------------------------------------------

/// Build VRA edge weights using the adaptive boost formula.
/// α = max(3.0, 10.0 × (1.0 − 0.7 × f_minority))
/// Returns dict mapping (u, v) → weight for boosted edges only.
#[pyfunction]
#[pyo3(signature = (edges, minority_fracs, threshold=0.40))]
fn build_vra_edge_weights(
    edges: Vec<(usize, usize)>,
    minority_fracs: PyReadonlyArray1<f64>,
    threshold: f64,
) -> PyResult<HashMap<(usize, usize), f64>> {
    let fracs = minority_fracs.as_slice()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(core_vra(&edges, fracs, threshold))
}

/// Build partisan edge weights using the adaptive boost formula.
/// α = max(3.0, 10.0 × (1.0 − 0.7 × f_partisan))
/// Symmetric: boosts D-D and R-R edges, never mixed or swing-involving edges.
/// Returns dict mapping (u, v) → weight for boosted edges only.
#[pyfunction]
#[pyo3(signature = (edges, dem_shares, dem_threshold=0.55, rep_threshold=0.45))]
fn build_partisan_weights(
    edges: Vec<(usize, usize)>,
    dem_shares: PyReadonlyArray1<f64>,
    dem_threshold: f64,
    rep_threshold: f64,
) -> PyResult<HashMap<(usize, usize), f64>> {
    let shares = dem_shares.as_slice()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(core_partisan_weights(&edges, shares, dem_threshold, rep_threshold))
}

// ---------------------------------------------------------------------------
// Compactness metrics
// ---------------------------------------------------------------------------

/// Compute Polsby-Popper score from WKB polygon (projected CRS required).
/// Returns (polsby_popper_score, perimeter_metres).
#[pyfunction]
fn compute_polsby_popper(
    py: Python<'_>,
    wkb: Vec<u8>,
) -> PyResult<(f64, f64)> {
    use redist_data::adjacency::parse_wkb_polygon_pub;
    let poly = parse_wkb_polygon_pub(&wkb, 0)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    rust_pp(&poly).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Compute Reock score from WKB polygon (projected CRS required).
#[pyfunction]
fn compute_reock(wkb: Vec<u8>) -> PyResult<f64> {
    use redist_data::adjacency::parse_wkb_polygon_pub;
    let poly = parse_wkb_polygon_pub(&wkb, 0)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    rust_reock(&poly).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Compute all three compactness metrics (PP, Reock, CHR) from WKB polygon.
/// Returns dict: {district, polsby_popper, reock, convex_hull_ratio, perimeter_m, area_m2}
#[pyfunction]
fn compute_all_compactness(
    py: Python<'_>,
    district: usize,
    wkb: Vec<u8>,
) -> PyResult<PyObject> {
    use redist_data::adjacency::parse_wkb_polygon_pub;
    let poly = parse_wkb_polygon_pub(&wkb, 0)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let m = rust_all_metrics(district, &poly)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let d = pyo3::types::PyDict::new_bound(py);
    d.set_item("district", m.district)?;
    d.set_item("polsby_popper", m.polsby_popper)?;
    d.set_item("reock", m.reock)?;
    d.set_item("convex_hull_ratio", m.convex_hull_ratio)?;
    d.set_item("perimeter_m", m.perimeter_m)?;
    d.set_item("area_m2", m.area_m2)?;
    Ok(d.into_any().unbind().into())
}

/// Compute VRA majority-minority district analysis.
/// Returns dict: {mm_count, mm_districts, districts: [{district, pct_minority, pct_black, pct_hispanic, is_mm}]}
#[pyfunction]
#[pyo3(signature = (assignments, total_pops, minority_pops, black_pops, hispanic_pops, mm_threshold=0.50))]
fn compute_vra_analysis(
    py: Python<'_>,
    assignments: HashMap<usize, usize>,
    total_pops: Vec<i64>,
    minority_pops: Vec<f64>,
    black_pops: Vec<f64>,
    hispanic_pops: Vec<f64>,
    mm_threshold: f64,
) -> PyResult<PyObject> {
    // Validate that all per-tract arrays have the same length
    let n = total_pops.len();
    if minority_pops.len() != n {
        return Err(PyValueError::new_err(format!(
            "length mismatch: total_pops={n}, minority_pops={}", minority_pops.len()
        )));
    }
    if black_pops.len() != n {
        return Err(PyValueError::new_err(format!(
            "length mismatch: total_pops={n}, black_pops={}", black_pops.len()
        )));
    }
    if hispanic_pops.len() != n {
        return Err(PyValueError::new_err(format!(
            "length mismatch: total_pops={n}, hispanic_pops={}", hispanic_pops.len()
        )));
    }
    let vra = rust_vra_analysis(
        &assignments, &total_pops, &minority_pops, &black_pops, &hispanic_pops, mm_threshold
    );
    let d = pyo3::types::PyDict::new_bound(py);
    d.set_item("mm_count", vra.mm_count)?;
    d.set_item("mm_districts", &vra.mm_districts)?;
    let districts: Vec<_> = vra.districts.iter().map(|dist| {
        let dd = pyo3::types::PyDict::new_bound(py);
        dd.set_item("district", dist.district).unwrap();
        dd.set_item("pct_minority", dist.pct_minority).unwrap();
        dd.set_item("pct_black", dist.pct_black).unwrap();
        dd.set_item("pct_hispanic", dist.pct_hispanic).unwrap();
        dd.set_item("is_mm", dist.is_mm).unwrap();
        dd.into_any().unbind()
    }).collect();
    d.set_item("districts", districts)?;
    Ok(d.into_any().unbind().into())
}

// ---------------------------------------------------------------------------
// TIGER shapefile reader
// ---------------------------------------------------------------------------

/// Read census tract records from a TIGER .shp file.
///
/// Returns list of (geoid, geometry_wkb, aland, awater) tuples.
/// Records sorted by GEOID. population=0 sentinel — join PL 94-171 separately.
#[pyfunction]
fn read_tiger_shp(
    py: Python<'_>,
    shp_path: String,
) -> PyResult<Vec<(String, pyo3::Py<pyo3::types::PyBytes>, i64, i64)>> {
    let records = read_tiger_tracts(&shp_path)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    records.iter().map(|r| {
        let wkb_bytes = pyo3::types::PyBytes::new_bound(py, &r.geometry_wkb).unbind();
        Ok((r.geoid.clone(), wkb_bytes, r.aland, r.awater))
    }).collect()
}

// ---------------------------------------------------------------------------
// Adjacency serialization
// ---------------------------------------------------------------------------

/// Serialize an adjacency graph to .adj.bin bytes (format v2 with vertex_weights).
#[pyfunction]
fn adjacency_to_bin(
    py: Python<'_>,
    adjacency: Vec<Vec<usize>>,
    vertex_weights: Vec<i64>,
    edge_weights: HashMap<(usize, usize), f64>,
    n_vertices: usize,
    n_edges: usize,
) -> PyResult<pyo3::Py<pyo3::types::PyBytes>> {
    let graph = AdjacencyGraph { adjacency, vertex_weights, edge_weights, n_vertices, n_edges, vertex_areas: vec![], vertex_ext_perimeters: vec![] };
    let bytes = serialize_adjacency(&graph);
    Ok(pyo3::types::PyBytes::new_bound(py, &bytes).unbind())
}

/// Deserialize .adj.bin bytes to an adjacency graph dict.
#[pyfunction]
fn adjacency_from_bin(
    py: Python<'_>,
    data: Vec<u8>,
) -> PyResult<PyObject> {
    let graph = deserialize_adjacency(&data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let d = pyo3::types::PyDict::new_bound(py);
    d.set_item("adjacency", &graph.adjacency)?;
    d.set_item("vertex_weights", &graph.vertex_weights)?;
    d.set_item("edge_weights", &graph.edge_weights
        .iter().map(|(&(u,v), &w)| ((u,v), w)).collect::<HashMap<_,_>>())?;
    d.set_item("n_vertices", graph.n_vertices)?;
    d.set_item("n_edges", graph.n_edges)?;
    Ok(d.into_any().unbind().into())
}

// ---------------------------------------------------------------------------
// Island bridging
// ---------------------------------------------------------------------------

/// Connect disconnected graph components (islands) to the main component.
///
/// Args:
///   adjacency: list[list[int]] — CSR adjacency from build_adjacency
///   centroids: list[(float, float)] — projected (x, y) centroid per tract
///   geoids: list[str] — 11-char GEOID per tract (for county extraction)
///
/// Returns list of (u, v) tuples to add as bridge edges (canonical u < v).
#[pyfunction]
fn connect_islands(
    adjacency: Vec<Vec<usize>>,
    centroids: Vec<(f64, f64)>,
    geoids: Vec<String>,
) -> Vec<(usize, usize)> {
    connect_island_components(&adjacency, &centroids, &geoids)
}

// ---------------------------------------------------------------------------
// Adjacency builder
// ---------------------------------------------------------------------------

/// Build spatial adjacency graph from WKB-encoded projected polygons.
///
/// Args:
///   polygons_wkb: list of WKB bytes (must be in equal-area projected CRS)
///   min_boundary_length: minimum shared boundary in metres (default 10.0)
///
/// Returns dict with:
///   'adjacency': list[list[int]] — CSR adjacency (adjacency[i] = sorted neighbors)
///   'edge_weights': dict[(int,int) -> float] — canonical (min,max) → length in metres
///   'n_vertices': int
///   'n_edges': int
#[pyfunction]
#[pyo3(signature = (polygons_wkb, min_boundary_length=10.0))]
fn build_adjacency(
    py: Python<'_>,
    polygons_wkb: Vec<Vec<u8>>,
    min_boundary_length: f64,
) -> PyResult<PyObject> {
    let graph = build_adjacency_graph(&polygons_wkb, min_boundary_length)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    let d = pyo3::types::PyDict::new_bound(py);
    d.set_item("adjacency", &graph.adjacency)?;
    d.set_item("edge_weights", &graph.edge_weights
        .iter()
        .map(|(&(u, v), &w)| ((u, v), w))
        .collect::<HashMap<_, _>>()
    )?;
    d.set_item("n_vertices", graph.n_vertices)?;
    d.set_item("n_edges", graph.n_edges)?;
    Ok(d.into_any().unbind().into())
}

// ---------------------------------------------------------------------------
// METIS file format
// ---------------------------------------------------------------------------

/// Generate METIS .graph file content as a Python string.
/// Edge weights: dict mapping (u, v) → weight in metres. Missing = 1.0m.
/// The caller writes this to disk and runs gpmetis.
#[pyfunction]
#[pyo3(signature = (adjacency, vertex_weights, edge_weights=None))]
fn metis_graph_content(
    adjacency: Vec<Vec<usize>>,
    vertex_weights: Vec<i64>,
    edge_weights: Option<HashMap<(usize, usize), f64>>,
) -> PyResult<String> {
    let ew_ref = edge_weights.as_ref();
    core_write_metis(&adjacency, &vertex_weights, ew_ref)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Parse gpmetis output partition file content (one assignment per line).
/// Returns list of partition IDs (0-based) matching vertex order.
#[pyfunction]
fn metis_parse_partition(content: String, expected_n: usize) -> PyResult<Vec<usize>> {
    core_parse_metis(&content, expected_n)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

// ---------------------------------------------------------------------------
// BisectionTree
// ---------------------------------------------------------------------------

/// Split schedule for k districts: which k_left/k_right at each depth.
/// Use nodes_at_depth(d) to drive parallel METIS calls level by level.
#[pyclass]
struct BisectionTree {
    inner: CoreBisectionTree,
}

#[pymethods]
impl BisectionTree {
    #[staticmethod]
    fn from_k(k: usize) -> PyResult<Self> {
        if k == 0 {
            return Err(PyValueError::new_err("k must be >= 1"));
        }
        Ok(BisectionTree { inner: CoreBisectionTree::from_k(k) })
    }

    fn max_depth(&self) -> usize { self.inner.max_depth }
    fn total_splits(&self) -> usize { self.inner.total_splits() }

    /// List of (k, k_left, k_right, depth, path) tuples for nodes at this depth.
    fn nodes_at_depth(&self, depth: usize) -> Vec<(usize, usize, usize, usize, String)> {
        self.inner.nodes_at_depth(depth)
            .iter()
            .map(|n| (n.k, n.k_left, n.k_right, n.depth, n.path.clone()))
            .collect()
    }

    /// Splits per depth as a list: [count_at_depth_0, count_at_depth_1, ...]
    fn splits_per_depth(&self) -> Vec<usize> {
        self.inner.splits_per_depth()
    }
}

#[pyfunction]
fn bisection_max_depth(k: usize) -> usize {
    max_depth_for_k(k)
}

#[pyfunction]
#[pyo3(signature = (depth, base_ufactor=5))]
fn bisection_ufactor(depth: usize, base_ufactor: u32) -> f64 {
    ufactor_for_depth(depth, base_ufactor)
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

#[pymodule]
fn redist_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Graph>()?;
    m.add_class::<Partition>()?;
    m.add_class::<BisectionTree>()?;
    m.add_function(wrap_pyfunction!(build_vra_edge_weights, m)?)?;
    m.add_function(wrap_pyfunction!(build_partisan_weights, m)?)?;
    m.add_function(wrap_pyfunction!(bisection_max_depth, m)?)?;
    m.add_function(wrap_pyfunction!(bisection_ufactor, m)?)?;
    m.add_function(wrap_pyfunction!(metis_graph_content, m)?)?;
    m.add_function(wrap_pyfunction!(metis_parse_partition, m)?)?;
    m.add_function(wrap_pyfunction!(read_tiger_shp, m)?)?;
    m.add_function(wrap_pyfunction!(compute_polsby_popper, m)?)?;
    m.add_function(wrap_pyfunction!(compute_reock, m)?)?;
    m.add_function(wrap_pyfunction!(compute_all_compactness, m)?)?;
    m.add_function(wrap_pyfunction!(compute_vra_analysis, m)?)?;
    m.add_function(wrap_pyfunction!(build_adjacency, m)?)?;
    m.add_function(wrap_pyfunction!(connect_islands, m)?)?;
    m.add_function(wrap_pyfunction!(adjacency_to_bin, m)?)?;
    m.add_function(wrap_pyfunction!(adjacency_from_bin, m)?)?;
    Ok(())
}
