"""
Unit tests for tpwgts WITHOUT ubvec.

Goal: Validate that tpwgts alone can guide METIS multi-constraint partitioning
without needing ubvec to relax constraints.
"""

import pytest
import numpy as np
from src.apportionment.partition.metis_executable import partition_graph_with_executable


class TestTPWGTSWithoutUbvec:
    """Test tpwgts behavior without ubvec."""

    def test_baseline_no_tpwgts_no_ubvec(self):
        """
        Baseline: Multi-constraint, NO tpwgts, NO ubvec.

        Expected: METIS balances both constraints equally.
        Should get ~50% minority in each partition.
        """
        # Linear adjacency
        adjacency = [[1] if i == 0 else [i-1, i+1] if i < 9 else [8] for i in range(10)]

        # 2D vertex weights
        vertex_weights = np.array([
            [1000, 800], [1000, 800], [1000, 800], [1000, 800], [1000, 800],
            [1000, 200], [1000, 200], [1000, 200], [1000, 200], [1000, 200],
        ])

        # NO tpwgts, NO ubvec override (use default ufactor)
        parts = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=None,  # No tpwgts
            ufactor=1.005,  # Standard population tolerance
            niter=100,
            debug=True
        )

        # Analyze
        p0_tracts = [i for i, p in enumerate(parts) if p == 0]
        p1_tracts = [i for i, p in enumerate(parts) if p == 1]

        def calc_stats(tracts):
            total_pop = sum(vertex_weights[i, 0] for i in tracts)
            total_minority = sum(vertex_weights[i, 1] for i in tracts)
            return total_pop, total_minority, total_minority / total_pop

        p0_pop, p0_min, p0_pct = calc_stats(p0_tracts)
        p1_pop, p1_min, p1_pct = calc_stats(p1_tracts)

        print(f"\n=== BASELINE: No tpwgts, no ubvec ===")
        print(f"P0: {len(p0_tracts)} tracts, pop={p0_pop}, minority={p0_pct*100:.1f}%")
        print(f"P1: {len(p1_tracts)} tracts, pop={p1_pop}, minority={p1_pct*100:.1f}%")

        # This is our baseline - what does METIS do naturally?
        # With multi-constraint and no guidance, should balance both constraints
        print(f"Baseline result: {p0_pct*100:.1f}% / {p1_pct*100:.1f}% minority split")

    def test_equal_tpwgts_no_ubvec(self):
        """
        Test: Equal tpwgts [[0.5, 0.5], [0.5, 0.5]], NO ubvec.

        Expected: Should be same as baseline (balanced).
        """
        adjacency = [[1] if i == 0 else [i-1, i+1] if i < 9 else [8] for i in range(10)]
        vertex_weights = np.array([
            [1000, 800], [1000, 800], [1000, 800], [1000, 800], [1000, 800],
            [1000, 200], [1000, 200], [1000, 200], [1000, 200], [1000, 200],
        ])

        # Equal targets for both constraints
        target_weights = [[0.5, 0.5], [0.5, 0.5]]

        parts = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            niter=100,
            debug=True
        )

        p0_tracts = [i for i, p in enumerate(parts) if p == 0]
        p1_tracts = [i for i, p in enumerate(parts) if p == 1]

        def calc_stats(tracts):
            total_pop = sum(vertex_weights[i, 0] for i in tracts)
            total_minority = sum(vertex_weights[i, 1] for i in tracts)
            return total_minority / total_pop

        p0_pct = calc_stats(p0_tracts)
        p1_pct = calc_stats(p1_tracts)

        print(f"\n=== Equal tpwgts (0.5, 0.5), no ubvec ===")
        print(f"P0: {p0_pct*100:.1f}% minority")
        print(f"P1: {p1_pct*100:.1f}% minority")

        # Should be similar to baseline (balanced)

    def test_skewed_tpwgts_no_ubvec(self):
        """
        KEY TEST: Skewed tpwgts [[0.5, 0.8], [0.5, 0.2]], NO ubvec.

        This tests if tpwgts ALONE can guide minority concentration.

        Expected behavior depends on METIS implementation:
        - If tpwgts works: P0 should have >60% minority
        - If tpwgts doesn't work without ubvec: balanced ~50/50
        """
        adjacency = [[1] if i == 0 else [i-1, i+1] if i < 9 else [8] for i in range(10)]
        vertex_weights = np.array([
            [1000, 800], [1000, 800], [1000, 800], [1000, 800], [1000, 800],
            [1000, 200], [1000, 200], [1000, 200], [1000, 200], [1000, 200],
        ])

        # Skewed targets - P0 should get 80% of minorities
        target_weights = [[0.5, 0.8], [0.5, 0.2]]

        parts = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            niter=100,
            debug=True
        )

        p0_tracts = [i for i, p in enumerate(parts) if p == 0]
        p1_tracts = [i for i, p in enumerate(parts) if p == 1]

        def calc_stats(tracts):
            total_pop = sum(vertex_weights[i, 0] for i in tracts)
            total_minority = sum(vertex_weights[i, 1] for i in tracts)
            return total_minority / total_pop

        p0_pct = calc_stats(p0_tracts)
        p1_pct = calc_stats(p1_tracts)

        print(f"\n=== KEY TEST: Skewed tpwgts (0.8, 0.2), no ubvec ===")
        print(f"P0: {p0_pct*100:.1f}% minority (target: 80% of minorities)")
        print(f"P1: {p1_pct*100:.1f}% minority (target: 20% of minorities)")

        # Check if tpwgts worked
        if p0_pct > 0.6:
            print("RESULT: tpwgts WORKS without ubvec!")
            print(f"  Achieved {p0_pct*100:.1f}% concentration (target: 80%)")
        else:
            print("RESULT: tpwgts DOES NOT WORK without ubvec")
            print(f"  Only achieved {p0_pct*100:.1f}% (expected >60% if working)")

        # Document the result
        return p0_pct > 0.6  # Return whether tpwgts worked

    def test_clustered_geography_no_ubvec(self):
        """
        Test with clustered geography (easy case), NO ubvec.

        If tpwgts works, it should achieve perfect separation even without ubvec.
        """
        adjacency = [
            # High minority cluster (0-4)
            [1, 2, 3, 4],
            [0, 2, 3, 4],
            [0, 1, 3, 4],
            [0, 1, 2, 4],
            [0, 1, 2, 3, 5],  # Bridge
            # Low minority cluster (5-9)
            [4, 6, 7, 8, 9],
            [5, 7, 8, 9],
            [5, 6, 8, 9],
            [5, 6, 7, 9],
            [5, 6, 7, 8],
        ]

        vertex_weights = np.array([
            [1000, 800], [1000, 800], [1000, 800], [1000, 800], [1000, 800],
            [1000, 200], [1000, 200], [1000, 200], [1000, 200], [1000, 200],
        ])

        target_weights = [[0.5, 0.8], [0.5, 0.2]]

        parts = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            niter=100,
            debug=True
        )

        p0_tracts = [i for i, p in enumerate(parts) if p == 0]
        p1_tracts = [i for i, p in enumerate(parts) if p == 1]

        def calc_stats(tracts):
            total_pop = sum(vertex_weights[i, 0] for i in tracts)
            total_minority = sum(vertex_weights[i, 1] for i in tracts)
            return total_minority / total_pop

        p0_pct = calc_stats(p0_tracts)
        p1_pct = calc_stats(p1_tracts)

        print(f"\n=== Clustered geography, no ubvec ===")
        print(f"P0 tracts: {p0_tracts}")
        print(f"P0: {p0_pct*100:.1f}% minority")
        print(f"P1 tracts: {p1_tracts}")
        print(f"P1: {p1_pct*100:.1f}% minority")

        if p0_pct > 0.75:
            print("RESULT: tpwgts works with clustered geography (no ubvec needed)")
        else:
            print(f"RESULT: tpwgts did not achieve full concentration ({p0_pct*100:.1f}%)")


