//! Huntington-Hill apportionment (2 U.S.C. §2a).
//!
//! Priority rule: a state with current seat count `n` receives the next seat
//! before any state whose priority `pop / sqrt(n*(n+1))` is lower. Every
//! state is guaranteed at least 1 seat (constitutional minimum).

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

/// Allocate `total_seats` congressional seats among states using Huntington-Hill.
///
/// `populations`: map from state code (e.g. `"CA"`) to total population.
/// `total_seats`: typically 435 for the U.S. House.
///
/// Returns the seat count for each state. Every state receives ≥ 1 seat.
///
/// # Panics
/// Panics if `total_seats < populations.len()` (fewer seats than states).
pub fn huntington_hill(
    populations: &HashMap<String, u64>,
    total_seats: u32,
) -> HashMap<String, u32> {
    let n_states = populations.len() as u32;
    assert!(
        total_seats >= n_states,
        "total_seats ({total_seats}) must be >= number of states ({n_states})"
    );

    // All states start with the constitutional minimum of 1 seat.
    let mut seats: HashMap<String, u32> =
        populations.keys().map(|k| (k.clone(), 1u32)).collect();

    // Priority queue entry. Higher priority → gets next seat.
    // Tie-break by state code for determinism.
    #[derive(PartialEq)]
    struct Entry {
        priority: f64,
        state: String,
    }
    impl Eq for Entry {}
    impl PartialOrd for Entry {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Entry {
        fn cmp(&self, other: &Self) -> Ordering {
            // Max-heap: higher priority wins. NaN treated as less than any finite value.
            match self.priority.partial_cmp(&other.priority) {
                Some(o) if o != Ordering::Equal => o,
                _ => self.state.cmp(&other.state).reverse(), // lex smaller state wins ties
            }
        }
    }

    let priority_for = |pop: u64, n: u32| -> f64 {
        let n = n as f64;
        pop as f64 / (n * (n + 1.0)).sqrt()
    };

    // Seed heap: each state has 1 seat, queue its priority to receive seat #2.
    let mut heap = BinaryHeap::with_capacity(populations.len());
    for (state, &pop) in populations {
        heap.push(Entry {
            priority: priority_for(pop, 1),
            state: state.clone(),
        });
    }

    // Assign the remaining seats one at a time.
    let remaining = total_seats - n_states;
    for _ in 0..remaining {
        let Entry { state, .. } = heap.pop().expect("heap is non-empty");
        let n = *seats.get(&state).unwrap();
        *seats.get_mut(&state).unwrap() += 1;
        let pop = populations[&state];
        heap.push(Entry {
            priority: priority_for(pop, n + 1),
            state,
        });
    }

    seats
}

/// Returns the apportionment priority of a state: `pop / sqrt(n*(n+1))`.
/// Useful for displaying the priority queue state at any point.
pub fn priority(pop: u64, current_seats: u32) -> f64 {
    let n = current_seats as f64;
    pop as f64 / (n * (n + 1.0)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pop(pairs: &[(&str, u64)]) -> HashMap<String, u64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_three_states_five_seats() {
        // A=6000, B=3000, C=1000, 5 seats.
        // Start: A=1, B=1, C=1 (3 seats used, 2 remaining).
        // Round 4: priorities A=6000/√2≈4243, B=3000/√2≈2121, C=1000/√2≈707
        //   → A wins (A=2). New A priority = 6000/√6 ≈ 2449.
        // Round 5: A=2449 > B=2121 → A wins again (A=3).
        // Final: A:3, B:1, C:1  (sum=5 ✓)
        let pops = pop(&[("A", 6_000), ("B", 3_000), ("C", 1_000)]);
        let result = huntington_hill(&pops, 5);
        assert_eq!(result["A"], 3);
        assert_eq!(result["B"], 1);
        assert_eq!(result["C"], 1);
    }

    #[test]
    fn test_minimum_one_seat_each() {
        let pops = pop(&[("X", 1_000_000), ("Y", 1)]);
        let result = huntington_hill(&pops, 2);
        assert_eq!(result["X"], 1);
        assert_eq!(result["Y"], 1);
    }

    #[test]
    fn test_total_equals_population_count() {
        let pops = pop(&[("A", 100), ("B", 200), ("C", 300)]);
        let result = huntington_hill(&pops, 3);
        let total: u32 = result.values().sum();
        assert_eq!(total, 3);
        assert_eq!(result["A"], 1);
        assert_eq!(result["B"], 1);
        assert_eq!(result["C"], 1);
    }

    #[test]
    fn test_seat_sum_correct() {
        let pops = pop(&[("A", 1_000), ("B", 2_000), ("C", 3_000), ("D", 500)]);
        let result = huntington_hill(&pops, 10);
        let total: u32 = result.values().sum();
        assert_eq!(total, 10);
    }

    #[test]
    #[should_panic]
    fn test_fewer_seats_than_states_panics() {
        let pops = pop(&[("A", 1_000), ("B", 2_000), ("C", 3_000)]);
        huntington_hill(&pops, 2); // 2 < 3 states
    }
}
