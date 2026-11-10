//! The native State wrapper's historical circular exterior-perimeter estimate.
use std::collections::HashMap;

/// Preserves the State wrapper's resolved edge-weight arithmetic, including
/// missing weights defaulting to zero. These are selection inputs, not measured geometry.
pub fn native_state_exterior_perimeter(
    index: usize,
    land_area: f64,
    neighbors: &[usize],
    resolved_edges: &HashMap<(usize, usize), f64>,
) -> f64 {
    let total = if land_area > 0.0 {
        2.0 * (std::f64::consts::PI * land_area).sqrt()
    } else {
        0.0
    };
    let shared: f64 = neighbors.iter().map(|&other| {
        resolved_edges.get(&(index.min(other), index.max(other))).copied().unwrap_or(0.0)
    }).sum();
    (total - shared).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_circular_estimate_preserves_missing_edges_and_zero_clamp() {
        let edges = HashMap::from([((0, 1), 3.0), ((1, 2), 2.0)]);
        let area = 100.0;
        assert_eq!(native_state_exterior_perimeter(1, area, &[0, 2, 3], &edges),
                   2.0 * (std::f64::consts::PI * area).sqrt() - 5.0);
        assert_eq!(native_state_exterior_perimeter(1, 0.0, &[0, 2], &edges), 0.0);
        assert_eq!(native_state_exterior_perimeter(1, 0.01, &[0, 2], &edges), 0.0);
    }
}