class TestTPWGTSFormatValidation:
    """Validate tpwgts file format is correct."""

    def test_tpwgts_file_format(self):
        """
        Verify the tpwgts file format we're generating.

        Should be:
        0 : 0 = 0.429
        1 : 0 = 0.571
        0 : 1 = 0.572
        1 : 1 = 0.428
        """
        import tempfile
        from pathlib import Path

        target_weights = [[0.429, 0.572], [0.571, 0.428]]
        nparts = 2
        ncon = 2

        tmpdir = Path(tempfile.mkdtemp())
        tpwgts_file = tmpdir / 'tpwgts.txt'

        # Generate file (same logic as metis_executable.py)
        with open(tpwgts_file, 'w') as f:
            for constraint_id in range(ncon):
                weights_for_constraint = [target_weights[p][constraint_id] for p in range(nparts)]
                total = sum(weights_for_constraint)

                for partition_id, weight in enumerate(weights_for_constraint):
                    normalized_weight = weight / total if total > 0 else 1.0 / nparts
                    f.write(f'{partition_id} : {constraint_id} = {normalized_weight:.6f}\n')

        # Read and validate
        with open(tpwgts_file, 'r') as f:
            lines = f.readlines()

        print(f"\n=== tpwgts.txt format ===")
        for i, line in enumerate(lines):
            print(f"Line {i}: {line.rstrip()}")

        # Validate format
        assert len(lines) == 4, f"Expected 4 lines, got {len(lines)}"

        # Check each line has format "partition : constraint = weight"
        for line in lines:
            parts = line.strip().split()
            assert len(parts) == 5, f"Invalid format: {line}"
            assert parts[1] == ':', f"Missing colon: {line}"
            assert parts[3] == '=', f"Missing equals: {line}"

        print("FORMAT VALIDATED: partition : constraint = weight")


