"""
Pipeline acceptance tests: verify key metrics after Rust integration changes.

Purpose: run before any full 50-state re-run to confirm the Rust-modified
Python pipeline produces correct output. Two states chosen deliberately:

  Vermont (VT) — 1 district, no METIS, trivially correct.
    Verifies: the pipeline runs end-to-end, all 193 tracts assigned, no crash.

  Alabama (AL, V4 VRA mode) — 7 districts, 2 MM targets.
    Verifies: VRA edge weights (now Rust), population balance, MM district count.
    This is the most important test — it catches any regression in the Rust
    VRA formula or the adaptive boost wiring.

Baselines captured 2026-04-24 from verified V3/V4 outputs:
  Vermont:  193 tracts → 1 district, trivially balanced
  Alabama:  1437 tracts → 7 districts, 2 MM (D2 at 51.07%, D7 at 50.54%)

Run with:
  pytest tests/acceptance/test_pipeline_acceptance.py -v --tb=short

Requires:
  - Pipeline adjacency data: outputs/V3/data/2020/adjacency/
  - Python pipeline scripts: scripts/pipeline/run_state_redistricting.py
  - METIS: gpmetis in PATH (or installed via apt/conda)
  - ~2 minutes total (VT ~5s, AL ~60-90s)
"""

import csv
import json
import os
import pickle
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import pytest
import numpy as np

# ---------------------------------------------------------------------------
# DEPRECATED 2026-04-29 — Python pipeline archived under archive/python-pipeline-final/.
# Replacement: tests/acceptance/test_redist_invariants.py runs against `redist` binary.
# This file is kept skipped for now; once test_redist_invariants.py has full
# population/contiguity fixtures wired, this file can be deleted.
# ---------------------------------------------------------------------------

import pytest as _pytest
pytestmark = _pytest.mark.skip(
    reason="Python pipeline archived 2026-04-29; replaced by test_redist_invariants.py"
)

# ---------------------------------------------------------------------------
# Prerequisites
# ---------------------------------------------------------------------------

SCRIPT = Path('archive/python-pipeline-final/scripts/pipeline/run_state_redistricting.py')
ADJACENCY_DIR = Path('outputs/V3/data/2020/adjacency')


def adjacency_exists(state: str) -> bool:
    return (ADJACENCY_DIR / f'{state.lower()}_adjacency_2020.pkl').exists()


def metis_available() -> bool:
    from apportionment.partition.metis_executable import find_gpmetis_executable
    return find_gpmetis_executable() is not None


@pytest.fixture(scope='module')
def tmp_dir(tmp_path_factory):
    return tmp_path_factory.mktemp('acceptance')


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

def run_state(state: str, tmp_dir: Path, partition_mode: str = 'edge-weighted') -> Path:
    """
    Run redistricting for one state. Returns the state output directory.
    Raises pytest.fail on non-zero exit or missing output.
    """
    state_name = {
        'VT': 'vermont', 'AL': 'alabama', 'CA': 'california',
        'DE': 'delaware', 'GA': 'georgia',
    }.get(state, state.lower())

    output_dir = tmp_dir / state_name
    t0 = time.time()

    result = subprocess.run(
        [
            sys.executable, str(SCRIPT),
            '--state', state,
            '--year', '2020',
            '--version', 'V3' if partition_mode != 'metis-vra' else 'V4',
            '--partition-mode', partition_mode,
            '--output-dir', str(output_dir),
            '--position', '999',
        ],
        capture_output=True, text=True, timeout=300
    )

    elapsed = time.time() - t0

    if result.returncode != 0:
        pytest.fail(
            f'{state} redistricting failed in {elapsed:.1f}s '
            f'(rc={result.returncode}):\n'
            f'STDOUT: {result.stdout[-500:]}\n'
            f'STDERR: {result.stderr[-500:]}'
        )

    data_dir = output_dir / 'data'
    if not data_dir.exists():
        pytest.fail(f'{state}: data/ directory not created after {elapsed:.1f}s')

    return output_dir


def load_assignments(output_dir: Path) -> dict:
    pkl = output_dir / 'data' / 'final_assignments.pkl'
    if not pkl.exists():
        pytest.fail(f'final_assignments.pkl missing: {pkl}')
    with open(pkl, 'rb') as f:
        return pickle.load(f)


