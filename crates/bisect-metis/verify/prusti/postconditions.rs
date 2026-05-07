//! Prusti postcondition documentation and activation file.
//!
//! These annotations are the three legally-cited properties from the
//! redist-metis deposition quote. Activate with:
//!   cargo prusti -p redist-metis
//!
//! Prusti version: 0.2.x (Viper backend, ETH Zurich)
//! See verify/prusti/GAPS.md for functions that cannot be verified.
//! See verify/prusti/artifacts/ for committed .vpr proof files.

// The three postconditions live in src/api.rs on `Partitioner::split`.
// This file documents them and serves as the legal reference.

/// Postcondition 1: Full coverage
/// Every vertex in the input graph appears in exactly one district.
///
/// ```text
/// #[ensures(result.is_ok() ==>
///     result.as_ref().unwrap().assignment.len() == g.n())]
/// ```
///
/// Legal meaning: No census tract is omitted from any district.
pub const POSTCONDITION_1_COVERAGE: &str = "
Every vertex assigned to exactly one district.
Activated in Partitioner::split postconditions.
";

/// Postcondition 2: Valid part IDs
/// All part IDs are in [0, k).
///
/// ```text
/// #[ensures(result.is_ok() ==>
///     forall(|i: usize| i < result.as_ref().unwrap().assignment.len()
///         ==> result.as_ref().unwrap().assignment[i] < k))]
/// ```
///
/// Legal meaning: No phantom or out-of-range districts.
pub const POSTCONDITION_2_VALIDITY: &str = "
All part IDs are valid (in [0, k)).
Activated in Partitioner::split postconditions.
";

/// Postcondition 3: Population balance ≤ ε
/// ε = ceil(total_pop × 0.005) = (total_pop × 5 + 999) / 1000
///
/// ```text
/// #[ensures(result.is_ok() ==>
///     population_balance(result.as_ref().unwrap(), g) <= epsilon(g))]
/// ```
///
/// Legal meaning: One-person-one-vote compliance.
/// This uses INTEGER arithmetic only — no float — preserving determinism.
pub const POSTCONDITION_3_BALANCE: &str = "
Population balance ≤ 0.5% (one-person-one-vote).
Uses integer arithmetic: epsilon = (total_pop * 5 + 999) / 1000.
Activated in Partitioner::split postconditions.
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postconditions_documented() {
        assert!(!POSTCONDITION_1_COVERAGE.is_empty());
        assert!(!POSTCONDITION_2_VALIDITY.is_empty());
        assert!(!POSTCONDITION_3_BALANCE.is_empty());
    }
}
