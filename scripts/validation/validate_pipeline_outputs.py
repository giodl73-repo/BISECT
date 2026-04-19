#!/usr/bin/env python3
"""
Validate pipeline outputs and identify missing artifacts.

Checks for expected outputs from the redistricting pipeline and reports
which scripts failed to generate their outputs. Useful for debugging
incomplete runs (e.g., missing maps in 2010/2000 census runs).

Usage:
    python scripts/validation/validate_pipeline_outputs.py --year 2010 --version v1
    python scripts/validation/validate_pipeline_outputs.py --year 2020 --version v3 --verbose
    python scripts/validation/validate_pipeline_outputs.py --year 2010 --version v1 --state california

Reports:
    - Brief console summary (always)
    - Detailed text report to outputs/us_{year}_{version}_validation.txt (always)
    - Optional CSV report with --csv flag
"""

import sys
import math
import os
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime
import glob


# Script-to-Output Mapping
# Maps each pipeline script to its expected outputs
PIPELINE_OUTPUTS = {
    # Core redistricting outputs (always required)
    "run_state_redistricting": {
        "outputs": [
            "data/final_assignments.pkl",
            "maps/all_districts.png"
        ],
        "location": "scripts/pipeline/run_state_redistricting.py",
        "scope": "per-state",
        "required": True,
        "condition": "Always runs"
    },
    "add_cities_to_districts": {
        "outputs": [
            "data/district_cities.csv",
            "maps/all_districts_with_cities.png"
        ],
        "location": "scripts/pipeline/add_cities_to_districts.py",
        "scope": "per-state",
        "required": True,
        "condition": "After data/final_assignments.pkl exists"
    },
    "create_final_district_summary": {
        "outputs": [
            "data/district_summary.csv",
            "data/rounds_hierarchy.csv"
        ],
        "location": "scripts/pipeline/create_final_district_summary.py",
        "scope": "per-state",
        "required": True,
        "condition": "After data/final_assignments.pkl exists"
    },
    "visualize_all_rounds": {
        "outputs": ["maps/rounds/round_{round_num:02d}.png"],  # Multiple files (zero-padded)
        "location": "scripts/pipeline/visualize_all_rounds.py",
        "scope": "per-state",
        "required": True,
        "condition": "After data/district_summary.csv exists"
    },
    "create_individual_district_maps": {
        "outputs": ["maps/districts/district_{district_num:02d}.png"],  # Multiple files (no city name)
        "location": "scripts/pipeline/visualize_individual_districts.py",
        "scope": "per-state",
        "required": True,
        "condition": "After data/final_assignments.pkl exists"
    },

    # Optional per-state analysis outputs
    "analyze_districts_political": {
        "outputs": [
            "political/district_political.csv",
            "political/rounds_political.csv"
        ],
        "location": "scripts/pipeline/analyze_districts.py",
        "scope": "per-state",
        "required": False,
        "condition": "--run-analysis AND year==2020 AND election data exists"
    },
    "visualize_partisan_lean_state": {
        "outputs": [
            "political/maps/partisan_lean.png"
        ],
        "location": "scripts/pipeline/visualize_partisan_lean.py",
        "scope": "per-state",
        "required": False,
        "condition": "--run-analysis AND year==2020"
    },
    "analyze_district_demographics": {
        "outputs": [
            "demographic/district_demographics.csv"
        ],
        "location": "scripts/pipeline/analyze_district_demographics.py",
        "scope": "per-state",
        "required": False,
        "condition": "--run-analysis AND demographic data exists"
    },
    "visualize_demographics_state": {
        "outputs": [
            "demographic/maps/diversity_index.png",
            "demographic/maps/gender_balance.png",
            "demographic/maps/majority_race.png"
        ],
        "location": "scripts/pipeline/visualize_district_demographics.py",
        "scope": "per-state",
        "required": False,
        "condition": "--run-analysis AND demographic data exists"
    },
    "visualize_compactness_state": {
        "outputs": [
            "compactness/maps/polsby_popper.png",
            "compactness/maps/reock.png"
        ],
        "location": "scripts/pipeline/visualize_compactness.py",
        "scope": "per-state",
        "required": False,
        "condition": "--run-analysis"
    },
    "create_metro_area_maps": {
        "outputs": [
            "maps/metros/"  # Directory with multiple PNG files
        ],
        "location": "scripts/pipeline/visualize_metro_areas.py",
        "scope": "per-state",
        "required": False,
        "condition": "--run-analysis AND state has metros"
    },

    # National aggregation outputs
    "create_us_aggregate": {
        "outputs": [
            "data/us_all_districts.csv",
            "data/us_district_summary.csv"
        ],
        "location": "scripts/pipeline/create_us_aggregate.py",
        "scope": "national",
        "required": True,
        "condition": "After all states complete"
    },
    "create_rounds_hierarchy": {
        "outputs": ["data/us_rounds_hierarchy.csv"],
        "location": "scripts/pipeline/create_us_rounds_hierarchy.py",
        "scope": "national",
        "required": True,
        "condition": "After all states complete"
    },
    "create_us_national_map": {
        "outputs": ["maps/us_all_districts.png"],
        "location": "scripts/pipeline/visualize_national_districts.py",
        "scope": "national",
        "required": True,
        "condition": "After data/us_all_districts.csv exists"
    },
    "create_us_national_rounds_progression": {
        "outputs": ["maps/rounds/round_{round_num:02d}.png"],  # 6 files (zero-padded)
        "location": "scripts/pipeline/visualize_national_rounds.py",
        "scope": "national",
        "required": True,
        "condition": "After data/us_rounds_hierarchy.csv exists"
    },
    "visualize_partisan_lean_national": {
        "outputs": ["maps/political/partisan_lean.png"],
        "location": "scripts/pipeline/create_us_national_political_map.py",
        "scope": "national",
        "required": False,
        "condition": "year==2020 AND --run-analysis"
    },
    "visualize_demographics_national": {
        "outputs": [
            "maps/demographic/majority_demographics.png"
        ],
        "location": "scripts/pipeline/create_us_national_demographic_map.py",
        "scope": "national",
        "required": False,
        "condition": "--run-analysis AND demographic data exists"
    },
    "visualize_compactness_national": {
        "outputs": [
            "maps/compactness/polsby_popper.png"
        ],
        "location": "scripts/pipeline/visualize_compactness.py",
        "scope": "national",
        "required": False,
        "condition": "--run-analysis"
    },
    "generate_dashboard": {
        "outputs": ["index.html"],
        "location": "scripts/web/generate_dashboard.py",
        "scope": "national",
        "required": True,
        "condition": "Final step after all aggregation"
    }
}


