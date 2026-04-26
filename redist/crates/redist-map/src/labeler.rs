// Task #53 — stub
use geo_types::{MultiPolygon, Polygon};
use geo::Area;

#[derive(Debug, Clone)]
pub struct LabelSpec {
    pub main: String,
    pub annotation: Option<String>,
    pub stat: Option<String>,
    pub lineage_superscript: Option<usize>,
}

pub fn adaptive_font_size(inscribed_radius_px: f64) -> f64 {
    (inscribed_radius_px * 0.4).clamp(6.0, 28.0)
}

pub fn label_fits(text: &str, font_px: f64, inscribed_radius_px: f64) -> bool {
    let w = text.chars().count() as f64 * font_px * 0.6;
    w <= inscribed_radius_px * 2.0
}

pub fn round_label(district_id: usize, districts_in_region: usize, _total: usize) -> LabelSpec {
    LabelSpec {
        main: district_id.to_string(),
        annotation: if districts_in_region > 1 { Some(districts_in_region.to_string()) } else { None },
        stat: None, lineage_superscript: None,
    }
}

pub fn round_label_with_lineage(district_id: usize, districts_in_region: usize, total: usize, parent: Option<usize>) -> LabelSpec {
    let mut l = round_label(district_id, districts_in_region, total);
    l.lineage_superscript = parent;
    l
}

pub fn political_label(district: usize, dem_frac: f64) -> LabelSpec {
    let margin = ((dem_frac - 0.5) * 200.0).round() as i32;
    let stat = if margin >= 0 { format!("D+{}%", margin) } else { format!("R+{}%", -margin) };
    LabelSpec { main: district.to_string(), annotation: None, stat: Some(stat), lineage_superscript: None }
}

pub fn demographic_label(district: usize, minority_frac: f64) -> LabelSpec {
    LabelSpec { main: district.to_string(), annotation: None,
        stat: Some(format!("{}% min", (minority_frac * 100.0).round() as u32)), lineage_superscript: None }
}

pub fn compactness_label(district: usize, pp: f64) -> LabelSpec {
    LabelSpec { main: district.to_string(), annotation: None,
        stat: Some(format!("PP: {:.2}", pp)), lineage_superscript: None }
}

pub fn halo_text_svg(cx: f64, cy: f64, main: &str, stat: &str, font_px: f64) -> String {
    let stat_size = font_px * 0.65;
    let stat_dy = font_px * 1.2;
    let common = format!(r#"x="{cx:.1}" text-anchor="middle" font-family="Liberation Sans, sans-serif""#);
    let halo = format!(
        r#"<text {common} y="{cy:.1}" font-size="{font_px:.1}" stroke="white" stroke-width="3" fill="white" paint-order="stroke"><tspan>{main}</tspan><tspan x="{cx:.1}" dy="{stat_dy:.1}" font-size="{stat_size:.1}">{stat}</tspan></text>"#
    );
    let fore = format!(
        "<text {common} y=\"{cy:.1}\" font-size=\"{font_px:.1}\" fill=\"#1a1a1a\"><tspan>{main}</tspan><tspan x=\"{cx:.1}\" dy=\"{stat_dy:.1}\" font-size=\"{stat_size:.1}\">{stat}</tspan></text>"
    );
    format!("{halo}{fore}")
}

pub fn largest_component(mp: &MultiPolygon<f64>) -> Option<&Polygon<f64>> {
    mp.0.iter().max_by(|a, b| {
        a.unsigned_area().partial_cmp(&b.unsigned_area()).unwrap()
    })
}

pub fn labels_overlap(a: (&str, f64, f64), b: (&str, f64, f64), font_px: f64) -> bool {
    let w_a = a.0.chars().count() as f64 * font_px * 0.6;
    let w_b = b.0.chars().count() as f64 * font_px * 0.6;
    let dx = (a.1 - b.1).abs();
    let dy = (a.2 - b.2).abs();
    dx < (w_a + w_b) / 2.0 && dy < font_px * 1.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_size_large_polygon() {
        assert_eq!(adaptive_font_size(200.0), 28.0);
    }

    #[test]
    fn test_font_size_small_polygon() {
        assert_eq!(adaptive_font_size(10.0), 6.0);
    }

    #[test]
    fn test_font_size_medium_polygon() {
        let size = adaptive_font_size(60.0);
        assert!((size - 24.0).abs() < 0.1);
    }

    #[test]
    fn test_label_fits() {
        assert!(label_fits("1 (26)", 20.0, 50.0));
        assert!(!label_fits("1 (26)", 20.0, 20.0));
    }

    #[test]
    fn test_round_label_early() {
        let l = round_label(1, 26, 26);
        assert_eq!(l.main, "1");
        assert_eq!(l.annotation, Some("26".into()));
    }

    #[test]
    fn test_round_label_final() {
        let l = round_label(3, 1, 52);
        assert_eq!(l.main, "3");
        assert_eq!(l.annotation, None);
    }

    #[test]
    fn test_round_label_lineage() {
        let l = round_label_with_lineage(7, 1, 52, Some(3));
        assert_eq!(l.lineage_superscript, Some(3));
    }

    #[test]
    fn test_political_label_dem() {
        let l = political_label(5, 0.67);
        assert_eq!(l.main, "5");
        assert_eq!(l.stat.as_deref(), Some("D+34%"));
    }

    #[test]
    fn test_political_label_rep() {
        let l = political_label(2, 0.38);
        assert!(l.stat.as_deref().unwrap().starts_with("R+"));
    }

    #[test]
    fn test_political_label_tossup() {
        let l = political_label(1, 0.501);
        assert_eq!(l.stat.as_deref(), Some("D+0%"));
    }

    #[test]
    fn test_demographic_label() {
        let l = demographic_label(3, 0.423);
        assert_eq!(l.stat.as_deref(), Some("42% min"));
    }

    #[test]
    fn test_compactness_label() {
        let l = compactness_label(7, 0.3456);
        assert_eq!(l.stat.as_deref(), Some("PP: 0.35"));
    }

    #[test]
    fn test_halo_svg_has_two_text_elements() {
        let svg = halo_text_svg(250.0, 300.0, "5", "D+12%", 18.0);
        assert_eq!(svg.matches("<text").count(), 2);
        assert!(svg.contains("stroke=\"white\""));
        assert!(svg.contains("#1a1a1a"));
    }

    #[test]
    fn test_labels_do_not_overlap_adjacent() {
        // Two labels far enough apart: centers 30px apart, font 6px → width ~3.6px each
        assert!(!labels_overlap(("1", 5.0, 15.0), ("2", 35.0, 15.0), 6.0));
    }

    #[test]
    fn test_labels_overlap_close() {
        // Two labels 3px apart with font 12px → should overlap
        assert!(labels_overlap(("1", 5.0, 15.0), ("2", 8.0, 15.0), 12.0));
    }
}
