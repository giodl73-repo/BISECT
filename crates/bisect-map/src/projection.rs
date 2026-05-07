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

// ── InsetProjection ───────────────────────────────────────────────────────────

/// National US map projection with Alaska and Hawaii insets.
///
/// Places three independent equirectangular sub-projections on one canvas:
///   - Continental: top 75% of canvas, lon -125→-65, lat 24→50
///   - Alaska inset: bottom-left 28%×25%, scaled to 35% of continental
///   - Hawaii inset: bottom-left, right of AK, scaled to ~90%
///
/// **Display only** — all three sub-projections are equirectangular.
/// Alaska at 61°N has significant east-west compression (a degree of
/// longitude ≈ 55km vs 111km at the equator). Do not use projected
/// coordinates for metric computation.
pub struct InsetProjection {
    // (sub-projection, x_offset_px, y_offset_px) for each region
    continental: (Projection, f64, f64),
    alaska:      (Projection, f64, f64),
    hawaii:      (Projection, f64, f64),
    canvas_w: u32,
    canvas_h: u32,
}

impl InsetProjection {
    /// Standard US national layout for the given canvas dimensions.
    /// `canvas_w` × `canvas_h` in pixels. Typical: 2400 × 1500.
    pub fn us_national(canvas_w: u32, canvas_h: u32) -> Self {
        let w = canvas_w as f64;
        let h = canvas_h as f64;

        // Continental: full width, top 75% of canvas
        let cont_h_px = (h * 0.75).round() as u32;
        let cont_proj = Projection::from_bbox(-125.0, 24.0, -65.0, 50.0,
                                              canvas_w, cont_h_px, 0.02);

        // Alaska inset: bottom-left, 28% width × 25% height
        let ak_w_px = (w * 0.28).round() as u32;
        let ak_h_px = (h * 0.25).round() as u32;
        let ak_y_offset = cont_h_px as f64;
        let ak_proj = Projection::from_bbox(-180.0, 51.0, -130.0, 72.0,
                                            ak_w_px, ak_h_px, 0.02);

        // Hawaii inset: right of AK inset, 12% width × 10% height, vertically centred
        let hi_w_px = (w * 0.12).round() as u32;
        let hi_h_px = (h * 0.10).round() as u32;
        let hi_x_offset = ak_w_px as f64 + 4.0;
        let hi_y_offset = cont_h_px as f64 + (ak_h_px as f64 - hi_h_px as f64) * 0.5;
        let hi_proj = Projection::from_bbox(-161.0, 18.0, -154.0, 23.0,
                                            hi_w_px, hi_h_px, 0.04);

        Self {
            continental: (cont_proj, 0.0, 0.0),
            alaska:      (ak_proj, 0.0, ak_y_offset),
            hawaii:      (hi_proj, hi_x_offset, hi_y_offset),
            canvas_w,
            canvas_h,
        }
    }

