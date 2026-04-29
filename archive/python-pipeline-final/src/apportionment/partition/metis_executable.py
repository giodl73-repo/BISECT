"""
Direct wrapper for METIS gpmetis executable.

Calls gpmetis.exe directly, bypassing pymetis. Useful when pymetis
compilation fails but you have a compiled METIS binary.
"""

import os
import subprocess
import tempfile
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np


def find_gpmetis_executable() -> Optional[str]:
    """
    Find gpmetis executable (cross-platform: Windows .exe or Linux/macOS binary).

    Searches in:
    1. Repository's bin/ directory (bundled Windows binary)
    2. System PATH (works for Linux after: apt-get install metis)
    3. Common installation directories

    Returns
    -------
    str or None
        Path to gpmetis if found
    """
    import platform
    is_windows = platform.system() == 'Windows'

    # Check repository's bin/ directory (bundled Windows binary)
    for name in ['gpmetis.exe', 'gpmetis']:
        repo_bin = Path(__file__).parent.parent.parent.parent / 'bin' / name
        if repo_bin.exists():
            return str(repo_bin.absolute())

    # Check system PATH (cross-platform)
    path_cmd = ['where', 'gpmetis'] if is_windows else ['which', 'gpmetis']
    try:
        result = subprocess.run(path_cmd, capture_output=True, text=True)
        gpmetis_path = result.stdout.strip()
        if gpmetis_path and Path(gpmetis_path).exists():
            return gpmetis_path
    except (FileNotFoundError, OSError):
        pass

    # Common Linux/macOS paths (e.g. after apt-get install metis)
    unix_paths = ['/usr/bin/gpmetis', '/usr/local/bin/gpmetis', '/opt/homebrew/bin/gpmetis']
    for path in unix_paths:
        if Path(path).exists():
            return path

    # Common Windows paths
    if is_windows:
        windows_paths = [
            r'C:\metis\bin\gpmetis.exe',
            r'C:\Program Files\metis\bin\gpmetis.exe',
        ]
        for path in windows_paths:
            if Path(path).exists():
                return path

    return None


