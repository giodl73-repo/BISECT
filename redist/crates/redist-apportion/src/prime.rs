//! Prime factorization utilities and split prescriptions for seat counts.
//!
//! Key function: `split_prescription(k)` — given a target district count k,
//! returns how to split a region of k districts into sub-regions.
//!
//! Rule (MAX_DIRECT_SPLIT = 3):
//!   k ≤ 3          → direct k-cut (2-way or 3-way)
//!   k composite    → split by smallest prime factor p: p equal groups of k/p
//!   k prime > 3    → binary floor/ceil: groups of floor(k/2) and ceil(k/2)
//!
//! This avoids impractical large-prime k-cuts (e.g. k=17 → direct 17-cut)
//! while preserving full reuse: all regions with the same k always receive
//! the same prescription, so their splits are structurally identical.

/// Maximum k for which a direct k-cut is performed.
/// k=2 and k=3 are well-supported by METIS; k≥4 prime falls back to binary.
pub const MAX_DIRECT_SPLIT: u32 = 3;

/// How to split a region containing `k` target districts.
#[derive(Debug, Clone, PartialEq)]
pub enum SplitStep {
    /// Split into `parts` equal sub-regions, each with `sub_k` targets.
    /// Used for composite k (split by smallest prime factor) and k ≤ MAX_DIRECT_SPLIT.
    Uniform { parts: u32, sub_k: u32 },
    /// Binary split for large-prime k: two sub-regions of `k_left` and `k_right`.
    /// Target population weights are k_left/k and k_right/k (not 50/50).
    Binary { k_left: u32, k_right: u32 },
}

impl SplitStep {
    /// Number of sub-regions this step produces.
    pub fn parts(&self) -> u32 {
        match self { Self::Uniform { parts, .. } => *parts, Self::Binary { .. } => 2 }
    }

    /// Target district count for sub-region i (0-based).
    pub fn sub_k(&self, i: usize) -> u32 {
        match self {
            Self::Uniform  { sub_k, .. } => *sub_k,
            Self::Binary   { k_left, k_right } => if i == 0 { *k_left } else { *k_right },
        }
    }

    /// Target population fraction for sub-region i (used for METIS tpwgts).
    pub fn fraction(&self, i: usize, total_k: u32) -> f32 {
        self.sub_k(i) as f32 / total_k as f32
    }
}

/// Returns the split prescription for a region with `k` target districts.
///
/// Rule (MAX_DIRECT_SPLIT = 3):
///   k ≤ 3          → direct k-cut
///   k composite    → split by **largest** prime factor p first: p sub-regions of k/p
///   k prime > 3    → binary floor/ceil fallback
///
/// Largest-prime-first means that k=34 and k=51 both produce 17 top-level
/// sub-regions of the same geographic region, enabling reuse of that shared
/// 17-district partition. The smaller factor (2 or 3) is then applied inside
/// each of the 17 sub-regions at the next level.
pub fn split_prescription(k: u32) -> SplitStep {
    assert!(k >= 1, "k must be >= 1");
    if k <= MAX_DIRECT_SPLIT {
        return SplitStep::Uniform { parts: k, sub_k: 1 };
    }
    if is_prime(k) {
        let k_left = k / 2;
        SplitStep::Binary { k_left, k_right: k - k_left }
    } else {
        // Composite: split by LARGEST prime factor first to maximise reuse.
        // k=34=2×17 → parts=17, sub_k=2 (same 17-map as k=51=3×17)
        // k=51=3×17 → parts=17, sub_k=3
        // k=52=4×13 → parts=13, sub_k=4 (same 13-map as k=26=2×13)
        let p = largest_prime_factor(k);
        SplitStep::Uniform { parts: p, sub_k: k / p }
    }
}

fn smallest_prime_factor(mut n: u32) -> u32 {
    if n % 2 == 0 { return 2; }
    let mut d = 3u32;
    while d * d <= n { if n % d == 0 { return d; } d += 2; }
    n
}

fn largest_prime_factor(n: u32) -> u32 {
    let mut n = n;
    let mut largest = 1u32;
    let mut d = 2u32;
    while d * d <= n {
        while n % d == 0 { largest = d; n /= d; }
        d += 1;
    }
    if n > 1 { n } else { largest }
}