def load_state_config(year: str) -> Dict[str, Dict[str, Any]]:
    """Load state configuration for the given census year."""
    sys.path.insert(0, str(Path(__file__).parent.parent.parent))

    if year == '2020':
        from scripts.config_2020 import STATE_CONFIG_2020
        return STATE_CONFIG_2020
    elif year == '2010':
        from scripts.config_2010 import STATE_CONFIG_2010
        return STATE_CONFIG_2010
    elif year == '2000':
        from scripts.config_2000 import STATE_CONFIG_2000
        return STATE_CONFIG_2000
    else:
        raise ValueError(f"Unsupported year: {year}")


def calculate_expected_rounds(num_districts: int) -> int:
    """Calculate expected number of round files based on district count."""
    # Recursive bisection creates log2(n) rounds
    return math.ceil(math.log2(num_districts))


def check_file_exists(file_path: Path) -> bool:
    """Check if file exists, supporting glob patterns."""
    if '*' in str(file_path) or '?' in str(file_path):
        # Glob pattern
        matches = glob.glob(str(file_path))
        return len(matches) > 0
    else:
        return file_path.exists()


def validate_state_outputs(
    state_dir: Path,
    state_code: str,
    state_name: str,
    num_districts: int,
    year: str,
    check_optional: bool = False,
    verbose: bool = False
) -> Dict[str, Any]:
    """
    Validate outputs for a single state.

    Returns dict with:
        - state: lowercase state name
        - state_code: two-letter code
        - num_districts: expected districts
        - results: list of check results
        - summary: aggregate stats
    """
    results = []
    expected_rounds = calculate_expected_rounds(num_districts)

    # Check each script's outputs
    for script_name, script_config in PIPELINE_OUTPUTS.items():
        if script_config['scope'] != 'per-state':
            continue

        # Skip optional checks unless requested
        if not script_config['required'] and not check_optional:
            continue

        # Special handling for single-district states
        if num_districts == 1:
            # Skip round-related outputs for single-district states (no bisection needed)
            if script_name in ['visualize_all_rounds']:
                continue
            # rounds_hierarchy.csv is not generated for single-district states
            if 'rounds_hierarchy.csv' in script_config['outputs']:
                continue

        for output_pattern in script_config['outputs']:
            # Handle multi-file patterns
            if 'round_num' in output_pattern:
                # Check all round files (format: round_N_REGIONS_regions.png)
                # Last round has exactly num_districts regions, earlier rounds have 2^N
                for round_num in range(1, expected_rounds + 1):
                    round_regions = min(2 ** round_num, num_districts)
                    file_path = state_dir / output_pattern.format(
                        round_num=round_num,
                        round_regions=round_regions
                    )
                    exists = file_path.exists()
                    results.append({
                        'file': str(file_path.relative_to(state_dir)),
                        'exists': exists,
                        'script': script_name,
                        'required': script_config['required'],
                        'condition': script_config['condition']
                    })
            elif 'district_num' in output_pattern:
                # Check district maps - use glob since we can't predict city names
                # Pattern: maps/districts/district_NN_*.png
                districts_dir = state_dir / 'maps' / 'districts'
                if districts_dir.exists():
                    district_files = sorted(districts_dir.glob('district_*.png'))
                    # Check if we have expected number of district files
                    expected_count = num_districts
                    actual_count = len(district_files)

                    # Report summary instead of individual files
                    results.append({
                        'file': f'maps/districts/ ({actual_count}/{expected_count} files)',
                        'exists': actual_count == expected_count,
                        'script': script_name,
                        'required': script_config['required'],
                        'condition': script_config['condition']
                    })
                else:
                    results.append({
                        'file': 'maps/districts/',
                        'exists': False,
                        'script': script_name,
                        'required': script_config['required'],
                        'condition': script_config['condition']
                    })
            elif output_pattern.endswith('/'):
                # Directory check (e.g., maps/metros/)
                dir_path = state_dir / output_pattern
                if dir_path.exists():
                    files = list(dir_path.glob('*.png'))
                    results.append({
                        'file': f'{output_pattern} ({len(files)} files)',
                        'exists': len(files) > 0,
                        'script': script_name,
                        'required': script_config['required'],
                        'condition': script_config['condition']
                    })
                else:
                    results.append({
                        'file': output_pattern,
                        'exists': False,
                        'script': script_name,
                        'required': script_config['required'],
                        'condition': script_config['condition']
                    })
            else:
                # Single file or simple pattern
                # Check if pattern needs formatting
                if '{' in output_pattern:
                    # Pattern has placeholders - format it
                    state_name_lowercase = state_name.lower().replace(' ', '_')
                    state_name_spaces = state_name.lower()

                    # Try underscore version first
                    file_path = state_dir / output_pattern.format(
                        state_name=state_name_lowercase,
                        num_districts=num_districts,
                        census_year=year
                    )
                    exists = check_file_exists(file_path)

                    # If not found, try space version
                    if not exists and '{state_name}' in output_pattern:
                        file_path_alt = state_dir / output_pattern.format(
                            state_name=state_name_spaces,
                            num_districts=num_districts,
                            census_year=year
                        )
                        exists = check_file_exists(file_path_alt)
                        if exists:
                            file_path = file_path_alt
                else:
                    # Simple path with no placeholders
                    file_path = state_dir / output_pattern
                    exists = check_file_exists(file_path)

                results.append({
                    'file': str(file_path.relative_to(state_dir)) if file_path.is_relative_to(state_dir) else str(file_path),
                    'exists': exists,
                    'script': script_name,
                    'required': script_config['required'],
                    'condition': script_config['condition']
                })

    # Calculate summary stats
    total_checks = len(results)
    passed = sum(1 for r in results if r['exists'])
    failed = total_checks - passed
    completion_pct = (passed / total_checks * 100) if total_checks > 0 else 0

    # Count required vs optional failures
    required_checks = [r for r in results if r['required']]
    required_failed = sum(1 for r in required_checks if not r['exists'])

    return {
        'state': state_name.lower().replace(' ', '_'),
        'state_code': state_code,
        'state_name': state_name,
        'num_districts': num_districts,
        'results': results,
        'summary': {
            'total_checks': total_checks,
            'passed': passed,
            'failed': failed,
            'completion_pct': completion_pct,
            'required_failed': required_failed
        }
    }


