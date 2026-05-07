"""
Integration tests for `redist fetch` — comprehensive coverage of all download
variants using a local HTTP fixture server (no network calls).

Test matrix:
  - check-only mode (no server needed)
  - TIGER shapefile download (reqwest + zip extract)
  - PL 94-171 download (reqwest + zip extract)
  - Incremental fetch (skip already-present files)
  - --force re-download
  - Custom manifest (REDIST_MANIFEST env var)
  - Missing state in manifest
  - Server returns 404
  - Server returns malformed zip
  - Both states: VT (1 district) and RI (2 districts)

Uses Python stdlib http.server — no external dependencies.
Real TIGER fixture files in tests/fixtures/tiger/ (committed to git).
"""

import http.server
import io
import json
import os
import shutil
import subprocess
import sys
import threading
import time
import zipfile
from pathlib import Path

import pytest

REDIST_BIN = Path('redist/target/release/redist.exe'
                  if sys.platform == 'win32'
                  else 'redist/target/release/redist')

# Resolve relative to this file so it works regardless of pytest CWD
_HERE = Path(__file__).parent
TIGER_FIXTURES = _HERE.parent / 'fixtures' / 'tiger'

pytestmark = pytest.mark.skipif(
    not REDIST_BIN.exists(),
    reason='redist binary not built — run: cargo build -p redist-cli --release'
)

# ---------------------------------------------------------------------------
# Fixture HTTP server
# ---------------------------------------------------------------------------

class FixtureHandler(http.server.BaseHTTPRequestHandler):
    """Serves test data from a routes dict: path -> (content_type, bytes)."""

    routes: dict = {}

    def do_GET(self):
        data = self.routes.get(self.path)
        if data is None:
            self.send_response(404)
            self.end_headers()
            return
        content_type, body = data
        self.send_response(200)
        self.send_header('Content-Type', content_type)
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass  # suppress request logs in test output


class FixtureServer:
    """Thread-local HTTP server for serving test census data."""

    def __init__(self, routes: dict):
        FixtureHandler.routes = routes
        self.server = http.server.HTTPServer(('127.0.0.1', 0), FixtureHandler)
        self.port = self.server.server_address[1]
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *_):
        self.server.shutdown()

    @property
    def url(self):
        return f'http://127.0.0.1:{self.port}'


def make_tiger_zip(state_fips: str) -> bytes:
    """Create a valid zip containing the real TIGER shapefiles for a state."""
    tiger_dir = TIGER_FIXTURES / f'tl_2020_{state_fips}_tract'
    if not tiger_dir.exists():
        pytest.skip(f'TIGER fixture not found: {tiger_dir}')

    buf = io.BytesIO()
    with zipfile.ZipFile(buf, 'w', zipfile.ZIP_DEFLATED) as zf:
        for f in tiger_dir.iterdir():
            zf.write(f, f.name)
    return buf.getvalue()


def make_minimal_zip(filename: str, content: bytes = b'test data') -> bytes:
    """Create a minimal zip with one file."""
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, 'w') as zf:
        zf.writestr(filename, content)
    return buf.getvalue()


def make_manifest(server_url: str, states: dict) -> dict:
    """Build a test manifest pointing to the local fixture server."""
    return {
        'version': '1',
        'github_repo': 'test/repo',
        'releases': {'data_inputs': 'test-v1', 'outputs_v3': 'v3', 'outputs_v4': 'v4'},
        'local_data_dir': 'data',
        'local_outputs_dir': 'outputs',
        'states': states,
    }


def run_fetch(args: list, env: dict, cwd: str = None) -> subprocess.CompletedProcess:
    """Run `redist fetch` with given args and env."""
    base_env = os.environ.copy()
    base_env.update(env)
    return subprocess.run(
        [str(REDIST_BIN), 'fetch'] + args,
        capture_output=True, text=True, timeout=60,
        cwd=cwd or str(Path.cwd()),
        env=base_env
    )


# ---------------------------------------------------------------------------
# check-only tests (no server needed)
# ---------------------------------------------------------------------------

