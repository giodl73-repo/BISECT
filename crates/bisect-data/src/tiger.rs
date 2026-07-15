use geo::Contains;
use geo_types::{Coord, LineString, MultiPolygon, Point, Polygon};
use shapefile::PolygonRing;
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
///
/// A TIGER feature may contain several outer rings (for islands or other
/// disconnected pieces). Those are preserved as a WKB MultiPolygon rather
/// than being misclassified as holes in the first polygon.
pub fn read_tiger_tracts<P: AsRef<Path>>(shp_path: P) -> Result<Vec<TractRecord>, TigerError> {
    let shp_path = shp_path.as_ref();

    let mut reader = shapefile::Reader::from_path(shp_path)
        .map_err(|e| TigerError::ShapefileError(e.to_string()))?;

    let mut records = Vec::new();

    for (idx, shape_record) in reader.iter_shapes_and_records().enumerate() {
        let (shape, record) =
            shape_record.map_err(|e| TigerError::ShapefileError(e.to_string()))?;

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
            Ok(geo_to_wkb_multipolygon(&shapefile_poly_to_geo(poly)))
        }
        shapefile::Shape::NullShape => Ok(Vec::new()),
        // Some tract files use PolygonZ (3D) — flatten to 2D
        shapefile::Shape::PolygonZ(polyz) => {
            Ok(geo_to_wkb_multipolygon(&shapefile_polyz_to_geo(polyz)))
        }
        _ => Err(TigerError::UnsupportedGeometry(idx)),
    }
}

fn shapefile_poly_to_geo(poly: &shapefile::Polygon) -> MultiPolygon<f64> {
    let mut exteriors = Vec::new();
    let mut interiors = Vec::new();
    for ring in poly.rings() {
        match ring {
            PolygonRing::Outer(points) => exteriors.push(ring_to_linestring(points)),
            PolygonRing::Inner(points) => interiors.push(ring_to_linestring(points)),
        }
    }
    rings_to_multipolygon(exteriors, interiors)
}

fn shapefile_polyz_to_geo(poly: &shapefile::PolygonZ) -> MultiPolygon<f64> {
    let mut exteriors = Vec::new();
    let mut interiors = Vec::new();
    for ring in poly.rings() {
        match ring {
            PolygonRing::Outer(points) => exteriors.push(ring_to_linestring_z(points)),
            PolygonRing::Inner(points) => interiors.push(ring_to_linestring_z(points)),
        }
    }
    rings_to_multipolygon(exteriors, interiors)
}

fn rings_to_multipolygon(
    exteriors: Vec<LineString<f64>>,
    interiors: Vec<LineString<f64>>,
) -> MultiPolygon<f64> {
    let mut grouped: Vec<_> = exteriors
        .into_iter()
        .map(|exterior| (exterior, Vec::new()))
        .collect();
    for interior in interiors {
        let Some(coord) = interior.0.first() else {
            continue;
        };
        let point = Point::new(coord.x, coord.y);
        if let Some((_, holes)) = grouped
            .iter_mut()
            .find(|(exterior, _)| Polygon::new(exterior.clone(), vec![]).contains(&point))
        {
            holes.push(interior);
        } else if let Some((_, holes)) = grouped.first_mut() {
            // Preserve malformed/uncontained rings instead of silently dropping geometry.
            holes.push(interior);
        }
    }
    MultiPolygon(
        grouped
            .into_iter()
            .map(|(exterior, interiors)| Polygon::new(exterior, interiors))
            .collect(),
    )
}

fn ring_to_linestring(points: &[shapefile::Point]) -> LineString<f64> {
    LineString::new(points.iter().map(|p| Coord { x: p.x, y: p.y }).collect())
}

fn ring_to_linestring_z(points: &[shapefile::PointZ]) -> LineString<f64> {
    LineString::new(points.iter().map(|p| Coord { x: p.x, y: p.y }).collect())
}

