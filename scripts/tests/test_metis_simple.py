"""
Simple METIS test with mock graph data.

Creates a small graph to verify gpmetis.exe works correctly.
"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

import tempfile
import subprocess
import numpy as np

from src.apportionment.partition.metis_executable import find_gpmetis_executable


def create_simple_graph():
    """
    Create a simple 12-vertex graph arranged in a 3x4 grid.

    Graph structure (vertices numbered 0-11):

    0 - 1 - 2 - 3
    |   |   |   |
    4 - 5 - 6 - 7
    |   |   |   |
    8 - 9 - 10- 11

    Each vertex has a population weight.
    """
    # 12 vertices
    n_vertices = 12

    # Vertex weights (populations)
    vertex_weights = np.array([
        100, 120, 110, 105,  # Row 1
        95,  130, 115, 100,  # Row 2
        110, 105, 120, 100   # Row 3
    ])

    # Build adjacency list (0-indexed for Python, will convert to 1-indexed for METIS)
    adjacency = [
        [1, 4],           # 0: neighbors are 1, 4
        [0, 2, 5],        # 1: neighbors are 0, 2, 5
        [1, 3, 6],        # 2: neighbors are 1, 3, 6
        [2, 7],           # 3: neighbors are 2, 7
        [0, 5, 8],        # 4
        [1, 4, 6, 9],     # 5
        [2, 5, 7, 10],    # 6
        [3, 6, 11],       # 7
        [4, 9],           # 8
        [5, 8, 10],       # 9
        [6, 9, 11],       # 10
        [7, 10]           # 11
    ]

    return n_vertices, vertex_weights, adjacency


def write_metis_graph(file_path, n_vertices, vertex_weights, adjacency):
    """
    Write graph in METIS format.

    Format:
    Line 1: <num_vertices> <num_edges> [fmt] [ncon]
    Line 2+: <vertex_weight> <neighbor1> <neighbor2> ...

    Note: METIS uses 1-based indexing!
    """
    n_edges = sum(len(neighbors) for neighbors in adjacency) // 2

    with open(file_path, 'w') as f:
        # Header: n_vertices n_edges fmt ncon
        # fmt=010 (has vertex weights, no edge weights), ncon=1 (one weight per vertex)
        f.write(f"{n_vertices} {n_edges} 010 1\n")

        # Write each vertex with weight and neighbors (1-based indexing)
        for i, (weight, neighbors) in enumerate(zip(vertex_weights, adjacency)):
            neighbors_1based = [str(n + 1) for n in neighbors]
            f.write(f"{int(weight)} {' '.join(neighbors_1based)}\n")


def read_metis_partition(file_path):
    """Read partition assignments from METIS output."""
    with open(file_path, 'r') as f:
        parts = [int(line.strip()) for line in f]
    return np.array(parts)


def run_simple_metis_test():
    """Run METIS on simple mock graph."""

    print("=" * 60)
    print("Simple METIS Test")
    print("=" * 60)
    print()

    # Find gpmetis
    gpmetis_exe = find_gpmetis_executable()
    if not gpmetis_exe:
        print("[ERROR] gpmetis.exe not found!")
        return

    print(f"[OK] Found gpmetis.exe at: {gpmetis_exe}")
    print()

    # Create simple graph
    print("Creating simple test graph...")
    n_vertices, vertex_weights, adjacency = create_simple_graph()

    total_population = vertex_weights.sum()
    print(f"  Vertices: {n_vertices}")
    print(f"  Total population: {total_population}")
    print(f"  Vertex weights: {vertex_weights.tolist()}")
    print()

    # Test splitting into 2, 3, 4, and 6 districts
    for nparts in [2, 3, 4, 6]:
        print(f"--- Test: Split into {nparts} districts ---")

        ideal_pop = total_population / nparts
        print(f"  Ideal population per district: {ideal_pop:.1f}")

        # Use fixed directory for debugging
        tmpdir = Path('C:/temp/metis_test')
        tmpdir.mkdir(parents=True, exist_ok=True)

        # Write graph file
        graph_file = tmpdir / 'test_graph.txt'
        write_metis_graph(graph_file, n_vertices, vertex_weights, adjacency)
        print(f"  Graph file written to: {graph_file}")

        # Run gpmetis
        partition_file = tmpdir / f'test_graph.txt.part.{nparts}'

        cmd = [gpmetis_exe, str(graph_file), str(nparts)]
        print(f"  Running: {' '.join(cmd)}")

        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=False,  # Don't raise on non-zero exit
            cwd=tmpdir
        )

        print(f"  Return code: {result.returncode}")
        if result.stdout:
            print(f"  stdout: {result.stdout[:500]}")
        if result.stderr:
            print(f"  stderr: {result.stderr[:500]}")

        # Check if successful
        if result.returncode != 0 or not partition_file.exists():
            print(f"  [ERROR] Partition file not created!")
            continue

        # Read partition assignments
        parts = read_metis_partition(partition_file)

        # Calculate statistics
        district_pops = {}
        for vertex_id, district_id in enumerate(parts):
            if district_id not in district_pops:
                district_pops[district_id] = 0
            district_pops[district_id] += vertex_weights[vertex_id]

        print(f"  [OK] Partitioning successful!")
        print(f"  Partition assignments: {parts.tolist()}")
        print(f"  District populations:")

        max_deviation = 0
        for district_id in sorted(district_pops.keys()):
            pop = district_pops[district_id]
            deviation = abs(pop - ideal_pop) / ideal_pop * 100
            max_deviation = max(max_deviation, deviation)
            print(f"    District {district_id}: {pop:>6.0f} (deviation: {deviation:>5.1f}%)")

        print(f"  Max population deviation: {max_deviation:.1f}%")

        print()

    print("=" * 60)
    print("Simple METIS Test Complete!")
    print("=" * 60)


if __name__ == '__main__':
    run_simple_metis_test()
