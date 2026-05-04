// Task #52 — full implementation
pub struct CategoricalScheme { colors: [(u8,u8,u8); 12] }
pub struct PoliticalScheme;
pub struct DemographicScheme;
pub struct CompactnessScheme;

impl Default for CategoricalScheme {
    fn default() -> Self {
        Self { colors: [
            (141,211,199),(255,255,179),(190,186,218),(251,128,114),
            (128,177,211),(253,180,98),(179,222,105),(252,205,229),
            (217,217,217),(188,128,189),(204,235,197),(255,237,111),
        ]}
    }
}
impl CategoricalScheme {
    pub fn color(&self, district_id: usize) -> (u8,u8,u8) { self.colors[district_id % 12] }
}

impl PoliticalScheme {
    /// t=0.0 → Rep red (220,50,47), t=0.5 → near-white (240,240,240), t=1.0 → Dem blue (31,119,180)
    pub fn color(&self, dem_frac: f64) -> (u8,u8,u8) {
        let t = dem_frac.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8, t: f64| -> u8 {
            (a as f64 + (b as f64 - a as f64) * t).round() as u8
        };
        if t <= 0.5 {
            // Rep red → near-white: t=0 → (220,50,47), t=0.5 → (240,240,240)
            let s = t * 2.0;
            (lerp(220, 240, s), lerp(50, 240, s), lerp(47, 240, s))
        } else {
            // near-white → Dem blue: t=0.5 → (240,240,240), t=1.0 → (31,119,180)
            let s = (t - 0.5) * 2.0;
            (lerp(240, 31, s), lerp(240, 119, s), lerp(240, 180, s))
        }
    }
}

impl DemographicScheme {
    /// Yellow → Brown sequential
    pub fn color(&self, frac: f64) -> (u8,u8,u8) {
        let t = frac.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8, t: f64| -> u8 {
            (a as f64 + (b as f64 - a as f64) * t).round() as u8
        };
        (lerp(255, 102, t), lerp(255, 51, t), lerp(178, 0, t))
    }
}

impl CompactnessScheme {
    /// Green sequential: light green → dark green
    pub fn color(&self, score: f64) -> (u8,u8,u8) {
        let t = score.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8, t: f64| -> u8 {
            (a as f64 + (b as f64 - a as f64) * t).round() as u8
        };
        (lerp(229, 0, t), lerp(245, 109, t), lerp(224, 44, t))
    }
}

