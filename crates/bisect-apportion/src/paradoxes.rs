//! Apportionment paradox detection.
//!
//! The **Alabama Paradox** occurs when a state *loses* a seat as the total
//! house size increases by one.  It was first observed in 1880 when Alabama
//! was allocated 8 seats in a 299-seat House but only 7 seats in a 300-seat
//! House under the Hamilton/Largest-Remainder method.
//!
//! Divisor methods (Webster, Adams, Jefferson, Huntington-Hill) are provably
//! *paradox-free* — they cannot exhibit the Alabama Paradox.  This module
//! lets callers verify that claim empirically for any population dataset and
//! any divisor rule.

use crate::divisor_methods::{apportionment_divisor, RoundingRule};
use std::collections::HashMap;

/// Check whether any state loses a seat when the house size increases.
///
/// Iterates over consecutive pairs in `house_sizes` (which need not be
/// contiguous; any monotonically increasing sequence works).  For each
/// adjacent pair `(h, h+δ)`, runs both apportionments and records every
/// state that received fewer seats at the larger house size.
///
/// # Arguments
/// * `populations`  — `(state_name, population)` pairs.
/// * `house_sizes`  — slice of house sizes to test, in ascending order.
/// * `rule`         — the divisor rounding rule to apply.
///
/// # Returns
/// A `Vec<(state_name, house_size_where_loss_occurs, previous_house_size)>`.
/// An empty vector means no paradox was detected across the given range.
///
/// # Panics
/// Panics if any value in `house_sizes` is less than `populations.len()`.
pub fn check_alabama_paradox(
    populations: &[(String, u64)],
    house_sizes: &[u32],
    rule: RoundingRule,
) -> Vec<(String, u32, u32)> {
    if house_sizes.len() < 2 {
        return Vec::new();
    }

    let mut paradox_events: Vec<(String, u32, u32)> = Vec::new();

    for window in house_sizes.windows(2) {
        let h_before = window[0];
        let h_after = window[1];

        let seats_before = apportionment_divisor(populations, h_before, rule);
        let seats_after = apportionment_divisor(populations, h_after, rule);

        // Build lookup maps keyed by state name.
        let before_map: HashMap<&str, u32> =
            seats_before.iter().map(|(n, s)| (n.as_str(), *s)).collect();
        let after_map: HashMap<&str, u32> =
            seats_after.iter().map(|(n, s)| (n.as_str(), *s)).collect();

        for (name, _) in populations {
            let b = before_map.get(name.as_str()).copied().unwrap_or(0);
            let a = after_map.get(name.as_str()).copied().unwrap_or(0);
            if a < b {
                paradox_events.push((name.clone(), h_after, h_before));
            }
        }
    }

    paradox_events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::divisor_methods::make_pops;

    // ── divisor methods are paradox-free ──────────────────────────────────────
    //
    // A theoretical guarantee: no divisor method can exhibit the Alabama Paradox.
    // Test this empirically over a realistic-ish population set and a wide range
    // of house sizes.

    fn run_no_paradox(rule: RoundingRule, label: &str) {
        let pops = make_pops(&[
            ("A", 6_000_000),
            ("B", 3_000_000),
            ("C", 1_500_000),
            ("D", 800_000),
            ("E", 200_000),
        ]);
        let house_sizes: Vec<u32> = (5..=50).collect();
        let events = check_alabama_paradox(&pops, &house_sizes, rule);
        assert!(
            events.is_empty(),
            "{} paradox events found under {} (should be zero for a divisor method): {:?}",
            events.len(),
            label,
            events
        );
    }

    #[test]
    fn no_alabama_paradox_huntington_hill() {
        run_no_paradox(RoundingRule::HuntingtonHill, "HuntingtonHill");
    }

    #[test]
    fn no_alabama_paradox_webster() {
        run_no_paradox(RoundingRule::Webster, "Webster");
    }

    #[test]
    fn no_alabama_paradox_adams() {
        run_no_paradox(RoundingRule::Adams, "Adams");
    }

    #[test]
    fn no_alabama_paradox_jefferson() {
        run_no_paradox(RoundingRule::Jefferson, "Jefferson");
    }

    // ── empty / trivial inputs ────────────────────────────────────────────────

    #[test]
    fn single_house_size_returns_empty() {
        let pops = make_pops(&[("A", 1_000_000), ("B", 500_000)]);
        let events = check_alabama_paradox(&pops, &[10], RoundingRule::Webster);
        assert!(
            events.is_empty(),
            "Single house size: no comparison possible"
        );
    }

    #[test]
    fn empty_house_sizes_returns_empty() {
        let pops = make_pops(&[("A", 1_000_000)]);
        let events = check_alabama_paradox(&pops, &[], RoundingRule::Adams);
        assert!(events.is_empty(), "Empty house_sizes: nothing to compare");
    }

    // ── monotone non-decrease: every state either holds or gains seats ────────

    #[test]
    fn seat_counts_never_decrease_for_any_divisor_method() {
        // A property test: for each state, seats[h] <= seats[h+1] for all h.
        let pops = make_pops(&[("X", 10_000_000), ("Y", 5_000_000), ("Z", 1_000_000)]);
        let house_sizes: Vec<u32> = (3..=30).collect();

        for rule in [
            RoundingRule::HuntingtonHill,
            RoundingRule::Webster,
            RoundingRule::Adams,
            RoundingRule::Jefferson,
        ] {
            // Build a time-series of seat allocations for each state.
            let series: Vec<HashMap<String, u32>> = house_sizes
                .iter()
                .map(|&h| {
                    apportionment_divisor(&pops, h, rule)
                        .into_iter()
                        .collect::<HashMap<_, _>>()
                })
                .collect();

            for (name, _) in &pops {
                let seat_vec: Vec<u32> = series.iter().map(|m| m[name]).collect();
                for window in seat_vec.windows(2) {
                    assert!(
                        window[1] >= window[0],
                        "State {} lost a seat under {:?}: {} -> {}",
                        name,
                        rule,
                        window[0],
                        window[1]
                    );
                }
            }
        }
    }
}