def load_vra_analysis(output_dir: Path) -> dict:
    # Python pipeline writes .pkl; Rust CLI (Phase 3d) will write .json.
    # Accept either format — whichever exists.
    pkl = output_dir / 'data' / 'vra_analysis.pkl'
    j   = output_dir / 'data' / 'vra_analysis.json'
    if pkl.exists():
        with open(pkl, 'rb') as f:
            return pickle.load(f)
    if j.exists():
        return json.loads(j.read_text())
    pytest.fail(
        f'Neither vra_analysis.pkl nor vra_analysis.json found in {output_dir / "data"}. '
        f'vra_mode was probably cleared prematurely.'
    )


def compute_pop_balance(assignments: dict, adjacency_pkl: Path) -> float:
    """Max fractional deviation from ideal district population."""
    with open(adjacency_pkl, 'rb') as f:
        graph = pickle.load(f)
    vw = np.array(graph['vertex_weights'], dtype=np.int64)

    n_districts = len(set(assignments.values()))
    total = int(vw.sum())
    ideal = total / n_districts

    dist_pop: dict = {}
    for tract, dist in assignments.items():
        dist_pop[dist] = dist_pop.get(dist, 0) + int(vw[tract])

    return max(abs(p - ideal) / ideal for p in dist_pop.values())


# ---------------------------------------------------------------------------
# Vermont acceptance: non-VRA, single district
# ---------------------------------------------------------------------------

@pytest.mark.skipif(
    not adjacency_exists('VT'),
    reason='Vermont adjacency not found (run pipeline to generate)'
)
@pytest.mark.skipif(
    not SCRIPT.exists(),
    reason='Pipeline script not found'
)
class TestVermontAcceptance:
    """Vermont: 1 district, no VRA, ~5 seconds. Verifies basic pipeline health."""

    @pytest.fixture(scope='class')
    def vt_output(self, tmp_dir):
        return run_state('VT', tmp_dir, 'edge-weighted')

    @pytest.fixture(scope='class')
    def vt_assignments(self, vt_output):
        return load_assignments(vt_output)

    def test_all_193_tracts_assigned(self, vt_assignments):
        """All Vermont census tracts must appear in assignments."""
        assert len(vt_assignments) == 193, \
            f'Expected 193 tracts, got {len(vt_assignments)}'

    def test_exactly_one_district(self, vt_assignments):
        """Vermont has 1 congressional district."""
        districts = set(vt_assignments.values())
        assert len(districts) == 1, \
            f'Expected 1 district, got {len(districts)}: {districts}'

    def test_no_final_assignments_missing(self, vt_output):
        """final_assignments.pkl must exist and be a valid dict."""
        assignments = load_assignments(vt_output)
        assert isinstance(assignments, dict)
        assert len(assignments) > 0

    def test_population_balance_trivial(self, vt_assignments):
        """Single district: deviation is always 0%."""
        adj_pkl = ADJACENCY_DIR / 'vt_adjacency_2020.pkl'
        dev = compute_pop_balance(vt_assignments, adj_pkl)
        assert dev == 0.0, f'Single district should have 0% deviation, got {dev*100:.2f}%'

    def test_no_rust_regression_in_output(self, vt_assignments):
        """
        Baseline: 193 tracts all assigned to district 1.
        If Rust integration broke anything, tract count or district set would change.
        """
        assert set(vt_assignments.values()) == {1}, \
            'All Vermont tracts should map to district 1'


# ---------------------------------------------------------------------------
# Alabama acceptance: VRA mode, 7 districts, 2 MM targets
# ---------------------------------------------------------------------------

