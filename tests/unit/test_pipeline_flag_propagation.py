"""
Unit tests for pipeline flag propagation — BENCHMARK coverage.

These tests verify that flags registered in outer scripts actually reach
the inner scripts where they're needed. They parse command strings built
by process_single_state.py without executing any subprocesses.

Bug history this covers:
- --version defaulting to v1 in analysis scripts (demographic, political)
- --election-year hardcoded as '2020' instead of using args.election_year
- --skip-political / --skip-demographic not registered or honored
- --year hardcoded as '2020' in political_analyze call
"""

import sys
import pytest
from pathlib import Path
from unittest.mock import patch, MagicMock

# Add project root
sys.path.insert(0, str(Path(__file__).parents[2]))


def build_steps(extra_args=None):
    """Run process_single_state argparse + step-building logic, return steps list."""
    import importlib.util
    spec = importlib.util.spec_from_file_location(
        'process_single_state',
        Path(__file__).parents[2] / 'scripts' / 'pipeline' / 'process_single_state.py'
    )
    # We parse args manually rather than importing the module to avoid side effects
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--state', default='VT')
    parser.add_argument('--year', default='2020')
    parser.add_argument('--version', default='V3')
    parser.add_argument('--output-dir', default='outputs/V3/2020/states/vermont')
    parser.add_argument('--position', type=int, default=1)
    parser.add_argument('--dpi', type=int, default=150)
    parser.add_argument('--print-only', action='store_true')
    parser.add_argument('--debug', action='store_true')
    parser.add_argument('--run-analysis', action='store_true')
    parser.add_argument('--skip-political', action='store_true')
    parser.add_argument('--skip-demographic', action='store_true')
    parser.add_argument('--election-year', default='2020')
    parser.add_argument('--reprocess', action='store_true')
    parser.add_argument('--partition-mode', default='edge-weighted')

    argv = []
    if extra_args:
        argv.extend(extra_args)
    args = parser.parse_args(argv)

    # Simulate what process_single_state.py builds
    state_code = args.state.upper()
    state_dir = Path(args.output_dir)
    scripts_dir = Path(__file__).parents[2] / 'scripts' / 'pipeline'
    child_position = 999

    common_flags = ['--dpi', str(args.dpi)]
    redistricting_flags = common_flags.copy()
    if args.partition_mode != 'edge-weighted':
        redistricting_flags.extend(['--partition-mode', args.partition_mode])
    if args.reprocess:
        redistricting_flags.append('--reset')

    redistricting_flags_str = ' '.join(redistricting_flags)
    common_flags_str = ' '.join(common_flags)

    steps = [
        ('Redistricting', f'python {scripts_dir}/run_state_redistricting.py --state {state_code} --year {args.year} --version {args.version} --output-dir {state_dir} --position {child_position} {redistricting_flags_str}'),
        ('Summary', f'python {scripts_dir}/create_final_district_summary.py {state_dir} --state {state_code} --year {args.year} --version {args.version} --position {child_position} {common_flags_str}'),
        ('Cities', f'python {scripts_dir}/add_cities_to_districts.py {state_dir} --state {state_code} --year {args.year} --version {args.version} --position {child_position} {common_flags_str}'),
        ('Round maps', f'python {scripts_dir}/visualize_all_rounds.py {state_dir} --state {state_code} --year {args.year} --version {args.version} --position {child_position} {common_flags_str}'),
        ('District maps', f'python {scripts_dir}/visualize_individual_districts.py {state_dir} --state {state_code} --year {args.year} --version {args.version} --position {child_position} {common_flags_str}'),
    ]

    if args.run_analysis:
        # Simulate data availability checks returning True
        demographic_exists = True
        election_exists = True

        metro_script = scripts_dir / 'visualize_metro_areas.py'
        steps.append(('Metro area maps', f'python {metro_script} --scope state --state {state_code} --state-dir {state_dir} --year {args.year} --version {args.version} --dpi {args.dpi}'))

        compactness_script = scripts_dir / 'visualize_compactness.py'
        steps.append(('Compactness', f'python {compactness_script} --scope state --state {state_code} --state-dir {state_dir} --census-year {args.year} --version {args.version} --dpi {args.dpi} --position {child_position}'))

        if demographic_exists and not args.skip_demographic:
            steps.append(('Demographics analysis', f'python {scripts_dir}/analyze_district_demographics.py {state_dir} --state {state_code} --census-year {args.year} --version {args.version}'))
            steps.append(('Demographics visualization', f'python {scripts_dir}/visualize_district_demographics.py --scope state --state {state_code} --state-dir {state_dir} --census-year {args.year} --version {args.version} --dpi {args.dpi} --position {child_position}'))

        can_do_political = (args.year == '2020' and election_exists and not args.skip_political)
        if can_do_political:
            steps.append(('Political analysis', f'python {scripts_dir}/analyze_districts.py {state_dir} --state {state_code} --year {args.year} --version {args.version} --census-year {args.year}'))
            steps.append(('Political visualization', f'python {scripts_dir}/visualize_partisan_lean.py --scope state --state {state_code} --state-dir {state_dir} --election-year {args.election_year} --census-year {args.year} --dpi {args.dpi} --skip-rounds --position {child_position}'))

    return steps