def validate_national_outputs(
    output_dir: Path,
    year: str,
    check_optional: bool = False,
    verbose: bool = False
) -> Dict[str, Any]:
    """
    Validate national-level outputs.

    Returns dict with results and summary.
    """
    results = []

    # Check each national script's outputs
    for script_name, script_config in PIPELINE_OUTPUTS.items():
        if script_config['scope'] != 'national':
            continue

        # Skip optional checks unless requested
        if not script_config['required'] and not check_optional:
            continue

        for output_pattern in script_config['outputs']:
            # Handle multi-file patterns
            if 'round_num' in output_pattern:
                # Check maps/rounds directory
                rounds_dir = output_dir / 'maps' / 'rounds'
                if rounds_dir.exists():
                    round_files = sorted(rounds_dir.glob('round_*.png'))
                    for round_file in round_files:
                        results.append({
                            'file': str(round_file.relative_to(output_dir)),
                            'exists': True,
                            'script': script_name,
                            'required': script_config['required'],
                            'condition': script_config['condition']
                        })
                else:
                    # Expected but missing
                    results.append({
                        'file': 'maps/rounds/round_*.png',
                        'exists': False,
                        'script': script_name,
                        'required': script_config['required'],
                        'condition': script_config['condition']
                    })
            else:
                # Single file
                file_path = output_dir / output_pattern
                exists = file_path.exists()
                results.append({
                    'file': output_pattern,
                    'exists': exists,
                    'script': script_name,
                    'required': script_config['required'],
                    'condition': script_config['condition']
                })

    # Calculate summary
    total_checks = len(results)
    passed = sum(1 for r in results if r['exists'])
    failed = total_checks - passed
    completion_pct = (passed / total_checks * 100) if total_checks > 0 else 0

    required_checks = [r for r in results if r['required']]
    required_failed = sum(1 for r in required_checks if not r['exists'])

    return {
        'results': results,
        'summary': {
            'total_checks': total_checks,
            'passed': passed,
            'failed': failed,
            'completion_pct': completion_pct,
            'required_failed': required_failed
        }
    }


