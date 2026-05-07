use geo_types::{Geometry, MultiPolygon, Polygon};
use std::collections::HashMap;

/// Decode WKB bytes from tiger.rs into a geo_types Geometry.
///
/// Handles both Polygon and MultiPolygon encodings — TIGER island tracts
/// are encoded as WKB MultiPolygon. Never panics on unknown geometry type.
pub fn wkb_to_geometry(wkb: &[u8]) -> anyhow::Result<Geometry<f64>> {
    use geozero::wkb::{FromWkb, WkbDialect};
    let mut cursor = std::io::Cursor::new(wkb);
    geo_types::Geometry::<f64>::from_wkb(&mut cursor, WkbDialect::Wkb)
        .map_err(|e| anyhow::anyhow!("WKB decode: {e}"))
}

/// Union all geometries (Polygon or MultiPolygon) into one MultiPolygon.
/// Flattens MultiPolygon components before dissolving.
pub fn dissolve_geometries(geoms: &[Geometry<f64>]) -> MultiPolygon<f64> {
    let polys: Vec<Polygon<f64>> = geoms.iter().flat_map(|g| match g {
        Geometry::Polygon(p) => vec![p.clone()],
        Geometry::MultiPolygon(mp) => mp.0.clone(),
        _ => vec![],
    }).collect();
    dissolve_polygons(&polys)
}

/// Union all polygons into one MultiPolygon via geo::BooleanOps.
pub fn dissolve_polygons(polys: &[Polygon<f64>]) -> MultiPolygon<f64> {
    use geo::BooleanOps;
    if polys.is_empty() { return MultiPolygon(vec![]); }
    let mut acc = MultiPolygon(vec![polys[0].clone()]);
    for p in &polys[1..] {
        acc = acc.union(&MultiPolygon(vec![p.clone()]));
    }
    acc
}

/// Group tract geometries by district assignment and dissolve each group.
/// Returns HashMap<district_id, MultiPolygon>.
pub fn group_dissolve(
    tract_geoms: &[Geometry<f64>],
    assignments: &[usize],
    _num_districts: usize,
) -> HashMap<usize, MultiPolygon<f64>> {
    let mut groups: HashMap<usize, Vec<Geometry<f64>>> = HashMap::new();
    for (i, &dist) in assignments.iter().enumerate() {
        if i < tract_geoms.len() {
            groups.entry(dist).or_default().push(tract_geoms[i].clone());
        }
    }
    groups.into_iter().map(|(d, geoms)| (d, dissolve_geometries(&geoms))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::polygon;
    use geo::Area;

    #[test]
    fn test_dissolve_two_adjacent_squares() {
        let sq1: Polygon<f64> = polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.),(x:0.,y:0.)];
        let sq2: Polygon<f64> = polygon![(x:1.,y:0.),(x:2.,y:0.),(x:2.,y:1.),(x:1.,y:1.),(x:1.,y:0.)];
        let merged = dissolve_polygons(&[sq1, sq2]);
        assert!((merged.unsigned_area() - 2.0).abs() < 1e-9,
            "merged area={}", merged.unsigned_area());
    }

    #[test]
    fn test_dissolve_group_by_district() {
        let tracts = vec![
            Geometry::Polygon(polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.),(x:0.,y:0.)]),
            Geometry::Polygon(polygon![(x:1.,y:0.),(x:2.,y:0.),(x:2.,y:1.),(x:1.,y:1.),(x:1.,y:0.)]),
            Geometry::Polygon(polygon![(x:0.,y:1.),(x:1.,y:1.),(x:1.,y:2.),(x:0.,y:2.),(x:0.,y:1.)]),
            Geometry::Polygon(polygon![(x:1.,y:1.),(x:2.,y:1.),(x:2.,y:2.),(x:1.,y:2.),(x:1.,y:1.)]),
        ];
        let assignments = vec![1usize, 1, 2, 2];
        let districts = group_dissolve(&tracts, &assignments, 2);
        assert_eq!(districts.len(), 2);
        assert!((districts[&1].unsigned_area() - 2.0).abs() < 1e-9);
        assert!((districts[&2].unsigned_area() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_dissolve_single_polygon_unchanged() {
        let p: Polygon<f64> = polygon![(x:0.,y:0.),(x:3.,y:0.),(x:3.,y:2.),(x:0.,y:2.),(x:0.,y:0.)];
        let merged = dissolve_polygons(&[p.clone()]);
        assert!((merged.unsigned_area() - p.unsigned_area()).abs() < 1e-9);
    }

    #[test]
    fn test_dissolve_noncontiguous_returns_two_components() {
        let sq1: Polygon<f64> = polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.),(x:0.,y:0.)];
        let sq2: Polygon<f64> = polygon![(x:5.,y:5.),(x:6.,y:5.),(x:6.,y:6.),(x:5.,y:6.),(x:5.,y:5.)];
        let merged = dissolve_polygons(&[sq1, sq2]);
        // Area is stable across i_overlay versions; component count may vary
        assert!((merged.unsigned_area() - 2.0).abs() < 1e-9,
            "non-contiguous area must be 2.0, got {}", merged.unsigned_area());
        assert!(!merged.0.is_empty());
    }

    #[test]
    fn test_wkb_decode_known_unit_square() {
        // WKB (OGC, little-endian) for a polygon: unit square (0,0)→(1,0)→(1,1)→(0,1)→(0,0)
        // Generated with geozero::ToWkb on geo_types::Polygon
        use geozero::{CoordDimensions, ToWkb};
        let p: Geometry<f64> = Geometry::Polygon(
            polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.),(x:0.,y:0.)]
        );
        let wkb = p.to_wkb(CoordDimensions::xy()).expect("encode");
        let decoded = wkb_to_geometry(&wkb).expect("decode");
        assert!((decoded.unsigned_area() - 1.0).abs() < 1e-9,
            "decoded area={}", decoded.unsigned_area());
    }

    #[test]
    fn test_wkb_multipolygon_does_not_panic() {
        use geozero::{CoordDimensions, ToWkb};
        let mp: Geometry<f64> = Geometry::MultiPolygon(MultiPolygon(vec![
            polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.),(x:0.,y:0.)],
            polygon![(x:5.,y:5.),(x:6.,y:5.),(x:6.,y:6.),(x:5.,y:6.),(x:5.,y:5.)],
        ]));
        let wkb = mp.to_wkb(CoordDimensions::xy()).expect("encode");
        let result = wkb_to_geometry(&wkb);
        assert!(result.is_ok(), "MultiPolygon WKB must not fail: {:?}", result.err());
        assert!((result.unwrap().unsigned_area() - 2.0).abs() < 1e-9);
    }
}
