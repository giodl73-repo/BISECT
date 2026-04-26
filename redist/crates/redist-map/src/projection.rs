/// Display projection: WGS84 bounding box → SVG pixel coordinates.
///
/// This is an equirectangular (plate carrée) projection — it maps lon/lat
/// linearly to pixel x/y. It is correct for DISPLAY ONLY. Do NOT use
/// projected pixel coordinates to compute area, perimeter, or compactness
/// metrics. All metric computation must use redist-analysis::compactness
/// which operates on the original WGS84 coordinates via geo::Area /
/// geo::EuclideanLength before projection.
///
/// At US latitudes (25°–49°N), a degree of longitude is ~60–85% of a degree
/// of latitude in linear distance, so east-west features appear compressed
/// relative to north-south. This is visually acceptable for district maps
/// but produces incorrect metric values if used for computation.
#[derive(Debug, Clone)]
pub struct Projection {
    x_offset: f64,
    y_offset: f64,
    scale: f64,
    canvas_h: f64,
}

impl Projection {
    /// Fit a WGS84 bounding box into a canvas with padding.
    /// Preserves aspect ratio (letterboxed). SVG y-axis is flipped
    /// (lat increases north → y decreases in SVG space).
    ///
    /// `padding_frac`: fraction of the shorter canvas dimension reserved
    /// as padding on each side (e.g. 0.05 = 5%).
    pub fn from_bbox(
        min_lon: f64, min_lat: f64,
        max_lon: f64, max_lat: f64,
        canvas_w: u32, canvas_h: u32,
        padding_frac: f64,
    ) -> Self {
        let w = canvas_w as f64;
        let h = canvas_h as f64;
        let pad = f64::min(w, h) * padding_frac;
        let drawable_w = w - 2.0 * pad;
        let drawable_h = h - 2.0 * pad;

        let lon_span = (max_lon - min_lon).max(1e-10);
        let lat_span = (max_lat - min_lat).max(1e-10);

        let scale_x = drawable_w / lon_span;
        let scale_y = drawable_h / lat_span;
        let scale = f64::min(scale_x, scale_y);

        // Centre the content within drawable area
        let content_w = lon_span * scale;
        let content_h = lat_span * scale;
        let x_offset = pad + (drawable_w - content_w) / 2.0 - min_lon * scale;
        // y_offset accounts for flipped y: max_lat maps to y=pad (top)
        let y_offset = pad + (drawable_h - content_h) / 2.0 + max_lat * scale;

        Self { x_offset, y_offset, scale, canvas_h: h }
    }

    /// Project (lon, lat) → (svg_x, svg_y). SVG y-axis is flipped.
    pub fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let x = lon * self.scale + self.x_offset;
        let y = self.y_offset - lat * self.scale;
        (x, y)
    }

    /// Project a slice of (lon, lat) pairs.
    pub fn project_coords(&self, coords: &[(f64, f64)]) -> Vec<(f64, f64)> {
        coords.iter().map(|&(lon, lat)| self.project(lon, lat)).collect()
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_bbox_maps_to_full_canvas() {
        let proj = Projection::from_bbox(0.0, 0.0, 1.0, 1.0, 500, 500, 0.05);
        let (x0, y0) = proj.project(0.0, 0.0); // bottom-left → bottom in SVG (high y)
        let (x1, y1) = proj.project(1.0, 1.0); // top-right → top in SVG (low y)
        // With 5% padding on a 500px canvas: pad = 25px
        assert!((x0 - 25.0).abs() < 1.0, "left edge x={x0}");
        assert!((x1 - 475.0).abs() < 1.0, "right edge x={x1}");
        assert!((y1 - 25.0).abs() < 1.0, "top edge y={y1}");  // lat=1 → low y
        assert!((y0 - 475.0).abs() < 1.0, "bottom edge y={y0}"); // lat=0 → high y
    }

    #[test]
    fn test_lon_increases_right() {
        let proj = Projection::from_bbox(-80.0, 35.0, -70.0, 45.0, 800, 600, 0.05);
        let (x1, _) = proj.project(-79.0, 40.0);
        let (x2, _) = proj.project(-71.0, 40.0);
        assert!(x2 > x1, "more east = higher x");
    }

    #[test]
    fn test_lat_increases_up_svg_y_decreases() {
        let proj = Projection::from_bbox(-80.0, 35.0, -70.0, 45.0, 800, 600, 0.05);
        let (_, y1) = proj.project(-75.0, 36.0); // south → high y
        let (_, y2) = proj.project(-75.0, 44.0); // north → low y
        assert!(y2 < y1, "more north = lower SVG y");
    }

    #[test]
    fn test_aspect_ratio_preserved_wide_bbox() {
        // 2:1 lon/lat bbox into square canvas → letterboxed vertically
        // NOTE: this test validates VISUAL centering only, not metric accuracy.
        let proj = Projection::from_bbox(0.0, 0.0, 2.0, 1.0, 500, 500, 0.0);
        let (x0, _) = proj.project(0.0, 0.5);
        let (x1, _) = proj.project(2.0, 0.5);
        // Full 500px width used (no padding)
        assert!((x1 - x0 - 500.0).abs() < 2.0, "width should span full canvas");
    }

    #[test]
    fn test_zero_padding_uses_full_canvas() {
        let proj = Projection::from_bbox(0.0, 0.0, 1.0, 1.0, 400, 400, 0.0);
        let (x0, _) = proj.project(0.0, 0.5);
        let (x1, _) = proj.project(1.0, 0.5);
        assert!((x0).abs() < 1.0);
        assert!((x1 - 400.0).abs() < 1.0);
    }
}
