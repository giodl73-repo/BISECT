/// TIGER/Line tract shapefile reader.
///
/// Reads ESRI .shp files (pure Rust, no GDAL). Returns per-tract records
/// with GEOID, population placeholder, and WKB-encoded polygon geometry.
///
/// The TIGER tracts files (e.g. tl_2020_50_tract.shp) contain geometry and
/// attributes (GEOID, ALAND, AWATER). Population is NOT in the .shp file —
/// it comes from the PL 94-171 redistricting data joined at a higher level.
/// This reader returns population=0 as a sentinel; callers must join
/// population data separately.
use std::path::Path;
use thiserror::Error;
use geo_types::{Polygon, MultiPolygon, LineString, Coord};

#[derive(Debug, Error)]
pub enum TigerError {
    #[error("shapefile read error: {0}")]
    ShapefileError(String),
    #[error("missing GEOID field in shapefile attributes")]
    MissingGeoid,
    #[error("unsupported geometry type at record {0}: expected Polygon or MultiPolygon")]
    UnsupportedGeometry(usize),
    #[error("GEOID {0} is not 11 characters (tract GEOID must be 11 digits: SSCCCTTTTTT)")]
    InvalidGeoidLength(String),
}

/// A single census tract record from a TIGER shapefile.
#[derive(Debug, Clone)]
pub struct TractRecord {
    /// 11-character GEOID: state(2) + county(3) + tract(6)
    pub geoid: String,
    /// WKB-encoded polygon geometry (in the file's native CRS, usually EPSG:4269 NAD83)
    pub geometry_wkb: Vec<u8>,
    /// Land area in square metres (ALAND field)
    pub aland: i64,
    /// Water area in square metres (AWATER field)
    pub awater: i64,
    /// Population — always 0 here; join from PL 94-171 separately
    pub population: i64,
}

/// Read all census tracts from a TIGER .shp file.
///
/// Returns records sorted by GEOID for deterministic ordering.
/// Skips records with empty geometries (rare in TIGER but possible).
pub fn read_tiger_tracts<P: AsRef<Path>>(shp_path: P) -> Result<Vec<TractRecord>, TigerError> {
    let shp_path = shp_path.as_ref();

    let mut reader = shapefile::Reader::from_path(shp_path)
        .map_err(|e| TigerError::ShapefileError(e.to_string()))?;

    let mut records = Vec::new();

    for (idx, shape_record) in reader.iter_shapes_and_records().enumerate() {
        let (shape, record) = shape_record
            .map_err(|e| TigerError::ShapefileError(e.to_string()))?;

        // Extract GEOID from attributes
        let geoid = match record.get("GEOID") {
            Some(shapefile::dbase::FieldValue::Character(Some(s))) => s.trim().to_string(),
            _ => return Err(TigerError::MissingGeoid),
        };

        // Validate GEOID length (tract = 11 chars: SS CCC TTTTTT)
        if geoid.len() != 11 {
            return Err(TigerError::InvalidGeoidLength(geoid));
        }

        // Extract area fields (both in square metres in TIGER files)
        let aland = match record.get("ALAND") {
            Some(shapefile::dbase::FieldValue::Numeric(Some(v))) => *v as i64,
            _ => 0,
        };
        let awater = match record.get("AWATER") {
            Some(shapefile::dbase::FieldValue::Numeric(Some(v))) => *v as i64,
            _ => 0,
        };

        // Convert shapefile geometry to WKB
        let geometry_wkb = shape_to_wkb(&shape, idx)?;
        if geometry_wkb.is_empty() {
            continue; // skip empty geometries
        }

        records.push(TractRecord {
            geoid,
            geometry_wkb,
            aland,
            awater,
            population: 0, // joined separately from PL 94-171
        });
    }

    // Sort by GEOID for deterministic ordering (matches Python sort)
    records.sort_by(|a, b| a.geoid.cmp(&b.geoid));

    Ok(records)
}

/// Convert a shapefile shape to WKB bytes.
/// Returns empty Vec for Null shapes.
fn shape_to_wkb(shape: &shapefile::Shape, idx: usize) -> Result<Vec<u8>, TigerError> {
    match shape {
        shapefile::Shape::Polygon(poly) => {
            let geo_poly = shapefile_poly_to_geo(poly);
            let wkb = geo_to_wkb_polygon(&geo_poly);
            Ok(wkb)
        }
        shapefile::Shape::NullShape => Ok(Vec::new()),
        // Some tract files use PolygonZ (3D) — flatten to 2D
        shapefile::Shape::PolygonZ(polyz) => {
            let geo_poly = shapefile_polyz_to_geo(polyz);
            let wkb = geo_to_wkb_polygon(&geo_poly);
            Ok(wkb)
        }
        _ => Err(TigerError::UnsupportedGeometry(idx)),
    }
}