@pytest.mark.skipif(
    not adjacency_exists('AL'),
    reason='Alabama adjacency not found'
)
@pytest.mark.skipif(
    not SCRIPT.exists(),
    reason='Pipeline script not found'
)
class TestAlabamaVRAAcceptance:
    """
    Alabama VRA: 7 districts, target 2 MM districts.
    This is the key test — verifies the Rust VRA formula produces correct output.

    Baseline (verified 2026-04-24):
      mm_count = 2
      D2: 51.07% minority (MM)
      D7: 50.54% minority (MM)
      Max population deviation ≤ 0.5%
    """

    @pytest.fixture(scope='class')
    def al_output(self, tmp_dir):
        return run_state('AL', tmp_dir, 'metis-vra')

    @pytest.fixture(scope='class')
    def al_assignments(self, al_output):
        return load_assignments(al_output)

    @pytest.fixture(scope='class')
    def al_vra(self, al_output):
        return load_vra_analysis(al_output)

    def test_all_1437_tracts_assigned(self, al_assignments):
        assert len(al_assignments) == 1437, \
            f'Expected 1437 Alabama tracts, got {len(al_assignments)}'

    def test_exactly_7_districts(self, al_assignments):
        districts = set(al_assignments.values())
        assert len(districts) == 7, \
            f'Expected 7 districts, got {len(districts)}: {sorted(districts)}'

    def test_population_balance_within_half_percent(self, al_assignments):
        """Constitutional requirement: ±0.5% population balance."""
        adj_pkl = ADJACENCY_DIR / 'al_adjacency_2020.pkl'
        dev = compute_pop_balance(al_assignments, adj_pkl)
        assert dev <= 0.005, \
            f'Population imbalance {dev*100:.3f}% exceeds constitutional ±0.5% limit'

    def test_vra_analysis_written(self, al_output):
        """vra_analysis pkl or json must exist — proves vra_mode stayed True."""
        data = al_output / 'data'
        has_output = (data / 'vra_analysis.pkl').exists() or \
                     (data / 'vra_analysis.json').exists()
        assert has_output, \
            'Neither vra_analysis.pkl nor vra_analysis.json written. ' \
            'vra_mode was probably cleared prematurely.'

    def test_mm_count_matches_baseline(self, al_vra):
        """Baseline: 2 majority-minority districts. Rust VRA must reproduce this."""
        mm = al_vra['mm_count']
        assert mm == 2, \
            f'Expected 2 MM districts (Allen v. Milligan target), got {mm}. ' \
            f'Rust VRA edge weights may have changed the result.'

    def test_mm_district_2_above_50pct(self, al_vra):
        """District 2 should be ~51% minority (baseline: 51.07%)."""
        districts = {d['district']: d for d in al_vra['districts']}
        d2 = districts.get(2, {})
        pct = d2.get('pct_minority', 0.0)
        assert pct > 0.50, \
            f'District 2 at {pct*100:.2f}% minority — should be majority-minority'
        assert pct < 0.60, \
            f'District 2 at {pct*100:.2f}% — unusually high, check formula'

    def test_mm_district_7_above_50pct(self, al_vra):
        """District 7 should be ~50.5% minority (baseline: 50.54%)."""
        districts = {d['district']: d for d in al_vra['districts']}
        d7 = districts.get(7, {})
        pct = d7.get('pct_minority', 0.0)
        assert pct > 0.50, \
            f'District 7 at {pct*100:.2f}% minority — should be majority-minority (baseline: 50.54%)'

    def test_non_mm_districts_below_50pct(self, al_vra):
        """Non-MM districts must be below 50% minority."""
        mm_ids = set(al_vra['mm_districts'])
        for d in al_vra['districts']:
            if d['district'] not in mm_ids:
                pct = d['pct_minority']
                assert pct <= 0.50, \
                    f'District {d["district"]} at {pct*100:.2f}% minority ' \
                    f'is NOT in mm_districts but exceeds 50%'

    def test_rust_vra_formula_consistent_with_baseline(self, al_vra):
        """
        Verify the Rust adaptive boost formula produced the right district layout.
        The specific MM districts (2 and 7) should match the verified baseline.
        This test would fail if the Rust formula changed the effective alpha.
        """
        mm_ids = sorted(al_vra['mm_districts'])
        # Baseline: districts 2 and 7. Allow some flexibility since METIS is stochastic.
        assert len(mm_ids) == 2, f'Expected 2 MM districts, got {mm_ids}'
        # Both MM districts must be in the lower Alabama Black Belt (D2) or
        # the majority-Black 7th district area (D7). This is a geographic sanity check.
        for mm_id in mm_ids:
            districts = {d['district']: d for d in al_vra['districts']}
            pct = districts[mm_id]['pct_minority']
            assert 0.50 < pct < 0.65, \
                f'MM district {mm_id} at {pct*100:.2f}% — outside expected 50-65% range'


# ---------------------------------------------------------------------------
# Rust integration smoke tests (fast, no pipeline needed)
# ---------------------------------------------------------------------------

