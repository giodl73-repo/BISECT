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
    fn test_assert_balanced_fails() {
        let assignments: HashMap<usize, usize> =
            [(0, 0), (1, 0), (2, 1), (3, 1), (4, 1)].into_iter().collect();
        let p = Partition::from_assignments(assignments);
        let weights = vec![1000i64, 1000, 1000, 1000, 1000];
        // District 0: 2000, District 1: 3000 — 20% deviation
        let result = p.assert_balanced(&weights, 2, 0.005);
        assert!(result.is_err());
    }
}
