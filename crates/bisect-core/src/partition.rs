use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PartitionError {
    #[error("district {district} population {pop} deviates {dev:.3} from ideal {ideal:.0} (tolerance ±{tol:.3})")]
    ImbalancedDistrict { district: usize, pop: i64, ideal: f64, dev: f64, tol: f64 },
}

/// Maps tract_index → district_id.
#[derive(Debug, Clone)]
pub struct Partition {
    pub assignments: HashMap<usize, usize>,
}

impl Partition {
    pub fn from_assignments(assignments: HashMap<usize, usize>) -> Self {
        Self { assignments }
    }

    pub fn to_assignments(&self) -> &HashMap<usize, usize> {
        &self.assignments
    }

    /// Max absolute fractional deviation from ideal district population.
    /// Only valid on **final** leaf-node partitions, not intermediate bisections.
    pub fn population_balance(&self, vertex_weights: &[i64], n_districts: usize) -> f64 {
        let total: i64 = vertex_weights.iter().sum();
        let ideal = total as f64 / n_districts as f64;

        let mut dist_pop: HashMap<usize, i64> = HashMap::new();
        for (&tract, &dist) in &self.assignments {
            assert!(
                tract < vertex_weights.len(),
                "tract index {tract} out of bounds (vertex_weights.len()={})",
                vertex_weights.len()
            );
            *dist_pop.entry(dist).or_insert(0) += vertex_weights[tract];
        }

        dist_pop.values()
            .map(|&pop| (pop as f64 - ideal).abs() / ideal)
            .fold(0.0_f64, f64::max)
    }

    /// Panics (returns Err) if any district exceeds tolerance.
    /// Only call on final partitions — not intermediate bisection nodes.
    pub fn assert_balanced(
        &self,
        vertex_weights: &[i64],
        n_districts: usize,
        tolerance: f64,
    ) -> Result<(), PartitionError> {
        let total: i64 = vertex_weights.iter().sum();
        let ideal = total as f64 / n_districts as f64;

        let mut dist_pop: HashMap<usize, i64> = HashMap::new();
        for (&tract, &dist) in &self.assignments {
            assert!(
                tract < vertex_weights.len(),
                "tract index {tract} out of bounds (vertex_weights.len()={})",
                vertex_weights.len()
            );
            *dist_pop.entry(dist).or_insert(0) += vertex_weights[tract];
        }

        for (&district, &pop) in &dist_pop {
            let dev = (pop as f64 - ideal).abs() / ideal;
            if dev > tolerance {
                return Err(PartitionError::ImbalancedDistrict {
                    district, pop, ideal, dev, tol: tolerance,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_population_balance_even() {
        let assignments: HashMap<usize, usize> =
            [(0, 0), (1, 0), (2, 1), (3, 1)].into_iter().collect();
        let p = Partition::from_assignments(assignments);
        let weights = vec![1000i64, 1000, 1000, 1000];
        let dev = p.population_balance(&weights, 2);
        assert!(dev < 1e-9, "perfectly balanced → deviation should be ~0");
    }

    #[test]
    fn test_population_balance_uneven() {
        // District 0: 2000, District 1: 3000 → max_dev = 300/2500 = 0.12
        let assignments: HashMap<usize, usize> =
            [(0, 0), (1, 0), (2, 1), (3, 1), (4, 1)].into_iter().collect();
        let p = Partition::from_assignments(assignments);
        let weights = vec![1000i64, 1000, 1000, 1000, 1000];
        let dev = p.population_balance(&weights, 2);
        assert!((dev - 0.20).abs() < 1e-9, "expected 20% deviation, got {dev}");
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_population_balance_out_of_bounds_panics() {
        let assignments: HashMap<usize, usize> =
            [(0, 0), (99, 1)].into_iter().collect(); // tract 99 doesn't exist
        let p = Partition::from_assignments(assignments);
        let weights = vec![1000i64, 1000, 1000, 1000]; // only indices 0-3
        let _ = p.population_balance(&weights, 2);
    }

    #[test]
    fn test_population_balance_single_district_zero() {
        // Single district = everything assigned to it → deviation is always 0
        let assignments: HashMap<usize, usize> = [(0, 0), (1, 0), (2, 0)].into_iter().collect();
        let p = Partition::from_assignments(assignments);
        let weights = vec![500i64, 700, 300];
        let dev = p.population_balance(&weights, 1);
        assert_eq!(dev, 0.0, "single district always zero deviation");
    }

    #[test]
    fn test_assert_balanced_fails() {
        let assignments: HashMap<usize, usize> =
            [(0, 0), (1, 0), (2, 1), (3, 1), (4, 1)].into_iter().collect();
        let p = Partition::from_assignments(assignments);
        let weights = vec![1000i64, 1000, 1000, 1000, 1000];
        // District 0: 2000, District 1: 3000 — 20% deviation
        let result = p.assert_balanced(&weights, 2, 0.005);
        assert!(result.is_err());
        // Error message should name the deviation
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("deviates"), "error should mention 'deviates': {msg}");
    }

    #[test]
    fn test_assert_balanced_passes_within_tolerance() {
        // 2450 vs 2550 → max_dev = 50/2500 = 2% — passes ±5% but fails ±0.5%
        let assignments: HashMap<usize, usize> =
            [(0, 0), (1, 0), (2, 1), (3, 1)].into_iter().collect();
        let p = Partition::from_assignments(assignments);
        let weights = vec![1200i64, 1250, 1250, 1300];
        // D0: 2450, D1: 2550, ideal: 2500, dev = 50/2500 = 2%
        assert!(p.assert_balanced(&weights, 2, 0.05).is_ok());  // passes ±5%
        assert!(p.assert_balanced(&weights, 2, 0.005).is_err()); // fails ±0.5%
    }

    #[test]
    fn test_to_assignments_returns_correct_map() {
        let input: HashMap<usize, usize> = [(0, 0), (1, 0), (2, 1)].into_iter().collect();
        let p = Partition::from_assignments(input.clone());
        assert_eq!(*p.to_assignments(), input);
    }
}