def categorize_failures_by_script(
    state_results: List[Dict[str, Any]],
    national_results: Dict[str, Any]
) -> Dict[str, Dict[str, Any]]:
    """
    Group missing files by the script that should have generated them.

    Returns dict mapping script_name -> {states, files, file_count, state_count}
    """
    failures = {}

    # Process state-level failures
    for state_result in state_results:
        state_code = state_result['state_code']
        for check in state_result['results']:
            if not check['exists']:
                script_name = check['script']
                if script_name not in failures:
                    failures[script_name] = {
                        'states': set(),
                        'files': [],
                        'file_count': 0,
                        'state_count': 0,
                        'script_location': None,
                        'condition': check['condition']
                    }

                failures[script_name]['states'].add(state_code)
                failures[script_name]['files'].append({
                    'state': state_code,
                    'file': check['file']
                })
                failures[script_name]['file_count'] += 1

                # Get script location from PIPELINE_OUTPUTS
                if failures[script_name]['script_location'] is None:
                    if script_name in PIPELINE_OUTPUTS:
                        failures[script_name]['script_location'] = PIPELINE_OUTPUTS[script_name]['location']

    # Process national-level failures
    for check in national_results['results']:
        if not check['exists']:
            script_name = check['script']
            if script_name not in failures:
                failures[script_name] = {
                    'states': set(),
                    'files': [],
                    'file_count': 0,
                    'state_count': 0,
                    'script_location': None,
                    'condition': check['condition']
                }

            failures[script_name]['files'].append({
                'state': 'NATIONAL',
                'file': check['file']
            })
            failures[script_name]['file_count'] += 1

            if failures[script_name]['script_location'] is None:
                if script_name in PIPELINE_OUTPUTS:
                    failures[script_name]['script_location'] = PIPELINE_OUTPUTS[script_name]['location']

    # Calculate state counts
    for script_name, info in failures.items():
        info['state_count'] = len(info['states'])

    return failures


