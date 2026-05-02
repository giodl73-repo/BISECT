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
/// See module-level doc for the three-case rule.
pub fn split_prescription(k: u32) -> SplitStep {
    assert!(k >= 1, "k must be >= 1");
    if k <= MAX_DIRECT_SPLIT {
        // k=1 (trivial), k=2, k=3: direct k-cut (sub_k=1 for leaves)
        return SplitStep::Uniform { parts: k, sub_k: 1 };
    }
    if is_prime(k) {
        // Large prime: binary fallback
        let k_left = k / 2;
        SplitStep::Binary { k_left, k_right: k - k_left }
    } else {
        // Composite: split by smallest prime factor
        let p = smallest_prime_factor(k);
        SplitStep::Uniform { parts: p, sub_k: k / p }
    }
}

fn smallest_prime_factor(mut n: u32) -> u32 {
    if n % 2 == 0 { return 2; }
    let mut d = 3u32;
    while d * d <= n { if n % d == 0 { return d; } d += 2; }
    n
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
        // k=4=2²: p=2, sub_k=2
        assert_eq!(split_prescription(4), SplitStep::Uniform { parts: 2, sub_k: 2 });
        // k=6=2×3: p=2, sub_k=3
        assert_eq!(split_prescription(6), SplitStep::Uniform { parts: 2, sub_k: 3 });
        // k=9=3²: p=3, sub_k=3
        assert_eq!(split_prescription(9), SplitStep::Uniform { parts: 3, sub_k: 3 });
        // k=52=4×13: p=2, sub_k=26
        assert_eq!(split_prescription(52), SplitStep::Uniform { parts: 2, sub_k: 26 });
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
        // Trace k=17 → (8,9) → 8=2³ (three binary levels), 9=3² (two ternary levels)
        let s17 = split_prescription(17);
        assert_eq!(s17, SplitStep::Binary { k_left: 8, k_right: 9 });

        let s8 = split_prescription(8);
        assert_eq!(s8, SplitStep::Uniform { parts: 2, sub_k: 4 }); // 8=2×4
        let s4 = split_prescription(4);
        assert_eq!(s4, SplitStep::Uniform { parts: 2, sub_k: 2 }); // 4=2×2
        let s2 = split_prescription(2);
        assert_eq!(s2, SplitStep::Uniform { parts: 2, sub_k: 1 }); // leaf

        let s9 = split_prescription(9);
        assert_eq!(s9, SplitStep::Uniform { parts: 3, sub_k: 3 }); // 9=3×3
        let s3 = split_prescription(3);
        assert_eq!(s3, SplitStep::Uniform { parts: 3, sub_k: 1 }); // leaf
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
}
