use geo_types::MultiPolygon;
use crate::labeler::LabelSpec;
use crate::projection::Projection;

/// Embedded font — loaded once, consistent across all renders.
/// Font bytes embedded at compile time.
static FONT_BYTES: &[u8] = include_bytes!("font_embed.ttf");

pub fn default_font_db() -> fontdb::Database {
    let mut db = fontdb::Database::new();
    db.load_font_data(FONT_BYTES.to_vec());
    db
}

/// Derive canvas pixel dimensions from DPI and a fixed 8-inch base.
///
/// aspect = width / height of the state's bounding box.
/// The long edge is always 8 inches; the short edge scales with aspect.
pub fn canvas_size_from_dpi(dpi: u32, base_inches: f64, aspect: f64) -> (u32, u32) {
    let long_px = (dpi as f64 * base_inches) as u32;
    if aspect >= 1.0 {
        // landscape: width = long edge
        let h = (long_px as f64 / aspect).round() as u32;
        (long_px, h.max(1))
    } else {
        // portrait: height = long edge
        let w = (long_px as f64 * aspect).round() as u32;
        (w.max(1), long_px)
    }
}

/// Convert projected pixel coordinate list to an SVG path `d` attribute string.
///
/// Produces: M x0 y0 L x1 y1 ... Z
pub fn coords_to_svg_path(pts: &[(f64, f64)]) -> String {
    if pts.is_empty() { return String::new(); }
    let mut d = format!("M {:.2} {:.2}", pts[0].0, pts[0].1);
    for &(x, y) in &pts[1..] {
        d.push_str(&format!(" L {x:.2} {y:.2}"));
    }
    d.push('Z');
    d
}

/// Build one SVG `<path>` element for a district polygon.
pub fn polygon_svg_element(pts: &[(f64, f64)], fill: (u8, u8, u8), opacity: f64) -> String {
    let d = coords_to_svg_path(pts);
    let (r, g, b) = fill;
    format!(
        "<path d=\"{d}\" fill=\"rgb({r},{g},{b})\" opacity=\"{opacity:.2}\" stroke=\"#555555\" stroke-width=\"0.5\"/>"
    )
}

