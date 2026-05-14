use ropt_core::{crowding_distance, dominates, fast_non_dominated_sort, ObjectiveVector};

#[derive(Debug)]
struct Candidate {
    compactness: f64,
    partisan_deviation: f64,
    vra_deficit: f64,
}

impl ObjectiveVector for Candidate {
    fn objective_count(&self) -> usize {
        3
    }

    fn objective_value(&self, index: usize) -> f64 {
        match index {
            0 => self.compactness,
            1 => self.partisan_deviation,
            2 => self.vra_deficit,
            _ => panic!("objective index out of bounds"),
        }
    }
}

#[test]
fn l1_three_objective_redistricting_like_fixture_sorts_fronts() {
    let candidates = vec![
        Candidate {
            compactness: 10.0,
            partisan_deviation: 1.0,
            vra_deficit: 0.0,
        },
        Candidate {
            compactness: 12.0,
            partisan_deviation: 1.2,
            vra_deficit: 0.2,
        },
        Candidate {
            compactness: 8.0,
            partisan_deviation: 2.0,
            vra_deficit: 0.0,
        },
    ];

    assert!(dominates(&candidates[0], &candidates[1]).unwrap());
    assert_eq!(
        fast_non_dominated_sort(&candidates).unwrap(),
        vec![vec![0, 2], vec![1]]
    );
}

#[test]
fn l1_crowding_uses_front_local_positions() {
    let candidates = vec![
        Candidate {
            compactness: 1.0,
            partisan_deviation: 1.0,
            vra_deficit: 1.0,
        },
        Candidate {
            compactness: 2.0,
            partisan_deviation: 2.0,
            vra_deficit: 2.0,
        },
        Candidate {
            compactness: 3.0,
            partisan_deviation: 3.0,
            vra_deficit: 3.0,
        },
    ];

    let distances = crowding_distance(&[0, 1, 2], &candidates).unwrap();

    assert!(distances[0].is_infinite());
    assert!(distances[1].is_finite());
    assert!(distances[2].is_infinite());
}