def print_brief_summary(
    output_dir: Path,
    state_results: List[Dict[str, Any]],
    national_results: Dict[str, Any],
    script_failures: Dict[str, Dict[str, Any]],
    report_file: Path
):
    """Print brief validation summary to console."""
    total_states = len(state_results)
    complete_states = sum(1 for s in state_results if s['summary']['completion_pct'] == 100)
    partial_states = sum(1 for s in state_results if 50 <= s['summary']['completion_pct'] < 100)
    failed_states = sum(1 for s in state_results if s['summary']['completion_pct'] < 50)

    print("=" * 60)
    print("  Pipeline Validation Summary")
    print(f"  Run: {output_dir}")
    print("=" * 60)

    if complete_states == total_states:
        print(f"[OK] All {total_states} states complete (100%)")
    else:
        print(f"[OK] {complete_states} states complete ({complete_states/total_states*100:.0f}%)")
        if partial_states:
            print(f"[WARN] {partial_states} states partially complete ({partial_states/total_states*100:.0f}%)")
        if failed_states:
            print(f"[FAIL] {failed_states} states with missing core outputs ({failed_states/total_states*100:.0f}%)")

    # National outputs
    national_summary = national_results['summary']
    print(f"\nNational outputs: {national_summary['passed']}/{national_summary['total_checks']} present ({national_summary['completion_pct']:.0f}%)")

    # Top 3 failing scripts
    if script_failures:
        top_failures = sorted(
            script_failures.items(),
            key=lambda x: x[1]['file_count'],
            reverse=True
        )[:3]

        print("\nTop script failures:")
        for script, info in top_failures:
            script_short = script.replace('_', ' ').title()
            if info['state_count'] > 0:
                print(f"  - {script_short} ({info['state_count']} states, {info['file_count']} files)")
            else:
                print(f"  - {script_short} ({info['file_count']} files)")
    else:
        print("\n[OK] No failures detected!")

    print(f"\nDetailed report: {report_file}")