    /// Project (lon, lat) → (svg_x, svg_y).
    ///
    /// Classifies the coordinate into continental / Alaska / Hawaii by
    /// geographic bounds, then applies the appropriate sub-projection + offset.
    pub fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        // Alaska: lon < -130 and lat > 51
        if lon < -130.0 && lat > 51.0 {
            let (p, ox, oy) = &self.alaska;
            let (x, y) = p.project(lon, lat);
            return (x + ox, y + oy);
        }
        // Hawaii: lon < -154 and lat < 25
        if lon < -154.0 && lat < 25.0 {
            let (p, ox, oy) = &self.hawaii;
            let (x, y) = p.project(lon, lat);
            return (x + ox, y + oy);
        }
        // Continental (everything else, including Puerto Rico which falls off-canvas)
        let (p, ox, oy) = &self.continental;
        let (x, y) = p.project(lon, lat);
        (x + ox, y + oy)
    }

    pub fn canvas_w(&self) -> u32 { self.canvas_w }
    pub fn canvas_h(&self) -> u32 { self.canvas_h }
    pub fn continental_height(&self) -> f64 {
        let (_, _, oy) = &self.alaska;
        *oy
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

    // ── InsetProjection tests ─────────────────────────────────────────────

    #[test]
    fn test_inset_continental_coord_in_main_area() {
        let proj = InsetProjection::us_national(1200, 800);
        // Kansas City, MO: lon=-94.6, lat=39.1 — centre of continental US
        let (x, y) = proj.project(-94.6, 39.1);
        // Should land in the continental area: top 600px of a 1200×800 canvas
        assert!(x > 200.0 && x < 1000.0, "x={x} outside expected continental range");
        assert!(y > 0.0 && y < 600.0, "y={y} outside continental area (top 75%)");
    }

    #[test]
    fn test_inset_alaska_coord_in_bottom_left() {
        let proj = InsetProjection::us_national(1200, 800);
        // Anchorage, AK: lon=-149.9, lat=61.2
        let (x, y) = proj.project(-149.9, 61.2);
        // AK inset occupies bottom-left: y > 600px (below continental)
        assert!(y > 580.0, "AK y={y} should be in bottom inset (y > 580)");
        // AK inset is 28% of 1200px = 336px wide
        assert!(x < 340.0, "AK x={x} should be in left portion of canvas");
    }

    #[test]
    fn test_inset_hawaii_coord_right_of_ak() {
        let proj = InsetProjection::us_national(1200, 800);
        // Honolulu, HI: lon=-157.8, lat=21.3
        let (x, y) = proj.project(-157.8, 21.3);
        // HI inset is to the right of AK inset (ak_w ~336px, so HI x > 336)
        assert!(x > 336.0, "HI x={x} should be right of AK inset");
        // HI is also in the bottom strip
        assert!(y > 580.0, "HI y={y} should be in bottom strip");
    }

    #[test]
    fn test_inset_same_latitude_same_y_continental() {
        let proj = InsetProjection::us_national(1200, 800);
        // Two points at same continental latitude → same SVG y
        let (_, y1) = proj.project(-120.0, 37.0);
        let (_, y2) = proj.project(-80.0, 37.0);
        assert!((y2 - y1).abs() < 2.0, "same lat should give same y: y1={y1} y2={y2}");
    }

    #[test]
    fn test_inset_continental_lon_increases_right() {
        let proj = InsetProjection::us_national(1200, 800);
        let (x1, _) = proj.project(-120.0, 37.0);
        let (x2, _) = proj.project(-80.0, 37.0);
        assert!(x2 > x1, "more east = higher x: x1={x1} x2={x2}");
    }

    #[test]
    fn test_inset_anchorage_is_in_alaska_inset() {
        let proj = InsetProjection::us_national(1200, 800);
        let (x, y) = proj.project(-149.9, 61.2); // Anchorage
        // Alaska inset is bottom-left: y > 600px (below continental area)
        assert!(y > 580.0, "Anchorage must be in bottom AK inset, y={y}");
        // And left portion of canvas
        assert!(x < 400.0, "Anchorage must be in left portion, x={x}");
    }

    #[test]
    fn test_inset_honolulu_is_in_hawaii_inset() {
        let proj = InsetProjection::us_national(1200, 800);
        let (x, y) = proj.project(-157.8, 21.3); // Honolulu
        // Hawaii inset is bottom strip, right of AK
        assert!(y > 580.0, "Honolulu must be in bottom strip, y={y}");
    }

    #[test]
    fn test_inset_seattle_is_in_continental() {
        let proj = InsetProjection::us_national(1200, 800);
        let (x, y) = proj.project(-122.3, 47.6); // Seattle
        // Continental area is top 75% of canvas (0 to 600px for 800px canvas)
        assert!(y < 600.0, "Seattle must be in continental area, y={y}");
        assert!(x > 0.0 && x < 1200.0, "Seattle x must be in canvas, x={x}");
    }

    #[test]
    fn test_inset_miami_is_in_continental() {
        let proj = InsetProjection::us_national(1200, 800);
        let (x, y) = proj.project(-80.2, 25.8); // Miami
        assert!(y < 600.0, "Miami must be in continental area, y={y}");
        // Miami is east — should be right side of canvas
        let (x_seattle, _) = proj.project(-122.3, 47.6);
        assert!(x > x_seattle, "Miami must be east of Seattle on map");
    }

    #[test]
    fn test_inset_canvas_size() {
        let proj = InsetProjection::us_national(1200, 800);
        assert_eq!(proj.canvas_w(), 1200);
        assert_eq!(proj.canvas_h(), 800);
    }

    // ── Additional projection tests ───────────────────────────────────────────

    #[test]
    fn test_project_coords_batch_matches_individual() {
        let proj = Projection::from_bbox(-80.0, 35.0, -70.0, 45.0, 800, 600, 0.05);
        let pairs = vec![(-79.0, 36.0), (-75.0, 40.0), (-71.0, 44.0)];
        let batch = proj.project_coords(&pairs);
        for (i, &(lon, lat)) in pairs.iter().enumerate() {
            let (ex, ey) = proj.project(lon, lat);
            assert!((batch[i].0 - ex).abs() < 1e-10);
            assert!((batch[i].1 - ey).abs() < 1e-10);
        }
    }

    #[test]
    fn test_scale_accessor_positive() {
        let proj = Projection::from_bbox(0.0, 0.0, 10.0, 10.0, 1000, 1000, 0.0);
        assert!(proj.scale() > 0.0, "scale must be positive");
    }

    #[test]
    fn test_scale_larger_canvas_gives_larger_scale() {
        let proj_small = Projection::from_bbox(0.0, 0.0, 10.0, 10.0, 100, 100, 0.0);
        let proj_large = Projection::from_bbox(0.0, 0.0, 10.0, 10.0, 1000, 1000, 0.0);
        assert!(proj_large.scale() > proj_small.scale());
    }

    #[test]
    fn test_project_midpoint_lands_near_canvas_center() {
        // With zero padding and square canvas, midpoint of bbox → canvas center
        let proj = Projection::from_bbox(0.0, 0.0, 10.0, 10.0, 500, 500, 0.0);
        let (cx, cy) = proj.project(5.0, 5.0);
        assert!((cx - 250.0).abs() < 2.0, "mid-x={cx}");
        assert!((cy - 250.0).abs() < 2.0, "mid-y={cy}");
    }

    #[test]
    fn test_project_coords_empty_slice_returns_empty() {
        let proj = Projection::from_bbox(0.0, 0.0, 1.0, 1.0, 100, 100, 0.0);
        let result = proj.project_coords(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_padding_moves_points_inward() {
        // With 10% padding, corners should not reach 0 or 500
        let proj = Projection::from_bbox(0.0, 0.0, 1.0, 1.0, 500, 500, 0.10);
        let (x0, _) = proj.project(0.0, 0.5); // left edge
        let (x1, _) = proj.project(1.0, 0.5); // right edge
        assert!(x0 > 0.0, "left edge must be padded in: x0={x0}");
        assert!(x1 < 500.0, "right edge must be padded in: x1={x1}");
    }

    #[test]
    fn test_inset_continental_height_is_positive() {
        let proj = InsetProjection::us_national(2400, 1500);
        assert!(proj.continental_height() > 0.0);
    }

    #[test]
    fn test_inset_continental_height_is_75_percent_of_canvas() {
        let proj = InsetProjection::us_national(2400, 1500);
        let expected = (1500.0_f64 * 0.75).round();
        assert!((proj.continental_height() - expected).abs() < 2.0,
            "continental_height={} expected~{}", proj.continental_height(), expected);
    }

    #[test]
    fn test_inset_alaska_is_below_continental() {
        let proj = InsetProjection::us_national(2400, 1500);
        let cont_h = proj.continental_height();
        // Anchorage should map to y > cont_h
        let (_, y) = proj.project(-149.9, 61.2);
        assert!(y > cont_h - 10.0, "Anchorage y={y} must be below continental boundary {cont_h}");
    }
}
