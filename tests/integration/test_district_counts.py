"""
Integration test: each state gets the correct number of districts per decade,
and the total across all states equals 435 for each year.
"""

import csv
import pytest
from pathlib import Path

OUTPUTS_ROOT = Path('outputs/V3')
YEARS = ['2020', '2010', '2000']


def load_config(year: str) -> dict:
    """Load state district counts from scripts/config_{year}.py."""
    import importlib.util, sys
    spec = importlib.util.spec_from_file_location(
        f'config_{year}',
        Path(f'scripts/config_{year}.py')
    )
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    # Config var is STATE_CONFIG_{YEAR} (e.g. STATE_CONFIG_2020)
    config_var = getattr(mod, f'STATE_CONFIG_{year}', None) or getattr(mod, 'STATE_CONFIG', None)
    return {v['name'].lower().replace(' ', '_'): v['districts']
            for v in config_var.values()}


@pytest.mark.parametrize('year', [y for y in YEARS if (OUTPUTS_ROOT / y / 'states').exists()])
def test_total_districts_equals_435(year):
    """Sum of all districts across 50 states must equal 435."""
    config = load_config(year)
    total = sum(config.values())
    assert total == 435, f'Year {year}: config sums to {total}, expected 435'


@pytest.mark.parametrize('year', [y for y in YEARS if (OUTPUTS_ROOT / y / 'states').exists()])
def test_district_counts_match_config(year):
    """Each state's actual output district count must match its config value."""
    config = load_config(year)
    states_dir = OUTPUTS_ROOT / year / 'states'

    mismatches = []
    missing = []

    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue
        state = state_dir.name
        expected = config.get(state)
        if expected is None:
            continue

        csv_path = state_dir / 'data' / 'district_summary.csv'
        if not csv_path.exists():
            missing.append(state)
            continue

        with open(csv_path) as f:
            rows = list(csv.DictReader(f))
        actual = len(rows)

        if actual != expected:
            mismatches.append(f'  {state}: expected {expected}, got {actual}')

    assert not missing, f'{year}: missing district_summary.csv for: {missing}'
    assert not mismatches, (
        f'{year}: district count mismatches:\n' + '\n'.join(mismatches)
    )
