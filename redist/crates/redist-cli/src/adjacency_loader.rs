/// Adjacency graph loader — native Rust, no Python subprocess.
///
/// Reads `.adj.bin` (binary format) + `_geoids.json` (GEOID mapping).
/// Falls back to the Python pkl shim if `.adj.bin` not found, so the
/// transition is backward-compatible.
///
/// The `.adj.bin` format is written by `redist_py.adjacency_to_bin()` and
/// read by `redist_data::deserialize_adjacency()`. It is ~2-5× smaller than
/// pkl and loads in <1ms (vs ~200ms for the Python shim).
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use redist_data::deserialize_adjacency;

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

/// Load adjacency graph from either a `.adj.bin` path or a `.pkl` path.
///
/// - If `path` ends in `.adj.bin`: loads directly (used for `--adjacency` override
///   with international states where the path IS the binary file).
/// - Otherwise: treats path as `.pkl`, derives `.adj.bin` from it, and falls
///   back to the Python pkl shim if `.adj.bin` is not present.
pub fn load_adjacency_pkl(path: &Path) -> Result<LoadedGraph, String> {
    let path_str = path.to_string_lossy();
    if path_str.ends_with(".adj.bin") {
        // Direct .adj.bin path — common when using --adjacency override
        let base = path_str.strip_suffix(".adj.bin").unwrap_or(&path_str);
        let geoid_path = std::path::PathBuf::from(format!("{base}_geoids.json"));
        return load_from_bin(path, &geoid_path);
    }

    // Legacy: derive .adj.bin from .pkl path
    let bin_path = path.with_extension("adj.bin");
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let geoid_path = path.with_file_name(format!("{stem}_geoids.json"));

    if bin_path.exists() {
        load_from_bin(&bin_path, &geoid_path)
    } else {
        eprintln!(
            "WARNING: {} not found — falling back to Python pkl shim. \
             Run scripts/data/convert_adj_bin_to_pkl.py to generate .adj.bin files.",
            bin_path.display()
        );
        load_from_pkl_shim(path)
    }
}

/// Load from native .adj.bin + _geoids.json — zero Python, sub-millisecond.
fn load_from_bin(bin_path: &Path, geoid_path: &Path) -> Result<LoadedGraph, String> {
    if !bin_path.exists() {
        return Err(format!("adjacency .adj.bin not found: {}", bin_path.display()));
    }

    let bytes = std::fs::read(bin_path)
        .map_err(|e| format!("read {}: {e}", bin_path.display()))?;

    let graph = deserialize_adjacency(&bytes)
        .map_err(|e| format!("deserialize {}: {e}", bin_path.display()))?;

    // Load geoid mapping (optional — used for VRA demographics alignment)
    let index_to_geoid: HashMap<usize, String> = if geoid_path.exists() {
        let content = std::fs::read_to_string(geoid_path)
            .map_err(|e| format!("read geoids {}: {e}", geoid_path.display()))?;
        let raw: HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| format!("parse geoids: {e}"))?;
        raw.into_iter()
            .filter_map(|(k, v)| k.parse::<usize>().ok().map(|ki| (ki, v)))
            .collect()
    } else {
        HashMap::new()
    };

    let n_vertices = graph.n_vertices;
    let n_edges = graph.n_edges;

    Ok(LoadedGraph {
        adjacency: graph.adjacency,
        vertex_weights: graph.vertex_weights,
        edge_weights: graph.edge_weights,
        index_to_geoid,
        n_vertices,
        n_edges,
    })
}

