"""
Integration test: verify all pipeline outputs exist for all states × years.

Checks V3 outputs for 2020, 2010, and 2000. Each state must have:
- data/district_summary.csv
- data/rounds_hierarchy.csv
- data/final_assignments.pkl
- maps/all_districts.png
- maps/rounds/ (at least one round map)

Skips gracefully if a year hasn't been run yet.
"""

import pytest
from pathlib import Path

OUTPUTS_ROOT = Path('outputs/V3')
REQUIRED_FILES = [
    'data/district_summary.csv',
    'data/rounds_hierarchy.csv',
    'data/final_assignments.pkl',
    'maps/all_districts.png',
]
YEARS = ['2020', '2010', '2000']


def get_completed_states(year: str):
    """Return list of (year, state_name) for states with outputs."""
    year_dir = OUTPUTS_ROOT / year / 'states'
    if not year_dir.exists():
        return []
    return [
        (year, state_dir.name)
        for state_dir in sorted(year_dir.iterdir())
        if state_dir.is_dir()
    ]


def pytest_generate_tests(metafunc):
    if 'year_state' in metafunc.fixturenames:
        params = []
        for year in YEARS:
            params.extend(get_completed_states(year))
        if not params:
            params = [pytest.param(('2020', 'vermont'), marks=pytest.mark.skip(reason='No outputs found'))]
        metafunc.parametrize('year_state', params, ids=[f'{y}/{s}' for y, s in params])


def test_required_files_exist(year_state):
    """Every completed state must have all required output files."""
    year, state = year_state
    state_dir = OUTPUTS_ROOT / year / 'states' / state

    missing = []
    for rel_path in REQUIRED_FILES:
        if not (state_dir / rel_path).exists():
            missing.append(rel_path)

    assert not missing, (
        f'{state} ({year}): missing files:\n  ' + '\n  '.join(missing)
    )


def test_round_maps_exist(year_state):
    """States with ≥2 districts must have at least one round progression map.

    Single-district states (AK, DE, MT, ND, SD, VT, WY in various decades)
    produce no bisection rounds — the whole state is district 1.
    """
    import csv
    year, state = year_state
    # Check district count — skip single-district states (no rounds to visualize)
    csv_path = OUTPUTS_ROOT / year / 'states' / state / 'data' / 'district_summary.csv'
    if csv_path.exists():
        with open(csv_path) as f:
            num_districts = sum(1 for _ in csv.DictReader(f))
        if num_districts <= 1:
            pytest.skip(f'{state} ({year}): single-district state has no bisection rounds')

    rounds_dir = OUTPUTS_ROOT / year / 'states' / state / 'maps' / 'rounds'
    assert rounds_dir.exists(), f'{state} ({year}): maps/rounds/ directory missing'
    round_maps = list(rounds_dir.glob('round_*.png'))
    assert len(round_maps) >= 1, f'{state} ({year}): no round maps in maps/rounds/'


def test_district_summary_not_empty(year_state):
    """district_summary.csv must have at least one data row."""
    import csv
    year, state = year_state
    csv_path = OUTPUTS_ROOT / year / 'states' / state / 'data' / 'district_summary.csv'
    if not csv_path.exists():
        pytest.skip('district_summary.csv missing (caught by test_required_files_exist)')
    with open(csv_path) as f:
        rows = list(csv.DictReader(f))
    assert len(rows) >= 1, f'{state} ({year}): district_summary.csv has no data rows'


def test_all_years_have_50_states():
    """Each completed year should have exactly 50 state directories."""
    for year in YEARS:
        year_dir = OUTPUTS_ROOT / year / 'states'
        if not year_dir.exists():
            continue  # year not run yet
        state_dirs = [d for d in year_dir.iterdir() if d.is_dir()]
        assert len(state_dirs) == 50, (
            f'Year {year}: expected 50 states, found {len(state_dirs)}: '
            f'{[d.name for d in state_dirs]}'
        )