/// Returns the canonical factorization sequence of `n`: prime factors in
/// non-decreasing order, with repetition.
///
/// `factor_sequence(1)` = `[]` (no factors).
/// `factor_sequence(8)` = `[2, 2, 2]`.
/// `factor_sequence(12)` = `[2, 2, 3]`.
/// `factor_sequence(51)` = `[3, 17]`.
/// `factor_sequence(52)` = `[2, 2, 13]`.
///
/// The product of the returned slice equals `n` (or 1 for empty slice).
///
/// # Panics
/// Panics if `n == 0`.
pub fn prime_factor_sequence(mut n: u32) -> Vec<u32> {
    assert!(n > 0, "n must be >= 1");
    if n == 1 {
        return vec![];
    }
    let mut factors = Vec::new();
    let mut d = 2u32;
    while d * d <= n {
        while n % d == 0 {
            factors.push(d);
            n /= d;
        }
        d += 1;
    }
    if n > 1 {
        factors.push(n);
    }
    factors
}

/// Length of the common prefix of two factorization sequences.
/// This is the number of PFR tree levels that can be reused when
/// transitioning from a map with `n` seats to one with `m` seats.
pub fn common_prefix_len(n: u32, m: u32) -> usize {
    let fn_ = prime_factor_sequence(n);
    let fm = prime_factor_sequence(m);
    fn_.iter().zip(fm.iter()).take_while(|(a, b)| a == b).count()
}

/// Returns true if `n` is prime.
pub fn is_prime(n: u32) -> bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    let mut d = 3u32;
    while d * d <= n {
        if n % d == 0 { return false; }
        d += 2;
    }
    true
}

/// Actual depth of the PFR factorization tree for seat count `k`.
///
/// This is the maximum path length from the root to any leaf, following
/// the `split_prescription` rules (largest-prime-first, binary fallback
/// for prime > MAX_DIRECT_SPLIT). Used to compute per-level METIS balance
/// tolerance such that cumulative error stays within the statutory limit.
///
/// Examples: k=8→3, k=17→4, k=38→5, k=52→3.
pub fn pfr_tree_depth(k: u32) -> u32 {
    if k <= 1 { return 0; }
    let step = split_prescription(k);
    let max_sub = (0..step.parts())
        .map(|i| pfr_tree_depth(step.sub_k(i as usize)))
        .max()
        .unwrap_or(0);
    1 + max_sub
}

