//! General divisor-method apportionment.
//!
//! All classical divisor methods share the same priority-queue algorithm;
//! they differ only in how they round a state's quotient to an integer seat count.
//! This module implements Webster, Adams, and Jefferson alongside Huntington-Hill
//! so they can be compared on equal footing.
//!
//! | Method           | Also known as          | Priority function                       |
//! |------------------|------------------------|-----------------------------------------|
//! | Huntington-Hill  | Equal-proportions (US) | pop / sqrt(n·(n+1))                     |
//! | Webster          | Sainte-Laguë           | pop / (n + 0.5)                         |
//! | Adams            | Smallest-divisors      | pop / n   (always rounds up)            |
//! | Jefferson        | D'Hondt / Largest-div  | pop / (n + 1) (always rounds down)      |
//!
//! The constitutional minimum of **1 seat per state** is enforced for all methods.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Which divisor-method rounding rule to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingRule {
    /// Geometric mean (2 U.S.C. §2a) — the current U.S. method.
    HuntingtonHill,
    /// Arithmetic mean (standard round-half-up). Minimises partisan bias.
    Webster,
    /// Always round up (ceiling). Favours small states.
    Adams,
    /// Always round down (floor). Favours large states (same as D'Hondt).
    Jefferson,
}

/// Allocate `house_size` seats among states using the specified divisor method.
///
/// `populations` is a slice of `(state_name, population)` pairs.
/// Every state receives at least 1 seat (constitutional minimum).
///
/// Returns a `Vec<(String, u32)>` with the same ordering as the input slice.
///
/// # Panics
/// Panics if `house_size < populations.len()` (fewer seats than states).
pub fn apportionment_divisor(
    populations: &[(String, u64)],
    house_size: u32,
    rule: RoundingRule,
) -> Vec<(String, u32)> {
    let n_states = populations.len() as u32;
    assert!(
        house_size >= n_states,
        "house_size ({house_size}) must be >= number of states ({n_states})"
    );

    // Track seats by index into the input slice for O(1) lookup.
    let mut seats: Vec<u32> = vec![1u32; populations.len()];

    // Internal priority queue entry.
    #[derive(PartialEq)]
    struct Entry {
        priority: f64,
        idx: usize,
        name: String, // kept only for deterministic tie-breaking
    }
    impl Eq for Entry {}
    impl PartialOrd for Entry {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Entry {
        fn cmp(&self, other: &Self) -> Ordering {
            match self.priority.partial_cmp(&other.priority) {
                Some(o) if o != Ordering::Equal => o,
                // Tie-break: lexicographically smaller name wins (consistent with HH).
                _ => self.name.cmp(&other.name).reverse(),
            }
        }
    }

    let priority_for = |pop: u64, n: u32, rule: RoundingRule| -> f64 {
        let n = n as f64;
        let p = pop as f64;
        match rule {
            RoundingRule::HuntingtonHill => p / (n * (n + 1.0)).sqrt(),
            RoundingRule::Webster => p / (n + 0.5),
            RoundingRule::Adams => p / n,
            RoundingRule::Jefferson => p / (n + 1.0),
        }
    };

    // Seed heap: every state starts at 1 seat; compute priority for seat #2.
    let mut heap = BinaryHeap::with_capacity(populations.len());
    for (idx, (name, pop)) in populations.iter().enumerate() {
        heap.push(Entry {
            priority: priority_for(*pop, 1, rule),
            idx,
            name: name.clone(),
        });
    }

    // Assign the remaining (house_size - n_states) seats one at a time.
    let remaining = house_size - n_states;
    for _ in 0..remaining {
        let Entry { idx, .. } = heap.pop().expect("heap is non-empty");
        let n = seats[idx];
        seats[idx] += 1;
        let pop = populations[idx].1;
        heap.push(Entry {
            priority: priority_for(pop, n + 1, rule),
            idx,
            name: populations[idx].0.clone(),
        });
    }

    // Reconstruct output in input order.
    populations
        .iter()
        .enumerate()
        .map(|(i, (name, _))| (name.clone(), seats[i]))
        .collect()
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Convenience: build a `Vec<(String, u64)>` from string-literal pairs.
#[cfg(test)]
pub(crate) fn make_pops(pairs: &[(&str, u64)]) -> Vec<(String, u64)> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── equal populations ────────────────────────────────────────────────────

    #[test]
    fn equal_pops_two_states_ten_seats_hh() {
        let pops = make_pops(&[("A", 1_000_000), ("B", 1_000_000)]);
        let result = apportionment_divisor(&pops, 10, RoundingRule::HuntingtonHill);
        let map: std::collections::HashMap<_, _> = result.into_iter().collect();
        assert_eq!(map["A"], 5, "HH: equal pops should split evenly");
        assert_eq!(map["B"], 5, "HH: equal pops should split evenly");
    }

    #[test]
    fn equal_pops_two_states_ten_seats_webster() {
        let pops = make_pops(&[("A", 1_000_000), ("B", 1_000_000)]);
        let result = apportionment_divisor(&pops, 10, RoundingRule::Webster);
        let map: std::collections::HashMap<_, _> = result.into_iter().collect();
        assert_eq!(map["A"], 5, "Webster: equal pops should split evenly");
        assert_eq!(map["B"], 5, "Webster: equal pops should split evenly");
    }

    #[test]
    fn equal_pops_two_states_ten_seats_adams() {
        let pops = make_pops(&[("A", 1_000_000), ("B", 1_000_000)]);
        let result = apportionment_divisor(&pops, 10, RoundingRule::Adams);
        let map: std::collections::HashMap<_, _> = result.into_iter().collect();
        assert_eq!(map["A"], 5, "Adams: equal pops should split evenly");
        assert_eq!(map["B"], 5, "Adams: equal pops should split evenly");
    }

    #[test]
    fn equal_pops_two_states_ten_seats_jefferson() {
        let pops = make_pops(&[("A", 1_000_000), ("B", 1_000_000)]);
        let result = apportionment_divisor(&pops, 10, RoundingRule::Jefferson);
        let map: std::collections::HashMap<_, _> = result.into_iter().collect();
        assert_eq!(map["A"], 5, "Jefferson: equal pops should split evenly");
        assert_eq!(map["B"], 5, "Jefferson: equal pops should split evenly");
    }

    // ── 3:1 population ratio ─────────────────────────────────────────────────

    #[test]
    fn three_to_one_ratio_four_seats_hh() {
        // 3:1 ratio, 4 seats → large=3, small=1 (exact quota, unambiguous).
        let pops = make_pops(&[("Large", 3_000_000), ("Small", 1_000_000)]);
        let result = apportionment_divisor(&pops, 4, RoundingRule::HuntingtonHill);
        let map: std::collections::HashMap<_, _> = result.into_iter().collect();
        assert_eq!(map["Large"], 3, "HH 3:1: large gets 3");
        assert_eq!(map["Small"], 1, "HH 3:1: small gets 1");
    }

    #[test]
    fn three_to_one_ratio_four_seats_webster() {
        let pops = make_pops(&[("Large", 3_000_000), ("Small", 1_000_000)]);
        let result = apportionment_divisor(&pops, 4, RoundingRule::Webster);
        let map: std::collections::HashMap<_, _> = result.into_iter().collect();
        assert_eq!(map["Large"], 3, "Webster 3:1: large gets 3");
        assert_eq!(map["Small"], 1, "Webster 3:1: small gets 1");
    }

    #[test]
    fn three_to_one_ratio_four_seats_adams() {
        let pops = make_pops(&[("Large", 3_000_000), ("Small", 1_000_000)]);
        let result = apportionment_divisor(&pops, 4, RoundingRule::Adams);
        let map: std::collections::HashMap<_, _> = result.into_iter().collect();
        assert_eq!(map["Large"], 3, "Adams 3:1: large gets 3");
        assert_eq!(map["Small"], 1, "Adams 3:1: small gets 1");
    }

    #[test]
    fn three_to_one_ratio_four_seats_jefferson() {
        let pops = make_pops(&[("Large", 3_000_000), ("Small", 1_000_000)]);
        let result = apportionment_divisor(&pops, 4, RoundingRule::Jefferson);
        let map: std::collections::HashMap<_, _> = result.into_iter().collect();
        assert_eq!(map["Large"], 3, "Jefferson 3:1: large gets 3");
        assert_eq!(map["Small"], 1, "Jefferson 3:1: small gets 1");
    }

    // ── Jefferson vs HH: large-state bias ────────────────────────────────────
    //
    // Three states: A=6M, B=3M, C=1M; 10 seats.
    // Jefferson (floor/D'Hondt) is known to favour large states, so A should
    // receive at least as many seats under Jefferson as under Huntington-Hill.

    #[test]
    fn jefferson_favours_large_states_vs_hh() {
        let pops = make_pops(&[("A", 6_000_000), ("B", 3_000_000), ("C", 1_000_000)]);
        let hh = apportionment_divisor(&pops, 10, RoundingRule::HuntingtonHill);
        let jeff = apportionment_divisor(&pops, 10, RoundingRule::Jefferson);

        let hh_map: std::collections::HashMap<_, _> = hh.into_iter().collect();
        let jeff_map: std::collections::HashMap<_, _> = jeff.into_iter().collect();

        // Sanity: seat totals must both equal 10.
        let hh_total: u32 = hh_map.values().sum();
        let jeff_total: u32 = jeff_map.values().sum();
        assert_eq!(hh_total, 10);
        assert_eq!(jeff_total, 10);

        // Jefferson allocates >= HH to the largest state (A).
        assert!(
            jeff_map["A"] >= hh_map["A"],
            "Jefferson should give large state A at least as many seats as HH \
             (got Jefferson={}, HH={})",
            jeff_map["A"],
            hh_map["A"]
        );

        // Correspondingly, Jefferson allocates <= HH to the smallest state (C).
        assert!(
            jeff_map["C"] <= hh_map["C"],
            "Jefferson should give small state C at most as many seats as HH \
             (got Jefferson={}, HH={})",
            jeff_map["C"],
            hh_map["C"]
        );
    }

    // ── Adams vs HH: small-state bias ────────────────────────────────────────
    //
    // Adams (ceiling) always rounds up, which inflates small-state seat counts.
    // With enough seats, the smallest state should get at least as many seats
    // under Adams as under HH.

    #[test]
    fn adams_favours_small_states_vs_hh() {
        let pops = make_pops(&[("A", 6_000_000), ("B", 3_000_000), ("C", 1_000_000)]);
        let hh = apportionment_divisor(&pops, 10, RoundingRule::HuntingtonHill);
        let adams = apportionment_divisor(&pops, 10, RoundingRule::Adams);

        let hh_map: std::collections::HashMap<_, _> = hh.into_iter().collect();
        let adams_map: std::collections::HashMap<_, _> = adams.into_iter().collect();

        let hh_total: u32 = hh_map.values().sum();
        let adams_total: u32 = adams_map.values().sum();
        assert_eq!(hh_total, 10);
        assert_eq!(adams_total, 10);

        // Adams allocates >= HH to the smallest state (C).
        assert!(
            adams_map["C"] >= hh_map["C"],
            "Adams should give small state C at least as many seats as HH \
             (got Adams={}, HH={})",
            adams_map["C"],
            hh_map["C"]
        );

        // Correspondingly, Adams allocates <= HH to the largest state (A).
        assert!(
            adams_map["A"] <= hh_map["A"],
            "Adams should give large state A at most as many seats as HH \
             (got Adams={}, HH={})",
            adams_map["A"],
            hh_map["A"]
        );
    }

    // ── seat totals always correct ────────────────────────────────────────────

    #[test]
    fn seat_total_always_equals_house_size() {
        let pops = make_pops(&[
            ("CA", 39_538_223),
            ("TX", 29_145_505),
            ("FL", 21_538_187),
            ("NY", 20_201_249),
            ("PA", 13_002_700),
        ]);
        for rule in [
            RoundingRule::HuntingtonHill,
            RoundingRule::Webster,
            RoundingRule::Adams,
            RoundingRule::Jefferson,
        ] {
            let result = apportionment_divisor(&pops, 20, rule);
            let total: u32 = result.iter().map(|(_, s)| s).sum();
            assert_eq!(total, 20, "Total seats must be 20 for rule {:?}", rule);
        }
    }

    // ── every state gets >= 1 seat ────────────────────────────────────────────

    #[test]
    fn minimum_one_seat_per_state_all_methods() {
        // Wyoming-style: one tiny state vs a huge one.
        let pops = make_pops(&[("Giant", 40_000_000), ("Tiny", 500_000)]);
        for rule in [
            RoundingRule::HuntingtonHill,
            RoundingRule::Webster,
            RoundingRule::Adams,
            RoundingRule::Jefferson,
        ] {
            let result = apportionment_divisor(&pops, 10, rule);
            for (name, seats) in &result {
                assert!(*seats >= 1, "State {} got 0 seats under {:?}", name, rule);
            }
        }
    }
}
