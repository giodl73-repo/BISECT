"""
Integration test: edge-weighted bisection outperforms unweighted by ≥40%.

Paper B.2 reports +56% compactness improvement (0.367 vs 0.236).
This test reads from outputs/data/2020/partitioner_comparison/ which
contains pre-computed comparison CSVs, or falls back to comparing
V3 (edge-weighted) against any available unweighted run data.
"""

import csv
import pytest
from pathlib import Path

from .conftest import get_outputs_root, get_years

OUTPUTS_ROOT = get_outputs_root()
YEARS = get_years(OUTPUTS_ROOT)

OUTPUTS_ROOT = Path('outputs')
COMPARISON_CSV = OUTPUTS_ROOT / 'data' / '2020' / 'partitioner_comparison' / 'partitioner_comparison_2020.csv'
MIN_IMPROVEMENT = 0.40  # ≥40% improvement (paper finds +56%)


def load_pp_from_states(version_dir: Path, year: str = '2020') -> float | None:
    """Compute national mean PP from a version's state district_summary.csv files."""
    states_dir = version_dir / year / 'states'
    if not states_dir.exists():
        return None
    all_pp = []
    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue
        csv_path = state_dir / 'data' / 'district_summary.csv'
        if not csv_path.exists():
            continue
        with open(csv_path) as f:
            rows = list(csv.DictReader(f))
        pp_col = next((c for c in (rows[0] if rows else {}) if 'polsby' in c.lower()), None)
        if pp_col:
            all_pp.extend(float(r[pp_col]) for r in rows if r.get(pp_col))
    return sum(all_pp) / len(all_pp) if all_pp else None


def test_edge_weighted_vs_comparison_csv():
    """Use partitioner_comparison CSV if available — most accurate source."""
    if not COMPARISON_CSV.exists():
        pytest.skip(f'Comparison CSV not found: {COMPARISON_CSV}')

    with open(COMPARISON_CSV) as f:
        rows = list(csv.DictReader(f))

    # Find edge-weighted and unweighted mean PP columns
    pp_ew_col = next((c for c in rows[0] if 'edge' in c.lower() and 'pp' in c.lower()), None)
    pp_uw_col = next((c for c in rows[0] if 'unweight' in c.lower() and 'pp' in c.lower()), None)

    if not pp_ew_col or not pp_uw_col:
        pytest.skip(f'Cannot find PP columns in comparison CSV. Columns: {list(rows[0].keys())}')

    ew_vals = [float(r[pp_ew_col]) for r in rows if r.get(pp_ew_col)]
    uw_vals = [float(r[pp_uw_col]) for r in rows if r.get(pp_uw_col)]

    mean_ew = sum(ew_vals) / len(ew_vals)
    mean_uw = sum(uw_vals) / len(uw_vals)
    improvement = (mean_ew - mean_uw) / mean_uw

    assert improvement >= MIN_IMPROVEMENT, (
        f'Edge-weighted improvement only {improvement*100:.1f}%, expected ≥{MIN_IMPROVEMENT*100:.0f}%. '
        f'Edge-weighted PP: {mean_ew:.3f}, Unweighted PP: {mean_uw:.3f}'
    )


def test_v3_edge_weighted_absolute_floor():
    """V3 national mean PP for 2020 should be well above the unweighted floor of ~0.24."""
    v3_dir = OUTPUTS_ROOT / 'V3'
    mean_pp = load_pp_from_states(v3_dir, '2020')

    if mean_pp is None:
        pytest.skip('V3/2020 outputs not found')

    # Paper B.2: unweighted baseline ≈ 0.236, edge-weighted ≈ 0.367
    # A floor of 0.30 is well above unweighted and confirms edge weighting is active
    assert mean_pp >= 0.30, (
        f'V3 2020 national mean PP = {mean_pp:.3f}, expected ≥ 0.30. '
        f'This suggests edge weighting may not be working correctly.'
    )


def test_v3_beats_partitioning_stats_csv():
    """Cross-check against partitioning_statistics CSV if available."""
    stats_csv = OUTPUTS_ROOT / 'data' / '2020' / 'partitioning' / 'partitioning_statistics_2020.csv'
    if not stats_csv.exists():
        pytest.skip(f'Partitioning statistics CSV not found: {stats_csv}')

    v3_dir = OUTPUTS_ROOT / 'V3'
    mean_pp_v3 = load_pp_from_states(v3_dir, '2020')
    if mean_pp_v3 is None:
        pytest.skip('V3/2020 outputs not found')

    with open(stats_csv) as f:
        rows = list(csv.DictReader(f))

    # Check if there's a baseline (unweighted) column
    uw_col = next((c for c in (rows[0] if rows else {})
                   if 'unweight' in c.lower() and 'pp' in c.lower()), None)
    if not uw_col:
        pytest.skip('No unweighted PP column in partitioning statistics')

    uw_vals = [float(r[uw_col]) for r in rows if r.get(uw_col)]
    mean_uw = sum(uw_vals) / len(uw_vals)

    improvement = (mean_pp_v3 - mean_uw) / mean_uw
    assert improvement >= MIN_IMPROVEMENT, (
        f'V3 improvement over unweighted: {improvement*100:.1f}%, expected ≥{MIN_IMPROVEMENT*100:.0f}%'
    )
