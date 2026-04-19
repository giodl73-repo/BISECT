"""
Unit tests for scripts.utils.args module.
"""

import pytest
import argparse
from scripts.utils.args import (
    add_common_pipeline_args,
    add_common_state_args,
    add_common_visualization_args,
    add_common_analysis_args,
)


class TestAddCommonPipelineArgs:
    """Test add_common_pipeline_args function."""

    def test_adds_year_argument(self):
        """Test that --year argument is added."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)

        args = parser.parse_args(['--year', '2020'])
        assert args.year == '2020'

    def test_year_default_is_2020(self):
        """Test that --year defaults to 2020."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)

        args = parser.parse_args([])
        assert args.year == '2020'

    def test_year_choices_are_valid(self):
        """Test that only valid census years are accepted."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)

        # Valid years should work
        for year in ['2000', '2010', '2020']:
            args = parser.parse_args(['--year', year])
            assert args.year == year

        # Invalid year should fail
        with pytest.raises(SystemExit):
            parser.parse_args(['--year', '2030'])

    def test_adds_dpi_argument(self):
        """Test that --dpi argument is added."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)

        args = parser.parse_args(['--dpi', '300'])
        assert args.dpi == 300

    def test_dpi_default_is_150(self):
        """Test that --dpi defaults to 150."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)

        args = parser.parse_args([])
        assert args.dpi == 150

    def test_adds_print_only_flag(self):
        """Test that --print-only flag is added."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)

        args_with = parser.parse_args(['--print-only'])
        args_without = parser.parse_args([])

        assert args_with.print_only is True
        assert args_without.print_only is False

    def test_adds_debug_flag(self):
        """Test that --debug flag is added."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)

        args_with = parser.parse_args(['--debug'])
        args_without = parser.parse_args([])

        assert args_with.debug is True
        assert args_without.debug is False

    def test_adds_position_argument(self):
        """Test that --position argument is added."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)

        args = parser.parse_args(['--position', '5'])
        assert args.position == 5

    def test_position_default_is_1(self):
        """Test that --position defaults to 1."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)

        args = parser.parse_args([])
        assert args.position == 1

    def test_returns_parser(self):
        """Test that function returns the parser (for chaining)."""
        parser = argparse.ArgumentParser()
        result = add_common_pipeline_args(parser)

        assert result is parser


class TestAddCommonStateArgs:
    """Test add_common_state_args function."""

    def test_adds_state_argument(self):
        """Test that --state argument is added."""
        parser = argparse.ArgumentParser()
        add_common_state_args(parser)

        args = parser.parse_args(['--state', 'CA', '--version', 'v1'])
        assert args.state == 'CA'

    def test_adds_version_argument(self):
        """Test that --version argument is added and is required."""
        parser = argparse.ArgumentParser()
        add_common_state_args(parser)

        # Should work with --version
        args = parser.parse_args(['--version', 'v1'])
        assert args.version == 'v1'

        # Should fail without --version
        with pytest.raises(SystemExit):
            parser.parse_args([])

    def test_state_is_optional(self):
        """Test that --state is optional."""
        parser = argparse.ArgumentParser()
        add_common_state_args(parser)

        args = parser.parse_args(['--version', 'v1'])
        assert args.state is None

    def test_returns_parser(self):
        """Test that function returns the parser (for chaining)."""
        parser = argparse.ArgumentParser()
        result = add_common_state_args(parser)

        assert result is parser


class TestAddCommonVisualizationArgs:
    """Test add_common_visualization_args function."""

    def test_adds_scope_argument(self):
        """Test that --scope argument is added."""
        parser = argparse.ArgumentParser()
        add_common_visualization_args(parser)

        args = parser.parse_args(['--scope', 'national'])
        assert args.scope == 'national'

    def test_scope_default_is_state(self):
        """Test that --scope defaults to state."""
        parser = argparse.ArgumentParser()
        add_common_visualization_args(parser)

        args = parser.parse_args([])
        assert args.scope == 'state'

    def test_scope_choices_are_valid(self):
        """Test that only valid scopes are accepted."""
        parser = argparse.ArgumentParser()
        add_common_visualization_args(parser)

        # Valid scopes should work
        for scope in ['state', 'national']:
            args = parser.parse_args(['--scope', scope])
            assert args.scope == scope

        # Invalid scope should fail
        with pytest.raises(SystemExit):
            parser.parse_args(['--scope', 'invalid'])

    def test_adds_skip_rounds_flag(self):
        """Test that --skip-rounds flag is added."""
        parser = argparse.ArgumentParser()
        add_common_visualization_args(parser)

        args_with = parser.parse_args(['--skip-rounds'])
        args_without = parser.parse_args([])

        assert args_with.skip_rounds is True
        assert args_without.skip_rounds is False

    def test_adds_force_flag(self):
        """Test that --force flag is added."""
        parser = argparse.ArgumentParser()
        add_common_visualization_args(parser)

        args_with = parser.parse_args(['--force'])
        args_without = parser.parse_args([])

        assert args_with.force is True
        assert args_without.force is False


class TestAddCommonAnalysisArgs:
    """Test add_common_analysis_args function."""

    def test_adds_census_year_argument(self):
        """Test that --census-year argument is added."""
        parser = argparse.ArgumentParser()
        add_common_analysis_args(parser)

        args = parser.parse_args(['--census-year', '2020'])
        assert args.census_year == '2020'

    def test_census_year_choices_are_valid(self):
        """Test that only valid census years are accepted."""
        parser = argparse.ArgumentParser()
        add_common_analysis_args(parser)

        # Valid years should work
        for year in ['2000', '2010', '2020']:
            args = parser.parse_args(['--census-year', year])
            assert args.census_year == year

        # Invalid year should fail
        with pytest.raises(SystemExit):
            parser.parse_args(['--census-year', '2030'])

    def test_adds_election_year_argument(self):
        """Test that --election-year argument is added."""
        parser = argparse.ArgumentParser()
        add_common_analysis_args(parser)

        args = parser.parse_args(['--election-year', '2020'])
        assert args.election_year == '2020'

    def test_adds_skip_political_flag(self):
        """Test that --skip-political flag is added."""
        parser = argparse.ArgumentParser()
        add_common_analysis_args(parser)

        args_with = parser.parse_args(['--skip-political'])
        args_without = parser.parse_args([])

        assert args_with.skip_political is True
        assert args_without.skip_political is False

    def test_adds_skip_demographic_flag(self):
        """Test that --skip-demographic flag is added."""
        parser = argparse.ArgumentParser()
        add_common_analysis_args(parser)

        args_with = parser.parse_args(['--skip-demographic'])
        args_without = parser.parse_args([])

        assert args_with.skip_demographic is True
        assert args_without.skip_demographic is False


class TestArgumentChaining:
    """Test that argument functions can be chained together."""

    def test_chain_pipeline_and_state_args(self):
        """Test chaining pipeline and state arguments."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)
        add_common_state_args(parser)

        args = parser.parse_args([
            '--year', '2020',
            '--state', 'CA',
            '--version', 'v1',
            '--dpi', '300',
            '--print-only'
        ])

        assert args.year == '2020'
        assert args.state == 'CA'
        assert args.version == 'v1'
        assert args.dpi == 300
        assert args.print_only is True

    def test_chain_all_arg_functions(self):
        """Test chaining all argument functions together."""
        parser = argparse.ArgumentParser()
        add_common_pipeline_args(parser)
        add_common_state_args(parser)
        add_common_visualization_args(parser)
        add_common_analysis_args(parser)

        args = parser.parse_args([
            '--year', '2020',
            '--state', 'TX',
            '--version', 'v1',
            '--scope', 'state',
            '--census-year', '2020',
            '--election-year', '2020',
            '--print-only',
            '--force'
        ])

        # Verify all arguments are present
        assert args.year == '2020'
        assert args.state == 'TX'
        assert args.version == 'v1'
        assert args.scope == 'state'
        assert args.census_year == '2020'
        assert args.election_year == '2020'
        assert args.print_only is True
        assert args.force is True