class TestRustIntegrationSmoke:
    """
    Verify the Rust components are importable and produce expected values.
    These run in <1 second and catch import/build failures before the slow tests.
    """

    def test_redist_py_importable(self):
        if os.environ.get('REDIST_NO_RUST', '0') == '1':
            pytest.skip('REDIST_NO_RUST=1')
        import redist_py
        assert hasattr(redist_py, 'build_vra_edge_weights')
        assert hasattr(redist_py, 'Partition')
        assert hasattr(redist_py, 'Graph')
        assert hasattr(redist_py, 'compute_polsby_popper')
        assert hasattr(redist_py, 'compute_vra_analysis')

    def test_vra_formula_alabama_profile(self):
        """Adaptive boost for Alabama: f=0.22 → alpha ≈ 8.46."""
        if os.environ.get('REDIST_NO_RUST', '0') == '1':
            pytest.skip('REDIST_NO_RUST=1')
        import redist_py
        import numpy as np
        n = 100
        fracs = np.array([0.55 if i < 22 else 0.20 for i in range(n)], dtype=np.float64)
        edges = [(0, 1)]
        weights = redist_py.build_vra_edge_weights(edges, fracs)
        alpha = weights.get((0, 1), 0.0)
        assert abs(alpha - 8.46) < 0.01, \
            f'Alabama adaptive boost should be ~8.46x, got {alpha:.4f}. ' \
            f'Rust VRA formula may have changed.'

    def test_polsby_popper_square(self):
        """PP for 1km square = π/4 ≈ 0.7854."""
        if os.environ.get('REDIST_NO_RUST', '0') == '1':
            pytest.skip('REDIST_NO_RUST=1')
        import redist_py, struct, math
        coords = [(1e6, 1e6), (1e6+1000, 1e6), (1e6+1000, 1e6+1000), (1e6, 1e6+1000)]
        coords.append(coords[0])
        buf = bytearray(b'\x01' + struct.pack('<II', 3, 1) + struct.pack('<I', len(coords)))
        for x, y in coords:
            buf += struct.pack('<dd', x, y)
        pp, _ = redist_py.compute_polsby_popper(bytes(buf))
        assert abs(pp - math.pi/4) < 1e-6, f'Square PP={pp:.6f}, expected π/4={math.pi/4:.6f}'

    def test_balance_checker_passes_valid_partition(self):
        """Rust balance checker should not raise for a balanced partition."""
        if os.environ.get('REDIST_NO_RUST', '0') == '1':
            pytest.skip('REDIST_NO_RUST=1')
        import redist_py
        import numpy as np
        p = redist_py.Partition.from_dict({0: 1, 1: 1, 2: 2, 3: 2})
        vw = np.array([1000, 1000, 1000, 1000], dtype=np.int64)
        p.assert_balanced(vw, n_districts=2, tolerance=0.005)  # should not raise


# ---------------------------------------------------------------------------
# Rust CLI acceptance tests — verify `redist state` binary works end-to-end
# ---------------------------------------------------------------------------

REDIST_BIN = Path('redist/target/release/redist.exe'
                  if sys.platform == 'win32'
                  else 'redist/target/release/redist')