def partition_graph_with_executable(
    adjacency: List[List[int]],
    vertex_weights: np.ndarray,
    nparts: int = 2,
    target_weights: Optional[List[float]] = None,
    ufactor: Optional[float] = None,
    ubvec: Optional[List[float]] = None,
    niter: int = 100,
    debug: bool = False,
    edge_weights: Optional[Dict[Tuple[int, int], float]] = None
) -> np.ndarray:
    """
    Partition graph using gpmetis.exe executable.

    Parameters
    ----------
    adjacency : List[List[int]]
        Adjacency list in CSR format
    vertex_weights : np.ndarray
        Vertex weights
    nparts : int, default 2
        Number of partitions
    target_weights : List[float], optional
        Target partition weights (not used - gpmetis doesn't support this directly)
    ufactor : float, optional
        Load imbalance tolerance factor
    niter : int, default 20
        Number of refinement iterations (default 10 in METIS)
    debug : bool, default False
        Print detailed progress and command information
    edge_weights : Dict[Tuple[int, int], float], optional
        Edge weights mapping (i, j) tuples to boundary lengths in meters.
        If provided, METIS will minimize weighted edge cut (boundary length).
        If None, METIS minimizes unweighted edge cut.

    Returns
    -------
    np.ndarray
        Partition assignments

    Raises
    ------
    RuntimeError
        If gpmetis.exe not found or execution fails
    """
    # Find gpmetis executable
    gpmetis_exe = find_gpmetis_executable()
    if not gpmetis_exe:
        raise RuntimeError(
            "gpmetis.exe not found. Please:\n"
            "  1. Build METIS using scripts/build_metis_windows.bat\n"
            "  2. Add gpmetis.exe to your PATH\n"
            "  3. Or install pymetis: pip install pymetis"
        )

    # Create temporary files for input/output
    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)

        # Write graph in METIS format
        graph_file = tmpdir / 'graph.txt'
        _write_metis_graph(graph_file, adjacency, vertex_weights, edge_weights=edge_weights, debug=debug)

        # Run gpmetis
        partition_file = tmpdir / 'graph.txt.part.{}'.format(nparts)

        # Note: Don't use -ptype=rb with -contig; rb doesn't support contiguity
        # We implement recursive bisection at a higher level, so just use
        # default k-way partitioning with contiguity for each split

        # Use provided ufactor or default to 1.005 (0.5% imbalance tolerance)
        ufactor_value = ufactor if ufactor is not None else 1.005

        # Detect multi-constraint mode
        is_multi_constraint = len(vertex_weights.shape) == 2 and vertex_weights.shape[1] > 1

        cmd = [
            gpmetis_exe,
            '-contig',        # Ensure contiguous districts (CRITICAL)
            '-minconn',       # Minimize subdomain connectivity
            f'-niter={niter}',  # Number of refinement iterations
        ]

        # For multi-constraint: use ufactor (no ubvec by default, let tpwgts guide)
        # For single-constraint: use ufactor for imbalance tolerance
        cmd.append(f'-ufactor={ufactor_value}')

        # Add ubvec if specified (per-constraint imbalance tolerance)
        if ubvec is not None:
            ubvec_str = ' '.join(str(v) for v in ubvec)
            cmd.append(f'-ubvec={ubvec_str}')
            if debug:
                print(f"  Using ubvec: {ubvec_str}")

        if debug and is_multi_constraint and ubvec is None:
            print(f"  Multi-constraint mode: using ufactor={ufactor_value}, relying on tpwgts for guidance")


        # Add target partition weights if specified
        # gpmetis expects a file with format: "partition_id = weight" per line (single constraint)
        # or "partition_id : constraint_id = weight" per line (multi-constraint)
        if target_weights is not None and len(target_weights) == nparts:
            tpwgts_file = tmpdir / 'tpwgts.txt'

            # Check if multi-constraint (2D list)
            is_multi = isinstance(target_weights[0], (list, np.ndarray))

            with open(tpwgts_file, 'w') as f:
                if is_multi:
                    # Multi-constraint format: partition_id : constraint_id = weight
                    # target_weights = [[left_pop, left_min], [right_pop, right_min]]
                    ncon = len(target_weights[0])
                    for constraint_id in range(ncon):
                        # Normalize weights for this constraint
                        weights_for_constraint = [target_weights[p][constraint_id] for p in range(nparts)]
                        total = sum(weights_for_constraint)

                        # Write n-1 partitions, METIS infers the last one (avoids floating point issues)
                        for partition_id in range(nparts - 1):
                            weight = weights_for_constraint[partition_id]
                            normalized_weight = weight / total if total > 0 else 1.0 / nparts
                            f.write(f'{partition_id} : {constraint_id} = {normalized_weight:.6f}\n')
                else:
                    # Single constraint format: partition_id = weight
                    normalized = [w / sum(target_weights) for w in target_weights]
                    # Write n-1 partitions, METIS infers the last one
                    for partition_id in range(len(normalized) - 1):
                        f.write(f'{partition_id} = {normalized[partition_id]:.6f}\n')

            cmd.append(f'-tpwgt={tpwgts_file}')

        cmd.extend([str(graph_file), str(nparts)])

        if debug:
            print(f"Running: {' '.join(cmd)}")

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                check=True,
                cwd=tmpdir
            )

            # Check if partition file was created
            if not partition_file.exists():
                raise RuntimeError(
                    f"gpmetis did not create partition file: {partition_file}\n"
                    f"stdout: {result.stdout}\n"
                    f"stderr: {result.stderr}"
                )

            # Read partition assignments
            parts = _read_metis_partition(partition_file)

            return parts

        except subprocess.CalledProcessError as e:
            raise RuntimeError(
                f"gpmetis execution failed:\n"
                f"Command: {' '.join(cmd)}\n"
                f"Return code: {e.returncode}\n"
                f"stdout: {e.stdout}\n"
                f"stderr: {e.stderr}"
            )


