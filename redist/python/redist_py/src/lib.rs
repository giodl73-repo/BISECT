use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use numpy::PyReadonlyArray1;
use std::collections::HashMap;
use redist_core::{
    Graph as CoreGraph, Partition as CorePartition, build_vra_edge_weights as core_vra,
    BisectionTree as CoreBisectionTree, max_depth_for_k, ufactor_for_depth,
    metis_format::{write_metis_graph as core_write_metis, parse_metis_partition as core_parse_metis},
};
use redist_data::{read_tiger_tracts, build_adjacency_graph};

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
fn bisection_ufactor(depth: usize) -> f64 {
    ufactor_for_depth(depth)
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
    m.add_function(wrap_pyfunction!(bisection_max_depth, m)?)?;
    m.add_function(wrap_pyfunction!(bisection_ufactor, m)?)?;
    m.add_function(wrap_pyfunction!(metis_graph_content, m)?)?;
    m.add_function(wrap_pyfunction!(metis_parse_partition, m)?)?;
    m.add_function(wrap_pyfunction!(read_tiger_shp, m)?)?;
    m.add_function(wrap_pyfunction!(build_adjacency, m)?)?;
    Ok(())
}