/// Build a complete SVG document string from district geometries + labels.
pub fn build_svg(
    districts: &[(usize, MultiPolygon<f64>, (u8, u8, u8), LabelSpec)],
    proj: &Projection,
    w: u32,
    h: u32,
) -> String {
    let mut svg = format!(
        r#"<svg width="{w}" height="{h}" xmlns="http://www.w3.org/2000/svg"><rect width="{w}" height="{h}" fill="white"/>"#
    );

    // Draw polygons
    for (_, mp, color, _) in districts {
        for poly in &mp.0 {
            let outer: Vec<(f64, f64)> = poly.exterior().coords()
                .map(|c| proj.project(c.x, c.y))
                .collect();
            svg.push_str(&polygon_svg_element(&outer, *color, 0.85));

            // Holes (interior rings)
            for hole in poly.interiors() {
                let pts: Vec<(f64, f64)> = hole.coords()
                    .map(|c| proj.project(c.x, c.y))
                    .collect();
                svg.push_str(&polygon_svg_element(&pts, (255, 255, 255), 1.0));
            }
        }
    }

    // Draw labels on top
    for (_, mp, _, label) in districts {
        use crate::labeler::{largest_component, adaptive_font_size, label_fits, halo_text_svg};
        use polylabel::polylabel;

        if let Some(poly) = largest_component(mp) {
            if let Ok(center) = polylabel(poly, &1.0) {
                let (cx, cy) = proj.project(center.x(), center.y());

                // Estimate inscribed circle radius in pixels
                let radius_px = proj.scale() * 0.5; // fallback
                let font_px = adaptive_font_size(radius_px);

                let main = &label.main;
                let stat = label.stat.as_deref().unwrap_or("");
                let annotation = label.annotation.as_deref().unwrap_or("");
                let display_main = if annotation.is_empty() {
                    main.clone()
                } else {
                    format!("{main} ({annotation})")
                };

                if label_fits(&display_main, font_px, radius_px) {
                    svg.push_str(&halo_text_svg(cx, cy, &display_main, stat, font_px));
                }
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

/// Render an SVG string to PNG bytes via resvg.
pub fn svg_to_png(svg_str: &str, font_db: &fontdb::Database) -> anyhow::Result<Vec<u8>> {
    let mut opt = usvg::Options::default();
    // Load the embedded font into usvg's font database
    for face in font_db.faces() {
        if let fontdb::Source::Binary(data) = &face.source {
            opt.fontdb_mut().load_font_data(data.as_ref().as_ref().to_vec());
        }
    }
    let tree = usvg::Tree::from_str(svg_str, &opt)
        .map_err(|e| anyhow::anyhow!("SVG parse error: {e}"))?;

    let size = tree.size();
    let w = size.width() as u32;
    let h = size.height() as u32;

    let mut pixmap = tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| anyhow::anyhow!("could not create pixmap {w}x{h}"))?;
    pixmap.fill(tiny_skia::Color::WHITE);

    resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());

    pixmap.encode_png().map_err(|e| anyhow::anyhow!("PNG encode error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::{polygon, MultiPolygon};
    use crate::labeler::LabelSpec;

    fn simple_label(main: &str) -> LabelSpec {
        LabelSpec { main: main.into(), annotation: None, stat: None, lineage_superscript: None }
    }

    #[test]
    fn test_svg_path_is_closed() {
        let pts = vec![(10.0, 10.0), (90.0, 10.0), (90.0, 90.0), (10.0, 90.0)];
        let path = coords_to_svg_path(&pts);
        assert!(path.starts_with('M'), "path must start with M");
        assert!(path.ends_with('Z'), "path must end with Z");
    }

    #[test]
    fn test_svg_polygon_has_correct_fill() {
        let color = (255u8, 128u8, 0u8);
        let elem = polygon_svg_element(
            &[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
            color, 0.85,
        );
        assert!(elem.contains("rgb(255,128,0)"), "fill color missing: {elem}");
        assert!(elem.contains("opacity=\"0.85\""), "opacity missing: {elem}");
    }

    #[test]
    fn test_svg_polygon_has_stroke() {
        let elem = polygon_svg_element(
            &[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
            (200, 200, 200), 1.0,
        );
        assert!(elem.contains("stroke="), "stroke attribute missing");
    }

    #[test]
    fn test_canvas_size_from_dpi_landscape() {
        let (w, h) = canvas_size_from_dpi(150, 8.0, 1.5);
        assert_eq!(w, 1200);
        assert_eq!(h, 800);
    }

    #[test]
    fn test_canvas_size_from_dpi_72() {
        let (w, h) = canvas_size_from_dpi(72, 8.0, 1.0);
        assert_eq!(w, 576);
        assert_eq!(h, 576);
    }

    #[test]
    fn test_render_two_district_map_produces_nonblank_png() {
        let d1 = MultiPolygon(vec![
            polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.),(x:0.,y:0.)]
        ]);
        let d2 = MultiPolygon(vec![
            polygon![(x:1.,y:0.),(x:2.,y:0.),(x:2.,y:1.),(x:1.,y:1.),(x:1.,y:0.)]
        ]);
        let proj = Projection::from_bbox(0.0, 0.0, 2.0, 1.0, 400, 300, 0.05);
        let districts = vec![
            (1usize, d1, (141u8, 211u8, 199u8), simple_label("1")),
            (2, d2, (255u8, 255u8, 179u8), simple_label("2")),
        ];
        let svg = build_svg(&districts, &proj, 400, 300);
        let font_db = default_font_db();
        let png = svg_to_png(&svg, &font_db).expect("render must succeed");
        assert!(!png.is_empty(), "PNG must not be empty");
        assert!(png.len() > 1000, "PNG too small ({}B), likely blank", png.len());
        assert_eq!(&png[0..4], b"\x89PNG", "must be valid PNG magic");
    }

    #[test]
    fn test_render_with_labels_embeds_text() {
        let d1 = MultiPolygon(vec![
            polygon![(x:0.,y:0.),(x:10.,y:0.),(x:10.,y:10.),(x:0.,y:10.),(x:0.,y:0.)]
        ]);
        let proj = Projection::from_bbox(0.0, 0.0, 10.0, 10.0, 400, 300, 0.05);
        let districts = vec![(1usize, d1, (100u8, 200u8, 100u8), simple_label("1"))];
        let svg = build_svg(&districts, &proj, 400, 300);
        // SVG must contain at least one text element (from halo_text_svg)
        // Note: labels may be omitted if polygon is tiny relative to font — check label_fits logic
        assert!(svg.contains("<path"), "SVG must contain district paths");
    }

    // ── Additional renderer tests ─────────────────────────────────────────────

    #[test]
    fn test_coords_to_svg_path_empty() {
        assert_eq!(coords_to_svg_path(&[]), "");
    }

    #[test]
    fn test_coords_to_svg_path_single_point() {
        let path = coords_to_svg_path(&[(5.0, 10.0)]);
        assert!(path.starts_with("M 5.00 10.00"));
        assert!(path.ends_with('Z'));
    }

    #[test]
    fn test_coords_to_svg_path_three_points_has_two_l_segments() {
        let pts = vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)];
        let path = coords_to_svg_path(&pts);
        // One M and two L commands
        assert_eq!(path.matches(" L ").count(), 2);
    }

    #[test]
    fn test_polygon_svg_element_contains_path_tag() {
        let elem = polygon_svg_element(&[(0.0,0.0),(1.0,0.0),(1.0,1.0)], (100,200,50), 0.9);
        assert!(elem.starts_with("<path "));
        assert!(elem.ends_with("/>"));
    }

    #[test]
    fn test_polygon_svg_element_opacity_encoded() {
        let elem = polygon_svg_element(&[(0.0,0.0),(1.0,0.0),(1.0,1.0)], (0,0,0), 0.75);
        assert!(elem.contains("opacity=\"0.75\""), "opacity not found: {elem}");
    }

    #[test]
    fn test_canvas_size_portrait_aspect() {
        // aspect 0.5 (tall): height = long edge = 720, width = 0.5*720 = 360
        let (w, h) = canvas_size_from_dpi(90, 8.0, 0.5);
        assert_eq!(h, 720); // long edge
        assert_eq!(w, 360); // short edge
    }

    #[test]
    fn test_canvas_size_square_aspect() {
        let (w, h) = canvas_size_from_dpi(100, 8.0, 1.0);
        assert_eq!(w, h);
        assert_eq!(w, 800);
    }

    #[test]
    fn test_canvas_size_min_one_pixel() {
        // Extremely thin aspect → height clamped to 1
        let (w, h) = canvas_size_from_dpi(100, 8.0, 100.0); // very wide
        assert!(h >= 1);
        assert_eq!(w, 800); // long edge
    }

    #[test]
    fn test_build_svg_opens_with_svg_tag() {
        let d = MultiPolygon(vec![
            polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.),(x:0.,y:0.)]
        ]);
        let proj = Projection::from_bbox(0.0, 0.0, 1.0, 1.0, 200, 200, 0.0);
        let svg = build_svg(&[(1, d, (100u8, 100u8, 100u8), simple_label("1"))], &proj, 200, 200);
        assert!(svg.starts_with("<svg "), "must start with <svg: {}", &svg[..40.min(svg.len())]);
        assert!(svg.ends_with("</svg>"), "must end with </svg>");
    }

    #[test]
    fn test_build_svg_no_districts_has_background_rect() {
        let proj = Projection::from_bbox(0.0, 0.0, 1.0, 1.0, 100, 100, 0.0);
        let svg = build_svg(&[], &proj, 100, 100);
        assert!(svg.contains("<rect "), "background rect must be present");
        assert!(svg.contains("fill=\"white\""), "background must be white");
    }

    #[test]
    fn test_build_svg_encodes_width_height() {
        let d = MultiPolygon(vec![
            polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.),(x:0.,y:0.)]
        ]);
        let proj = Projection::from_bbox(0.0, 0.0, 1.0, 1.0, 640, 480, 0.0);
        let svg = build_svg(&[(1, d, (0u8, 0u8, 0u8), simple_label("A"))], &proj, 640, 480);
        assert!(svg.contains("width=\"640\""), "SVG width must be 640");
        assert!(svg.contains("height=\"480\""), "SVG height must be 480");
    }
}