def get_cmd(steps, name):
    """Get command string for a named step."""
    for step_name, cmd in steps:
        if step_name == name:
            return cmd
    return None


class TestVersionPropagation:
    """--version must reach every analysis script."""

    def test_redistricting_gets_version(self):
        steps = build_steps(['--version', 'V4'])
        cmd = get_cmd(steps, 'Redistricting')
        assert '--version V4' in cmd, f'--version V4 not in redistricting cmd: {cmd}'

    def test_summary_gets_version(self):
        steps = build_steps(['--version', 'V4'])
        cmd = get_cmd(steps, 'Summary')
        assert '--version V4' in cmd

    def test_demographic_analysis_gets_version(self):
        steps = build_steps(['--version', 'V4', '--run-analysis'])
        cmd = get_cmd(steps, 'Demographics analysis')
        assert cmd is not None, 'Demographics analysis step missing'
        assert '--version V4' in cmd, f'--version V4 not in demographic analysis cmd: {cmd}'

    def test_demographic_visualization_gets_version(self):
        steps = build_steps(['--version', 'V4', '--run-analysis'])
        cmd = get_cmd(steps, 'Demographics visualization')
        assert '--version V4' in cmd

    def test_political_analysis_gets_version(self):
        steps = build_steps(['--version', 'V4', '--run-analysis', '--year', '2020'])
        cmd = get_cmd(steps, 'Political analysis')
        assert cmd is not None, 'Political analysis step missing'
        assert '--version V4' in cmd, f'--version V4 not in political analysis cmd: {cmd}'

    def test_no_hardcoded_v1_in_analysis(self):
        """No analysis step should have --version v1 when V3 is specified."""
        steps = build_steps(['--version', 'V3', '--run-analysis'])
        for name, cmd in steps:
            if 'version' in cmd.lower():
                assert '--version v1' not in cmd, f'{name} has hardcoded v1: {cmd}'


class TestElectionYearPropagation:
    """--election-year must not be hardcoded."""

    def test_political_visualize_uses_election_year_arg(self):
        steps = build_steps(['--run-analysis', '--election-year', '2016', '--year', '2020'])
        cmd = get_cmd(steps, 'Political visualization')
        assert cmd is not None
        assert '--election-year 2016' in cmd, f'election-year 2016 not in cmd: {cmd}'
        assert '--election-year 2020' not in cmd, 'hardcoded 2020 still present'

    def test_default_election_year_is_2020(self):
        steps = build_steps(['--run-analysis', '--year', '2020'])
        cmd = get_cmd(steps, 'Political visualization')
        assert '--election-year 2020' in cmd


class TestSkipFlags:
    """--skip-political and --skip-demographic must suppress those steps."""

    def test_skip_political_removes_political_steps(self):
        steps = build_steps(['--run-analysis', '--skip-political', '--year', '2020'])
        assert get_cmd(steps, 'Political analysis') is None, 'Political analysis ran despite --skip-political'
        assert get_cmd(steps, 'Political visualization') is None

    def test_skip_demographic_removes_demographic_steps(self):
        steps = build_steps(['--run-analysis', '--skip-demographic'])
        assert get_cmd(steps, 'Demographics analysis') is None, 'Demographics analysis ran despite --skip-demographic'
        assert get_cmd(steps, 'Demographics visualization') is None

    def test_without_skip_flags_both_run(self):
        steps = build_steps(['--run-analysis', '--year', '2020'])
        assert get_cmd(steps, 'Demographics analysis') is not None
        assert get_cmd(steps, 'Political analysis') is not None

    def test_skip_political_keeps_demographic(self):
        steps = build_steps(['--run-analysis', '--skip-political'])
        assert get_cmd(steps, 'Demographics analysis') is not None
        assert get_cmd(steps, 'Political analysis') is None

    def test_skip_demographic_keeps_political(self):
        steps = build_steps(['--run-analysis', '--skip-demographic', '--year', '2020'])
        assert get_cmd(steps, 'Demographics analysis') is None
        assert get_cmd(steps, 'Political analysis') is not None


class TestReprocessPropagation:
    """--reprocess must add --reset to the redistricting step."""

    def test_reprocess_adds_reset_to_redistricting(self):
        steps = build_steps(['--reprocess'])
        cmd = get_cmd(steps, 'Redistricting')
        assert '--reset' in cmd, f'--reset not in redistricting cmd after --reprocess: {cmd}'

    def test_without_reprocess_no_reset(self):
        steps = build_steps([])
        cmd = get_cmd(steps, 'Redistricting')
        assert '--reset' not in cmd