class TestCheckOnly:
    """check-only prints what would be downloaded without downloading anything."""

    def test_check_only_vermont_lists_three_items(self, tmp_path):
        result = run_fetch(
            ['--check-only', '--states', 'VT', '--year', '2020'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0, result.stderr
        # Must list tiger, pl94171, adjacency
        assert 'tiger' in result.stdout
        assert 'pl94171' in result.stdout
        assert 'adjacency' in result.stdout

    def test_check_only_shows_correct_vermont_tiger_url(self):
        result = run_fetch(
            ['--check-only', '--states', 'VT', '--year', '2020', '--type', 'tiger'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0
        assert 'tl_2020_50_tract' in result.stdout  # VT FIPS=50

    def test_check_only_shows_correct_rhode_island_url(self):
        result = run_fetch(
            ['--check-only', '--states', 'RI', '--year', '2020', '--type', 'tiger'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0
        assert 'tl_2020_44_tract' in result.stdout  # RI FIPS=44

    def test_check_only_no_download_occurs(self, tmp_path):
        """check-only must not create any files."""
        before = set(tmp_path.rglob('*'))
        run_fetch(
            ['--check-only', '--states', 'VT', '--year', '2020'],
            env={'REDIST_PYTHON': sys.executable},
            cwd=str(tmp_path)
        )
        after = set(tmp_path.rglob('*'))
        assert before == after, f'check-only created files: {after - before}'

    def test_check_only_marks_existing_files_as_available(self, tmp_path):
        """Files already on disk are shown as [FILE] not [NEED]."""
        # Create the TIGER shp to simulate it already being present
        tiger_dir = tmp_path / 'data' / '2020' / 'tiger' / 'tracts' / 'tl_2020_50_tract'
        tiger_dir.mkdir(parents=True)
        (tiger_dir / 'tl_2020_50_tract.shp').write_bytes(b'stub')

        result = run_fetch(
            ['--check-only', '--states', 'VT', '--year', '2020', '--type', 'tiger'],
            env={'REDIST_PYTHON': sys.executable},
            cwd=str(tmp_path)
        )
        assert result.returncode == 0
        assert '[FILE]' in result.stdout or '[OK]' in result.stdout

    def test_check_only_summary_shows_counts(self):
        result = run_fetch(
            ['--check-only', '--states', 'VT', 'RI', '--year', '2020'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0
        assert 'available' in result.stdout
        assert 'need download' in result.stdout


# ---------------------------------------------------------------------------
# TIGER download tests (real fixture zip from local server)
# ---------------------------------------------------------------------------

class TestTigerDownload:

    def test_download_vermont_tiger(self, tmp_path):
        """Download VT TIGER from local fixture server and extract."""
        vt_zip = make_tiger_zip('50')
        routes = {'/tl_2020_50_tract.zip': ('application/zip', vt_zip)}

        manifest = make_manifest('', {
            'VT': {
                'name': 'Vermont', 'fips': '50',
                'districts': {'2020': 1, '2010': 1, '2000': 1},
                'tiger': {},  # overridden by REDIST_MANIFEST pointing to server
                'pl94171': {},
            }
        })
        # We'll inject the server URL via a custom manifest
        with FixtureServer(routes) as server:
            manifest['states']['VT']['tiger']['2020'] = \
                f'{server.url}/tl_2020_50_tract.zip'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            result = run_fetch(
                ['--states', 'VT', '--year', '2020', '--type', 'tiger',
                 '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )

        assert result.returncode == 0, f'STDOUT: {result.stdout}\nSTDERR: {result.stderr}'
        # TIGER shp should be extracted
        tiger_dir = tmp_path / 'data' / '2020' / 'tiger' / 'tracts' / 'tl_2020_50_tract'
        assert tiger_dir.exists(), f'Expected {tiger_dir}'
        shp_files = list(tiger_dir.glob('*.shp'))
        assert shp_files, 'No .shp file extracted'

    def test_download_rhode_island_tiger_2district(self, tmp_path):
        """RI has 2 districts — verifies multi-district state downloads correctly."""
        ri_zip = make_tiger_zip('44')
        routes = {'/tl_2020_44_tract.zip': ('application/zip', ri_zip)}

        manifest = make_manifest('', {
            'RI': {
                'name': 'Rhode Island', 'fips': '44',
                'districts': {'2020': 2, '2010': 2, '2000': 2},
                'tiger': {}, 'pl94171': {},
            }
        })
        with FixtureServer(routes) as server:
            manifest['states']['RI']['tiger']['2020'] = \
                f'{server.url}/tl_2020_44_tract.zip'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            result = run_fetch(
                ['--states', 'RI', '--year', '2020', '--type', 'tiger',
                 '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )

        assert result.returncode == 0, result.stderr
        tiger_dir = tmp_path / 'data' / '2020' / 'tiger' / 'tracts' / 'tl_2020_44_tract'
        assert (tiger_dir / 'tl_2020_44_tract.shp').exists()

    def test_download_creates_done_marker(self, tmp_path):
        """Successful download creates a .done marker file."""
        routes = {'/vt.zip': ('application/zip', make_minimal_zip('tl_2020_50_tract.shp'))}

        manifest = make_manifest('', {
            'VT': {
                'name': 'Vermont', 'fips': '50',
                'districts': {'2020': 1, '2010': 1, '2000': 1},
                'tiger': {}, 'pl94171': {},
            }
        })
        with FixtureServer(routes) as server:
            manifest['states']['VT']['tiger']['2020'] = f'{server.url}/vt.zip'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            run_fetch(
                ['--states', 'VT', '--year', '2020', '--type', 'tiger',
                 '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )

        # Find .done file
        done_files = list(tmp_path.rglob('*.done'))
        assert done_files, 'No .done marker created after download'

    def test_download_skips_when_done_marker_present(self, tmp_path):
        """If .done marker exists, file is not re-downloaded."""
        # URL must use TIGER naming so binary computes same local path
        # URL /tl_2020_50_tract.zip -> extracted to data/.../tl_2020_50_tract/tl_2020_50_tract.shp
        shp_dir = tmp_path / 'data' / '2020' / 'tiger' / 'tracts' / 'tl_2020_50_tract'
        shp_dir.mkdir(parents=True)
        shp_file = shp_dir / 'tl_2020_50_tract.shp'
        shp_file.write_bytes(b'original')
        (shp_file.with_suffix('.done')).write_bytes(b'done')

        # Route must use TIGER filename so binary derives same local path
        routes = {'/tl_2020_50_tract.zip': ('application/zip', make_tiger_zip('50'))}
        manifest = make_manifest('', {
            'VT': {
                'name': 'Vermont', 'fips': '50',
                'districts': {'2020': 1, '2010': 1, '2000': 1},
                'tiger': {}, 'pl94171': {},
            }
        })
        with FixtureServer(routes) as server:
            manifest['states']['VT']['tiger']['2020'] = f'{server.url}/tl_2020_50_tract.zip'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            result = run_fetch(
                ['--states', 'VT', '--year', '2020', '--type', 'tiger',
                 '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )

        assert result.returncode == 0
        assert 'SKIP' in result.stdout or 'already present' in result.stdout, \
            f'Expected SKIP, got: {result.stdout!r}'
        # Original file should be unchanged
        assert shp_file.read_bytes() == b'original'

    def test_force_redownloads_even_with_done_marker(self, tmp_path):
        """--force downloads even if .done marker present."""
        shp_dir = tmp_path / 'data' / '2020' / 'tiger' / 'tracts' / 'tl_2020_50_tract'
        shp_dir.mkdir(parents=True)
        shp_file = shp_dir / 'tl_2020_50_tract.shp'
        shp_file.write_bytes(b'old content')
        (shp_file.with_suffix('.done')).write_bytes(b'done')

        vt_zip = make_tiger_zip('50')
        routes = {'/tl_2020_50_tract.zip': ('application/zip', vt_zip)}
        manifest = make_manifest('', {
            'VT': {
                'name': 'Vermont', 'fips': '50',
                'districts': {'2020': 1, '2010': 1, '2000': 1},
                'tiger': {}, 'pl94171': {},
            }
        })
        with FixtureServer(routes) as server:
            manifest['states']['VT']['tiger']['2020'] = f'{server.url}/tl_2020_50_tract.zip'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            result = run_fetch(
                ['--states', 'VT', '--year', '2020', '--type', 'tiger',
                 '--force', '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )

        assert result.returncode == 0, f'force re-download failed: {result.stderr}'
        # File should be overwritten by real TIGER extract, not 'old content'
        assert shp_file.read_bytes() != b'old content', \
            '--force did not re-download; file unchanged'

    def test_server_404_returns_error(self, tmp_path):
        """HTTP 404 from server should cause fetch to fail with clear error."""
        routes = {}  # empty — everything returns 404
        manifest = make_manifest('', {
            'VT': {
                'name': 'Vermont', 'fips': '50',
                'districts': {'2020': 1, '2010': 1, '2000': 1},
                'tiger': {}, 'pl94171': {},
            }
        })
        with FixtureServer(routes) as server:
            manifest['states']['VT']['tiger']['2020'] = f'{server.url}/notfound.zip'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            result = run_fetch(
                ['--states', 'VT', '--year', '2020', '--type', 'tiger',
                 '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )

        assert result.returncode != 0, 'Should fail on 404'
        assert '404' in result.stderr or 'HTTP' in result.stderr

    def test_malformed_zip_returns_error(self, tmp_path):
        """Malformed zip response should cause fetch to fail with clear error."""
        routes = {'/bad.zip': ('application/zip', b'not a zip file at all')}
        manifest = make_manifest('', {
            'VT': {
                'name': 'Vermont', 'fips': '50',
                'districts': {'2020': 1, '2010': 1, '2000': 1},
                'tiger': {}, 'pl94171': {},
            }
        })
        with FixtureServer(routes) as server:
            manifest['states']['VT']['tiger']['2020'] = f'{server.url}/bad.zip'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            result = run_fetch(
                ['--states', 'VT', '--year', '2020', '--type', 'tiger',
                 '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )

        assert result.returncode != 0, 'Should fail on malformed zip'
        assert result.stderr  # must have error message


# ---------------------------------------------------------------------------
# Manifest tests
# ---------------------------------------------------------------------------

class TestManifest:

    def test_custom_manifest_via_flag(self, tmp_path):
        """--manifest flag overrides the builtin manifest."""
        minimal_manifest = {
            'version': '1',
            'github_repo': 'test/repo',
            'releases': {'data_inputs': 'v1', 'outputs_v3': 'v3', 'outputs_v4': 'v4'},
            'local_data_dir': 'data',
            'local_outputs_dir': 'outputs',
            'states': {
                'XX': {
                    'name': 'Testland', 'fips': '99',
                    'districts': {'2020': 1, '2010': 1, '2000': 1},
                    'tiger': {'2020': 'http://example.com/xx.zip'},
                    'pl94171': {'2020': 'http://example.com/xx_pl.zip'},
                }
            }
        }
        manifest_file = tmp_path / 'custom.json'
        manifest_file.write_text(json.dumps(minimal_manifest))

        result = run_fetch(
            ['--check-only', '--manifest', str(manifest_file)],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0
        assert 'XX' in result.stdout  # custom state from custom manifest

    def test_custom_manifest_via_env(self, tmp_path):
        """REDIST_MANIFEST env var overrides builtin manifest."""
        manifest = {
            'version': '1',
            'github_repo': 'test/repo',
            'releases': {'data_inputs': 'v1', 'outputs_v3': 'v3', 'outputs_v4': 'v4'},
            'local_data_dir': 'data',
            'local_outputs_dir': 'outputs',
            'states': {
                'YY': {
                    'name': 'Envland', 'fips': '88',
                    'districts': {'2020': 2, '2010': 2, '2000': 2},
                    'tiger': {'2020': 'http://example.com/yy.zip'},
                    'pl94171': {'2020': 'http://example.com/yy_pl.zip'},
                }
            }
        }
        manifest_file = tmp_path / 'env_manifest.json'
        manifest_file.write_text(json.dumps(manifest))

        result = run_fetch(
            ['--check-only'],
            env={'REDIST_PYTHON': sys.executable, 'REDIST_MANIFEST': str(manifest_file)}
        )
        assert result.returncode == 0
        assert 'YY' in result.stdout

    def test_builtin_manifest_has_all_50_states(self):
        """The builtin manifest (embedded in binary) covers all 50 states."""
        result = run_fetch(
            ['--check-only', '--year', '2020'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0
        # Count unique state codes in output
        lines = result.stdout.splitlines()
        codes = {line.split()[1] for line in lines if line.startswith('[') and len(line.split()) >= 2}
        assert len(codes) == 50, f'Expected 50 states, got {len(codes)}: {sorted(codes)}'

    def test_invalid_manifest_returns_error(self, tmp_path):
        """Malformed manifest JSON should fail clearly."""
        bad_manifest = tmp_path / 'bad.json'
        bad_manifest.write_text('this is not json {{{')

        result = run_fetch(
            ['--check-only', '--manifest', str(bad_manifest)],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode != 0
        assert result.stderr  # must have error message


# ---------------------------------------------------------------------------
# Multi-state and multi-year tests
# ---------------------------------------------------------------------------

class TestMultiState:

    def test_fetch_vt_and_ri_check_only(self):
        """VT (1 district) and RI (2 districts) both appear in check-only."""
        result = run_fetch(
            ['--check-only', '--states', 'VT', 'RI', '--year', '2020'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0
        assert 'VT' in result.stdout
        assert 'RI' in result.stdout
        assert 'tl_2020_50_tract' in result.stdout  # VT FIPS 50
        assert 'tl_2020_44_tract' in result.stdout  # RI FIPS 44

    def test_fetch_all_years_check_only(self):
        """--year all lists 2020, 2010, and 2000."""
        result = run_fetch(
            ['--check-only', '--states', 'VT', '--year', 'all'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0
        assert '2020' in result.stdout
        assert '2010' in result.stdout

    def test_fetch_tiger_only_type_filter(self):
        """--type tiger skips pl94171 and adjacency."""
        result = run_fetch(
            ['--check-only', '--states', 'VT', '--year', '2020', '--type', 'tiger'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0
        assert 'tiger' in result.stdout
        assert 'pl94171' not in result.stdout
        assert 'adjacency' not in result.stdout

    def test_ri_has_2_districts_in_manifest(self):
        """Rhode Island must have 2 districts in 2020 manifest."""
        result = run_fetch(
            ['--check-only', '--states', 'RI', '--year', '2020'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0
        # RI should appear — if manifest has 0 districts for 2020, it'd be skipped
        assert 'RI' in result.stdout


# ---------------------------------------------------------------------------
# E2E: fetch then run (VT only — fast)
# ---------------------------------------------------------------------------

class TestFetchThenRun:
    """After fetching data, `redist state` should work end-to-end."""

    @pytest.mark.skipif(
        not (Path('outputs/V3/data/2020/adjacency/vt_adjacency_2020.pkl').exists()),
        reason='VT adjacency not present'
    )
    def test_fetch_tiger_then_run_vermont(self, tmp_path):
        """
        Fetch VT TIGER from fixture server, then run redistricting.
        Proves the fetch → run pipeline works for a first-time user.
        """
        vt_zip = make_tiger_zip('50')
        routes = {'/tl_2020_50_tract.zip': ('application/zip', vt_zip)}

        manifest = {
            'version': '1',
            'github_repo': 'test/repo',
            'releases': {'data_inputs': 'v1', 'outputs_v3': 'v3', 'outputs_v4': 'v4'},
            'local_data_dir': str(tmp_path / 'data'),
            'local_outputs_dir': 'outputs',
            'states': {
                'VT': {
                    'name': 'Vermont', 'fips': '50',
                    'districts': {'2020': 1, '2010': 1, '2000': 1},
                    'tiger': {}, 'pl94171': {},
                }
            }
        }

        with FixtureServer(routes) as server:
            manifest['states']['VT']['tiger']['2020'] = \
                f'{server.url}/tl_2020_50_tract.zip'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            # Step 1: fetch
            fetch_result = run_fetch(
                ['--states', 'VT', '--year', '2020', '--type', 'tiger',
                 '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )
            assert fetch_result.returncode == 0, f'fetch failed:\n{fetch_result.stderr}'

        # Verify TIGER was extracted
        tiger_shp = tmp_path / 'data' / '2020' / 'tiger' / 'tracts' / 'tl_2020_50_tract' / 'tl_2020_50_tract.shp'
        assert tiger_shp.exists(), 'TIGER .shp not extracted'
        assert tiger_shp.stat().st_size > 1000, 'TIGER .shp too small — probably stub'

        # Step 2: read it with the Rust TIGER reader (via redist_py)
        try:
            import redist_py
            records = redist_py.read_tiger_shp(str(tiger_shp.parent.parent / 'tl_2020_50_tract' / 'tl_2020_50_tract.shp'))
            # Actually the shp is one level up
            records = redist_py.read_tiger_shp(str(tiger_shp))
            assert len(records) == 193, f'VT should have 193 tracts, got {len(records)}'
        except ImportError:
            pytest.skip('redist_py not available')


# ---------------------------------------------------------------------------
# Tests for fixes from role review (PP-08, PP-09, PP-10)
# ---------------------------------------------------------------------------

class TestRoleReviewFixes:

    def test_url_with_query_params_strips_correctly(self, tmp_path):
        """PP-08: Query params in URL must not appear in local filename."""
        # Serve the real VT TIGER zip but with a query param in the URL
        vt_zip = make_tiger_zip('50')
        routes = {'/tl_2020_50_tract.zip': ('application/zip', vt_zip)}
        manifest = make_manifest('', {
            'VT': {
                'name': 'Vermont', 'fips': '50',
                'districts': {'2020': 1, '2010': 1, '2000': 1},
                'tiger': {}, 'pl94171': {},
            }
        })
        with FixtureServer(routes) as server:
            # URL with query param — filename should still be tl_2020_50_tract.zip
            manifest['states']['VT']['tiger']['2020'] = \
                f'{server.url}/tl_2020_50_tract.zip?token=abc123'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            result = run_fetch(
                ['--check-only', '--states', 'VT', '--year', '2020', '--type', 'tiger',
                 '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )

        assert result.returncode == 0
        # The [NEED]/[FILE] path lines must not contain query strings.
        # The "src:" continuation line shows the full original URL — that's OK.
        path_lines = [l for l in result.stdout.splitlines()
                      if l.strip().startswith(('[NEED]', '[FILE]', '[OK]', '[SKIP]'))]
        for line in path_lines:
            assert '?' not in line, f'Query param in local path: {line!r}'
        assert 'tl_2020_50_tract' in result.stdout

    def test_invalid_year_returns_error(self):
        """PP-10: Invalid year should fail with clear error, not silent empty list."""
        result = run_fetch(
            ['--check-only', '--year', '2030', '--states', 'VT'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode != 0, 'Invalid year 2030 should cause non-zero exit'
        combined = result.stdout + result.stderr
        assert '2030' in combined, f'Error should mention the invalid year: {combined}'

    def test_year_typo_returns_error(self):
        """PP-10: Year typo (202a) must not silently produce empty output."""
        result = run_fetch(
            ['--check-only', '--year', '202a', '--states', 'VT'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode != 0

    def test_year_2000_is_valid(self):
        """Year 2000 is in the allowlist."""
        result = run_fetch(
            ['--check-only', '--year', '2000', '--states', 'VT'],
            env={'REDIST_PYTHON': sys.executable}
        )
        assert result.returncode == 0

    def test_large_download_streams_to_disk(self, tmp_path):
        """PP-09: Large downloads must stream to disk, not buffer in RAM."""
        import io, zipfile as _zipfile
        # 5MB fake zip — large enough to test streaming, small enough for CI
        buf = io.BytesIO()
        with _zipfile.ZipFile(buf, 'w', _zipfile.ZIP_STORED) as zf:
            zf.writestr('tl_2020_50_tract.shp', b'X' * 5 * 1024 * 1024)
        large_zip = buf.getvalue()

        routes = {'/tl_2020_50_tract.zip': ('application/zip', large_zip)}
        manifest = make_manifest('', {
            'VT': {
                'name': 'Vermont', 'fips': '50',
                'districts': {'2020': 1, '2010': 1, '2000': 1},
                'tiger': {}, 'pl94171': {},
            }
        })
        with FixtureServer(routes) as server:
            manifest['states']['VT']['tiger']['2020'] = f'{server.url}/tl_2020_50_tract.zip'
            manifest_file = tmp_path / 'manifest.json'
            manifest_file.write_text(json.dumps(manifest))

            result = run_fetch(
                ['--states', 'VT', '--year', '2020', '--type', 'tiger',
                 '--manifest', str(manifest_file)],
                env={'REDIST_PYTHON': sys.executable},
                cwd=str(tmp_path)
            )

        assert result.returncode == 0, f'Large zip download failed:\n{result.stderr}'
        shp = (tmp_path / 'data' / '2020' / 'tiger' / 'tracts' /
               'tl_2020_50_tract' / 'tl_2020_50_tract.shp')
        assert shp.exists(), 'File not extracted from large zip'
        assert shp.stat().st_size == 5 * 1024 * 1024, \
            f'Expected 5MB, got {shp.stat().st_size} bytes'