def _write_metis_graph(
    file_path: Path,
    adjacency: List[List[int]],
    vertex_weights: np.ndarray,
    edge_weights: Optional[Dict[Tuple[int, int], float]] = None,
    debug: bool = False
):
    """
    Write graph in METIS format.

    METIS graph format:
    Line 1: <num_vertices> <num_edges> [fmt] [ncon]
      fmt = 010 (vertex weights, no edge weights) or 011 (vertex weights + edge weights)
      ncon = 1 (one weight per vertex) or 2 (multi-constraint: population + minority VAP)
    Line 2+:
      Single constraint: <vertex_weight> <neighbor1> <neighbor2> ...
      Multi-constraint: <weight1> <weight2> <neighbor1> <neighbor2> ...
      With edge weights: <vertex_weight(s)> <neighbor1> <edge_weight1> <neighbor2> <edge_weight2> ...

    Note: METIS uses 1-based indexing!
    Note: Edge weights must be positive integers (we scale boundary lengths to centimeters)
    """
    n_vertices = len(adjacency)
    n_edges = sum(len(neighbors) for neighbors in adjacency) // 2

    has_edge_weights = edge_weights is not None and len(edge_weights) > 0

    # Detect multi-constraint mode (2D vertex_weights array)
    is_multi_constraint = len(vertex_weights.shape) == 2 and vertex_weights.shape[1] > 1
    ncon = vertex_weights.shape[1] if is_multi_constraint else 1

    # Use buffered writing for better performance
    if debug:
        mode_str = "with edge weights" if has_edge_weights else "without edge weights"
        constraint_str = f"multi-constraint (ncon={ncon})" if is_multi_constraint else "single constraint"
        print(f"Writing METIS graph {mode_str}, {constraint_str} with {n_vertices:,} vertices to {file_path.name}...")

    with open(file_path, 'w', buffering=8*1024*1024) as f:  # 8MB buffer
        # Header: n_vertices n_edges fmt ncon
        # fmt: 010 = vertex weights only, 011 = vertex weights + edge weights
        fmt_code = "011" if has_edge_weights else "010"
        f.write(f"{n_vertices} {n_edges} {fmt_code} {ncon}\n")

        # Write vertices in chunks to avoid memory issues
        chunk_size = 50000
        for start_idx in range(0, n_vertices, chunk_size):
            end_idx = min(start_idx + chunk_size, n_vertices)

            # Build chunk of lines
            lines = []
            for i in range(start_idx, end_idx):
                # Get vertex weights (handle both single and multi-constraint)
                if is_multi_constraint:
                    # Multi-constraint: format weights as space-separated integers
                    weight_str = ' '.join(str(int(vertex_weights[i, j])) for j in range(ncon))
                else:
                    # Single constraint
                    weight_str = str(int(vertex_weights[i]))

                neighbors = adjacency[i]

                if has_edge_weights:
                    # Format: vertex_weight(s) neighbor1 edge_weight1 neighbor2 edge_weight2 ...
                    # Edge weights in centimeters (multiply by 100 for integer precision)
                    neighbor_pairs = []
                    for n in neighbors:
                        edge_key = (min(i, n), max(i, n))  # Canonical ordering
                        edge_weight_m = edge_weights.get(edge_key, 1.0)  # Default 1m if missing
                        edge_weight_cm = max(1, int(edge_weight_m * 100))  # Convert to cm, min 1
                        neighbor_pairs.append(f"{n + 1} {edge_weight_cm}")
                    neighbors_str = ' '.join(neighbor_pairs)
                else:
                    # Format: vertex_weight(s) neighbor1 neighbor2 ...
                    neighbors_str = ' '.join(str(n + 1) for n in neighbors)

                lines.append(f"{weight_str} {neighbors_str}\n")

            # Write chunk
            f.writelines(lines)

            if debug and (start_idx // chunk_size + 1) % 5 == 0:  # Print every 5 chunks
                print(f"  Written {end_idx:,} / {n_vertices:,} vertices...")

    if debug:
        print(f"  Graph file written successfully")


def _read_metis_partition(file_path: Path) -> np.ndarray:
    """
    Read partition assignments from METIS output file.

    METIS partition file format:
    Each line contains the partition ID (0-based) for that vertex.
    """
    with open(file_path, 'r') as f:
        parts = [int(line.strip()) for line in f]

    return np.array(parts, dtype=np.int32)


def test_gpmetis_installation():
    """Test if gpmetis.exe is available and working."""
    gpmetis_exe = find_gpmetis_executable()

    if gpmetis_exe:
        print(f"[OK] Found gpmetis.exe at: {gpmetis_exe}")

        # Test execution
        try:
            result = subprocess.run(
                [gpmetis_exe],
                capture_output=True,
                text=True
            )
            # gpmetis without args should print usage and exit with non-zero
            if 'Usage:' in result.stdout or 'Usage:' in result.stderr:
                print("[OK] gpmetis.exe is executable and working")
                return True
            else:
                print("[WARNING] gpmetis.exe found but may not be working correctly")
                return False
        except Exception as e:
            print(f"[ERROR] Error testing gpmetis.exe: {e}")
            return False
    else:
        print("[NOT FOUND] gpmetis.exe not found")
        print("\nSearch paths checked:")
        print("  - System PATH")
        print("  - C:\\metis\\bin\\gpmetis.exe")
        print("  - C:\\Program Files\\metis\\bin\\gpmetis.exe")
        print("  - C:\\Users\\giodl\\sources\\repo\\Apportionment\\tools\\...")
        return False


if __name__ == '__main__':
    print("Testing METIS gpmetis.exe installation...")
    test_gpmetis_installation()
