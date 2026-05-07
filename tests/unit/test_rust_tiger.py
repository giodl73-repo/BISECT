"""
L2 tests for redist_py.read_tiger_shp — TIGER shapefile reader via PyO3.

Verifies the Rust TIGER reader produces correct GEOIDs, WKB geometry,
and area attributes for known shapefiles.

All tests skip if data/ directory is not present (CI without TIGER data).
"""

import os
import pytest
from pathlib import Path

RUST_AVAILABLE = os.environ.get('REDIST_NO_RUST', '0') != '1'
try:
    import redist_py
    REDIST_PY_IMPORTABLE = True
except ImportError:
    REDIST_PY_IMPORTABLE = False

TIGER_VT = Path('data/2020/tiger/tracts/tl_2020_50_tract/tl_2020_50_tract.shp')

pytestmark = pytest.mark.skipif(
    not RUST_AVAILABLE or not REDIST_PY_IMPORTABLE,
    reason='redist_py not available'
)


@pytest.fixture(scope='module')
def vt_records():
    if not TIGER_VT.exists():
        pytest.skip('Vermont TIGER shapefile not found (data/ not present)')
    return redist_py.read_tiger_shp(str(TIGER_VT))


class TestTigerReaderVermont:

    def test_record_count(self, vt_records):
        assert len(vt_records) == 193, f"Vermont should have 193 tracts, got {len(vt_records)}"

    def test_geoid_length(self, vt_records):
        for geoid, _, _, _ in vt_records:
            assert len(geoid) == 11, f"GEOID {geoid!r} is not 11 chars"

    def test_geoid_state_prefix(self, vt_records):
        for geoid, _, _, _ in vt_records:
            assert geoid.startswith('50'), f"Vermont GEOID should start with '50', got {geoid}"

    def test_geoids_are_sorted(self, vt_records):
        geoids = [r[0] for r in vt_records]
        assert geoids == sorted(geoids), "Records should be sorted by GEOID"

    def test_geoids_are_unique(self, vt_records):
        geoids = [r[0] for r in vt_records]
        assert len(set(geoids)) == len(geoids), "GEOIDs must be unique"

    def test_wkb_is_bytes(self, vt_records):
        for geoid, wkb, _, _ in vt_records:
            assert isinstance(wkb, bytes), f"{geoid}: geometry_wkb should be bytes"

    def test_wkb_is_nonempty(self, vt_records):
        for geoid, wkb, _, _ in vt_records:
            assert len(wkb) > 0, f"{geoid}: WKB should not be empty"

    def test_wkb_little_endian_marker(self, vt_records):
        """WKB first byte = 1 (little-endian byte order)."""
        for geoid, wkb, _, _ in vt_records:
            assert wkb[0] == 1, f"{geoid}: WKB byte order should be little-endian (1)"

    def test_wkb_polygon_type(self, vt_records):
        """WKB bytes 1-4 = type 3 (Polygon) in little-endian."""
        import struct
        for geoid, wkb, _, _ in vt_records:
            wkb_type = struct.unpack_from('<I', wkb, 1)[0]
            assert wkb_type == 3, f"{geoid}: WKB type should be 3 (Polygon), got {wkb_type}"

    def test_aland_positive(self, vt_records):
        for geoid, _, aland, _ in vt_records:
            assert aland > 0, f"{geoid}: ALAND should be positive"

    def test_awater_nonnegative(self, vt_records):
        for geoid, _, _, awater in vt_records:
            assert awater >= 0, f"{geoid}: AWATER should be non-negative"

    def test_wkb_parseable_by_shapely(self, vt_records):
        """WKB output should be parseable by Shapely (integration parity check)."""
        try:
            from shapely.wkb import loads as wkb_loads
        except ImportError:
            pytest.skip('shapely not available for WKB parity check')

        for geoid, wkb, _, _ in vt_records[:10]:  # check first 10 for speed
            geom = wkb_loads(wkb)
            assert geom is not None, f"{geoid}: Shapely could not parse WKB"
            assert geom.geom_type == 'Polygon', f"{geoid}: expected Polygon, got {geom.geom_type}"
            assert geom.area > 0, f"{geoid}: polygon area should be positive"

    def test_matches_geopandas_geoid_list(self, vt_records):
        """GEOIDs must match GeoPandas reader for same file."""
        try:
            import geopandas as gpd
        except ImportError:
            pytest.skip('geopandas not available')
        gdf = gpd.read_file(str(TIGER_VT))
        py_geoids = sorted(gdf['GEOID'].tolist())
        rs_geoids = [r[0] for r in vt_records]  # already sorted
        assert rs_geoids == py_geoids, "Rust and Python GEOID lists must match exactly"