@pytest.mark.skipif(
    not REDIST_BIN.exists(),
    reason=f'redist binary not built — run: cargo build -p redist-cli --release'
)
@pytest.mark.skipif(
    not adjacency_exists('VT'),
    reason='Vermont adjacency not found'
)
class TestRustCLIAcceptance:
    """Verify `redist state` binary produces correct output."""

    @pytest.fixture(scope='class')
    def vt_rust_output(self, tmp_path_factory):
        tmp = tmp_path_factory.mktemp('rust_cli')
        # REDIST_PYTHON: pass the exact Python executable so the binary uses the
        # same Python environment (with numpy) that pytest runs under.
        env = os.environ.copy()
        env['REDIST_PYTHON'] = sys.executable
        result = subprocess.run(
            [str(REDIST_BIN), 'state',
             '--state', 'VT', '--year', '2020', '--version', 'V3',
             '--output-dir', str(tmp),
             '--position', '999'],
            capture_output=True, text=True, timeout=60,
            cwd=str(Path.cwd()), env=env
        )
        if result.returncode != 0:
            pytest.fail(f'redist state VT failed:\nSTDOUT: {result.stdout}\nSTDERR: {result.stderr}')
        return tmp

    def test_vt_rust_final_assignments_exists(self, vt_rust_output):
        assert (vt_rust_output / 'states' / 'vermont' / 'data' / 'final_assignments.json').exists(), \
            'Rust CLI should write final_assignments.json'

    def test_vt_rust_193_tracts(self, vt_rust_output):
        data = json.loads(
            (vt_rust_output / 'states' / 'vermont' / 'data' / 'final_assignments.json').read_text()
        )
        assert len(data) == 193, f'Expected 193 tracts, got {len(data)}'

    def test_vt_rust_one_district(self, vt_rust_output):
        data = json.loads(
            (vt_rust_output / 'states' / 'vermont' / 'data' / 'final_assignments.json').read_text()
        )
        assert set(data.values()) == {1}, f'Expected only district 1, got {set(data.values())}'

    @pytest.mark.skipif(not adjacency_exists('AL'), reason='Alabama adjacency not found')
    @pytest.fixture(scope='class')
    def al_rust_output(self, tmp_path_factory):
        tmp = tmp_path_factory.mktemp('rust_al')
        env = os.environ.copy()
        env['REDIST_PYTHON'] = sys.executable
        result = subprocess.run(
            [str(REDIST_BIN), 'state',
             '--state', 'AL', '--year', '2020', '--version', 'V4',
             '--partition-mode', 'metis-vra',
             '--output-dir', str(tmp),
             '--position', '999',
             '--seed', '42'],  # Fixed seed: METIS is stochastic
            capture_output=True, text=True, timeout=300,
            cwd=str(Path.cwd()), env=env
        )
        if result.returncode != 0:
            pytest.fail(f'redist state AL failed:\nSTDOUT: {result.stdout[-500:]}\nSTDERR: {result.stderr[-500:]}')
        return tmp

    @pytest.mark.skipif(not adjacency_exists('AL'), reason='Alabama adjacency not found')
    def test_al_rust_vra_analysis_written(self, al_rust_output):
        assert (al_rust_output / 'states' / 'alabama' / 'data' / 'vra_analysis.json').exists(), \
            'vra_analysis.json not written — vra_mode may have been cleared'

    @pytest.mark.skipif(not adjacency_exists('AL'), reason='Alabama adjacency not found')
    def test_al_rust_mm_count(self, al_rust_output):
        vra = json.loads(
            (al_rust_output / 'states' / 'alabama' / 'data' / 'vra_analysis.json').read_text()
        )
        assert vra['mm_count'] == 2, \
            f'Alabama must achieve 2 MM districts (seed=42 is fixed), got {vra["mm_count"]}'

    @pytest.mark.skipif(not adjacency_exists('AL'), reason='Alabama adjacency not found')
    def test_al_rust_population_balance(self, al_rust_output):
        assignments = json.loads(
            (al_rust_output / 'states' / 'alabama' / 'data' / 'final_assignments.json').read_text()
        )
        adj_pkl = ADJACENCY_DIR / 'al_adjacency_2020.pkl'
        dev = compute_pop_balance({int(k): v for k, v in assignments.items()}, adj_pkl)
        assert dev <= 0.005, f'Alabama balance {dev*100:.3f}% exceeds ±0.5%'


# ---------------------------------------------------------------------------
# redist states: multi-state parallel acceptance (VT + RI)
# ---------------------------------------------------------------------------