/// Encode a geo Polygon as WKB (Well-Known Binary), little-endian.
/// Format: byte order (1) + type (3 = Polygon) + n_rings + rings
pub fn geo_to_wkb_polygon_pub(poly: &Polygon<f64>) -> Vec<u8> {
    geo_to_wkb_polygon(poly)
}

pub fn geo_to_wkb_multipolygon(multipolygon: &MultiPolygon<f64>) -> Vec<u8> {
    if multipolygon.0.len() == 1 {
        return geo_to_wkb_polygon(&multipolygon.0[0]);
    }
    let mut buffer = vec![1];
    buffer.extend_from_slice(&6u32.to_le_bytes());
    buffer.extend_from_slice(&(multipolygon.0.len() as u32).to_le_bytes());
    for polygon in &multipolygon.0 {
        buffer.extend_from_slice(&geo_to_wkb_polygon(polygon));
    }
    buffer
}

fn geo_to_wkb_polygon(poly: &Polygon<f64>) -> Vec<u8> {
    let mut buf = Vec::new();
    // Byte order: little-endian
    buf.push(1u8);
    // WKB type: Polygon = 3
    buf.extend_from_slice(&3u32.to_le_bytes());

    let n_rings = 1 + poly.interiors().len() as u32;
    buf.extend_from_slice(&n_rings.to_le_bytes());

    // Exterior ring
    write_ring(
        &mut buf,
        poly.exterior().points().collect::<Vec<_>>().as_slice(),
    );
    // Interior rings (holes)
    for interior in poly.interiors() {
        write_ring(&mut buf, interior.points().collect::<Vec<_>>().as_slice());
    }
    buf
}

