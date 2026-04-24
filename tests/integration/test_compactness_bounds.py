"""
Integration test: compactness scores match the paper findings.

Paper B.2 reports mean Polsby-Popper = 0.367 for 2020 edge-weighted.
Paper C.2 shows ~0.320 for 2010, with 2020 > 2010 consistently.
These tests encode those findings as regression guards.
"""

import csv
import pytest
from pathlib import Path

from .conftest import get_outputs_root, get_years

OUTPUTS_ROOT = get_outputs_root()
YEARS = get_years(OUTPUTS_ROOT)


# Expected national means from papers (with tolerance)
EXPECTED_MEANS = {
    '2020': (0.367, 0.05),   # Paper B.2: 0.367 ± 0.05
    '2010': (0.320, 0.06),   # Paper C.2: ~0.320 ± 0.06
    '2000': (0.300, 0.08),   # Estimated from cross-census trend
}


def load_national_pp(year: str) -> tuple[float, list[float]]:
    """Return (national_mean_pp, list_of_state_means) for a year."""
    states_dir = OUTPUTS_ROOT / year / 'states'
    all_pp = []
    state_means = []

    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue
        csv_path = state_dir / 'data' / 'district_summary.csv'
        if not csv_path.exists():
            continue
        with open(csv_path) as f:
            rows = list(csv.DictReader(f))
        pp_col = next((c for c in (rows[0] if rows else {}) if 'polsby' in c.lower()), None)
        if not pp_col:
            continue
        state_pp = [float(r[pp_col]) for r in rows if r.get(pp_col)]
        if state_pp:
            all_pp.extend(state_pp)
            state_means.append(sum(state_pp) / len(state_pp))

    national_mean = sum(all_pp) / len(all_pp) if all_pp else 0.0
    return national_mean, state_means


@pytest.mark.parametrize('year', [y for y in YEARS if (OUTPUTS_ROOT / y / 'states').exists()])
def test_national_mean_pp_in_range(year):
    """National mean Polsby-Popper must be within expected range."""
    national_mean, _ = load_national_pp(year)
    expected, tolerance = EXPECTED_MEANS.get(year, (0.30, 0.10))

    assert national_mean > 0, f'{year}: no PP data found'
    assert abs(national_mean - expected) <= tolerance, (
        f'{year}: national mean PP = {national_mean:.3f}, '
        f'expected {expected:.3f} ± {tolerance:.3f}'
    )


def test_2020_beats_2010_nationally():
    """2020 edge-weighted should outperform 2010 (geography changed but method is same)."""
    if not (OUTPUTS_ROOT / '2020' / 'states').exists():
        pytest.skip('2020 outputs not found')
    if not (OUTPUTS_ROOT / '2010' / 'states').exists():
        pytest.skip('2010 outputs not found')

    mean_2020, _ = load_national_pp('2020')
    mean_2010, _ = load_national_pp('2010')

    assert mean_2020 > mean_2010, (
        f'Expected 2020 PP ({mean_2020:.3f}) > 2010 PP ({mean_2010:.3f})'
    )


@pytest.mark.parametrize('year', [y for y in YEARS if (OUTPUTS_ROOT / y / 'states').exists()])
def test_no_state_mean_below_floor(year):
    """No state mean PP should be unreasonably low (< 0.05).

    Hawaii has naturally low PP (~0.09) due to island coastlines inflating perimeter.
    Floor of 0.05 catches genuine data errors while allowing coastal/island states.
    """
    _, state_means = load_national_pp(year)
    below_floor = [m for m in state_means if m < 0.05]
    assert not below_floor, (
        f'{year}: {len(below_floor)} states have mean PP < 0.05 (possible data error)'
    )


@pytest.mark.parametrize('year', [y for y in YEARS if (OUTPUTS_ROOT / y / 'states').exists()])
def test_no_district_pp_above_ceiling(year):
    """No individual district PP should exceed 1.0 (mathematically impossible)."""
    states_dir = OUTPUTS_ROOT / year / 'states'
    violations = []
    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue
        csv_path = state_dir / 'data' / 'district_summary.csv'
        if not csv_path.exists():
            continue
        with open(csv_path) as f:
            rows = list(csv.DictReader(f))
        pp_col = next((c for c in (rows[0] if rows else {}) if 'polsby' in c.lower()), None)
        if not pp_col:
            continue
        for r in rows:
            pp = float(r.get(pp_col, 0) or 0)
            if pp > 1.0:
                violations.append(f'{state_dir.name} district {r.get("district", "?")}: PP={pp:.3f}')
    assert not violations, f'{year}: districts with PP > 1.0:\n  ' + '\n  '.join(violations)
