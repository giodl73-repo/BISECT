use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use numpy::PyReadonlyArray1;
use std::collections::HashMap;
use redist_core::{
    Graph as CoreGraph, Partition as CorePartition, build_vra_edge_weights as core_vra,
    BisectionTree as CoreBisectionTree, max_depth_for_k, ufactor_for_depth,
};

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
    Ok(())
}