def write_detailed_report(
    output_dir: Path,
    state_results: List[Dict[str, Any]],
    national_results: Dict[str, Any],
    script_failures: Dict[str, Dict[str, Any]],
    output_file: Path,
    year: str,
    version: str
):
    """Write comprehensive validation report to text file."""

    # Calculate overall stats
    total_checks = sum(s['summary']['total_checks'] for s in state_results) + national_results['summary']['total_checks']
    total_passed = sum(s['summary']['passed'] for s in state_results) + national_results['summary']['passed']
    total_failed = total_checks - total_passed

    total_states = len(state_results)
    complete_states = sum(1 for s in state_results if s['summary']['completion_pct'] == 100)
    partial_states = sum(1 for s in state_results if 50 <= s['summary']['completion_pct'] < 100)
    failed_states = sum(1 for s in state_results if s['summary']['completion_pct'] < 50)

    with open(output_file, 'w', encoding='utf-8') as f:
        f.write("=" * 70 + "\n")
        f.write("PIPELINE OUTPUT VALIDATION REPORT\n")
        f.write("=" * 70 + "\n\n")

        f.write(f"Run: {output_dir}\n")
        f.write(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"Year: {year}, Version: {version}\n")
        f.write(f"Total Checks: {total_checks:,}\n")
        f.write(f"Passed: {total_passed:,} ({total_passed/total_checks*100:.1f}%)\n")
        f.write(f"Failed: {total_failed:,} ({total_failed/total_checks*100:.1f}%)\n\n")

        # Per-state validation
        f.write("=" * 70 + "\n")
        f.write("PER-STATE VALIDATION\n")
        f.write("=" * 70 + "\n\n")

        f.write(f"States Checked: {total_states}\n")
        f.write(f"  Fully Complete: {complete_states} ({complete_states/total_states*100:.1f}%)\n")
        f.write(f"  Partially Complete: {partial_states} ({partial_states/total_states*100:.1f}%)\n")
        f.write(f"  Missing Core Outputs: {failed_states} ({failed_states/total_states*100:.1f}%)\n\n")

        # Show states with missing outputs
        states_with_failures = [s for s in state_results if s['summary']['failed'] > 0]
        if states_with_failures:
            for state in sorted(states_with_failures, key=lambda s: s['summary']['completion_pct']):
                f.write("-" * 70 + "\n")
                f.write(f"{state['state_name'].upper()} ({state['summary']['completion_pct']:.1f}% complete)\n")
                f.write("-" * 70 + "\n")
                f.write(f"Expected: {state['summary']['total_checks']} outputs\n")
                f.write(f"Found: {state['summary']['passed']}\n")
                f.write(f"Missing: {state['summary']['failed']}\n\n")

                # List missing files
                missing_files = [r for r in state['results'] if not r['exists']]
                if missing_files:
                    f.write("Missing Files:\n")
                    for check in missing_files:
                        f.write(f"  [X] {check['file']}\n")
                        script_loc = PIPELINE_OUTPUTS.get(check['script'], {}).get('location', check['script'])
                        f.write(f"    Script: {script_loc}\n")
                        f.write(f"    Condition: {check['condition']}\n")
                        if not check['required']:
                            f.write(f"    Required: No (optional analysis)\n")
                        f.write("\n")
                f.write("\n")
        else:
            f.write("[OK] All states have complete required outputs!\n\n")

        # National validation
        f.write("=" * 70 + "\n")
        f.write("NATIONAL VALIDATION\n")
        f.write("=" * 70 + "\n\n")

        national_summary = national_results['summary']
        f.write(f"Expected: {national_summary['total_checks']} files\n")
        f.write(f"Found: {national_summary['passed']}\n")
        f.write(f"Missing: {national_summary['failed']}\n\n")

        for check in national_results['results']:
            if check['exists']:
                f.write(f"[OK] {check['file']}\n")
            else:
                f.write(f"[X] {check['file']}")
                script_loc = PIPELINE_OUTPUTS.get(check['script'], {}).get('location', check['script'])
                f.write(f" [{script_loc}]\n")
        f.write("\n")

        # Scripts with failures
        if script_failures:
            f.write("=" * 70 + "\n")
            f.write("SCRIPTS WITH FAILURES\n")
            f.write("=" * 70 + "\n\n")

            for script_name in sorted(script_failures.keys()):
                info = script_failures[script_name]
                f.write(f"{info['script_location'] or script_name}:\n")
                if info['state_count'] > 0:
                    states_list = ', '.join(sorted(info['states']))
                    f.write(f"  States Affected: {info['state_count']} ({states_list})\n")
                f.write(f"  Files Missing: {info['file_count']}\n")
                f.write(f"  Condition: {info['condition']}\n\n")

        # Recommended actions
        f.write("=" * 70 + "\n")
        f.write("RECOMMENDED ACTIONS\n")
        f.write("=" * 70 + "\n\n")

        if script_failures:
            # Group by scope
            state_scripts = {k: v for k, v in script_failures.items()
                           if PIPELINE_OUTPUTS.get(k, {}).get('scope') == 'per-state'}
            national_scripts = {k: v for k, v in script_failures.items()
                              if PIPELINE_OUTPUTS.get(k, {}).get('scope') == 'national'}

            if state_scripts:
                f.write("1. Re-run per-state scripts for affected states:\n")
                for script_name in sorted(state_scripts.keys()):
                    info = script_failures[script_name]
                    script_loc = info['script_location'] or script_name
                    f.write(f"   python {script_loc} --year {year} --version {version} --force\n")
                f.write("\n")

            if national_scripts:
                f.write("2. Re-run national post-processing:\n")
                for script_name in sorted(national_scripts.keys()):
                    info = script_failures[script_name]
                    script_loc = info['script_location'] or script_name
                    f.write(f"   python {script_loc} --year {year} --version {version}\n")
                f.write("\n")

            f.write("3. For detailed per-file report, run with --verbose:\n")
            f.write(f"   python scripts/validation/validate_pipeline_outputs.py --year {year} --version {version} --verbose\n\n")
        else:
            f.write("[OK] No actions needed - all outputs present!\n\n")

        f.write("=" * 70 + "\n")


