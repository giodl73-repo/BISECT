use sha2::{Digest, Sha256};
use thiserror::Error;

pub trait ObjectiveVector {
    fn objective_count(&self) -> usize;
    fn objective_value(&self, index: usize) -> f64;
}

impl<const N: usize> ObjectiveVector for [f64; N] {
    fn objective_count(&self) -> usize {
        N
    }

    fn objective_value(&self, index: usize) -> f64 {
        self[index]
    }
}

impl ObjectiveVector for Vec<f64> {
    fn objective_count(&self) -> usize {
        self.len()
    }

    fn objective_value(&self, index: usize) -> f64 {
        self[index]
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum RoptError {
    #[error("[INPUT] objective vector {point_index} must not be empty")]
    EmptyObjectiveVector { point_index: usize },
    #[error(
        "[INPUT] objective vector {point_index} dimension mismatch: expected {expected}, found {found}"
    )]
    ObjectiveDimensionMismatch {
        point_index: usize,
        expected: usize,
        found: usize,
    },
    #[error("[INPUT] objective vector {point_index} has non-finite value {value} at dimension {dimension}")]
    NonFiniteObjective {
        point_index: usize,
        dimension: usize,
        value: f64,
    },
    #[error("[INPUT] front entry {front_index} references objective index {point_index}, but len is {len}")]
    FrontIndexOutOfBounds {
        front_index: usize,
        point_index: usize,
        len: usize,
    },
    #[error("[INPUT] seed domain must not be empty")]
    EmptySeedDomain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedPart {
    U32(u32),
    U64(u64),
}

pub fn dominates<T: ObjectiveVector>(a: &T, b: &T) -> Result<bool, RoptError> {
    validate_pair(a, b)?;
    let mut strictly_better = false;
    for dimension in 0..a.objective_count() {
        let av = a.objective_value(dimension);
        let bv = b.objective_value(dimension);
        if av > bv {
            return Ok(false);
        }
        if av < bv {
            strictly_better = true;
        }
    }
    Ok(strictly_better)
}

pub fn fast_non_dominated_sort<T: ObjectiveVector>(
    objectives: &[T],
) -> Result<Vec<Vec<usize>>, RoptError> {
    validate_objectives(objectives)?;
    let n = objectives.len();
    if n == 0 {
        return Ok(vec![]);
    }

    let mut dominates_set: Vec<Vec<usize>> = vec![vec![]; n];
    let mut domination_count: Vec<usize> = vec![0; n];

    for p in 0..n {
        for q in 0..n {
            if p == q {
                continue;
            }
            if dominates(&objectives[p], &objectives[q])? {
                dominates_set[p].push(q);
            } else if dominates(&objectives[q], &objectives[p])? {
                domination_count[p] += 1;
            }
        }
    }

    let mut fronts = Vec::new();
    let mut current_front: Vec<usize> = (0..n).filter(|&p| domination_count[p] == 0).collect();

    while !current_front.is_empty() {
        fronts.push(current_front.clone());
        let mut next_front = Vec::new();
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

    Ok(fronts)
}

pub fn crowding_distance<T: ObjectiveVector>(
    front: &[usize],
    objectives: &[T],
) -> Result<Vec<f64>, RoptError> {
    validate_objectives(objectives)?;
    validate_front(front, objectives.len())?;
    let n = front.len();
    if n == 0 {
        return Ok(vec![]);
    }
    if n <= 2 {
        return Ok(vec![f64::INFINITY; n]);
    }

    let dimensions = objectives[0].objective_count();
    let mut distances = vec![0.0; n];

    for dimension in 0..dimensions {
        let mut sorted: Vec<usize> = (0..n).collect();
        sorted.sort_by(|&a, &b| {
            let va = objectives[front[a]].objective_value(dimension);
            let vb = objectives[front[b]].objective_value(dimension);
            va.total_cmp(&vb)
        });

        distances[sorted[0]] = f64::INFINITY;
        distances[sorted[n - 1]] = f64::INFINITY;

        let obj_min = objectives[front[sorted[0]]].objective_value(dimension);
        let obj_max = objectives[front[sorted[n - 1]]].objective_value(dimension);
        let obj_range = obj_max - obj_min;
        if obj_range == 0.0 {
            continue;
        }

        for i in 1..n - 1 {
            let prev = objectives[front[sorted[i - 1]]].objective_value(dimension);
            let next = objectives[front[sorted[i + 1]]].objective_value(dimension);
            distances[sorted[i]] += (next - prev) / obj_range;
        }
    }

    Ok(distances)
}

pub fn derive_seed(domain: &[u8], parts: &[SeedPart]) -> Result<u64, RoptError> {
    if domain.is_empty() {
        return Err(RoptError::EmptySeedDomain);
    }
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            hasher.update(b"_");
        }
        match part {
            SeedPart::U32(value) => hasher.update(value.to_le_bytes()),
            SeedPart::U64(value) => hasher.update(value.to_le_bytes()),
        }
    }
    let digest = hasher.finalize();
    Ok(u64::from_le_bytes(
        digest[..8].try_into().expect("SHA-256 digest has 32 bytes"),
    ))
}

