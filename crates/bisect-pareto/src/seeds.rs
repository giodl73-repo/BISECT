//! SHA-256 domain-separated seed derivation for NSGA-II.
//!
//! All stochastic operations derive their seeds from `base_seed` via SHA-256
//! with domain-separated prefixes. This ensures full reproducibility.
//!
//! Per spec §6.

use ropt_core::{derive_seed, SeedPart};

/// Derive an initialisation seed for plan `i`.
///
/// SHA-256("PARETO_INIT_" || i:u32le || "_" || base_seed:u64le) → first 8 bytes as u64le
/// Input: 12 + 4 + 1 + 8 = 25 bytes total
pub fn init_seed(base_seed: u64, i: u32) -> u64 {
    derive_seed(
        b"PARETO_INIT_",
        &[SeedPart::U32(i), SeedPart::U64(base_seed)],
    )
    .expect("non-empty seed domain")
}

/// Derive a crossover seed for generation `gen`, plan pair index `i`.
///
/// SHA-256("PARETO_CROSS_" || gen:u32le || "_" || i:u32le || "_" || base_seed:u64le)
/// → first 8 bytes as u64le
/// Input: 13 + 4 + 1 + 4 + 1 + 8 = 31 bytes total
pub fn cross_seed(base_seed: u64, gen: u32, i: u32) -> u64 {
    derive_seed(
        b"PARETO_CROSS_",
        &[
            SeedPart::U32(gen),
            SeedPart::U32(i),
            SeedPart::U64(base_seed),
        ],
    )
    .expect("non-empty seed domain")
}

/// Derive a mutation seed for generation `gen`, offspring index `i`.
///
/// SHA-256("PARETO_MUT_" || gen:u32le || "_" || i:u32le || "_" || base_seed:u64le)
/// → first 8 bytes as u64le
/// Input: 11 + 4 + 1 + 4 + 1 + 8 = 29 bytes total
pub fn mut_seed(base_seed: u64, gen: u32, i: u32) -> u64 {
    derive_seed(
        b"PARETO_MUT_",
        &[
            SeedPart::U32(gen),
            SeedPart::U32(i),
            SeedPart::U64(base_seed),
        ],
    )
    .expect("non-empty seed domain")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_seeds_distinct() {
        let s = 42u64;
        assert_ne!(
            init_seed(s, 0),
            init_seed(s, 1),
            "init_seed(s,0) != init_seed(s,1)"
        );
        assert_ne!(
            init_seed(s, 0),
            init_seed(s, 2),
            "init_seed(s,0) != init_seed(s,2)"
        );
        assert_ne!(
            init_seed(s, 1),
            init_seed(s, 2),
            "init_seed(s,1) != init_seed(s,2)"
        );
    }

    #[test]
    fn cross_mut_seeds_distinct() {
        let s = 42u64;
        let gen = 0u32;
        let i = 0u32;
        assert_ne!(
            cross_seed(s, gen, i),
            mut_seed(s, gen, i),
            "cross_seed != mut_seed for same inputs"
        );
    }

    #[test]
    fn seed_prefixes_distinct() {
        // PARETO_INIT_ (12 bytes) != PARETO_CROSS_ (13 bytes) != PARETO_MUT_ (11 bytes)
        // Test that seeds derived from identical gen/i/base values differ across types
        let s = 0u64;
        let a = init_seed(s, 0);
        let b = cross_seed(s, 0, 0);
        let c = mut_seed(s, 0, 0);
        assert_ne!(a, b, "PARETO_INIT_ != PARETO_CROSS_");
        assert_ne!(a, c, "PARETO_INIT_ != PARETO_MUT_");
        assert_ne!(b, c, "PARETO_CROSS_ != PARETO_MUT_");
    }

    #[test]
    fn init_seed_deterministic() {
        let s = 99u64;
        assert_eq!(
            init_seed(s, 5),
            init_seed(s, 5),
            "same inputs → same output"
        );
    }

    #[test]
    fn cross_seed_deterministic() {
        let s = 99u64;
        assert_eq!(cross_seed(s, 3, 7), cross_seed(s, 3, 7));
    }

    #[test]
    fn mut_seed_deterministic() {
        let s = 99u64;
        assert_eq!(mut_seed(s, 3, 7), mut_seed(s, 3, 7));
    }

    #[test]
    fn cross_seeds_distinct_across_gen() {
        let s = 42u64;
        let i = 0u32;
        assert_ne!(
            cross_seed(s, 0, i),
            cross_seed(s, 1, i),
            "different gen → different cross_seed"
        );
    }

    #[test]
    fn mut_seeds_distinct_across_gen() {
        let s = 42u64;
        let i = 0u32;
        assert_ne!(
            mut_seed(s, 0, i),
            mut_seed(s, 1, i),
            "different gen → different mut_seed"
        );
    }

    #[test]
    fn different_base_seeds_give_different_outputs() {
        assert_ne!(
            init_seed(0, 0),
            init_seed(1, 0),
            "different base_seed → different init_seed"
        );
    }
}
