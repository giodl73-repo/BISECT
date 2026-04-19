"""
Unit tests for scripts.utils.subprocess_helpers module.
"""

import pytest
import sys
from argparse import Namespace
from pathlib import Path
from scripts.utils.subprocess_helpers import (
    build_pipeline_command,
    build_command_string,
    add_common_flags_to_command,
    get_python_executable,
)


class TestBuildPipelineCommand:
    """Test build_pipeline_command function."""

    def test_basic_command(self):
        """Test basic command building."""
        cmd = build_pipeline_command('scripts/test.py')

        assert isinstance(cmd, list)
        assert len(cmd) >= 2
        assert cmd[1] == 'scripts/test.py'

    def test_with_args_namespace(self):
        """Test command building with argparse namespace."""
        args = Namespace(year='2020', dpi=150, print_only=False, debug=False)
        cmd = build_pipeline_command('scripts/test.py', args)

        assert '--year' in cmd
        assert '2020' in cmd
        assert '--dpi' in cmd
        assert '150' in cmd

    def test_with_print_only_flag(self):
        """Test that print_only flag is added when True."""
        args = Namespace(year='2020', print_only=True, debug=False)
        cmd = build_pipeline_command('scripts/test.py', args)

        assert '--print-only' in cmd

    def test_without_print_only_flag(self):
        """Test that print_only flag is not added when False."""
        args = Namespace(year='2020', print_only=False, debug=False)
        cmd = build_pipeline_command('scripts/test.py', args)

        assert '--print-only' not in cmd

    def test_with_debug_flag(self):
        """Test that debug flag is added when True."""
        args = Namespace(debug=True)
        cmd = build_pipeline_command('scripts/test.py', args)

        assert '--debug' in cmd

    def test_with_force_flag(self):
        """Test that force flag is added when True."""
        args = Namespace(force=True)
        cmd = build_pipeline_command('scripts/test.py', args)

        assert '--force' in cmd

    def test_with_extra_args(self):
        """Test command building with extra arguments."""
        cmd = build_pipeline_command(
            'scripts/test.py',
            state='CA',
            version='v1',
            position=2
        )

        assert '--state' in cmd
        assert 'CA' in cmd
        assert '--version' in cmd
        assert 'v1' in cmd
        assert '--position' in cmd
        assert '2' in cmd

    def test_extra_args_with_underscores(self):
        """Test that underscores in argument names are converted to hyphens."""
        cmd = build_pipeline_command(
            'scripts/test.py',
            census_year='2020'
        )

        assert '--census-year' in cmd
        assert '2020' in cmd

    def test_boolean_extra_args(self):
        """Test boolean extra arguments."""
        cmd = build_pipeline_command(
            'scripts/test.py',
            skip_political=True,
            skip_demographic=False
        )

        assert '--skip-political' in cmd
        assert '--skip-demographic' not in cmd

    def test_none_values_are_skipped(self):
        """Test that None values are not added to command."""
        cmd = build_pipeline_command(
            'scripts/test.py',
            state=None,
            version='v1'
        )

        assert '--state' not in cmd
        assert '--version' in cmd

    def test_path_argument_is_converted_to_string(self):
        """Test that Path objects are converted to strings."""
        script_path = Path('scripts/test.py')
        cmd = build_pipeline_command(script_path)

        assert isinstance(cmd[1], str)


class TestBuildCommandString:
    """Test build_command_string function."""

    def test_returns_string(self):
        """Test that function returns a string."""
        cmd_str = build_command_string('scripts/test.py')

        assert isinstance(cmd_str, str)

    def test_string_contains_script_path(self):
        """Test that returned string contains script path."""
        cmd_str = build_command_string('scripts/test.py')

        assert 'scripts/test.py' in cmd_str

    def test_string_with_arguments(self):
        """Test string building with arguments."""
        cmd_str = build_command_string(
            'scripts/test.py',
            year='2020',
            state='CA'
        )

        assert 'scripts/test.py' in cmd_str
        assert '--year 2020' in cmd_str
        assert '--state CA' in cmd_str

    def test_string_is_space_separated(self):
        """Test that command parts are space-separated."""
        cmd_str = build_command_string(
            'scripts/test.py',
            year='2020'
        )

        # Should not have multiple consecutive spaces
        assert '  ' not in cmd_str


