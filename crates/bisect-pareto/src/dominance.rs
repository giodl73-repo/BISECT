//! Non-dominated sorting and crowding distance for NSGA-II.
//!
//! Implements Deb et al. (2002) Algorithm 1 (fast_non_dominated_sort) and
//! the crowding distance assignment used for diversity preservation.
//!
//! Per spec §3.2 and §3.3.

use crate::objectives::Objectives;
use ropt_core::ObjectiveVector;

impl ObjectiveVector for Objectives {
    fn objective_count(&self) -> usize {
        3
    }

    fn objective_value(&self, index: usize) -> f64 {
        match index {
            0 => self.ec,
            1 => self.d_seats,
            2 => self.vra_deficit,
            _ => panic!("objective index out of bounds"),
        }
    }
}

/// Returns true if `a` dominates `b`:
/// `a` is no worse than `b` on all objectives AND strictly better on at least one.
pub fn dominates(a: &Objectives, b: &Objectives) -> bool {
    ropt_core::dominates(a, b).expect("valid finite bisect-pareto objectives")
}

/// Fast non-dominated sort (Deb et al. 2002, Algorithm 1).
///
/// Returns `fronts` where `fronts[0]` is the Pareto front (rank 0),
/// `fronts[1]` is the next front, etc. Each entry is an index into `objectives`.
///
/// The variable `dominates_set[p]` = set of plans that plan `p` dominates
/// (i.e., `p` is strictly better on at least one objective and no worse on any).
pub fn fast_non_dominated_sort(objectives: &[Objectives]) -> Vec<Vec<usize>> {
    ropt_core::fast_non_dominated_sort(objectives).expect("valid finite bisect-pareto objectives")
}

/// Compute crowding distance for plans within a single front.
///
/// Returns a Vec of distances, indexed by position in `front` (not by plan index).
/// Plans at the extremes of any objective receive f64::INFINITY.
///
/// If `front` has fewer than 2 plans, all distances are f64::INFINITY.
pub fn crowding_distance(front: &[usize], objectives: &[Objectives]) -> Vec<f64> {
    ropt_core::crowding_distance(front, objectives).expect("valid finite bisect-pareto objectives")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(ec: f64, d: f64, vra: f64) -> Objectives {
        Objectives {
            ec,
            d_seats: d,
            vra_deficit: vra,
        }
    }

    #[test]
    fn dominates_strict() {
        let a = obj(1.0, 1.0, 1.0);
        let b = obj(2.0, 2.0, 2.0);
        assert!(dominates(&a, &b));
        assert!(!dominates(&b, &a));
    }

    #[test]
    fn dominates_equal_not() {
        let a = obj(1.0, 1.0, 1.0);
        let b = obj(1.0, 1.0, 1.0);
        assert!(!dominates(&a, &b));
        assert!(!dominates(&b, &a));
    }

    #[test]
    fn dominates_mixed_not() {
        // a=(1,2,1) vs b=(2,1,2): a wins ec+vra, b wins d_seats — neither dominates
        let a = obj(1.0, 2.0, 1.0);
        let b = obj(2.0, 1.0, 2.0);
        assert!(!dominates(&a, &b));
        assert!(!dominates(&b, &a));
    }

    #[test]
    fn non_dominated_sort_basic() {
        // A dominates B and C; B and C are mutually non-dominated
        let objectives = vec![
            obj(1.0, 1.0, 1.0), // A=0: best on all
            obj(2.0, 2.0, 2.0), // B=1: dominated by A
            obj(2.0, 3.0, 1.5), // C=2: dominated by A, not by B
        ];
        let fronts = fast_non_dominated_sort(&objectives);
        assert!(!fronts.is_empty());
        // A must be in front 0
        assert!(fronts[0].contains(&0), "A must be in front 0");
        // B and C must be in front 1
        assert!(fronts.len() >= 2, "must have at least 2 fronts");
        assert!(fronts[1].contains(&1), "B must be in front 1");
        assert!(fronts[1].contains(&2), "C must be in front 1");
    }

    #[test]
    fn non_dominated_sort_all_equal() {
        // All plans equal — nobody dominates anyone — all in front 0
        let objectives = vec![obj(1.0, 1.0, 1.0); 5];
        let fronts = fast_non_dominated_sort(&objectives);
        assert_eq!(fronts.len(), 1, "all equal plans → single front");
        assert_eq!(fronts[0].len(), 5, "all 5 plans in front 0");
    }

    #[test]
    fn crowding_extremes_infinite() {
        // Plans at extremes of any objective get infinity
        let front = vec![0, 1, 2, 3];
        let objectives = vec![
            obj(1.0, 5.0, 0.0), // min EC, mid D, min VRA
            obj(5.0, 1.0, 3.0), // max EC, min D, mid VRA
            obj(3.0, 3.0, 6.0), // mid EC, mid D, max VRA
            obj(2.0, 8.0, 2.0), // low EC, max D, low VRA
        ];
        let dist = crowding_distance(&front, &objectives);
        assert_eq!(dist.len(), 4);

        // Index 0: min EC → infinity
        assert!(
            dist[0].is_infinite(),
            "index 0 (min EC) must be infinite: {}",
            dist[0]
        );
        // Index 1: min D_seats → infinity
        assert!(
            dist[1].is_infinite(),
            "index 1 (min D) must be infinite: {}",
            dist[1]
        );
        // Index 2: max VRA → infinity
        assert!(
            dist[2].is_infinite(),
            "index 2 (max VRA) must be infinite: {}",
            dist[2]
        );
        // Index 3: max D_seats → infinity
        assert!(
            dist[3].is_infinite(),
            "index 3 (max D) must be infinite: {}",
            dist[3]
        );
    }

    #[test]
    fn crowding_single_plan() {
        let front = vec![0];
        let objectives = vec![obj(1.0, 1.0, 1.0)];
        let dist = crowding_distance(&front, &objectives);
        assert_eq!(dist.len(), 1);
        assert!(dist[0].is_infinite());
    }

    #[test]
    fn crowding_two_plans() {
        let front = vec![0, 1];
        let objectives = vec![obj(1.0, 1.0, 1.0), obj(2.0, 2.0, 2.0)];
        let dist = crowding_distance(&front, &objectives);
        assert_eq!(dist.len(), 2);
        assert!(dist[0].is_infinite());
        assert!(dist[1].is_infinite());
    }

    #[test]
    fn crowding_empty() {
        let dist = crowding_distance(&[], &[]);
        assert!(dist.is_empty());
    }
}