def write_state_report(
    state_result: Dict[str, Any],
    output_file: Path,
    year: str
):
    """Write validation report for a single state."""
    state_code = state_result['state_code']
    state_name = state_result['state_name']
    summary = state_result['summary']

    with open(output_file, 'w', encoding='utf-8') as f:
        f.write("=" * 70 + "\n")
        f.write(f"{state_name} ({state_code}) - PIPELINE OUTPUT VALIDATION\n")
        f.write("=" * 70 + "\n\n")
        f.write(f"Census Year: {year}\n")
        f.write(f"Validation Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")

        # Summary
        f.write("SUMMARY\n")
        f.write("-" * 70 + "\n")
        f.write(f"Total files checked: {summary['total_checks']}\n")
        f.write(f"Files present: {summary['passed']}\n")
        f.write(f"Files missing: {summary['failed']}\n")
        f.write(f"Required files failed: {summary['required_failed']}\n")
        f.write(f"Completion: {summary['completion_pct']:.1f}%\n\n")

        # Status
        if summary['required_failed'] == 0:
            f.write("[OK] All required outputs present!\n\n")
        else:
            f.write(f"[FAILED] {summary['required_failed']} required outputs missing\n\n")

        # Details
        f.write("DETAILED RESULTS\n")
        f.write("-" * 70 + "\n\n")

        missing_files = [c for c in state_result['results'] if not c['exists']]
        present_files = [c for c in state_result['results'] if c['exists']]

        if missing_files:
            f.write(f"Missing Files ({len(missing_files)}):\n")
            for check in missing_files:
                required_str = "[REQUIRED]" if check['required'] else "[OPTIONAL]"
                f.write(f"  {required_str} {check['file']}\n")
                f.write(f"    Script: {check['script']}\n")
            f.write("\n")

        if present_files:
            f.write(f"Present Files ({len(present_files)}):\n")
            for check in present_files:
                f.write(f"  [OK] {check['file']}\n")
            f.write("\n")

        f.write("=" * 70 + "\n")