class TestAddCommonFlagsToCommand:
    """Test add_common_flags_to_command function."""

    def test_adds_print_only_flag(self):
        """Test that print_only flag is added."""
        cmd = ['python', 'script.py']
        args = Namespace(print_only=True, debug=False, force=False)

        add_common_flags_to_command(cmd, args)

        assert '--print-only' in cmd

    def test_does_not_add_print_only_when_false(self):
        """Test that print_only flag is not added when False."""
        cmd = ['python', 'script.py']
        args = Namespace(print_only=False, debug=False, force=False)

        add_common_flags_to_command(cmd, args)

        assert '--print-only' not in cmd

    def test_adds_debug_flag(self):
        """Test that debug flag is added."""
        cmd = ['python', 'script.py']
        args = Namespace(print_only=False, debug=True, force=False)

        add_common_flags_to_command(cmd, args)

        assert '--debug' in cmd

    def test_adds_force_flag(self):
        """Test that force flag is added."""
        cmd = ['python', 'script.py']
        args = Namespace(print_only=False, debug=False, force=True)

        add_common_flags_to_command(cmd, args)

        assert '--force' in cmd

    def test_adds_multiple_flags(self):
        """Test that multiple flags can be added."""
        cmd = ['python', 'script.py']
        args = Namespace(print_only=True, debug=True, force=True)

        add_common_flags_to_command(cmd, args)

        assert '--print-only' in cmd
        assert '--debug' in cmd
        assert '--force' in cmd

    def test_handles_missing_attributes(self):
        """Test that missing attributes are handled gracefully."""
        cmd = ['python', 'script.py']
        args = Namespace(print_only=True)  # Missing debug and force

        # Should not raise error
        add_common_flags_to_command(cmd, args)

        assert '--print-only' in cmd

    def test_returns_modified_list(self):
        """Test that function returns the modified command list."""
        cmd = ['python', 'script.py']
        args = Namespace(print_only=True, debug=False, force=False)

        result = add_common_flags_to_command(cmd, args)

        assert result is cmd
        assert '--print-only' in result

    def test_modifies_in_place(self):
        """Test that list is modified in-place."""
        cmd = ['python', 'script.py']
        original_id = id(cmd)
        args = Namespace(print_only=True, debug=False, force=False)

        add_common_flags_to_command(cmd, args)

        # Should be same object
        assert id(cmd) == original_id


class TestGetPythonExecutable:
    """Test get_python_executable function."""

    def test_returns_string(self):
        """Test that function returns a string."""
        python = get_python_executable()

        assert isinstance(python, str)

    def test_returns_executable_path(self):
        """Test that returned path is to Python executable."""
        python = get_python_executable()

        # Should contain 'python' (case-insensitive)
        assert 'python' in python.lower()

    def test_matches_sys_executable(self):
        """Test that returned value matches sys.executable."""
        python = get_python_executable()

        assert python == sys.executable


class TestCommandBuildingIntegration:
    """Test integration between command building functions."""

    def test_list_and_string_builders_produce_equivalent_output(self):
        """Test that list and string builders produce equivalent commands."""
        args = Namespace(year='2020', dpi=150, print_only=True, debug=False)

        cmd_list = build_pipeline_command('scripts/test.py', args, state='CA')
        cmd_string = build_command_string('scripts/test.py', args, state='CA')

        # String should be equivalent to space-joined list
        assert cmd_string == ' '.join(cmd_list)

    def test_complex_command_building(self):
        """Test building complex command with multiple arguments."""
        args = Namespace(
            year='2020',
            dpi=300,
            print_only=True,
            debug=True,
            force=False
        )

        cmd = build_pipeline_command(
            'scripts/pipeline/process_single_state.py',
            args,
            state='CA',
            version='v1',
            position=5
        )

        # Verify all components present
        assert 'scripts/pipeline/process_single_state.py' in cmd
        assert '--year' in cmd and '2020' in cmd
        assert '--dpi' in cmd and '300' in cmd
        assert '--print-only' in cmd
        assert '--debug' in cmd
        assert '--force' not in cmd
        assert '--state' in cmd and 'CA' in cmd
        assert '--version' in cmd and 'v1' in cmd
        assert '--position' in cmd and '5' in cmd
