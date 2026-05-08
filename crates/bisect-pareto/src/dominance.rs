//! Non-dominated sorting and crowding distance for NSGA-II.
//!
//! Implements Deb et al. (2002) Algorithm 1 (fast_non_dominated_sort) and
//! the crowding distance assignment used for diversity preservation.
//!
//! Per spec §3.2 and §3.3.

use crate::objectives::Objectives;

/// Returns true if `a` dominates `b`:
/// `a` is no worse than `b` on all objectives AND strictly better on at least one.
pub fn dominates(a: &Objectives, b: &Objectives) -> bool {
    a.ec <= b.ec && a.d_seats <= b.d_seats && a.vra_deficit <= b.vra_deficit
        && (a.ec < b.ec || a.d_seats < b.d_seats || a.vra_deficit < b.vra_deficit)
}

/// Fast non-dominated sort (Deb et al. 2002, Algorithm 1).
///
/// Returns `fronts` where `fronts[0]` is the Pareto front (rank 0),
/// `fronts[1]` is the next front, etc. Each entry is an index into `objectives`.
///
/// The variable `dominates_set[p]` = set of plans that plan `p` dominates
/// (i.e., `p` is strictly better on at least one objective and no worse on any).
pub fn fast_non_dominated_sort(objectives: &[Objectives]) -> Vec<Vec<usize>> {
    let n = objectives.len();
    if n == 0 {
        return vec![];
    }

    // dominates_set[p] = indices of plans that p dominates
    let mut dominates_set: Vec<Vec<usize>> = vec![vec![]; n];
    // domination_count[p] = number of plans that dominate p
    let mut domination_count: Vec<usize> = vec![0; n];

    for p in 0..n {
        for q in 0..n {
            if p == q { continue; }
            if dominates(&objectives[p], &objectives[q]) {
                dominates_set[p].push(q);
            } else if dominates(&objectives[q], &objectives[p]) {
                domination_count[p] += 1;
            }
        }
    }

    let mut fronts: Vec<Vec<usize>> = Vec::new();
    let mut current_front: Vec<usize> = (0..n)
        .filter(|&p| domination_count[p] == 0)
        .collect();

    while !current_front.is_empty() {
        fronts.push(current_front.clone());
        let mut next_front: Vec<usize> = Vec::new();
        for &p in &current_front {
            for &q in &dominates_set[p] {
                domination_count[q] -= 1;
                if domination_count[q] == 0 {
                    next_front.push(q);
                }
            }
        }
        current_front = next_front;
    }

    fronts
}

/// Compute crowding distance for plans within a single front.
///
/// Returns a Vec of distances, indexed by position in `front` (not by plan index).
/// Plans at the extremes of any objective receive f64::INFINITY.
///
/// If `front` has fewer than 2 plans, all distances are f64::INFINITY.
pub fn crowding_distance(front: &[usize], objectives: &[Objectives]) -> Vec<f64> {
    let n = front.len();
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![f64::INFINITY];
    }
    if n == 2 {
        return vec![f64::INFINITY, f64::INFINITY];
    }

    let mut distances = vec![0.0f64; n];

    // Process each objective dimension separately.
    // Closure approach avoids fn-pointer coercion of capturing closures.
    for obj_dim in 0..3usize {
        // Sort front-local indices by this objective
        let mut sorted: Vec<usize> = (0..n).collect();
        sorted.sort_by(|&a, &b| {
            let va = match obj_dim {
                0 => objectives[front[a]].ec,
                1 => objectives[front[a]].d_seats,
                _ => objectives[front[a]].vra_deficit,
            };
            let vb = match obj_dim {
                0 => objectives[front[b]].ec,
                1 => objectives[front[b]].d_seats,
                _ => objectives[front[b]].vra_deficit,
            };
            va.partial_cmp(&vb).unwrap()
        });

        let obj_val = |i: usize| -> f64 {
            match obj_dim {
                0 => objectives[front[i]].ec,
                1 => objectives[front[i]].d_seats,
                _ => objectives[front[i]].vra_deficit,
            }
        };

        // Boundary plans get infinity
        distances[sorted[0]] = f64::INFINITY;
        distances[sorted[n - 1]] = f64::INFINITY;

        let obj_min = obj_val(sorted[0]);
        let obj_max = obj_val(sorted[n - 1]);
        let obj_range = obj_max - obj_min;
        if obj_range == 0.0 {
            continue; // all plans identical on this objective
        }

        for i in 1..n - 1 {
            distances[sorted[i]] +=
                (obj_val(sorted[i + 1]) - obj_val(sorted[i - 1])) / obj_range;
        }
    }

    distances
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(ec: f64, d: f64, vra: f64) -> Objectives {
        Objectives { ec, d_seats: d, vra_deficit: vra }
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
            obj(1.0, 5.0, 0.0),  // min EC, mid D, min VRA
            obj(5.0, 1.0, 3.0),  // max EC, min D, mid VRA
            obj(3.0, 3.0, 6.0),  // mid EC, mid D, max VRA
            obj(2.0, 8.0, 2.0),  // low EC, max D, low VRA
        ];
        let dist = crowding_distance(&front, &objectives);
        assert_eq!(dist.len(), 4);

        // Index 0: min EC → infinity
        assert!(dist[0].is_infinite(), "index 0 (min EC) must be infinite: {}", dist[0]);
        // Index 1: min D_seats → infinity
        assert!(dist[1].is_infinite(), "index 1 (min D) must be infinite: {}", dist[1]);
        // Index 2: max VRA → infinity
        assert!(dist[2].is_infinite(), "index 2 (max VRA) must be infinite: {}", dist[2]);
        // Index 3: max D_seats → infinity
        assert!(dist[3].is_infinite(), "index 3 (max D) must be infinite: {}", dist[3]);
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
