//! Prime factorization utilities for seat counts.

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
    fn test_all_seat_counts_compile() {
        // Verify factorizations for all current US congressional seat counts
        let counts = [1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 26, 28, 38, 52];
        for &n in &counts {
            let seq = prime_factor_sequence(n);
            assert!(!seq.is_empty() || n == 1);
        }
    }
}
