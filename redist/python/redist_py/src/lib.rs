use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use numpy::PyReadonlyArray1;
use std::collections::HashMap;
use redist_core::{Graph as CoreGraph, Partition as CorePartition, build_vra_edge_weights as core_vra};

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

/// Census tract adjacency graph. CSR format: adjacency[i] = list of neighbors.
#[pyclass]
struct Graph {
    inner: CoreGraph,
}

#[pymethods]
impl Graph {
    /// Build from a Python dict with keys:
    ///   - 'adjacency': list[list[int]]  (CSR — adjacency[i] = neighbor indices)
    ///   - 'vertex_weights': list[int] or 1D numpy array (population per tract)
    ///   - 'n_vertices': int
    ///
    /// Raises ValueError if vertex_weights is 2D (multi-constraint not supported).
    #[staticmethod]
    fn from_csr(data: &PyAny) -> PyResult<Self> {
        let py = data.py();

        // Extract adjacency
        let adj_obj = data.get_item("adjacency")?;
        let adjacency: Vec<Vec<usize>> = adj_obj.extract()?;

        // Extract vertex_weights — reject if 2D
        let wt_obj = data.get_item("vertex_weights")?;

        // Check if it's a numpy array with ndim > 1
        if let Ok(arr) = wt_obj.extract::<PyReadonlyArray1<i64>>() {
            let weights: Vec<i64> = arr.as_slice()?.to_vec();
            let inner = CoreGraph::new(adjacency, weights)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            return Ok(Graph { inner });
        }

        // Try extracting as plain Python list/sequence
        if let Ok(weights) = wt_obj.extract::<Vec<i64>>() {
            let inner = CoreGraph::new(adjacency, weights)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            return Ok(Graph { inner });
        }

        // If it looks 2D (list of lists or 2D ndarray)
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

/// District assignment: maps tract_index → district_id.
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

    /// Max absolute fractional deviation from ideal district population.
    /// Only valid on final leaf partitions, not intermediate bisection nodes.
    fn population_balance(&self, vertex_weights: PyReadonlyArray1<i64>, n_districts: usize) -> PyResult<f64> {
        let weights = vertex_weights.as_slice()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(self.inner.population_balance(weights, n_districts))
    }

    /// Raises ValueError if any district exceeds tolerance (default ±0.5%).
    fn assert_balanced(
        &self,
        vertex_weights: PyReadonlyArray1<i64>,
        n_districts: usize,
        #[pyo3(signature = (tolerance=0.005))]
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
///
/// α = max(3.0, 10.0 × (1.0 − 0.7 × f_minority))
/// where f_minority = fraction of tracts with minority_frac ≥ threshold.
///
/// Returns a dict mapping (u, v) → weight for boosted edges only.
/// Unboosted edges have implicit weight 1.0.
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
// Module
// ---------------------------------------------------------------------------

#[pymodule]
fn redist_py(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Graph>()?;
    m.add_class::<Partition>()?;
    m.add_function(wrap_pyfunction!(build_vra_edge_weights, m)?)?;
    Ok(())
}