/// Greedy graph coloring using the categorical scheme. Returns color per node.
pub fn graph_color(adjacency: &[Vec<usize>], scheme: &CategoricalScheme) -> Vec<(u8,u8,u8)> {
    let n = adjacency.len();
    let mut assigned = vec![usize::MAX; n];
    for i in 0..n {
        let used: std::collections::HashSet<usize> = adjacency[i].iter()
            .filter(|&&j| assigned[j] != usize::MAX).map(|&j| assigned[j]).collect();
        assigned[i] = (0..).find(|c| !used.contains(c)).unwrap();
    }
    assigned.iter().map(|&c| scheme.color(c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorical_12_colors_all_distinct() {
        let scheme = CategoricalScheme::default();
        let colors: Vec<_> = (0..12).map(|i| scheme.color(i)).collect();
        let unique: std::collections::HashSet<_> = colors.iter().collect();
        assert_eq!(unique.len(), 12);
    }

    #[test]
    fn test_categorical_cycles_after_12() {
        let scheme = CategoricalScheme::default();
        assert_eq!(scheme.color(0), scheme.color(12));
    }

    #[test]
    fn test_choropleth_political_dem_end_is_blue() {
        let scheme = PoliticalScheme;
        let (r, g, b) = scheme.color(1.0);
        assert!(b > r && b > g);
    }

    #[test]
    fn test_choropleth_political_rep_end_is_red() {
        let scheme = PoliticalScheme;
        let (r, g, b) = scheme.color(0.0);
        assert!(r > b && r > g);
    }

    #[test]
    fn test_choropleth_midpoint_is_near_white() {
        let scheme = PoliticalScheme;
        let (r, g, b) = scheme.color(0.5);
        assert!(r > 200 && g > 200 && b > 200);
    }

    #[test]
    fn test_graph_coloring_triangle() {
        let adjacency = vec![vec![1usize, 2], vec![0, 2], vec![0, 1]];
        let colors = graph_color(&adjacency, &CategoricalScheme::default());
        assert_ne!(colors[0], colors[1]);
        assert_ne!(colors[1], colors[2]);
        assert_ne!(colors[0], colors[2]);
    }

    // ── Additional colorscheme tests ─────────────────────────────────────────

    #[test]
    fn test_political_scheme_clamps_below_zero() {
        let scheme = PoliticalScheme;
        let at_zero = scheme.color(0.0);
        let below = scheme.color(-1.0);
        assert_eq!(at_zero, below, "clamping below 0 must equal color(0)");
    }

    #[test]
    fn test_political_scheme_clamps_above_one() {
        let scheme = PoliticalScheme;
        let at_one = scheme.color(1.0);
        let above = scheme.color(2.0);
        assert_eq!(at_one, above, "clamping above 1 must equal color(1)");
    }

    #[test]
    fn test_political_scheme_upper_half_blue_dominates_red() {
        // In the upper half (t > 0.5) blue channel value > red channel value
        let scheme = PoliticalScheme;
        let (r07, _, b07) = scheme.color(0.7);
        let (r10, _, b10) = scheme.color(1.0);
        assert!(b07 > r07, "at t=0.7 blue channel must exceed red");
        assert!(b10 > r10, "at t=1.0 blue channel must exceed red");
    }

    #[test]
    fn test_political_scheme_monotone_red_channel_lower_half() {
        // In the lower half (t=0..0.5), red channel should increase toward white
        let scheme = PoliticalScheme;
        let (r00, _, _) = scheme.color(0.0);
        let (r03, _, _) = scheme.color(0.3);
        let (r05, _, _) = scheme.color(0.5);
        assert!(r03 >= r00, "red must increase from 0.0→0.3");
        assert!(r05 >= r03, "red must increase from 0.3→0.5");
    }

    #[test]
    fn test_demographic_scheme_clamps_bounds() {
        let scheme = DemographicScheme;
        assert_eq!(scheme.color(-0.5), scheme.color(0.0));
        assert_eq!(scheme.color(1.5), scheme.color(1.0));
    }

    #[test]
    fn test_demographic_scheme_endpoints() {
        let scheme = DemographicScheme;
        let (r0, g0, b0) = scheme.color(0.0);
        let (r1, g1, b1) = scheme.color(1.0);
        // At t=0 the lerp gives the 'a' endpoint values
        assert_eq!(r0, 255);
        assert_eq!(g0, 255);
        assert_eq!(b0, 178);
        // At t=1 the lerp gives the 'b' endpoint values
        assert_eq!(r1, 102);
        assert_eq!(g1, 51);
        assert_eq!(b1, 0);
    }

    #[test]
    fn test_compactness_scheme_clamps_bounds() {
        let scheme = CompactnessScheme;
        assert_eq!(scheme.color(-0.1), scheme.color(0.0));
        assert_eq!(scheme.color(1.1), scheme.color(1.0));
    }

    #[test]
    fn test_compactness_scheme_endpoints() {
        let scheme = CompactnessScheme;
        let (r0, g0, b0) = scheme.color(0.0);
        let (r1, g1, b1) = scheme.color(1.0);
        // t=0 → light green (229,245,224)
        assert_eq!(r0, 229);
        assert_eq!(g0, 245);
        assert_eq!(b0, 224);
        // t=1 → dark green (0,109,44)
        assert_eq!(r1, 0);
        assert_eq!(g1, 109);
        assert_eq!(b1, 44);
    }

    #[test]
    fn test_categorical_color_24_cycles_twice() {
        let scheme = CategoricalScheme::default();
        for i in 0..12 {
            assert_eq!(scheme.color(i), scheme.color(i + 24),
                "color({i}) should equal color({})", i + 24);
        }
    }

    #[test]
    fn test_graph_coloring_single_node() {
        let adjacency: Vec<Vec<usize>> = vec![vec![]];
        let colors = graph_color(&adjacency, &CategoricalScheme::default());
        assert_eq!(colors.len(), 1);
    }

    #[test]
    fn test_graph_coloring_path_uses_two_colors() {
        // 4-node path: 0-1-2-3 — only 2 colors needed
        let adjacency = vec![
            vec![1usize],
            vec![0, 2],
            vec![1, 3],
            vec![2usize],
        ];
        let colors = graph_color(&adjacency, &CategoricalScheme::default());
        assert_ne!(colors[0], colors[1]);
        assert_ne!(colors[1], colors[2]);
        assert_ne!(colors[2], colors[3]);
        // Path graph is bipartite: alternating colors
        assert_eq!(colors[0], colors[2]);
        assert_eq!(colors[1], colors[3]);
    }

    #[test]
    fn test_graph_coloring_disconnected_uses_same_color_for_isolated_nodes() {
        // Nodes 0 and 2 have no adjacency — can share a color
        let adjacency = vec![vec![], vec![0usize, 2], vec![]];
        let colors = graph_color(&adjacency, &CategoricalScheme::default());
        assert_ne!(colors[1], colors[0], "node 1 must differ from node 0");
        assert_ne!(colors[1], colors[2], "node 1 must differ from node 2");
    }
}