/// Python pkl shim fallback (used when .adj.bin not present).
/// REDIST_PYTHON must be set to the Python executable with numpy available.
fn load_from_pkl_shim(pkl_path: &Path) -> Result<LoadedGraph, String> {
    if !pkl_path.exists() {
        return Err(format!("adjacency pkl not found: {}", pkl_path.display()));
    }

    // REDIST_PYTHON must be set explicitly in CI/containers.
    // Default `py` (Windows) and `python3` (Linux/macOS) may not be correct if:
    //   - Running in Alpine/minimal containers where only `python` exists
    //   - Running inside a venv where Python is at a non-standard path
    // Set REDIST_PYTHON=<full path> to override.
    let python_cmd = std::env::var("REDIST_PYTHON").unwrap_or_else(|_| {
        if cfg!(windows) { "py".to_string() } else { "python3".to_string() }
    });

    const PYTHON_DUMP: &str = r#"
import pickle, json, sys
path = sys.argv[1]
d = pickle.load(open(path, 'rb'))
adj = [list(map(int, row)) for row in d['adjacency']]
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

    #[derive(serde::Deserialize)]
    struct AdjacencyJson {
        adjacency: Vec<Vec<usize>>,
        vertex_weights: Vec<i64>,
        edge_weights_list: Vec<(usize, usize, f64)>,
        index_to_geoid: HashMap<String, String>,
    }

    let data: AdjacencyJson = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("JSON parse failed: {e}"))?;

    let edge_weights: HashMap<(usize, usize), f64> = data.edge_weights_list
        .into_iter()
        .map(|(u, v, w)| ((u.min(v), u.max(v)), w))
        .collect();

    let n_edges = edge_weights.len();
    let n_vertices = data.adjacency.len();
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

    // Derive paths for test data
    fn vt_bin_path() -> std::path::PathBuf {
        std::path::Path::new("outputs/V3/data/2020/adjacency/vt_adjacency_2020.adj.bin")
            .to_path_buf()
    }

    fn vt_pkl_path() -> std::path::PathBuf {
        std::path::Path::new("outputs/V3/data/2020/adjacency/vt_adjacency_2020.pkl")
            .to_path_buf()
    }

    #[test]
    fn test_load_from_bin_vermont() {
        if !vt_bin_path().exists() { return; }
        let graph = load_from_bin(
            &vt_bin_path(),
            &vt_bin_path().with_file_name("vt_adjacency_2020_geoids.json"),
        ).expect("should load VT .adj.bin");
        assert_eq!(graph.n_vertices, 193, "VT should have 193 tracts");
        assert_eq!(graph.n_edges, 500, "VT should have 500 edges");
        assert!(graph.vertex_weights.iter().all(|&v| v > 0), "all weights positive");
    }

    #[test]
    fn test_load_from_bin_geoids_present() {
        if !vt_bin_path().exists() { return; }
        let geoid_path = vt_bin_path().with_file_name("vt_adjacency_2020_geoids.json");
        let graph = load_from_bin(&vt_bin_path(), &geoid_path).unwrap();
        if geoid_path.exists() {
            assert!(!graph.index_to_geoid.is_empty(), "GEOIDs should be loaded");
            for (_, geoid) in &graph.index_to_geoid {
                assert_eq!(geoid.len(), 11, "GEOID must be 11 chars: {geoid}");
                assert!(geoid.starts_with("50"), "VT GEOIDs start with 50");
            }
        }
    }

    #[test]
    fn test_load_adjacency_pkl_prefers_bin() {
        // When .adj.bin exists, no Python subprocess should be needed
        if !vt_bin_path().exists() { return; }
        // Temporarily set REDIST_PYTHON to a nonexistent path — if bin is used,
        // this won't matter (no subprocess); if pkl shim is used, it will fail
        let old = std::env::var("REDIST_PYTHON").ok();
        std::env::set_var("REDIST_PYTHON", "/nonexistent/python");
        let result = load_adjacency_pkl(&vt_pkl_path());
        if let Some(old_val) = old {
            std::env::set_var("REDIST_PYTHON", old_val);
        } else {
            std::env::remove_var("REDIST_PYTHON");
        }
        assert!(result.is_ok(), ".adj.bin should be used without Python: {:?}", result.err());
        let graph = result.unwrap();
        assert_eq!(graph.n_vertices, 193);
    }

    #[test]
    fn test_edge_weights_canonical_order() {
        if !vt_bin_path().exists() { return; }
        let graph = load_from_bin(
            &vt_bin_path(),
            &vt_bin_path().with_file_name("vt_adjacency_2020_geoids.json"),
        ).unwrap();
        for &(u, v) in graph.edge_weights.keys() {
            assert!(u < v, "edge ({u},{v}) not canonical after bin load");
        }
    }

    #[test]
    fn test_missing_bin_falls_back_to_pkl() {
        // Test that a nonexistent .adj.bin triggers fallback (not a panic)
        let fake_pkl = std::path::Path::new("/nonexistent/fake_adjacency_2020.pkl");
        let result = load_adjacency_pkl(fake_pkl);
        assert!(result.is_err(), "should fail if neither bin nor pkl exists");
        // Error message should mention the file path
        let err = result.unwrap_err();
        assert!(err.contains("not found") || err.contains("adjacency"),
            "error should be descriptive: {err}");
    }
}
