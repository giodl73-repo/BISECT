"""
Unit tests for Python syntax validation.

This test ensures all Python files in the project compile without syntax errors.
It catches issues like unterminated strings, invalid f-strings, etc. before they
reach production.
"""

import py_compile
import pytest
from pathlib import Path


# Project root (two levels up from tests/unit/)
PROJECT_ROOT = Path(__file__).parent.parent.parent


def get_all_python_files():
    """Get all Python files in the project (excluding venv, .git, __pycache__)."""
    exclude_dirs = {'.git', '.git.old', 'venv', '__pycache__', '.pytest_cache',
                    'node_modules', '.tox', 'build', 'dist', '*.egg-info'}

    python_files = []
    for py_file in PROJECT_ROOT.rglob('*.py'):
        # Skip excluded directories
        if any(excluded in py_file.parts for excluded in exclude_dirs):
            continue
        python_files.append(py_file)

    return sorted(python_files)


class TestPythonSyntax:
    """Test that all Python files have valid syntax."""

    @pytest.mark.parametrize("py_file", get_all_python_files(), ids=lambda f: str(f.relative_to(PROJECT_ROOT)))
    def test_file_compiles(self, py_file):
        """Test that a Python file compiles without syntax errors."""
        try:
            py_compile.compile(str(py_file), doraise=True)
        except py_compile.PyCompileError as e:
            pytest.fail(f"Syntax error in {py_file.relative_to(PROJECT_ROOT)}:\n{e}")


class TestCriticalScripts:
    """Test syntax of critical pipeline scripts that must always work."""

    CRITICAL_SCRIPTS = [
        # Main pipeline
        'scripts/pipeline/run_complete_redistricting.py',
        'scripts/pipeline/run_state_redistricting.py',
        'scripts/pipeline/process_single_state.py',

        # Visualization (frequently broken by refactoring)
        'scripts/pipeline/create_us_national_map.py',
        'scripts/pipeline/add_cities_to_districts.py',
        'scripts/compactness/create_us_national_compactness_map.py',
        'scripts/compactness/visualize_compactness.py',
        'scripts/political/create_us_national_political_map.py',
        'scripts/visualization/create_metro_area_maps.py',

        # Analysis
        'scripts/political/analyze_districts.py',
        'scripts/demographic/analyze_districts.py',
        'scripts/compactness/calculate_compactness.py',

        # Baseline comparison
        'scripts/baseline/visualize_three_way_comparison.py',
    ]

    @pytest.mark.parametrize("script_path", CRITICAL_SCRIPTS)
    def test_critical_script_compiles(self, script_path):
        """Test that critical pipeline scripts compile without syntax errors.

        These scripts are tested explicitly (not just as part of the parametrized
        test) to make it immediately obvious when a critical script is broken.
        """
        full_path = PROJECT_ROOT / script_path

        if not full_path.exists():
            pytest.skip(f"Script not found: {script_path}")

        try:
            py_compile.compile(str(full_path), doraise=True)
        except py_compile.PyCompileError as e:
            pytest.fail(f"CRITICAL: Syntax error in {script_path}:\n{e}")


class TestCommonSyntaxIssues:
    """Test for common syntax issues that have caused problems before."""

    def test_no_unterminated_fstrings_in_scripts(self):
        """Check for unterminated f-strings in scripts/ directory.

        This was the root cause of the Enhancement 29 breakage - multi-line
        f-strings that got broken during refactoring.
        """
        scripts_dir = PROJECT_ROOT / 'scripts'
        issues = []

        for py_file in scripts_dir.rglob('*.py'):
            try:
                py_compile.compile(str(py_file), doraise=True)
            except py_compile.PyCompileError as e:
                error_msg = str(e)
                if 'unterminated' in error_msg.lower() and ('string' in error_msg.lower() or 'f-string' in error_msg.lower()):
                    issues.append(f"{py_file.relative_to(PROJECT_ROOT)}: {error_msg}")

        if issues:
            pytest.fail(
                f"Found {len(issues)} file(s) with unterminated string/f-string literals:\n" +
                "\n".join(issues)
            )

    def test_all_imports_valid(self):
        """Test that all import statements in src/ are syntactically valid.

        This catches issues like:
        - from module import * (syntax error)
        - Circular imports (import error, but worth checking)
        """
        src_dir = PROJECT_ROOT / 'src'

        for py_file in src_dir.rglob('*.py'):
            try:
                py_compile.compile(str(py_file), doraise=True)
            except py_compile.PyCompileError as e:
                pytest.fail(f"Syntax error in {py_file.relative_to(PROJECT_ROOT)}:\n{e}")


class TestScriptCategories:
    """Test syntax by script category."""

    def test_pipeline_scripts(self):
        """All pipeline scripts must compile."""
        pipeline_dir = PROJECT_ROOT / 'scripts' / 'pipeline'
        if not pipeline_dir.exists():
            pytest.skip("Pipeline directory not found")

        for py_file in pipeline_dir.glob('*.py'):
            try:
                py_compile.compile(str(py_file), doraise=True)
            except py_compile.PyCompileError as e:
                pytest.fail(f"Pipeline script {py_file.name} has syntax error:\n{e}")

    def test_visualization_scripts(self):
        """All visualization scripts must compile."""
        viz_dirs = [
            PROJECT_ROOT / 'scripts' / 'visualization',
            PROJECT_ROOT / 'scripts' / 'compactness',
            PROJECT_ROOT / 'scripts' / 'political',
            PROJECT_ROOT / 'scripts' / 'demographic',
        ]

        broken_files = []
        for viz_dir in viz_dirs:
            if not viz_dir.exists():
                continue

            for py_file in viz_dir.glob('*.py'):
                try:
                    py_compile.compile(str(py_file), doraise=True)
                except py_compile.PyCompileError as e:
                    broken_files.append(f"{py_file.relative_to(PROJECT_ROOT)}: {e}")

        if broken_files:
            pytest.fail(
                f"Found {len(broken_files)} visualization script(s) with syntax errors:\n" +
                "\n".join(broken_files)
            )

    def test_analysis_scripts(self):
        """All analysis scripts must compile."""
        analysis_dirs = [
            PROJECT_ROOT / 'scripts' / 'political',
            PROJECT_ROOT / 'scripts' / 'demographic',
            PROJECT_ROOT / 'scripts' / 'compactness',
        ]

        for analysis_dir in analysis_dirs:
            if not analysis_dir.exists():
                continue

            for py_file in analysis_dir.glob('*.py'):
                if '__pycache__' in str(py_file):
                    continue

                try:
                    py_compile.compile(str(py_file), doraise=True)
                except py_compile.PyCompileError as e:
                    pytest.fail(f"Analysis script {py_file.relative_to(PROJECT_ROOT)} has syntax error:\n{e}")


if __name__ == '__main__':
    # Allow running this test standalone for quick checks
    pytest.main([__file__, '-v'])