fn validate_pair<T: ObjectiveVector>(a: &T, b: &T) -> Result<(), RoptError> {
    validate_objective(a, 0, a.objective_count())?;
    validate_objective(b, 1, a.objective_count())
}

fn validate_objectives<T: ObjectiveVector>(objectives: &[T]) -> Result<(), RoptError> {
    let Some(first) = objectives.first() else {
        return Ok(());
    };
    let expected = first.objective_count();
    for (point_index, objective) in objectives.iter().enumerate() {
        validate_objective(objective, point_index, expected)?;
    }
    Ok(())
}

fn validate_objective<T: ObjectiveVector>(
    objective: &T,
    point_index: usize,
    expected: usize,
) -> Result<(), RoptError> {
    let found = objective.objective_count();
    if found == 0 {
        return Err(RoptError::EmptyObjectiveVector { point_index });
    }
    if found != expected {
        return Err(RoptError::ObjectiveDimensionMismatch {
            point_index,
            expected,
            found,
        });
    }
    for dimension in 0..found {
        let value = objective.objective_value(dimension);
        if !value.is_finite() {
            return Err(RoptError::NonFiniteObjective {
                point_index,
                dimension,
                value,
            });
        }
    }
    Ok(())
}

fn validate_front(front: &[usize], len: usize) -> Result<(), RoptError> {
    for (front_index, &point_index) in front.iter().enumerate() {
        if point_index >= len {
            return Err(RoptError::FrontIndexOutOfBounds {
                front_index,
                point_index,
                len,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l0_dominance_requires_no_worse_and_one_strictly_better() {
        assert!(dominates(&[1.0, 1.0, 1.0], &[2.0, 1.0, 1.0]).unwrap());
        assert!(!dominates(&[1.0, 1.0, 1.0], &[1.0, 1.0, 1.0]).unwrap());
        assert!(!dominates(&[1.0, 2.0, 1.0], &[2.0, 1.0, 2.0]).unwrap());
    }

    #[test]
    fn l0_non_dominated_sort_orders_chain() {
        let objectives = [[1.0, 1.0], [2.0, 2.0], [3.0, 3.0]];

        assert_eq!(
            fast_non_dominated_sort(&objectives).unwrap(),
            vec![vec![0], vec![1], vec![2]]
        );
    }

    #[test]
    fn l0_crowding_marks_extremes_infinite() {
        let objectives = [[1.0, 1.0], [2.0, 2.0], [3.0, 3.0], [4.0, 4.0]];
        let distance = crowding_distance(&[0, 1, 2, 3], &objectives).unwrap();

        assert!(distance[0].is_infinite());
        assert!(distance[3].is_infinite());
        assert!(distance[1].is_finite());
        assert!(distance[2].is_finite());
    }

    #[test]
    fn l0_dimension_mismatch_is_rejected() {
        assert_eq!(
            fast_non_dominated_sort(&[vec![1.0, 2.0], vec![1.0]]),
            Err(RoptError::ObjectiveDimensionMismatch {
                point_index: 1,
                expected: 2,
                found: 1
            })
        );
    }

    #[test]
    fn l0_non_finite_objective_is_rejected() {
        match crowding_distance(&[0], &[[f64::NAN]]) {
            Err(RoptError::NonFiniteObjective {
                point_index,
                dimension,
                value,
            }) => {
                assert_eq!((point_index, dimension), (0, 0));
                assert!(value.is_nan());
            }
            other => panic!("expected non-finite objective error, got {other:?}"),
        }
    }

    #[test]
    fn l0_seed_derivation_is_deterministic_and_domain_separated() {
        let a = derive_seed(b"PARETO_INIT_", &[SeedPart::U32(0), SeedPart::U64(42)]).unwrap();
        let b = derive_seed(b"PARETO_INIT_", &[SeedPart::U32(0), SeedPart::U64(42)]).unwrap();
        let c = derive_seed(b"PARETO_MUT_", &[SeedPart::U32(0), SeedPart::U64(42)]).unwrap();

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn l0_seed_derivation_rejects_empty_domain() {
        assert_eq!(
            derive_seed(b"", &[SeedPart::U64(42)]),
            Err(RoptError::EmptySeedDomain)
        );
    }
}