/// The "disruption level" of an `n → n+1` reapportionment transition:
/// the depth at which the factorization trees first diverge (1-based).
/// Returns 0 if both sequences are empty (n=1 case).
pub fn first_divergence_level(n: u32, np1: u32) -> usize {
    common_prefix_len(n, np1) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factor_sequence_one() {
        assert_eq!(prime_factor_sequence(1), Vec::<u32>::new());
    }

    #[test]
    fn test_factor_sequence_prime() {
        assert_eq!(prime_factor_sequence(17), vec![17]);
        assert_eq!(prime_factor_sequence(53), vec![53]);
        assert_eq!(prime_factor_sequence(2), vec![2]);
    }

    #[test]
    fn test_factor_sequence_powers_of_2() {
        assert_eq!(prime_factor_sequence(8), vec![2, 2, 2]);
        assert_eq!(prime_factor_sequence(4), vec![2, 2]);
        assert_eq!(prime_factor_sequence(2), vec![2]);
    }

    #[test]
    fn test_factor_sequence_composite() {
        assert_eq!(prime_factor_sequence(12), vec![2, 2, 3]);
        assert_eq!(prime_factor_sequence(51), vec![3, 17]);
        assert_eq!(prime_factor_sequence(52), vec![2, 2, 13]);
        assert_eq!(prime_factor_sequence(6), vec![2, 3]);
        assert_eq!(prime_factor_sequence(9), vec![3, 3]);
    }

    #[test]
    fn test_product_invariant() {
        for n in 1u32..=100 {
            let seq = prime_factor_sequence(n);
            let product: u32 = seq.iter().product::<u32>().max(1);
            assert_eq!(product, n, "product of factors should equal n={n}");
        }
    }

    #[test]
    fn test_common_prefix() {
        // 51=[3,17] and 52=[2,2,13]: no common prefix
        assert_eq!(common_prefix_len(51, 52), 0);
        // 6=[2,3] and 8=[2,2,2]: prefix [2]
        assert_eq!(common_prefix_len(6, 8), 1);
        // 4=[2,2] and 8=[2,2,2]: prefix [2,2]
        assert_eq!(common_prefix_len(4, 8), 2);
    }

    #[test]
    fn test_is_prime() {
        assert!(is_prime(2));
        assert!(is_prime(17));
        assert!(is_prime(53));
        assert!(!is_prime(1));
        assert!(!is_prime(52));
        assert!(!is_prime(51));
    }

    #[test]
    fn test_split_prescription_small() {
        // k ≤ 3: direct uniform splits
        assert_eq!(split_prescription(1), SplitStep::Uniform { parts: 1, sub_k: 1 });
        assert_eq!(split_prescription(2), SplitStep::Uniform { parts: 2, sub_k: 1 });
        assert_eq!(split_prescription(3), SplitStep::Uniform { parts: 3, sub_k: 1 });
    }

    #[test]
    fn test_split_prescription_composite() {
        // k=4=2²: largest prime=2, parts=2, sub_k=2
        assert_eq!(split_prescription(4), SplitStep::Uniform { parts: 2, sub_k: 2 });
        // k=6=2×3: largest prime=3, parts=3, sub_k=2
        assert_eq!(split_prescription(6), SplitStep::Uniform { parts: 3, sub_k: 2 });
        // k=9=3²: largest prime=3, parts=3, sub_k=3
        assert_eq!(split_prescription(9), SplitStep::Uniform { parts: 3, sub_k: 3 });
        // k=52=4×13: largest prime=13, parts=13, sub_k=4
        assert_eq!(split_prescription(52), SplitStep::Uniform { parts: 13, sub_k: 4 });
        // k=34=2×17: largest prime=17, parts=17, sub_k=2
        assert_eq!(split_prescription(34), SplitStep::Uniform { parts: 17, sub_k: 2 });
        // k=51=3×17: largest prime=17, parts=17, sub_k=3
        assert_eq!(split_prescription(51), SplitStep::Uniform { parts: 17, sub_k: 3 });
        // k=26=2×13: largest prime=13, parts=13, sub_k=2  (same top level as k=52)
        assert_eq!(split_prescription(26), SplitStep::Uniform { parts: 13, sub_k: 2 });
    }

    #[test]
    fn test_split_prescription_large_prime() {
        // k=17 (prime >3): binary 8+9
        assert_eq!(split_prescription(17), SplitStep::Binary { k_left: 8, k_right: 9 });
        // k=11: binary 5+6
        assert_eq!(split_prescription(11), SplitStep::Binary { k_left: 5, k_right: 6 });
        // k=13: binary 6+7
        assert_eq!(split_prescription(13), SplitStep::Binary { k_left: 6, k_right: 7 });
        // k=5 (prime >3): binary 2+3
        assert_eq!(split_prescription(5), SplitStep::Binary { k_left: 2, k_right: 3 });
    }

    #[test]
    fn test_split_prescription_trace_17() {
        // k=17 (prime): binary (8, 9) — unchanged
        assert_eq!(split_prescription(17), SplitStep::Binary { k_left: 8, k_right: 9 });
        // k=34=2×17: parts=17 (largest prime), sub_k=2 — REUSES k=17 map
        assert_eq!(split_prescription(34), SplitStep::Uniform { parts: 17, sub_k: 2 });
        // k=51=3×17: parts=17 (largest prime), sub_k=3 — SAME k=17 map
        assert_eq!(split_prescription(51), SplitStep::Uniform { parts: 17, sub_k: 3 });
    }

    #[test]
    fn test_shared_base_map() {
        // Both k=34 and k=51 start with a 17-region top-level split.
        // Both k=26 and k=52 start with a 13-region top-level split.
        assert_eq!(split_prescription(34).parts(), 17);
        assert_eq!(split_prescription(51).parts(), 17);
        assert_eq!(split_prescription(26).parts(), 13);
        assert_eq!(split_prescription(52).parts(), 13);
    }

    #[test]
    fn test_pfr_tree_depth() {
        assert_eq!(pfr_tree_depth(1), 0);
        assert_eq!(pfr_tree_depth(2), 1);  // one 2-cut
        assert_eq!(pfr_tree_depth(3), 1);  // one 3-cut
        assert_eq!(pfr_tree_depth(4), 2);  // 2-cut → 2-cut
        assert_eq!(pfr_tree_depth(8), 3);  // 2→2→2
        assert_eq!(pfr_tree_depth(9), 2);  // 3-cut → 3-cut
        // k=17 (prime): Binary(8,9). depth = 1 + max(depth(8), depth(9)) = 1+max(3,2) = 4
        assert_eq!(pfr_tree_depth(17), 4);
        // k=52=4×13: Uniform{parts:13, sub_k:4}. depth = 1 + depth(4) = 1+2 = 3
        assert_eq!(pfr_tree_depth(52), 3);
        // k=38=2×19: Uniform{parts:19, sub_k:2}. depth = 1 + depth(2) = 1+1 = 2
        // (the 19-way cut is one single METIS call, not 19 recursive levels)
        assert_eq!(pfr_tree_depth(38), 2);
    }

    #[test]
    fn test_all_seat_counts_compile() {
        // Verify factorizations for all current US congressional seat counts
        let counts = [1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 26, 28, 38, 52];
        for &n in &counts {
            let seq = prime_factor_sequence(n);
            assert!(!seq.is_empty() || n == 1);
        }
    }

    // -----------------------------------------------------------------------
    // New prime.rs tests
    // -----------------------------------------------------------------------

    /// SplitStep::parts() returns correct count for both variants.
    #[test]
    fn split_step_parts_both_variants() {
        let u = SplitStep::Uniform { parts: 5, sub_k: 3 };
        assert_eq!(u.parts(), 5);
        let b = SplitStep::Binary { k_left: 7, k_right: 8 };
        assert_eq!(b.parts(), 2);
    }

    /// SplitStep::sub_k(i) returns correct value per index.
    #[test]
    fn split_step_sub_k_both_variants() {
        let u = SplitStep::Uniform { parts: 4, sub_k: 13 };
        assert_eq!(u.sub_k(0), 13);
        assert_eq!(u.sub_k(3), 13); // all slots the same

        let b = SplitStep::Binary { k_left: 6, k_right: 7 };
        assert_eq!(b.sub_k(0), 6);
        assert_eq!(b.sub_k(1), 7);
    }

    /// SplitStep::fraction() sums to 1.0 over all sub-regions.
    #[test]
    fn split_step_fraction_sums_to_one() {
        // Uniform k=12=4×3 → parts=3, sub_k=4 (largest prime first)
        let step = split_prescription(12);
        let total_k = 12u32;
        let frac_sum: f32 = (0..step.parts() as usize)
            .map(|i| step.fraction(i, total_k))
            .sum();
        assert!((frac_sum - 1.0).abs() < 1e-5, "fractions should sum to 1.0, got {frac_sum}");
    }

    /// first_divergence_level for same n returns len+1 (all-common prefix).
    #[test]
    fn first_divergence_same_n() {
        // n=8=[2,2,2], n+1=9=[3,3]: first diverges at level 1 (no common prefix)
        let fdl = first_divergence_level(8, 9);
        assert_eq!(fdl, 1, "8=[2,2,2] and 9=[3,3] diverge at level 1");

        // n=4=[2,2], n+1=5=[5]: diverge at level 1
        let fdl2 = first_divergence_level(4, 5);
        assert_eq!(fdl2, 1);
    }

    /// first_divergence_level for n=7→8 — shares no prefix.
    #[test]
    fn first_divergence_level_seven_to_eight() {
        // 7=[7], 8=[2,2,2]: no common prefix → divergence at level 1
        let fdl = first_divergence_level(7, 8);
        assert_eq!(fdl, 1);
    }

    /// is_prime rejects 0 and 1, accepts 2.
    #[test]
    fn is_prime_edge_cases() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
    }

    /// pfr_tree_depth for k=1 returns 0.
    #[test]
    fn pfr_tree_depth_k1_is_zero() {
        assert_eq!(pfr_tree_depth(1), 0);
    }

    /// split_prescription for k=2: direct Uniform{parts:2, sub_k:1}.
    #[test]
    fn split_prescription_k2_direct_cut() {
        assert_eq!(split_prescription(2), SplitStep::Uniform { parts: 2, sub_k: 1 });
    }

    /// common_prefix_len for identical n: full length.
    #[test]
    fn common_prefix_same_n() {
        // 8=[2,2,2] and 8=[2,2,2]: full prefix of length 3
        assert_eq!(common_prefix_len(8, 8), 3);
        // 12=[2,2,3] and 12=[2,2,3]: length 3
        assert_eq!(common_prefix_len(12, 12), 3);
    }

    /// prime_factor_sequence product property holds for n=1 (empty product = 1).
    #[test]
    fn prime_factor_sequence_one_empty_product() {
        let seq = prime_factor_sequence(1);
        assert!(seq.is_empty(), "prime_factor_sequence(1) should be empty");
        let product: u32 = seq.iter().product::<u32>().max(1);
        assert_eq!(product, 1u32);
    }

    // ── Additional prime.rs tests ─────────────────────────────────────────────

    /// smallest_prime_factor via composite split: k=15=3×5 should split by largest=5.
    #[test]
    fn split_prescription_k15_largest_prime_first() {
        // 15 = 3 × 5; largest prime = 5; so parts=5, sub_k=3
        let step = split_prescription(15);
        assert_eq!(step, SplitStep::Uniform { parts: 5, sub_k: 3 });
    }

    /// k=7 (prime) → binary 3+4.
    #[test]
    fn split_prescription_k7_binary() {
        assert_eq!(split_prescription(7), SplitStep::Binary { k_left: 3, k_right: 4 });
    }

    /// k=19 (prime) → binary 9+10.
    #[test]
    fn split_prescription_k19_binary() {
        assert_eq!(split_prescription(19), SplitStep::Binary { k_left: 9, k_right: 10 });
    }

    /// pfr_tree_depth for k=6=2×3 → Uniform{parts:3, sub_k:2};
    /// depth = 1 + depth(2) = 1 + 1 = 2.
    #[test]
    fn pfr_tree_depth_k6() {
        assert_eq!(pfr_tree_depth(6), 2);
    }

    /// pfr_tree_depth for k=51=3×17 → Uniform{parts:17, sub_k:3};
    /// depth = 1 + depth(3) = 1 + 1 = 2.
    #[test]
    fn pfr_tree_depth_k51() {
        assert_eq!(pfr_tree_depth(51), 2);
    }

    /// pfr_tree_depth for k=34=2×17 → Uniform{parts:17, sub_k:2};
    /// depth = 1 + depth(2) = 1 + 1 = 2.
    #[test]
    fn pfr_tree_depth_k34() {
        assert_eq!(pfr_tree_depth(34), 2);
    }

    /// first_divergence_level: n=6=[2,3], n+1=7=[7]; no common prefix → level 1.
    #[test]
    fn first_divergence_six_to_seven() {
        assert_eq!(first_divergence_level(6, 7), 1);
    }

    /// first_divergence_level: n=2, n+1=3; [2] vs [3] → diverge at level 1.
    #[test]
    fn first_divergence_two_to_three() {
        assert_eq!(first_divergence_level(2, 3), 1);
    }

    /// is_prime: large known primes and non-primes.
    #[test]
    fn is_prime_larger_values() {
        assert!(is_prime(97));
        assert!(is_prime(101));
        assert!(!is_prime(100));
        assert!(!is_prime(99));
    }

    /// common_prefix_len: n=1 and m=1 both have empty sequences; prefix = 0.
    #[test]
    fn common_prefix_one_and_one() {
        assert_eq!(common_prefix_len(1, 1), 0);
    }

    /// prime_factor_sequence for k=30=2×3×5: sorted [2,3,5].
    #[test]
    fn prime_factor_sequence_30() {
        assert_eq!(prime_factor_sequence(30), vec![2, 3, 5]);
    }

    /// SplitStep::fraction for Binary variant sums to ~1.0.
    #[test]
    fn split_step_fraction_binary_sums_to_one() {
        let step = split_prescription(17); // Binary { k_left: 8, k_right: 9 }
        let total_k = 17u32;
        let frac_sum: f32 = (0..2).map(|i| step.fraction(i, total_k)).sum();
        assert!((frac_sum - 1.0).abs() < 1e-5,
            "Binary fractions must sum to 1.0, got {frac_sum}");
    }
}
