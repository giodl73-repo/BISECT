"""
Integration test: VRA compliance results for V4 (metis-vra) run.

Validates the 42% threshold finding from Paper D.1:
- States with ≥42% minority (Georgia, Mississippi) achieve all MM district targets
- States with <37% minority (Alabama, South Carolina) achieve 0 MM districts
- Louisiana (borderline) achieves partial compliance

Reads vra_analysis.json exported alongside vra_analysis.pkl by generate_dashboard.py.
"""

import json
import pytest
from pathlib import Path

VRA_OUTPUTS = Path('outputs/V4/2020/states')

# (state_dir_name, expected_mm_districts, comparison, note)
VRA_TARGETS = [
    ('georgia',        5, '>=', 'Should achieve all 5 (42.4% minority)'),
    ('mississippi',    2, '>=', 'Should achieve all 2 (46.1% minority)'),
    ('alabama',        0, '==', 'Cannot achieve any (36.9% minority, geography dispersed)'),
    ('south_carolina', 0, '==', 'Cannot achieve any (35.1% minority)'),
    ('louisiana',      1, '>=', 'Partial: achieves at least 1 of 2 (41.6% minority, borderline)'),
]


def load_vra_data(state: str) -> dict | None:
    """Load vra_analysis.json for a state."""
    json_path = VRA_OUTPUTS / state / 'data' / 'vra_analysis.json'
    if not json_path.exists():
        return None
    with open(json_path) as f:
        return json.load(f)


@pytest.mark.parametrize('state,expected,comparison,note', VRA_TARGETS,
                         ids=[s for s, *_ in VRA_TARGETS])
def test_vra_mm_district_count(state, expected, comparison, note):
    """Majority-minority district count must meet expected threshold."""
    if not VRA_OUTPUTS.exists():
        pytest.skip('V4 VRA outputs not found — run with --partition-mode metis-vra')

    data = load_vra_data(state)
    if data is None:
        pytest.skip(f'{state}: vra_analysis.json not found (run V4 pipeline)')

    actual = data.get('mm_count', 0)

    if comparison == '>=':
        assert actual >= expected, (
            f'{state}: expected ≥{expected} MM districts, got {actual}. {note}'
        )
    elif comparison == '==':
        assert actual == expected, (
            f'{state}: expected exactly {expected} MM districts, got {actual}. {note}'
        )
    elif comparison == '<=':
        assert actual <= expected, (
            f'{state}: expected ≤{expected} MM districts, got {actual}. {note}'
        )


def test_georgia_minority_percentages():
    """Georgia's MM districts should each have ≥50% minority population."""
    if not VRA_OUTPUTS.exists():
        pytest.skip('V4 VRA outputs not found')

    data = load_vra_data('georgia')
    if data is None:
        pytest.skip('georgia vra_analysis.json not found')

    mm_district_ids = set(data.get('mm_districts', []))
    below_threshold = []
    for d in data.get('districts', []):
        if d['district'] in mm_district_ids:
            if d['pct_minority'] < 0.50:
                below_threshold.append(
                    f"District {d['district']}: {d['pct_minority']*100:.1f}% minority"
                )

    assert not below_threshold, (
        f'Georgia MM districts below 50% minority threshold:\n  '
        + '\n  '.join(below_threshold)
    )


def test_vra_data_structure():
    """VRA analysis JSON must have expected keys for all available states."""
    if not VRA_OUTPUTS.exists():
        pytest.skip('V4 VRA outputs not found')

    required_keys = {'mm_count', 'mm_districts', 'districts'}
    district_keys = {'district', 'pct_minority', 'pct_black', 'pct_hispanic', 'is_mm'}

    for state_dir in sorted(VRA_OUTPUTS.iterdir()):
        json_path = state_dir / 'data' / 'vra_analysis.json'
        if not json_path.exists():
            continue
        with open(json_path) as f:
            data = json.load(f)

        missing_keys = required_keys - set(data.keys())
        assert not missing_keys, f'{state_dir.name}: missing keys {missing_keys}'

        if data['districts']:
            missing_d_keys = district_keys - set(data['districts'][0].keys())
            assert not missing_d_keys, (
                f'{state_dir.name}: district missing keys {missing_d_keys}'
            )