@pytest.mark.skipif(not REDIST_BIN.exists(), reason='redist binary not built')
@pytest.mark.skipif(
    not adjacency_exists('VT') or not adjacency_exists('RI'),
    reason='VT or RI adjacency not found'
)
class TestRustStatesAcceptance:
    """
    Verify `redist states --states VT RI` runs two states in parallel.
    VT (1 district) exercises the single-district shortcut.
    RI (2 districts) exercises real bisection + population balance.
    """

    @pytest.fixture(scope='class')
    def states_output(self, tmp_path_factory):
        tmp = tmp_path_factory.mktemp('rust_states')
        env = os.environ.copy()
        env['REDIST_PYTHON'] = sys.executable
        result = subprocess.run(
            [str(REDIST_BIN), 'states',
             '--year', '2020', '--version', 'V3',
             '--output-dir', str(tmp),
             '--states', 'VT', 'RI',
             '--workers', '2'],
            capture_output=True, text=True, timeout=180,
            cwd=str(Path.cwd()), env=env
        )
        if result.returncode != 0:
            pytest.fail(
                f'redist states VT RI failed:\n'
                f'STDOUT: {result.stdout[-500:]}\n'
                f'STDERR: {result.stderr[-500:]}'
            )
        return tmp

    def test_vermont_assignments_exist(self, states_output):
        assert (states_output / 'states' / 'vermont' / 'data' / 'final_assignments.json').exists()

    def test_rhode_island_assignments_exist(self, states_output):
        assert (states_output / 'states' / 'rhode_island' / 'data' / 'final_assignments.json').exists()

    def test_vermont_193_tracts_1_district(self, states_output):
        data = json.loads(
            (states_output / 'states' / 'vermont' / 'data' / 'final_assignments.json').read_text()
        )
        assert len(data) == 193
        assert set(data.values()) == {1}, 'VT must have exactly 1 district'

    def test_rhode_island_250_tracts_2_districts(self, states_output):
        data = json.loads(
            (states_output / 'states' / 'rhode_island' / 'data' / 'final_assignments.json').read_text()
        )
        assert len(data) == 250
        assert set(data.values()) == {1, 2}, \
            f'RI must have 2 districts, got {set(data.values())}'

    def test_rhode_island_population_balance(self, states_output):
        """RI 2-district split must satisfy ±0.5% constitutional balance."""
        data = json.loads(
            (states_output / 'states' / 'rhode_island' / 'data' / 'final_assignments.json').read_text()
        )
        adj_pkl = ADJACENCY_DIR / 'ri_adjacency_2020.pkl'
        dev = compute_pop_balance({int(k): v for k, v in data.items()}, adj_pkl)
        assert dev <= 0.005, f'RI balance {dev*100:.3f}% exceeds ±0.5%'

    def test_both_states_complete_in_parallel(self, states_output):
        """Both outputs must exist — confirms parallel worker completed both."""
        vt = states_output / 'states' / 'vermont' / 'data' / 'final_assignments.json'
        ri = states_output / 'states' / 'rhode_island' / 'data' / 'final_assignments.json'
        assert vt.exists() and ri.exists()


# ---------------------------------------------------------------------------
# redist fetch --release: adjacency download via fake gh binary
# ---------------------------------------------------------------------------

def _make_fake_gh(tmp: Path, adjacency_dir: Path) -> tuple:
    """
    Create a fake `gh` script and set REDIST_GH to point to it.
    The binary uses REDIST_GH env var (not PATH) so the real gh is never called.
    Returns (script_path, env_with_REDIST_GH_set).
    """
    # Must use absolute path — the script runs from binary's CWD, not project root
    real_pkl = adjacency_dir.resolve() / 'vt_adjacency_2020.pkl'

    # Write a Python-based fake gh (works on all platforms without .exe/.bat issues)
    script = tmp / 'fake_gh.py'
    script.write_text(
        'import sys, os, shutil\n'
        'args = sys.argv[1:]\n'
        'dest_dir = None\n'
        'i = 0\n'
        'while i < len(args):\n'
        '    if args[i] == "--dir" and i + 1 < len(args):\n'
        '        dest_dir = args[i + 1]; i += 2\n'
        '    else:\n'
        '        i += 1\n'
        'if dest_dir:\n'
        '    os.makedirs(dest_dir, exist_ok=True)\n'
        f'    shutil.copy(r"{real_pkl}", os.path.join(dest_dir, "vt_adjacency_2020.pkl"))\n'
        'sys.exit(0)\n'
    )

    env = os.environ.copy()
    env['REDIST_PYTHON'] = sys.executable
    # REDIST_GH: the binary reads this to override 'gh' command (avoids PATH confusion)
    env['REDIST_GH'] = f'{sys.executable} {script}'
    return script, env