def write_csv_report(
    state_results: List[Dict[str, Any]],
    national_results: Dict[str, Any],
    output_file: Path
):
    """Write detailed validation results to CSV format."""
    import csv

    with open(output_file, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(['state', 'file_path', 'exists', 'required', 'script_name', 'script_location', 'condition'])

        # Write state results
        for state in state_results:
            state_code = state['state_code']
            for check in state['results']:
                script_loc = PIPELINE_OUTPUTS.get(check['script'], {}).get('location', check['script'])
                writer.writerow([
                    state_code,
                    check['file'],
                    check['exists'],
                    check['required'],
                    check['script'],
                    script_loc,
                    check['condition']
                ])

        # Write national results
        for check in national_results['results']:
            script_loc = PIPELINE_OUTPUTS.get(check['script'], {}).get('location', check['script'])
            writer.writerow([
                'NATIONAL',
                check['file'],
                check['exists'],
                check['required'],
                check['script'],
                script_loc,
                check['condition']
            ])


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description='Validate pipeline outputs and identify missing artifacts'
    )
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year')
    parser.add_argument('--version', type=str, default='v1',
                       help='Run version (e.g., v1, v2, v3)')
    parser.add_argument('--output-dir', type=str,
                       help='Path to us_{year}_{version} directory (auto-detected if not specified)')
    parser.add_argument('--state', type=str,
                       help='Validate single state only (state code, e.g., CA)')
    parser.add_argument('--verbose', action='store_true',
                       help='Show all checks in console (for debugging validator)')
    parser.add_argument('--csv', action='store_true',
                       help='Also save CSV format report (in addition to text)')
    parser.add_argument('--check-optional', action='store_true',
                       help='Check optional analysis outputs (political, demographic, compactness)')
    parser.add_argument('--force', action='store_true',
                       help='Force re-validation (ignore cached reports)')

    args = parser.parse_args()

    # Determine output directory
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        output_dir = Path(f'outputs/us_{args.year}_{args.version}')

    if not output_dir.exists():
        print(f"ERROR: Output directory not found: {output_dir}")
        return 1

    # Check if validation report already exists (skip logic)
    report_file = output_dir / "validation.txt"
    if report_file.exists() and not args.force:
        print(f"Validation report already exists: {report_file}")
        print("Use --force to regenerate")
        return 0

    # Load state configuration
    try:
        state_config = load_state_config(args.year)
    except Exception as e:
        print(f"ERROR: Failed to load state config: {e}")
        return 1

    # Validate states
    state_results = []
    states_dir = output_dir / 'states'

    if args.state:
        # Single state validation
        state_code = args.state.upper()
        if state_code not in state_config:
            print(f"ERROR: Unknown state code: {state_code}")
            return 1

        config = state_config[state_code]
        state_dir = states_dir / config['name'].lower().replace(' ', '_')

        if not state_dir.exists():
            print(f"ERROR: State directory not found: {state_dir}")
            return 1

        result = validate_state_outputs(
            state_dir,
            state_code,
            config['name'],
            config['districts'],
            args.year,
            args.check_optional,
            args.verbose
        )
        state_results = [result]

        # Write per-state validation report
        state_report_file = state_dir / "validation.txt"
        write_state_report(result, state_report_file, args.year)
    else:
        # All states validation
        print(f"Validating {len(state_config)} states...")

        for state_code in sorted(state_config.keys()):
            config = state_config[state_code]
            state_dir = states_dir / config['name'].lower().replace(' ', '_')

            if not state_dir.exists():
                if args.verbose:
                    print(f"  SKIP: {state_code} - directory not found")
                continue

            result = validate_state_outputs(
                state_dir,
                state_code,
                config['name'],
                config['districts'],
                args.year,
                args.check_optional,
                args.verbose
            )
            state_results.append(result)

            # Write per-state validation report
            state_report_file = state_dir / "validation.txt"
            write_state_report(result, state_report_file, args.year)

    # Validate national outputs
    national_results = validate_national_outputs(
        output_dir,
        args.year,
        args.check_optional,
        args.verbose
    )

    # Categorize failures by script
    script_failures = categorize_failures_by_script(state_results, national_results)

    # Print brief summary to console
    print()
    print_brief_summary(
        output_dir,
        state_results,
        national_results,
        script_failures,
        report_file
    )

    # Write detailed text report
    write_detailed_report(
        output_dir,
        state_results,
        national_results,
        script_failures,
        report_file,
        args.year,
        args.version
    )

    # Optionally write CSV report
    if args.csv:
        csv_file = output_dir / "validation.csv"
        write_csv_report(state_results, national_results, csv_file)
        print(f"CSV report: {csv_file}")

    # Return exit code based on validation results
    # 0 = all outputs present, 1 = some outputs missing
    total_failed = sum(s['summary']['failed'] for s in state_results) + national_results['summary']['failed']
    return 0 if total_failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