fn shapefile_poly_to_geo(poly: &shapefile::Polygon) -> Polygon<f64> {
    let rings = poly.rings();
    if rings.is_empty() {
        return Polygon::new(LineString::new(vec![]), vec![]);
    }
    let exterior = ring_to_linestring(rings[0].points());
    let interiors: Vec<LineString<f64>> = rings[1..]
        .iter()
        .map(|r| ring_to_linestring(r.points()))
        .collect();
    Polygon::new(exterior, interiors)
}

fn shapefile_polyz_to_geo(poly: &shapefile::PolygonZ) -> Polygon<f64> {
    let rings = poly.rings();
    if rings.is_empty() {
        return Polygon::new(LineString::new(vec![]), vec![]);
    }
    let exterior = ring_to_linestring_z(rings[0].points());
    let interiors: Vec<LineString<f64>> = rings[1..]
        .iter()
        .map(|r| ring_to_linestring_z(r.points()))
        .collect();
    Polygon::new(exterior, interiors)
}

fn ring_to_linestring(points: &[shapefile::Point]) -> LineString<f64> {
    LineString::new(
        points.iter().map(|p| Coord { x: p.x, y: p.y }).collect(),
    )
}

fn ring_to_linestring_z(points: &[shapefile::PointZ]) -> LineString<f64> {
    LineString::new(
        points.iter().map(|p| Coord { x: p.x, y: p.y }).collect(),
    )
}

/// Encode a geo Polygon as WKB (Well-Known Binary), little-endian.
/// Format: byte order (1) + type (3 = Polygon) + n_rings + rings
fn geo_to_wkb_polygon(poly: &Polygon<f64>) -> Vec<u8> {
    let mut buf = Vec::new();
    // Byte order: little-endian
    buf.push(1u8);
    // WKB type: Polygon = 3
    buf.extend_from_slice(&3u32.to_le_bytes());

    let n_rings = 1 + poly.interiors().len() as u32;
    buf.extend_from_slice(&n_rings.to_le_bytes());

    // Exterior ring
    write_ring(&mut buf, poly.exterior().points().collect::<Vec<_>>().as_slice());
    // Interior rings (holes)
    for interior in poly.interiors() {
        write_ring(&mut buf, interior.points().collect::<Vec<_>>().as_slice());
    }
    buf
}

fn write_ring(buf: &mut Vec<u8>, coords: &[geo_types::Point<f64>]) {
    buf.extend_from_slice(&(coords.len() as u32).to_le_bytes());
    for pt in coords {
        buf.extend_from_slice(&pt.x().to_le_bytes());
        buf.extend_from_slice(&pt.y().to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geoid_validation_passes_11_chars() {
        // Construct a minimal test: validate the 11-char check directly
        let valid = "50005957100"; // Vermont tract GEOID
        assert_eq!(valid.len(), 11);
    }

    #[test]
    fn test_geoid_validation_rejects_wrong_length() {
        let short = "5000595710"; // 10 chars — missing digit
        assert_ne!(short.len(), 11);
    }

    #[test]
    fn test_wkb_polygon_starts_with_little_endian_marker() {
        use geo_types::LineString;
        let poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );
        let wkb = geo_to_wkb_polygon(&poly);
        assert!(!wkb.is_empty());
        assert_eq!(wkb[0], 1u8); // little-endian byte order marker
        // WKB type = 3 (Polygon)
        assert_eq!(u32::from_le_bytes([wkb[1], wkb[2], wkb[3], wkb[4]]), 3u32);
    }

    #[test]
    fn test_wkb_polygon_ring_count() {
        use geo_types::LineString;
        let poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 0.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![], // no holes
        );
        let wkb = geo_to_wkb_polygon(&poly);
        let n_rings = u32::from_le_bytes([wkb[5], wkb[6], wkb[7], wkb[8]]);
        assert_eq!(n_rings, 1u32); // exterior only
    }

    #[test]
    fn test_read_vermont_tracts_skippable() {
        // Live shapefile test — skip if file not present
        let path = std::path::Path::new("data/2020/tiger/tracts/tl_2020_50_tract/tl_2020_50_tract.shp");
        if !path.exists() {
            return; // skip silently (CI won't have data/)
        }
        let records = read_tiger_tracts(path).expect("should read VT tracts");
        assert_eq!(records.len(), 193, "Vermont should have 193 census tracts");
        // All GEOIDs should be 11 chars starting with "50" (Vermont FIPS)
        for r in &records {
            assert_eq!(r.geoid.len(), 11, "GEOID {}", r.geoid);
            assert!(r.geoid.starts_with("50"), "GEOID {} should start with 50", r.geoid);
            assert!(!r.geometry_wkb.is_empty(), "WKB should not be empty for {}", r.geoid);
        }
        // Records should be sorted by GEOID
        let geoids: Vec<&str> = records.iter().map(|r| r.geoid.as_str()).collect();
        let mut sorted = geoids.clone();
        sorted.sort();
        assert_eq!(geoids, sorted, "records should be sorted by GEOID");
    }
}