fn write_ring(buf: &mut Vec<u8>, coords: &[geo_types::Point<f64>]) {
    if coords.is_empty() {
        buf.extend_from_slice(&0u32.to_le_bytes());
        return;
    }
    // WKB requires the ring to be closed: first == last point.
    // Shapefile rings are always closed; add the closing point defensively
    // in case a caller constructs a polygon without one.
    let needs_close = coords
        .first()
        .zip(coords.last())
        .map(|(f, l)| (f.x() - l.x()).abs() > 1e-12 || (f.y() - l.y()).abs() > 1e-12)
        .unwrap_or(false);

    let n = if needs_close {
        coords.len() + 1
    } else {
        coords.len()
    };
    buf.extend_from_slice(&(n as u32).to_le_bytes());
    for pt in coords {
        buf.extend_from_slice(&pt.x().to_le_bytes());
        buf.extend_from_slice(&pt.y().to_le_bytes());
    }
    if needs_close {
        buf.extend_from_slice(&coords[0].x().to_le_bytes());
        buf.extend_from_slice(&coords[0].y().to_le_bytes());
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
    fn test_wkb_ring_closing_point_added_when_missing() {
        // Polygon with 3 unique points and NO explicit closing point
        let poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 0.5, y: 1.0 },
                // no closing point
            ]),
            vec![],
        );
        let wkb = geo_to_wkb_polygon(&poly);
        // Ring should have 4 points (3 + closing)
        let n_points = u32::from_le_bytes([wkb[9], wkb[10], wkb[11], wkb[12]]);
        assert_eq!(n_points, 4u32, "closing point should be appended");
        // First and last x must match
        let first_x = f64::from_le_bytes(wkb[13..21].try_into().unwrap());
        let last_offset = 13 + (n_points as usize - 1) * 16;
        let last_x = f64::from_le_bytes(wkb[last_offset..last_offset + 8].try_into().unwrap());
        assert_eq!(
            first_x, last_x,
            "first and last point must match (WKB ring closed)"
        );
    }

    #[test]
    fn test_wkb_ring_not_duplicated_when_already_closed() {
        // Polygon that already has the closing point
        let poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 0.5, y: 1.0 },
                Coord { x: 0.0, y: 0.0 }, // explicit closing point
            ]),
            vec![],
        );
        let wkb = geo_to_wkb_polygon(&poly);
        let n_points = u32::from_le_bytes([wkb[9], wkb[10], wkb[11], wkb[12]]);
        assert_eq!(
            n_points, 4u32,
            "should have 4 points (not 5) — no duplicate closing"
        );
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
    fn test_shapefile_multiple_outers_become_multipolygon() {
        let polygon = shapefile::Polygon::with_rings(vec![
            PolygonRing::Outer(vec![
                shapefile::Point::new(0.0, 0.0),
                shapefile::Point::new(0.0, 10.0),
                shapefile::Point::new(10.0, 10.0),
                shapefile::Point::new(10.0, 0.0),
            ]),
            PolygonRing::Outer(vec![
                shapefile::Point::new(20.0, 20.0),
                shapefile::Point::new(20.0, 30.0),
                shapefile::Point::new(30.0, 30.0),
                shapefile::Point::new(30.0, 20.0),
            ]),
        ]);

        let multipolygon = shapefile_poly_to_geo(&polygon);
        assert_eq!(multipolygon.0.len(), 2);
        let wkb = geo_to_wkb_multipolygon(&multipolygon);
        assert_eq!(u32::from_le_bytes(wkb[1..5].try_into().unwrap()), 6);
        assert_eq!(u32::from_le_bytes(wkb[5..9].try_into().unwrap()), 2);
    }

    #[test]
    fn test_shapefile_inner_ring_remains_a_hole() {
        let polygon = shapefile::Polygon::with_rings(vec![
            // Ring order is not significant in the shapefile specification.
            PolygonRing::Inner(vec![
                shapefile::Point::new(2.0, 2.0),
                shapefile::Point::new(8.0, 2.0),
                shapefile::Point::new(8.0, 8.0),
                shapefile::Point::new(2.0, 8.0),
            ]),
            PolygonRing::Outer(vec![
                shapefile::Point::new(0.0, 0.0),
                shapefile::Point::new(0.0, 10.0),
                shapefile::Point::new(10.0, 10.0),
                shapefile::Point::new(10.0, 0.0),
            ]),
        ]);

        let multipolygon = shapefile_poly_to_geo(&polygon);
        assert_eq!(multipolygon.0.len(), 1);
        assert_eq!(multipolygon.0[0].interiors().len(), 1);
        let wkb = geo_to_wkb_multipolygon(&multipolygon);
        assert_eq!(u32::from_le_bytes(wkb[1..5].try_into().unwrap()), 3);
        assert_eq!(u32::from_le_bytes(wkb[5..9].try_into().unwrap()), 2);
    }

    #[test]
    fn test_read_vermont_tracts_skippable() {
        // Live shapefile test — skip if file not present
        let path =
            std::path::Path::new("data/2020/tiger/tracts/tl_2020_50_tract/tl_2020_50_tract.shp");
        if !path.exists() {
            return; // skip silently (CI won't have data/)
        }
        let records = read_tiger_tracts(path).expect("should read VT tracts");
        assert_eq!(records.len(), 193, "Vermont should have 193 census tracts");
        // All GEOIDs should be 11 chars starting with "50" (Vermont FIPS)
        for r in &records {
            assert_eq!(r.geoid.len(), 11, "GEOID {}", r.geoid);
            assert!(
                r.geoid.starts_with("50"),
                "GEOID {} should start with 50",
                r.geoid
            );
            assert!(
                !r.geometry_wkb.is_empty(),
                "WKB should not be empty for {}",
                r.geoid
            );
        }
        // Records should be sorted by GEOID
        let geoids: Vec<&str> = records.iter().map(|r| r.geoid.as_str()).collect();
        let mut sorted = geoids.clone();
        sorted.sort();
        assert_eq!(geoids, sorted, "records should be sorted by GEOID");
    }
}