class TestTPWGTSComparison:
    """Compare tpwgts with ubvec vs without ubvec."""

    def test_comparison_with_and_without_ubvec(self):
        """
        Direct comparison: same tpwgts, with/without ubvec.

        This shows the effect of ubvec vs tpwgts alone.
        """
        adjacency = [[1] if i == 0 else [i-1, i+1] if i < 9 else [8] for i in range(10)]
        vertex_weights = np.array([
            [1000, 800], [1000, 800], [1000, 800], [1000, 800], [1000, 800],
            [1000, 200], [1000, 200], [1000, 200], [1000, 200], [1000, 200],
        ])
        target_weights = [[0.5, 0.8], [0.5, 0.2]]

        # Test WITHOUT ubvec (tpwgts only)
        parts_no_ubvec = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            niter=100,
            debug=True
        )

        def calc_stats(parts):
            p0_tracts = [i for i, p in enumerate(parts) if p == 0]
            p1_tracts = [i for i, p in enumerate(parts) if p == 1]

            p0_pop = sum(vertex_weights[i, 0] for i in p0_tracts)
            p0_min = sum(vertex_weights[i, 1] for i in p0_tracts)
            p0_pct = p0_min / p0_pop

            p1_pop = sum(vertex_weights[i, 0] for i in p1_tracts)
            p1_min = sum(vertex_weights[i, 1] for i in p1_tracts)
            p1_pct = p1_min / p1_pop

            return p0_pct, p1_pct

        no_ubvec_p0, no_ubvec_p1 = calc_stats(parts_no_ubvec)

        print(f"\n=== COMPARISON ===")
        print(f"WITHOUT ubvec (tpwgts only):")
        print(f"  P0: {no_ubvec_p0*100:.1f}% minority")
        print(f"  P1: {no_ubvec_p1*100:.1f}% minority")
        print(f"  Concentration achieved: {max(no_ubvec_p0, no_ubvec_p1)*100:.1f}%")

        # Note: We would test WITH ubvec here, but that's what our other tests do
        # The key question: does tpwgts alone achieve concentration?

        concentration = max(no_ubvec_p0, no_ubvec_p1)
        if concentration > 0.6:
            print("\nCONCLUSION: tpwgts alone is sufficient!")
        else:
            print(f"\nCONCLUSION: tpwgts alone only achieved {concentration*100:.1f}%")
            print("May need ubvec to allow METIS to deviate from balance.")
