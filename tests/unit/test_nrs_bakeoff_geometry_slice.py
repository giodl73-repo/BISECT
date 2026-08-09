from pathlib import Path
import math
import sys

import geopandas as gpd
import pytest
from shapely.geometry import MultiPolygon, Polygon


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from analyze_nrs_bakeoff_geometry_slice import (
    BakeoffError,
    dissolve_assignments,
    measure_geometry,
)


def test_square_metrics_obey_frozen_identities() -> None:
    metrics = measure_geometry(Polygon([(0, 0), (2, 0), (2, 2), (0, 2)]))

    assert metrics["area_m2"] == 4.0
    assert metrics["perimeter_m"] == 8.0
    assert metrics["polsby_popper"] == pytest.approx(math.pi / 4.0)
    assert metrics["convex_hull_ratio"] == 1.0
    assert metrics["schwartzberg"] == pytest.approx(
        1.0 / math.sqrt(metrics["polsby_popper"])
    )


def test_multipart_geometry_retains_all_components() -> None:
    geometry = MultiPolygon(
        [
            Polygon([(0, 0), (1, 0), (1, 1), (0, 1)]),
            Polygon([(3, 0), (4, 0), (4, 1), (3, 1)]),
        ]
    )

    metrics = measure_geometry(geometry)

    assert metrics["component_count"] == 2
    assert metrics["area_m2"] == 2.0
    assert metrics["perimeter_m"] == 8.0


def test_invalid_geometry_is_rejected_without_repair() -> None:
    bowtie = Polygon([(0, 0), (2, 2), (0, 2), (2, 0)])

    with pytest.raises(BakeoffError, match="invalid district geometry"):
        measure_geometry(bowtie)


def test_assignment_geometry_universe_mismatch_is_rejected() -> None:
    blocks = gpd.GeoDataFrame(
        {
            "GEOID20": ["a", "b"],
            "geometry": [
                Polygon([(0, 0), (1, 0), (1, 1), (0, 1)]),
                Polygon([(1, 0), (2, 0), (2, 1), (1, 1)]),
            ],
        },
        crs="EPSG:32130",
    )

    with pytest.raises(BakeoffError, match="assignment and geometry universes differ"):
        dissolve_assignments(blocks, {"a": 1})
