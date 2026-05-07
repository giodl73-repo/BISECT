"""
L2 tests for redist_py.metis_graph_content and metis_parse_partition.

Verifies the Rust METIS file format writer and parser match the Python
implementation (_write_metis_graph / _read_metis_partition in metis_executable.py).
"""

import os
import pytest

RUST_AVAILABLE = os.environ.get('REDIST_NO_RUST', '0') != '1'
try:
    import redist_py
    REDIST_PY_IMPORTABLE = True
except ImportError:
    REDIST_PY_IMPORTABLE = False

pytestmark = pytest.mark.skipif(
    not RUST_AVAILABLE or not REDIST_PY_IMPORTABLE,
    reason='redist_py not available'
)

# Simple linear graph: 0-1-2-3
LINEAR_ADJ = [[1], [0, 2], [1, 3], [2]]
LINEAR_VW = [1000, 1200, 900, 1100]


class TestMetisGraphContent:

    def test_header_no_edge_weights(self):
        content = redist_py.metis_graph_content(LINEAR_ADJ, LINEAR_VW)
        header = content.splitlines()[0]
        # 4 vertices, 3 edges, fmt 010, ncon 1
        assert header == '4 3 010 1'

    def test_header_with_edge_weights(self):
        ew = {(0, 1): 150.0}
        content = redist_py.metis_graph_content(LINEAR_ADJ, LINEAR_VW, edge_weights=ew)
        header = content.splitlines()[0]
        assert header == '4 3 011 1'

    def test_vertex_count(self):
        content = redist_py.metis_graph_content(LINEAR_ADJ, LINEAR_VW)
        # header + 4 vertex lines
        assert len(content.splitlines()) == 5

    def test_one_based_neighbor_indices(self):
        content = redist_py.metis_graph_content(LINEAR_ADJ, LINEAR_VW)
        lines = content.splitlines()
        # Vertex 0: weight=1000, neighbor=1 (0-based) → "2" (1-based)
        assert lines[1] == '1000 2'
        # Vertex 1: weight=1200, neighbors=0,2 → "1 3"
        assert lines[2] == '1200 1 3'
        # Vertex 3: weight=1100, neighbor=2 → "3"
        assert lines[4] == '1100 3'

    def test_edge_weights_scaled_to_cm(self):
        ew = {(0, 1): 2.5}  # 2.5 metres → 250 cm
        content = redist_py.metis_graph_content(LINEAR_ADJ, LINEAR_VW, edge_weights=ew)
        lines = content.splitlines()
        # Vertex 0: "1000 2 250" (2.5m = 250cm)
        assert lines[1] == '1000 2 250'

    def test_missing_edge_defaults_to_1m_equals_100cm(self):
        # Only boost edge (0,1); edge (1,2) missing → defaults to 1.0m = 100cm
        ew = {(0, 1): 200.0}
        content = redist_py.metis_graph_content(LINEAR_ADJ, LINEAR_VW, edge_weights=ew)
        lines = content.splitlines()
        # Vertex 1: neighbors 0 (boosted 20000cm) and 2 (default 100cm)
        assert lines[2] == '1200 1 20000 3 100'

    def test_empty_edge_weights_dict_treated_as_no_ew(self):
        content_none = redist_py.metis_graph_content(LINEAR_ADJ, LINEAR_VW)
        content_empty = redist_py.metis_graph_content(LINEAR_ADJ, LINEAR_VW, edge_weights={})
        # Empty dict → no edge weights → same fmt as None
        assert content_none.splitlines()[0] == content_empty.splitlines()[0]

    def test_matches_python_implementation_no_ew(self):
        """Cross-check against the Python _write_metis_graph for a known graph."""
        import sys
        from pathlib import Path
        sys.path.insert(0, str(Path(__file__).parents[2] / 'src'))
        try:
            from apportionment.partition.metis_executable import _write_metis_graph
            import numpy as np
            import tempfile
        except ImportError:
            pytest.skip('metis_executable not importable')

        vw = np.array(LINEAR_VW, dtype=np.int32)

        # Write Python version to temp file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            tmppath = f.name
        _write_metis_graph(Path(tmppath), LINEAR_ADJ, vw, edge_weights=None)
        py_content = Path(tmppath).read_text()
        Path(tmppath).unlink()

        # Compare with Rust version
        rs_content = redist_py.metis_graph_content(LINEAR_ADJ, LINEAR_VW)

        # Headers must match exactly
        assert py_content.splitlines()[0] == rs_content.splitlines()[0]
        # Same number of lines
        assert len(py_content.splitlines()) == len(rs_content.splitlines())


class TestMetisParsePartition:

    def test_parses_valid_partition(self):
        content = '0\n1\n0\n1\n'
        parts = redist_py.metis_parse_partition(content, 4)
        assert parts == [0, 1, 0, 1]

    def test_strips_whitespace(self):
        content = '  0  \n  1  \n  0  \n  1  \n'
        parts = redist_py.metis_parse_partition(content, 4)
        assert parts == [0, 1, 0, 1]

    def test_ignores_blank_lines(self):
        content = '0\n\n1\n0\n\n1\n'
        parts = redist_py.metis_parse_partition(content, 4)
        assert parts == [0, 1, 0, 1]

    def test_length_mismatch_raises(self):
        with pytest.raises(ValueError):
            redist_py.metis_parse_partition('0\n1\n0\n', 4)

    def test_invalid_content_raises(self):
        with pytest.raises(ValueError):
            redist_py.metis_parse_partition('0\nbad\n1\n1\n', 4)

    def test_single_partition_valid(self):
        parts = redist_py.metis_parse_partition('0\n', 1)
        assert parts == [0]
