"""
L2 tests for the `redist` CLI binary — argument parsing and STATUS protocol.

Tests the compiled binary directly via subprocess, verifying:
- All subcommands parse correctly (parity with Python argparse)
- Default values match Python defaults
- STATUS protocol produces ASCII-only output
- --help output contains expected flags

Requires redist binary to exist at redist/target/release/redist[.exe].
Skip if not compiled.
"""

import os
import subprocess
import sys
import pytest
from pathlib import Path

REDIST_BIN = Path('redist/target/release/redist.exe' if sys.platform == 'win32'
                  else 'redist/target/release/redist')

pytestmark = pytest.mark.skipif(
    not REDIST_BIN.exists(),
    reason=f'redist binary not found at {REDIST_BIN} — run: cargo build -p redist-cli --release'
)


def run(args: list[str], **kwargs) -> subprocess.CompletedProcess:
    return subprocess.run(
        [str(REDIST_BIN)] + args,
        capture_output=True, text=True, timeout=10, **kwargs
    )


class TestCliHelp:

    def test_top_level_help(self):
        r = run(['--help'])
        assert r.returncode == 0
        assert 'run' in r.stdout
        assert 'state' in r.stdout
        assert 'states' in r.stdout

    def test_run_help_has_key_flags(self):
        r = run(['run', '--help'])
        assert r.returncode == 0
        for flag in ['--year', '--version', '--workers', '--partition-mode',
                     '--states', '--reprocess', '--reset']:
            assert flag in r.stdout, f'Missing flag: {flag}'

    def test_state_help_has_key_flags(self):
        r = run(['state', '--help'])
        assert r.returncode == 0
        for flag in ['--state', '--year', '--version', '--partition-mode',
                     '--ufactor', '--niter', '--seed', '--reset']:
            assert flag in r.stdout, f'Missing flag: {flag}'

    def test_states_help_has_key_flags(self):
        r = run(['states', '--help'])
        assert r.returncode == 0
        for flag in ['--year', '--version', '--output-dir', '--workers',
                     '--partition-mode', '--run-analysis']:
            assert flag in r.stdout, f'Missing flag: {flag}'


class TestCliDefaults:
    """Verify default values in help text match Python pipeline defaults."""

    def test_run_default_year_all(self):
        r = run(['run', '--help'])
        assert 'all' in r.stdout, '--year default should be "all"'

    def test_run_default_version_v1(self):
        r = run(['run', '--help'])
        assert 'v1' in r.stdout, '--version default should be v1'

    def test_run_default_workers_12(self):
        r = run(['run', '--help'])
        assert '12' in r.stdout, '--workers default should be 12'

    def test_run_default_partition_mode(self):
        r = run(['run', '--help'])
        assert 'edge-weighted' in r.stdout, '--partition-mode default should be edge-weighted'

    def test_state_default_ufactor_5(self):
        r = run(['state', '--help'])
        assert '5' in r.stdout, '--ufactor default should be 5'

    def test_state_default_niter_100(self):
        r = run(['state', '--help'])
        assert '100' in r.stdout, '--niter default should be 100'

    def test_partition_mode_choices(self):
        r = run(['state', '--help'])
        for mode in ['unweighted', 'edge-weighted', 'metis-vra']:
            assert mode in r.stdout, f'Missing partition mode: {mode}'

    def test_year_choices(self):
        r = run(['state', '--help'])
        for year in ['2020', '2010', '2000']:
            assert year in r.stdout, f'Missing year choice: {year}'


class TestCliParsing:
    """Verify specific flag combinations parse without error."""

    def test_state_required_flag(self):
        # Missing --state should fail
        r = run(['state'])
        assert r.returncode != 0

    def test_state_with_valid_state_code(self):
        # Should parse (will fail at runtime, not at parse time)
        r = run(['state', '--state', 'VT'])
        # Runtime stub exits 1 with "not yet implemented"
        assert 'not yet implemented' in r.stderr or r.returncode == 1

    def test_vra_partition_mode_parses(self):
        r = run(['state', '--state', 'AL', '--partition-mode', 'metis-vra'])
        assert 'not yet implemented' in r.stderr or r.returncode == 1

    def test_invalid_partition_mode_fails(self):
        r = run(['state', '--state', 'VT', '--partition-mode', 'invalid-mode'])
        assert r.returncode != 0

    def test_run_unknown_flag_fails(self):
        r = run(['run', '--nonexistent-flag'])
        assert r.returncode != 0


class TestStatusProtocol:
    """Verify STATUS output is ASCII-only (MERIDIAN: Windows CP1252 constraint)."""

    def test_no_unicode_in_help_output(self):
        r = run(['--help'])
        assert r.stdout.isascii(), f'Non-ASCII in help output: {r.stdout!r}'

    def test_no_unicode_in_error_output(self):
        # Invalid args should produce ASCII error messages
        r = run(['state'])
        combined = r.stdout + r.stderr
        assert combined.isascii(), f'Non-ASCII in error output: {combined!r}'
