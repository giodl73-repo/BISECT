"""
Unit tests for run configuration module.
"""

import json
import pytest
from pathlib import Path
import tempfile
import shutil

from apportionment.config import RunConfig, write_config, read_config, validate_config


class TestRunConfigCreation:
    """Test RunConfig creation and defaults."""

    def test_create_minimal_config(self):
        """Test creating config with minimal required parameters."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted"
        )

        assert config.schema_version == "1.0"
        assert config.metadata.version == "v1"
        assert config.metadata.census_year == 2020
        assert config.metadata.run_type == "production"
        assert config.metadata.scope == "us"
        assert config.metadata.states == ["all"]
        assert config.algorithm.partition_mode == "edge_weighted"
        assert config.algorithm.data_level == "tract"
        assert config.pipeline.dpi == 150

    def test_create_experiment_config(self):
        """Test creating experiment configuration."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="unweighted",
            run_type="experiment",
            experiment_name="tract_vs_block_2020",
            description="Testing tract vs block level data"
        )

        assert config.metadata.run_type == "experiment"
        assert config.metadata.experiment_name == "tract_vs_block_2020"
        assert config.metadata.description == "Testing tract vs block level data"

    def test_create_state_config(self):
        """Test creating single-state configuration."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted",
            scope="state",
            states=["california"]
        )

        assert config.metadata.scope == "state"
        assert config.metadata.states == ["california"]

    def test_create_test_config(self):
        """Test creating test/dev configuration."""
        config = RunConfig.create(
            version="test",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted",
            run_type="test",
            states=["vermont", "delaware"]
        )

        assert config.metadata.run_type == "test"
        assert config.metadata.states == ["vermont", "delaware"]

    def test_pipeline_kwargs(self):
        """Test pipeline configuration via kwargs."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted",
            skip_political=True,
            skip_demographic=True,
            dpi=300
        )

        assert config.pipeline.skip_political is True
        assert config.pipeline.skip_demographic is True
        assert config.pipeline.dpi == 300


class TestConfigIO:
    """Test config read/write operations."""

    def setup_method(self):
        """Create temporary directory for tests."""
        self.temp_dir = Path(tempfile.mkdtemp())

    def teardown_method(self):
        """Clean up temporary directory."""
        if self.temp_dir.exists():
            shutil.rmtree(self.temp_dir)

    def test_write_config(self):
        """Test writing config to JSON file."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted"
        )

        config_path = write_config(config, self.temp_dir)

        assert config_path.exists()
        assert config_path.name == "config.json"

        # Verify JSON structure
        with open(config_path, 'r') as f:
            data = json.load(f)

        assert data['schema_version'] == "1.0"
        assert data['metadata']['version'] == "v1"
        assert data['algorithm']['partition_mode'] == "edge_weighted"

    def test_read_config(self):
        """Test reading config from JSON file."""
        # Create and write config
        original_config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted",
            description="Test configuration"
        )

        config_path = write_config(original_config, self.temp_dir)

        # Read it back
        loaded_config = read_config(config_path)

        assert loaded_config.schema_version == original_config.schema_version
        assert loaded_config.metadata.version == original_config.metadata.version
        assert loaded_config.metadata.census_year == original_config.metadata.census_year
        assert loaded_config.algorithm.partition_mode == original_config.algorithm.partition_mode
        assert loaded_config.metadata.description == original_config.metadata.description

    def test_read_nonexistent_config(self):
        """Test reading config from nonexistent file raises error."""
        with pytest.raises(FileNotFoundError):
            read_config(self.temp_dir / "nonexistent.json")

    def test_roundtrip(self):
        """Test write and read roundtrip preserves data."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="unweighted",
            data_level="block",
            run_type="experiment",
            experiment_name="test_experiment",
            skip_political=True,
            dpi=300
        )

        config_path = write_config(config, self.temp_dir)
        loaded_config = read_config(config_path)

        # Compare key fields
        assert loaded_config.metadata.version == config.metadata.version
        assert loaded_config.metadata.run_type == config.metadata.run_type
        assert loaded_config.metadata.experiment_name == config.metadata.experiment_name
        assert loaded_config.algorithm.partition_mode == config.algorithm.partition_mode
        assert loaded_config.algorithm.data_level == config.algorithm.data_level
        assert loaded_config.pipeline.skip_political == config.pipeline.skip_political
        assert loaded_config.pipeline.dpi == config.pipeline.dpi


class TestConfigValidation:
    """Test configuration validation."""

    def test_valid_config(self):
        """Test validation of valid configuration."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted"
        )

        result = validate_config(config)
        assert result['valid'] is True
        assert len(result['errors']) == 0

    def test_invalid_run_type(self):
        """Test validation catches invalid run_type."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted"
        )
        config.metadata.run_type = "invalid_type"

        result = validate_config(config)
        assert result['valid'] is False
        assert any("run_type" in error for error in result['errors'])

    def test_invalid_partition_mode(self):
        """Test validation catches invalid partition_mode."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted"
        )
        config.algorithm.partition_mode = "invalid_mode"

        result = validate_config(config)
        assert result['valid'] is False
        assert any("partition_mode" in error for error in result['errors'])

    def test_invalid_data_level(self):
        """Test validation catches invalid data_level."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted"
        )
        config.algorithm.data_level = "invalid_level"

        result = validate_config(config)
        assert result['valid'] is False
        assert any("data_level" in error for error in result['errors'])

    def test_invalid_census_year(self):
        """Test validation catches invalid census_year."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted"
        )
        config.metadata.census_year = 2025

        result = validate_config(config)
        assert result['valid'] is False
        assert any("census_year" in error for error in result['errors'])

    def test_multiple_errors(self):
        """Test validation catches multiple errors."""
        config = RunConfig.create(
            version="v1",
            census_year=2020,
            election_year=2020,
            partition_mode="edge_weighted"
        )
        config.metadata.run_type = "invalid"
        config.algorithm.partition_mode = "invalid"

        result = validate_config(config)
        assert result['valid'] is False
        assert len(result['errors']) >= 2
