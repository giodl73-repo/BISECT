/// Adjacency graph loader: pickle → Python subprocess shim → Rust types.
///
/// Rust cannot natively read Python pickle. This module invokes a one-liner
/// Python process to convert the pkl to JSON, then Rust parses the JSON.
/// The pkl load is ms-scale so subprocess overhead is negligible.
///
/// DEPENDENCY: Python must be in PATH with access to numpy and pickle.
/// This dependency is eliminated in Phase 2a when the Rust TIGER reader
/// produces .adj.bin natively.
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AdjacencyJson {
    adjacency: Vec<Vec<usize>>,
    /// vertex_weights as integers (numpy int32 → Python int → JSON number → i64)
    vertex_weights: Vec<i64>,
    /// edge weights as list of [u, v, weight] triples (JSON can't use tuple keys)
    edge_weights_list: Vec<(usize, usize, f64)>,
    index_to_geoid: HashMap<String, String>,
}

/// Loaded adjacency graph ready for use in run_single_state.
#[derive(Debug, Clone)]
pub struct LoadedGraph {
    pub adjacency: Vec<Vec<usize>>,
    pub vertex_weights: Vec<i64>,
    /// Canonical (min, max) → boundary length in metres
    pub edge_weights: HashMap<(usize, usize), f64>,
    /// adjacency index → 11-char GEOID
    pub index_to_geoid: HashMap<usize, String>,
    pub n_vertices: usize,
    pub n_edges: usize,
}

/// Python one-liner that dumps a pkl adjacency graph as JSON.
/// numpy int32 values are forced to Python int before JSON serialisation.
const PYTHON_DUMP: &str = r#"
import pickle, json, sys
path = sys.argv[1]
d = pickle.load(open(path, 'rb'))
adj = [list(map(int, row)) for row in d['adjacency']]
# vertex_weights is a numpy int32 array; .tolist() converts without importing numpy
try:
    vw = d['vertex_weights'].tolist()
except AttributeError:
    vw = [int(x) for x in d['vertex_weights']]
ew = d.get('edge_weights') or {}
ew_list = [[int(i), int(j), float(w)] for (i, j), w in ew.items()]
ig = {str(k): str(v) for k, v in d.get('index_to_geoid', {}).items()}
print(json.dumps({'adjacency': adj, 'vertex_weights': vw,
                  'edge_weights_list': ew_list, 'index_to_geoid': ig}))
"#;

/// Load an adjacency graph from a Python pickle file.
///
/// Internally spawns one Python subprocess to convert pkl → JSON.
/// Returns an error if Python is not available or the file is malformed.
pub fn load_adjacency_pkl(pkl_path: &Path) -> Result<LoadedGraph, String> {
    if !pkl_path.exists() {
        return Err(format!("adjacency pkl not found: {}", pkl_path.display()));
    }

    // REDIST_PYTHON must be set explicitly in CI/containers (Critical 4).
    // Default `py` (Windows) and `python3` (Linux/macOS) may not be correct if:
    //   - Running in Alpine/minimal containers where only `python` exists
    //   - Running inside a venv where Python is at a non-standard path
    // Set REDIST_PYTHON=<full path> to override. Example:
    //   REDIST_PYTHON=$(which python3) redist state --state VT
    let python_cmd = std::env::var("REDIST_PYTHON").unwrap_or_else(|_| {
        if cfg!(windows) { "py".to_string() } else { "python3".to_string() }
    });

    let output = Command::new(&python_cmd)
        .args(["-c", PYTHON_DUMP, pkl_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("failed to invoke {python_cmd}: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "python pkl dump failed (rc={}):\n{}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let data: AdjacencyJson = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("JSON parse failed: {e}\nraw: {}", String::from_utf8_lossy(&output.stdout).chars().take(200).collect::<String>()))?;

    // Build canonical (min,max) edge weight map
    let edge_weights: HashMap<(usize, usize), f64> = data.edge_weights_list
        .into_iter()
        .map(|(u, v, w)| ((u.min(v), u.max(v)), w))
        .collect();

    let n_edges = edge_weights.len();
    let n_vertices = data.adjacency.len();

    // Convert string keys to usize
    let index_to_geoid: HashMap<usize, String> = data.index_to_geoid
        .into_iter()
        .filter_map(|(k, v)| k.parse::<usize>().ok().map(|ki| (ki, v)))
        .collect();

    Ok(LoadedGraph {
        adjacency: data.adjacency,
        vertex_weights: data.vertex_weights,
        edge_weights,
        index_to_geoid,
        n_vertices,
        n_edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacency_loader_on_vermont() {
        let pkl = std::path::Path::new(
            "outputs/V3/data/2020/adjacency/vt_adjacency_2020.pkl"
        );
        if !pkl.exists() {
            return; // skip if data not present
        }
        let graph = load_adjacency_pkl(pkl).expect("should load VT adjacency");
        assert_eq!(graph.n_vertices, 193, "Vermont should have 193 tracts");
        assert_eq!(graph.n_edges, 500, "Vermont should have 500 edges");
        assert_eq!(graph.vertex_weights.len(), 193);
        // All edge keys should be canonical
        for &(u, v) in graph.edge_weights.keys() {
            assert!(u < v, "edge ({u},{v}) not canonical");
        }
    }

    #[test]
    fn test_load_missing_file_returns_error() {
        let result = load_adjacency_pkl(std::path::Path::new("/nonexistent/path.pkl"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
