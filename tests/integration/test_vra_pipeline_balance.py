"""
Integration test: VRA pipeline must maintain constitutional population balance.

Runs run_state_redistricting.py end-to-end in VRA mode on a small state
(Vermont = 1 district, skips to edge-weighted) and Alabama (7 districts,
known VRA target) and verifies ±0.5% population balance is maintained.

Requires pipeline adjacency data to exist (outputs/V3/data/2020/adjacency/).
"""

import csv
import pickle
import subprocess
import sys
import tempfile
import pytest
from pathlib import Path


ADJACENCY_DIR = Path('outputs/V3/data/2020/adjacency')
SCRIPT = Path('scripts/pipeline/run_state_redistricting.py')


def adjacency_exists(state_code: str) -> bool:
    f = ADJACENCY_DIR / f'{state_code.lower()}_adjacency_2020.pkl'
    return f.exists()


def run_redistricting(state_code: str, tmp_dir: Path) -> tuple[dict, dict]:
    """
    Run run_state_redistricting.py in VRA mode.
    Returns (assignments dict, vertex_weights array) for balance checking.
    """
    state_names = {
        'AL': 'alabama', 'GA': 'georgia', 'MS': 'mississippi',
        'VT': 'vermont', 'CA': 'california',
    }
    state_name = state_names.get(state_code, state_code.lower())
    output_dir = tmp_dir / state_name

    result = subprocess.run([
        sys.executable, str(SCRIPT),
        '--state', state_code,
        '--year', '2020',
        '--version', 'V3',            # use V3 adjacency/tract data
        '--partition-mode', 'metis-vra',
        '--output-dir', str(output_dir),
        '--position', '999',
    ], capture_output=True, text=True, timeout=300)

    if result.returncode != 0:
        pytest.fail(
            f'{state_code} redistricting failed (rc={result.returncode}):\n'
            f'STDOUT: {result.stdout[-500:]}\n'
            f'STDERR: {result.stderr[-500:]}'
        )

    pkl = output_dir / 'data' / 'final_assignments.pkl'
    if not pkl.exists():
        pytest.fail(f'{state_code}: final_assignments.pkl not created')

    with open(pkl, 'rb') as f:
        assignments = pickle.load(f)

    # Load vertex weights to compute population per district
    adj_pkl = ADJACENCY_DIR / f'{state_code.lower()}_adjacency_2020.pkl'
    with open(adj_pkl, 'rb') as f:
        graph = pickle.load(f)
    vertex_weights = graph['vertex_weights']

    return assignments, vertex_weights


@pytest.mark.skipif(not ADJACENCY_DIR.exists(), reason='Pipeline adjacency data not found')
class TestVRAPopulationBalance:

    @pytest.fixture(scope='class')
    def tmp(self, tmp_path_factory):
        return tmp_path_factory.mktemp('vra_test')

    def _check_balance(self, assignments, vertex_weights, num_districts,
                        state: str, tolerance: float = 0.005):
        """Compute district populations from assignments and check balance."""
        if num_districts <= 1:
            return  # single-district state — trivially balanced

        total = int(vertex_weights.sum())
        ideal = total / num_districts
        dist_pop = {}
        for tract_idx, dist_id in assignments.items():
            dist_pop[dist_id] = dist_pop.get(dist_id, 0) + vertex_weights[tract_idx]

        actual_districts = len(dist_pop)
        assert actual_districts == num_districts, (
            f'{state}: expected {num_districts} districts, got {actual_districts}'
        )

        violations = []
        for dist_id, pop in sorted(dist_pop.items()):
            dev = abs(pop - ideal) / ideal
            if dev > tolerance:
                violations.append(
                    f'District {dist_id}: pop={pop:.0f}, ideal={ideal:.0f}, '
                    f'dev={dev*100:.2f}%'
                )
        assert not violations, (
            f'{state} VRA redistricting violates ±{tolerance*100:.1f}% '
            f'constitutional population balance:\n' + '\n'.join(violations)
        )

    @pytest.mark.skipif(not adjacency_exists('AL'), reason='Alabama adjacency not found')
    def test_alabama_vra_population_balance(self, tmp):
        """Alabama VRA: 7 districts must stay within ±0.5% of ideal."""
        assignments, vw = run_redistricting('AL', tmp)
        self._check_balance(assignments, vw, 7, 'Alabama')

    @pytest.mark.skipif(not adjacency_exists('MS'), reason='Mississippi adjacency not found')
    def test_mississippi_vra_population_balance(self, tmp):
        """Mississippi VRA: 4 districts must stay within ±0.5%."""
        assignments, vw = run_redistricting('MS', tmp)
        self._check_balance(assignments, vw, 4, 'Mississippi')

    @pytest.mark.skipif(not adjacency_exists('GA'), reason='Georgia adjacency not found')
    def test_georgia_vra_population_balance(self, tmp):
        """Georgia VRA: 14 districts must stay within ±0.5%."""
        assignments, vw = run_redistricting('GA', tmp)
        self._check_balance(assignments, vw, 14, 'Georgia')

    @pytest.mark.skipif(not adjacency_exists('CA'), reason='California adjacency not found')
    def test_california_vra_population_balance(self, tmp):
        """California VRA: 52 districts, very high minority (63.5%) — must still balance."""
        assignments, vw = run_redistricting('CA', tmp)
        self._check_balance(assignments, vw, 52, 'California')

    @pytest.mark.skipif(not adjacency_exists('TX'), reason='Texas adjacency not found')
    def test_texas_vra_population_balance(self, tmp):
        """Texas VRA: 38 districts, high minority — must still balance."""
        assignments, vw = run_redistricting('TX', tmp)
        self._check_balance(assignments, vw, 38, 'Texas')


@pytest.mark.skipif(not SCRIPT.exists(), reason='Pipeline script not found')
class TestVRACodePathIntegrity:
    """Verify VRA mode does not corrupt vertex weights or leave bad state."""

    def test_vra_does_not_use_multidimensional_vertex_weights(self):
        """
        When run_state_redistricting runs in metis-vra mode, the final
        assignments file must exist and be a valid dict (not corrupted by
        multi-constraint mode accidentally being enabled).
        """
        if not adjacency_exists('VT'):
            pytest.skip('Vermont adjacency not found')

        with tempfile.TemporaryDirectory() as tmp:
            output_dir = Path(tmp) / 'vermont'
            result = subprocess.run([
                sys.executable, str(SCRIPT),
                '--state', 'VT',
                '--year', '2020',
                '--version', 'V3',
                '--partition-mode', 'metis-vra',
                '--output-dir', str(output_dir),
                '--position', '999',
            ], capture_output=True, text=True, timeout=60)

            # Must not crash (the old bug: IndexError from 2D vertex weight access)
            assert result.returncode == 0, (
                f'VRA redistricting crashed (may indicate multi-constraint bug):\n'
                f'{result.stderr[-500:]}'
            )

            pkl = output_dir / 'data' / 'final_assignments.pkl'
            assert pkl.exists(), 'final_assignments.pkl not created'

            import pickle
            with open(pkl, 'rb') as f:
                assignments = pickle.load(f)
            assert isinstance(assignments, dict), 'assignments should be a dict'