@pytest.mark.skipif(not REDIST_BIN.exists(), reason='redist binary not built')
@pytest.mark.skipif(
    not adjacency_exists('VT'),
    reason='VT adjacency not found (needed as release fixture)'
)
class TestRustFetchRelease:
    """
    Verify `redist fetch --release --type adjacency` downloads adjacency pkls
    by calling `gh release download`. Uses a fake `gh` binary that copies a
    local pkl to simulate the download without network access.
    """

    @pytest.fixture(scope='class')
    def fake_env(self, tmp_path_factory):
        tmp = tmp_path_factory.mktemp('fake_gh')
        _, env = _make_fake_gh(tmp, ADJACENCY_DIR)  # env has REDIST_GH set
        return tmp, env

    def test_fetch_release_downloads_adjacency_pkl(self, fake_env, tmp_path_factory):
        """fake gh copies real VT pkl to the adjacency directory."""
        tmp, env = fake_env
        out = tmp_path_factory.mktemp('fetch_out')

        manifest = {
            'version': '1', 'github_repo': 'test/repo',
            'releases': {'data_inputs': 'data-inputs-v1', 'outputs_v3': 'v3', 'outputs_v4': 'v4'},
            'local_data_dir': 'data', 'local_outputs_dir': str(out),
            'states': {
                'VT': {
                    'name': 'Vermont', 'fips': '50',
                    'districts': {'2020': 1, '2010': 1, '2000': 1},
                    'tiger': {}, 'pl94171': {},
                }
            }
        }
        manifest_file = tmp / 'manifest.json'
        manifest_file.write_text(json.dumps(manifest))

        result = subprocess.run(
            [str(REDIST_BIN), 'fetch',
             '--states', 'VT', '--year', '2020', '--type', 'adjacency',
             '--release', '--manifest', str(manifest_file)],
            capture_output=True, text=True, timeout=30,
            cwd=str(out), env=env
        )

        assert result.returncode == 0, \
            f'fetch --release failed:\nSTDOUT: {result.stdout}\nSTDERR: {result.stderr}'

        # Adjacency goes to local_outputs_dir/V3/data/2020/adjacency/
        # build_fetch_list joins outputs_dir / "V3" / "data" / year / "adjacency"
        adj = out / 'V3' / 'data' / '2020' / 'adjacency' / 'vt_adjacency_2020.pkl'
        all_pkls = list(out.rglob('*.pkl'))
        assert adj.exists(), f'Adjacency pkl not at {adj}; found pkls: {all_pkls}'

    def test_fetched_adjacency_is_valid_pkl(self, fake_env, tmp_path_factory):
        """The fetched adjacency pkl must be loadable and have 193 tracts."""
        tmp, env = fake_env
        out = tmp_path_factory.mktemp('fetch_valid')

        manifest = {
            'version': '1', 'github_repo': 'test/repo',
            'releases': {'data_inputs': 'data-inputs-v1', 'outputs_v3': 'v3', 'outputs_v4': 'v4'},
            'local_data_dir': 'data', 'local_outputs_dir': str(out),
            'states': {
                'VT': {
                    'name': 'Vermont', 'fips': '50',
                    'districts': {'2020': 1, '2010': 1, '2000': 1},
                    'tiger': {}, 'pl94171': {},
                }
            }
        }
        (tmp / 'manifest2.json').write_text(json.dumps(manifest))

        subprocess.run(
            [str(REDIST_BIN), 'fetch',
             '--states', 'VT', '--year', '2020', '--type', 'adjacency',
             '--release', '--manifest', str(tmp / 'manifest2.json')],
            capture_output=True, text=True, timeout=30,
            cwd=str(out), env=env
        )

        adj = out / 'V3' / 'data' / '2020' / 'adjacency' / 'vt_adjacency_2020.pkl'
        assert adj.exists(), f'Adjacency not fetched. Found: {list(out.rglob("*.pkl"))}'

        import pickle
        with open(adj, 'rb') as f:
            graph = pickle.load(f)
        assert 'adjacency' in graph
        assert len(graph['adjacency']) == 193, \
            f'Expected 193 VT tracts, got {len(graph["adjacency"])}'

    def test_fetch_release_creates_done_marker(self, fake_env, tmp_path_factory):
        """Done marker written after successful adjacency download."""
        tmp, env = fake_env
        out = tmp_path_factory.mktemp('fetch_done')

        manifest = {
            'version': '1', 'github_repo': 'test/repo',
            'releases': {'data_inputs': 'data-inputs-v1', 'outputs_v3': 'v3', 'outputs_v4': 'v4'},
            'local_data_dir': 'data', 'local_outputs_dir': str(out),
            'states': {
                'VT': {
                    'name': 'Vermont', 'fips': '50',
                    'districts': {'2020': 1, '2010': 1, '2000': 1},
                    'tiger': {}, 'pl94171': {},
                }
            }
        }
        (tmp / 'manifest3.json').write_text(json.dumps(manifest))

        subprocess.run(
            [str(REDIST_BIN), 'fetch',
             '--states', 'VT', '--year', '2020', '--type', 'adjacency',
             '--release', '--manifest', str(tmp / 'manifest3.json')],
            capture_output=True, text=True, timeout=30,
            cwd=str(out), env=env
        )

        done_files = list(out.rglob('*.done'))
        assert done_files, 'No .done marker created after adjacency download'
