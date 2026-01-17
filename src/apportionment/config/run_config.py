"""
Run configuration management.

Tracks algorithmic choices, pipeline settings, and system information
for experimental variants and reproducible redistricting runs.
"""

import json
import platform
import sys
from dataclasses import dataclass, field, asdict
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Dict, Any


@dataclass
class MetadataConfig:
    """Metadata about the redistricting run."""
    created: str
    version: str
    census_year: int
    election_year: int
    run_type: str  # "production", "experiment", or "test"
    scope: str  # "us" or "state"
    states: List[str] = field(default_factory=lambda: ["all"])
    experiment_name: Optional[str] = None
    description: str = ""


@dataclass
class AlgorithmConfig:
    """Algorithm configuration and parameters."""
    partition_mode: str  # "edge_weighted" or "unweighted"
    data_level: str  # "tract" or "block"


@dataclass
class PipelineConfig:
    """Pipeline execution settings."""
    skip_political: bool = False
    skip_demographic: bool = False
    skip_compactness: bool = False
    skip_metro: bool = False
    dpi: int = 150


@dataclass
class SystemConfig:
    """System information at run time."""
    python_version: str
    metis_version: str = "5.1.0"
    execution_time_seconds: Optional[int] = None
    hostname: str = ""


@dataclass
class RunConfig:
    """
    Complete configuration for a redistricting run.

    Tracks all algorithmic choices, pipeline settings, and system information
    to enable reproducible experiments and systematic variant comparison.
    """
    schema_version: str
    metadata: MetadataConfig
    algorithm: AlgorithmConfig
    pipeline: PipelineConfig
    system: SystemConfig

    @classmethod
    def create(
        cls,
        version: str,
        census_year: int,
        election_year: int,
        partition_mode: str,
        data_level: str = "tract",
        run_type: str = "production",
        scope: str = "us",
        states: List[str] = None,
        experiment_name: Optional[str] = None,
        description: str = "",
        **pipeline_kwargs
    ) -> 'RunConfig':
        """
        Create a RunConfig with sensible defaults.

        Args:
            version: Version identifier (e.g., "v1")
            census_year: Census year (2000, 2010, 2020)
            election_year: Election year for political data
            partition_mode: "edge_weighted" or "unweighted"
            data_level: "tract" or "block"
            run_type: "production", "experiment", or "test"
            scope: "us" or "state"
            states: List of state names or ["all"]
            experiment_name: Name for experiment runs
            description: Human-readable description
            **pipeline_kwargs: Pipeline settings (skip_political, dpi, etc.)

        Returns:
            RunConfig instance
        """
        if states is None:
            states = ["all"]

        metadata = MetadataConfig(
            created=datetime.now().isoformat(),
            version=version,
            census_year=census_year,
            election_year=election_year,
            run_type=run_type,
            scope=scope,
            states=states,
            experiment_name=experiment_name,
            description=description
        )

        algorithm = AlgorithmConfig(
            partition_mode=partition_mode,
            data_level=data_level
        )

        pipeline = PipelineConfig(**pipeline_kwargs)

        system = SystemConfig(
            python_version=f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}",
            hostname=platform.node()
        )

        return cls(
            schema_version="1.0",
            metadata=metadata,
            algorithm=algorithm,
            pipeline=pipeline,
            system=system
        )


def write_config(config: RunConfig, output_dir: Path) -> Path:
    """
    Write configuration to config.json in output directory.

    Args:
        config: RunConfig instance
        output_dir: Output directory path

    Returns:
        Path to written config.json file
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    config_path = output_dir / "config.json"

    # Convert to dict and write JSON
    config_dict = asdict(config)

    with open(config_path, 'w', encoding='utf-8') as f:
        json.dump(config_dict, f, indent=2)

    return config_path


def read_config(config_path: Path) -> RunConfig:
    """
    Read configuration from config.json file.

    Args:
        config_path: Path to config.json file

    Returns:
        RunConfig instance

    Raises:
        FileNotFoundError: If config file doesn't exist
        ValueError: If config is invalid
    """
    config_path = Path(config_path)

    if not config_path.exists():
        raise FileNotFoundError(f"Config file not found: {config_path}")

    with open(config_path, 'r', encoding='utf-8') as f:
        data = json.load(f)

    # Reconstruct nested dataclasses
    metadata = MetadataConfig(**data['metadata'])
    algorithm = AlgorithmConfig(**data['algorithm'])
    pipeline = PipelineConfig(**data['pipeline'])
    system = SystemConfig(**data['system'])

    return RunConfig(
        schema_version=data['schema_version'],
        metadata=metadata,
        algorithm=algorithm,
        pipeline=pipeline,
        system=system
    )


def validate_config(config: RunConfig) -> Dict[str, Any]:
    """
    Validate configuration completeness and correctness.

    Args:
        config: RunConfig instance

    Returns:
        Dict with 'valid' bool and 'errors' list
    """
    errors = []

    # Validate schema version
    if config.schema_version != "1.0":
        errors.append(f"Unknown schema version: {config.schema_version}")

    # Validate run_type
    valid_run_types = ["production", "experiment", "test"]
    if config.metadata.run_type not in valid_run_types:
        errors.append(f"Invalid run_type: {config.metadata.run_type}. Must be one of {valid_run_types}")

    # Validate scope
    valid_scopes = ["us", "state"]
    if config.metadata.scope not in valid_scopes:
        errors.append(f"Invalid scope: {config.metadata.scope}. Must be one of {valid_scopes}")

    # Validate partition_mode
    valid_partition_modes = ["edge_weighted", "unweighted"]
    if config.algorithm.partition_mode not in valid_partition_modes:
        errors.append(f"Invalid partition_mode: {config.algorithm.partition_mode}. Must be one of {valid_partition_modes}")

    # Validate data_level
    valid_data_levels = ["tract", "block"]
    if config.algorithm.data_level not in valid_data_levels:
        errors.append(f"Invalid data_level: {config.algorithm.data_level}. Must be one of {valid_data_levels}")

    # Validate census_year
    valid_census_years = [2000, 2010, 2020]
    if config.metadata.census_year not in valid_census_years:
        errors.append(f"Invalid census_year: {config.metadata.census_year}. Must be one of {valid_census_years}")

    return {
        'valid': len(errors) == 0,
        'errors': errors
    }
